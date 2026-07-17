---
task_id: TASK-IMP-108
audited: 2026-07-17
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 8
template: task@1
audit_rubric_version: audit_rubric@2.0
audited_file_sha256: 91b11f27bb0711f1
audited_body_sha256_prefix: 0f839af69ebfe7fd
machine_floor: task-lint clean (exit 0) - run FIRST per TASK-IMP-084
---

## §1 - Verdict summary

Spec is 104 lines, 7 §1 clauses, 6 ACs, 7 edge cases. Merges three handoff findings the handoff itself asks to be treated as one batch. Every claim verified on main (336 draft rows; 18 routed_back_count references, 0 reading it as a limit). Passes after 8 findings.

## §2 - Findings (all resolved)

### ISS-001 - Scope could be read as re-opening the rename decision
The operator decided 2026-07-17 to keep `implement`; a task touching status semantics must not relitigate it. Resolved: Non-Goals state the 12-value enum is unchanged and name the decision; Alternatives records why removing `draft` was rejected.

### ISS-002 - Backfilling draft_reason across 336 drafts would fabricate reasons
Inventing a reason for a task this run did not author is the `# UNREVIEWED` mistake with better manners. Resolved: explicit Non-Goal; §3 requires absent to render as unknown.

### ISS-003 - entered_via may be redundant with routed_back_count
The counter already separates fresh from rework; only `spec_rejected` needs a new value. Resolved: Alternatives records the fallback explicitly so review can collapse the field - the cheaper option stays live rather than being argued away.

### ISS-004 - A ceiling of 3 is a judgment, not a derivation
Presenting it as a fact would be false precision. Resolved: the Proposed Solution says so plainly; AC 5 pins that 2 still re-enters, making the number testable rather than asserted.

### ISS-005 - spec_rejected routing to ready_to_implement would loop forever
Handing an unchanged wrong spec to an implementer rebuilds the same wrong thing. Resolved: §1 #1.5 routes to `draft` for re-authoring and re-audit; AC 3 asserts the landing status.

### ISS-006 - Ceiling could be resolved by a swarm sub-agent
The verdict is the operator's, and §11a says shared gates belong to the parent. Resolved: §3 edge case makes the halt the parent's.

### ISS-007 - Shared-file conflicts with five sibling tasks were unrecorded
Touches STATUS-REFERENCE, ship-tasks, backlog-mutate, render-status-hub, and test_workflow_evolution - all contended. Resolved: Dependencies carries a §11a serialisation note; these MUST NOT be swarm members of one round.

### ISS-008 - Staleness report could be read as licence to auto-close
A report that ages drafts is one step from a job that closes them. Resolved: §1 #1.7 forbids status change; Alternatives rejects auto-close as an operator's decision.

## §3 - Resolution

All 8 concerns addressed. The machine floor (task-lint) ran FIRST and was clean before any judgment family was applied, per TASK-IMP-084. **Score = 10/10.**

---

*End of TASK-IMP-108 audit.*
