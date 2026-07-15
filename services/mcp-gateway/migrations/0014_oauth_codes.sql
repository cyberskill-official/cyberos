-- TASK-MCP-004 Migration 0011: oauth_codes + oauth_code_state enum
-- DEC-811 (30s TTL), DEC-812 (one-time-use), §1.16 (enum), §1.27 (schema)

CREATE TYPE oauth_code_state AS ENUM ('active', 'consumed', 'expired');

CREATE TABLE oauth_codes (
    code                  TEXT PRIMARY KEY CHECK (length(code) = 43),  -- 256-bit base64url-no-pad
    client_id             UUID NOT NULL REFERENCES oauth_clients(id),
    subject_id            UUID NOT NULL REFERENCES subjects(id),
    tenant_id             UUID NOT NULL REFERENCES tenants(id),
    redirect_uri          TEXT NOT NULL,
    code_challenge        TEXT NOT NULL CHECK (length(code_challenge) BETWEEN 43 AND 128),
    code_challenge_method TEXT NOT NULL CHECK (code_challenge_method = 'S256'),
    scope                 TEXT NOT NULL,
    audience              TEXT NOT NULL,  -- target MCP resource server URL
    nonce                 TEXT NOT NULL,
    state                 TEXT NOT NULL,  -- client-supplied CSRF state
    issued_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at            TIMESTAMPTZ NOT NULL,  -- issued_at + INTERVAL '30 seconds'
    consumed_at           TIMESTAMPTZ,
    memory_chain_hash     CHAR(64) NOT NULL
);

CREATE INDEX oauth_codes_expires ON oauth_codes (expires_at);

-- Append-only: only oauth_code_consumer may set consumed_at
REVOKE UPDATE, DELETE ON oauth_codes FROM cyberos_app;
GRANT INSERT, SELECT ON oauth_codes TO oauth_code_consumer;
GRANT UPDATE(consumed_at) ON oauth_codes TO oauth_code_consumer;
