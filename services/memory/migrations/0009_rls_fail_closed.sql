-- MEM-002 (report R74, F16) — make the brain-table RLS policies FAIL-CLOSED.
--
-- Migrations 0006/0007/0008 shipped each brain-table policy with THREE arms:
--     tenant_id::text = current_setting('app.tenant_id', true)                       -- the real-tenant arm
--  OR current_setting('app.tenant_id', true) = '00000000-0000-0000-0000-000000000000' -- nil-uuid admin bypass
--  OR current_setting('app.tenant_id', true) IS NULL                                 -- unset => FAIL OPEN
--
-- The third arm is the bug (F16): a query path that forgets `brain::tenant_tx` runs with no `app.tenant_id`
-- GUC and therefore reads EVERY tenant's rows instead of none. The second arm is a magic-string bypass that
-- nothing actually needs — every runtime and admin path (the ingest/tiering/summary workers, the recall
-- handler, `cyberos-memory-admin rebuild|brain-*`, and `brain::backfill`) is per-tenant and already sets
-- `app.tenant_id` to a REAL tenant via `tenant_tx`. There is no cross-tenant brain-table reader, so removing
-- the bypass breaks nothing (verified 2026-07-08).
--
-- This migration recreates the four policies with ONLY the real-tenant arm. An unset GUC now yields
-- `tenant_id::text = NULL` -> NULL -> no rows: a forgotten `tenant_tx` reads zero rows, not all rows. ENABLE
-- + FORCE are re-asserted idempotently (FORCE so even the table owner is constrained; superusers still bypass
-- RLS by design, which is why the MEM-002 test uses a non-superuser probe role).
--
-- The nil-uuid GUC is no longer special: setting `app.tenant_id` to the nil uuid now simply scopes to a
-- tenant whose id IS the nil uuid (a normal, empty tenant), not an admin bypass. Cross-tenant admin reads,
-- when they are eventually needed (l1_audit_log / l2_* discovery + metrics), get a dedicated least-privilege
-- role in MEM-003 rather than a magic tenant value.

-- brain_event_embedding -----------------------------------------------------
ALTER TABLE brain_event_embedding ENABLE ROW LEVEL SECURITY;
ALTER TABLE brain_event_embedding FORCE  ROW LEVEL SECURITY;
DROP POLICY IF EXISTS brain_event_isolation ON brain_event_embedding;
CREATE POLICY brain_event_isolation ON brain_event_embedding
    USING (tenant_id::text = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id::text = current_setting('app.tenant_id', true));

-- brain_summary -------------------------------------------------------------
ALTER TABLE brain_summary ENABLE ROW LEVEL SECURITY;
ALTER TABLE brain_summary FORCE  ROW LEVEL SECURITY;
DROP POLICY IF EXISTS brain_summary_isolation ON brain_summary;
CREATE POLICY brain_summary_isolation ON brain_summary
    USING (tenant_id::text = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id::text = current_setting('app.tenant_id', true));

-- brain_ingest_cursor + brain_tier_watermark --------------------------------
DO $$
DECLARE t TEXT;
BEGIN
  FOREACH t IN ARRAY ARRAY['brain_ingest_cursor','brain_tier_watermark'] LOOP
    EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', t);
    EXECUTE format('ALTER TABLE %I FORCE  ROW LEVEL SECURITY', t);
    EXECUTE format('DROP POLICY IF EXISTS %I_isolation ON %I', t, t);
    EXECUTE format(
      'CREATE POLICY %I_isolation ON %I
         USING (tenant_id::text = current_setting(''app.tenant_id'', true))
         WITH CHECK (tenant_id::text = current_setting(''app.tenant_id'', true))', t, t);
  END LOOP;
END $$;
