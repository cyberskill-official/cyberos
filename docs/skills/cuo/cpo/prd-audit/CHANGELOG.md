# CHANGELOG — `cuo/cpo/prd-audit`

> Format: Keep a Changelog 1.1.0. SemVer at the skill level: MAJOR breaks the rubric (changes a rule_id, adds an error rule, removes a rule) or breaks the audit-report file format. MINOR adds new warning rules or extends the output envelope additively. PATCH is editorial.

---

## v0.1.0 — 2026-05-06 (initial scaffold)

### Added

- `SKILL.md` — full v0.2.0 frontmatter; standalone+chained; `prd@1` validation_target; `nats-subjects@v1` wire-protocol emission.
- `RUBRIC.md` — `prd_rubric@1.0` with 6 rule families: FM-001..118 (frontmatter), SEC-001..012 (required sections), COND-001..005 (conditional sections), AUTH-001..004 (authority markers — NEW vs fr-audit), QA-001..009 (quality heuristics, mostly warning per Q4), SAFE-001..004 (untrusted-content), STALE-001 (cross-skill).
- `INVARIANTS.md` — 6 invariants. INV-001 (verdict reproducibility on mechanical rules; LLM-judgement rules are band-reproducible only).
- `AUDIT_LOOP.md` — 8-step loop algorithm; deterministic-input rule (mechanical-rule majority only).
- `REPORT_FORMAT.md` — `*.audit.md` format spec.
- `STANDALONE_INTERVIEW.md` — 1-2 question entry for chat invocations.
- `HUMAN_SUMMARY.md` — chat-rendered batch-completion template.
- `envelopes/prd-audit.input.json` + `prd-audit.output.json`.
- `acceptance/README.md` — priority test scenarios pending v0.3.0 harness.

### Driver

User said "do all stages" after registry v0.2.4 ship. Stage B per the strategic roadmap: prd-audit closes the quality gate between `prd-author` and downstream consumers (fr-author, future srs-author).

### What this version DOESN'T do (intentionally)

- No executable runtime — gated.
- No reference docs (HITL_PROTOCOL, UNTRUSTED_CONTENT, etc.) — at v0.2.0; expect divergence from cpo siblings per REF-015.
- No PIPELINE.md worked example — pending one chained run.

### Backwards compatibility

First version. No predecessor.
