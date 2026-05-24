-- FR-PROJ-002 — memory-anchored decision rows for issue state changes.

BEGIN;

CREATE TABLE proj_decisions (
    id                      UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id               UUID         NOT NULL,
    issue_id                UUID         NOT NULL REFERENCES issues(id) ON DELETE RESTRICT,
    from_status             TEXT         NOT NULL CHECK (from_status IN ('triage', 'todo', 'doing', 'review', 'done', 'deleted')),
    to_status               TEXT         NOT NULL CHECK (to_status IN ('triage', 'todo', 'doing', 'review', 'done', 'deleted')),
    reason                  TEXT         CHECK (reason IS NULL OR length(reason) <= 500),
    decided_by_subject_id   UUID         NOT NULL,
    prior_decision_chain    TEXT         CHECK (prior_decision_chain IS NULL OR prior_decision_chain ~ '^[0-9a-f]{64}$'),
    cross_module_links      TEXT[]       NOT NULL DEFAULT '{}',
    request_id              TEXT         NOT NULL,
    sync_class              TEXT         NOT NULL CHECK (sync_class IN ('private', 'shareable')) DEFAULT 'shareable',
    acl                     TEXT[]       NOT NULL DEFAULT '{}',
    decision_session_id     UUID         NOT NULL,
    decision_attributes     JSONB        NOT NULL DEFAULT '{}'::jsonb,
    memory_chain_hash       TEXT         CHECK (memory_chain_hash IS NULL OR memory_chain_hash ~ '^[0-9a-f]{64}$'),
    chain_anchor_in_payload TEXT         CHECK (chain_anchor_in_payload IS NULL OR chain_anchor_in_payload ~ '^[0-9a-f]{64}$'),
    retracted_at            TIMESTAMPTZ,
    retracted_by_subject_id UUID,
    retraction_reason       TEXT,
    created_at              TIMESTAMPTZ  NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, issue_id, request_id)
);

CREATE TABLE proj_decision_retractions (
    id                      UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id               UUID         NOT NULL,
    retracts_decision_id    UUID         NOT NULL REFERENCES proj_decisions(id) ON DELETE RESTRICT,
    retraction_reason       TEXT         NOT NULL CHECK (length(retraction_reason) BETWEEN 1 AND 500),
    retracted_by_subject_id UUID         NOT NULL,
    request_id              TEXT         NOT NULL,
    created_at              TIMESTAMPTZ  NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, retracts_decision_id, request_id)
);

CREATE INDEX proj_decisions_issue_idx ON proj_decisions (tenant_id, issue_id, created_at DESC);
CREATE INDEX proj_decisions_chain_idx ON proj_decisions (tenant_id, memory_chain_hash) WHERE memory_chain_hash IS NOT NULL;
CREATE INDEX proj_decision_retractions_idx ON proj_decision_retractions (tenant_id, retracts_decision_id, created_at DESC);

ALTER TABLE proj_decisions ENABLE ROW LEVEL SECURITY;
ALTER TABLE proj_decisions FORCE ROW LEVEL SECURITY;
ALTER TABLE proj_decision_retractions ENABLE ROW LEVEL SECURITY;
ALTER TABLE proj_decision_retractions FORCE ROW LEVEL SECURITY;

CREATE POLICY proj_decisions_tenant_scoped ON proj_decisions
    FOR ALL
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.current_tenant_id', true) = '00000000-0000-0000-0000-000000000000'
    );

CREATE POLICY proj_decision_retractions_tenant_scoped ON proj_decision_retractions
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
