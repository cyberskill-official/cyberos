-- TASK-TEN-004 — metering_events substrate (DEC-702 / DEC-704 / DEC-715).

DO $$ BEGIN
  CREATE TYPE metering_axis AS ENUM ('seats', 'api_calls', 'ai_tokens', 'storage_bytes');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
  CREATE TYPE metering_overage_policy AS ENUM ('block', 'warn', 'allow');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
  CREATE TYPE metering_event_state AS ENUM ('active', 'corrected', 'superseded');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS metering_events (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id        TEXT NOT NULL,
  axis             metering_axis NOT NULL,
  quantity         BIGINT NOT NULL,
  unit             TEXT NOT NULL,
  idempotency_key  TEXT NOT NULL,
  source_service   TEXT NOT NULL,
  occurred_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  state            metering_event_state NOT NULL DEFAULT 'active',
  correction_to    UUID NULL REFERENCES metering_events(id),
  memory_chain_hash TEXT NULL,
  extra            JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (tenant_id, axis, idempotency_key)
);

CREATE INDEX IF NOT EXISTS metering_events_tenant_period_idx
  ON metering_events (tenant_id, occurred_at);

DO $$ BEGIN
  REVOKE UPDATE, DELETE ON metering_events FROM cyberos_app;
EXCEPTION WHEN undefined_object THEN NULL;
END $$;
