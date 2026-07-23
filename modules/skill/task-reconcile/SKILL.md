---
# ── Identity ─────────────────────────────────────────────────────────
name: task-reconcile
description: >-
  Evidence ladder for a task whose status claims work ship-tasks did not
  perform — already implemented mid-shipping or long "done", with no run
  manifest, a manifest that fails verify, or a missing phase artefact set.
  Runs `docs-tools/task-reconcile.mjs` as its machine floor and emits
  `reconcile-report@1`: per-rung verdicts (spec integrity, artefact set,
  manifest, committed-object presence, cited tests) and exactly ONE
  recommendation — resume_at_phase, route_back, or adopt_candidate. Use when
  entering a drifted task per ship-tasks' Reconcile entry § (v2.7.0), or when
  an operator asks whether existing work should be reworked or built upon.
  Read-only; the agent NEVER executes a recommendation without the recorded
  human verdict. Do NOT use when a valid ship-manifest exists — resume
  semantics own that task.
skill_version: 1.0.0
artefact: reconcile-report@1
tool: docs-tools/task-reconcile.mjs
hitl: required

# ── Untrusted-content discipline ─────────────────────────────────────
untrusted_inputs:
  wrap_in_marker: "untrusted_content"
  injection_scan: required
  on_marker_hit: surface_to_human
---

# task-reconcile - measure a task whose status claims work this workflow cannot vouch for

## When this skill runs

ship-tasks trusts its own manifests (hash-verified resume) and its own gates (route-back). This skill covers the third state: a task that arrives **already implemented** - status past `ready_to_implement` with no manifest, a manifest that fails verify, or a phase artefact set that does not exist. Mid-shipping or long-shipped, audited or never audited: if the claim outruns the evidence, reconcile measures the gap before anyone builds on it.

It does NOT run when a valid ship-manifest exists - resume semantics own that task (`ship-tasks.md`, Resume semantics §). Reconcile is for work this workflow did not perform.

## Machine floor first

Run the tool BEFORE forming any opinion:

```
node .cyberos/docs-tools/task-reconcile.mjs <task-ID> --run-tests
```

It is read-only (rungs 1-4 execute nothing; rung 5 runs only the suite files the spec's own §2 cites) and emits `reconcile-report@1`: per-rung verdicts, a drift score, and exactly one recommendation. The rungs are mechanical and the model does not re-derive them:

| Rung | Question it answers |
|---|---|
| R1 spec integrity | Does the lint pass, does an audit exist at `pass`, and does the audit still describe the spec's normative half? |
| R2 artefact set | Does the phase set implied by the claimed status exist (either artefact home)? |
| R3 manifest | Is there a run manifest, and does it verify? |
| R4 committed object | Is every claimed deliverable present at HEAD - or only in a working view (the TASK-IMP-086 class)? |
| R5 cited tests | Do the spec's own cited suites exist and pass NOW? |

## What the model judges

The tool produces the verdicts; the model produces the *understanding*:

- **Read the reds in context.** "Cited suite fails now" on a task whose module was refactored by a later task is a different story from the same red on untouched code. Say which story the evidence supports - and say when it supports neither.
- **Weigh the binding gap honestly.** A note that the audit's sha matches no committed version is an evidence-hygiene problem, not proof the spec drifted; the normative-half comparison is the substantive answer. Do not upgrade a note into a verdict.
- **Draft the gate question.** State the claimed status, the recommendation, the two or three facts that drive it, and what each branch costs the operator. One screen, no padding.
- **Name what reconcile cannot see.** Passing rungs mean the *evidence* is consistent - not that the design is right. If the work looks sound and the spec looks wrong, say so; that is a judgment the ladder has no rung for.

## The hard rule

**The agent NEVER executes a recommendation - resume, route back, or adopt - without the recorded human verdict.** The report is evidence for a decision, never the decision. This is a third, conditional human gate; the two acceptance gates (reviewing -> ready_to_test, testing -> done) are untouched and still apply afterwards.

## The fork

| Recommendation | Meaning | On a human YES |
|---|---|---|
| `resume_at_phase(N)` | every rung supports the claim | re-enter the chain at step N; the run continues normally |
| `route_back` | a load-bearing rung is red | `ready_to_implement`, `routed_back_count += 1` (STATUS-REFERENCE §1.3), reasons recorded from the report |
| `adopt_candidate` | deliverables green at HEAD, artefacts missing | backfill the phase artefact set from the evidence, then re-enter at the verified phase |
| `not_applicable` | status is `draft` / `ready_to_implement` | nothing claimed - the normal chain applies |

Every executed branch emits its memory row (`task_routed_back` on route-back; `memory.status_overridden` when the human overrides the recommendation).
