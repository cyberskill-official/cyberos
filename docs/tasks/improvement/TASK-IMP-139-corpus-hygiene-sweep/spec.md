---
id: TASK-IMP-139
title: Corpus hygiene - UNREVIEWED fork, module-case lint, stuck-WIP triage
template: task@1
type: improvement
module: improvement
status: implementing
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-23T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-100, TASK-IMP-101, TASK-IMP-108, TASK-IMP-117]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.1.0"
owner: Stephen Cheng (CTO)
created: 2026-07-23
memory_chain_hash: null
effort_hours: 12
service: docs/tasks
new_files:
  - scripts/tests/test_corpus_hygiene.sh
modified_files:
  - tools/install/docs-tools/task-lint.mjs
  - modules/skill/task-audit/RUBRIC.md
  - "docs/tasks/**/spec.md (bulk: 251 module-case normalizations; up to 170 UNREVIEWED-marker files per the fork branch chosen)"
  - CHANGELOG.md
source_pages:
  - "measured 2026-07-23: 170 non-draft spec.md files carry `# UNREVIEWED` markers (336 marker lines across them) - by status: 151 done, 12 implementing, 4 ready_to_implement, 2 closed, 1 on_hold. The markers were auto-set by the 2026-07-14 schema migration on ai_authorship / eu_ai_act_risk_class fields with the instruction 'a human MUST confirm before this task leaves draft'; the tasks left draft anyway. FM-112 (task-lint.mjs:122-124) makes the marker an error past draft. (The 2026-07-23 audit counted 167 files / 148 done; three more accrued between audit and authoring.)"
  - "measured 2026-07-23: 251 spec.md files carry a `module:` frontmatter value that is not lowercase (e.g. `module: AUTH` in docs/tasks/auth/) - the folder name is lowercase everywhere, the regenerator groups by folder, the status hub reads frontmatter; task-lint.mjs has no module-field rule at all today"
  - "measured 2026-07-23: exactly 12 tasks sit in status implementing - TASK-MCP-003/005/006/007/008 (all created_at 2026-05-17), TASK-OBS-001/003/005/007/008/009 (all 2026-05-15), and TASK-APP-001 (2026-07-14, NOT May-era). The audit report said '6 MCP, 6 OBS'; the measured truth is 5 MCP + 6 OBS + 1 APP. No forward mid-state (ready_to_review/reviewing/ready_to_test/testing) is occupied anywhere in the corpus"
  - "tools/install/docs-tools/task-reconcile.mjs (the evidence-ladder tool, TASK-IMP-100) + TASK-IMP-101 (reconcile entry + deps gating) - the designated instrument for deciding what a stale in-flight status actually reflects"
  - ".cyberos/cuo/STATUS-REFERENCE.md §1.3 (route-back is a recorded routing decision) and §1.4 (operator overrides emit memory.status_overridden)"
source_decisions:
  - "2026-07-23 operator: CyberOS Hardening Plan approved; Phase 2 T9 'Corpus hygiene' authored as an improvement task (plan file cyberos_hardening_plan_49404998; audit findings H2 + medium corpus items). The plan and this spec both mark the UNREVIEWED bulk decision as an explicit operator fork."
  - "2026-07-23 authoring: the 12-task triage is specified as PER-TASK operator verdicts via task-reconcile - the author does not pre-judge any of the twelve (notably TASK-APP-001, which is July-created and may be legitimately in flight), and this task changes no status by itself."
---

# TASK-IMP-139: Corpus hygiene - UNREVIEWED disposition, module-case lint, stuck-WIP triage

## Summary

Three corpus-integrity debts: 170 non-draft specs (151 of them `done`) still carry the `# UNREVIEWED` markers the 2026-07-14 migration attached to compliance fields with the instruction "a human MUST confirm before this task leaves draft" - so the corpus's `ai_authorship` and `eu_ai_act_risk_class` claims on shipped work are unconfirmed and every one of those files fails FM-112 if linted; 251 specs carry mixed-case `module:` values with no lint rule guarding the field; and 12 tasks sit in `implementing` - eleven since mid-May - with every forward mid-state otherwise unoccupied, evidence the full lifecycle is rarely exercised and stale WIP accumulates silently. This task dispositions the markers under an explicit operator fork (bulk-clear with recorded verdict vs re-audit wave), normalizes module case and adds the missing lint rule, and triages the 12 via `task-reconcile` with per-task operator verdicts.

## Implementation note: two operator gates inside this task

**Gate 1 - the UNREVIEWED fork (do not pre-empt).** The marker disposition is a process decision the operator must make explicitly, recorded as a dated `source_decisions` entry on this spec before any marker is touched:
- **Branch clear:** one recorded bulk verdict ("the operator accepts the migrated `ai_authorship`/`eu_ai_act_risk_class` values on the enumerated 170 files as-is") authorizes removing all markers mechanically; the verdict text ships in the commit body and CHANGELOG. Cheap, honest about being a batch acceptance, leaves per-file confirmation undone forever.
- **Branch re-audit:** a wave re-runs task-audit's compliance families over the 170 files, confirming or correcting the two fields per file before its markers drop; markers survive on any file whose fields a human has not yet confirmed. Expensive, produces real per-file confirmation.

**Gate 2 - the stuck-WIP triage (per-task verdicts).** Each of the 12 `implementing` tasks gets a `task-reconcile` evidence report (what the ladder says actually shipped vs the status), and the operator records ONE verdict per task - resume / route back per STATUS-REFERENCE §1.3 / `on_hold` - via the standard override path (which emits `memory.status_overridden` once TASK-CUO-303 lands, or its documented equivalent before). This task performs no status flip on its own authority; TASK-APP-001 (July-created) is expected to be a legitimate resume, which is exactly why the verdicts are per-task, not batch.

## Problem

Verified first-hand 2026-07-23:

1. **Compliance fields unconfirmed at scale (audit H2).** The migration marked every auto-set `ai_authorship` / `eu_ai_act_risk_class` with `# UNREVIEWED ... a human MUST confirm before this task leaves draft`. 170 files left draft anyway - 151 all the way to `done`. FM-112 exists precisely to stop this (error, marker past draft) but only fires when the linter runs against a file, which happens at authoring/audit time, not retroactively over the corpus. The EU-AI-Act-adjacent fields on shipped work are, today, machine guesses wearing confirmed clothes.
2. **`module:` field ungoverned.** 251 files say `module: AUTH`-style uppercase inside lowercase folders. Nothing breaks loudly (the regenerator groups by folder), but the field feeds the status hub and any future module-scoped tooling, and task-lint has NO rule for it - the only frontmatter field of its kind with zero validation.
3. **Stale WIP, invisible.** 11 of 12 `implementing` tasks are ~10 weeks old with zero forward mid-states occupied corpus-wide; either work stalled silently or statuses were never advanced. Both are lifecycle-integrity failures, and nothing surfaces them (the stuck-WIP sentinel is a v3.x roadmap item; this triage is the manual pass that clears today's backlog of doubt).

## Proposed Solution

**Markers:** enumerate the 170 files mechanically (the enumeration script ships in the test suite so review sees the exact set); halt at Gate 1 for the fork verdict; execute the chosen branch; either way the end state is zero `# UNREVIEWED` markers on non-draft specs whose fields carry a recorded confirmation trail, and FM-112 keeps guarding the future. **Module case:** one mechanical commit lowercasing the 251 `module:` values; add a task-lint rule (next free FM id per the rubric's numbering, documented in RUBRIC.md in the same change per the lint-and-rubric-ship-together discipline) asserting `module:` is lowercase AND equals the containing `docs/tasks/<module>/` folder name; regen + status hub verified unchanged (the field now matches what the folder always said). **Triage:** run `task-reconcile` per stuck task, attach each evidence report to the triage record, halt at Gate 2 for per-task verdicts, and apply exactly the operator's verdicts through the standard paths. **Guard:** `scripts/tests/test_corpus_hygiene.sh` pins the end states - zero non-draft markers, zero module-case mismatches, lint rule fires on fixtures - and rides `run_all.sh`'s glob so regression is loud.

## Alternatives Considered

- **Author picks the marker branch (bulk-clear, it's cheaper).** Rejected: the plan's approval boundary explicitly reserves "the T9 UNREVIEWED bulk decision" as an operator fork; a batch acceptance of compliance fields is a liability posture only the operator can adopt.
- **Silently drop the markers as lint noise.** Rejected: the marker IS the record that a human never confirmed the fields; deleting it without a verdict manufactures confirmation - the exact laundering FM-112 exists to prevent.
- **Make the module-case rule warning-severity.** Rejected: after the normalization commit the corpus is 100% conformant, so error-severity costs nothing and prevents re-drift; a warning would be the always-ignorable kind.
- **Batch verdict for the 12 stuck tasks ("route all back").** Rejected: the set is heterogeneous - TASK-APP-001 is nine days old and plausibly live; the OBS/MCP eleven may have real partial implementations the evidence ladder will surface. Twelve two-minute decisions beat one wrong batch decision.
- **Build the stuck-WIP sentinel (auto-detection) in this task.** Rejected: roadmap v3.x per the audit; this task clears the existing debt manually and leaves automated detection to its own spec (G13 in TASK-IMP-140 defines the gate; the hub work is future).

## Success Metrics

- Primary: by the next CyberOS release - zero `# UNREVIEWED` markers on non-draft specs (from 170 files / 336 markers), zero mixed-case `module:` values (from 251), the new lint rule live in task-lint + RUBRIC.md, and all 12 formerly-stuck tasks carrying a dated per-task verdict record with statuses reflecting those verdicts. Baselines as measured 2026-07-23.
- Guardrail: `python3 scripts/migrate_improvement_to_task.py --backlog` output is byte-stable across the normalization (module grouping comes from folders, which do not change), and no task's `status` changes except through a recorded Gate-2 verdict.

## Scope

In scope: the marker enumeration + fork execution, the 251-file normalization, the lint rule + rubric documentation, the 12 reconcile runs + verdict application, the hygiene test suite, CHANGELOG.

### Out of scope / Non-Goals

- Re-auditing the CONTENT of any done task beyond the two compliance fields (Branch re-audit confirms fields, not whole specs).
- The automated stuck-WIP sentinel on the status hub (v3.x; G13's definition lives in TASK-IMP-140).
- Changing FM-112's semantics or the draft-gate rules (TASK-IMP-108 owns status semantics).
- The `memory.status_overridden` emission mechanics - TASK-CUO-303; Gate-2 verdicts use whatever recorded-override path exists when this task runs.

## Dependencies

None blocking. Related: TASK-IMP-100/101 (task-reconcile, the Gate-2 instrument), TASK-IMP-108 (status semantics + FM-115/116 the reconcile verdicts may set), TASK-IMP-117 (the FM-001 conformance precedent for corpus-wide frontmatter sweeps - the module-case normalization follows its mechanical-commit pattern).

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS `task-author` skill in Cursor, as the task-authoring wave of the 2026-07-23 hardening plan.
- **Scope:** the 170/336/151 marker census, the 251 module-case census, and the 12-task list with creation dates were measured by script against the working tree at authoring time (the audit's 167/148 and "6 MCP" figures are corrected to measured values, discrepancies recorded in source_pages); no spec's status or content was modified during authoring.
- **Human review:** the hardening plan was operator-approved 2026-07-23 with the UNREVIEWED fork explicitly reserved; both gates in this spec implement that reservation.

## 1. Description (normative)

- 1.1 The marker disposition MUST NOT begin until a dated operator verdict selecting Branch clear or Branch re-audit is recorded on this spec; the enumeration of affected files (mechanically derived, 170 at authoring) MUST be attached to the verdict so the operator approves a known set, not a description.
- 1.2 After the chosen branch executes, zero non-draft spec.md files may contain `# UNREVIEWED`, and the confirmation trail MUST exist: Branch clear = the bulk verdict text in commit body + CHANGELOG; Branch re-audit = per-file confirmation in each file's audit record. Draft specs keep their markers (FM-112 permits them there; they are honest).
- 1.3 All `module:` frontmatter values MUST be lowercase and equal to the containing `docs/tasks/<module>/` folder name, via one mechanical normalization commit (251 files at authoring).
- 1.4 `task-lint.mjs` MUST gain an error-severity rule (next free FM id) enforcing 1.3's invariant, and `modules/skill/task-audit/RUBRIC.md` MUST document the rule in the same change - the lint and the rubric ship together per the authoring discipline.
- 1.5 Each of the 12 `implementing` tasks MUST receive a `task-reconcile` evidence report, and any status change MUST be applied only per a recorded per-task operator verdict (resume / route back / on_hold) through the standard override path. This task MUST NOT flip any status on its own authority, and the triage record MUST cover all 12 (a verdict may legitimately be "resume unchanged").
- 1.6 A new suite `scripts/tests/test_corpus_hygiene.sh` MUST assert: zero non-draft markers corpus-wide; zero module-case mismatches; the lint rule fires on a mixed-case fixture and on a folder-mismatch fixture and passes on a conformant one; and the backlog regenerator is byte-stable across a re-run (idempotence guard). It registers via the `run_all.sh` glob.
- 1.7 `CHANGELOG.md` MUST record the chosen marker branch, the normalization count, the new lint rule id, and the triage outcome summary (n resumed / n routed back / n held).

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - the spec carries the dated fork verdict with the attached enumeration BEFORE any marker-touching commit (verified in spec + git history at review) - test: `scripts/tests/test_corpus_hygiene.sh::t01_fork_verdict_recorded`
- [ ] AC 2 (traces_to: #1.2) - `grep -rl '# UNREVIEWED' docs/tasks/**/spec.md` intersected with non-draft statuses returns empty, and the branch-appropriate confirmation trail exists (bulk verdict text present, or per-file audit confirmations) - test: `scripts/tests/test_corpus_hygiene.sh::t02_no_nondraft_markers`
- [ ] AC 3 (traces_to: #1.3) - a corpus scan finds zero `module:` values that are non-lowercase or unequal to their folder name - test: `scripts/tests/test_corpus_hygiene.sh::t03_module_case_conformant`
- [ ] AC 4 (traces_to: #1.4) - task-lint exits non-zero naming the new rule id on a mixed-case fixture and on a folder-mismatch fixture, exits 0 on a conformant fixture, and RUBRIC.md documents the id - test: `scripts/tests/test_corpus_hygiene.sh::t04_lint_rule_fires`
- [ ] AC 5 (traces_to: #1.5) - twelve reconcile evidence reports exist, each stuck task has a dated verdict record, every status change in the triage commit maps 1:1 to a verdict, and no other task's status changed - test: `scripts/tests/test_corpus_hygiene.sh::t05_triage_verdict_per_task`
- [ ] AC 6 (traces_to: #1.6) - the suite runs green under `bash scripts/tests/run_all.sh` discovery and the regenerator produces byte-identical BACKLOG.md on consecutive runs post-normalization - test: `scripts/tests/test_corpus_hygiene.sh::t06_registered_and_idempotent`
- [ ] AC 7 (traces_to: #1.7) - CHANGELOG's top entry names the branch, the count 251, the rule id, and the triage tally - test: `scripts/tests/test_corpus_hygiene.sh::t07_changelog_records_hygiene`

## 3. Edge cases

- **New UNREVIEWED files accrue between authoring and implementation:** the enumeration is re-derived at Gate 1 (the 170 is the authoring-time census, not a frozen list); the verdict covers the re-derived set, which is why 1.1 requires attaching it.
- **A file is BOTH mixed-case and marker-bearing:** the normalization commit is field-mechanical and marker-preserving; order is normalization first (no judgment), markers second (gated) - so the mechanical half never waits on the fork.
- **`_archive/` and `_audits/` trees:** the census and the sweeps scope to `docs/tasks/*/TASK-*/spec.md` exactly as the layout suite does; archived flat files are historical record and are not rewritten.
- **A stuck task's reconcile evidence shows it actually FINISHED (code shipped, status never advanced):** the ladder surfaces it; the verdict is still the operator's - likely a forward flip through the standard gates with evidence attached, not a silent correction. The spec deliberately does not enumerate verdict outcomes beyond the three canonical ones plus "resume unchanged".
- **Branch re-audit finds a genuinely wrong risk class on a done task:** correcting the field is in scope (that is the point of the branch); anything larger it uncovers routes to a new task rather than scope-creeping this one.
- **Case-only rename collisions on case-insensitive filesystems:** no file renames occur - only frontmatter VALUES change; folder names were already lowercase, so APFS/NTFS case folding is never exercised.
- **Security-class:** text-only corpus edits + read-only evidence tooling; the compliance fields' VALUES change only under recorded human verdicts; no execution surface. The lint rule reduces spoofing surface (a task claiming module `AUTH` while living in `improvement/` now fails loudly).
