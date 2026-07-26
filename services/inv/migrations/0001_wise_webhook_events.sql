-- TASK-INV-004 — Wise webhook events (starts at 0001; prior INV migrations do not exist yet).

DO $$ BEGIN
  CREATE TYPE wise_event_type AS ENUM (
    'transfers_state_change',
    'balances_credit',
    'balances_update'
  );
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
  CREATE TYPE wise_receipt_state AS ENUM (
    'received',
    'matched',
    'currency_mismatch',
    'dead_lettered',
    'manually_resolved'
  );
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS wise_webhook_events (
  id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  profile_id           BIGINT NOT NULL,
  event_id             UUID NOT NULL,
  event_type           wise_event_type NOT NULL,
  state                wise_receipt_state NOT NULL DEFAULT 'received',
  body                 JSONB NOT NULL,
  webhook_received_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  webhook_processed_at TIMESTAMPTZ NULL,
  retry_count          INT NOT NULL DEFAULT 0,
  UNIQUE (profile_id, event_id)
);

CREATE TABLE IF NOT EXISTS unmatched_receipts (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       TEXT NOT NULL,
  source          TEXT NOT NULL DEFAULT 'wise',
  source_event_id UUID NOT NULL,
  currency        TEXT NOT NULL,
  amount_minor    BIGINT NOT NULL,
  occurred_at     TIMESTAMPTZ NOT NULL,
  notes           TEXT NOT NULL DEFAULT '',
  resolved_at     TIMESTAMPTZ NULL,
  resolved_by     TEXT NULL
);

DO $$ BEGIN
  REVOKE UPDATE, DELETE ON wise_webhook_events FROM cyberos_app;
EXCEPTION WHEN undefined_object THEN NULL;
END $$;
