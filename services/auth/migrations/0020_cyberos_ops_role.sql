-- TASK-AUTH-003 §1 #5 — slice-1 audit-fix G-002
--
-- Create the `cyberos_ops` BYPASSRLS role for legitimate cross-tenant
-- operations (compliance reports, regulator audits, ops investigations)
-- and the audit table that records every usage.
--
-- Every query executed under `cyberos_ops` SHOULD be preceded by an
-- INSERT into `auth_rls_bypass_audit` via `rls::emit_cyberos_ops_audit_row()`.
-- The Rust helper does this best-effort (failure to audit does NOT block
-- the bypass query) but emits a tracing::warn so the missed-audit case
-- is observable. Sev-2 alarm fires on `auth.rls_bypass_used` counter
-- increment beyond baseline (TASK-OBS-001 wires the alarm).
--
-- ROLE LIFECYCLE
-- --------------
--   * created here at migration time with a password placeholder; the
--     actual password is rotated by the deployment pipeline via Vault.
--   * BYPASSRLS attribute means RLS policies are ignored entirely for
--     this role's sessions. INHERIT means it picks up granted privileges
--     from member roles.
--   * NOSUPERUSER NOCREATEROLE NOCREATEDB — no other privilege escalation.
--   * LOGIN — yes, this role connects directly (vs being granted to
--     other roles). The deployment pipeline issues a separate connection
--     string for it.

DO $do$
BEGIN
    -- Idempotent: skip if already exists (migration replay safety).
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'cyberos_ops') THEN
        CREATE ROLE cyberos_ops
            WITH LOGIN
                 NOSUPERUSER
                 NOCREATEDB
                 NOCREATEROLE
                 INHERIT
                 BYPASSRLS
                 PASSWORD 'rotate-via-vault-immediately';
        COMMENT ON ROLE cyberos_ops IS
            'TASK-AUTH-003 §1 #5 — BYPASSRLS role for legitimate cross-tenant ops. '
            'Password rotated by Vault on bootstrap. Usage MUST be preceded by '
            'INSERT INTO auth_rls_bypass_audit via rls::emit_cyberos_ops_audit_row().';
    END IF;
END;
$do$;

-- ----- auth_rls_bypass_audit -----------------------------------------------
--
-- Append-only audit log of every BYPASSRLS query the application emitted.
-- We rely on application-layer emission rather than Postgres-level logging
-- because the application knows the *purpose* (compliance report vs
-- regulator audit vs incident-response query) and the *operator identity*
-- (root-admin email_hash16, never the raw user id which could leak PII).
--
-- This table is NOT tenant-scoped (it's about the *bypass*, which by
-- definition crosses tenants), so no RLS. It IS append-only — only
-- INSERT is granted; UPDATE/DELETE are revoked.
--
-- Retention: 7 years (compliance requirement — surfaced in
-- docs/tasks/auth/TASK-AUTH-003-rls-enforcement.audit.md §10.5).
CREATE TABLE IF NOT EXISTS auth_rls_bypass_audit (
    audit_id        BIGSERIAL PRIMARY KEY,
    operator_id     TEXT NOT NULL,        -- email_hash16 of the operator (NOT raw email)
    query_purpose   TEXT NOT NULL,        -- "compliance_report" | "regulator_audit" | "incident_response" | "ops_investigation"
    used_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Future-proof: which db user actually connected (Postgres reports
    -- session_user). Default to current_user so the row records the
    -- effective bypass role even if the helper is called via a different
    -- service identity in the future.
    session_user_at_use TEXT NOT NULL DEFAULT CURRENT_USER
);

CREATE INDEX IF NOT EXISTS auth_rls_bypass_audit_used_at_idx
    ON auth_rls_bypass_audit (used_at DESC);
CREATE INDEX IF NOT EXISTS auth_rls_bypass_audit_operator_idx
    ON auth_rls_bypass_audit (operator_id, used_at DESC);

COMMENT ON TABLE auth_rls_bypass_audit IS
    'TASK-AUTH-003 §1 #5 — every BYPASSRLS query the application emits writes one row here. '
    'Append-only; UPDATE/DELETE revoked. 7-year retention per TASK-AUTH-003 §10.5.';

-- Grants:
--   * cyberos_app and cyberos_ops can INSERT (write audit rows from app code)
--   * cyberos_ro can SELECT (compliance reports / dashboards)
--   * UPDATE/DELETE — nobody. Append-only enforced at grant level.
GRANT INSERT ON auth_rls_bypass_audit TO cyberos_app;
GRANT INSERT ON auth_rls_bypass_audit TO cyberos_ops;
GRANT SELECT ON auth_rls_bypass_audit TO cyberos_ro;
GRANT USAGE, SELECT ON SEQUENCE auth_rls_bypass_audit_audit_id_seq TO cyberos_app;
GRANT USAGE, SELECT ON SEQUENCE auth_rls_bypass_audit_audit_id_seq TO cyberos_ops;

-- Grant cyberos_ops read access to every tenant-scoped table (it has
-- BYPASSRLS so policies don't apply, but it still needs SELECT/INSERT/
-- UPDATE/DELETE at the table-grant level).
GRANT SELECT, INSERT, UPDATE, DELETE ON
    admin_idempotency,
    auth_migration_state,
    auth_signing_keys,
    hibp_audit,
    login_history_geo,
    lumi_token_issuance_log,
    mfa_factors,
    oidc_idp_configs,
    passkey_enrolment_state,
    saml_idp_configs,
    subject_roles,
    subjects,
    travel_cidr_allowlist,
    travel_policy,
    travel_policy_audit
TO cyberos_ops;

-- Also grant the tenants table (the root entity).
GRANT SELECT, INSERT, UPDATE, DELETE ON tenants TO cyberos_ops;
