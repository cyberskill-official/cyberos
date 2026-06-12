-- FR-AI-002 — Preserve request context needed by post-call reconcile audits.
-- Additive migration for existing FR-AI-001 hold rows.

ALTER TABLE cost_ledger_hold
  ADD COLUMN IF NOT EXISTS agent_persona TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS model_alias    TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS warn_crossed   BOOLEAN NOT NULL DEFAULT FALSE;
