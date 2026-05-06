# `prd-audit` self-audit invariants (scaffold)

> Truths the auditor enforces about its own behaviour. Mirrors fr-audit's INVARIANTS.md but the deterministic-input rule applies to the mechanical rule majority only (per RUBRIC §15.10's split).

## Invariants

### INV-001 — verdict reproducibility on mechanical rules

**Statement.** Two runs against the same `audited_prd_sha256` and the same `rubric_version` produce byte-identical verdict sets ON MECHANICAL RULES (FM-* / SEC-* / COND-* / SAFE-* / AUTH-001 / AUTH-002 / QA-005 / QA-007 / QA-008 / STALE-001). LLM-judgement rules are band-reproducible only — the same input yields a verdict within the same confidence band, but the exact float can drift between model versions.

**Check.** Re-run audit on a just-audited PRD; diff the mechanical-rule subset; non-empty diff = breach.

**Severity.** `error`.

### INV-002 — rubric coverage

**Statement.** Every rule_id in RUBRIC.md appears in this run's audit report under either `passed_rules:` / `failed_rules:` / `skipped_rules:` (with reason). No rule silently elided.

**Severity.** `error`.

### INV-003 — needs_human is precise

**Statement.** A `needs_human` verdict only fires when the rule's ambiguity criterion is met; never as a fallback for "I don't know."

**Severity.** `error`.

### INV-004 — citation completeness

**Statement.** Every failed rule cites: rule_id + line number (or "frontmatter") + exact substring being flagged.

**Severity.** `warning`.

### INV-005 — no false-pass on STALE

**Statement.** If the PRD's current SHA-256 differs from `audited_prd_sha256` declared at run start, verdict MUST be `STALE` (or run aborts with `inputs_changed`). Never produce `pass` against a SHA that doesn't match the file at write time.

**Severity.** `error`.

### INV-006 — no rubric drift mid-batch

**Statement.** Within a single batch, `rubric_version` MUST NOT change. If RUBRIC.md is edited mid-batch (rare; usually means manual fine-tune is happening), batch aborts with `RUBRIC_CHANGED_MID_BATCH`.

**Severity.** `error`.

## Adding a new invariant

Same procedure as fr-audit. Author + cpo propose; registry maintainer reviews; acceptance test added.
