-- FR-AUTH-106 — login geo / IP history + impossible-travel audit.
--
-- Every successful login (password / OIDC / SAML / Passkey) inserts a row here.
-- The next login checks the previous row's IP + ts; if the network distance +
-- time-delta implies impossible travel, the response carries a
-- `needs_mfa_challenge: true` flag and the client must complete a fresh
-- TOTP / WebAuthn before the access token is honored.
--
-- ADR: ADR-101-rbac-22-role-catalogue (audit + adaptive-MFA layering)

CREATE TABLE login_history_geo (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    subject_id      UUID NOT NULL REFERENCES subjects(id),
    flow            TEXT NOT NULL,                              -- 'password' | 'oidc' | 'saml' | 'passkey'
    ip              INET NOT NULL,
    ip_prefix24     INET NOT NULL,                              -- /24 group for fast same-network checks
    user_agent      TEXT,
    country_iso     CHAR(2),                                    -- ISO 3166-1 alpha-2; null when GeoIP unresolved
    region          TEXT,                                       -- region/state — null when GeoIP unresolved
    lat             DOUBLE PRECISION,                           -- null in slice 1; populated when GeoIP sidecar ships
    lon             DOUBLE PRECISION,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT login_history_flow_enum CHECK (flow IN ('password', 'oidc', 'saml', 'passkey'))
);

CREATE INDEX login_history_subject_idx ON login_history_geo (subject_id, occurred_at DESC);
CREATE INDEX login_history_tenant_idx  ON login_history_geo (tenant_id, occurred_at DESC);

ALTER TABLE login_history_geo ENABLE ROW LEVEL SECURITY;
ALTER TABLE login_history_geo FORCE ROW LEVEL SECURITY;
CREATE POLICY login_history_geo_tenant_scoped ON login_history_geo
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

GRANT SELECT, INSERT ON login_history_geo TO cyberos_app;
GRANT SELECT ON login_history_geo TO cyberos_ro;

-- ---------------------------------------------------------------------------
-- travel_audit — every impossible-travel flag (with detail).
-- ---------------------------------------------------------------------------
CREATE TABLE travel_audit (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID NOT NULL REFERENCES tenants(id),
    subject_id          UUID NOT NULL REFERENCES subjects(id),
    prev_login_id       UUID REFERENCES login_history_geo(id),
    current_login_id    UUID NOT NULL REFERENCES login_history_geo(id),
    detection_kind      TEXT NOT NULL,                          -- 'same_network_burst' | 'cross_continent_velocity' | 'geo_velocity_exceeded'
    delta_seconds       INTEGER NOT NULL,
    detail              JSONB NOT NULL DEFAULT '{}'::jsonb,
    outcome             TEXT NOT NULL,                          -- 'mfa_challenged' | 'mfa_passed' | 'mfa_failed' | 'mfa_skipped_grace'
    occurred_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT travel_detection_kind_enum CHECK (
        detection_kind IN ('same_network_burst', 'cross_continent_velocity', 'geo_velocity_exceeded')
    ),
    CONSTRAINT travel_outcome_enum CHECK (
        outcome IN ('mfa_challenged', 'mfa_passed', 'mfa_failed', 'mfa_skipped_grace')
    )
);

CREATE INDEX travel_audit_subject_idx ON travel_audit (subject_id, occurred_at DESC);

ALTER TABLE travel_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE travel_audit FORCE ROW LEVEL SECURITY;
CREATE POLICY travel_audit_tenant_scoped ON travel_audit
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

GRANT SELECT, INSERT, UPDATE ON travel_audit TO cyberos_app;
GRANT SELECT ON travel_audit TO cyberos_ro;
