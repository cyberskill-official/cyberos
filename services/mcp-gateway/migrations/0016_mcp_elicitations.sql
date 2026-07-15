-- TASK-MCP-008 Migration 0016: mcp_elicitations + closed enums
-- DEC-1141 (elicitation_type), DEC-1144 (pending TTL row), DEC-1150 (retry cap), DEC-1157 (PII scrub).
--
-- Persistence for the elicitation primitive that ships in-memory in services/mcp-gateway/src/elicitation.
-- The gateway keeps the in-memory store as the no-database path; when a pool is configured the store
-- write-throughs to this table (the sqlx wiring lands in the follow-on code commit, which also threads the
-- caller's tenant_id / subject from the verified access-token claims into elicitation creation).
--
-- RLS note: like the OAuth tables (0013-0015) this uses the append-only GRANT model rather than
-- CREATE POLICY. The gateway does not set a per-connection `auth.tenant_id` GUC, so enabling RLS
-- policies here would deny every query. Cross-tenant / cross-caller isolation (DEC-1159) is enforced in
-- the handler (caller_subject_id check); GUC-based RLS is deferred until the gateway adopts the
-- per-connection GUC pattern that auth/memory use.

DO $$ BEGIN CREATE ROLE mcp_elicitation_writer NOLOGIN; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

CREATE TYPE elicitation_type AS ENUM (
    'string_input',
    'single_choice',
    'multi_choice',
    'confirmation',
    'file_upload'
);

CREATE TYPE elicitation_status AS ENUM (
    'pending',
    'responded',
    'expired',
    'cancelled',
    'validation_failed'
);

CREATE TABLE mcp_elicitations (
    elicitation_id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id                 UUID NOT NULL REFERENCES tenants(id),
    task_id                   UUID,                                  -- NULL for sync-tool elicitations
    caller_subject_id         UUID NOT NULL REFERENCES subjects(id),
    tool_id                   TEXT NOT NULL,
    elicitation_type          elicitation_type NOT NULL,
    status                    elicitation_status NOT NULL DEFAULT 'pending',
    prompt                    JSONB NOT NULL,
    response_schema           JSONB NOT NULL,
    choices                   JSONB NOT NULL DEFAULT '[]'::jsonb,    -- allowed values for the choice types ([] otherwise); lets a persisted elicitation be re-validated on respond
    response_payload_kms_blob BYTEA,                                 -- filled on respond; KMS-encrypted
    response_payload_sha256   CHAR(64),                              -- the only payload form in the chain
    confirmed                 BOOLEAN,                               -- set on respond for confirmation types; the TASK-MCP-006 gate reads it without opening the sealed blob
    validation_errors         JSONB,
    retry_count               INT NOT NULL DEFAULT 0 CHECK (retry_count BETWEEN 0 AND 3),
    timeout_seconds           INT NOT NULL CHECK (timeout_seconds BETWEEN 1 AND 1800),
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    responded_at              TIMESTAMPTZ,
    expires_at                TIMESTAMPTZ NOT NULL,
    trace_id                  CHAR(32)
);

CREATE INDEX idx_elicit_caller_pending
    ON mcp_elicitations (caller_subject_id, status, created_at DESC)
    WHERE status = 'pending';
CREATE INDEX idx_elicit_task ON mcp_elicitations (task_id) WHERE task_id IS NOT NULL;
CREATE INDEX idx_elicit_expiry ON mcp_elicitations (expires_at) WHERE status = 'pending';

-- Append-only grant structure (mirrors 0013-0015). Status + response columns are the only mutable set;
-- rows are never updated wholesale or deleted by the app (30-day pruning is a deferred cyberos_pruner job).
REVOKE UPDATE, DELETE ON mcp_elicitations FROM cyberos_app;
GRANT INSERT, SELECT ON mcp_elicitations TO mcp_elicitation_writer;
GRANT UPDATE(status, response_payload_kms_blob, response_payload_sha256, confirmed, validation_errors, retry_count, responded_at)
    ON mcp_elicitations TO mcp_elicitation_writer;
