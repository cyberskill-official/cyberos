-- FR-PROJ-001 §3 — engagements table.
--
-- Engagements scope ALL project work. Every issue belongs to an engagement
-- (DEC-213 — no orphan issues). client_id is a forward-reference to the
-- CRM clients table; that FK is added in slice 2 once FR-CRM-001 ships.

BEGIN;

CREATE TABLE engagements (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID         NOT NULL,
    client_id   UUID         NULL,                              -- FK to CRM clients (slice 2)
    name        TEXT         NOT NULL CHECK (length(name) BETWEEN 1 AND 200),
    status      TEXT         NOT NULL CHECK (status IN ('active', 'completed', 'paused', 'cancelled')) DEFAULT 'active',
    started_at  DATE         NOT NULL,
    ended_at    DATE         NULL CHECK (ended_at IS NULL OR ended_at >= started_at),
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX engagements_tenant_idx        ON engagements (tenant_id);
CREATE INDEX engagements_tenant_status_idx ON engagements (tenant_id, status);

ALTER TABLE engagements ENABLE ROW LEVEL SECURITY;
ALTER TABLE engagements FORCE ROW LEVEL SECURITY;

CREATE POLICY engagements_tenant_scoped ON engagements
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
