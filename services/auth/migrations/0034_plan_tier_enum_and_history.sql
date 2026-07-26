-- TASK-TEN-203 — cut over tenants.plan_tier TEXT(+sandbox) → closed plan_tier enum,
-- ensure founder/history substrate, and install P0301 same-TX history guard.

DO $$ BEGIN
  CREATE TYPE plan_tier AS ENUM ('starter', 'team', 'enterprise');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
  CREATE TYPE plan_change_effective AS ENUM ('immediate', 'next_period', 'defer_billing_only');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

-- Relocate legacy sandbox rows before enum cast (DEC-771 closed set of 3).
UPDATE tenants SET plan_tier = 'starter' WHERE plan_tier = 'sandbox';

ALTER TABLE tenants DROP CONSTRAINT IF EXISTS tenants_plan_enum;

-- Cast TEXT → enum (idempotent when already enum).
DO $$ BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_name = 'tenants' AND column_name = 'plan_tier'
      AND udt_name = 'text'
  ) THEN
    ALTER TABLE tenants
      ALTER COLUMN plan_tier DROP DEFAULT,
      ALTER COLUMN plan_tier TYPE plan_tier USING plan_tier::plan_tier,
      ALTER COLUMN plan_tier SET DEFAULT 'starter'::plan_tier;
  END IF;
END $$;

ALTER TABLE tenants
  ADD COLUMN IF NOT EXISTS is_founder_tenant BOOLEAN NOT NULL DEFAULT false;

ALTER TABLE tenants
  ADD COLUMN IF NOT EXISTS metering_caps_yaml TEXT;

ALTER TABLE tenants
  ADD COLUMN IF NOT EXISTS plan_effective_since TIMESTAMPTZ NOT NULL DEFAULT now();

CREATE TABLE IF NOT EXISTS tenant_plan_history (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id       UUID NOT NULL REFERENCES tenants(id),
  from_tier       plan_tier NOT NULL,
  to_tier         plan_tier NOT NULL,
  actor_id        UUID NOT NULL,
  occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  effective_at    TIMESTAMPTZ NOT NULL,
  proration_amount_cents BIGINT NOT NULL DEFAULT 0,
  reason          TEXT NOT NULL DEFAULT '',
  effective       plan_change_effective NOT NULL DEFAULT 'immediate'
);

CREATE INDEX IF NOT EXISTS tenant_plan_history_tenant_occurred_idx
  ON tenant_plan_history (tenant_id, occurred_at DESC);

DO $$ BEGIN
  REVOKE UPDATE, DELETE ON tenant_plan_history FROM cyberos_app;
EXCEPTION WHEN undefined_object THEN NULL;
END $$;

-- Session flag: history INSERT arms plan_tier UPDATE within the same TX.
CREATE OR REPLACE FUNCTION ten_arm_plan_history()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  PERFORM set_config('app.plan_change_ok', '1', true);
  RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS tenant_plan_history_arm ON tenant_plan_history;
CREATE TRIGGER tenant_plan_history_arm
  AFTER INSERT ON tenant_plan_history
  FOR EACH ROW
  EXECUTE FUNCTION ten_arm_plan_history();

CREATE OR REPLACE FUNCTION ten_require_plan_history()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
  IF NEW.plan_tier IS DISTINCT FROM OLD.plan_tier THEN
    IF current_setting('app.plan_change_ok', true) IS DISTINCT FROM '1' THEN
      RAISE EXCEPTION 'P0301 plan_tier update requires tenant_plan_history INSERT in same TX'
        USING ERRCODE = 'check_violation';
    END IF;
    PERFORM set_config('app.plan_change_ok', '0', true);
    NEW.plan_effective_since := now();
  END IF;
  RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS tenants_require_plan_history ON tenants;
CREATE TRIGGER tenants_require_plan_history
  BEFORE UPDATE OF plan_tier ON tenants
  FOR EACH ROW
  EXECUTE FUNCTION ten_require_plan_history();
