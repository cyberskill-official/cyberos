---
task_id: TASK-IMP-138
audited: 2026-07-23
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_revision: 10/10
issues_resolved: 6
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean - `node tools/install/docs-tools/task-lint.mjs docs/tasks/improvement/TASK-IMP-138-entrypoint-identity-fork/spec.md` exits 0 with zero findings, run against the audited revision
---

## §1 — Verdict summary

Seven §1 clauses, six ACs, six edge cases. The unusual shape - a spec that PASSES audit while implementation is BLOCKED on an operator fork - is deliberate and per the plan's approval boundary: the fork is presented to implementable depth on both branches, the acceptance surface is branch-independent, and clause 1.1 makes the halt mechanical for any implementer the queue routes here. The audit focused on whether the fork is genuinely undecided in the text (it must not smuggle a preference), whether the ACs survive either branch, and whether the block marker meets the §9.1-rule-5 discipline.

## §2 — Findings (all resolved)

### ISS-001 — the block was a prose note in the first draft, not a mechanical halt
"Operator decides" without a normative clause leaves the queue free to hand the task to an implementer who starts Branch A by taste. Resolved: clause 1.1 makes the recorded decision a MUST-precondition and names the halt behavior; AC 1 verifies the decision record precedes any implementation commit; the edge case declares the halt the designed outcome.

### ISS-002 — first-draft ACs were Branch-A-shaped
Early ACs asserted "root AGENTS.md is the spine" - deciding the fork through the acceptance surface, which is the same smuggling clause 1.1 forbids. Resolved: ACs 2-6 assert branch-independent invariants (first-screen reachability, truthful pointers, single normative source, branch-consistency) with per-branch arms only where the branches genuinely diverge (AC 5).

### ISS-003 — the CLAUDE.md divergence hazard was treated as future-proofing, not a live risk
AGENTS.md and CLAUDE.md are full duplicates TODAY, and the protocol has been amended four times (P19-P22) - a silent fork is one edit away, and unifying the copies during implementation could silently drop an edit one copy never received. Resolved: the edge case makes surfacing any discovered diff to the operator mandatory before unification (protocol-content territory, §0.2), and clause 1.4's drift check covers the marked-duplicate future.

### ISS-004 — Branch A's blast radius was understated
Moving the protocol's normative home touches more than three files: walker citation strings ("AGENTS.md §3"), memory docs, and the installer exception all assume the root home. Resolved: the Branch A description names the sweep explicitly; clause 1.5's Branch A arm requires it; AC 5 greps for stale assumptions.

### ISS-005 — "decision dependency" vs `depends_on` needed an explicit convention call
Encoding the fork as `depends_on` would wedge the task on a nonexistent task id; leaving it out entirely loses the block from the graph view. Resolved: Dependencies states the distinction (decision dependency, deliberately not a task-graph edge) and the body block marker carries it per §9.1 rule 5 - the queue-visible surface.

### ISS-006 — verify axis and testability tension (structural docs task with grep ACs)
A `verify: I` task whose ACs cite a shell suite risks TRACE-friction (inspection claims with test citations). Resolved: the suite is scoped to the mechanically-checkable invariants (line-window greps, marker presence, drift check), while AC 1's decision-record half is explicitly review-time inspection - each AC names its real verification mode and the `verify: I` axis reflects the task's center of gravity.

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demand | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST NOT begin until decision recorded | decision entry precedes implementation commits | AC 1: spec-history inspection + t01 asserts the entry exists | sufficient (inspection-mode, declared) |
| 1.2 MUST surface task law in first 30 lines | head-30 grep for the spine reference | AC 2: asserts the window | sufficient |
| 1.3 MUST name spine + truthful identity in six files | per-file greps incl. Branch-B negative phrasing | AC 3: asserts both halves | sufficient |
| 1.4 MUST have one normative source; copies declare | exactly-one-unmarked census + drift-check pass/fail | AC 4: asserts census AND mutated-copy failure | sufficient |
| 1.5 MUST remove exception + sweep (A) / comment it (B) | branch-scoped greps | AC 5: asserts the chosen branch's arm | sufficient |
| 1.6 MUST assert invariants, glob-registered | suite green under run_all discovery | AC 6: asserts registration | sufficient |
| 1.7 MUST record branch in CHANGELOG | substring in top entry | AC 6: asserts it | sufficient |

## §4 — Resolution

Six findings - two fork-integrity, four material - all resolved in the audited revision. **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per `STATUS-REFERENCE.md` §1.1 - with the explicit note that clause 1.1 blocks implementation on the operator fork; the task sits in the queue as the fork's forcing function, not as buildable-now work. The two human-acceptance gates in `/ship-tasks` are unchanged.

---

*End of TASK-IMP-138 audit.*
