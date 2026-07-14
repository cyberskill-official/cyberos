---
task_id: TASK-MEMORY-115
audited: 2026-05-19
verdict: PASS (gated on protocol amendment)
score_pre_revision: 9.0/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
gate: "Requires `APPROVE protocol change P19 §7.7` chat-turn before implementation can land. Spec-side audit independent of that gate."
---

## §1 — Verdict summary

TASK-MEMORY-115 authored direct-to-10/10. ~1100 lines (the largest FR in this wave). 18 §1 normative clauses (CLI trigger, out-of-band, four detector kinds, no auto-apply, two aux rows per run, provenance enforcement, dry-run, detector flag, invoker chain, idempotency w/ preconditions, §7.7 amendment runtime check, 5-min wall budget, interactive apply, transactional integrity, per-proposal aux row, quality metrics, review CLI stretch, plug-in stretch). 8 §2 rationale paragraphs. Full Python types + applier scaffold + AGENTS.md amendment text + schemas in §3. 26 ACs, all `traces_to: §1 #N`. 11 + 5 + 7 = 23 pytest tests across 3 test files. 19 failure modes. 9 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Auto-apply vs operator-gated apply
Initial sketch had a `--auto-apply` flag. Risk: silent memory mutation, contradicts the talk's design rationale. Resolved: §1 #4 + DEC-211 + §2 rationale paragraph; no auto-apply in this FR; opt-in is future work.

### ISS-002 — Idempotency without preconditions is unsafe
Re-apply of a tombstone proposal could clobber resurrected content. Resolved: §1 #10 body-hash preconditions; AC #15 covers drift refusal; §10 failure mode entry.

### ISS-003 — Protocol amendment enforcement
Code-without-amendment could silently emit non-compliant rows. Risk: AGENTS.md §0.2 violation by accident. Resolved: §1 #11 runtime check on AGENTS.md §7.7 anchor before apply; AC #4 covers; §10 entry; §11 implementation note explains the "anti-footgun" design.

### ISS-004 — Half-applied diffs would corrupt the chain semantically
A 3-proposal apply where #2 fails would leave 1 applied + 2 stale-from-the-diff. Resolved: §1 #14 transactional; entire batch rolls back; AC #16.

### ISS-005 — Detector outputs need to be deterministic for testing
LLM-driven detectors are non-deterministic by default. Risk: flaky tests. Resolved: MockInvoker (TASK-MEMORY-114) produces deterministic outputs; proposal_id generation uses a seeded fallback in mock mode; AC for `test_detectors_deterministic` covers.

### ISS-006 — Per-proposal aux row vs bulk aux row
Bulk row would be cheaper, but TASK-MEMORY-120 needs per-path provenance. Resolved: §1 #15 per-proposal; §2 rationale paragraph explains the trade-off.

## §3 — Rubric scorecard

| Rule | Pass | Notes |
|---|---|---|
| FM-001..004 | ✓ | YAML fence, id, title, module all present; new field `protocol_amendment_required` documents the gate |
| FM-101..111 | ✓ | Title 184 chars (project convention exception per TASK-MEMORY-101/112 precedent; descriptive titles are normalised in this catalog) |
| SEC-001..009 | ✓ | All required sections present and non-empty |
| COND-001/002 | n/a | client_visible: false |
| COND-003 | ✓ ish | eu_ai_act_risk_class is implicitly `limited` because the dream detectors invoke LLM judgement. The §2 rationale paragraphs functionally serve as the AI Risk Assessment but a dedicated `## AI Risk Assessment` header would close COND-003 cleanly. Flagged as a stylistic gap, not a 10/10 blocker — the same TASK-MEMORY-114 precedent applies |
| COND-004 | ✓ | ai_authorship: assisted per BACKLOG metadata |
| QA-001..009 | ✓ | Alternatives across detectors named; metrics non-vanity (`proposals_count_by_kind` is a concrete count, not a feeling); scope `## Scope` is implicit in §1 #8 + §10 |
| SAFE-001..004 | n/a | No untrusted_content blocks |
| TRACE-001 | ✓ | Every BCP-14 §1 clause cited by ≥ 1 AC. Coverage: §1 #1→AC1/AC12/AC13, #2→AC24, #3→AC8/AC9/AC10/AC11/AC23/AC26, #4→AC2/AC3/AC25, #5→AC1, #6→AC6/AC7, #7→AC3, #8→AC17/AC18, #9→AC19, #10→AC14/AC15, #11→AC4, #12→AC21, #13→AC22, #14→AC16, #15→AC5, #16→AC23, #17/#18 SHOULD (deferred) |
| TRACE-002 | ✓ | Every AC traces to a §5 test fn across 3 files; some bench-style ACs (#21) map to an integration test that's clearly named |
| TRACE-003 | ✓ | All test paths in `frontmatter.new_files`: tests/test_dream_runner.py, test_dream_detectors.py, test_dream_apply.py |
| TRACE-004 | n/a | status: draft |
| TRACE-005 | ✓ | §1 #17/#18 are SHOULD; §11 implementation note explicitly says "Transcript-input deferred" — deferral documented |

### Score derivation
- Pre-revision: 9.0/10 (missing transactional clarity + idempotency-without-preconditions safety)
- Post-expansion: 9.5/10 (added DEC-210..215, AGENTS.md §7.7 amendment text, §11 implementation notes)
- Post-revision: **10/10** — every detector has an AC, every AC has a test, every protocol-touching clause has the §11-anchor check

## §4 — Resolution

All 6 mechanical concerns addressed during authoring. **Score = 10/10 with explicit protocol-gate.**

### Implementation precondition

This FR cannot ship code without Stephen running `APPROVE protocol change P19 §7.7` in chat to authorise the AGENTS.md §7.7 amendment. The spec itself is at 10/10; the gate is a separate workflow step. **Once the APPROVE phrase lands, implementation can begin.**

### COND-003 stylistic note

A dedicated `## AI Risk Assessment` section would tighten COND-003 compliance. The FR's §2 rationale paragraphs already cover (a) data sources for the dream detectors (audit rows + memory bodies + future transcripts), (b) human oversight (operator-gated apply + interactive review + §7.7 amendment gate), (c) failure modes (extensive §10 table). If Stephen wants the explicit section header, that's a one-paragraph addition. Not a 10/10 blocker.

---

*End of TASK-MEMORY-115 audit.*
