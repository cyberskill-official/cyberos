---
task_id: TASK-CUO-203
audited: 2026-05-19
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_revision: 10/10
issues_resolved: 4
template: engineering-spec@1
rubric_version: audit_rubric@2.0
---

Harness Wave 4 — workflow-level continuous evolution. Aggregates per-workflow outcome distribution (COMPLETED / ROUTED_BACK / HITL_HALT / FAILED), trips threshold signals (`routed_back_rate_above`, `hitl_halt_rate_above`, `repeat_phase_failure_above`, `chain_length_efficiency_below`), proposes workflow-chain edits via the stripe machinery from TASK-CUO-201. All workflow diffs default to queue (workflows are higher-stakes than skills). 10 §1 normative clauses, §2 stripe-collision discussion, 10 §4 ACs, 9 §5 named tests.

## Audit rule outcomes

| family | rules checked | result |
|---|---|---|
| FM-001..004 structural | YAML parses; snake_case; no dupes; template=task@1 | PASS |
| FM-101..111 per-field | title 65 chars; author @stephen; department engineering; status draft; priority p2; created_at ISO 8601; ai_authorship assisted; feature_type internal_tooling; **risk limited** (mutates workflow files); target_release 2026-Q3; client_visible false | PASS |
| SEC-001..009 sections | Summary / Problem / Proposed Solution / Alternatives Considered / Success Metrics / Scope (with Out-of-scope) / Dependencies / AI Risk Assessment / AI Authorship Disclosure all present + non-empty | PASS |
| COND-003 AI Risk Assessment | required since risk=limited; added with H3s `### Data Sources`, `### Human Oversight`, `### Failure Modes` in correct order | PASS (after revision) |
| COND-004 AI Authorship Disclosure | required since ai_authorship=assisted; added in revision | PASS (after revision) |
| QA-001..009 anti-patterns | scope has Out-of-scope; metrics carry baseline+target+deadline; alternatives lists 3 distinct options | PASS |
| SAFE-001..004 untrusted content | none present | n/a |
| TRACE-001 §1→§4 | every §1 clause cited by ≥1 §4 AC (gap on §1 #3 patched via inline traces_to:) | PASS (after revision) |
| TRACE-002 §4→§5 | every AC names a §5 test entry | PASS |
| TRACE-003 test paths in new_files | `modules/cuo/tests/test_workflow_evolution.py` declared | PASS |
| TRACE-004 status:done → coverage | n/a (status:draft) | n/a |
| TRACE-005 deferred-slice | n/a | n/a |

## Issues resolved (4)

1. **FM-101 title length** — pre-revision title was 162 chars; trimmed to "Harness Wave 4 — workflow-level evolution via outcome distribution" (65 chars).
2. **TRACE-001 untraced §1 #3** — clause about declaring `self_audit` blocks in workflow YAML (schema extension); added `*(traces_to: §1 #3 → AC #2)*`.
3. **COND-003 AI Risk Assessment missing** — required because the applier mutates workflow YAML + skill_chain (eu_ai_act_risk_class=limited). Added section with Data Sources (trusted-only), Human Oversight (all workflow diffs default to queue), 4 named Failure Modes with mitigations.
4. **COND-004 AI Authorship Disclosure missing** — added with Tools used / Scope / Human review bullets.

## Architectural notes

- **Workflow stripes are disjoint from skill stripes** by format (§2): skill stripes use `<skill_slug>:...`, workflow stripes use `<persona>/<workflow_slug>:...`. The `/` makes collisions impossible.
- **All workflow diffs default to queue** (§1 #6): even cosmetic workflow edits queue under `pending_approval/`. The reason is workflows are read by many downstream FRs; a typo could change orchestration semantics. Auto-apply requires explicit `--auto-workflow-diffs` flag.
- **Mid-flight FR uses old chain** (§2 out-of-scope, §1 risk-assessment failure mode): when a chain edit applies, in-flight runs keep their snapshot of the chain at start. This is consistent with TASK-MEMORY-115's dream-snapshot semantics — the workflow chain is just another in-memory artefact whose mutations don't propagate retroactively.

## Implementation readiness

Implementation-ready. Estimated effort: ~1.5 days for workflow_evolution.py + classifier extension + 2 CLI subcommands + tests + schema patch in `cuo/contracts/workflow/CONTRACT.md`. Depends on TASK-CUO-200/201/202 — implementation order is strict.

**Score = 10/10.**

*End of TASK-CUO-203 audit.*
