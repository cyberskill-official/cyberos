-- FR-AUTH-103 — SAML 2.0 SSO per-tenant identity-provider config.
--
-- Three tables mirror the OIDC layout (migration 0011) but with SAML-specific
-- fields: IdP metadata XML / signing certificate / NameID format / attribute
-- mapping. Audit trail in `saml_login_history`; per-IdP user mapping in
-- `saml_subject_link`.
--
-- ADR: ADR-101-rbac-22-role-catalogue (subject-creation surface)

CREATE TABLE saml_idp_configs (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID NOT NULL REFERENCES tenants(id),
    name                TEXT NOT NULL,                  -- 'okta', 'azure-ad', 'google-workspace-saml', etc.
    -- IdP endpoint URLs
    sso_url             TEXT NOT NULL,                  -- SingleSignOnService (HTTP-Redirect or HTTP-POST)
    slo_url             TEXT,                           -- SingleLogoutService (optional)
    issuer              TEXT NOT NULL,                  -- IdP's EntityID
    -- IdP signing cert (PEM-encoded X.509 certificate used to verify SAMLResponse signature)
    signing_cert_pem    TEXT NOT NULL,
    -- SP config (these are CyberOS-side values published in SP metadata)
    sp_entity_id        TEXT NOT NULL,                  -- typically https://auth.cyberos.local/saml/{tenant_slug}
    sp_acs_url          TEXT NOT NULL,                  -- AssertionConsumerService — where IdP POSTs the SAMLResponse
    -- NameID + attribute mapping
    name_id_format      TEXT NOT NULL DEFAULT 'urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress',
    attribute_email     TEXT NOT NULL DEFAULT 'email',  -- which SAML attribute carries the email
    attribute_handle    TEXT,                           -- which SAML attribute carries the handle (optional)
    -- JIT provisioning
    auto_provision      BOOLEAN NOT NULL DEFAULT TRUE,
    default_roles       TEXT[] NOT NULL DEFAULT ARRAY['tenant-member']::TEXT[],
    -- Lifecycle
    status              TEXT NOT NULL DEFAULT 'active', -- 'active' | 'disabled'
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT saml_idp_unique UNIQUE (tenant_id, name),
    CONSTRAINT saml_idp_status_enum CHECK (status IN ('active', 'disabled'))
);

CREATE INDEX saml_idp_configs_tenant_idx ON saml_idp_configs (tenant_id);

ALTER TABLE saml_idp_configs ENABLE ROW LEVEL SECURITY;
ALTER TABLE saml_idp_configs FORCE ROW LEVEL SECURITY;
CREATE POLICY saml_idp_configs_tenant_scoped ON saml_idp_configs
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

GRANT SELECT, INSERT, UPDATE, DELETE ON saml_idp_configs TO cyberos_app;
GRANT SELECT ON saml_idp_configs TO cyberos_ro;

-- ---------------------------------------------------------------------------
-- saml_authn_request_log — every SP-initiated request logged so we can
-- correlate the IdP's later POST against an issued RequestID.
-- ---------------------------------------------------------------------------
CREATE TABLE saml_authn_request_log (
    request_id      TEXT PRIMARY KEY,                       -- SAML AuthnRequest ID (we generate)
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    idp_config_id   UUID NOT NULL REFERENCES saml_idp_configs(id),
    relay_state     TEXT,                                   -- caller-supplied state echoed back
    issued_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '10 minutes',
    consumed_at     TIMESTAMPTZ                             -- set when the IdP's response lands
);

CREATE INDEX saml_authn_request_log_tenant_idx ON saml_authn_request_log (tenant_id, issued_at DESC);
CREATE INDEX saml_authn_request_log_expires_idx ON saml_authn_request_log (expires_at);

ALTER TABLE saml_authn_request_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE saml_authn_request_log FORCE ROW LEVEL SECURITY;
CREATE POLICY saml_authn_request_log_tenant_scoped ON saml_authn_request_log
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

GRANT SELECT, INSERT, UPDATE, DELETE ON saml_authn_request_log TO cyberos_app;
GRANT SELECT ON saml_authn_request_log TO cyberos_ro;

-- ---------------------------------------------------------------------------
-- saml_login_history — outcome of every ACS POST.
-- ---------------------------------------------------------------------------
CREATE TABLE saml_login_history (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    idp_config_id   UUID NOT NULL REFERENCES saml_idp_configs(id),
    request_id      TEXT,                                   -- original AuthnRequest ID (NULL if IdP-initiated)
    subject_id      UUID,                                   -- null until JIT-provisioned
    outcome         TEXT NOT NULL,                          -- 'success' | 'sig_invalid' | 'audience_mismatch' | 'expired' | 'replay' | 'other'
    detail          TEXT,                                   -- short error description on failure
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT saml_login_outcome_enum CHECK (
        outcome IN ('success', 'sig_invalid', 'audience_mismatch', 'expired', 'replay', 'other')
    )
);

CREATE INDEX saml_login_history_tenant_idx ON saml_login_history (tenant_id, occurred_at DESC);

ALTER TABLE saml_login_history ENABLE ROW LEVEL SECURITY;
ALTER TABLE saml_login_history FORCE ROW LEVEL SECURITY;
CREATE POLICY saml_login_history_tenant_scoped ON saml_login_history
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

GRANT SELECT, INSERT, UPDATE ON saml_login_history TO cyberos_app;
GRANT SELECT ON saml_login_history TO cyberos_ro;

-- ---------------------------------------------------------------------------
-- saml_subject_link — per-IdP identifier mapping.
-- ---------------------------------------------------------------------------
CREATE TABLE saml_subject_link (
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    subject_id      UUID NOT NULL REFERENCES subjects(id),
    idp_config_id   UUID NOT NULL REFERENCES saml_idp_configs(id),
    idp_name_id     TEXT NOT NULL,                          -- the SAML NameID value
    idp_email       TEXT,
    linked_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at   TIMESTAMPTZ,
    PRIMARY KEY (idp_config_id, idp_name_id)
);

CREATE INDEX saml_subject_link_subject_idx ON saml_subject_link (subject_id);

ALTER TABLE saml_subject_link ENABLE ROW LEVEL SECURITY;
ALTER TABLE saml_subject_link FORCE ROW LEVEL SECURITY;
CREATE POLICY saml_subject_link_tenant_scoped ON saml_subject_link
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

GRANT SELECT, INSERT, UPDATE, DELETE ON saml_subject_link TO cyberos_app;
GRANT SELECT ON saml_subject_link TO cyberos_ro;
