-- FR-AUTH-107 — Have I Been Pwned breach-check audit table.
--
-- Every password set/rotate hits the k-anonymity HIBP API. If the password's
-- SHA-1 prefix returns a hit in the result set, we REFUSE the set with
-- 409 password_breached. The audit row records (subject_id, attempted_at,
-- outcome) — never the password or its hash — so operators can spot
-- breach-pressure patterns.
--
-- ADR: ADR-101-rbac-22-role-catalogue covers the broader auth governance;
--      this migration adds an audit table only (no new role/permission rows).

CREATE TABLE hibp_audit (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id     UUID NOT NULL REFERENCES tenants(id),
    subject_id    UUID,                                          -- nullable for signup before subject exists
    flow          TEXT NOT NULL,                                  -- 'signup' | 'rotation' | 'admin-set'
    outcome       TEXT NOT NULL,                                  -- 'allowed' | 'breached' | 'api-unreachable'
    breach_count  INTEGER,                                        -- HIBP count when outcome=breached; null otherwise
    sha1_prefix   CHAR(5) NOT NULL,                               -- the 5-char prefix sent to HIBP (no PII leak)
    attempted_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT hibp_flow_enum     CHECK (flow IN ('signup', 'rotation', 'admin-set')),
    CONSTRAINT hibp_outcome_enum  CHECK (outcome IN ('allowed', 'breached', 'api-unreachable')),
    CONSTRAINT hibp_breach_consistency CHECK (
        (outcome = 'breached' AND breach_count IS NOT NULL)
        OR (outcome != 'breached' AND breach_count IS NULL)
    )
);

CREATE INDEX hibp_audit_tenant_idx ON hibp_audit (tenant_id, attempted_at DESC);
CREATE INDEX hibp_audit_subject_idx ON hibp_audit (subject_id, attempted_at DESC) WHERE subject_id IS NOT NULL;

ALTER TABLE hibp_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE hibp_audit FORCE ROW LEVEL SECURITY;
CREATE POLICY hibp_audit_tenant_scoped ON hibp_audit
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

GRANT SELECT, INSERT ON hibp_audit TO cyberos_app;
GRANT SELECT ON hibp_audit TO cyberos_ro;
