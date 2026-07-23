---
task_id: TASK-IMP-140
audited: 2026-07-23
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_revision: 10/10
issues_resolved: 7
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean - `node tools/install/docs-tools/task-lint.mjs docs/tasks/improvement/TASK-IMP-140-benchmark-gates-drift-protection/spec.md` exits 0 with zero findings, run against the audited revision
---

## §1 — Verdict summary

Seven §1 clauses, seven ACs, seven edge cases, plus two embedded content contracts (the full G1-G16 definitions and the seven risk-register rows) that make the spec self-contained per the coordinator's instruction. The audit's pressure points: one-owner-per-gate discipline (nine of sixteen checkers belong to sibling tasks and must not be re-implemented here), the green-at-HEAD paradox for checkers that measure unfixed defects, and protocol-legality of the BRAIN recording step.

## §2 — Findings (all resolved)

### ISS-001 — checker ownership was ambiguous in the first draft (sixteen gates, one suite)
Implementing all sixteen checkers here would duplicate the G1/G2/G7-G12/G14/G15 checkers that the sibling hardening tasks ship as their own test suites - two authorities per gate, guaranteed drift. Resolved: `source_decisions` records the one-gate-one-checker-one-owner rule; every embedded gate definition carries an Owner line; clause 1.2 scopes this task's suite to exactly the six unowned checkers; the doc's status table is the coordination surface.

### ISS-002 — the suite could not be green at HEAD if its checkers enforce unshipped fixes
G3's enum fork and G6's vendored-CAF structural failure are real defects today, owned by sibling tasks; a suite that enforces them fails `run_all.sh` repo-wide the day it lands (and the pre-commit hook would block every commit). Resolved: clause 1.3 defines report-only mode with the doc's status table as the declared state and the flip-to-enforcing tied to the owning fix; AC 3 asserts green-at-HEAD with report-only gates declared.

### ISS-003 — the BRAIN recording step was protocol-illegal as first drafted
Recording "as soon as possible" would write to a store that `cyberos doctor` reports below READY - §12 forbids it and §1's pre-write checklist halts on it; the audit's own record would violate the protocol it measured. Resolved: `depends_on: [TASK-MEMORY-303]` (with the reciprocal `blocks` entry on 303), and clause 1.6 carries an explicit READY-precondition so even a manual override cannot execute the clause against a frozen store without visibly breaking the spec.

### ISS-004 — G13 could be read as an auto-triage
A detector that flips stale tasks to `on_hold` "helpfully" is a status mutation without a verdict - the exact class TASK-CUO-303 locks. Resolved: the G13 definition pins detect+human tier; clause 1.4 forbids status changes; AC 4 asserts byte-identical specs after a detector run.

### ISS-005 — G5's walker would false-positive on illustrative paths
Vendored docs legitimately mention example paths that are not delivery promises; a naive walker fails on them and someone "fixes" it by weakening the walk. Resolved: edge case defines the inline exemption marker with the allowlist-with-reasons pattern (visible in diffs, greppable), keeping the walker strict for real references.

### ISS-006 — G16's "modulo timestamps" could silently grow
An open-ended diff-exclusion list is how an idempotency check becomes a tautology. Resolved: edge case requires the exclusion list to live in the checker with a per-entry comment - growth is reviewable, not silent.

### ISS-007 — transcript provenance vs §11 untrusted-content discipline
The gate definitions originate in a conversation transcript, which is untrusted-by-protocol for authorizing anything. Resolved: the definitions are embedded here as the operator-approved plan's Phase-3 content (the plan approval is the authorization; the transcript is provenance), normalized to testable shape - and the Alternatives entry records why verbatim transcript import was rejected.

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demand | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST carry 16 gates, 7 fields each, status table, matching this spec | section census + field completeness + severity/tier match | AC 1: asserts all three | sufficient |
| 1.2 MUST implement six checkers, each failing on violation, glob-registered | six negative fixtures + discovery | AC 2: asserts the negatives; AC 3/AC 6 cover discovery + modes | sufficient |
| 1.3 report-only MUST be loud + declared; suite green at HEAD | green suite + report blocks + table declaration | AC 3: asserts all three | sufficient after revision (ISS-002) |
| 1.4 MUST NOT change status | byte-identical corpus after run | AC 4: asserts it directly | sufficient after revision (ISS-004) |
| 1.5 MUST add seven complete R-EXT rows with G-references | row count + field completeness + G-reference presence | AC 5: asserts all three | sufficient |
| 1.6 MUST record post-303 through canonical writer, READY-gated | fixture-store demonstration in CI + live-store verification at final acceptance | AC 6: asserts the fixture half; the live half is the HITL gate's evidence | sufficient after revision (ISS-003) |
| 1.7 MUST record four deliverables | four substrings in top entry | AC 7: asserts all four | sufficient |

## §4 — Resolution

Seven findings - two protocol-legality, five material - all resolved in the audited revision. **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per `STATUS-REFERENCE.md` §1.1 - with the note that the `depends_on: [TASK-MEMORY-303]` edge gates clause 1.6 (and final acceptance), while clauses 1.1-1.5/1.7 are buildable immediately. The two human-acceptance gates in `/ship-tasks` are unchanged - this audit clears the spec-correctness gate only.

---

*End of TASK-IMP-140 audit.*
