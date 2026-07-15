-- TASK-AUTH-110 §1 #1 - first-party OIDC relying-party (RP) registry.
--
-- Admin-registered confidential clients (CHAT/Mattermost, PORTAL) that federate
-- to AUTH's OIDC provider. redirect_uris are locked (exact-match enforced in
-- op::redirect, DEC-2491). client_secret is stored as a SHA-256 hash, not the
-- secret nor a reversible KMS blob (ADR refining DEC-2483): an RP secret is only
-- ever VERIFIED at the token endpoint (Mattermost authenticating to us), never
-- recovered, so a one-way hash is both codebase-consistent (no KMS exists here -
-- oidc_idp_configs stores upstream secrets plain because it must replay them to
-- Google) and stronger. The plaintext is revealed exactly once at create.
-- allow_refresh is opt-in per RP (DEC-2499, default off - Mattermost mints its
-- own session and needs no refresh token). Tenant-scoped via the same global-GUC
-- RLS pattern as 0021_sessions.sql.

CREATE TABLE IF NOT EXISTS auth_oidc_rp_clients (
    id                         UUID        PRIMARY KEY,
    tenant_id                  UUID        NOT NULL REFERENCES tenants(id)  ON DELETE CASCADE,
    name                       TEXT        NOT NULL,
    client_id                  TEXT        NOT NULL UNIQUE,
    client_secret_hash         TEXT        NOT NULL,
    redirect_uris              TEXT[]      NOT NULL,
    post_logout_redirect_uris  TEXT[]      NOT NULL DEFAULT '{}',
    allow_refresh              BOOLEAN     NOT NULL DEFAULT false,
    is_active                  BOOLEAN     NOT NULL DEFAULT true,
    created_at                 TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by_subject_id      UUID        NOT NULL REFERENCES subjects(id) ON DELETE RESTRICT,
    CONSTRAINT rp_clients_name_nonempty     CHECK (length(name) BETWEEN 1 AND 100),
    CONSTRAINT rp_clients_redirects_bounded CHECK (cardinality(redirect_uris) BETWEEN 1 AND 10)
);

CREATE UNIQUE INDEX IF NOT EXISTS auth_oidc_rp_clients_name_idx
    ON auth_oidc_rp_clients (tenant_id, name);

ALTER TABLE auth_oidc_rp_clients ENABLE ROW LEVEL SECURITY;
ALTER TABLE auth_oidc_rp_clients FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS auth_oidc_rp_clients_tenant_isolation ON auth_oidc_rp_clients;
CREATE POLICY auth_oidc_rp_clients_tenant_isolation ON auth_oidc_rp_clients
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

GRANT SELECT, INSERT, UPDATE, DELETE ON auth_oidc_rp_clients TO cyberos_app;
