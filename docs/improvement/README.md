# Improvement backlog

This directory operationalizes `docs/strategy/cyberos-deep-audit-and-auto-evolution-plan-2026-07-06.md` (the audit report, R1-R52 plus the Stage 0-5 auto-evolution program). Every recommendation is broken into an executable task with its own ID, spec, and acceptance gate.

## Layout

| File | Purpose |
|---|---|
| `BACKLOG.md` | Master index: every task with wave, refs, priority, effort, deps, status. The single source of truth for status. |
| `wave-1-see-and-survive.md` | Tasks IMP-001..011: production visibility, safety nets, test spine (target: first 30 days). |
| `wave-2-measure-and-evaluate.md` | Tasks IMP-012..022: coverage, contracts, chain integrity, staging, SLOs, outcome scoring, evals (target: day 60). |
| `wave-3-widen-the-envelope.md` | Tasks IMP-023..030: dream-loop ranking and gates, auto mode, skill curation, fine-tuning pilot (target: day 90). |
| `wave-4-hardening.md` | Tasks IMP-031..045: architecture and security hardening, continuous. |
| `wave-5-platform-and-process.md` | Tasks IMP-046..062: platform operations, data, docs, governance, continuous. |
| `wave-6-go-live.md` | Tasks IMP-063..067: operationalizes `docs/deploy/go-live-guide.md` (serve a model, sign apps, activate the brain) plus the go-live readiness gate. Mostly operator work with an agent verify/gate share. |
| `PROMPT.md` | The trigger prompt for an implementation agent, plus the human review protocol. |
| `LEDGER.md` | Append-only execution ledger; every task run adds an entry. |

## Module packs in subdirectories

Module-scoped improvement packs may live alongside this repo-wide backlog in subdirectories (currently `chat/` and `memory/`, produced by the module enterprise-plan sessions). They carry their own ID spaces, task files, and trigger prompts; nothing in this file governs them. Precedence rule: for a module's internals, the module pack wins; if an IMP task turns out to duplicate a module-pack task, mark the IMP task `superseded by <pack>/<id>` in the ledger instead of doing the work twice.

## Task lifecycle

`todo -> doing -> review -> done` (plus `blocked`). The implementing agent flips `todo -> doing -> review` and updates `BACKLOG.md` in the same commit as the work. Only the human reviewer flips `review -> done`, after the checklist in `PROMPT.md` passes. If a task cannot proceed, set `blocked` with a one-line reason in the ledger and move on.

## Conventions

- IDs are `IMP-NNN` and never renumber. New tasks append at the end of the relevant wave file and to `BACKLOG.md`.
- `refs:` ties each task back to the audit report (R-numbers or Stage). Read the referenced report section before implementing; the task block is the spec, the report is the rationale.
- One task = one branch `auto/imp-NNN-short-slug` = one review packet. Small related tasks may share a branch when the spec says "pairs with".
- Acceptance checklists are binding. A task is not `review`-ready until every box is checked with evidence (test output, screenshot path, log line, commit hash).
- Evidence goes to `LEDGER.md` (format defined there) in the same commit.
- These tasks are intentionally separate from `docs/feature-requests/` (different ID space, different lifecycle). A task that grows into product scope should be converted to an FR and the IMP task marked `superseded by FR-...` in the ledger.

## How to trigger

Paste the agent prompt from `PROMPT.md` into a fresh session, or simply say:

> Implement IMP-004 per docs/improvement/PROMPT.md

With no task ID given, the agent takes the next eligible `todo` in `BACKLOG.md` order (dependencies satisfied, wave order, then ID order).

## Safety invariants (non-negotiable)

- Never push to main, deploy, or rotate a secret without the operator. The agent stops at those edges and hands over.
- Never touch the dream-loop denylist paths (auth, audit, RLS, PII, cost ledger, secrets, deploy tooling) unless the task spec explicitly names them; when it does, the review packet must call it out in the "sensitive paths" section.
- Gates are the Mac-gate loop: `cargo fmt --all --check`, `cargo clippy -p <crate> -- -D warnings`, `cargo test -p <crate>` for touched crates, plus `scripts/caf_gate.sh <module>` / `scripts/awh_ai_gate.sh <module>` where a module is involved, plus web checks for `apps/web`.
