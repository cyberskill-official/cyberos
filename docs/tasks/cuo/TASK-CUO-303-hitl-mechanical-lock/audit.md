---
task_id: TASK-CUO-303
audited: 2026-07-23
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_revision: 10/10
issues_resolved: 7
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean - `node tools/install/docs-tools/task-lint.mjs docs/tasks/cuo/TASK-CUO-303-hitl-mechanical-lock/spec.md` exits 0 with zero findings, run against the audited revision
---

## §1 — Verdict summary

Six §1 clauses, seven ACs, six edge cases including a security-class row. All clauses trace to ACs via `traces_to`. The audit's pressure points were authority-vs-integrity separation (refusal precedence), the audit-before-action ordering on the store-present path, and an honest statement of the frontmatter-edit bypass this task deliberately does not close. One fabricated test-suite reference was caught and corrected against the real corpus.

## §2 — Findings (all resolved)

### ISS-001 — guardrail metric cited a test suite that does not exist
The Success Metrics guardrail named `test_backlog_mutate.sh`; the backlog-mutate coverage actually lives in `tools/install/tests/test_workflow_helpers.sh` and the lifecycle spine in `tools/install/tests/test_e2e_skeleton.sh` (verified by listing `tools/install/tests/` and grepping for `backlog-mutate`). A metric anchored to a nonexistent file is unfalsifiable — the anti-fabrication class. Resolved: guardrail now names the two real suites, and both were added to `modified_files` since the e2e drives the gate transitions and must gain the flags.

### ISS-002 — refusal precedence was implied, not contractual (exit 6 vs exit 8)
If the verdict gate evaluated before the pre-image checks, a racing flip would report "verdict required" when the truth is "your pre-image drifted" — misdiagnosis that sends the operator hunting the wrong fix, and a verdict could be consumed by a doomed flip. Resolved: clause 1.2 pins the evaluation order; AC 3 constructs the drift+no-flags case and asserts exit 6, not 8.

### ISS-003 — store-present append failure was originally silent-tolerable
The first draft let the flip succeed when the row append failed, which inverts audit-before-action (§3.8 of the authoring discipline): the index would move with no audit row on a store that exists. Resolved: clause 1.4 makes append failure on a *present* store fail the whole flip; AC 5 asserts it with an unwritable store fixture, and distinguishes the legitimately store-less path (flip succeeds, stderr says the evidence file is the record).

### ISS-004 — row kind naming had to match the appender's grammar, not the doctrine's prose
STATUS-REFERENCE §1.4 says `memory.status_overridden`; the appender's existing kinds are bare (`task_routed_back`, not `memory.task_routed_back`) and become the row's `op` field. A spec that demanded the dotted form would have shipped an inconsistency with the appender's own closed-set grammar at `memory-append.mjs:102`. Resolved: clause 1.3 specifies kind `status_overridden` emitted as op `status_overridden`, "consistent with the existing four kinds" — the doctrine's dotted name is the audit-row *taxonomy* name, the appender's bare kind is the wire form, and the spec now says which is which.

### ISS-005 — the frontmatter-edit bypass was unstated in the first draft
The lock guards the tool path; an agent editing `spec.md` directly and regenerating the backlog bypasses it. Omitting that would oversell the control — exactly the doctrine-vs-enforcement gap the parent audit exists to close. Resolved: Non-Goals + edge case state the residual explicitly, cite TASK-CUO-205 (single write path) as why the tool gate is still load-bearing, and name the 1.5.0 state engine as the full closure.

### ISS-006 — evidence-file validation semantics were unpinned
"An evidence file" left directories, empty files, and unreadable paths undefined. Resolved: clause 1.1 pins exists + regular file + non-empty at flip time; AC 2 asserts missing and empty both refuse; the edge case pins directory/unreadable as "does not exist" and scopes content quality to the human reviewer, not the tool.

### ISS-007 — operator superset overrides needed an explicit boundary
STATUS-REFERENCE §1.4 grants operators any-to-any override power; a careless reading of this task could extend the verdict gate to all of them (breaking re-audit and skip-review flows) or none (breaking nothing but recording nothing). Resolved: edge case pins the boundary — exactly the two forward gate transitions are locked in this task; widening verdict recording to all overrides is named future scope.

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demand | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST refuse bare gate flips (exit 8, no write); others unchanged | refusal code + byte-identical file + flagged success + route-back flag-free | AC 1: all four asserted; AC 2: evidence validation refusals | sufficient |
| 1.2 MUST evaluate after existing refusals | constructed drift case yields 6 not 8 | AC 3: asserts the precedence directly | sufficient |
| 1.3 MUST accept status_overridden with validated payload | append succeeds complete, refuses per-missing-field, unknown kind still refused | AC 4: asserts all three behaviors | sufficient |
| 1.4 MUST append exactly one row store-present; MUST fail flip on append failure; MUST succeed store-less with stderr note | row count + payload match + unwritable-store failure + storeless success | AC 5: asserts all four halves | sufficient after revision (ISS-003) |
| 1.5 MUST stop emitting HITL_REQUIRED, keep prose | negative substring + positive prose in generated file | AC 6: asserts both halves against scratch install | sufficient |
| 1.6 MUST document flags in ship-tasks + breaking CHANGELOG | positive substrings in both files | AC 7: asserts both | sufficient |

## §4 — Resolution

Seven findings — one anti-fabrication, six material contract gaps — all resolved in the audited revision. **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per `STATUS-REFERENCE.md` §1.1. The two human-acceptance gates in `/ship-tasks` are unchanged and remain recorded human verdicts — this audit clears the spec-correctness gate only. (This task is itself the one that makes those two gates mechanically refusable.)

---

*End of TASK-CUO-303 audit.*
