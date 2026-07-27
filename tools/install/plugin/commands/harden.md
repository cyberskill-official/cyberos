---
description: Remediate a lint-clean inspection-report@1 — plan HITL, work findings under scope, emit hardening-record@1, audit the session. Never push/merge/rotate.
argument-hint: "[path to inspection-report@1 markdown]"
---
Run `/harden` against ${1:-ask the user for the lint-clean inspection report path}. This command REMEDIATES inspection findings; it never discovers new findings, never pushes/merges/deploys, never rotates credentials, and is **not** ship-tasks "harden a task" (`class: improvement`).

Skills live under `${CLAUDE_PLUGIN_ROOT}/skills/` and `.cyberos/cuo/skills/` after `/install`.

1. Machine floor first — refuse to read findings until lint exits 0:
   ```
   node ${CLAUDE_PLUGIN_ROOT}/skills/harden-record-author/tools/inspect-lint.mjs <report.md>
   ```

2. Plan — ordered actor classification:
   ```
   node ${CLAUDE_PLUGIN_ROOT}/skills/harden-record-author/tools/harden-plan.mjs <report.md>
   ```
   Present the plan at the plan HITL gate (HRD-HITL-1). Honour `operator_prerequisites`: a finding that needs work outside the repo is `operator` or `split`, never pure `agent`. Get `APPROVE | REVISE | ABORT` before changing any file.

3. Author — `harden-record-author`.
- Work one finding (or one root-cause cluster) per cycle under declared scope only.
- Record verification output verbatim; silence is not approval at the review gate.
- Emit `hardening-record@1`.

4. Audit — `harden-record-audit`.
- Score against `RUBRIC.md` (HRA-*). One scope breach or silent close fails the session. Pass does not prove the defect is gone — that is an optional re-`/inspect`.

5. Report. Findings worked / not worked / operator procedures, audit verdict, and whether a re-inspect is warranted.

## Rules

- **Lint-clean input or stop.** Never work a report that fails `inspect-lint`.
- **No push, merge, deploy, or credential rotation.** Irreversible work stays with the operator.
- **Not ship-tasks.** Backlog improvement shipping is `/ship-tasks`; this command only consumes `inspection-report@1`.

Never set task `done`. If `.cyberos/` is missing, tell the user to `/install` first.
