-- TASK-AUTH-005 §1 #10 + #13 + G-010/G-013 — active-jti tracking table.
--
-- Every successful JWT issue (TASK-AUTH-004) inserts a row. The revoke path
-- (TASK-AUTH-005 §1 #3 + G-003) enumerates this table per subject to know
-- which jtis to push into the in-memory deny-list (G-011) for instant
-- revocation. The expires_at field matches the JWT's `exp` claim; rows can
-- be reaped by a sweeper job after `now() > expires_at` since no valid
-- token can still reference them.
--
-- Why TEXT for jti (not UUID)? jtis come from `uuid::Uuid::new_v4()` today
-- but the spec doesn't bind the format. TEXT keeps the door open for KSUID
-- / ULID variants in future without a column-type migration.
--
-- source_ip_hash16 = first 16 hex chars of SHA-256(source_ip) — opaque
-- enough to prevent cross-tenant correlation without leaking the raw IP
-- into the audit chain (mirrors `brain_bridge::email_hash16` discipline).

CREATE TABLE IF NOT EXISTS sessions (
    jti              TEXT PRIMARY KEY,
    subject_id       UUID NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    tenant_id        UUID NOT NULL REFERENCES tenants(id)  ON DELETE CASCADE,
    issued_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at       TIMESTAMPTZ NOT NULL,
    source_ip_hash16 TEXT NOT NULL,
    CONSTRAINT sessions_jti_nonempty   CHECK (length(jti) > 0),
    CONSTRAINT sessions_ip_hash_length CHECK (length(source_ip_hash16) = 16)
);

-- Look-up by subject (revoke path enumerates the subject's active jtis).
CREATE INDEX IF NOT EXISTS sessions_subject_id_idx
    ON sessions (subject_id);

-- Reaper job query: WHERE expires_at < NOW() — index supports range scan
-- + the tenant-scoped variant ("show me my tenant's active sessions").
CREATE INDEX IF NOT EXISTS sessions_tenant_expires_idx
    ON sessions (tenant_id, expires_at);

-- TASK-AUTH-005 §1 #13 + G-013 — sessions is tenant-scoped → RLS coverage
-- via the same global-GUC pattern as 0005 and 0019.
ALTER TABLE sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE sessions FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS sessions_tenant_isolation ON sessions;
CREATE POLICY sessions_tenant_isolation ON sessions
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

-- Grant to the app role so RLS predicates fire (rather than blanket allow).
GRANT SELECT, INSERT, UPDATE, DELETE ON sessions TO cyberos_app;
