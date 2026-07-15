-- TASK-AUTH-108 — Lumi tenant-identity JWT issuance log.
--
-- Lumi is the cloud-hosted org BRAIN that serves as the tenant identity
-- federation point for Personal BRAINs syncing into a shared CyberSkill
-- (or external customer) workspace. Every Lumi JWT issuance lands here
-- so the operator can audit cross-personal-BRAIN data flows.
--
-- The JWT itself reuses the existing `auth_signing_keys` infrastructure;
-- this migration adds only the audit log. The verifier path is the
-- existing `JwtService::verify` with a wider audience allowlist.
--
-- ADR: ADR-101-rbac-22-role-catalogue (tenant-identity surface)

CREATE TABLE lumi_token_issuance_log (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID NOT NULL REFERENCES tenants(id),
    subject_id          UUID NOT NULL REFERENCES subjects(id),
    -- Lumi-side identifier — opaque to AUTH, meaningful to Lumi sync code.
    lumi_workspace_id   TEXT NOT NULL,
    -- Token shape
    aud                 TEXT[] NOT NULL,            -- e.g. ['lumi', 'brain-sync']
    scope_grants        TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    expires_at          TIMESTAMPTZ NOT NULL,
    issued_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    kid                 TEXT NOT NULL,              -- which signing key minted this
    jti                 TEXT NOT NULL UNIQUE,       -- replay-detection
    -- Origin trace
    issued_via          TEXT NOT NULL,              -- 'admin' | 'self-service' | 'lumi-sync'
    traceparent         TEXT,
    -- Revocation
    revoked_at          TIMESTAMPTZ,
    revoked_by          UUID,
    revoke_reason       TEXT,

    CONSTRAINT lumi_issuance_via_enum CHECK (issued_via IN ('admin', 'self-service', 'lumi-sync'))
);

CREATE INDEX lumi_issuance_tenant_idx       ON lumi_token_issuance_log (tenant_id, issued_at DESC);
CREATE INDEX lumi_issuance_subject_idx      ON lumi_token_issuance_log (subject_id, issued_at DESC);
CREATE INDEX lumi_issuance_active_idx       ON lumi_token_issuance_log (jti) WHERE revoked_at IS NULL;
CREATE INDEX lumi_issuance_workspace_idx    ON lumi_token_issuance_log (lumi_workspace_id);

ALTER TABLE lumi_token_issuance_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE lumi_token_issuance_log FORCE ROW LEVEL SECURITY;

CREATE POLICY lumi_issuance_tenant_scoped ON lumi_token_issuance_log
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

GRANT SELECT, INSERT, UPDATE ON lumi_token_issuance_log TO cyberos_app;
GRANT SELECT ON lumi_token_issuance_log TO cyberos_ro;
