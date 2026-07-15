-- TASK-AUTH-110 §1 #3 + DEC-2489 - AUTH SSO browser session.
--
-- The "log in once, every first-party app trusts it" linchpin. The browser holds
-- a __Host-cyberos_sso cookie carrying this row's id; this table is the source of
-- truth so the session is revocable (a subject revoke cascades revoked_at here per
-- §1 #26, so silent SSO stops too, not only new logins). Sliding 8h on
-- last_seen_at, absolute 24h on absolute_expiry. Tenant-scoped RLS as 0021.

CREATE TABLE IF NOT EXISTS auth_sso_sessions (
    id               UUID        PRIMARY KEY,
    tenant_id        UUID        NOT NULL REFERENCES tenants(id)  ON DELETE CASCADE,
    subject_id       UUID        NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    absolute_expiry  TIMESTAMPTZ NOT NULL,
    revoked_at       TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS auth_sso_sessions_subject_idx
    ON auth_sso_sessions (subject_id) WHERE revoked_at IS NULL;

ALTER TABLE auth_sso_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE auth_sso_sessions FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS auth_sso_sessions_tenant_isolation ON auth_sso_sessions;
CREATE POLICY auth_sso_sessions_tenant_isolation ON auth_sso_sessions
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

GRANT SELECT, INSERT, UPDATE, DELETE ON auth_sso_sessions TO cyberos_app;
