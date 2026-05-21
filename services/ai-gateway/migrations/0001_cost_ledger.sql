-- FR-AI-001: cost-ledger tables for pre-call budget gating.
-- Replaces the placeholder migration.

CREATE TABLE cost_ledger (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       TEXT NOT NULL,
    period          DATE NOT NULL,             -- first-of-month UTC
    spent_usd       NUMERIC(12,4) NOT NULL DEFAULT 0,
    monthly_cap_usd NUMERIC(12,2) NOT NULL,
    UNIQUE (tenant_id, period)
);

CREATE TABLE cost_ledger_hold (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        TEXT NOT NULL,
    idempotency_key  TEXT NOT NULL,
    estimated_usd    NUMERIC(12,4) NOT NULL,
    resolved_provider TEXT NOT NULL,
    resolved_model   TEXT NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at       TIMESTAMPTZ NOT NULL,
    state            TEXT NOT NULL CHECK (state IN ('held','reconciled','expired','refused')),
    UNIQUE (tenant_id, idempotency_key)
);

CREATE INDEX cost_ledger_period_idx ON cost_ledger (tenant_id, period);
CREATE INDEX cost_ledger_hold_expiry_idx ON cost_ledger_hold (expires_at) WHERE state = 'held';
