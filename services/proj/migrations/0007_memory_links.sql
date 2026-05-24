-- FR-PROJ-009 — typed Issue ↔ memory links.

BEGIN;

CREATE TABLE memory_links (
    id                      UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id               UUID         NOT NULL,
    issue_id                UUID         NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    memory_path             TEXT         NOT NULL CHECK (length(memory_path) BETWEEN 1 AND 1000),
    memory_row_id           TEXT,
    link_type               TEXT         NOT NULL CHECK (link_type IN ('cites', 'implements', 'supersedes', 'cites_with_quote')),
    annotation              TEXT         CHECK (annotation IS NULL OR length(annotation) <= 500),
    quoted_text             TEXT         CHECK (quoted_text IS NULL OR length(quoted_text) <= 2048),
    link_strength           TEXT         NOT NULL CHECK (link_strength IN ('weak', 'medium', 'strong')) DEFAULT 'medium',
    review_pending          BOOLEAN      NOT NULL DEFAULT false,
    metadata                JSONB        NOT NULL DEFAULT '{}'::jsonb,
    created_by_subject_id   UUID         NOT NULL,
    created_at              TIMESTAMPTZ  NOT NULL DEFAULT now(),
    removed_at              TIMESTAMPTZ,
    removed_by_subject_id   UUID,
    removal_reason          TEXT,
    CHECK (
        (removed_at IS NULL AND removed_by_subject_id IS NULL AND removal_reason IS NULL)
        OR (removed_at IS NOT NULL AND removed_by_subject_id IS NOT NULL AND length(removal_reason) > 0)
    )
);

CREATE UNIQUE INDEX memory_links_active_unique
    ON memory_links (tenant_id, issue_id, memory_path, link_type)
    WHERE removed_at IS NULL;

CREATE INDEX memory_links_issue_idx ON memory_links (tenant_id, issue_id, created_at DESC);
CREATE INDEX memory_links_memory_idx ON memory_links (tenant_id, memory_path, created_at DESC);
CREATE INDEX memory_links_type_idx ON memory_links (tenant_id, link_type, created_at DESC);

ALTER TABLE memory_links ENABLE ROW LEVEL SECURITY;
ALTER TABLE memory_links FORCE ROW LEVEL SECURITY;

CREATE POLICY memory_links_tenant_scoped ON memory_links
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

COMMIT;
