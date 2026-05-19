-- FR-PROJ-001 §3 — issues table.
--
-- Per DEC-210, status is the 5-value closed enum; `deleted` is a reserved
-- 6th value used by the soft-delete API path (root-admin only — §4 #17).
-- Per DEC-211, priority is 4-value closed enum.

BEGIN;

CREATE TABLE issues (
    id                  UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID         NOT NULL,
    engagement_id       UUID         NOT NULL REFERENCES engagements(id) ON DELETE RESTRICT,
    cycle_id            UUID         NULL REFERENCES cycles(id) ON DELETE SET NULL,
    title               TEXT         NOT NULL CHECK (length(title) BETWEEN 1 AND 200),
    body                TEXT         NULL CHECK (length(coalesce(body, '')) <= 50000),
    status              TEXT         NOT NULL CHECK (status IN ('triage', 'todo', 'doing', 'review', 'done', 'deleted')) DEFAULT 'triage',
    priority            TEXT         NOT NULL CHECK (priority IN ('urgent', 'high', 'normal', 'low')) DEFAULT 'normal',
    assignee_subject_id UUID         NULL,
    estimate_hours      DOUBLE PRECISION NULL CHECK (estimate_hours IS NULL OR (estimate_hours > 0 AND estimate_hours <= 9999.99)),
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- Indexes per spec §3 ---------------------------------------------------------
CREATE INDEX issues_engagement_idx ON issues (tenant_id, engagement_id);
CREATE INDEX issues_assignee_idx   ON issues (tenant_id, assignee_subject_id) WHERE assignee_subject_id IS NOT NULL;
CREATE INDEX issues_status_idx     ON issues (tenant_id, status);
CREATE INDEX issues_cycle_idx      ON issues (tenant_id, cycle_id) WHERE cycle_id IS NOT NULL;

ALTER TABLE issues ENABLE ROW LEVEL SECURITY;
ALTER TABLE issues FORCE ROW LEVEL SECURITY;

CREATE POLICY issues_tenant_scoped ON issues
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

-- Auto-update `updated_at` on row UPDATE — supports the optimistic-lock
-- pattern in §1 #13 (If-Match header → 412 if updated_at differs).
CREATE OR REPLACE FUNCTION proj_issues_set_updated_at() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END $$ LANGUAGE plpgsql;

CREATE TRIGGER issues_updated_at_trg
    BEFORE UPDATE ON issues
    FOR EACH ROW
    WHEN (OLD.* IS DISTINCT FROM NEW.*)
    EXECUTE FUNCTION proj_issues_set_updated_at();

COMMIT;
