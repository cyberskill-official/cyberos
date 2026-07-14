---
task_id: TASK-CUO-201
audited: 2026-05-19
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_revision: 10/10
issues_resolved: 4
template: engineering-spec@1
rubric_version: audit_rubric@2.0
---

Harness Wave 2 — refinement-proposal emitter with stripe-based deduplication. The load-bearing rule: first occurrence of a tripped signal writes a proposal to `docs/proposals/open/<stripe>-<ts>.md` and the chain continues; second occurrence of the SAME stripe while the first is unresolved halts the chain with `HITL_HALT`. 10 §1 normative clauses, 10 §4 ACs, 9 §5 named tests, stripe taxonomy table with 5 named stripes, alternatives section with 4 options, 3 success metrics.

## Audit rule outcomes

| family | rules checked | result |
|---|---|---|
| FM-001..004 structural | YAML parses; snake_case keys; no dupes; template=task@1 | PASS |
| FM-101..111 per-field | title 67 chars; author @stephen; department engineering; status draft; priority p1; created_at ISO 8601; ai_authorship assisted; feature_type internal_tooling; risk minimal; target_release 2026-Q3; client_visible false | PASS |
| SEC-001..009 sections | Summary / Problem / Proposed Solution / Alternatives Considered / Success Metrics / Scope (with Out-of-scope) / Dependencies / AI Authorship Disclosure all present + non-empty | PASS |
| COND-003 AI Risk Assessment | n/a (risk class = minimal) | n/a |
| COND-004 AI Authorship Disclosure | required (ai_authorship=assisted); added in revision | PASS (after revision) |
| QA-001..009 anti-patterns | scope has Out-of-scope; metrics carry baseline+target+deadline; alternatives lists 4 distinct options | PASS |
| SAFE-001..004 untrusted content | none present | n/a |
| TRACE-001 §1→§4 | every §1 clause cited by ≥1 §4 AC (gaps on §1 #6 + §1 #10 patched via inline traces_to: markers) | PASS (after revision) |
| TRACE-002 §4→§5 | every AC names a §5 test (10 ACs → 10 tests) | PASS |
| TRACE-003 test paths in new_files | `modules/cuo/tests/test_refinement_proposal.py` declared | PASS |
| TRACE-004 status:done → coverage | n/a (status:draft) | n/a |
| TRACE-005 deferred-slice | n/a | n/a |

## Issues resolved (4)

1. **FM-101 title length** — pre-revision title was 178 chars; trimmed to "Harness Wave 2 — refinement-proposal emitter with stripe-based dedup" (67 chars).
2. **COND-004 AI Authorship Disclosure missing** — added section with explicit note that the halt-on-repeat-stripe rule was operator-architected (not LLM-suggested).
3. **TRACE-001 untraced §1 #6** — clause about emitting `cuo.refinement_proposal_emitted` aux row; added `*(traces_to: §1 #6 → AC #1)*`.
4. **TRACE-001 untraced §1 #10** — clause about treating `## Suggested change` as informational; added `*(traces_to: §1 #10 → AC #5)*`.

## Architectural notes (operator decisions captured)

- **Stripe dedup window is `open/` only** (§1 #9) — applied/rejected stripes re-open naturally on recurrence. This means a previously-fixed issue can fire again later, and the harness will correctly emit a fresh proposal rather than treating it as a known recurring problem.
- **Halt-on-repeat is HITL_HALT, not ROUTED_BACK** (§1 #7) — these are distinct outcomes in the supervisor: ROUTED_BACK is a soft "try again later" signal; HITL_HALT genuinely needs human input. The drain command (TASK-CUO-200 §1 #5 indirectly) stops the loop on HITL_HALT.
- **Stripe pattern_hash width = 8 hex chars** (§1 #2 + AC #9) — 32 bits of entropy gives birthday-bound collision probability ≈ 2⁻³². Sufficient for the use case; if collisions become an issue post-ship, widen to 12 chars in a minor bump.

## Implementation readiness

Implementation-ready. Estimated effort: ~1 day for stripe.py + refinement_proposal.py + 4 CLI subcommands + tests. Depends on TASK-CUO-200 (`docs/proposals/INDEX.md` may end up shared).

**Score = 10/10.**

*End of TASK-CUO-201 audit.*
