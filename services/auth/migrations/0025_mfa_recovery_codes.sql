-- services/auth/migrations/0025_mfa_recovery_codes.sql
BEGIN;

CREATE TABLE mfa_recovery_codes (
    id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID         NOT NULL,
    subject_id          UUID         NOT NULL REFERENCES subjects(id) ON DELETE RESTRICT,
    code_bcrypt_hash    TEXT         NOT NULL,
    batch_id            UUID         NOT NULL,
    consumed            BOOLEAN      NOT NULL DEFAULT false,
    consumed_at         TIMESTAMPTZ,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX mfa_recovery_subject_batch_idx ON mfa_recovery_codes (subject_id, batch_id, consumed);

ALTER TABLE mfa_recovery_codes ENABLE ROW LEVEL SECURITY;
CREATE POLICY mfa_recovery_tenant_iso ON mfa_recovery_codes
    USING (tenant_id = current_setting('auth.tenant_id', true)::uuid
           OR current_setting('auth.is_root_admin', true) = 'true')
    WITH CHECK (tenant_id = current_setting('auth.tenant_id', true)::uuid);

-- Update of `consumed` is the only allowed mutation
REVOKE DELETE ON mfa_recovery_codes FROM cyberos_app;
REVOKE UPDATE ON mfa_recovery_codes FROM cyberos_app;
GRANT UPDATE (consumed, consumed_at) ON mfa_recovery_codes TO cyberos_app;
GRANT INSERT, SELECT ON mfa_recovery_codes TO cyberos_app;

COMMIT;
