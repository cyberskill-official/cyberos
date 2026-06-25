-- FR-MCP-004 Migration 0012: oauth_refresh_families, oauth_revocation_list, oauth_consents
-- DEC-805 (TTLs), DEC-806 (rotation), §1.17 (enum), §1.21 (revocation), §1.29 (consent)

CREATE TYPE oauth_refresh_state AS ENUM ('active', 'used', 'compromised');

CREATE TABLE oauth_refresh_families (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    family_id            UUID NOT NULL,
    client_id            UUID NOT NULL REFERENCES oauth_clients(id),
    subject_id           UUID NOT NULL REFERENCES subjects(id),
    tenant_id            UUID NOT NULL REFERENCES tenants(id),
    audience             TEXT NOT NULL,
    scope                TEXT NOT NULL,
    token_hash           CHAR(64) NOT NULL,    -- SHA-256 hex of the opaque refresh token
    parent_token_hash    CHAR(64),             -- NULL for root of family
    issued_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at           TIMESTAMPTZ NOT NULL,  -- issued_at + INTERVAL '30 days'
    state                oauth_refresh_state NOT NULL DEFAULT 'active',
    state_changed_at     TIMESTAMPTZ,
    memory_chain_hash    CHAR(64) NOT NULL
);

CREATE UNIQUE INDEX oauth_refresh_token_hash ON oauth_refresh_families (token_hash);
CREATE INDEX oauth_refresh_family_active ON oauth_refresh_families (family_id) WHERE state = 'active';

REVOKE UPDATE, DELETE ON oauth_refresh_families FROM cyberos_app;
GRANT INSERT, SELECT ON oauth_refresh_families TO oauth_refresh_writer;
GRANT UPDATE(state, state_changed_at) ON oauth_refresh_families TO oauth_refresh_writer;

-- Revocation list for access token JTIs (RFC 7009 + DEC-817)
CREATE TABLE oauth_revocation_list (
    jti          UUID PRIMARY KEY,
    revoked_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at   TIMESTAMPTZ NOT NULL,  -- matches JWT exp; row eligible for TTL eviction
    reason       TEXT
);

CREATE INDEX oauth_revocation_expires ON oauth_revocation_list (expires_at);

REVOKE UPDATE, DELETE ON oauth_revocation_list FROM cyberos_app;
GRANT INSERT, SELECT ON oauth_revocation_list TO oauth_writer;

-- Consent records (§1.29)
CREATE TABLE oauth_consents (
    subject_id   UUID NOT NULL REFERENCES subjects(id),
    client_id    UUID NOT NULL REFERENCES oauth_clients(id),
    scopes       TEXT NOT NULL,
    granted_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (subject_id, client_id)
);
