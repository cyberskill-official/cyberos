# `prd-audit/acceptance/` — priority test scenarios

> Stub state at v0.1.0. Fixtures pending v0.3.0 harness.

## Priority scenarios (10)

### sev-0 (gate v0.1.0 → v0.2.0)

1. **INV-001: mechanical-rule reproducibility.** Run audit twice on the same passing PRD; mechanical-rule subset of verdict diff = empty.
2. **AUTH-001: missing authority marker.** PRD with a Goal lacking `<!-- authority: ... -->`. Expected: `needs_human` verdict, hitl_category `authority_marker_missing`.
3. **AUTH-002: llm-implicit on Goal.** Same as above with `<!-- authority: llm-implicit -->`. Expected: `needs_human`, hitl_category `authority_too_weak`.
4. **COND-002: high-risk PRD without AI Risk Assessment.** PRD frontmatter `eu_ai_act_risk_class: high`, body missing `## High-Risk AI Risk Assessment`. Expected: `fail` with COND-002 error.
5. **STALE-001: PRD changed after audit.** Run audit; modify PRD; re-run. Expected: STALE verdict.

### sev-1 (gate v0.2.x → v1.0.0)

6. **QA-008: confidentiality loosening.** PRD `confidentiality: internal`; brief was `regulated`. Expected: `fail` with QA-008.
7. **Advisory-leaning verification.** PRD with 5 warning-severity issues + 0 error issues. Expected: `pass` with `exit_code: 2`; advisory-leaning rubric does NOT block.
8. **HITL round-trip.** First run produces 2 needs_human; user answers; second run resolves both → pass.
9. **Confidence-band reporting (RUBRIC §15.10).** Audit report includes mechanical rules at `confidence ≥ 0.95` and LLM-judgement rules with their actual band ≤ 0.7.

### sev-2 (regression coverage)

10. **Empty PRD list.** Input `prd_paths: []`. Expected: schema validation fails (minItems: 1), BOOT-003.

## Citations

- Pattern source — `cuo/cpo/fr-audit/acceptance/README.md`.
- Q4 of registry v0.2.4 — advisory-leaning rubric verified by scenario 7.
