-- TASK-AUTH-003 — Row-Level Security policies (USING + WITH CHECK).
--
-- AUTHORING_DISCIPLINE §3.4 rule 13: RLS MUST have BOTH `USING` and
-- `WITH CHECK`. `USING` alone protects reads; `WITH CHECK` is required
-- to block cross-tenant INSERTs and UPDATEs.
--
-- The middleware sets `app.current_tenant_id` per request via
-- `SET LOCAL app.current_tenant_id = $1::text`. Postgres then applies
-- the policies below to every query under role `cyberos_app`.

-- ---------------------------------------------------------------------------
-- tenants — special case. The auth service needs to read all tenants
-- (login flow looks up the tenant before issuing a JWT), so the policy
-- is `tenant_id = current OR current = root`. The root tenant has
-- super-admin reach.
-- ---------------------------------------------------------------------------
ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenants FORCE ROW LEVEL SECURITY;  -- enforces RLS even for table owner

CREATE POLICY tenants_select ON tenants
    FOR SELECT
    USING (
        id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) =
           '00000000-0000-0000-0000-000000000000'
    );

CREATE POLICY tenants_modify ON tenants
    FOR ALL
    USING (
        current_setting('app.current_tenant_id', true) =
        '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        current_setting('app.current_tenant_id', true) =
        '00000000-0000-0000-0000-000000000000'
    );

-- ---------------------------------------------------------------------------
-- subjects — standard tenant-scoped RLS. A subject can only see other
-- subjects in the same tenant; only root can cross tenant boundaries.
-- ---------------------------------------------------------------------------
ALTER TABLE subjects ENABLE ROW LEVEL SECURITY;
ALTER TABLE subjects FORCE ROW LEVEL SECURITY;

CREATE POLICY subjects_tenant_scoped ON subjects
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

-- ---------------------------------------------------------------------------
-- admin_idempotency — tenant-scoped, same shape as subjects.
-- ---------------------------------------------------------------------------
ALTER TABLE admin_idempotency ENABLE ROW LEVEL SECURITY;
ALTER TABLE admin_idempotency FORCE ROW LEVEL SECURITY;

CREATE POLICY admin_idempotency_tenant_scoped ON admin_idempotency
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
