-- FR-BRAIN-101 — Layer-1 audit-log mirror.
--
-- The Personal-BRAIN client (modules/memory/) writes its append-only chain
-- to `~/.cyberos-memory/audit/*.ndjson`. The brain-sync daemon (FR-BRAIN-103)
-- pushes those rows into Cloud BRAIN, which writes them here. Layer-2
-- ingest then tails this table.
--
-- Note: this table is the "binlog" the layer2 ingest worker tails. In a
-- pure-personal deployment (no Cloud BRAIN), the worker can tail the
-- on-disk ndjson directly via `binlog_tail::poll_from_disk`. The two
-- paths converge on the same row shape.

CREATE TABLE l1_audit_log (
    seq             BIGSERIAL PRIMARY KEY,
    tenant_id       UUID NOT NULL,
    subject_id      UUID,                                 -- nullable for system rows
    op              TEXT NOT NULL,                        -- 'put' | 'move' | 'delete' | 'view'
    path            TEXT NOT NULL,                        -- canonical store path
    body            TEXT,                                 -- markdown body (for put)
    prev_hash_hex   TEXT,                                 -- 64-char SHA-256 hex of prev row
    chain_anchor_hex TEXT NOT NULL,                       -- SHA-256(prev_hash ‖ body)
    ts_ns           BIGINT NOT NULL,                      -- timestamp from L1 (ns since epoch)
    ingested_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT l1_op_enum CHECK (op IN ('put', 'move', 'delete', 'view'))
);

-- Tenant + monotonic seq is the natural cursor key.
CREATE INDEX l1_audit_log_tenant_seq_idx ON l1_audit_log (tenant_id, seq);
CREATE INDEX l1_audit_log_ingested_idx   ON l1_audit_log (ingested_at DESC);

-- Grants for the layer2 ingest worker.
GRANT SELECT ON l1_audit_log TO cyberos_app;
GRANT INSERT ON l1_audit_log TO cyberos_app;
GRANT USAGE, SELECT ON SEQUENCE l1_audit_log_seq_seq TO cyberos_app;
