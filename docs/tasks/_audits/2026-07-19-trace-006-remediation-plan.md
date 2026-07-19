# TRACE-006 remediation plan (2026-07-19)

Operator decision: remediate ALL findings. This file converts 453 findings
(85 WEAK + 368 INSUFFICIENT) into an executable program.

## The key fact: 453 findings are NOT 453 tasks

They collapse hard by root cause. Honest shape is roughly 45-60 fix-tasks
gated by 6 operator dispositions. The binding constraint is NOT agent
throughput - it is the two HITL gates per task (reviewing -> ready_to_test,
testing -> done), which only the operator can give. ~50 tasks = ~100 gate
decisions, so this is a multi-session program. Sequencing matters.

## Tier A - strengthen an existing green test (85 WEAK) - NO decisions needed

Cheapest and safest: the behaviour ships, the test is just too weak. ~20-25 tasks.

| cluster | findings | task est | note |
|---|---|---|---|
| chat audit harness | 4 | 1 | all four collapse to one fix: the integration harness runs `audit_pool: None`, so no chat clause can be proven at its sink |
| auth payload-vs-sink | 7 | 2 | auth already has the STRONG pattern (`l1_audit_log` readback) in 2 places; port it to memory_bridge + op::audit kinds |
| ai cost rows | 5 | 1 | precheck/reconcile audit rows never read back |
| ai streaming tautologies | 3 | 1 | AI-010 #3 sends events into an mpsc and asserts they return in order; #4 asserts nothing |
| ai zdr parser | 3 | 1 | tests assert against an in-test RE-IMPLEMENTATION of the parser, not shipped `init_zdr_table` |
| ai residency disjunction | 2 | 1 | match accepts a cost-miss arm, so it greens without the refusal firing |
| ai label-constant tests | 2 | 1 | assert `as_metric_label()` strings; no counter observed |
| memory dream/importance | 6 | 2 | assert returned objects / HEAD deltas instead of row op+payload |
| memory MEMORY-112 #12 | 1 | 1 | **LIVE BUG - see below. Do this first.** |
| memory acute tightenings | 9 | 2 | preserve needs before/after equality; refuse needs effect-absent |
| imp self-fulfilling tests | 2 | 1 | IMP-112 #1.1/#1.6: the test writes the artefact then asserts it exists |
| imp doctrine-greps | 4 | 1 | IMP-111 #1.2/#1.4, IMP-108 #1.5/#1.6 grep SKILL.md prose instead of driving behaviour |
| cuo halt assertions | 4 | 1 | assert a returned `StripeRepeatHalt` object, not that execution stopped |
| proj/email/obs/tpl/docs builders | 8 | 3 | pure row-builder tests standing in for sink observation |

## Tier B - write a missing test for behaviour that DOES ship

Subset of the 368 where code exists but no test binds the clause. Needs a
per-clause pass to separate from Tier C. Rough estimate 60-80 findings,
~15-20 tasks. Sequenced after Tier A because Tier A teaches the sink-assertion
pattern each of these will reuse.

## Tier C - behaviour never built as specified - 6 OPERATOR DISPOSITIONS

This is the bulk (~250-280 findings). Each cluster needs one call: BUILD it,
or AMEND the spec / close the clause. I cannot make these - they are roadmap
and cost decisions. Each disposition then fans into a handful of tasks.

1. OBSERVABILITY LAYER (~90-100 findings). No OTel span/metric assertion
   exists anywhere in services/auth; largely absent across ai, memory, skill,
   proj, email, mcp, obs. `circuit_breaker_test.rs` is the only metric-registry
   read in all of ai-gateway. Question: is OTel wiring on the roadmap, or were
   those clauses aspirational? Build = ~5-6 tasks (one per service). Amend =
   one spec-amendment sweep.
2. proj-sync + apps/web (~82). `services/proj-sync/` does not exist; `apps/web`
   has zero test files. Build vs descope.
3. cuo supervisor + services/cuo (~24). Never built; real cuo is modules/cuo.
4. skill never-built surface (~40). Vietnam crates, skill-registry/OCI,
   migrate-wrap-in, sweep-placeholders. skill-broker shipped as a thinner
   helper. NOTE: skill's 9 WEAK are blocked here too - there is no memory
   writer in skill-broker, so no skill clause can currently reach the bar.
5. auth security features (~50-60). Mixed: some impl exists but is untested
   (-> Tier B), some never built (-> Tier C). Needs a split pass. Includes
   replay/bloom, revoke deny-list, SAML assertion-sig, OIDC discovery+skew,
   passkey (no `mod tests` at all), travel/geo, HIBP (impl returns 409 not the
   spec's 422, no threshold), Lumi (on-disk lumi.rs is a different feature),
   cutover state machine.
6. mcp/email/obs named-but-unwritten suites (~25). mcp-gateway holds only
   sep986_*; email and obs suites named in specs were never written.

## Spec-hygiene fixes (cheap, fold into Tier A)

- AUTH-104 §1 has TWO clauses numbered 7 - breaks traces_to mapping. Add a lint.
- SKILL-101 drift: spec says `skill.invoked_started`, code emits
  `skill.invocation_started`.
- AUTH-005 #6 asserts `n <= 1`, satisfied at n=0 - a test that passes when
  nothing happened.

## FIRST ACTION - verified, no disposition needed

MEMORY-112 #12 is a live functional bug, confirmed directly on disk:
`cyberos/core/dream/detectors.py:202` filters `row.get("op") != "episode.logged"`,
but NOTHING in the repo produces that row - `episode.py::log` routes through
`ops.put` and emits only an `op="put"` row. The only other references are
docstrings. Consequence: the `cyberos dream` patterns detector matches zero
rows and is silently dead, and TASK-MEMORY-115's patterns detector never fires.
Fix-task: emit the `episode.logged` aux row per §1 #12, plus a sink-assertion
test (HEAD +2, op + payload) - which also closes the WEAK finding.

## Sequencing

Tier A (decision-free) starts immediately and runs in batches. Tier C
dispositions can be answered in parallel and unblock Tier B/C authoring.
Goal 4 (install everywhere) and Goal 5 (sachviet loop) remain pending and are
NOT blocked by this program - they can interleave.
