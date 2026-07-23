---
task_id: TASK-MEMORY-303
audited: 2026-07-23
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_revision: 10/10
issues_resolved: 8
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean - `node tools/install/docs-tools/task-lint.mjs docs/tasks/memory/TASK-MEMORY-303-memory-contract-hardening/spec.md` exits 0 with zero findings, run against the audited revision
---

## §1 — Verdict summary

Eight §1 clauses, eight ACs, seven edge cases including a security-class row. The largest task in the batch; the audit pressure fell on overlap discipline with the two existing draft memory tasks (261, 302), the direction of schema unification, ordering hazards (doctor gate vs frozen store), and keeping the BRAIN-write prohibition intact through authoring. One measured-truth correction (two stray dirs, not five) and one plan-shape correction (copy census) are recorded in `source_decisions`.

## §2 — Findings (all resolved)

### ISS-001 — scope collision with TASK-MEMORY-261 and TASK-MEMORY-302
The plan's T6 bullet includes "repair live store layout"; TASK-MEMORY-261 (draft) already specifies the canonical-set single-sourcing + five-dir ADR, and TASK-MEMORY-302 (draft bug) owns the applier root cause. A spec that re-claimed either would put two tasks on one deliverable. Resolved: Non-Goals carves both out explicitly; clause 1.5 executes the repair *under 261's ADR procedure* (running its decision step first if unshipped, inside this task's HITL flow); the relationship is expressed via `related_tasks` + prose because adding `depends_on` would require editing an existing task's frontmatter for reciprocity, which is outside this authoring wave's write scope.

### ISS-002 — unification direction was assertable both ways
"Unify the copies" permits rolling StoreAcl BACK out of the package data. Resolved: `source_decisions` + Alternatives pin package-data-forward with the normative justification (§14.4.7 + shipped TASK-MEMORY-117 enforcement); clause 1.1 requires the regenerated root copy to carry the three ACL definitions, so the direction is testable, not just stated.

### ISS-003 — the drift test could keep its skip-on-missing behavior after the path fix
Fixing `_COMMITTED` while leaving `pytest.skip` on absence rebuilds today's failure mode one deletion away. Resolved: clause 1.2 makes missing-schema a FAIL; AC 2 asserts it by monkeypatching the path to a missing file.

### ISS-004 — live-store ACs were unverifiable without violating the no-BRAIN-writes rule
AC 5's first draft demonstrated the repair ON the live store, but authoring-time and CI-time verification must not mutate `.cyberos/memory/store/`. Resolved: AC 5 splits verification - the mechanical demonstration runs on a fixture store cloned from the live layout; the live-store result (doctor OK, verify OK, rows present, operator approval recorded) is verified at the human review gate, which is where an operator-gated mutation belongs.

### ISS-005 — doctor-gate ordering hazard (the gate would RED the repo before the repair lands)
Wiring 1.6 before 1.5 completes makes this repo's own gates RED mid-task. Resolved: the edge case makes repair-before-gate-wiring normative implementation order and instructs the review gate to verify it was honored.

### ISS-006 — `sessions` allowlist addition vs the `/sessions/` sandbox fragment
`_SANDBOX_FRAGMENTS` contains `/sessions/`; a careless reader (or implementer) could conflate the store-path check with the child-dir check and either break sandbox rejection or re-reject the new dir. Resolved: edge case pins the non-interference semantics and requires the walker test to cover it.

### ISS-007 — new invariants needed constructed-violation coverage, not just clean-store passes
Declaring `dream-applied-row-has-provenance`, `store-yaml-acl-valid`, and `session-lifecycle` with only happy-path tests would repeat the declared-but-unverified pattern this task exists to close. Resolved: clause 1.4 requires each to fail on a constructed violating fixture; AC 4 enumerates the three fixtures.

### ISS-008 — CLI-name collision in the doctor probe
Gating on a binary named `cyberos` reintroduces the TASK-IMP-130 PATH-collision class (an unrelated tool answering the name). Resolved: edge case requires probing module importability (`python3 -c "import cyberos.core"`), not binary presence; clause 1.6's wiring is `python3 -m cyberos doctor` accordingly.

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demand | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST regenerate; copies byte-identical + ACL-bearing | --check green + three-way hash equality + definition keys present | AC 1: asserts all three | sufficient |
| 1.2 MUST point at real path; missing MUST fail not skip | runs with 0 skips + forced-missing FAILs | AC 2: asserts both halves | sufficient after revision (ISS-003) |
| 1.3 MUST exist <= 6000 chars, five anchors, vendored | length bound + content anchors + payload presence | AC 3: asserts all three | sufficient |
| 1.4 MUST allow sessions/dreams + three invariants pass/fail correctly | layout pass + three violating fixtures each fail their own invariant | AC 4: asserts positive and all three negatives | sufficient after revision (ISS-007) |
| 1.5 MUST relocate via ledger moves under operator approval; doctor OK + chain intact | fixture relocation preserves chain; live-store outcome verified at HITL review | AC 5: split verification per ISS-004 | sufficient |
| 1.6 MUST run doctor when present, RED on FAIL, SKIP with provenance when absent | three-state behavior on scratch repos | AC 6: asserts all three states | sufficient |
| 1.7 MUST stamp iff active session | presence with active + absence without, across all three ops | AC 7: asserts both directions | sufficient |
| 1.8 MUST record in CHANGELOG | four groups named in top entry | AC 8: asserts the four substrings | sufficient |

## §4 — Resolution

Eight findings - two scope-discipline, one measured-truth, five material contract/ordering gaps - all resolved in the audited revision. **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per `STATUS-REFERENCE.md` §1.1. The two human-acceptance gates in `/ship-tasks` are unchanged - this audit clears the spec-correctness gate only. The live-store repair inside this task additionally carries its own explicit operator approval per §1.5.

---

*End of TASK-MEMORY-303 audit.*
