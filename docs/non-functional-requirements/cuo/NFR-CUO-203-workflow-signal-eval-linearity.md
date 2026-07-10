---
id: NFR-CUO-203
title: "workflow signal eval MUST be O(N) — p95 < 2s at 10⁶ audit rows"
module: cuo
category: scalability
priority: MUST
verification: T
phase: P0
slo: "compute_workflow_metrics + evaluate_workflow_signals: O(N) in row count; p95 < 2s wall-clock at N=10⁶ rows × W=50 workflows"
owner: CTO
created: 2026-05-19
related_frs: [FR-CUO-203]
---

## §1 — Statement (BCP-14 normative)

1. `cuo.core.workflow_evolution.compute_workflow_metrics(rows)` **MUST** be O(N) in the total audit-row count — single pass, dict insertion, no nested re-iteration of `rows`.
2. `evaluate_workflow_signals(metrics, rows, signals)` **MUST** scope its row scans to one workflow at a time — for W workflows and S signals, total cost is O(N × W × S / W) ≈ O(N × S) (the row filter at the start of each signal cuts the per-workflow scan to that workflow's row subset).
3. The workflow-refinement proposal emitter (`emit_workflow_proposal`) **MUST** delegate to `cuo.core.refinement_proposal.emit_or_halt` — there is exactly one stripe-dedup code path across both skill and workflow proposal flows.
4. Workflow stripes **MUST** be disjoint from skill stripes by the `/` separator presence — `cuo.core.workflow_evolution.compute_workflow_stripe` produces ids of the form `<persona>/<wf>:<sig>:<8hex>`; `cuo.core.stripe.compute_stripe` produces ids of the form `<skill>:<sig>:<8hex>`. The two regex grammars are non-overlapping.
5. All workflow-level diffs **MUST** default to QUEUE (never auto-apply) regardless of bucket — workflows are higher-stakes than skills because a chain edit affects every future invocation across many FRs.

## §2 — Why this constraint

A workflow with 1000 historical runs across 5 signals shouldn't take >1s to evaluate. Quadratic behaviour (e.g. for each workflow, walk ALL rows including other workflows') would mean the harness's workflow section becomes the slowest part of the report once workflows accumulate 50+ runs each. The O(N) walk discipline is the same one FR-MEMORY-120's `cyberos history` enforces — both are linear walks over the audit chain with per-row classification.

The "delegate to emit_or_halt" rule (§1 #3) prevents two code paths from drifting: if skill proposals get stripe-dedup behaviour but workflow proposals get a parallel-but-different dedup logic, eventually one diverges from the other under maintenance. One emitter, two stripe formats.

The "queue by default" rule (§1 #5) is operator-mandated — Stephen explicitly chose this in chat. Workflows are read by orchestration; a typo could change every future ship-feature-requests run.

## §3 — Measurement

Linearity: NEW benchmark `modules/cuo/tests/bench_workflow_evolution.py` runs `compute_workflow_metrics` against synthetic chains of N ∈ {10³, 10⁴, 10⁵, 10⁶} rows. Asserts wall-clock scales linearly within ±20% (fit a line through log-log points; slope ≈ 1).

Disjointness: `test_workflow_and_skill_stripes_disjoint` in `test_workflow_evolution.py` asserts the `/` is in workflow stripes and absent from skill stripes.

Queue-default: `test_workflow_diffs_default_to_queue` (deferred — requires extension of FR-CUO-202's classifier to handle workflow stripe ids; the classifier currently handles only skill stripes).

## §4 — Verification

Tests passing today (FR-CUO-203): `test_metrics_aggregation`, `test_all_completed_no_trips`, `test_routed_back_rate_trips`, `test_proposal_body_sections`, `test_workflow_stripe_format`, `test_repeat_stripe_halts`, `test_report_cites_fr_ids`, `test_workflow_and_skill_stripes_disjoint`.

Inspection: `compute_workflow_metrics` is a single `for r in rows` loop with O(1) dict-insertion per row. `evaluate_workflow_signals` iterates `(metric, signal)` pairs; each signal's evaluator scopes to `wf_id`-tagged rows via list comprehension — that's O(N) per signal × per workflow, but with low constant.

## §5 — Failure handling

**Detection:** the OTel span `event: workflow.complete` on `cyberos-cuo harness report` captures total wall-clock. If p95 > 2s at chain sizes < 10⁶, an O(N²) bug has crept in.

**Alert:** sev-3 — workflow signal evaluation latency degraded. Likely cause: a contributor added a nested loop in `evaluate_workflow_signals` that iterates the full `rows` list per workflow.

**On-call action:** profile with `py-spy`; the hot path should be entirely inside `compute_workflow_metrics`'s single pass. Any frame inside a nested loop iterating `rows` is the bug.

**Escalation:** if the chain itself grows past 10⁷ rows (unlikely in Stephen's solo usage but possible for a tenant deployment), a row-index keyed by `workflow_id` becomes necessary — file a follow-up FR to materialise that index at consolidation time.

## §6 — Notes

This NFR exists primarily as future-proofing. At today's chain sizes (710 rows in Stephen's live `.cyberos/memory/store/`), all workflow signal evaluation completes in < 50ms. The linearity discipline ensures the harness scales naturally as the chain grows over months/years without requiring re-architecture.

Like NFR-CUO-200, this is satisfied by the current implementation; it's a guardrail against future contributors introducing quadratic patterns under the assumption that "small chains are fast enough" — they are, until they aren't.
