-- TASK-BRAIN-101 — per-tenant ingest cursor (DEC-073).
--
-- Each tenant has its own cursor pointing at the highest L1 seq successfully
-- materialized into l2_memory. On restart, the ingest worker resumes from
-- (last_seq + 1) for that tenant. Per DEC-073: no global cursor; restart
-- is per-tenant idempotent.

CREATE TABLE l2_ingest_cursor (
    tenant_id        UUID PRIMARY KEY,
    last_seq         BIGINT NOT NULL DEFAULT 0,
    last_chain_anchor BYTEA,                        -- for cross-check on next batch
    last_updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- p95 lag observability (§1 #5 — 1s p95 lag floor)
    last_lag_ms      BIGINT NOT NULL DEFAULT 0
);

-- Audit history for cursor advances. Useful for forensics and §C catch-up
-- analysis when the ingest worker crashes mid-batch.
CREATE TABLE l2_ingest_cursor_history (
    history_id       BIGSERIAL PRIMARY KEY,
    tenant_id        UUID NOT NULL,
    from_seq         BIGINT NOT NULL,
    to_seq           BIGINT NOT NULL,
    batch_rows       INTEGER NOT NULL,
    batch_duration_ms BIGINT NOT NULL,
    applied_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX l2_ingest_cursor_history_tenant_idx
    ON l2_ingest_cursor_history (tenant_id, applied_at DESC);
