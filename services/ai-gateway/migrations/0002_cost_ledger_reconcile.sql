-- TASK-AI-002 — Add reconcile columns to cost_ledger_hold.
-- Additive migration; does not modify existing rows.

ALTER TABLE cost_ledger_hold
  ADD COLUMN actual_usd          NUMERIC(12,4) NULL,
  ADD COLUMN reconciled_at       TIMESTAMPTZ   NULL,
  ADD COLUMN refunded_at         TIMESTAMPTZ   NULL,
  ADD COLUMN refund_reason       TEXT          NULL,
  ADD COLUMN provider_request_id TEXT          NULL;

-- Expand state constraint to include reconciled + refunded + expired.
ALTER TABLE cost_ledger_hold
  DROP CONSTRAINT IF EXISTS cost_ledger_hold_state_check;

ALTER TABLE cost_ledger_hold
  ADD CONSTRAINT cost_ledger_hold_state_check
    CHECK (state IN ('held','reconciled','refunded','expired'));

-- Warn-threshold de-duplication (AC #7).
ALTER TABLE cost_ledger
  ADD COLUMN warn_emitted_at TIMESTAMPTZ NULL;

-- Index for OBS "AI invocations in last 24h" query.
CREATE INDEX cost_ledger_hold_reconciled_at_idx
  ON cost_ledger_hold (reconciled_at DESC)
  WHERE state = 'reconciled';
