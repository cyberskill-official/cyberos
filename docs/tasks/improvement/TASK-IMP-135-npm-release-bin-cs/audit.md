---
task_id: TASK-IMP-135
audited: 2026-07-23
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_revision: 10/10
issues_resolved: 6
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean — `node tools/install/docs-tools/task-lint.mjs --json …/TASK-IMP-135-npm-release-bin-cs/spec.md` exits 0 with `[]`
---

## §1 — Verdict summary

Six §1 clauses, five ACs, five edge cases including one security-class row. Closes the batch gap TASK-IMP-133 ISS-005 named: an owned, status-gating task whose `done` criterion is live registry evidence of `bin.cs`, not a scratch build.

## §2 — Findings (all resolved)

### ISS-001 — AC 3 lacked a concrete, re-runnable success filter
The first draft pointed at `gh run view` prose without requiring `conclusion=success` on the specific `npm` job for the cut tag. An implementer could paste a green overall workflow while the npm job was skipped or failed. Resolved: AC 3 now requires the `release.yml` run for `v$V` to show job `npm` with `conclusion=success`.

### ISS-002 — AC 2 would pass on an Unreleased bullet alone
"CHANGELOG mentions the rename" is true today under `## [Unreleased]` without any release cut. Resolved: AC 2 requires a dated `## [$V]` heading for the published VERSION, with the rename markers inside that section (not only under Unreleased).

### ISS-003 — clause 1.5's "must not fake publish" was prose-only
Without an AC that forbids substituting a local tarball, an implementer could `npm pack` a scratch build and call the task done. Resolved: AC 4 requires live `npm view`; AC 5 requires the recorded evidence be that `npm view` stdout and forbids flipping IMP-133 in the same commit.

### ISS-004 — laptop `npm publish` was only discouraged in Alternatives, not a non-goal
A hurried implementer reading only Scope could still try a token publish when OIDC failed. Resolved: added an explicit Out of scope bullet forbidding local `npm publish`, inventing `NPM_TOKEN`, or renaming `release.yml`.

### ISS-005 — reciprocity with TASK-IMP-133 was one-sided in the first draft body
The draft said this task blocks IMP-133, but IMP-133's frontmatter still listed only `depends_on: [TASK-IMP-130]` and its Scope still said "Publishing … is TASK-IMP-130's job." Resolved: IMP-133 `depends_on` gains TASK-IMP-135; Scope/Dependencies prose updated to name this task as the publish owner; IMP-130's `blocks` list gains TASK-IMP-135.

### ISS-006 — projected `1.1.0` risked becoming a hard-coded false requirement
Pinning AC text to the literal string `1.1.0` would fail a legitimate `Release-As: 1.2.0` cut that still ships `bin.cs`. Resolved: normative target is "first published version whose bin is cs"; ACs bind to `V=$(cat VERSION)` / `npm view @…@$V`, with `1.1.0` retained only as the projected default from `cyberos-version.mjs --check`.

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demand | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST land on main via PR | ancestor check + bin string on origin/main | AC 1 | sufficient |
| 1.2 MUST cut VERSION + dated CHANGELOG | dated section contains rename | AC 2 (revised) | sufficient after ISS-002 |
| 1.3 MUST tag v$V at bump commit | tag points at VERSION commit | AC 3 | sufficient |
| 1.4 MUST publish via release.yml npm/OIDC | npm job conclusion=success | AC 3 (revised) | sufficient after ISS-001 |
| 1.5 MUST prove live bin.cs | npm view bin keys | AC 4 + AC 5 | sufficient after ISS-003 |
| 1.6 MUST NOT done without 1.5; MUST NOT close 133 | status discipline | AC 5 | sufficient |

## §4 — Resolution

Six findings resolved in the audited revision. **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per `STATUS-REFERENCE.md` §1.1. Execution still requires a green PR merge to `main` and a successful OIDC publish — operator HALT conditions remain if either is blocked.

---

*End of TASK-IMP-135 audit.*
