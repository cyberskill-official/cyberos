---
task_id: TASK-IMP-139
audited: 2026-07-23
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_revision: 10/10
issues_resolved: 7
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean - `node tools/install/docs-tools/task-lint.mjs docs/tasks/improvement/TASK-IMP-139-corpus-hygiene-sweep/spec.md` exits 0 with zero findings, run against the audited revision
---

## §1 — Verdict summary

Seven §1 clauses, seven ACs, seven edge cases. Like TASK-IMP-138, this task carries operator gates INSIDE its implementation (the UNREVIEWED fork, the per-task triage verdicts) and the audit's first job was verifying the spec forces those gates mechanically rather than trusting the implementer's manners. Second job: measurement honesty - the audit-report figures (167 files, 148 done, "6 MCP") were all corrected to fresh measurements (170, 151, 5 MCP + 6 OBS + 1 APP) with the discrepancies recorded in `source_pages`.

## §2 — Findings (all resolved)

### ISS-001 — the fork could be pre-empted by an implementer with an opinion
"Operator decides the marker disposition" without an ordering clause lets a queue-picked implementer bulk-clear first and record a verdict after. Resolved: clause 1.1 makes the dated verdict + attached enumeration a MUST-precondition to any marker-touching commit; AC 1 verifies precedence via spec + git history at review.

### ISS-002 — the verdict would have covered a description, not a set
"Clear the ~170 files" approves a moving target; files accrue markers between authoring and implementation (three did between audit and authoring). Resolved: 1.1 requires attaching the re-derived enumeration to the verdict; the edge case names the re-derivation explicitly.

### ISS-003 — batch-verdict temptation on the 12 stuck tasks
The plan's phrasing ("triage the 12") invites one collective route-back - and the measured set contains TASK-APP-001, created nine days ago, plausibly live. A batch verdict would route back genuinely in-flight work. Resolved: clause 1.5 + the implementation note require per-task verdicts; Alternatives records why batch was rejected; "resume unchanged" is named as a legitimate verdict so the record can be complete without forced motion.

### ISS-004 — silent-status-change risk in the triage commit
A triage that flips statuses "per verdicts" needs a mechanical check that no OTHER status moved in the same commit. Resolved: AC 5 asserts the 1:1 verdict-to-change mapping AND that no unlisted task's status changed.

### ISS-005 — normalization vs marker-sweep ordering was undefined
Both sweeps touch overlapping files; if the gated marker work blocks the mechanical case fix, 251 files wait on a fork that has nothing to do with them. Resolved: edge case pins normalization-first (mechanical, ungated) with markers second (gated) - the fork never blocks the mechanical half.

### ISS-006 — lint rule needed the folder-equality half, not just lowercase
`module: auth` inside `docs/tasks/improvement/` is lowercase and still wrong; the real invariant is field == folder. Resolved: clause 1.3/1.4 pin both halves; AC 4 tests a folder-mismatch fixture separately from the case fixture.

### ISS-007 — case-insensitive-filesystem hazard checked and discharged
A case sweep that renamed FILES would hit APFS case-folding hazards. Verified: only frontmatter VALUES change; folders were already lowercase. Resolved as an explicit edge case so the implementer doesn't "helpfully" rename anything.

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demand | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST NOT begin markers until verdict + enumeration recorded | verdict precedence + attached set | AC 1: asserts both (review-time inspection + t01 presence) | sufficient |
| 1.2 zero non-draft markers + confirmation trail | empty census + branch-appropriate trail | AC 2: asserts both halves | sufficient |
| 1.3 MUST lowercase + equal folder | zero mismatches corpus-wide | AC 3: asserts the full invariant | sufficient after revision (ISS-006) |
| 1.4 MUST add error rule + document in RUBRIC | fires on two fixture classes, passes conformant, rubric documents id | AC 4: asserts all four | sufficient |
| 1.5 MUST reconcile all 12; changes only per verdict | 12 reports + per-task verdicts + 1:1 mapping + no stray changes | AC 5: asserts all four | sufficient after revision (ISS-004) |
| 1.6 MUST assert end states + idempotence, glob-registered | suite green under discovery + byte-stable regen | AC 6: asserts both | sufficient |
| 1.7 MUST record branch/count/rule/tally | four substrings in top entry | AC 7: asserts all four | sufficient |

## §4 — Resolution

Seven findings - three gate-integrity, one measurement-truth, three material - all resolved in the audited revision. **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per `STATUS-REFERENCE.md` §1.1 - noting that clauses 1.1 and 1.5 embed the two operator gates inside implementation, per the plan's approval boundary. The two human-acceptance gates in `/ship-tasks` are unchanged.

---

*End of TASK-IMP-139 audit.*
