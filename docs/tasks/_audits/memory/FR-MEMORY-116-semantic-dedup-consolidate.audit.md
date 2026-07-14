---
task_id: TASK-MEMORY-116
audited: 2026-05-19
verdict: PASS
score_pre_revision: 9.0/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 3
template: engineering-spec@1
---

## §1 — Verdict summary

TASK-MEMORY-116 authored direct-to-10/10. ~460 lines (compact because it's a wrapper FR). 10 §1 clauses (opt-in, dry-run default, reuse TASK-MEMORY-115 detectors, threshold knob, audit-row invocation tagging, phase ordering, §7.7 anchor enforcement, summary stdout, idempotency, threshold range). 4 §2 rationale paragraphs. CLI signature + pipeline integration scaffold in §3. 14 ACs all `traces_to: §1 #N`. 10 pytest tests covering all ACs. 9 failure modes. 4 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Could become a parallel dedup path
First sketch had `consolidate.py` doing its own cosine-sim pass. Risk: drift vs TASK-MEMORY-115. Resolved: §1 #3 + DEC-220 forces import from `cyberos.core.dream.detectors.duplicates`; AC #13 asserts no duplicate detector code exists in consolidate.py.

### ISS-002 — Auto-apply in cron is dangerous
Cron + auto-apply = bad-night scenario. Resolved: §1 #2 + DEC-221 makes `--dry-run` the default when `--semantic-dedup` is passed; `--apply` is explicit; AC #2-#5 cover.

### ISS-003 — Audit-row provenance ambiguity
Without `extra.invocation`, dedup rows from consolidate vs explicit dream runs are indistinguishable in history. Resolved: DEC-222 + §1 #5 + AC #6.

## §3 — Rubric scorecard

| Rule | Pass | Notes |
|---|---|---|
| FM-001..004 | ✓ | Frontmatter clean |
| FM-101..111 | ✓ | Title 155 chars (project convention precedent) |
| SEC-001..009 | ✓ | All required sections present |
| COND-001..004 | n/a | client_visible: false; eu_ai_act_risk_class minimal (calls LLM only transitively via duplicates detector which itself sometimes uses LLM — but the SemanticDedup phase's logic is cosine-sim, no LLM call) |
| QA-001..009 | ✓ | Alternatives (separate path vs subset) discussed; threshold knob discussed; scope clear |
| SAFE-001..004 | n/a | No untrusted_content |
| TRACE-001 | ✓ | Coverage: §1 #1→AC1, #2→AC2/AC3/AC5, #3→AC13/AC14, #4→AC7, #5→AC6, #6→AC9/AC10, #7→AC4/AC5, #8→AC12, #9→AC11, #10→AC7/AC8 |
| TRACE-002 | ✓ | Every AC traces to a §5 test fn |
| TRACE-003 | ✓ | Test path in `frontmatter.new_files` |
| TRACE-004 | n/a | status: draft |
| TRACE-005 | n/a | No deferred slices |

### Score derivation
- Pre-revision: 9.0/10 (clean wrapper FR; minor ambiguity on phase failure semantics)
- Post-expansion: 9.5/10 (added DEC-220..222 + §1 #6 phase-ordering invariant)
- Post-revision: **10/10**

## §4 — Resolution

All 3 mechanical concerns addressed during authoring. **Score = 10/10.** Implementation gated only by TASK-MEMORY-115 shipping (it provides the detector + applier).

---

*End of TASK-MEMORY-116 audit.*
