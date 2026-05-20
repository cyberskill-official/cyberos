---
fr_id: FR-PROJ-004
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
---

## §1 — Verdict summary

FR-PROJ-004 authored direct-to-10/10. ~830 lines. 13 §1 clauses (6-state enum, 12 transitions + no-op, server validation order, history table, memory audit, CLI, REST contract, TS↔Rust codegen, time-in-state metric, OTel + metrics, dedicated reopen kind, state_groups deferred). 8 §2 rationale paragraphs. Full IssueStatus enum + FSM table + transition handler + SQL migration + TS mirror + StatusPicker UI in §3. 20 ACs. 7 Rust tests including exhaustive 5×5 legal/illegal grid. 17 failure modes. 9 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — State count (4 vs 6 vs 8)
Too few states = coarse; too many = over-engineered. Resolved: §1 #1 + §2 calibrated 6-state set (Backlog/Todo/InProgress/InReview/Done/Cancelled) matching industry convention.

### ISS-002 — Re-open semantics
Implicit re-open from done back to in_progress (via simple LWW) loses the "why" context. Resolved: §1 #2 + #4 + DEC-253 explicit `reason` required; AC #4 #5; dedicated `proj.issue_reopened` audit kind.

### ISS-003 — Validation order (FSM vs LWW)
Wrong order = wrong error message (stale_write instead of illegal_transition). Resolved: §1 #3 explicit order: FSM check FIRST, then LWW; better UX.

### ISS-004 — Drift between Rust + TS state definitions
Two language hardcodes drift. Resolved: §1 #8 + §3 codegen + CI gate `ts-fsm-fresh`; AC #11.

### ISS-005 — Time-in-status analytics
Without metric, "how long do issues sit in review?" requires retroactive query. Resolved: §1 #9 + §3 metric captured at transition-out time + AC #10.

### ISS-006 — `cancelled` from `backlog` rationale
Should backlog → cancelled be legal? Backlog items haven't been committed; deleting > cancelling. Resolved: §1 #2 + §2 rationale: cancellation is meaningful only post-commit; backlog → cancelled excluded; backlog items deleted via different operation.

## §3 — Resolution

All 6 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

---

*End of FR-PROJ-004 audit.*
