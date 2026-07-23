---
description: Create tasks from a PRD, spec, or a plain idea - draft them, audit them against the rubric, and land them ready_to_implement in the backlog, ready for /ship-tasks.
argument-hint: "[path to a PRD/spec, or just describe the idea]"
---
Author and audit tasks for this repo. Input = ${1:-ask the user for the PRD/spec path, or the idea to turn into tasks}. This command CREATES the backlog; it never implements code. `/ship-tasks` is what implements.

Run the two skills in order. Both are bundled with this plugin (`${CLAUDE_PLUGIN_ROOT}/skills/`) and also vendored at `.cyberos/cuo/skills/` once `/install` has run.

1. Author - `task-author`.
- If given a PRD / spec / SRS document, expand it into task markdowns using the RESOLVED template: explicit operator override for this invocation, else `.cyberos/config.yaml` `task_template`, else default `engineering-spec@1`. Echo the resolved template (value + source) in the PLAN so the operator approves template + content together (profiles: TEMPLATE_PROFILES.md, TASK-CUO-208).
- If given only an idea (no document), use the skill's standalone interview to elicit scope, then draft from that.
- It HALTS at PLAN approval: show the user the proposed task set (ids, titles, class) and get their go-ahead before writing files. Respect that halt - do not auto-approve on their behalf.
- Write tasks to `docs/tasks/` (module subfolder in a monorepo, flat otherwise). Each starts at `status: draft`. Cross-cutting hardening work is `class: improvement`; everything else is `class: product` (the default).

2. Audit - `task-audit`.
- Audit every task just drafted against `audit_rubric@2.0` (the FM / SEC / COND / QA / SAFE / TRACE rule families).
- A clean audit drives the `draft -> ready_to_implement` transition per `STATUS-REFERENCE.md`. Write the audit as `<STEM>/audit.md` beside the spec plus the batch summary.
- It HALTS on any `needs_human` verdict. Surface those to the user and stop - do not guess the verdict.

3. Backlog. Delegate every row to `backlog-state-update-author` + `backlog-state-update-audit` - one `mutation_kind: insert-row` mutation per landed task (batching per module section allowed). Never edit `BACKLOG.md` inline: the pair is the single audited write path (same one /ship-tasks uses), with regenerator-identical row grammar (`(improvement)` suffix on `class: improvement` rows) and a uniqueness gate. Task frontmatter `status` stays the record of truth; the backlog is the index and must match it.

4. Report. List each task: id, title, class, final status, and the audit verdict. Then state the next move plainly: the tasks now at `ready_to_implement` are ready, and `/ship-tasks` will drive the next eligible one through implement -> review -> test, halting at the two human-acceptance gates.

5. Surface the material findings - `SPEC_DEFECTS_FOUND`. A batch can pass 10/10 and still have had real defects fixed on the way there; those are the most valuable output of the run and they are exactly what gets skimmed past when they are written as prose. After the report, emit ONE fenced block in the same family as `CONTRACT_ECHO` / `AUDIT_BATCH_SUMMARY`:

```
SPEC_DEFECTS_FOUND
count:        <N>
severity:     <would_have_failed | conflicts_with_shipped | unrecorded_constraint | cosmetic>
- task:       TASK-XXX-NNN
  iss:        ISS-NNN
  defect:     <one line - what was WRONG, not what was added>
  evidence:   <the file:line or artefact that proves it>
  resolved:   <the clause or AC that now covers it>
```

Emission rules:
- Emit the block whenever >= 1 finding is `material`. A finding is MATERIAL when it changed a spec's normative content: a false claim, an AC that would have failed at testing, a conflict with a shipped task, an unrecorded cross-task constraint, or a security-class gap. Wording, tone, and formatting findings are NOT material and MUST NOT be listed.
- Zero material findings: emit `SPEC_DEFECTS_FOUND` with `count: 0` and nothing else. The empty block is the signal that the check ran - silence is indistinguishable from not looking.
- The block is emitted even when every task passes 10/10. Passing is not the absence of defects; it is their resolution.
- Never pad to a count, never re-list a finding the operator already saw in an earlier batch, and never describe a defect in the language of the fix ("added X") - name what was wrong ("X was claimed and was false").
- The block is a report, not a gate. It halts nothing; the two HITL gates downstream are unchanged.

This is a REPORTING contract, not a learning loop: the model writes it by reading its own audit files. The mechanism that finds these patterns ACROSS runs, without being asked, is `workflow-improver` (TASK-IMP-110) - shipped; run it via `/improve` (bundled with this plugin, vendored at `.cyberos/cuo/skills/` after `/install`). Do not let this block imply it replaces that loop.

Never set `done`, never push, merge, or deploy. If the repo has no `.cyberos/` yet, tell the user to run `/install` first.

## Task folder scaffolding (TASK-SKILL-120 / TASK-DOCS-004)

Every new task is born as a folder: `docs/tasks/<module>/<STEM>/spec.md` (the spec, engineering-spec@1 by default per the template resolution chain) with its audit at `<STEM>/audit.md`. Media lives in the task's own `<STEM>/assets/` (created on first asset, never empty) and is referenced relatively as `assets/<file>` - never reach into another task's folder. Rendered CDS pages (TASK-DOCS-005) and the status hub pick the folder up automatically; the presentation contract is `modules/templates/contracts/TEMPLATE.md` (authoring stays markdown).
