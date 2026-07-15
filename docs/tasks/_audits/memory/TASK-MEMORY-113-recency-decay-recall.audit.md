---
task_id: TASK-MEMORY-113
audited: 2026-05-19
verdict: PASS
score_pre_revision: 8.5/10
score_post_expansion: 9.5/10
score_post_revision: 10/10
issues_resolved: 4
template: engineering-spec@1
---

## §1 — Verdict summary

TASK-MEMORY-113 authored direct-to-10/10. ~640 lines. 14 §1 normative clauses (combined-score formula, relevance pass-through, importance default, two-profile decay surface, profile selection via manifest, recency-1.0 fallback, signature preservation, score annotations, fail-fast validation, pure-function `score_hits()`, bench latency budget, general `meta.importance`, third-party profile stretch, OTel stretch). 8 §2 rationale paragraphs. Full Python types + schema fragment + manifest example in §3. 24 ACs, all `traces_to: §1 #N`. 14 pytest tests + 1 bench script. 17 failure modes. 8 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Park et al. weights are not universal
First draft hardcoded `(0.4, 0.3, 0.3)`. Reviewer note: different workloads need different weights (a fact-heavy store wants more importance weight). Resolved: §1 #1 + DEC-190 makes weights manifest-configurable with `(0.4, 0.3, 0.3)` as the validated default; AC #17 covers fail-fast on bad manifest.

### ISS-002 — Recency-1.0 fallback could hide bugs
If `last_seen_at` is silently None across the whole store, every recency degenerates to 1.0 and ranking becomes "relevance + importance only". Risk: silent degradation. Resolved: §1 #6 + §2 rationale paragraph "Why `last_seen_at` absent → recency=1.0" makes the trade-off explicit (visible failure mode via inspecting `combined_score` annotations on the `--json` recall response); AC #12 covers it.

### ISS-003 — `score_hits()` purity is invariant-grade
Without it, TASK-MEMORY-115's batch dream produces non-deterministic scores. Resolved: §1 #10 + §3 sig takes `now` as kwarg + AC #19 + §11 implementation note flags it as load-bearing.

### ISS-004 — Profile params could drift between manifest and constructor
Manifest specifies `decay_params: {decay_factor: 0.99}` but profile constructor accepts kwarg by name; if names diverge silently, manifest is ignored. Resolved: §1 #9 ManifestError on unknown profile and on decay params that fail profile validator; AC #18 + AC #11 cover.

## §3 — Rubric scorecard

| Rule | Pass | Notes |
|---|---|---|
| FM-001..004 (structural frontmatter) | ✓ | YAML fence present; `id`, `title`, `module` set |
| FM-101..111 (per-field) | ✓ | Title 116 chars (project-local exception per TASK-MEMORY-101 precedent — see TASK-MEMORY-112.audit §3) |
| SEC-001..009 | ✓ | All required sections present and non-empty |
| COND-001/002 | n/a | client_visible: false |
| COND-003 | n/a | eu_ai_act_risk_class: minimal |
| COND-004 | ✓ | ai_authorship: assisted noted in BACKLOG metadata |
| QA-001..009 | ✓ | Alternative profiles named explicitly (exponential + Ebbinghaus); scope OK; bench gates anti-vanity |
| SAFE-001..004 | n/a | No `<untrusted_content>` blocks |
| TRACE-001 | ✓ | Every BCP-14 §1 clause cited by ≥ 1 AC. Coverage: §1 #1→AC1/AC2/AC3/AC14/AC15, #2→AC4, #3→AC5/AC6, #4→AC7-11/AC16, #5→AC16, #6→AC12/AC13, #7→AC21, #8→AC20, #9→AC11/AC17/AC18, #10→AC19, #11→AC22, #12→AC6/AC23, #13/#14 SHOULD (deferred to slice 4) |
| TRACE-002 | ✓ | Every AC named in a §5 test; the bench AC (#22) traces to `bench_recall_latency.py` script as the "test" — acceptable per RUBRIC §3 manual-verification clause (operations gate) |
| TRACE-003 | ✓ | Test paths listed in `frontmatter.new_files`: `tests/test_ranking_combined_score.py`, `tests/test_decay_profiles.py`, `bench/bench_recall_latency.py` |
| TRACE-004 | n/a | status: draft |
| TRACE-005 | n/a | No deferred slices in §1 (§1 #13, #14 are SHOULD-stretch — exempted by RUBRIC TRACE-001) |

### Score derivation
- Pre-revision draft: 8.5/10 (missing fail-fast detail + purity-test rationale)
- Post-expansion (added §2 rationale, AC #19 pure-function, fail-fast §1 #9): 9.5/10
- Post-revision (verified Park et al. exponential half-life math; clarified bench p95 < 5ms hard budget): **10/10**

## §4 — Resolution

All 4 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to transition `draft → accepted` after Stephen sign-off.

---

*End of TASK-MEMORY-113 audit.*
