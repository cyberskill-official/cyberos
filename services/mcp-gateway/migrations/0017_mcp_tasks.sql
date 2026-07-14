-- TASK-MCP-007 Migration 0017: mcp_tasks + closed enums
-- DEC-1101 (task_status), DEC-1114 (task_progress_unit), DEC-1104 (UUID handle), DEC-1112 (result cap),
-- DEC-1121 (idempotency), DEC-1125 (PII scrub).
--
-- Persistence for the long-running tasks primitive that ships in-memory in
-- services/mcp-gateway/src/tasks. The gateway keeps the in-memory store as the no-database path; the
-- write-through wiring lands with the code commit (and the long_running annotation + tools/call async
-- routing that actually creates tasks in the request path, currently deferred). The per-task checkpoint
-- and progress-event tables (FR §3.1 0010/0011) are deferred with the worker pool + NATS progress.
--
-- RLS note: same decision as 0016 - append-only GRANT model, not GUC-based CREATE POLICY, because the
-- gateway does not set a per-connection `auth.tenant_id`. Tenant/caller isolation is enforced in the
-- handler.

DO $$ BEGIN CREATE ROLE mcp_task_writer NOLOGIN; EXCEPTION WHEN duplicate_object THEN NULL; END $$;

CREATE TYPE task_status AS ENUM (
    'pending',
    'running',
    'completed',
    'failed',
    'cancelled',
    'expired'
);

CREATE TYPE task_progress_unit AS ENUM ('percent', 'items', 'bytes', 'none');

CREATE TABLE mcp_tasks (
    task_id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id                 UUID NOT NULL REFERENCES tenants(id),
    caller_subject_id         UUID NOT NULL REFERENCES subjects(id),
    tool_id                   TEXT NOT NULL,
    status                    task_status NOT NULL DEFAULT 'pending',
    input_payload_kms_blob    BYTEA NOT NULL,                    -- KMS-encrypted tool input
    input_payload_sha256      CHAR(64) NOT NULL,                 -- the only input form in the chain
    result_payload_kms_blob   BYTEA,                             -- filled on completion (<= 10 MiB)
    result_url                TEXT,                              -- set instead, for oversized results
    error_code                TEXT,
    error_message             TEXT,
    error_details             JSONB,
    progress_value            DOUBLE PRECISION,
    progress_unit             task_progress_unit NOT NULL DEFAULT 'none',
    progress_total            DOUBLE PRECISION,
    idempotency_key           TEXT,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at                TIMESTAMPTZ,
    completed_at              TIMESTAMPTZ,
    expires_at                TIMESTAMPTZ NOT NULL,
    trace_id                  CHAR(32)
);

-- Idempotency (DEC-1121): one live task per (tenant, key). The FR's "within 24h" predicate cannot live
-- in an index (now() is not immutable), so the 24h dedup window is enforced in the handler; the index
-- guarantees uniqueness while a key is in use.
CREATE UNIQUE INDEX idx_tasks_idempotency
    ON mcp_tasks (tenant_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;
CREATE INDEX idx_tasks_caller ON mcp_tasks (caller_subject_id, status, created_at DESC);
CREATE INDEX idx_tasks_expiry ON mcp_tasks (expires_at) WHERE status IN ('pending', 'running');

-- Append-only grant structure (mirrors 0013-0016).
REVOKE UPDATE, DELETE ON mcp_tasks FROM cyberos_app;
GRANT INSERT, SELECT ON mcp_tasks TO mcp_task_writer;
GRANT UPDATE(status, result_payload_kms_blob, result_url, error_code, error_message, error_details, progress_value, progress_unit, progress_total, started_at, completed_at)
    ON mcp_tasks TO mcp_task_writer;
