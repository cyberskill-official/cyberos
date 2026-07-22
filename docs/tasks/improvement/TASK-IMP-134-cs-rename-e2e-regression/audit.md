---
task_id: TASK-IMP-134
audited: 2026-07-22
verdict: PASS (after revision)
score_pre_revision: 5/10
score_post_revision: 10/10
issues_resolved: 7
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean after one fix — first run flagged FM-101 (title 76 chars); title shortened to "End-to-end regression for the cs/memory/cuo rename"; re-run exits 0 with zero findings
---

## §1 — Verdict summary

Six §1 clauses, six ACs, three edge cases (one of which is the manual checklist itself, one a documented scope-narrowing note, one security-class). The most consequential finding was a `depends_on` error that would have blocked this task's fully-automatable portion behind an indefinite external gate for no real reason — a batch-level sequencing defect, not a wording issue.

## §2 — Findings (all resolved)

### ISS-001 — `depends_on` included TASK-IMP-133, blocking the automated portion behind an indefinite external gate
This task's clauses 1.1-1.5 need only TASK-IMP-130/131/132's CODE to exist. TASK-IMP-133's own audit (this same batch, ISS-005) established that task cannot reach `done` until a real npm release is cut and published — an operational event with no owning task and no fixed timeline. Making this task depend on TASK-IMP-133's completion would have blocked its own immediately-implementable, fully-automated test behind that indefinite wait, for a manual checklist section that isn't even gated by this task's own status transitions. Resolved: removed TASK-IMP-133 from `depends_on`; it remains a named soft-prerequisite for the manual checklist only.

### ISS-002 — AC 1 deferred to "code review" for a mechanically checkable claim
Clause 1.1 (build exactly once, before any check) is directly checkable by grepping the test file's own source for the build-invocation count and its position relative to the first check function — the same technique AC 5 already used successfully on the same file. The first draft instead said "confirmed by reading the test file's own control flow at review time." Resolved: converted to the same grep-based technique as AC 5.

### ISS-003 — AC 6 deferred to "manual review of this document" for an equally mechanical claim, with a confusing self-referential sub-clause
Clause 1.6 (the spec records a manual checklist) is checkable by grepping this very spec file for the required heading text. The original AC additionally required the checklist section "not be referenced by any traces_to in this AC list," a confusing, circular condition that added no real verification value. Resolved: converted to a plain grep-count check on the spec file itself; the confusing sub-clause was dropped rather than fixed, since it was not protecting against anything the simpler check misses.

### ISS-004 — clause 1.1's single-shared-build requirement didn't cite the concrete precedent that makes it implementable
The clause stated the principle ("one build, not per-assertion") without pointing at `test_sync_host_plugins.sh`'s own file-level build-once-then-many-checks pattern, the exact structure an implementer should copy. Resolved: the clause now cites that file directly.

### ISS-005 — AC 3's ten-verb check was vulnerable to substring false positives
A naive `grep` for each of ten command names against `cs -h`'s usage text could pass on an incidental substring match (a hypothetical future word containing "create" or similar) without the actual command name being listed. Resolved: tightened to word-boundary-safe matching (`grep -wo` or equivalent).

### ISS-006 — the guardrail metric's completion trigger was subjective
"Before the `cs` rename is announced as complete to any external user" has no fixed, checkable moment - "announced" is a judgment call, not an event. Resolved: retied to a concrete artifact already established elsewhere in this batch — the release that includes TASK-IMP-130's own CHANGELOG entry (clause 1.6 of that task) being tagged and published.

### ISS-007 — FM-101: title exceeded the 72-character limit (caught by the machine floor)
`task-lint.mjs`, run after the six findings above were resolved, flagged the title at 76 code points. Resolved: shortened to "End-to-end regression for the cs/memory/cuo rename" (title metadata only).

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demand | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST build once, shared by all checks | a countable, positioned fact about the test file's own source | AC 1 (revised): grep count + line-position check | sufficient after revision (was manual-review - ISS-002) |
| 1.2 MUST confirm bin.cs / no bin.cyberos | both halves on the built artifact | AC 2: both asserted directly | sufficient |
| 1.3 MUST list all ten verbs | precise, non-substring-vulnerable match | AC 3 (revised): word-boundary-safe | sufficient after revision (was substring-vulnerable - ISS-005) |
| 1.4 MUST confirm memory+cuo work together on shared build | both dispatches succeed in one run | AC 4: both asserted in sequence | sufficient |
| 1.5 MUST run offline | absence of network-invoking commands in the test's own source | AC 5: grep-based | sufficient |
| 1.6 MUST record manual checklist in spec body | a countable fact about this spec file's own text | AC 6 (revised): grep count on the spec file itself | sufficient after revision (was manual-review - ISS-003) |

## §4 — Resolution

Six findings, including one batch-sequencing defect (ISS-001, the most consequential), all resolved in the audited revision. **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per `STATUS-REFERENCE.md` §1.1. The two `/ship-tasks` human-acceptance gates remain unchanged; the manual release-time checklist inside this spec is explicitly NOT part of that gate and is tracked as a separate release-process step.

---

*End of TASK-IMP-134 audit.*
