-- TASK-AUTH-102 — MFA factors (TOTP first slice; WebAuthn rows added in slice 2).
--
-- Closed factor enum per AUTHORING_DISCIPLINE §3.4 (no ABAC slide). Factor
-- secrets are stored encrypted at rest in production (KMS-wrapped). For
-- dev + first-boot, base32 plaintext is acceptable and clearly marked.
--
-- ADR: ADR-101-rbac-22-role-catalogue (factor types listed by RFC; spec is
-- self-evident, no new ADR needed for this migration).

CREATE TABLE mfa_factors (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    subject_id      UUID NOT NULL REFERENCES subjects(id),
    factor_type     TEXT NOT NULL,                     -- 'totp' | 'webauthn' | 'recovery-code'
    label           TEXT NOT NULL,                     -- 'iPhone Authenticator', 'YubiKey 5C', etc.
    -- TOTP-specific
    totp_secret     TEXT,                              -- base32-encoded; null for non-TOTP factors
    -- WebAuthn-specific (slice 2)
    cred_id         BYTEA,
    public_key      BYTEA,
    sign_count      BIGINT NOT NULL DEFAULT 0,
    -- Lifecycle
    status          TEXT NOT NULL DEFAULT 'pending',   -- 'pending' | 'active' | 'revoked'
    enrolled_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    activated_at    TIMESTAMPTZ,
    revoked_at      TIMESTAMPTZ,
    last_used_at    TIMESTAMPTZ,

    CONSTRAINT mfa_factor_type_enum CHECK (factor_type IN ('totp', 'webauthn', 'recovery-code')),
    CONSTRAINT mfa_status_enum CHECK (status IN ('pending', 'active', 'revoked'))
);

-- A subject can have at most one ACTIVE factor per type at a time. Pending
-- and revoked rows are unconstrained for forensics.
CREATE UNIQUE INDEX mfa_factors_one_active_per_type
    ON mfa_factors (subject_id, factor_type)
    WHERE status = 'active';

CREATE INDEX mfa_factors_tenant_idx ON mfa_factors (tenant_id);
CREATE INDEX mfa_factors_subject_idx ON mfa_factors (subject_id);

ALTER TABLE mfa_factors ENABLE ROW LEVEL SECURITY;
ALTER TABLE mfa_factors FORCE ROW LEVEL SECURITY;

CREATE POLICY mfa_factors_tenant_scoped ON mfa_factors
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

GRANT SELECT, INSERT, UPDATE, DELETE ON mfa_factors TO cyberos_app;
GRANT SELECT ON mfa_factors TO cyberos_ro;
