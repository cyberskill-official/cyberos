-- FR-MEMORY-123 §3 / §1 #4,#5,#9 — the BRAIN rolling-summary table.
--
-- DEC-2724 (summaries-first): one row per (scope_kind, scope_id, version). A summary compacts the events
-- in its window into a short natural-language digest + its own embedding (generated via the SAME ai-gateway
-- path as event embeddings, DEC-2723). Recall queries summaries first and drills into raw hot events only on
-- demand or below the confidence floor — keeping long-term recall compact + cheap as the log grows.
--
-- Versioning (§1 #4): when new events land in an already-summarised window, the worker writes a NEW version
-- and points the prior row's `superseded_by` at it (the prior is RETAINED for audit, never overwritten).
-- Recall reads only the current version (`superseded_by IS NULL`).
--
-- Provenance (§1 #9): `covered_seq_range` is the inclusive source_seq range compacted; `top_contributors`
-- holds the top audit_row_ids so a summary hit can cite exact Layer-1 rows for FR-EVAL-003.

CREATE TABLE IF NOT EXISTS brain_summary (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID   NOT NULL,
    scope_kind          TEXT   NOT NULL CHECK (scope_kind IN ('subject','channel','time_window')),
    scope_id            TEXT   NOT NULL,                 -- subject_id | channel_id | window key (e.g. '2026-W26')
    subject_id          UUID,                            -- the access subject for scope_kind='subject' (NULL otherwise)
    window_start_ns     BIGINT NOT NULL,
    window_end_ns       BIGINT NOT NULL,
    covered_seq_lo      BIGINT NOT NULL,                 -- inclusive low source_seq compacted (§1 #4)
    covered_seq_hi      BIGINT NOT NULL,                 -- inclusive high source_seq compacted
    digest              TEXT   NOT NULL,                 -- short natural-language summary
    embedding           VECTOR(1024),                    -- NULL while pending_summary_retry (§1 #13)
    embed_model_version TEXT   NOT NULL DEFAULT 'unknown',
    version             BIGINT NOT NULL DEFAULT 1,
    superseded_by       UUID,                            -- newer version that replaces this one (NULL = current)
    top_contributors    JSONB  NOT NULL DEFAULT '[]'::jsonb,   -- top audit_row_ids for provenance (§1 #9)
    summary_state       TEXT   NOT NULL DEFAULT 'complete'
                        CHECK (summary_state IN ('complete','pending_summary_retry')),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, scope_kind, scope_id, version)
);

-- summaries-first recall index (§1 #5); current versions only, so the index size is the live summary set.
CREATE INDEX IF NOT EXISTS brain_summary_current_hnsw ON brain_summary
    USING hnsw (embedding vector_cosine_ops) WHERE superseded_by IS NULL AND embedding IS NOT NULL;
CREATE INDEX IF NOT EXISTS brain_summary_scope_idx ON brain_summary (tenant_id, scope_kind, scope_id)
    WHERE superseded_by IS NULL;
-- subject-scoped access filtering on recall (§1 #8) needs a fast subject lookup over current rows.
CREATE INDEX IF NOT EXISTS brain_summary_subject_idx ON brain_summary (tenant_id, subject_id)
    WHERE superseded_by IS NULL AND subject_id IS NOT NULL;

ALTER TABLE brain_summary ENABLE ROW LEVEL SECURITY;
ALTER TABLE brain_summary FORCE  ROW LEVEL SECURITY;
DROP POLICY IF EXISTS brain_summary_isolation ON brain_summary;
CREATE POLICY brain_summary_isolation ON brain_summary
    USING (
        tenant_id::text = current_setting('app.tenant_id', true)
        OR current_setting('app.tenant_id', true) = '00000000-0000-0000-0000-000000000000'
        OR current_setting('app.tenant_id', true) IS NULL
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.tenant_id', true)
        OR current_setting('app.tenant_id', true) = '00000000-0000-0000-0000-000000000000'
        OR current_setting('app.tenant_id', true) IS NULL
    );

DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'cyberos_app') THEN
    -- Summaries supersede by version (a new INSERT + an UPDATE of the prior row's superseded_by); the
    -- runtime role needs INSERT + UPDATE. It never DELETEs (old versions are prunable later by an admin job).
    GRANT SELECT, INSERT, UPDATE ON brain_summary TO cyberos_app;
  END IF;
END $$;
