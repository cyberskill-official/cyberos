-- FR-AUTH-005 — local L1 audit bridge for auth-originated memory rows.
--
-- AUTH handlers emit auth.* audit rows transactionally with the subject
-- mutation. In separate-database deployments this table must exist in the AUTH
-- database; MEMORY owns the ingestion/projection service, but AUTH owns the
-- source event transaction.

CREATE TABLE IF NOT EXISTS l1_audit_log (
    seq              BIGSERIAL PRIMARY KEY,
    tenant_id        UUID NOT NULL,
    subject_id       UUID,
    op               TEXT NOT NULL,
    path             TEXT NOT NULL,
    body             TEXT,
    prev_hash_hex    TEXT,
    chain_anchor_hex TEXT NOT NULL,
    ts_ns            BIGINT NOT NULL,
    ingested_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT l1_op_enum CHECK (op IN ('put', 'move', 'delete', 'view'))
);

CREATE INDEX IF NOT EXISTS l1_audit_log_tenant_seq_idx
    ON l1_audit_log (tenant_id, seq);
CREATE INDEX IF NOT EXISTS l1_audit_log_ingested_idx
    ON l1_audit_log (ingested_at DESC);

GRANT SELECT ON l1_audit_log TO cyberos_app;
GRANT INSERT ON l1_audit_log TO cyberos_app;
GRANT USAGE, SELECT ON SEQUENCE l1_audit_log_seq_seq TO cyberos_app;
