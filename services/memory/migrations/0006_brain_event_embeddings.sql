-- FR-MEMORY-123 §3 / §1 #2,#3,#6,#16 — the BRAIN event-embedding table.
--
-- DEC-2721: l1_audit_log (the hash chain) stays the SYSTEM OF RECORD. This table is a DERIVED,
-- rebuildable fast lens over the FR-MEMORY-121 interaction-event rows on that chain — never authoritative,
-- always reconstructable from Layer 1 by `cyberos-memory-admin brain --rebuild`. Layer 1 wins on any
-- conflict; a row found inconsistent with its Layer-1 source is flagged `stale` for re-ingest, never trusted
-- over the chain.
--
-- The cursor key is l1_audit_log.seq (the per-tenant monotonic seq the FR-MEMORY-121 emitters write).
-- `chain_anchor` mirrors l2_memory: BYTEA = SHA-256(prev_hash_hex_bytes || body_bytes), the same anchor
-- compute layer2/chain_anchor.rs uses, so read-time verify (§1 #10) recomputes against the live Layer-1 row.
--
-- The pgvector extension is loaded by services/dev/postgres-init.sql at container init; the explicit
-- CREATE here keeps a fresh database / a non-dev deploy honest (§10 "pgvector extension missing" failure
-- mode — the service refuses to start if this fails).

CREATE EXTENSION IF NOT EXISTS vector;

-- ---------------------------------------------------------------------------
-- brain_event_embedding — one row per ingested interaction-event (§1 #2).
-- Idempotent on (tenant_id, source_seq) so a restart-mid-batch re-ingest is a no-op (§1 #12).
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS brain_event_embedding (
    tenant_id           UUID   NOT NULL,                 -- FR-AUTH-003 tenant scope (RLS below)
    source_seq          BIGINT NOT NULL,                 -- l1_audit_log.seq — the cursor key (§1 #1)
    audit_row_id        TEXT   NOT NULL,                 -- provenance pointer into l1_audit_log (§1 #9, #26)
    subject_id          UUID   NOT NULL,                 -- whose interaction (FR-EVAL-001 access subject)
    channel_id          UUID,                            -- where (chat channel / module surface), nullable
    kind                TEXT   NOT NULL,                 -- interaction event kind from FR-MEMORY-121 payload
    ts_ns               BIGINT NOT NULL,                 -- occurred-at ns (from the Layer-1 row)
    embedding           VECTOR(1024),                    -- bge-m3 dims; NULL while pending_embed_retry (§1 #13)
    embed_model_version TEXT   NOT NULL DEFAULT 'unknown',-- for re-embed migrations (§1 #14)
    chain_anchor        BYTEA  NOT NULL,                 -- SHA-256(prev_hash || body) for read-time verify (§1 #10)
    chain_anchor_hex    TEXT GENERATED ALWAYS AS (encode(chain_anchor, 'hex')) STORED,
    tier                TEXT   NOT NULL DEFAULT 'hot'
                        CHECK (tier IN ('hot','warm','cold')),
    embed_state         TEXT   NOT NULL DEFAULT 'complete'
                        CHECK (embed_state IN ('complete','pending_embed_retry')),
    stale               BOOL   NOT NULL DEFAULT FALSE,   -- flagged when Layer 1 diverges (§1 #11); re-ingest target
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, source_seq)
);

-- hot-tier HNSW index for sub-second semantic recall (§1 #3); partial so the index — and every query's
-- cost — is bounded by the hot window, not the lifetime log (§1 #6). cosine ops match the query distance.
CREATE INDEX IF NOT EXISTS brain_event_embedding_hot_hnsw ON brain_event_embedding
    USING hnsw (embedding vector_cosine_ops) WHERE tier = 'hot' AND embedding IS NOT NULL;
CREATE INDEX IF NOT EXISTS brain_event_embedding_subject_idx
    ON brain_event_embedding (tenant_id, subject_id, ts_ns DESC);
CREATE INDEX IF NOT EXISTS brain_event_embedding_tier_idx
    ON brain_event_embedding (tenant_id, tier);
CREATE INDEX IF NOT EXISTS brain_event_embedding_audit_row_idx
    ON brain_event_embedding (tenant_id, audit_row_id);
CREATE INDEX IF NOT EXISTS brain_event_embedding_pending_idx
    ON brain_event_embedding (tenant_id)
    WHERE embed_state = 'pending_embed_retry';

-- §1 #16 — tenant isolation via RLS (FR-AUTH-003 pattern). The recall handler sets app.tenant_id per
-- transaction; the nil tenant is an admin bypass for the rebuild/backfill paths, mirroring eval's
-- governance migration. USING + WITH CHECK + FORCE so even the table owner is constrained.
ALTER TABLE brain_event_embedding ENABLE ROW LEVEL SECURITY;
ALTER TABLE brain_event_embedding FORCE  ROW LEVEL SECURITY;
DROP POLICY IF EXISTS brain_event_isolation ON brain_event_embedding;
CREATE POLICY brain_event_isolation ON brain_event_embedding
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

-- Grants for the brain ingest worker + recall handler (runtime role). The worker never writes Layer 1.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'cyberos_app') THEN
    GRANT SELECT, INSERT, UPDATE ON brain_event_embedding TO cyberos_app;
  END IF;
END $$;
