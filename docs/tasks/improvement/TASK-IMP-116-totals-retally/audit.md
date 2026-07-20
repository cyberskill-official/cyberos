---
task_id: TASK-IMP-116
audited: 2026-07-17
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: task@1
audit_rubric_version: audit_rubric@2.0
audited_file_sha256: d2ba0b35fe2237da
audited_body_sha256_prefix: ec457c0fa96665ca
machine_floor: task-lint clean (exit 0) - run FIRST per TASK-IMP-084
---

## §1 - Verdict summary

Spec is 96 lines, 7 §1 clauses, 7 ACs, 6 edge cases. It amends a contract three tests pin, which is the reason it exists as a task rather than an edit. Every claim is verified against the live tree: the P1's numbers, TASK-IMP-092's rationale at backlog-mutate.mjs:146, the three pins, and the regenerator's FileNotFoundError. Passes after 6 findings.

## §2 - Findings (all resolved)

### ISS-001 - the task could read as overruling a gate rather than amending a contract
Rewriting t06/t07/t11 to let a fix through is how a gate quietly stops gating, and the first attempt at this fix did exactly that before the suite refused it. Resolved: Alternatives records "move the pins without amending the contract" as explicitly rejected; §1 #1.6 makes the new footprint normative and requires the skill's contract to state it; AC 7 verifies the prose moved with the code.

### ISS-002 - widening a mutation's footprint invites further widening
Two lines became three; nothing stopped it becoming five. Resolved: §1 #1.6 caps it at exactly three and says "MUST NOT grow further"; AC 3 asserts the footprint, so a fourth line reds.

### ISS-003 - a no-op write would churn the file
Rewriting an already-correct Totals on every mutation adds a diff line to every commit that changes nothing - the churn TASK-IMP-082 spent a task removing from the status page. Resolved: §1 #1.7 requires byte-identical + no `totals_line` report when already correct; AC 5 pins it.

### ISS-004 - three counting rules could drift apart
retallyHeader, regen_backlog, and this would each decide what counts as a row. Resolved: §1 #1.5 binds all three to one rule (out-of-enum statuses excluded), and AC 6 tests it - the same single-source discipline TASK-IMP-104's t05 enforces on `ver_lt`.

### ISS-005 - "fix the regenerator instead" was the cheaper option and is not obviously wrong
It keeps the footprint minimal. Resolved: recorded in Alternatives with the operator's reasoning - a number maintained by a ritual rots between rituals, and "nobody ran it" is exactly how the 086 rows sat wrong. Repairing regen_backlog stays a real task; this one removes its urgency rather than its need, and Non-Goals says so.

### ISS-006 - a Totals line inside a fenced block would match `^Totals: `
A doc example at column 0 would be rewritten. Resolved: §3 names the risk rather than hiding it - BACKLOG.md is a generated index and a column-0 fence there is already a corpus defect. Named, bounded, revisitable. An accepted risk stated is not the same as a risk missed.

## §3 - Resolution

All 6 concerns addressed. The machine floor (task-lint) ran FIRST and was clean before any judgment family was applied, per TASK-IMP-084. **Score = 10/10.**

---

*End of TASK-IMP-116 audit.*
