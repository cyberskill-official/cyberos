---
id: TASK-IMP-114
title: Cost and cycle telemetry - make the loop's economics visible
template: task@1
type: improvement
module: improvement
status: testing
priority: p3
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T14:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-084]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.0"
owner: Stephen Cheng (CTO)
created: 2026-07-17
memory_chain_hash: null
effort_hours: 4
service: modules/cuo
new_files:
  - tools/install/tests/test_batch_economics.sh
modified_files:
  - tools/docs-site/render-status-hub.mjs
  - modules/cuo/chief-technology-officer/workflows/ship-tasks.md
source_pages:
  - "IMPROVEMENT_HANDOFF.md §8 IMP-23, §11 IMP-31"
  - "How to build a self-improving code review agent (Zach Lloyd, 2026-07-15): couple it with metrics to see how much you are spending, how many cycles it takes, and how often the reviewer has to be corrected"
  - "This run: two API spend-limit cutoffs mid-batch, one losing three of four swarm agents"
source_decisions:
  - "2026-07-17 Stephen: PLAN gate - scope C (all 13 actionable handoff findings), template override to task@1 (recorded HITL answer)."
---

# TASK-IMP-114: Cost and cycle telemetry

## Summary

`routed_back_count` is the only cycle metric and it is never aggregated; there is no wall-time or token accounting anywhere. Add a per-batch economics row to the status page - tasks shipped, route-backs, gate re-asks, wall time, and tokens where the harness exposes them - so "is this loop worth running" has a number.

## Problem

This run's real numbers are exactly the data that would have helped before it hurt: two API spend-limit cutoffs, one batch losing three of four swarm agents mid-flight, and five batches whose relative cost nobody can compare. The articles are blunt that the loop depends on this measurement, and we have none of it.

## Proposed Solution

Emit a per-batch row: batch id, tasks shipped, total route-backs, gate re-asks, wall time from first phase commit to final acceptance, and tokens when the harness reports them. Render it on the status page beside the corpus counts. Everything derives from artefacts that already exist except tokens, which is harness-dependent and therefore optional by construction - a metric that requires a specific host is a metric that expires.

## Alternatives Considered

- Per-model cost estimates. Rejected: harness-specific, priced in dollars that change, and stale the day they are written.
- Per-step timing. Rejected as premature: the batch is the unit an operator decides about.
- A dashboard. Rejected: the status page exists and one row is the whole finding.

## Success Metrics

- Primary: after a batch, the status page shows its economics row with every non-optional field populated from artefacts - suite-asserted. Baseline: no cost or cycle data exists.
- Guardrail: a harness that reports no tokens yields a row with tokens omitted and every other field present - the row degrades, it does not vanish.

## Scope

In scope: the economics row, its derivation from existing artefacts, the status-page render, suite arms.

### Out of scope / Non-Goals

- Dollar cost estimates or per-model pricing.
- Any budget gate or spend limit - this measures, it does not stop.
- Per-step or per-skill timing (TASK-IMP-113 covers per-skill outcomes).

## Dependencies

None logically. Derives from phase commits, gate logs, and frontmatter.

**Serialisation note:** touches `render-status-hub.mjs` (shared with TASK-IMP-108) and `ship-tasks.md` (shared with 108, 109, 113, 115). Parent-serialised per §11a; never concurrent swarm members.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMPROVEMENT_HANDOFF.md §8 IMP-23; the motivating incidents are this run's own recorded agent cutoffs.
- **Human review:** scope approved at the 2026-07-17 PLAN gate; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 A per-batch economics row MUST record: batch id, tasks shipped, total route-backs, gate re-asks, and wall time from first phase commit to final acceptance.
- 1.2 Tokens MUST be recorded when the harness exposes them and MUST be omitted (not zeroed) when it does not - a zero would assert a fact nobody measured.
- 1.3 Every non-optional field MUST derive from an artefact that already exists; this task MUST NOT add a new writer to the phase path.
- 1.4 The row MUST render on the status page.
- 1.5 The row MUST NOT gate, block, or warn on any threshold.
- 1.6 Every rendered field MUST derive deterministically from committed artefacts, so a re-render of an unchanged corpus stays byte-identical per TASK-IMP-082's fp- fingerprint. A field that cannot be derived deterministically (a harness-reported token count that varies per read) MUST be omitted from the RENDERED row rather than rendered as a varying value.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.3) - a fixture batch yields a row with all non-optional fields derived from its artefacts - test: `tools/install/tests/test_batch_economics.sh::t01_row_derived`
- [ ] AC 2 (traces_to: #1.2) - a harness reporting no tokens yields a row with tokens omitted and all else present - test: `tools/install/tests/test_batch_economics.sh::t02_tokens_optional`
- [ ] AC 3 (traces_to: #1.4) - the row renders on the status page - test: `tools/docs-site/tests/test_render_status_hub.sh::t09_economics_row`
- [ ] AC 4 (traces_to: #1.5) - no threshold, gate, or warning exists on any field - verify: recorded grep in the gate log (a negative structural claim; same rationale as TASK-IMP-090 AC 1).
- [ ] AC 5 (traces_to: #1.6) - re-rendering an unchanged corpus is byte-identical, economics row included - test: `tools/docs-site/tests/test_render_status_hub.sh::t10_economics_row_deterministic`

## 3. Edge cases

- A batch cut mid-flight (this run, twice): wall time has no end. The row MUST mark it `incomplete` rather than compute a duration to now - an unfinished batch has no wall time, and inventing one would be the fabrication the whole corpus guards against.
- A batch of one task: the row is still emitted. Small batches are the comparison baseline.
- Route-backs spanning two batches (a task deferred and resumed later): counted in the batch where the route-back happened, which is where the cost fell.
- A batch with a re-run phase after a sandbox kill: wall time includes it, because it was real time spent.
- Tokens reported by one harness and not another: the rendered page would differ per-author, breaking TASK-IMP-082's byte-stability. 1.6 resolves it - the token field lives in the artefact, never in the rendered row, unless it is itself committed.
- Security-class: reads timestamps, counters, and commit metadata. Nothing executed, nothing interpolated.
