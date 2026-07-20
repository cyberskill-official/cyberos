---
task_id: TASK-IMP-129
audited: 2026-07-20
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 7
template: task@1
audit_rubric_version: audit_rubric@2.0
audited_body_sha256_prefix: e63177bb1090005c
audited_file_sha256_prefix: 24253bae26f5e773
machine_floor: task-lint clean (0 findings) after FM-101 fix
---

## §1 — Verdict summary

Four §1 clauses, four ACs, six edge cases including one security-class row. Machine floor clean after one title-length fix. This spec scored lowest pre-revision of the batch because its founding premise was wrong and had to be replaced before anything else could be judged — see ISS-001, which is the most consequential finding in this batch. TRACE-006 compared per clause: all four cited tests meet their clause's verb.

## §2 — Findings (all resolved)

### ISS-001 — the task's original premise was false and would have shipped a wrong fix
The task was commissioned as "uninstall destroys operator gate configuration in `gates.env`", from a directly observed regression: an uninstall/install cycle on this repo blanked `TEST_CMD` and silently reverted a same-day non-vacuous-gates fix. The observation was correct; the diagnosis was not. Reading `install.sh:324-326` before drafting showed `gates.env` is machine-owned **by design** under TASK-CUO-207, regenerated on every install on purpose, with the operator explicitly directed elsewhere for durable overrides. A spec written to the original premise would have required uninstall to preserve a file the system deliberately regenerates, contradicting a shipped task and making install's own regeneration message a lie. Material: would have failed at review against TASK-CUO-207. Resolved: reframed to the actual defect — `config.yaml`, the file all three surfaces name as the durable alternative, is itself deleted by uninstall. The correction is recorded in `source_decisions` rather than silently absorbed.

### ISS-002 — the "documented home" claim needed more than one citation to stand
The reframed premise rests on the system telling the operator that `config.yaml` is durable. One citation would make that an interpretation. Resolved: three independent surfaces cited and read at HEAD — `install.sh:326` (directs the operator there), `install.sh:328` (scaffolds once, never clobbers), `run-gates.sh:24-25` and `:75` (reads it as the override layer and instructs the operator to use it). A never-clobber promise is only meaningful if the file persists, which is the contradiction.

### ISS-003 — the deletion claim was asserted where it could be measured
"Uninstall removes it" is the load-bearing fact. Resolved: stated as a grep count (zero occurrences of `config.yaml` in `uninstall.sh`) plus the preserve-list citation at `:4` showing only the BRAIN store is kept, so the claim is checkable by a reader without running an uninstall.

### ISS-004 — XCHAIN: two adjacent tasks were undeclared
`TASK-IMP-121` (done) settled uninstall's repo-restoration for `.agents`, `.gitignore`, hooks and native channel parents; `TASK-IMP-122` (p1, on_hold) independently classifies `config.yaml` as operator-owned via `exempt:config.yaml`. Neither was in `related_tasks`. Material: unrecorded cross-task constraints, and in 122's case a corroboration that materially strengthens this task's premise. Resolved: both added, with Dependencies stating the 121 boundary (121 never reaches inside `.cyberos/`, because everything there was assumed disposable — this file is the counterexample) and the 122 corroboration (two tasks reaching the same classification from unrelated directions).

### ISS-005 — §1.4 was needed to stop the fix over-preserving
A clause requiring preservation, unbounded, invites an implementation that keeps more of `.cyberos/` than intended and quietly ends the machine-owned discipline TASK-CUO-207 established. Resolved: §1.4 binds the inverse — `gates.env` MUST still be regenerated and everything outside the documented preserve set MUST still be removed — traced by AC 4.

### ISS-006 — the autodetect half needed to be scoped so it cannot re-rank existing cases
§1.3 asks autodetect to recognise a shell test entrypoint. Unbounded, an implementer could reasonably make the shell path win over an npm script, changing behaviour for every node repo in the fleet — none of which have the defect. Resolved: §3 edge case requires existing precedence to be preserved; 1.3 fills a gap and MUST NOT re-rank cases autodetect already resolves.

### ISS-007 — "unedited config.yaml" is not distinguishable from "reset to defaults"
An implementation that preserves only *modified* config files would look like a reasonable optimisation and would silently discard a file an operator deliberately reset. Resolved: §3 edge case requires preservation regardless of edit state, naming the reason — guessing reintroduces the silent-loss failure this task exists to fix.

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demands | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST preserve AND MUST report in banner | two observables — file survives, and is named in output | AC 1: asserts file present after uninstall AND named in banner output | sufficient on both halves |
| 1.2 override MUST still be in effect | behavioural evidence after a full cycle, not mere file presence | AC 2: asserts `run-gates.sh` resolves the operator's command, not the autodetected default | sufficient — resolution is stronger than presence |
| 1.3 MUST populate TEST_CMD for a shell entrypoint | a non-empty value where "unknown" is produced today | AC 3: asserts non-empty `TEST_CMD` on a shell-only fixture | sufficient |
| 1.4 MUST still remove machine-owned paths | regeneration of `gates.env` and absence of other machine files | AC 4: asserts `gates.env` freshly regenerated and no other machine path survived | sufficient on both halves |

## §4 — Resolution

Seven findings, all material, all resolved in the audited revision. Machine floor clean. **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per STATUS-REFERENCE.md §1.1. The two human-acceptance gates downstream are unchanged and remain recorded human verdicts.

---

*End of TASK-IMP-129 audit.*
