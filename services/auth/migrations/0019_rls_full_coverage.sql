-- FR-AUTH-003 §1 #1 — slice-1 audit-fix G-001
--
-- Enable RLS on every tenant-scoped table that was added in migrations
-- 0007..0018 without RLS policies. Each policy uses the same global-GUC
-- pattern as 0005_rls_enable_on_tables.sql:
--
--   USING       (tenant_id::text = current_setting('app.current_tenant_id', true)
--                OR current_setting = nil-UUID)
--   WITH CHECK  (same)
--
-- The nil-UUID branch is the root-tenant escape hatch (only root-admin in
-- tenant 0 can read/write across tenant boundaries via this endpoint set).
-- Other admin operations that legitimately need cross-tenant reach go
-- through the `cyberos_ops` BYPASSRLS role in 0020 with audit-row emission.
--
-- Tables enabled here: subject_roles · mfa_factors · hibp_audit ·
-- oidc_idp_configs · passkey_enrolment_state · saml_idp_configs ·
-- login_history_geo · auth_migration_state · lumi_token_issuance_log ·
-- travel_policy · travel_cidr_allowlist · travel_policy_audit.
--
-- auth_signing_keys is service-global (not tenant-scoped) and is registered
-- in TENANT_SCOPED_TABLES for the existence check only; no RLS policy needed.

-- Helper: a single macro-like template would be nice, but Postgres doesn't
-- support DO-blocks that DDL-loop over a string array with CREATE POLICY
-- (parameter binding inside CREATE POLICY is restricted). So we expand by hand.

-- ----- subject_roles ----------------------------------------------------------
ALTER TABLE subject_roles ENABLE ROW LEVEL SECURITY;
ALTER TABLE subject_roles FORCE ROW LEVEL SECURITY;
CREATE POLICY subject_roles_tenant_scoped ON subject_roles
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

-- ----- mfa_factors -----------------------------------------------------------
ALTER TABLE mfa_factors ENABLE ROW LEVEL SECURITY;
ALTER TABLE mfa_factors FORCE ROW LEVEL SECURITY;
CREATE POLICY mfa_factors_tenant_scoped ON mfa_factors
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

-- ----- hibp_audit ------------------------------------------------------------
ALTER TABLE hibp_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE hibp_audit FORCE ROW LEVEL SECURITY;
CREATE POLICY hibp_audit_tenant_scoped ON hibp_audit
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

-- ----- oidc_idp_configs ------------------------------------------------------
ALTER TABLE oidc_idp_configs ENABLE ROW LEVEL SECURITY;
ALTER TABLE oidc_idp_configs FORCE ROW LEVEL SECURITY;
CREATE POLICY oidc_idp_configs_tenant_scoped ON oidc_idp_configs
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

-- ----- passkey_enrolment_state -----------------------------------------------
ALTER TABLE passkey_enrolment_state ENABLE ROW LEVEL SECURITY;
ALTER TABLE passkey_enrolment_state FORCE ROW LEVEL SECURITY;
CREATE POLICY passkey_enrolment_state_tenant_scoped ON passkey_enrolment_state
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

-- ----- saml_idp_configs ------------------------------------------------------
ALTER TABLE saml_idp_configs ENABLE ROW LEVEL SECURITY;
ALTER TABLE saml_idp_configs FORCE ROW LEVEL SECURITY;
CREATE POLICY saml_idp_configs_tenant_scoped ON saml_idp_configs
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

-- ----- login_history_geo -----------------------------------------------------
ALTER TABLE login_history_geo ENABLE ROW LEVEL SECURITY;
ALTER TABLE login_history_geo FORCE ROW LEVEL SECURITY;
CREATE POLICY login_history_geo_tenant_scoped ON login_history_geo
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

-- ----- auth_migration_state --------------------------------------------------
ALTER TABLE auth_migration_state ENABLE ROW LEVEL SECURITY;
ALTER TABLE auth_migration_state FORCE ROW LEVEL SECURITY;
CREATE POLICY auth_migration_state_tenant_scoped ON auth_migration_state
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

-- ----- lumi_token_issuance_log -----------------------------------------------
ALTER TABLE lumi_token_issuance_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE lumi_token_issuance_log FORCE ROW LEVEL SECURITY;
CREATE POLICY lumi_token_issuance_log_tenant_scoped ON lumi_token_issuance_log
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

-- ----- travel_policy ---------------------------------------------------------
ALTER TABLE travel_policy ENABLE ROW LEVEL SECURITY;
ALTER TABLE travel_policy FORCE ROW LEVEL SECURITY;
CREATE POLICY travel_policy_tenant_scoped ON travel_policy
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

-- ----- travel_cidr_allowlist -------------------------------------------------
ALTER TABLE travel_cidr_allowlist ENABLE ROW LEVEL SECURITY;
ALTER TABLE travel_cidr_allowlist FORCE ROW LEVEL SECURITY;
CREATE POLICY travel_cidr_allowlist_tenant_scoped ON travel_cidr_allowlist
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

-- ----- travel_policy_audit ---------------------------------------------------
ALTER TABLE travel_policy_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE travel_policy_audit FORCE ROW LEVEL SECURITY;
CREATE POLICY travel_policy_audit_tenant_scoped ON travel_policy_audit
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

-- Grant cyberos_app the standard DML on all of the above (covers the
-- newly-added tables that the default-privileges grant in 0004 only
-- covers for FUTURE creates, not retroactive ones).
GRANT SELECT, INSERT, UPDATE, DELETE ON
    subject_roles,
    mfa_factors,
    hibp_audit,
    oidc_idp_configs,
    passkey_enrolment_state,
    saml_idp_configs,
    login_history_geo,
    auth_migration_state,
    lumi_token_issuance_log,
    travel_policy,
    travel_cidr_allowlist,
    travel_policy_audit
TO cyberos_app;
GRANT SELECT ON
    subject_roles,
    mfa_factors,
    hibp_audit,
    oidc_idp_configs,
    passkey_enrolment_state,
    saml_idp_configs,
    login_history_geo,
    auth_migration_state,
    lumi_token_issuance_log,
    travel_policy,
    travel_cidr_allowlist,
    travel_policy_audit
TO cyberos_ro;
