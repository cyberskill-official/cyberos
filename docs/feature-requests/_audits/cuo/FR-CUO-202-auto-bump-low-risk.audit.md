---
fr_id: FR-CUO-202
audited: 2026-05-19
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_revision: 10/10
issues_resolved: 3
template: engineering-spec@1
rubric_version: audit_rubric@2.0
---

Harness Wave 3 — auto-bump applier for low-risk refinement proposals. Classifies diffs into 7 buckets (cosmetic, wording_polish, threshold_tune, rule_addition, rule_removal, contract_field_change, safety_class), auto-applies the safe ones with patch/minor version bumps, queues the rest for explicit operator APPROVE. Test-gate via skill `acceptance/TRIGGER_TESTS.md` before commit. 10 §1 normative clauses, §2 bump-level table, 10 §4 ACs, 9 §5 named tests.

## Audit rule outcomes

| family | rules checked | result |
|---|---|---|
| FM-001..004 structural | YAML parses; snake_case; no dupes; template=feature_request@1 | PASS |
| FM-101..111 per-field | title 67 chars; author @stephen; department engineering; status draft; priority p2; created_at ISO 8601; ai_authorship assisted; feature_type internal_tooling; **risk limited** (mutates skill files); target_release 2026-Q3; client_visible false | PASS |
| SEC-001..009 sections | Summary / Problem / Proposed Solution / Alternatives Considered / Success Metrics / Scope (with Out-of-scope) / Dependencies / AI Risk Assessment / AI Authorship Disclosure all present + non-empty | PASS |
| COND-003 AI Risk Assessment | required since risk=limited; added with H3s `### Data Sources`, `### Human Oversight`, `### Failure Modes` in correct order | PASS (after revision) |
| COND-004 AI Authorship Disclosure | required since ai_authorship=assisted; added in revision | PASS (after revision) |
| QA-001..009 anti-patterns | scope has Out-of-scope; metrics carry baseline+target+deadline; alternatives lists 3 distinct options | PASS |
| SAFE-001..004 untrusted content | none present | n/a |
| TRACE-001 §1→§4 | every §1 clause cited by ≥1 §4 AC | PASS |
| TRACE-002 §4→§5 | every AC names a §5 test (10 ACs → 9 tests; AC #1+#8 share `test_cosmetic_auto_applies` legitimately) | PASS |
| TRACE-003 test paths in new_files | `modules/cuo/tests/test_proposal_applier.py` declared | PASS |
| TRACE-004 status:done → coverage | n/a (status:draft) | n/a |
| TRACE-005 deferred-slice | n/a | n/a |

## Issues resolved (3)

1. **FM-101 title length** — pre-revision title was 137 chars; trimmed to "Harness Wave 3 — auto-bump applier for low-risk refinement proposals" (67 chars).
2. **COND-003 AI Risk Assessment missing** — required because the applier mutates skill files (eu_ai_act_risk_class=limited). Added section with `### Data Sources` (trusted-only), `### Human Oversight` (major/safety always queue; pre-apply TRIGGER_TESTS gate), `### Failure Modes` (4 named failure modes with mitigations).
3. **COND-004 AI Authorship Disclosure missing** — added with Tools used / Scope / Human review bullets.

## Architectural notes

- **Safety-class is the protocol's defence-in-depth** (§1 #10): even if the classifier mis-categorises a diff, `## Risk class: safety` in the proposal body NEVER auto-applies. This is the "two-key system" — classifier + body-marker — for diffs that touch security or audit-chain semantics.
- **Pre-apply test gate** (§1 #8): the skill's own `acceptance/TRIGGER_TESTS.md` runs against the BUMPED version BEFORE the applied state is written. A test regression aborts the apply + emits `cuo.proposal_apply_failed`, leaving the queue in a known-good state.
- **CHANGELOG.md append** (§1 #9): every applied proposal lands a human-readable entry. This is the operator's primary audit surface for "what got auto-evolved last week?"

## Implementation readiness

Implementation-ready. Estimated effort: ~1.5 days for proposal_applier.py + version_bump.py + classifier + 3 CLI subcommands + tests. Depends on FR-CUO-201 (proposals must exist + be enumerated through the same docs/proposals/ tree).

**Score = 10/10.**

*End of FR-CUO-202 audit.*
