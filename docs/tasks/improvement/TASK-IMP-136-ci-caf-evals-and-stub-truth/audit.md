---
task_id: TASK-IMP-136
audited: 2026-07-23
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_revision: 10/10
issues_resolved: 7
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean - `node tools/install/docs-tools/task-lint.mjs docs/tasks/improvement/TASK-IMP-136-ci-caf-evals-and-stub-truth/spec.md` exits 0 with zero findings, run against the audited revision
---

## §1 — Verdict summary

Six §1 clauses, six ACs, six edge cases including a security-class row and an Operator-steps row (branch protection). The heavy audit pressure was on scope honesty (the run_all.sh CI job already belongs to TASK-IMP-128), deletion discipline (never-delete-without-reason), and making the stub sweep a per-file *recorded* judgment rather than a blanket rm.

## §2 — Findings (all resolved)

### ISS-001 — plan bullet included a job an existing draft task already owns
The plan's T4 says "root CI job for scripts/tests/run_all.sh"; repo inspection found TASK-IMP-128 (draft, p1, authored 2026-07-20) specifies exactly that job with its own test file. Duplicating it here would create two specs claiming one deliverable — the two-tasks-one-name corruption the id-allocation contract exists to prevent. Resolved: scoped out in Non-Goals with the reference; recorded as a plan-vs-repo adjustment in `source_decisions`; `related_tasks` carries TASK-IMP-128.

### ISS-002 — stub deletion originally lacked the per-file judgment record
A blanket "delete the 9 stubs" violates the never-delete-without-stated-reason rule and erases the trail from each declaring task (several `done`) to its unbuilt gate. Resolved: clause 1.4 requires a 9-row disposition table in the implementation PR, deletion commits naming file + declaring task, and an implement-override rule bounded by two testable conditions (embedded YAML complete AND dependencies exist).

### ISS-003 — the regrowth guard was missing from the first draft
Deleting today's stubs leaves nothing preventing tomorrow's — the generator pattern ("auto-generated from task build_envelope references") could re-emit them. Resolved: clause 1.5(c) makes the placeholder marker itself a test failure; edge case names this as the regrowth guard and states the honest future path (real YAML or nothing).

### ISS-004 — awh hook wiring could reintroduce the documented SIGPIPE pitfall
`.githooks/pre-commit`'s header documents the exact `git diff --cached --name-only | grep -Eq` failure (grep -q SIGPIPE under pipefail silently skipping blocks). A wiring instruction that didn't name the idiom would invite the regression the hook's own comments warn about. Resolved: clause 1.2 mandates the `matches()` herestring idiom by name; AC 2 asserts no pipeline form is used.

### ISS-005 — `.pre-commit-config.yaml` removal needed a veto path
File deletion is review-sensitive; a reviewer may prefer keeping the file for contributors who run the framework standalone. Resolved: clause 1.3 records the authored default (remove) plus the explicit fallback state (non-authoritative header) so the review gate decides between two specified states rather than an unspecified middle.

### ISS-006 — branch-protection interaction was unaddressed
Deleting a workflow whose check name is required by branch protection wedges every PR — a server-side effect no repo file shows. Resolved: edge case adds Operator steps (query protection via `gh api`, confirm none of the 9 names is required, remove first if so) per ship-tasks' in-task operator-guideline rule.

### ISS-007 — CAF eval sharding could quietly subset the fixture corpus
"Shard if slow" without a floor invites running 10 of 40 fixtures and calling it green. Resolved: edge case pins "MUST NOT subset silently: all 40 run per gate invocation".

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demand | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST run both commands on PR paths + schedule; failures fail the job | workflow content (commands, triggers, schedule) + a real broken-fixture failure | AC 1: asserts file content AND the scratch-branch negative run | sufficient |
| 1.2 MUST invoke awh-gate via matches() idiom | invocation present + idiom form + non-trigger on unrelated staging | AC 2: asserts all three | sufficient |
| 1.3 MUST remove the dead config (with recorded reason) | file absence + CHANGELOG naming | AC 3: asserts both | sufficient |
| 1.4 MUST disposition every stub; none survive unchanged | zero placeholder markers + per-deletion naming vs the committed table | AC 4: asserts both halves | sufficient after revision (ISS-002) |
| 1.5 MUST assert the four truths offline; auto-registered by glob | green under run_all discovery + each assert's negative path | AC 5: self-test mode exercises all four negatives | sufficient after revision (ISS-003) |
| 1.6 MUST record the sweep in CHANGELOG | four named items present in top entry | AC 6: asserts the four substrings | sufficient |

## §4 — Resolution

Seven findings — one scope-truth, six material — all resolved in the audited revision. **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per `STATUS-REFERENCE.md` §1.1. The two human-acceptance gates in `/ship-tasks` are unchanged — this audit clears the spec-correctness gate only.

---

*End of TASK-IMP-136 audit.*
