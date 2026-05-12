# fr-with-tasks INVARIANTS

Invariants are MUST-hold conditions. Violations are boot-errors or self-audit failures.

## INV-001 — Every emitted FR includes a `tasks:` list (may be empty for purely-strategic FRs)

If `tasks:` is empty, the FR must explicitly state in its body: `_(no tasks at this stage; FR captures a strategic decision only)_`.

## INV-002 — Every task in `tasks:` carries every required field per task@1

Required fields: id, title, description, preconditions, deliverables, acceptance_test, sizing, dependencies, parallelisable, assignable_to, status. Plus `agent_profile + estimated_tokens` when `ai-agent` in assignable_to. Plus `estimated_hours` when `human` in assignable_to.

## INV-003 — Task IDs are unique within an FR and follow `FR-NNN-T-MM`

Two-digit zero-padded, sequential, starts at 01.

## INV-004 — `description` is ≥ 200 characters for every task

Forces operator comprehensiveness. The skill MUST extend a too-short description before emitting, or pause for human input.

## INV-005 — `acceptance_test` is concrete

Either `shell` (a runnable command) or `assertion` (a structured assertion). NEVER "TBD", "see PR", "TBD by reviewer", or any free-form prose lacking a check the operator can run.

## INV-006 — Dependency graph is acyclic

If A depends on B and B depends on A, refuse with self-audit failure SA-006.

## INV-007 — `parallelisable: true` requires all dependencies to be in `done` status OR empty

If a task's dependencies include any non-`done` task, `parallelisable: false` is the only valid value during the dependency window.

## INV-008 — `assignable_to` has at least one entry from `[human, ai-agent]`

An unassignable task is a malformed task.

## INV-009 — Total estimated effort fits FR target sizing

Sum of task `estimated_hours` (when human) and task estimated_tokens / 5000 (rough conversion when ai-agent) should not exceed the FR's stated sizing. If it does, the skill suggests splitting the FR.

## INV-010 — No prompt-injection markers in any task description

Run §4.2 content-gate on every task description before emitting. Markers include `[INST]`, `<system>`, `<|im_start|>`, `###Instruction`, "ignore previous instructions".

## INV-011 — Voice standard

Every task description passes `cyberos voice --strict`: no em dashes, no AI vocabulary (leverage / robust / ensure / comprehensive / seamless / delve / navigate / tapestry).

## INV-012 — Cross-references resolve

Any `<task-id>` referenced in `dependencies` must resolve to a real task in the same FR or an earlier FR.

## INV-013 — `chain_profile` field is `solo` on every emitted FR

The skill ONLY emits solo-profile FRs. If the caller's manifest declares `standard` or `full`, refuse with BOOT-005.

## INV-014 — Lifecycle starts at `draft`

Emitted tasks are `status: draft`. Status transitions happen later (via `cyberos proj sync` or operator edits).
