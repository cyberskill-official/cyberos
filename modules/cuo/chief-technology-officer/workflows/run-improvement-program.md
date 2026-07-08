---
workflow_id: chief-technology-officer/run-improvement-program
workflow_version: 1.0.0
purpose: Drive any CyberOS improvement program — a backlog of enterprise-hardening tasks under `docs/improvement/<program>/` (repo-wide deep-audit, or a module pack such as memory/ or chat/) — from spec to `done`. Generic over programs — every program-specific value (backlog file plus status vocabulary, id prefix, branch, gate commands, ledger, protected invariants) is declared in that program's `program.yaml`, so one protocol runs every program without edits. The implementation half advances tasks `ready -> doing -> review` with a green gate and ledgered evidence per task; the review half moves `review -> done` under a human verdict. Distinct from `ship-feature-requests` (that drives the `docs/feature-requests/` product lifecycle); improvement programs are a separate, lighter track with their own id spaces.
persona: chief-technology-officer
cadence: per-task (loops continuously over the program backlog)
status: shipped   # CUO-workflow lifecycle: planned | shipped | retired
pattern: loop

inputs:
  - { name: program_dir,   source: workflow-caller,                                   format: "path under docs/improvement/ (e.g. docs/improvement/memory)" }
  - { name: manifest,      source: "<program_dir>/program.yaml",                       format: yaml }
  - { name: backlog,       source: "manifest.backlog.file",                            format: "yaml | markdown" }
  - { name: report,        source: "manifest.report",                                  format: markdown }
  - { name: stop_signal,   source: operator (Ctrl-C / workflow-stop event),            format: bool }

outputs:
  - { name: updated_backlog,   format: "backlog with status mutations (task -> review)",   recipient: repo HEAD (program branch) }
  - { name: implementation_diff, format: "git diff, one commit per task",                  recipient: human-reviewer (push/merge manual) }
  - { name: ledger_entries,    format: "per-task evidence (manifest.ledger)",              recipient: "manifest.ledger.path" }
  - { name: review_packet,     format: "review-packet-<date>.md (assisted review, read-only)", recipient: "<program_dir>/notes/" }

skill_chain:
  - { step: 1, skill: cyberos-improve-implement, inputs_from: { program_dir: program_dir }, outputs_to: tasks_in_review, phase: "ready -> doing -> review", note: "loops the per-task implementation gate; terminal state per task is review; never done, never push" }
  - { step: 2, skill: cyberos-improve-review,    inputs_from: { program_dir: program_dir }, outputs_to: tasks_done,      phase: "review -> done", note: "human verdict (optionally agent-prepared packet); only a human sets done" }

builds_on:
  - modules/cuo/EXECUTION-DISCIPLINE.md          # the continuous-run / halt-only-at-§2 discipline both skills enforce
---

# Run an improvement program

The official CyberOS protocol for executing an improvement backlog. The two load-bearing artifacts are the
skills, not this file:

- `.claude/skills/cyberos/cyberos-improve-implement/SKILL.md` — the implementation loop.
- `.claude/skills/cyberos/cyberos-improve-review/SKILL.md` — the review pass.
- `.claude/skills/cyberos/README.md` — the `program.yaml` schema and how to onboard a new program or a new
  repo.

## When the CTO uses it

Whenever an enterprise-grade audit has been turned into an executable backlog under `docs/improvement/<x>/`
(with a `program.yaml`), this workflow drives it. The current programs: `docs/improvement/` (repo-wide
deep-audit, `IMP-*`), `docs/improvement/memory/` (`MEM-*`), `docs/improvement/chat/` (`T-*`).

## Invocation

- Claude / Cowork: invoke the `cyberos-improve-implement` skill and name the program ("implement the memory
  backlog"); for review, `cyberos-improve-review`.
- Any other agent (Codex, etc.): hand it the skill file path plus the program dir — the skill bodies are
  self-contained Markdown.

## Guarantees (from the skills, enforced here)

- One task = one commit; a task reaches `review` only with a green gate and ledgered, revert-would-fail
  evidence.
- The agent never sets `done`, never pushes/deploys/merges, and never weakens a `program.yaml`
  `guardrails.protected` invariant to pass a gate (that is a fork → park + ledger + continue).
- Only a human moves a task `review -> done`; phase/wave exit bars and operator-only actions (push, filings,
  secrets, deploys) are the operator's.
