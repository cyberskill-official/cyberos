# Task Status Reference

> Canonical answer to "what statuses can a task be in?" Every other file (RUBRIC, skill SKILL.md files, workflow `.md` files, BACKLOG.md prose) MUST defer to the list below.
>
> **Last updated:** 2026-05-19 alongside the lifecycle-simplification wave (`STATUS-WAVE-2026-05`) — collapsed the previous "tag soup" enum (in-flight + terminal-success-with-modifier + freeform `[BLOCKED: ...]` / `[FAILED: ...]` + governance) into one orthogonal lifecycle axis. Implementation-quality and failure metadata moved out of `status` into separate frontmatter fields or aux audit rows.
> **Source of truth:** this file. Other files MUST defer to the list below.

---

## 1. The full status enum

A task carries exactly one status at any point in time. There are **10** valid values, all lowercase snake_case, drawn from a single linear lifecycle axis (no embedded modifiers, no freeform tags).

### 1.1 The lifecycle (in order)

| # | Status | Meaning | Default writer |
|---|---|---|---|
| 1 | `draft` | Author has started writing the spec; not yet audited. | `task-author` |
| 2 | `ready_to_implement` | Spec passes `task-audit` at 10/10; eligible for the build queue. **Also the status a task returns to when an in-cycle step (implementing / reviewing / testing) fails or is blocked — see §1.3.** | `task-audit`, `backlog-state-update-author` (rework path) |
| 3 | `implementing` | Build is in flight; code is being written, tests partially in place. | `ship-tasks` workflow step entry |
| 4 | `ready_to_review` | Implementer finished writing code + tests; awaiting reviewer pickup. | `ship-tasks` |
| 5 | `reviewing` | Reviewer is reading the diff against §1 clauses + AC matrix. | `ship-tasks` |
| 6 | `ready_to_test` | A human reviewer recorded approval (HITL, §1.4); awaiting tester pickup. | human review verdict (via `ship-tasks`) |
| 7 | `testing` | Tester is running `coverage-gate-author` + `coverage-gate-audit` (every §1 clause's named test passes in the coverage report). | `ship-tasks` |
| 8 | `done` | Tester certified: all clauses traced to passing tests, AND a human recorded final acceptance (HITL, §1.4). Task is shipped. Terminal success. | human acceptance verdict (via `ship-tasks`) |

### 1.2 Off-ramps (operator-decided, no time pressure)

| # | Status | Meaning |
|---|---|---|
| 9 | `on_hold` | Deliberately deferred — out of scope for the current wave, will revisit later. Stays in BACKLOG.md as a future candidate. Skipped by the default `ship-tasks` queue. |
| 10 | `closed` | Terminal kill — won't be built (rejected, superseded by another task, deduplicated, won't-do). Stays in BACKLOG.md for audit-trail purposes. Skipped by the default queue. |
| 11 | `cannot_reproduce` | **`type: bug` only.** The reproduction steps do not reproduce. Terminal, but *softly*: a bug that cannot be reproduced is not a bug that does not exist, and this status says so honestly rather than laundering it through `closed`. Re-opening is a normal operator flip back to `draft`, and the audit chain shows how many times that happened — which is itself the signal that the repro is wrong, not the bug. |
| 12 | `duplicate` | Superseded by another task. REQUIRES a `duplicate_of: TASK-<ID>` frontmatter field pointing at a task that exists. Distinct from `closed` because `closed` loses the link, and the link is the whole value: it is how you discover that six reports were one cause. |

Both were added 2026-07-14 with the `type` discriminator. `closed` alone forced
every non-fix outcome through one door and destroyed the reason. A backlog where
`closed` means six different things is a backlog you cannot learn from.

### 1.3 What happened to `[FAILED: ...]` and `[BLOCKED: ...]`?

In the previous enum, these were sticky terminal statuses. They are no longer states — they are **routing decisions**:

- A circuit-breaker failure in `implementing` (e.g. 5 consecutive test failures within a task) → status drops back to `ready_to_implement`.
- A non-fatal blocker discovered during `reviewing` or `testing` (e.g. spec ambiguity, missing dependency) → status drops back to `ready_to_implement`.
- The reason text moves into an aux audit row (`memory.task_routed_back` — TBD row_kind) and/or an inline comment on the BACKLOG row.

**LANDED 2026-07-14 — the "Issue Request artefact" is a `type: bug` task.**

This section previously said:

> Future hook — Issue Request artefact (TBD): when an TASK is routed back to
> `ready_to_implement` from a downstream stage, the system will eventually auto-spawn
> an Issue Request (a new artefact type, distinct from TASK) carrying the failure
> reason, the failing test name(s), and the reverting commit hash.

That artefact needed no new type. It is a task with `type: bug`. The route-back path
now auto-drafts one, pre-filled from the evidence that already exists at the moment
of failure:

| Issue-Request field (as specified) | `type: bug` field it became |
|---|---|
| failure reason | `## Root cause` (draft) + `## Expected vs observed` |
| failing test name(s) | `regression_test` |
| reverting commit hash | `first_bad_commit` |

**Second intake path.** `services/obs-router/src/cuo_triage.rs` and
`modules/cuo/cuo/triage_server.py` already route production alerts into CUO triage.
An alert that survives triage emits a `type: bug` task with the reproduction
pre-filled from the trace. Both intake paths — a gate failing on the way out, and an
alert firing in production — now land in the same artefact, with the same rubric and
the same regression gate. That is the whole reason to have a type discriminator.

### 1.4 HITL — Human-in-the-loop is REQUIRED

Human acceptance is mandatory, not optional (see `modules/cuo/EXECUTION-DISCIPLINE.md` §2a, which governs platform-wide). The `ship-tasks` workflow drives the machine-verifiable transitions automatically, but two transitions are human-acceptance gates that the agent MUST NOT cross by itself:

- **Review acceptance** (`reviewing → ready_to_test`): a human reviewer records the approval verdict after reading the diff against the §1 clauses and the edge-case matrix.
- **Final acceptance** (`testing → done`): a human records the acceptance verdict after every machine gate (coverage, TRACE-004, awh, caf) is green. The agent NEVER self-sets `done`.

The agent brings the task up to each gate with evidence and halts; the recorded human verdict is what advances the cell. At each gate the agent MUST explain the decision in plain language, with the context the decider needs, BEFORE presenting options (see `modules/cuo/EXECUTION-DISCIPLINE.md` §2c). An operator retains the superset power to override any cell to any other cell at any time (park, resurrect, re-audit, or explicitly skip a gate for a trivial task) — that override authority is unchanged. What changed: the forward path no longer auto-crosses the two acceptance gates on green alone; a human verdict is required there.

Common operator operations:
- **Re-audit a shipped task:** flip `done` → `ready_to_review` (or `ready_to_test`) to force `ship-tasks` to re-run review + test gates from that point.
- **Skip review** for a trivial task: flip `ready_to_review` → `ready_to_test` directly (an explicit, recorded operator override).
- **Park an in-flight task:** flip `implementing` → `on_hold`.
- **Resurrect a closed task:** flip `closed` → `ready_to_implement`.

Every human verdict or override emits one `memory.status_overridden` aux audit row capturing `{actor, task_id, prior_status, new_status, reason}`. The audit chain tells the full lifecycle story, and now also proves a human accepted each task at the two mandatory gates.

---

## 2. Status transition diagram

```
                       ┌──────────────────────────────────────────────────────────┐
                       │   task-author                                 │
                       ▼                                                          │
                    draft                                                         │
                       │ task-audit (10/10)                            │
                       ▼                                                          │
                ready_to_implement ◄──────── (in-cycle fail/blocked rework) ──────┤
                       │ ship-tasks start                              │
                       ▼                                                          │
                  implementing ─────────────────► (5-test circuit breaker)  ──────┤
                       │ build complete                                           │
                       ▼                                                          │
                ready_to_review ───────────────► (reviewer rejects)         ──────┤
                       │ reviewer claims                                          │
                       ▼                                                          │
                   reviewing ─────────────────► (review uncovers blocker)   ──────┤
                       │ review approved                                          │
                       ▼                                                          │
                ready_to_test                                                     │
                       │ tester claims                                            │
                       ▼                                                          │
                    testing ─────────────────► (coverage-gate fails)        ──────┘
                       │ coverage-gate-audit 10/10
                       ▼
                     done

Off-ramps (any → on_hold | closed, operator-decided):

  any state ──────► on_hold     (deferred — not now)
  any state ──────► closed      (rejected / superseded / won't-do)

Human-acceptance gates (REQUIRED - agent must not self-cross, see §1.4):

  reviewing ──────► ready_to_test   human review verdict required
  testing   ──────► done            human acceptance verdict required

HITL overrides (any → any, operator-decided):

  any state ◄────► any state    emits memory.status_overridden aux row
```

---

## 3. Frontmatter fields adjacent to `status`

Now that `status` is a single linear axis, two pieces of metadata that used to be encoded as `shipped + <modifier>` move into their own frontmatter fields:

| Field | Type | Values | Default |
|---|---|---|---|
| `status` | enum | see §1.1 / §1.2 | `draft` |
| `implementation_kind` | enum (optional) | `real` \| `mocked` | `real` |
| `routed_back_count` | int (optional) | 0..N — increments every time the task drops to `ready_to_implement` from a downstream stage | `0` |
| `draft_reason` | enum (optional) | `authoring` \| `migrated_stub` \| `needs_spec` \| `parked_idea` — WHICH KIND of draft. Absent means unknown, which is the honest value for the 336 drafts nobody has triaged. Lint: FM-115 (TASK-IMP-108) | absent |
| `entered_via` | enum (optional) | `audit` \| `rework` \| `spec_rejected` — WHICH KIND of `ready_to_implement`. `audit` = passed 10/10, never built. `rework` = built, failed downstream, going round again (pairs with `routed_back_count > 0`). `spec_rejected` = the SPEC was wrong; see §1.3. Lint: FM-116 (TASK-IMP-108) | absent |

`implementation_kind: mocked` replaces the old `shipped + mocked-dependency` status. It means the implementation shipped against a mock service (parity-only contract test) because the real dependency isn't available. The task can still reach `done` — the mocked-ness is a property of the implementation, not the lifecycle stage. **NOTE:** decision pending — Stephen indicated "drop" for `mocked-dependency`; this field is retained here as the recommended way to capture that information if needed later. Default behaviour treats every task as `real`; if Stephen confirms total drop, this row will be removed in a follow-up patch.

---

## 4. Cross-references

- `audit_rubric@2.0` — `modules/skill/task-audit/RUBRIC.md` (FM-104 enforces the frontmatter `status:` field against the 10-value enum)
- `coverage_rubric@1.0` — `modules/skill/coverage-gate-audit/RUBRIC.md` (gates the `testing → done` transition; every §1 clause's named test must pass — the transition itself still requires the mandatory human acceptance verdict, §1.4)
- `backlog-state-update-author` skill — `modules/skill/backlog-state-update-author/SKILL.md` (writes status cells from workflow outcomes)
- `ship-tasks` workflow — `modules/cuo/chief-technology-officer/workflows/ship-tasks.md` (drives `ready_to_implement → done` and back-routes on failure)
- `task-audit` skill — `modules/skill/task-audit/SKILL.md` (drives `draft → ready_to_implement`)
- `task-audit` skill — `task-audit` skill (no-partial-ship rule §9.1)

If any of those files contradicts this reference, this file wins; please patch the contradicting file.

---

## 5. Migration notes from the previous enum

The previous (pre-2026-05-19) enum is **fully retired**. For repository archaeology, here is the mapping that `D1. BACKLOG.md mass status migration` applied:

| Old value | New value | Notes |
|---|---|---|
| `draft` | `draft` | unchanged |
| `in_review` | `ready_to_implement` | merged |
| `audited` | `ready_to_implement` | merged |
| `accepted` | `ready_to_implement` | merged |
| `building` | `implementing` | renamed |
| `in_progress` | `implementing` | legacy alias also merged |
| `shipped + strict-audited` | `done` | modifier dropped — `done` is sufficient |
| `shipped + mocked-dependency` | `done` | per Stephen's "drop" decision; if needed, set `implementation_kind: mocked` in frontmatter (§3) |
| `[FAILED: UNRESOLVABLE ERROR]` | `ready_to_implement` (with `routed_back_count += 1`, `entered_via: rework`) | now a rework path, not terminal |
| `[BLOCKED: <reason>]` | `ready_to_implement` (with `routed_back_count += 1`, `entered_via: rework`) | now a rework path, not terminal |
| `[SPEC REJECTED: <reason>]` | **`draft`** (with `routed_back_count += 1`, `entered_via: spec_rejected`) | TASK-IMP-108. When review or testing fails because the SPEC is wrong - not the code - the task MUST return to `draft` for re-authoring and re-audit. Routing it to `ready_to_implement` hands an unchanged wrong spec to an implementer, who builds the same wrong thing. §1.3 previously sent EVERY failure to `ready_to_implement`, which quietly assumed the spec was always right. |
| `deferred` | `on_hold` | renamed |
| `rejected` | `closed` | merged |
| `superseded` | `closed` | merged |

The `re_audit` mode is retired — the operator just HITL-flips `done → ready_to_review` instead, and `ship-tasks` re-runs the review + test gates naturally.

---

*End of STATUS-REFERENCE.md.*
