# `plan-author` — pipeline

This document describes how `plan-author` chains with upstream and downstream skills. It is the front-door pipeline: nothing in the SDP chain runs before it, so its upstream is the operator.

## Upstream

| Upstream skill | Trigger | Hand-off |
|---|---|---|
| (none — standalone) | User hands an idea ("plan this idea", "where do we start?") | Operator provides `idea` (and `repo_root` when one exists) via the input envelope. |
| `create-tasks` (no-document case) | create-tasks is invoked with an idea instead of a `source_files` document | create-tasks routes the idea here; the emitted `plan@1` returns as its `source_files` input. |

## Mid-run sub-invocation (brownfield only)

| Skill | Trigger | Hand-off |
|---|---|---|
| `repo-context-map-author` (`scope: repo`) | Mode detection resolves `brownfield` (SKILL.md §2) | Invoked BEFORE the interview with `{repo_root, scope: repo}`; the returned `repo-context-map@1` path becomes the plan's `scan_ref`. plan-author emits no decision without it (SKILL.md §3). |

## Downstream

| Downstream skill | Trigger | Hand-off |
|---|---|---|
| `plan-audit` | Default after every emitted `plan@1` | `next_skill_recommendation: plan-audit` in the output envelope; the audit loads `plan_path`. |
| `create-tasks` → `task-author` | After `plan-audit` returns PASS (10/10) and the operator hands the plan on | `docs/plans/PLAN-<slug>-<YYYYMMDD>/plan.md` is passed as `source_files` unmodified — `## 6. Proposed Task Set` is the create-tasks input contract (SKILL.md §6). |
| (none — terminal) | User opts out of chaining | `chain_to: []` in the input envelope. |

## Event emission

This skill publishes the following NATS subjects (per `cyberos/skill/contracts/nats-subjects/`):

| subject | payload | when |
|---|---|---|
| `plan_author.plan_written` | `{plan_id, plan_path, mode, decision_confidence, gate_verdict}` | After the operator verdict is recorded and the artefact is emitted. |
| `plan_author.hitl_pause` | `{plan_id, options_count, proposed_decision, confidence}` | At the §7 decision gate — always, on every run that reaches a decision. |
| `plan_author.mode_halt` | `{repo_root, evidence}` | When mode detection resolves `ambiguous` and the skill HALTs to ask (SKILL.md §2). |
| `plan_author.aborted` | `{plan_id, reason}` | On an `ABORT` verdict at the decision gate — no artefact is written. |

## Halting and resuming

The chain halts on:

- The §7 decision gate — every run halts here; `APPROVE | REVISE: <edits> | ABORT` resumes it. This gate is structural, not exceptional.
- `ambiguous` mode (no `.cyberos/`, no git HEAD, but substantive uncommitted source) — resumes only when the operator names the mode.
- Brownfield scan unavailability — `repo-context-map-author` unreachable or failing blocks the decision (never guessed around).
- Operator interrupt.

The chain resumes when:

- The operator answers the gate (`APPROVE` emits; `REVISE` loops the interview with the edits; `ABORT` exits with no file ops).
- The operator names the mode after an `ambiguous` HALT.

## Idempotency

`plan-author` keeps no manifest. Re-running with the same idea produces a NEW gate pass — the artefact at `docs/plans/PLAN-<slug>-<YYYYMMDD>/plan.md` is only ever written under a fresh recorded verdict, and a same-day same-slug re-run overwrites only after that verdict. An `ABORT`ed run leaves the filesystem untouched (SKILL.md §7).

## Cross-references

- `../repo-context-map-author/SKILL.md` — the brownfield scan this pipeline invokes mid-run.
- `../plan-audit/SKILL.md` — the sibling audit skill that validates this skill's output.
- `../rubrics/plan_rubric.md` — `plan_rubric@1.0`, the rule source the audit walks.
- `cyberos/skill/contracts/nats-subjects/` — the NATS subject naming contract.
