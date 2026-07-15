-- TASK-AUTH-106 slice-3 — per-tenant impossible-travel policy + CIDR allowlist.
--
-- Three tables:
--   travel_policy           — one row per tenant, default-row inserted lazily.
--   travel_cidr_allowlist   — CIDRs that short-circuit the detector (office IPs).
--   travel_policy_audit     — append-only history of policy mutations.
--
-- Policy mutations are gated by the `security_admin` role at the handler
-- layer; the migration just shapes the tables.

CREATE TABLE travel_policy (
    tenant_id           UUID PRIMARY KEY REFERENCES tenants(id),
    -- Action when impossible travel is detected:
    --   'challenge' — issue token + return needs_mfa_challenge=true
    --   'block'     — return 403 immediately
    --   'warn_only' — issue token, log audit row, no challenge
    action              TEXT NOT NULL DEFAULT 'challenge',
    -- Speed threshold in km/h. The hard-coded default in code is 1000;
    -- per-tenant override accepted in [200, 5000].
    threshold_kmh       DOUBLE PRECISION NOT NULL DEFAULT 1000.0,
    -- VPN/Tor handling — when TRUE, an anonymous-IP login is auto-blocked
    -- even when no prior login exists. Off by default because many legit
    -- users come from corporate VPNs.
    block_anonymous_ip  BOOLEAN NOT NULL DEFAULT FALSE,
    -- Sticky-challenge suppression window (minutes). After a successful
    -- MFA challenge for a (subject, /24), repeat logins from the same /24
    -- within this window skip re-challenging.
    sticky_suppress_min INTEGER NOT NULL DEFAULT 30,
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT travel_policy_action_enum CHECK (
        action IN ('challenge', 'block', 'warn_only')
    ),
    CONSTRAINT travel_policy_threshold_range CHECK (
        threshold_kmh BETWEEN 200.0 AND 5000.0
    ),
    CONSTRAINT travel_policy_sticky_range CHECK (
        sticky_suppress_min BETWEEN 0 AND 1440
    )
);

ALTER TABLE travel_policy ENABLE ROW LEVEL SECURITY;
ALTER TABLE travel_policy FORCE ROW LEVEL SECURITY;
CREATE POLICY travel_policy_tenant_scoped ON travel_policy
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

GRANT SELECT, INSERT, UPDATE ON travel_policy TO cyberos_app;
GRANT SELECT ON travel_policy TO cyberos_ro;

-- ---------------------------------------------------------------------------
-- CIDR allowlist — when a login's IP matches one of these CIDRs the detector
-- is short-circuited to Clear (no travel_audit row written).
-- ---------------------------------------------------------------------------
CREATE TABLE travel_cidr_allowlist (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    cidr            INET NOT NULL,
    label           TEXT NOT NULL,        -- 'sg-office', 'home-team-vpn', etc.
    added_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    added_by        UUID REFERENCES subjects(id),

    -- TASK-AUTH-106 §1 #X — minimum-prefix tightness: only allow CIDRs at or
    -- tighter than /9 for IPv4 and /17 for IPv6. Prevents a misconfigured
    -- "allow the whole internet" entry.
    CONSTRAINT travel_cidr_prefix_tight CHECK (
        (family(cidr) = 4 AND masklen(cidr) >= 9)
        OR (family(cidr) = 6 AND masklen(cidr) >= 17)
    )
);

CREATE INDEX travel_cidr_allowlist_tenant_idx ON travel_cidr_allowlist (tenant_id);

ALTER TABLE travel_cidr_allowlist ENABLE ROW LEVEL SECURITY;
ALTER TABLE travel_cidr_allowlist FORCE ROW LEVEL SECURITY;
CREATE POLICY travel_cidr_allowlist_tenant_scoped ON travel_cidr_allowlist
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

GRANT SELECT, INSERT, DELETE ON travel_cidr_allowlist TO cyberos_app;
GRANT SELECT ON travel_cidr_allowlist TO cyberos_ro;

-- ---------------------------------------------------------------------------
-- travel_policy_audit — append-only history. Every UPDATE to travel_policy
-- + every CIDR add/remove writes a row here, with the actor + reason.
-- ---------------------------------------------------------------------------
CREATE TABLE travel_policy_audit (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    actor_id        UUID NOT NULL REFERENCES subjects(id),
    change_kind     TEXT NOT NULL,
    detail          JSONB NOT NULL DEFAULT '{}'::jsonb,
    reason          TEXT NOT NULL,
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT travel_policy_audit_kind_enum CHECK (
        change_kind IN (
            'policy_updated',
            'cidr_added',
            'cidr_removed'
        )
    ),
    CONSTRAINT travel_policy_audit_reason_min CHECK (length(reason) >= 10)
);

CREATE INDEX travel_policy_audit_tenant_idx
    ON travel_policy_audit (tenant_id, occurred_at DESC);

ALTER TABLE travel_policy_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE travel_policy_audit FORCE ROW LEVEL SECURITY;
CREATE POLICY travel_policy_audit_tenant_scoped ON travel_policy_audit
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

GRANT SELECT, INSERT ON travel_policy_audit TO cyberos_app;
GRANT SELECT ON travel_policy_audit TO cyberos_ro;
