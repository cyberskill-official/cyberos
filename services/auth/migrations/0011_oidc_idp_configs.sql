-- FR-AUTH-104 — OIDC SSO per-tenant identity-provider config + login history.
--
-- One tenant can have multiple IdPs (e.g. Google + Microsoft + Okta) keyed by
-- a tenant-supplied `name`. The discovery URL is required; client credentials
-- are stored encrypted at rest in production (KMS-wrapped) — for dev they live
-- as plaintext columns clearly marked. The audit trail on login lives in
-- `oidc_login_history` so DPO can answer "who logged in via SSO when".
--
-- ADR: ADR-101-rbac-22-role-catalogue (touches subject creation surface only)

CREATE TABLE oidc_idp_configs (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID NOT NULL REFERENCES tenants(id),
    name                TEXT NOT NULL,                  -- 'google-workspace', 'microsoft-entra', 'okta', etc.
    discovery_url       TEXT NOT NULL,                  -- .well-known/openid-configuration URL
    client_id           TEXT NOT NULL,
    client_secret       TEXT NOT NULL,                  -- KMS-wrapped in prod
    redirect_uri        TEXT NOT NULL,                  -- canonical callback (must match registered)
    scopes              TEXT[] NOT NULL DEFAULT ARRAY['openid', 'email', 'profile']::TEXT[],
    -- JIT provisioning
    auto_provision      BOOLEAN NOT NULL DEFAULT TRUE,  -- create subjects on first successful login
    default_roles       TEXT[] NOT NULL DEFAULT ARRAY['tenant-member']::TEXT[],
    -- Lifecycle
    status              TEXT NOT NULL DEFAULT 'active', -- 'active' | 'disabled'
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT oidc_idp_unique UNIQUE (tenant_id, name),
    CONSTRAINT oidc_idp_status_enum CHECK (status IN ('active', 'disabled'))
);

CREATE INDEX oidc_idp_configs_tenant_idx ON oidc_idp_configs (tenant_id);

ALTER TABLE oidc_idp_configs ENABLE ROW LEVEL SECURITY;
ALTER TABLE oidc_idp_configs FORCE ROW LEVEL SECURITY;
CREATE POLICY oidc_idp_configs_tenant_scoped ON oidc_idp_configs
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) =
           '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) =
           '00000000-0000-0000-0000-000000000000'
    );

GRANT SELECT, INSERT, UPDATE, DELETE ON oidc_idp_configs TO cyberos_app;
GRANT SELECT ON oidc_idp_configs TO cyberos_ro;

-- ---------------------------------------------------------------------------
-- oidc_login_history — every initiate + callback is logged for DPO + DSAR.
-- ---------------------------------------------------------------------------
CREATE TABLE oidc_login_history (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    idp_config_id   UUID NOT NULL REFERENCES oidc_idp_configs(id),
    subject_id      UUID,                                   -- null until JIT-provisioned
    flow_state      TEXT NOT NULL,                          -- 'initiated' | 'callback_ok' | 'callback_err' | 'jit_provisioned'
    state_token     TEXT NOT NULL,                          -- CSRF state; HMAC of (nonce, tenant_id, idp_id)
    code_verifier_hash TEXT,                                -- PKCE: SHA-256 of the code_verifier we generated
    error_code      TEXT,                                   -- 'invalid_state' | 'token_exchange_failed' | etc.
    error_detail    TEXT,
    ts_ns           BIGINT NOT NULL,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT oidc_login_flow_enum CHECK (
        flow_state IN ('initiated', 'callback_ok', 'callback_err', 'jit_provisioned')
    )
);

CREATE INDEX oidc_login_history_tenant_idx ON oidc_login_history (tenant_id, occurred_at DESC);
CREATE INDEX oidc_login_history_state_idx ON oidc_login_history (state_token);

ALTER TABLE oidc_login_history ENABLE ROW LEVEL SECURITY;
ALTER TABLE oidc_login_history FORCE ROW LEVEL SECURITY;
CREATE POLICY oidc_login_history_tenant_scoped ON oidc_login_history
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) =
           '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) =
           '00000000-0000-0000-0000-000000000000'
    );

GRANT SELECT, INSERT, UPDATE ON oidc_login_history TO cyberos_app;
GRANT SELECT ON oidc_login_history TO cyberos_ro;

-- ---------------------------------------------------------------------------
-- oidc_subject_link — links a CyberOS subject to its IdP-side identifier
-- (the `sub` claim from the ID token). One subject can be linked to
-- multiple IdPs for federation-bridge scenarios.
-- ---------------------------------------------------------------------------
CREATE TABLE oidc_subject_link (
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    subject_id      UUID NOT NULL REFERENCES subjects(id),
    idp_config_id   UUID NOT NULL REFERENCES oidc_idp_configs(id),
    idp_sub         TEXT NOT NULL,                          -- the OIDC `sub` claim
    idp_email       TEXT,                                   -- snapshot at link time
    linked_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at   TIMESTAMPTZ,
    PRIMARY KEY (idp_config_id, idp_sub)
);

CREATE INDEX oidc_subject_link_subject_idx ON oidc_subject_link (subject_id);

ALTER TABLE oidc_subject_link ENABLE ROW LEVEL SECURITY;
ALTER TABLE oidc_subject_link FORCE ROW LEVEL SECURITY;
CREATE POLICY oidc_subject_link_tenant_scoped ON oidc_subject_link
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) =
           '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) =
           '00000000-0000-0000-0000-000000000000'
    );

GRANT SELECT, INSERT, UPDATE, DELETE ON oidc_subject_link TO cyberos_app;
GRANT SELECT ON oidc_subject_link TO cyberos_ro;
