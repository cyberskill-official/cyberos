-- services/auth/migrations/0023_mfa_factor_history.sql
BEGIN;

CREATE TABLE mfa_factor_history (
    id                       BIGSERIAL    PRIMARY KEY,
    tenant_id                UUID         NOT NULL,
    subject_id               UUID         NOT NULL,
    factor_id                UUID,
    action                   TEXT         NOT NULL CHECK (action IN ('enrolled','removed','status_changed')),
    factor_kind              factor_kind  NOT NULL,
    display_name             TEXT,
    changed_at               TIMESTAMPTZ  NOT NULL DEFAULT now(),
    changed_by_subject_id    UUID,
    reason                   TEXT
);

CREATE INDEX mfa_factor_history_subject_idx ON mfa_factor_history (subject_id, changed_at DESC);

ALTER TABLE mfa_factor_history ENABLE ROW LEVEL SECURITY;
CREATE POLICY mfa_factor_history_tenant_iso ON mfa_factor_history
    USING (tenant_id = current_setting('auth.tenant_id', true)::uuid
           OR current_setting('auth.is_root_admin', true) = 'true')
    WITH CHECK (tenant_id = current_setting('auth.tenant_id', true)::uuid);

REVOKE UPDATE, DELETE ON mfa_factor_history FROM cyberos_app;
GRANT INSERT, SELECT ON mfa_factor_history TO cyberos_app;

COMMIT;
