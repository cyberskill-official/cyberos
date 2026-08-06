-- TASK-DATA-103 — Least-privilege application role for the finance ledger.
--
-- Context. The fin_* tables (fin_transactions, fin_lookups, fin_imported_pdfs,
-- fin_bank_import_lines) were created directly against the CyberOS Supabase
-- database and have never been under version control. This is the first fin
-- migration; it does not attempt to define those tables, only to fix how the
-- application connects to them. A schema baseline follows separately.
--
-- Problem. The finance app connects as `postgres`, which holds rolbypassrls.
-- Row Level Security is enabled AND forced on all four tables, with correct
-- tenant-scoped policies carrying both USING and WITH CHECK, and none of it
-- applies, because BYPASSRLS exempts the role from row security entirely.
-- Proven on 2026-08-05: with app.current_tenant_id set to a foreign tenant, a
-- query still returned another tenant's row.
--
-- Tenant isolation is therefore enforced today only by every query in
-- lib/db_pg.js remembering to filter on tenant_id. That is application
-- discipline, not a database boundary, and staging is about to share this
-- database separated only by tenant id.
--
-- This migration:
--   1. creates a finance_app login role WITHOUT BYPASSRLS or SUPERUSER
--   2. grants it exactly what the application needs, nothing more
--   3. removes the nil-UUID escape hatch from the four tenant policies
--
-- The password is NOT set here. Set it out of band so it never enters version
-- control:
--     ALTER ROLE finance_app WITH PASSWORD '<generated>';
--
-- Rollback: point CFO_DATABASE_URL back at the previous role. This migration
-- adds a role and tightens policies; it drops no data and no columns. To fully
-- revert the policy change, re-add the nil-UUID clause shown in the comment at
-- the foot of this file.

BEGIN;

-- 1. Role -------------------------------------------------------------------

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'finance_app') THEN
        -- NOBYPASSRLS is deliberately NOT written here. Only a superuser may set
        -- that attribute explicitly, and the Supabase `postgres` role is not one
        -- (rolsuper = false, rolcreaterole = true). The PostgreSQL default for a
        -- new role is NOBYPASSRLS, which is what we want; naming it would make
        -- this migration fail with "permission denied to alter role".
        CREATE ROLE finance_app WITH LOGIN NOSUPERUSER
            NOCREATEDB NOCREATEROLE NOINHERIT;
    END IF;
END $$;

-- Assert the security property rather than assuming it. If this role ever ends
-- up with BYPASSRLS, every tenant policy below becomes decorative, which is the
-- exact defect this migration exists to fix. Fail loudly instead.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'finance_app' AND rolbypassrls) THEN
        RAISE EXCEPTION
            'finance_app has BYPASSRLS; tenant policies would not apply. A superuser must run: ALTER ROLE finance_app WITH NOBYPASSRLS;';
    END IF;
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'finance_app' AND rolsuper) THEN
        RAISE EXCEPTION 'finance_app is a superuser; refusing to proceed.';
    END IF;
END $$;

-- 2. Grants -----------------------------------------------------------------

GRANT CONNECT ON DATABASE postgres TO finance_app;
GRANT USAGE ON SCHEMA public TO finance_app;

GRANT SELECT, INSERT, UPDATE, DELETE ON
    fin_transactions,
    fin_lookups,
    fin_imported_pdfs,
    fin_bank_import_lines
TO finance_app;

-- No table creation, no DDL, no access to any other table in the schema.
-- Deliberately NOT granted: ALL PRIVILEGES, ownership, TRUNCATE, REFERENCES,
-- or any default privileges on future objects.

-- 3. Remove the nil-UUID escape hatch ---------------------------------------
--
-- The existing policies read:
--     (tenant_id::text = current_setting('app.current_tenant_id', true))
--     OR (current_setting('app.current_tenant_id', true)
--         = '00000000-0000-0000-0000-000000000000')
--
-- CFO_TENANT_ID is validated only as UUID-shaped (lib/pg.js), and the nil UUID
-- is UUID-shaped, so that second clause is reachable by configuration alone and
-- grants cross-tenant read and write. Removing it leaves a single, honest rule.

DROP POLICY IF EXISTS fin_transactions_tenant_scoped ON fin_transactions;
CREATE POLICY fin_transactions_tenant_scoped ON fin_transactions
    FOR ALL
    USING (tenant_id::text = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id::text = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS fin_lookups_tenant_scoped ON fin_lookups;
CREATE POLICY fin_lookups_tenant_scoped ON fin_lookups
    FOR ALL
    USING (tenant_id::text = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id::text = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS fin_imported_pdfs_tenant_scoped ON fin_imported_pdfs;
CREATE POLICY fin_imported_pdfs_tenant_scoped ON fin_imported_pdfs
    FOR ALL
    USING (tenant_id::text = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id::text = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS fin_bank_import_lines_tenant_scoped ON fin_bank_import_lines;
CREATE POLICY fin_bank_import_lines_tenant_scoped ON fin_bank_import_lines
    FOR ALL
    USING (tenant_id::text = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id::text = current_setting('app.current_tenant_id', true));

COMMIT;

-- Verification (run as finance_app, not as postgres):
--
--   BEGIN;
--   SELECT set_config('app.current_tenant_id','<foreign-uuid>', true);
--   SELECT count(*) FROM fin_transactions;   -- must be 0
--   COMMIT;
--
-- If that returns rows, the role still bypasses RLS and the migration has not
-- achieved its purpose.
