-- TASK-MCP-004 Migration 0013: oauth_clients + closed enums
-- DEC-803, DEC-807, DEC-808, DEC-820

-- Least-privilege roles the OAuth grants below (and in 0014/0015) target. Created here, in the first
-- OAuth migration, so every later GRANT finds them. Idempotent DO blocks (mirrors auth/0004_rls_roles)
-- so re-applying against a populated database succeeds silently. cyberos_app is created by the auth
-- migrations, which run before these in deploy order.
DO $$ BEGIN CREATE ROLE oauth_writer NOLOGIN;        EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN CREATE ROLE oauth_reader NOLOGIN;        EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN CREATE ROLE oauth_code_consumer NOLOGIN; EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN CREATE ROLE oauth_refresh_writer NOLOGIN;EXCEPTION WHEN duplicate_object THEN NULL; END $$;

CREATE TYPE client_type AS ENUM ('public', 'confidential');
CREATE TYPE oauth_grant_type AS ENUM ('authorization_code', 'refresh_token');
CREATE TYPE oauth_error_code AS ENUM (
    'invalid_request',
    'invalid_client',
    'invalid_grant',
    'unauthorized_client',
    'unsupported_grant_type',
    'invalid_scope'
);

CREATE TABLE oauth_clients (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id            UUID REFERENCES tenants(id),   -- NULL for public CLI clients
    client_type          client_type NOT NULL,
    client_secret_hash   TEXT,                          -- NULL for public; Argon2 for confidential
    redirect_uris        JSONB NOT NULL,
    client_name          TEXT CHECK (client_name IS NULL OR length(client_name) <= 64),
    scope                TEXT NOT NULL CHECK (length(scope) BETWEEN 1 AND 1024),
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at           TIMESTAMPTZ,
    CONSTRAINT confidential_has_secret CHECK (
        (client_type = 'confidential' AND client_secret_hash IS NOT NULL)
     OR (client_type = 'public' AND client_secret_hash IS NULL)
    ),
    CONSTRAINT redirect_uris_max_5 CHECK (jsonb_array_length(redirect_uris) BETWEEN 1 AND 5),
    CONSTRAINT confidential_has_tenant CHECK (
        client_type = 'public' OR tenant_id IS NOT NULL
    )
);

CREATE INDEX oauth_clients_tenant ON oauth_clients (tenant_id) WHERE revoked_at IS NULL;

-- Append-only grant structure (TASK-MCP-004 §1.28 + §3.1)
REVOKE UPDATE, DELETE ON oauth_clients FROM cyberos_app;
GRANT INSERT, SELECT, UPDATE(revoked_at) ON oauth_clients TO oauth_writer;
GRANT SELECT ON oauth_clients TO oauth_reader;
