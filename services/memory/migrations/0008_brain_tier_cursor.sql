-- FR-MEMORY-123 §3 / §1 #1,#6 — the BRAIN ingest cursor + tier watermark, per tenant.
--
-- brain_ingest_cursor (§1 #1): the last consumed l1_audit_log.seq per tenant. Persisting it to Postgres
-- means a restart resumes without re-embedding or skipping (the ingest worker reads it at the start of each
-- batch and advances it IN THE SAME TRANSACTION as the embedding INSERT, so a crash between the two is
-- impossible — the UPSERT on (tenant_id, source_seq) makes a replay a no-op regardless, §1 #12).
--
-- brain_tier_watermark (§1 #6): the age boundary the tiering pass has advanced to. Events older than
-- (now - hot_max_age) are demoted hot->warm, older than (now - warm_max_age) warm->cold. The watermark
-- makes the tiering pass idempotent — re-running it does not duplicate or lose rows.

CREATE TABLE IF NOT EXISTS brain_ingest_cursor (
    tenant_id       UUID PRIMARY KEY,
    last_source_seq BIGINT NOT NULL DEFAULT 0,           -- 0 = never ingested; resume from last_source_seq + 1
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS brain_tier_watermark (
    tenant_id            UUID PRIMARY KEY,
    last_tiered_ts_ns    BIGINT NOT NULL DEFAULT 0,       -- the high-water occurred-at the tiering pass reached
    last_tier_run_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Both cursor tables are tenant-scoped state; RLS keeps a tenant's cursor invisible to another tenant's
-- transaction (§1 #16) — the same USING/WITH CHECK/FORCE idiom as the embedding + summary tables.
DO $$
DECLARE t TEXT;
BEGIN
  FOREACH t IN ARRAY ARRAY['brain_ingest_cursor','brain_tier_watermark'] LOOP
    EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', t);
    EXECUTE format('ALTER TABLE %I FORCE  ROW LEVEL SECURITY', t);
    EXECUTE format('DROP POLICY IF EXISTS %I_isolation ON %I', t, t);
    EXECUTE format(
      'CREATE POLICY %I_isolation ON %I USING (
         tenant_id::text = current_setting(''app.tenant_id'', true)
         OR current_setting(''app.tenant_id'', true) = ''00000000-0000-0000-0000-000000000000''
         OR current_setting(''app.tenant_id'', true) IS NULL
       ) WITH CHECK (
         tenant_id::text = current_setting(''app.tenant_id'', true)
         OR current_setting(''app.tenant_id'', true) = ''00000000-0000-0000-0000-000000000000''
         OR current_setting(''app.tenant_id'', true) IS NULL
       )', t, t);
  END LOOP;
END $$;

DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'cyberos_app') THEN
    GRANT SELECT, INSERT, UPDATE ON brain_ingest_cursor  TO cyberos_app;
    GRANT SELECT, INSERT, UPDATE ON brain_tier_watermark TO cyberos_app;
  END IF;
END $$;
