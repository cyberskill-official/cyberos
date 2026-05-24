-- FR-EMAIL-011 — DSAR export job ledger.

BEGIN;

CREATE TABLE dsar_export_jobs (
    id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID         NOT NULL,
    subject_id          UUID         NOT NULL,
    requested_by        UUID,
    status              TEXT         NOT NULL CHECK (status IN ('queued', 'running', 'completed', 'failed')) DEFAULT 'queued',
    idempotency_key     TEXT         NOT NULL,
    output_jsonl_s3_key TEXT,
    output_sha256       CHAR(64) CHECK (output_sha256 IS NULL OR output_sha256 ~ '^[0-9a-f]{64}$'),
    message_count       BIGINT       NOT NULL DEFAULT 0 CHECK (message_count >= 0),
    attachment_count    BIGINT       NOT NULL DEFAULT 0 CHECK (attachment_count >= 0),
    error_code          TEXT,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    started_at          TIMESTAMPTZ,
    completed_at        TIMESTAMPTZ,
    UNIQUE (tenant_id, subject_id, idempotency_key)
);

CREATE TABLE message_subject_refs (
    tenant_id           UUID         NOT NULL,
    message_id          UUID         NOT NULL REFERENCES message_metadata(id) ON DELETE CASCADE,
    subject_id          UUID         NOT NULL,
    relation            TEXT         NOT NULL CHECK (relation IN ('author', 'recipient', 'cc')),
    address_hash16      CHAR(16)     CHECK (address_hash16 IS NULL OR address_hash16 ~ '^[0-9a-f]{16}$'),
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, message_id, subject_id, relation)
);

CREATE TABLE message_attachment_refs (
    tenant_id           UUID         NOT NULL,
    message_id          UUID         NOT NULL REFERENCES message_metadata(id) ON DELETE CASCADE,
    ordinal             INT          NOT NULL CHECK (ordinal >= 0),
    filename            TEXT         NOT NULL CHECK (length(filename) BETWEEN 1 AND 255),
    s3_key              TEXT         NOT NULL,
    sha256              CHAR(64)     NOT NULL CHECK (sha256 ~ '^[0-9a-f]{64}$'),
    size                BIGINT       NOT NULL CHECK (size >= 0),
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, message_id, ordinal)
);

CREATE INDEX dsar_export_jobs_subject_idx ON dsar_export_jobs (tenant_id, subject_id, created_at DESC);
CREATE INDEX dsar_export_jobs_status_idx ON dsar_export_jobs (tenant_id, status, created_at DESC);
CREATE INDEX message_subject_refs_subject_idx ON message_subject_refs (tenant_id, subject_id, created_at DESC);
CREATE INDEX message_attachment_refs_message_idx ON message_attachment_refs (tenant_id, message_id, ordinal);

ALTER TABLE dsar_export_jobs ENABLE ROW LEVEL SECURITY;
ALTER TABLE dsar_export_jobs FORCE ROW LEVEL SECURITY;
ALTER TABLE message_subject_refs ENABLE ROW LEVEL SECURITY;
ALTER TABLE message_subject_refs FORCE ROW LEVEL SECURITY;
ALTER TABLE message_attachment_refs ENABLE ROW LEVEL SECURITY;
ALTER TABLE message_attachment_refs FORCE ROW LEVEL SECURITY;

CREATE POLICY dsar_export_jobs_tenant_scoped ON dsar_export_jobs
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

CREATE POLICY message_subject_refs_tenant_scoped ON message_subject_refs
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

CREATE POLICY message_attachment_refs_tenant_scoped ON message_attachment_refs
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
        EXECUTE 'REVOKE UPDATE, DELETE ON dsar_export_jobs FROM cyberos_app';
        EXECUTE 'REVOKE UPDATE, DELETE ON message_subject_refs FROM cyberos_app';
        EXECUTE 'REVOKE UPDATE, DELETE ON message_attachment_refs FROM cyberos_app';
    END IF;
END $$;

COMMIT;
