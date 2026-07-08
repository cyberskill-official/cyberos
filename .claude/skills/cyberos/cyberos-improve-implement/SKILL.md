---
name: cyberos-improve-implement
description: "Use when asked to implement, run, drive, or AUTO_WORK a CyberOS improvement program — a backlog of enterprise-hardening tasks under docs/improvement/<program>/ (e.g. memory, chat, or the repo-wide deep-audit). Triggers: \"implement the memory backlog\", \"work the chat improvement program\", \"AUTO_WORK docs/improvement\", \"drive the improvement tasks\", \"run the enterprise backlog\". This is the official CyberOS execution protocol; it is generic and reads each program's program.yaml for the specifics. For the human/agent review pass use cyberos-improve-review instead."
---

# CyberOS improvement-program implementation protocol

The one official way an agent drives any CyberOS improvement backlog to `review`. It is generic: every
program-specific value (paths, branch, id prefix, gate commands, ledger, guardrails) comes from that
program's `program.yaml` manifest, so the same protocol runs the memory, chat, deep-audit, and any future
program without edits. You supply the program; this skill supplies the discipline.

You never move a task to `done` and never push, deploy, or merge — those are the human's. Your terminal state
for a task is `review`.

## Step 0 — resolve the program and load its manifest

1. Determine the program directory from the request (e.g. `docs/improvement/memory`). If the user named a
   task id (e.g. `MEM-006`, `T-012`, `IMP-004`) but no program, infer the program from the id prefix in the
   manifests. If neither is given and only one program exists, use it; otherwise ask which program.
2. Read `<program>/program.yaml` — the adapter that declares this program's `backlog`, `id_prefix`, `branch`,
   `gates`, `ledger`, `report`, `task_specs`, `statuses` (its words mapped to the canonical lifecycle), and
   `guardrails`. If there is no `program.yaml`, tell the operator the program is not onboarded and offer to
   create one from the schema in `.claude/skills/cyberos/README.md` — do not guess the layout.
3. Everything below refers to manifest fields in `{braces}`. The canonical task lifecycle is
   `ready -> doing -> review -> done (+ blocked)`; translate to/from the program's own words via
   `{statuses}` when you read or write the backlog.

## Read first, in this order (skim; do not re-audit the module)

1. `<program>/README.md` — the program's own contract (overrides this skill where they conflict).
2. `{backlog.file}` — the single source of truth for status + eligibility.
3. The section of `{report}` named in the task's `refs:` — the *why*. The task block/card is the binding
   *what*. Read only the referenced sections; do not re-derive the report.
4. The task spec (`{task_specs}` for the task's phase/wave).
5. Then read the actual code you will touch. If the code contradicts the spec, trust the code: adjust the
   approach, record the delta in the ledger, keep the diff minimal, and do NOT rewrite the report.

## Selection rule

Pick the next task per `{selection}` (default: lowest phase/wave first; within it, highest priority, then
lowest id; a task is eligible only when its status is `ready` and every dependency is `done`). Never start a
`blocked:*` task, never reorder phases, never work two tasks at once. Flip a `blocked` task to `ready` only
when its blockers clear. If everything actionable is blocked, write a blockers summary (§ session end) and
stop.

## Per-task loop

1. **Claim.** Set the task to `doing` in `{backlog.file}` (in the same commit as your first change).
2. **Understand.** Read its spec + the `refs:` report section + the real code. Reconcile per the read rule
   above.
3. **Branch.** Work on `{branch.name}` (for `branch.mode: one-per-program`) or `{branch.name-pattern}` (for
   `one-per-task`). Create it from the latest default branch if absent; never work on the default branch.
   Migrations are expand/contract — additive only within a task; no destructive change in the same release as
   its readers.
4. **Implement exactly what the card says.** The card wins over your own ideas; do not refactor beyond its
   scope. New behavior needs tests in the same task. Copy patterns from the codebase; do not invent APIs — if
   one seems missing, verify before assuming.
5. **Self-verify continuously (no pausing).** Run every command in `{gates}` until green:
   fmt, lint (`-D warnings`), tests for touched crates/packages, migrations on a throwaway DB, plus any
   named smoke/eval and, once it exists, the program's golden/eval gate. Fix red gates yourself; a gate you
   broke is never a reason to stop. If `{gates.environment}` is `mac-gate` and this environment cannot build
   (sandbox), author the change and route the gates through the Mac-gate loop (Desktop Commander on the
   operator's machine), recording the transcript in the ledger. **No recorded green gate = the task stays
   `doing`.**
6. **Prove acceptance.** Every acceptance bullet on the card must have evidence that would fail if the change
   were reverted — a test name, a command-output summary, or a reproducible script, never "looks correct".
7. **Ledger.** Write the evidence to `{ledger}` (per-session file or single append, per the manifest):
   tasks touched, gate outputs (paste the tail, not the world), decisions, deviations, anything routed back.
   ADR-class decisions get a real file under `docs/adrs/`.
8. **Hand to review.** Set the task to `review` in `{backlog.file}` and commit as `{commit_format}` — one
   task per commit, including the status change and the ledger update in that commit.
9. **Continue** to the next eligible task.

If a task cannot pass its gate within the circuit-breaker budget (default 5 consecutive gate failures on the
same task), route it back to `ready` with a `blocked_note`, ledger the blocker, and move to the next task.

## Guardrails — the only reasons to stop (EXECUTION-DISCIPLINE §2)

Reference `modules/cuo/EXECUTION-DISCIPLINE.md` when present; the essentials are embedded here so the protocol
is portable to any repo:

- **Never** `git push`, deploy, or merge. Stop and name the action for the operator instead.
- **Never** run destructive operations on shared/staging/prod data, enter or rotate secrets, or file legal
  documents. Prepare them and hand over.
- **Never** weaken a protected invariant to get a gate green. `{guardrails.protected}` lists this program's
  (e.g. RLS fail-closed, deny-by-default access, consent default-deny, hash-chained audit, tenant isolation,
  EN+VI parity). If a task seems to require it, that is a fork: ledger it, route the task back, continue.
- **Operator-decision forks** (genuinely direction-setting, costly to reverse, ADR-class): pick the obvious
  default and record it, OR — if there is no obvious default — park the task `blocked:fork` with a 3-line
  statement of the options, ledger it, and continue with the next task. Do not wait.
- **Reporting is not pausing.** Emit a one-line milestone note (`T-0NN -> review, moving to T-0MM`) and keep
  going. "I finished X, continuing to Y" is a report; "I finished X, shall I do Y?" is a forbidden pause when
  Y is self-resolvable.
- Review checklists on the cards are for the human. You do not self-approve. `review` is your terminal state.
- Any id in `{guardrails.human_only_ids}` (legal, pen-test, decision verdicts) is never closed by an agent.

## Sanity check before your first commit each session

Working tree clean apart from your work; you are on the program's branch; `{backlog.file}` parses (for YAML
backlogs, `python -c "import yaml,sys; yaml.safe_load(open('<path>'))"`).

## Session end (or when you stop)

Write a summary to the program's session-notes location (`{ledger}` convention, e.g.
`<program>/notes/session-<date>.md` or `docs/auto-work/<date>-<program>-summary.md`): tasks moved to `review`
with one-line evidence each, tasks parked and why, blockers needing the operator, and the exact next task the
following session should pick. Leave the tree clean and committed. Then hand off to `cyberos-improve-review`.
