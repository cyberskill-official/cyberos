-- FR-AUTH-110 §1 #2 + #16 - single-use, 60s-TTL, PKCE-bound authorization codes.
--
-- The PK is sha256(code), never the code itself (DB-dump safety). Single-use is
-- enforced NOT by an append-only UPDATE on this table but by a sibling
-- `auth_oidc_code_consumptions(code_hash PK)` first-insert-wins guard
-- (audit OPEN-001 ADR #1): the token endpoint INSERTs into consumptions; the
-- first wins, a second raises a unique violation = replay -> invalid_grant. That
-- leaves this codes table reapable (a sweeper DELETEs rows past expires_at)
-- instead of forced-append-only. The forensic append-only record is
-- auth_op_login_history (0030), not this table.

CREATE TABLE IF NOT EXISTS auth_oidc_auth_codes (
    code_hash       TEXT        PRIMARY KEY,
    tenant_id       UUID        NOT NULL REFERENCES tenants(id)  ON DELETE CASCADE,
    rp_client_id    TEXT        NOT NULL,
    subject_id      UUID        NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    redirect_uri    TEXT        NOT NULL,
    code_challenge  TEXT        NOT NULL,
    nonce           TEXT,
    scope           TEXT        NOT NULL,
    sso_session_id  UUID        NOT NULL,
    issued_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMPTZ NOT NULL,
    CONSTRAINT auth_codes_challenge_nonempty CHECK (length(code_challenge) > 0)
);

CREATE INDEX IF NOT EXISTS auth_oidc_auth_codes_expiry_idx
    ON auth_oidc_auth_codes (expires_at);

ALTER TABLE auth_oidc_auth_codes ENABLE ROW LEVEL SECURITY;
ALTER TABLE auth_oidc_auth_codes FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS auth_oidc_auth_codes_tenant_isolation ON auth_oidc_auth_codes;
CREATE POLICY auth_oidc_auth_codes_tenant_isolation ON auth_oidc_auth_codes
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

GRANT SELECT, INSERT, DELETE ON auth_oidc_auth_codes TO cyberos_app;

-- Single-use guard (first INSERT wins; second = replay).
CREATE TABLE IF NOT EXISTS auth_oidc_code_consumptions (
    code_hash    TEXT        PRIMARY KEY,
    tenant_id    UUID        NOT NULL,
    consumed_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE auth_oidc_code_consumptions ENABLE ROW LEVEL SECURITY;
ALTER TABLE auth_oidc_code_consumptions FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS auth_oidc_code_consumptions_tenant_isolation ON auth_oidc_code_consumptions;
CREATE POLICY auth_oidc_code_consumptions_tenant_isolation ON auth_oidc_code_consumptions
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

GRANT SELECT, INSERT, DELETE ON auth_oidc_code_consumptions TO cyberos_app;
