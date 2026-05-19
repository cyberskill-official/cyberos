-- services/auth/migrations/0024_mfa_challenge_log.sql
BEGIN;

CREATE TABLE mfa_challenge_log (
    id                  BIGSERIAL    PRIMARY KEY,
    tenant_id           UUID         NOT NULL,
    subject_id          UUID         NOT NULL,
    challenge_id        UUID         NOT NULL UNIQUE,
    factor_id           UUID         REFERENCES mfa_factors(id) ON DELETE SET NULL,
    challenge_kind      TEXT         NOT NULL CHECK (challenge_kind IN ('totp','webauthn')),
    status              TEXT         NOT NULL CHECK (status IN ('pending','consumed','expired','failed')),
    issued_at           TIMESTAMPTZ  NOT NULL DEFAULT now(),
    expires_at          TIMESTAMPTZ  NOT NULL,
    consumed_at         TIMESTAMPTZ,
    source_ip_hash16    TEXT
);

CREATE INDEX mfa_challenge_log_subject_idx ON mfa_challenge_log (subject_id, issued_at DESC);
CREATE INDEX mfa_challenge_log_status_idx ON mfa_challenge_log (status, expires_at);

ALTER TABLE mfa_challenge_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY mfa_challenge_log_tenant_iso ON mfa_challenge_log
    USING (tenant_id = current_setting('auth.tenant_id', true)::uuid
           OR current_setting('auth.is_root_admin', true) = 'true')
    WITH CHECK (tenant_id = current_setting('auth.tenant_id', true)::uuid);

REVOKE UPDATE, DELETE ON mfa_challenge_log FROM cyberos_app;
-- Status transitions are via privileged mfa_challenge_writer role (granted to handlers)
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'mfa_challenge_writer') THEN
        CREATE ROLE mfa_challenge_writer;
    END IF;
END
$$;
GRANT INSERT ON mfa_challenge_log TO mfa_challenge_writer, cyberos_app;
GRANT UPDATE (status, consumed_at) ON mfa_challenge_log TO mfa_challenge_writer;
GRANT SELECT ON mfa_challenge_log TO cyberos_app;

COMMIT;
