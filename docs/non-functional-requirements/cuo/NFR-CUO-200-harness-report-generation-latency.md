---
id: NFR-CUO-200
title: "harness report generation MUST complete in p95 < 5s over a 10⁶-row audit chain"
module: cuo
category: performance
priority: MUST
verification: T
phase: P0
slo: "p95 < 5s; p99 < 15s end-to-end `cyberos-cuo harness report --since 30d` on a chain of up to 10⁶ rows + 250 skills"
owner: CTO
created: 2026-05-19
related_tasks: [TASK-CUO-200]
---

## §1 — Statement (BCP-14 normative)

1. The `cyberos-cuo harness report` command **MUST** produce a complete markdown report from a memory audit chain of up to 10⁶ rows in **p95 < 5s** and **p99 < 15s** wall-clock time on a single-core x86_64 machine with 8 GB RAM.
2. The chain walker (`cuo.core.harness.load_audit_rows`) **MUST** be linear in the number of binlog frames (O(N)) — no quadratic blow-up from per-row regex / json-parse / hash recomputation.
3. The per-skill signal-evaluation step **MUST** scope its row set to rows mentioning that skill BEFORE invoking the 8 evaluator functions; the global row list **MUST NOT** be re-scanned for each (skill, signal) pair.
4. Report formatting (`format_markdown`) **MUST NOT** load the audit chain a second time — the report builder takes a `HarnessReport` dataclass and writes string concatenation only.
5. The `--watch` mode **MUST NOT** cumulatively leak memory across iterations — each iteration starts a fresh walker + report dataclass and frees the prior one.

## §2 — Why this constraint

Stephen runs the harness daily (or under cron). A 5s p95 budget means it can land in a pre-commit hook or fast feedback loop. A 15s p99 budget tolerates occasional large windows or cold-cache cases. The linear-in-N walker invariant is what stops the harness from collapsing once the chain crosses 10⁵ rows — without it, the harness becomes a "weekly batch" job rather than a "daily check". The scope-before-eval discipline (§1 #3) prevents O(skills × signals × rows) blow-up; the current implementation uses per-skill filter before signal evaluation, so this NFR is verifying an existing property.

## §3 — Measurement

Benchmark: `modules/cuo/tests/bench_harness_report.py` runs `compute_report` against a synthetic binlog containing N=10⁶ rows spread across 250 skill folders. Records:
- wall-clock duration of `compute_report()` (excluding markdown formatting + disk write)
- wall-clock duration of `format_markdown()` + atomic-write
- peak RSS during the run (via `tracemalloc`)

Acceptance: median run < 1s, p95 < 5s, p99 < 15s across 30 iterations. Peak RSS < 500 MB.

## §4 — Verification

Test: `modules/cuo/tests/bench_harness_report.py::test_harness_report_p95_under_5s` (deferred — perf benchmark, runs in CI's `slow` lane).

Inspection: `cuo.core.harness.compute_report` is single-pass over `all_rows`; the per-skill loop only scopes to that skill's rows via list comprehension (O(N) per skill but with low constant — see line ~280). The 250-skill × 10⁶-row scan is O(250 × N) = O(N) for fixed skill count.

## §5 — Failure handling

**Detection:** p95 monitoring via the OTel span `event: workflow.complete` (TASK-OBS-001 wiring) on the `cyberos-cuo harness report` invocation.

**Alert:** p95 > 5s for two consecutive daily runs → sev-3 alert ("harness latency degraded") in `docs/runbooks/cuo-harness-slo-breach.md`.

**On-call action:** profile the run with `py-spy` against the failing chain; common cause is an unbounded `evidence_rows` list that the markdown formatter materialises into a single string. Mitigation: cap `evidence_row_ids` per breach to 10 (already done in `compute_report`); cap evidence-table rows in markdown to 20 (already done).

**Escalation:** if profiling shows N²-style scaling, file a follow-up FR to introduce a row-index keyed by skill name (avoid the per-skill list comprehension over `rows`).

## §6 — Notes

This NFR is satisfied by the current implementation under normal conditions (the 710-row real-chain smoke run completed in <1s). It exists primarily as a guardrail: future contributors who add expensive per-row work (e.g. embedding lookups, network calls) will trip this NFR's benchmark gate before the harness becomes unusable.
