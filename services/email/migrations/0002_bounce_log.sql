-- TASK-EMAIL-001 §3.3 — bounce_log table.
--
-- Bounces are recorded as pure inserts. Hard/soft/transient classification
-- per RFC 3463 SMTP enhanced status codes. The reputation alarm consumes
-- this table via the OTel exporter (TASK-EMAIL-001 §1 #17).

BEGIN;

CREATE TABLE bounce_log (
    id              BIGSERIAL    PRIMARY KEY,
    tenant_id       UUID         NOT NULL,
    message_id      UUID         NOT NULL REFERENCES message_metadata(id) ON DELETE RESTRICT,
    bounce_kind     TEXT         NOT NULL CHECK (bounce_kind IN ('hard', 'soft', 'transient')),
    bounce_reason   TEXT         NOT NULL CHECK (length(bounce_reason) BETWEEN 1 AND 2000),
    bounce_code     TEXT         CHECK (bounce_code IS NULL OR bounce_code ~ '^[0-9]{3}( [0-9]+\.[0-9]+\.[0-9]+)?$'),
    remote_peer     TEXT,
    ts              TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX bounce_log_tenant_ts_idx    ON bounce_log (tenant_id, ts DESC);
CREATE INDEX bounce_log_message_idx      ON bounce_log (message_id);
CREATE INDEX bounce_log_tenant_kind_idx  ON bounce_log (tenant_id, bounce_kind, ts DESC);

ALTER TABLE bounce_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE bounce_log FORCE ROW LEVEL SECURITY;

CREATE POLICY bounce_log_tenant_scoped ON bounce_log
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'cyberos_app') THEN
        EXECUTE 'REVOKE UPDATE, DELETE ON bounce_log FROM cyberos_app';
    END IF;
END $$;

COMMIT;
