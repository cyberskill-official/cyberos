-- FR-AUTH-003 — RLS role + per-connection tenant pin.
--
-- The `cyberos_app` role is the runtime app's database identity. Every
-- query runs as this role with `SET LOCAL app.current_tenant_id = '<uuid>'`
-- set per connection by the auth middleware. RLS policies in 0005 use that
-- GUC to filter every tenant-scoped table.
--
-- Migration idempotency: wrap CREATE ROLE in DO blocks so re-runs against
-- a database that already has the role succeed silently. CI environments
-- + dev rollbacks re-apply migrations against state that may already have
-- some objects.

DO $$
BEGIN
    CREATE ROLE cyberos_app NOLOGIN;
EXCEPTION
    WHEN duplicate_object THEN NULL;
END
$$;

GRANT USAGE ON SCHEMA public TO cyberos_app;

-- Future-table-default: every new table created under `public` schema by
-- the app role should grant SELECT/INSERT/UPDATE/DELETE to cyberos_app.
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO cyberos_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT USAGE, SELECT ON SEQUENCES TO cyberos_app;

-- A separate read-only role for analytics / OBS dashboards / DSAR exports.
-- Cannot mutate; cannot read across tenants (RLS still applies).
DO $$
BEGIN
    CREATE ROLE cyberos_ro NOLOGIN;
EXCEPTION
    WHEN duplicate_object THEN NULL;
END
$$;

GRANT USAGE ON SCHEMA public TO cyberos_ro;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT SELECT ON TABLES TO cyberos_ro;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT SELECT ON SEQUENCES TO cyberos_ro;

-- Backfill grants for the tables already created (tenants, subjects, admin_idempotency).
-- GRANT is naturally idempotent; no DO-block wrapping needed.
GRANT SELECT, INSERT, UPDATE, DELETE ON tenants            TO cyberos_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON subjects           TO cyberos_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON admin_idempotency  TO cyberos_app;
GRANT SELECT ON tenants, subjects, admin_idempotency TO cyberos_ro;
