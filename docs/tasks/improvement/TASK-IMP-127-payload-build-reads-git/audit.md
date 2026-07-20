---
task_id: TASK-IMP-127
audited: 2026-07-20
verdict: PASS (after revision)
score_pre_revision: 8/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: task@1
audit_rubric_version: audit_rubric@2.0
audited_body_sha256_prefix: 72dab841e1003ab3
audited_file_sha256_prefix: 11af553572e5fc95
machine_floor: task-lint clean (0 findings)
---

## §1 — Verdict summary

Four §1 clauses, three ACs, six edge cases including one security-class row. Machine floor (`task-lint.mjs`) clean on the audited revision. Every factual claim in the spec is measured rather than inferred: the two fingerprints, the six-file diff, and the "zero content differences" claim are outputs of commands run on 2026-07-20, and the spec cites them as measurements rather than asserting them as background fact. TRACE-006 compared per clause below: all four cited tests assert at least what their clause's verb demands.

## §2 — Findings (all resolved)

### ISS-001 — XCHAIN: adjacent p1 task on the same axis was not declared
The corpus carries `TASK-IMP-122` (`rules_sha must be recomputed, not recalled`, p1, `on_hold`) governing the same fingerprint. The spec's `related_tasks` named only TASK-IMP-074, so an implementer picking this up had no signal that a second p1 task touches `rules_sha`, and could plausibly have concluded that 122 subsumed this work or vice versa. Material: an unrecorded cross-task constraint. Resolved: `related_tasks` extended to `[TASK-IMP-074, TASK-IMP-122]`, and Dependencies now carries an explicit complementarity statement with the reason neither subsumes the other — 122 governs the comparison side, this task the production side.

### ISS-002 — The claim "122 would catch this" was checked and is false
Before accepting ISS-001's framing, 122 §1.6 Direction 1 was tested against this defect: it fails the build on a path under `$CY` that no cone entry classifies. The six contaminating files sit at `cuo/gates/caf/caf/...`, beneath 122's `dir:cuo` entry, so they ARE classified and hashed — Direction 1 passes them while they corrupt the fingerprint. Had this not been checked, the two tasks could have been merged on a false premise and the defect left unfixed. Resolved: recorded in Dependencies as the specific reason 122 does not deliver this task.

### ISS-003 — §1.1's prohibition needed to bind on gitignored files, not only untracked ones
`.DS_Store` is matched by `.gitignore:15` and was still copied. A clause phrased only against "untracked" files would leave the observed defect half-uncovered, since a gitignored file may or may not also be untracked depending on history. Resolved: §1.1 binds on "untracked or gitignored", and §3 carries the inverse edge case — a gitignored path that is nonetheless tracked MUST ship, so the test is tracked-ness, not ignore-status.

### ISS-004 — §1.4 was needed to stop the fix shrinking the payload
A git-driven copy that silently omitted a tracked file would satisfy §1.1–§1.3 perfectly while breaking the product. The success metric named this as a guardrail but no clause bound it. Resolved: §1.4 added ("MUST NOT drop any tracked file the current working-tree copy produces from a clean checkout"), traced by AC 3 which compares file sets rather than sampling.

### ISS-005 — the no-git build path was an unhandled failure mode
A consumer building from a release tarball has no `.git`. Absent a clause, the natural implementation falls back to the filesystem copy — silently reinstating §1.1 in exactly the environment where nobody would notice. Resolved: §3 edge case requires that path to work or fail naming the missing repository, and explicitly forbids a silent fallback.

### ISS-006 — the dirty-tree guard needed a scope bound
§1.3 as first drafted would have failed a build because of an untracked file anywhere in the repo, including `docs/` and scratch files, making it hostile enough that an implementer would weaken it. Resolved: §3 edge case scopes the guard to the module tree the payload is built from.

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demands | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 select-from-git / MUST NOT enter | absence of the untracked file in the produced payload | AC 1: payload byte-identical to a clean-tree build with `.DS_Store` + `egg-info/` planted — byte-identity entails their absence | sufficient |
| 1.2 MUST produce byte-identical payload and fingerprint | equality of both artefacts across dirty and clean trees | AC 1: asserts both the payload bytes and `rules_sha` | sufficient |
| 1.3 MUST fail non-zero AND name paths | two observables — exit code and stderr content | AC 2: asserts non-zero exit AND that stderr names ≥1 offending path | sufficient on both halves |
| 1.4 MUST NOT drop tracked files | set equality, not sampling | AC 3: compares the full file list of git-driven vs pre-change build | sufficient |

## §4 — Resolution

Six findings, all material, all resolved in the audited revision. Machine floor clean. **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per STATUS-REFERENCE.md §1.1. The two human-acceptance gates downstream are unchanged and remain recorded human verdicts.

---

*End of TASK-IMP-127 audit.*
