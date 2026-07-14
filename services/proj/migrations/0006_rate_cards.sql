-- TASK-PROJ-005 — append-only per-engagement rate cards.

BEGIN;

CREATE TABLE rate_cards (
    id                      UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id               UUID         NOT NULL,
    engagement_id           UUID         NOT NULL REFERENCES engagements(id) ON DELETE RESTRICT,
    role                    TEXT         NOT NULL CHECK (role IN ('engineer', 'designer', 'pm', 'qa', 'analyst', 'exec')),
    currency                TEXT         NOT NULL CHECK (currency IN ('VND', 'USD', 'SGD', 'EUR', 'JPY')),
    hourly_rate_minor       BIGINT       NOT NULL CHECK (hourly_rate_minor >= 0),
    billable_default        BOOLEAN      NOT NULL DEFAULT true,
    effective_from          DATE         NOT NULL,
    effective_to            DATE,
    archived                BOOLEAN      NOT NULL DEFAULT false,
    supersedes_rate_card_id UUID         REFERENCES rate_cards(id) ON DELETE RESTRICT,
    created_by_subject_id   UUID         NOT NULL,
    created_at              TIMESTAMPTZ  NOT NULL DEFAULT now(),
    CHECK (effective_to IS NULL OR effective_to > effective_from)
);

CREATE UNIQUE INDEX rate_cards_active_unique
    ON rate_cards (tenant_id, engagement_id, role, currency)
    WHERE effective_to IS NULL AND archived = false;

CREATE INDEX rate_cards_lookup_idx
    ON rate_cards (tenant_id, engagement_id, role, currency, effective_from DESC);

ALTER TABLE rate_cards ENABLE ROW LEVEL SECURITY;
ALTER TABLE rate_cards FORCE ROW LEVEL SECURITY;

CREATE POLICY rate_cards_tenant_scoped ON rate_cards
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
