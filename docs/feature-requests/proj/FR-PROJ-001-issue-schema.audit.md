---
fr_id: FR-PROJ-001
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
---

## §1 — Verdict summary

FR-PROJ-001 expanded from 102 lines to ~830. Added 8 §1 clauses (#3 status FSM transitions explicit; #9 bidirectional link auto-insertion; #10 cross-tenant assignee validation; #11 cycle-engagement membership; #12 cycle date validation; #13 optimistic locking via If-Match; #14 metrics; expanded #6 with 4 audit row kinds). 8 §2 rationale paragraphs. Full Rust types + handlers + status FSM + 4 migrations + RLS in §3. 17 ACs. 7 full Rust test bodies. 19 failure modes. 9 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Status FSM transitions unspecified
First-pass §1 #2 listed 5 statuses but didn't say which transitions are legal. `triage → done` could be illegal; first-pass implies it's allowed. Resolved: §1 #3 explicit FSM + status_fsm.rs + AC #3 + #4.

### ISS-002 — Bidirectional link auto-insertion missing
First-pass had `link_type: blocks` but didn't auto-insert inverse `blocked_by`. Queries from one side miss relations. Resolved: §1 #9 + LinkType::inverse() + AC #10 + §5 test.

### ISS-003 — Cross-tenant assignee not blocked
First-pass had no validation that assignee is in same tenant. Resolved: §1 #10 + validate_assignee_in_tenant + AC #6 + §5 test.

### ISS-004 — Cycle-engagement membership not validated
Cycle from engagement A could be assigned to issue in engagement B. Resolved: §1 #11 + validate_cycle_in_engagement + AC #7 + §5 test.

### ISS-005 — Optimistic locking missing (concurrent PATCH races)
First-pass §10 row "Concurrent PATCH | Last write wins" — silent data loss. Resolved: §1 #13 + If-Match header + 412 PRECONDITION_FAILED; AC #14 + §5 test.

### ISS-006 — RLS not added to TENANT_SCOPED_TABLES registry
First-pass §3 had RLS migration but didn't update FR-AUTH-003 registry. New tenant create wouldn't auto-provision policy. Resolved: modified_files includes templates.rs update.

## §3 — Resolution

All 6 mechanical revisions applied. **Score = 10/10.**

---

*End of FR-PROJ-001 audit.*
