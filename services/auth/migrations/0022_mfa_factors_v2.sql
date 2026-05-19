-- services/auth/migrations/0022_mfa_factors_v2.sql
BEGIN;

-- Drop the stub table from 0009
DROP TABLE IF EXISTS mfa_factors CASCADE;

CREATE TYPE factor_kind AS ENUM ('totp', 'webauthn_platform', 'webauthn_cross_platform');

CREATE TABLE mfa_factors (
    id                          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id                   UUID         NOT NULL,
    subject_id                  UUID         NOT NULL REFERENCES subjects(id) ON DELETE RESTRICT,
    factor_kind                 factor_kind  NOT NULL,
    display_name                TEXT         NOT NULL CHECK (length(display_name) BETWEEN 1 AND 80),
    totp_secret_kms_blob        BYTEA,
    totp_kms_key_id             TEXT,
    webauthn_credential_id      BYTEA,
    webauthn_public_key         BYTEA,
    webauthn_aaguid             UUID,
    webauthn_signature_count    BIGINT       NOT NULL DEFAULT 0,
    enrolled_at                 TIMESTAMPTZ  NOT NULL DEFAULT now(),
    last_used_at                TIMESTAMPTZ,
    status                      TEXT         NOT NULL CHECK (status IN ('active','removed')) DEFAULT 'active'
);

CREATE UNIQUE INDEX uniq_subject_webauthn_cred ON mfa_factors (subject_id, webauthn_credential_id)
    WHERE webauthn_credential_id IS NOT NULL;
CREATE INDEX mfa_factors_subject_idx ON mfa_factors (subject_id, status);
CREATE INDEX mfa_factors_tenant_idx ON mfa_factors (tenant_id);

ALTER TABLE mfa_factors ENABLE ROW LEVEL SECURITY;
CREATE POLICY mfa_factors_tenant_iso ON mfa_factors
    USING (tenant_id = current_setting('auth.tenant_id', true)::uuid
           OR current_setting('auth.is_root_admin', true) = 'true')
    WITH CHECK (tenant_id = current_setting('auth.tenant_id', true)::uuid);

GRANT SELECT, INSERT, UPDATE, DELETE ON mfa_factors TO cyberos_app;
GRANT SELECT ON mfa_factors TO cyberos_ro;

COMMIT;
