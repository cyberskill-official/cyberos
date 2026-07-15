-- TASK-PROJ-001 §3 — cycles table.
--
-- A cycle is a time-boxed window within an engagement (sprint-equivalent).
-- The CHECK constraint enforces `ends_at > starts_at` per §1 #12.

BEGIN;

CREATE TABLE cycles (
    id            UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id     UUID         NOT NULL,
    engagement_id UUID         NOT NULL REFERENCES engagements(id) ON DELETE CASCADE,
    name          TEXT         NOT NULL CHECK (length(name) BETWEEN 1 AND 100),
    starts_at     DATE         NOT NULL,
    ends_at       DATE         NOT NULL CHECK (ends_at > starts_at),
    state         TEXT         NOT NULL CHECK (state IN ('upcoming', 'active', 'closed')) DEFAULT 'upcoming',
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX cycles_engagement_idx    ON cycles (tenant_id, engagement_id);
CREATE INDEX cycles_tenant_state_idx  ON cycles (tenant_id, state);

ALTER TABLE cycles ENABLE ROW LEVEL SECURITY;
ALTER TABLE cycles FORCE ROW LEVEL SECURITY;

CREATE POLICY cycles_tenant_scoped ON cycles
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
