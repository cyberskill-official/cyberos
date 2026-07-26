-- TASK-TEN-002 — append-only tenant_plan_history (DEC-775 / DEC-776).

CREATE TABLE IF NOT EXISTS tenant_plan_history (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       TEXT NOT NULL,
  from_tier       plan_tier NOT NULL,
  to_tier         plan_tier NOT NULL,
  actor_id        TEXT NOT NULL,
  occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  effective_at    TIMESTAMPTZ NOT NULL,
  proration_amount_cents BIGINT NOT NULL DEFAULT 0,
  reason          TEXT NOT NULL DEFAULT '',
  effective       plan_change_effective NOT NULL DEFAULT 'immediate'
);

CREATE INDEX IF NOT EXISTS tenant_plan_history_tenant_occurred_idx
  ON tenant_plan_history (tenant_id, occurred_at DESC);

-- Append-only: app role may INSERT/SELECT but not UPDATE/DELETE.
DO $$ BEGIN
  REVOKE UPDATE, DELETE ON tenant_plan_history FROM cyberos_app;
EXCEPTION WHEN undefined_object THEN NULL;
END $$;
