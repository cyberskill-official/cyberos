---
id: TASK-IMP-086
title: Backfill the improvement backlog index rows 068-081 to frontmatter truth
template: task@1
type: chore
module: improvement
status: reviewing
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-16T15:12:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-085]
routed_back_count: 0
awh: N/A
verify: I
phase: "pre-1.0.0 hardening"
owner: Stephen Cheng (CTO)
created: 2026-07-16
shipped: null
memory_chain_hash: null
effort_hours: 1
service: docs/tasks
new_files: []
modified_files:
  - docs/tasks/BACKLOG.md
source_pages:
  - "docs/tasks/BACKLOG.md improvement section (rows end at TASK-IMP-067 while folders exist through TASK-IMP-081 - fourteen tasks unindexed; found while inserting batch-1 rows on 2026-07-16)"
  - "docs/tasks/improvement/ (the folder corpus whose frontmatter is the record of truth)"
  - ".cyberos/cuo/STATUS-REFERENCE.md §1 (on any mismatch, repair the backlog toward frontmatter)"
source_decisions:
  - "2026-07-16 Stephen: PLAN batch 2 approved with this chore at p1."
---

# TASK-IMP-086: Backfill the improvement backlog index rows 068-081 to frontmatter truth

## Summary

The improvement section of docs/tasks/BACKLOG.md indexes rows through TASK-IMP-067, but the corpus on disk reaches TASK-IMP-081 - fourteen tasks (including shipped ones like 074) have no row at all, and the section header counts predate the gap. Backfill one row per missing task with the status read from its frontmatter at write time, in stem order, matching the section's existing row grammar, and reconcile the header counts against the whole section.

## Problem

Found live during batch-1's insert step: the uniqueness pre-image scan showed rows stopping at 067. An index that silently omits fourteen tasks defeats the queue (eligible work is invisible to ship-tasks' state engine) and makes the header counts fiction. STATUS-REFERENCE §1 names the repair direction:

<untrusted_content source=".cyberos/cuo/STATUS-REFERENCE.md">
docs/tasks/BACKLOG.md is the index the state engine reads and keeps in lockstep with it (on any mismatch, repair the backlog toward frontmatter).
</untrusted_content>

## Proposed Solution

A one-shot, reviewed backfill: read each missing task's frontmatter (id, title, status), emit `- [<status>] <STEM> - <title>` matching the section's existing untagged row grammar, place rows in stem-ascending order within the contiguous block, and recompute the section header's status counts from every row in the section afterward. No other section is touched; no existing row is edited beyond the header count line.

## Alternatives Considered

- Full-section regeneration via `scripts/migrate_improvement_to_task.py regen_backlog` (the byte authority). Preferred if the script runs cleanly against today's corpus - the implementer MUST try it first and fall back to the surgical backfill only if the regenerator's output would churn unrelated rows; either path must satisfy the same ACs.
- Leave it to TASK-IMP-085's insert tool run fourteen times. Rejected as a dependency: 085 ships in the same batch and coupling them serializes the batch; the chore is one review-sized diff either way.
- Do nothing (frontmatter is the truth anyway). Rejected: the queue reads the index; invisible tasks cannot be selected, and wrong counts mislead every human glance.

## Success Metrics

- Primary: index parity - every docs/tasks/improvement/TASK-IMP-* folder has exactly one row and header counts equal the per-status row tally. Baseline: 14 folders unindexed, counts stale. Deadline: this task's final acceptance.
- Guardrail: the diff touches only the improvement section (row insertions plus one header line), proven by the recorded `git diff --stat` in the gate log.

## Scope

In scope: the fourteen missing rows, the header count line, one recorded parity check.

### Out of scope / Non-Goals

- Other module sections' drift (audit them separately if wanted; this diff stays reviewable).
- Changing row grammar or section layout.
- A permanent repo-wide parity test (would go red on other sections' pre-existing drift; a follow-up once they are reconciled).

## Dependencies

- None. Cone note: this task owns docs/tasks/BACKLOG.md content during the implementing phase; the parent workflow's own phase flips also write that file but only at phase boundaries, which the batch schedule serializes.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted by the model from the batch-1 run finding; implementation follows under ship-tasks supervision.
- **Human review:** PLAN approved by the operator on 2026-07-16; spec audit and both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 Every folder matching `docs/tasks/improvement/TASK-IMP-*` MUST have exactly one row in the improvement section after this task (fourteen backfilled: 068 through 081).
- 1.2 Each backfilled row's status cell MUST equal that task's frontmatter `status` at write time, and its title MUST be the frontmatter `title` verbatim.
- 1.3 Rows MUST use the section's existing grammar (`- [<status>] <STEM> - <title>`, untagged as the section's corpus rows are) and land in stem-ascending order within the contiguous row block.
- 1.4 The section header's counts MUST be recomputed from ALL rows in the section after the backfill and match a per-status tally exactly.
- 1.5 The change MUST NOT modify any line outside the improvement section, and MUST NOT edit any pre-existing row other than the header count line.
- 1.6 A parity check MUST be recorded in the gate log: folder count equals row count, zero duplicate stems, counts match tally (the three commands and their output).

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: §1 #1.1, #1.2, #1.3) - fourteen rows present, statuses and titles from frontmatter, stem-sorted - verify: recorded parity commands in audit.md §gate-log (ops verification: a one-shot content chore; the permanent-test alternative is explicitly out of scope to avoid going red on other sections' pre-existing drift).
- [ ] AC 2 (traces_to: §1 #1.4) - header counts equal the per-status tally - verify: recorded tally command and header line in audit.md §gate-log (same ops rationale).
- [ ] AC 3 (traces_to: §1 #1.5) - diff footprint is rows + one header line inside the improvement section only - verify: recorded `git diff --stat` and hunk header list in audit.md §gate-log (same ops rationale).
- [ ] AC 4 (traces_to: §1 #1.6) - zero duplicate stems repo-wide for TASK-IMP ids - verify: recorded duplicate-scan command output in audit.md §gate-log (same ops rationale).

## 3. Edge cases

- A 068-081 task whose frontmatter status is an off-ramp (`on_hold`, `closed`): row carries that status; counts include it (AC 2).
- Title containing ` - ` (the grammar's own separator): title is emitted verbatim after the first separator; the stem token is what parsers key on (AC 1).
- The regenerator path (preferred alternative) reordering or reformatting pre-existing rows: that output violates §1 #1.5 - fall back to the surgical backfill (AC 3 catches it).
- Folder present but spec.md missing/unparseable: surface it and halt rather than inventing a row (none known today; the guard is stated so the implementer checks).
- Duplicate stem discovered during the pre-image scan: halt and surface (AC 4's scan is also the pre-check).
- Security-class: none - a tracked markdown index edit with recorded evidence; no execution surface.

## 4. Out of scope / non-goals

Duplicated intentionally with `## Scope` for template conformance: other sections, grammar changes, permanent parity tests.

## 5. Protected invariants this task must not weaken

- Frontmatter remains the record of truth; this repair flows one way, toward the index.
- No row deletion, no row reordering of pre-existing entries.
- HITL: both human-acceptance gates are recorded verdicts; the agent never sets done.

*End of TASK-IMP-086.*
