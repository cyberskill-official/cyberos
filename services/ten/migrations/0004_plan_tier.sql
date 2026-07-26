-- TASK-TEN-002 — closed plan_tier enum (DEC-770 / DEC-771).
-- Auth historically used TEXT + CHECK including 'sandbox'; TEN owns the closed 3-value enum.
-- This migration is additive for a dedicated TEN schema / tenants table. When reconciling with
-- services/auth, drop sandbox or relocate it before applying the enum cast.

DO $$ BEGIN
  CREATE TYPE plan_tier AS ENUM ('starter', 'team', 'enterprise');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
  CREATE TYPE plan_change_effective AS ENUM ('immediate', 'next_period', 'defer_billing_only');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

ALTER TABLE IF EXISTS tenants
  ADD COLUMN IF NOT EXISTS is_founder_tenant BOOLEAN NOT NULL DEFAULT false;

ALTER TABLE IF EXISTS tenants
  ADD COLUMN IF NOT EXISTS metering_caps_yaml TEXT;

-- Default new tenants to starter when the column is present as TEXT (auth substrate).
-- Full enum cast is an operator cutover step once sandbox rows are cleared.
