# `fr-audit` acceptance fixtures

Layer 2 (functional) regression tests per registry README Part 13.3. Empty in v0.2.1 — production fixtures pending the runtime build (Phase D of the host-adapter pipeline, registry README Part 9).

## What goes here

For each canonical scenario the auditor must handle, ship one folder containing:

- `golden-input.json` — a known input envelope matching `../envelopes/fr-audit.input.json` plus the FR(s) at the declared paths
- `golden-fr/FR-NNN-<slug>.md` — the FR(s) under audit (the input artefacts)
- `golden-output.json` — the expected output envelope at AUDIT_BATCH_SUMMARY
- `golden-audit/FR-NNN-<slug>.audit.md` — the expected audit reports (byte-equal for deterministic verdicts; INV-001 enforces this)
- `description.md` — what scenario this covers, which rules fire, expected verdict

## Scenarios to cover (priority order)

The first three are Tier-1:

1. **all-pass** — well-formed FR; every rule passes; PASS verdict. Verifies the rubric runs cleanly + report shape + INV-001 (verdict determinism).
2. **single-error** — FR with one FM-110 violation; verifies fail-on-error behaviour + report citation completeness (INV-004).
3. **needs-human-qa007** — FR with an unsourced numeric target; verifies QA-007 → needs_human escalation + HITL_BATCH_REQUEST + INV-003 (precise needs_human).

Tier-2 scenarios:

4. **client-visible-cond** — FR with `client_visible: true` triggering COND-001 + COND-002 checks.
5. **eu-ai-act-high-risk** — FR with `eu_ai_act_risk_class: high` triggering COND-003.
6. **stale-detection** — FR's on-disk SHA differs from upstream manifest's `fr_hash`; verifies STALE-001 + INV-005 (no false-pass on STALE).
7. **untrusted-injection-attempt** — FR with prompt-injection markers inside `<untrusted_content>`; verifies SAFE-003 detection.
8. **rubric-coverage** — input forcing the report to enumerate every rule ID; verifies INV-002 (no rule silently elided).
9. **deterministic-replay** — same FR + same rubric, two runs; assert byte-identical reports (INV-001 = sev-0).

Tier-3:

10. **localised-fr-vietnamese** — Vietnamese-language FR; verifies QA-009 plain-English check is correctly skipped or routed to QA-009-vi (when added).

## How to run (when the runtime ships)

Same as `fr-author/acceptance/`. Today: paste SKILL.md body into Claude.ai with the input envelope as user message; compare to golden audit report excluding the `last_audit_at` line. Determinism check is byte-equal.

## Status

📋 **Empty (pending runtime + harness).** First three Tier-1 fixtures are a v0.3.0 milestone task.

## See also

- Registry README Part 11 (canonical fr-author → fr-audit chain trace).
- Recipe 8 in registry README Part 19 — fixture authoring procedure.
- `INVARIANTS.md` — the determinism contract (INV-001) the harness MUST enforce.
- `RUBRIC.md` — the catalogue of rules every fixture must trigger or not.
