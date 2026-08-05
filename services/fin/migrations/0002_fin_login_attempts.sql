-- TASK-AUTH-102 — Durable login rate limiting for the finance ledger.
--
-- Problem. lib/rateLimit.js keeps attempt counters in a module-level Map. On
-- Vercel every lambda instance has private memory, so the counter is per
-- instance, not per IP. The limiter therefore constrains one instance at a
-- time and not the attacker.
--
-- Proven against production on 2026-08-05, not inferred:
--   sequential  attempts 1-4 -> 401, attempts 5-8 -> 429   (looks like it works)
--   parallel    12 concurrent attempts from the SAME already-locked-out IP
--               -> 12 x 401, every one allowed through
-- Concurrency defeats it completely, and an attacker can scale concurrency at
-- will. The wiring in app/api/login/route.js is correct; only the storage is.
--
-- This matters more than it would in a private app. Decision D-23 keeps all
-- read endpoints public, so the ledger advertises itself, and the admin login
-- is the only thing standing between the internet and every write endpoint.
--
-- This migration adds the shared store the limiter needs. It is deliberately
-- one small table with no foreign keys: rate limiting must keep working when
-- other things are broken, and must never block a login by referential
-- accident.
--
-- Rollback: DROP TABLE fin_login_attempts; and revert lib/rateLimit.js. No
-- other object is touched and no existing data is read or written.

BEGIN;

-- 1. Table ------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS fin_login_attempts (
    tenant_id          uuid        NOT NULL,
    -- Client IP as seen by the edge. Kept as text, not inet: the value comes
    -- from x-forwarded-for and may be a non-address fallback such as
    -- 'unknown-ip'. Storing it as inet would raise on those and fail a login.
    ip_key             text        NOT NULL,
    attempt_count      integer     NOT NULL DEFAULT 0,
    window_started_at  timestamptz NOT NULL DEFAULT now(),
    last_attempt_at    timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, ip_key)
);

COMMENT ON TABLE fin_login_attempts IS
    'Shared login attempt counters. Replaces per-lambda in-memory state, which concurrency defeated (TASK-AUTH-102).';
COMMENT ON COLUMN fin_login_attempts.ip_key IS
    'Client IP from x-forwarded-for; text rather than inet because non-address fallbacks are possible.';
COMMENT ON COLUMN fin_login_attempts.window_started_at IS
    'Start of the current fixed window. The application expires the window; no background job is required.';

-- Supports periodic pruning of rows nobody has touched in a long time. Not
-- needed for correctness - the application expires windows by timestamp - so
-- cleanup can be a manual DELETE whenever the table grows.
CREATE INDEX IF NOT EXISTS fin_login_attempts_last_attempt_idx
    ON fin_login_attempts (last_attempt_at);

-- 2. Row Level Security -----------------------------------------------------
--
-- Same posture as the other fin_* tables: enabled AND forced, so it applies to
-- the table owner too, with a single tenant rule carrying both USING and
-- WITH CHECK. No nil-UUID escape hatch; migration 0001 removed those for a
-- reason and this table must not reintroduce the pattern.

ALTER TABLE fin_login_attempts ENABLE ROW LEVEL SECURITY;
ALTER TABLE fin_login_attempts FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS fin_login_attempts_tenant_scoped ON fin_login_attempts;
CREATE POLICY fin_login_attempts_tenant_scoped ON fin_login_attempts
    FOR ALL
    USING (tenant_id::text = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id::text = current_setting('app.current_tenant_id', true));

-- 3. Grants -----------------------------------------------------------------
--
-- SELECT, INSERT and UPDATE only. DELETE is withheld on purpose: the login
-- path never needs it, and without it a compromised application credential
-- cannot erase the evidence of a brute-force attempt. Pruning is an
-- administrative action, run as the owner.

GRANT SELECT, INSERT, UPDATE ON fin_login_attempts TO finance_app;

COMMIT;

-- Verification.
--
-- 1. RLS is on and forced:
--
--      SELECT relrowsecurity, relforcerowsecurity
--      FROM pg_class WHERE relname = 'fin_login_attempts';
--      -- expect: t | t
--
-- 2. finance_app cannot delete:
--
--      SELECT has_table_privilege('finance_app','fin_login_attempts','DELETE');
--      -- expect: f
--
-- 3. The counter is atomic under concurrency. The application uses a single
--    INSERT ... ON CONFLICT DO UPDATE ... RETURNING, so concurrent writers
--    serialize on the row lock and each observes a distinct count. The
--    behavioural proof is the parallel probe above: after this lands, 12
--    concurrent attempts from a locked-out IP must return 429, not 401.
