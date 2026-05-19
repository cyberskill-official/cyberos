-- services/auth/migrations/0026_mfa_lockout_state.sql
BEGIN;

CREATE TABLE mfa_lockout_state (
    subject_id           UUID         PRIMARY KEY REFERENCES subjects(id) ON DELETE RESTRICT,
    tenant_id            UUID         NOT NULL,
    failed_count         INT          NOT NULL DEFAULT 0 CHECK (failed_count >= 0),
    window_started_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    locked_until         TIMESTAMPTZ,
    last_attempt_at      TIMESTAMPTZ
);

CREATE INDEX mfa_lockout_state_locked_idx ON mfa_lockout_state (locked_until) WHERE locked_until IS NOT NULL;

ALTER TABLE mfa_lockout_state ENABLE ROW LEVEL SECURITY;
CREATE POLICY mfa_lockout_state_tenant_iso ON mfa_lockout_state
    USING (tenant_id = current_setting('auth.tenant_id', true)::uuid
           OR current_setting('auth.is_root_admin', true) = 'true');

REVOKE UPDATE, DELETE ON mfa_lockout_state FROM cyberos_app;
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'mfa_lockout_writer') THEN
        CREATE ROLE mfa_lockout_writer;
    END IF;
END
$$;
GRANT INSERT, UPDATE (failed_count, window_started_at, locked_until, last_attempt_at) ON mfa_lockout_state TO mfa_lockout_writer;
GRANT SELECT ON mfa_lockout_state TO cyberos_app, mfa_lockout_writer;

COMMIT;
