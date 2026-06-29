-- FR-AUTH-110 §1 #4 + DEC-2495 - append-only OIDC-provider login history.
--
-- Forensic record of authorize/token issuance and denials. Append-only at the SQL
-- grant: cyberos_app may SELECT + INSERT but UPDATE/DELETE are revoked, so "who
-- authenticated into which app when, and who was denied" cannot be rewritten.
-- Tenant-scoped RLS as 0021.

CREATE TABLE IF NOT EXISTS auth_op_login_history (
    id               BIGSERIAL   PRIMARY KEY,
    tenant_id        UUID        NOT NULL,
    rp_client_id     TEXT,
    subject_id       UUID,
    outcome          TEXT        NOT NULL,
    reason           TEXT,
    source_ip_hash16 TEXT,
    ts               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT op_login_outcome_valid CHECK (outcome IN ('authorize_issued','token_issued','denied')),
    CONSTRAINT op_login_ip_hash_len   CHECK (source_ip_hash16 IS NULL OR length(source_ip_hash16) = 16)
);

CREATE INDEX IF NOT EXISTS auth_op_login_history_tenant_ts_idx
    ON auth_op_login_history (tenant_id, ts DESC);

ALTER TABLE auth_op_login_history ENABLE ROW LEVEL SECURITY;
ALTER TABLE auth_op_login_history FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS auth_op_login_history_tenant_isolation ON auth_op_login_history;
CREATE POLICY auth_op_login_history_tenant_isolation ON auth_op_login_history
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

-- Append-only: INSERT + SELECT only; UPDATE/DELETE revoked (forensic integrity).
GRANT SELECT, INSERT ON auth_op_login_history TO cyberos_app;
GRANT USAGE, SELECT ON SEQUENCE auth_op_login_history_id_seq TO cyberos_app;
REVOKE UPDATE, DELETE ON auth_op_login_history FROM cyberos_app;
