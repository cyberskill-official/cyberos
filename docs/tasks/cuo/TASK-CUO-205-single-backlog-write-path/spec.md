---
id: TASK-CUO-205
title: "Single backlog write path - /create-tasks delegates BACKLOG.md rows to backlog-state-update@2 (insert-row)"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: improvement
created_at: 2026-07-12T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: cuo
priority: p0
status: done
verify: T
phase: Wave C - strengthen the workflows
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: 2026-07-12
memory_chain_hash: null
related_tasks: [TASK-SKILL-118, TASK-CUO-206, TASK-CUO-208]
depends_on: []
blocks: []
source_pages:
  - tools/cyberos-install/plugin/commands/create-tasks.md
  - modules/skill/backlog-state-update-author/SKILL.md
  - modules/skill/backlog-state-update-audit/SKILL.md
source_decisions:
  - "2026-07-12 investigation: /ship-tasks writes BACKLOG.md exclusively through backlog-state-update-author/-audit (optimistic concurrency on old_line, audited mutations), while /create-tasks step 3 edits the same file inline with no skill, no audit, no concurrency gate. One file, two disciplines."
  - "Sequencing: land TASK-SKILL-118 first so the @2 contract change edits a full pair, not a thin one."
language: markdown + JSON (skill contracts + command doc)
service: modules/skill/ + tools/cyberos-install/plugin/
new_files:
  - modules/skill/backlog-state-update-author/acceptance/INSERT_ROW_CASES.md
modified_files:
  - modules/skill/backlog-state-update-author/SKILL.md
  - modules/skill/backlog-state-update-audit/SKILL.md
  - modules/skill/backlog-state-update-audit/RUBRIC.md
  - tools/cyberos-install/plugin/commands/create-tasks.md
---

# TASK-CUO-205: Single backlog write path

## §1 - Description

BACKLOG.md is the index every workflow trusts, yet only one of its two writers is governed. This task extends the backlog-state-update contract from status-cell rewrites to row insertion and routes /create-tasks through it, leaving exactly one audited write path to the file.

Normative clauses:

1. The artefact MUST bump to `backlog-state-update@2` with a `mutation_kind` enum of exactly `{status-cell-only, insert-row}`. `@1` artefacts (implicitly status-cell-only) MUST remain valid inputs to the audit during a one-release transition window.
2. `insert-row` payload MUST carry: `task_id`, `slug`, `title`, `class` (product|improvement), `status` (10-value enum), `module`, and `expected_absent: true`. Its concurrency gate is the inverse of `old_line`: the audit MUST verify no row for `task_id` existed in the pre-image and exactly one exists in the post-image.
3. The inserted row MUST match the regenerator's format byte-for-byte: `- [<status>] <task-ID-slug> - <title>` with the ` (improvement)` suffix when `class: improvement`, placed inside the module's `## <module>` section (section created per regen conventions when absent), rows kept in sorted order within the section.
4. The audit rubric MUST gain insert-row rules: BSU-INS-001 (row absent before, present once after), BSU-INS-002 (format + suffix exact), BSU-INS-003 (placed in the correct module section, sort order kept), BSU-INS-004 (no other line of the file changed except that section's header counts when they are present), BSU-INS-005 (status is a valid enum value and equals the task frontmatter status at write time).
5. `plugin/commands/create-tasks.md` step 3 MUST instruct delegation to `backlog-state-update-author` + `-audit` (one insert-row mutation per task, batched per module section allowed), and MUST NOT instruct inline editing of BACKLOG.md. The step MUST remind that task frontmatter stays the record of truth and the backlog the index.
6. /ship-tasks behavior MUST be unchanged (status-cell-only remains its only mutation kind); `workflow_complete` audit-row semantics of the existing contract are untouched.

## §2 - Why this design

Insert-row is the smallest extension that removes the ungoverned writer: same file, same optimistic-concurrency philosophy (uniqueness instead of byte-match), same audit discipline. Reusing the regenerator's row grammar means `migrate_improvement_to_task.py --backlog` remains a drop-in rebuild at any time - the inserted rows are indistinguishable from regenerated ones, so the two mechanisms can coexist without churn.

## §3 - Contract

```json
// backlog-state-update@2, mutation_kind=insert-row (input envelope fragment)
{
  "mutation_kind": "insert-row",
  "task_id": "TASK-TEN-208",
  "slug": "TASK-TEN-208-rls-audit-sweep",
  "title": "RLS audit sweep",
  "class": "improvement",
  "status": "ready_to_implement",
  "module": "ten",
  "expected_absent": true
}
```

Audit verdict: pass | fail | needs_human, score /10, findings keyed BSU-INS-00x (plus the existing status-cell rules for that kind).

## §4 - Acceptance criteria

1. **@2 enum is closed and back-compatible** (§1 #1) - the author SKILL.md declares exactly the two kinds; an @1 artefact (no mutation_kind) audits as status-cell-only with a transition note, not a failure.
2. **Duplicate insert is refused** (§1 #2) - an insert-row mutation for an task_id already present in the pre-image fails audit with BSU-INS-001.
3. **Row grammar is regenerator-identical** (§1 #3) - inserting a row then running the regenerator produces zero diff for that row (fixture asserts byte equality, both classes: with and without the improvement suffix).
4. **Section handling** (§1 #3) - insert into an existing module section keeps sort order; insert for a module with no section creates the section per regen conventions (fixture covers both).
5. **Whole-file discipline** (§1 #4) - a mutation that also touches an unrelated line fails BSU-INS-004.
6. **Status honesty** (§1 #4 BSU-INS-005) - inserting a row whose status differs from the task file's frontmatter fails audit.
7. **Command doc delegates** (§1 #5) - create-tasks.md step 3 names the pair and contains no inline-edit instruction (the current "done inline" wording is gone).
8. **Ship path untouched** (§1 #6) - the ship workflow doc and the status-cell rules are diff-clean apart from the @2 version reference.

## §5 - Verification

```markdown
# modules/skill/backlog-state-update-author/acceptance/INSERT_ROW_CASES.md
# Executable case table (each case: pre-image fixture, mutation JSON, expected verdict + finding).
CASE-01 insert into existing section, product row          -> pass          # AC 3, 4
CASE-02 insert with improvement suffix                     -> pass          # AC 3
CASE-03 duplicate task_id in pre-image                        -> fail BSU-INS-001   # AC 2
CASE-04 new module section created, sorted                 -> pass          # AC 4
CASE-05 stray edit to an unrelated line                    -> fail BSU-INS-004   # AC 5
CASE-06 row status != task frontmatter status                -> fail BSU-INS-005   # AC 6
CASE-07 @1 artefact (no mutation_kind)                     -> pass (transition note)  # AC 1
CASE-08 regenerator round-trip byte equality               -> pass          # AC 3
```

Plus doc assertions for AC 7/8: grep create-tasks.md for the delegation wording and absence of inline-edit wording; `git diff` scope check on the ship workflow doc.

## §6 - Implementation skeleton

Author SKILL.md: add the mutation_kind section, the insert payload table, and the placement algorithm (find `## <module>` header; binary-insert by row string; create section per regen template when missing). Audit SKILL.md + RUBRIC.md: add the BSU-INS family. Command doc: rewrite step 3 to the delegation form.

## §7 - Dependencies

Sequencing preference (not a hard dep): TASK-SKILL-118 first, so RUBRIC.md exists as the place BSU-INS rules land; if this task goes first, the rules land in SKILL.md prose and migrate. TASK-CUO-208 later threads template choice through the same command doc - independent sections, no conflict.

## §8 - Example payloads

Pre-image row absent; post-image gains exactly:

```
- [ready_to_implement] TASK-TEN-208-rls-audit-sweep - RLS audit sweep (improvement)
```

## §9 - Open questions

None blocking. Batched inserts (one artefact carrying N rows for one module) are permitted by §1 #5's "batched per module section allowed"; the audit treats each row against BSU-INS-001..005 individually.

## §10 - Failure modes inventory

1. Two agents insert concurrently - second audit sees the first's row in its pre-image only if it re-reads; the `expected_absent` gate plus post-image single-occurrence check turns a race into a deterministic fail-and-retry, mirroring put_if semantics from the memory protocol.
2. Module header counts drift (headers carry status counts) - BSU-INS-004 permits touching that section's header counts only; the regenerator remains the periodic reconciler.
3. Insert against a hand-mangled BACKLOG (nonstandard section headers) - placement algorithm requires the exact `## <module>` grammar; anything else -> needs_human, never a guess.
4. Class/suffix mismatch (product row with improvement suffix) - BSU-INS-002 exact-format rule catches it.
5. Transition-window ambiguity (@1 vs @2) - one release later, @1 acceptance drops; the audit SKILL.md carries the sunset date at implementation time.

## §11 - Implementation notes

Keep the placement algorithm identical to `regen_backlog()` in scripts/migrate_improvement_to_task.py (same sort key: row string ascending) - cite the function in the SKILL.md so future edits to either side know about the other.

*End of TASK-CUO-205.*
