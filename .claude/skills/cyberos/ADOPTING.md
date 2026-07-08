# Adopting the improvement-program protocol in another project

This guide takes the two CyberOS skills in this folder (`cyberos-improve-implement`,
`cyberos-improve-review`) and stands them up in a different repo so that repo can run the same gated,
one-task-one-commit hardening loop CyberOS uses. It is the cross-project companion to `README.md` (which
documents the `program.yaml` schema and onboarding a program inside cyberos itself).

Read `README.md` first for the manifest schema. This file is the ordered "do it in a new repo" procedure.

## What the protocol does and does not do

It EXECUTES an existing backlog. It does not author one. Before you adopt it in a repo you need two things
already written for that repo: a strategy or audit report, and a backlog of tasks derived from it (with a
status field, ids, and dependencies). If you do not have those yet, produce them first (that is an authoring
step, separate from this loop). The protocol then drives that backlog task by task to `review`, and a human
signs each task off to `done`.

## Step by step

### 1. Copy the skills bundle into the target repo

From a cyberos checkout (call its path `$CYBEROS`), copy the whole `cyberos/` skills folder into the target
repo's `.claude/skills/`:

```bash
mkdir -p <target-repo>/.claude/skills
cp -R "$CYBEROS/.claude/skills/cyberos" <target-repo>/.claude/skills/cyberos
```

You now have `cyberos-improve-implement/`, `cyberos-improve-review/`, `README.md`, and this `ADOPTING.md` in
the target repo. The skill bodies are self-contained Markdown and hardcode no cyberos paths, so nothing else
needs editing to make them run.

### 2. Make the skills tracked in the target repo

Many repos gitignore `.claude/` (it also holds local settings and personal skills). If yours does, add the
same precise carve-out CyberOS uses so ONLY the shared skills are tracked and local state stays ignored.
Append to the target repo's `.gitignore`:

```gitignore
# Track the shared CyberOS improvement-program skills; keep local Claude state ignored.
.claude/*
!.claude/skills/
.claude/skills/*
!.claude/skills/cyberos/
```

Verify: `git check-ignore .claude/skills/cyberos/README.md` should print nothing (not ignored), while
`git check-ignore .claude/settings.local.json` should still print a match (ignored).

### 3. Decide how the halt discipline travels

The skills embed the load-bearing halt rules (EXECUTION-DISCIPLINE section 2: what forces a stop vs. what to
work through) and additionally reference `modules/cuo/EXECUTION-DISCIPLINE.md` when that file is present. Two
options:

- Rely on the embedded rules. Do nothing; the skills still enforce the halts.
- Bring the full doctrine. Copy `EXECUTION-DISCIPLINE.md` into the target repo (for example to
  `docs/EXECUTION-DISCIPLINE.md`) if you want the complete text in-repo. Optional.

### 4. Create the program directory

Under the target repo, make `docs/improvement/<program>/` and put in it:

- a backlog file with a status field (a `backlog.yaml` or a `BACKLOG.md` table),
- task specs (phase or wave files, or one card per task),
- a short `README.md` stating the program's contract,
- the strategy or audit report the backlog operationalizes (commonly under `docs/strategy/`).

Copy the shape from a cyberos program if useful (`$CYBEROS/docs/improvement/memory/` is the yaml-backlog
shape; `$CYBEROS/docs/improvement/chat/` is the markdown-table shape).

### 5. Write the program.yaml adapter

Drop a `program.yaml` beside the backlog. Copy the full schema from `README.md` and fill it in for THIS repo.
The fields you will almost always change:

- `backlog.file` / `format` / `id_prefix` - where the backlog is and what the ids look like.
- `statuses` - map this program's status words onto the canonical `ready | doing | review | done | blocked`.
- `branch` - `one-per-program` (a single `auto/<program>` branch) or `one-per-task`.
- `commit_format` - the one-task-one-commit message shape.
- `gates.commands` - the REAL fmt, lint, test, and build commands for this repo (see step 6).
- `guardrails` - this repo's protected invariants, the operator-only actions, and any `human_only_ids`.

### 6. Set the gate environment

`gates.environment` decides where the green gate runs:

- `local` - the agent runs where the repo builds, so it runs the gate commands itself.
- `mac-gate` - the agent runs somewhere that cannot build (for example a sandbox with no toolchain, or one
  that cannot write the working tree). It authors the change, and the gate commands run on your machine (in
  CyberOS, via Desktop Commander on the operator's Mac). Put the exact commands in `gates.commands` either
  way, so whoever runs them runs the same thing.

### 7. Set the guardrails for this repo

`guardrails.protected` is the list of invariants no task may weaken just to make a gate pass (auth model,
tenant isolation, audit integrity, and so on). If a task can only go green by weakening one of these, that is
a fork: the agent parks the task, ledgers why, and moves on. `operator_only` and `human_only_ids` capture the
actions the agent prepares but never performs (push, deploy, merge, destructive migrations, secrets, legal
filings, decision verdicts).

### 8. Trigger implementation

- Claude / Claude Code / Cowork: invoke the `cyberos-improve-implement` skill and name the program, for
  example "implement the `<program>` backlog". The skill reads `program.yaml` and runs the loop.
- Codex or any other agent: hand it the path directly, for example "Follow
  `.claude/skills/cyberos/cyberos-improve-implement/SKILL.md` for `docs/improvement/<program>`."

The agent claims the next eligible task, implements it, runs the gate, ledgers the evidence, sets the task to
`review`, and commits (one task, one commit) on the program branch. It never sets `done` and never pushes.

### 9. Review and sign off

When tasks reach `review`, invoke `cyberos-improve-review` (or walk the checklist yourself). Only a human
moves a task to `done`.

### 10. Operator actions

Push, merge, deploys, and any `operator_only` items are yours to run when a phase or wave is ready. The agent
never does these.

## Quickstart checklist

```text
[ ] cp -R $CYBEROS/.claude/skills/cyberos  <target>/.claude/skills/cyberos
[ ] add the .gitignore carve-out; verify with git check-ignore
[ ] docs/improvement/<program>/  <- backlog + specs + README + report
[ ] program.yaml  <- backlog/ids/branch/commit/gates/guardrails for THIS repo
[ ] pick gates.environment (local | mac-gate) + fill real gate commands
[ ] trigger cyberos-improve-implement, name the program
[ ] review -> human sets done; operator pushes/merges
```

## Keeping in sync with cyberos

The skills are the shared, versioned asset; your `program.yaml` is yours. When the protocol improves in
cyberos, re-copy `.claude/skills/cyberos/` into the target repo (step 1) to pick up the new skill bodies. Your
`docs/improvement/<program>/` and its `program.yaml` are untouched by that copy.

## Gotchas

- One task, one commit. Do not batch tasks into a commit; the ledger and the revert-would-fail evidence are
  per task.
- Gates must run in a realistic context, not a privileged one. A gate that asserts a security property has to
  run where that property is actually enforced. (In CyberOS, the dev database user is a superuser and bypasses
  row-level security, so the RLS tests run under a separate non-superuser probe role. Find and remove the
  equivalent false-green in your repo before trusting the gate.)
- The agent never sets `done` on this track and never pushes, deploys, or merges. Those are human and operator
  actions by design.
- If a task can only pass by weakening a `guardrails.protected` invariant, that is a fork to park and ledger,
  not a change to make.

## Optional: package instead of copy

Copying the folder is the simplest path and needs no tooling. If you adopt this across many repos, consider
packaging `.claude/skills/cyberos/` as a Claude plugin (or a marketplace entry) so a new repo installs the
skills with one command and picks up updates without a manual re-copy. The per-program `program.yaml` stays
in each repo regardless.
