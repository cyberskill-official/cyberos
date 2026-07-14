---
task_id: TASK-HR-001
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 11
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per task-audit skill §0)
---

## §1 — Verdict summary

TASK-HR-001 ships the HR Member schema — the canonical "is this person currently employed?" record. Scope: 26 §1 normative clauses covering closed 6-state FSM, closed 7-level enum, RLS isolation, comp-exclusion CI gate, leave_balance read-only materialised view, immutable start_date post-active, status history append-only, sabbatical accrual SQL function (Decree 145 Art. 113), CCCD-encrypted column with sev-1 access audit, 5 REST handlers, idempotency, OTel emission, two SQL views (member_active_view + sabbatical_eligible_view), AUTH-bound auto-create trigger stub. 16 rationale paragraphs. §3 contains 7 code blocks: 3 migrations + status FSM + Member struct + comp-exclusion guard + REST handlers. 27 ACs. 32 failure-mode rows. 22 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Compensation column drift risk
First-pass had no CI gate against `base_salary` column drift. Resolved: §1 #11 + DEC-203 + DB CHECK + CI `comp_exclusion_test` parsing SQL migrations; AC #17.

### ISS-002 — Leave balance double-writer
First-pass let both this FR and TASK-HR-004 write leave_balance_days. Resolved: §1 #10 + DEC-204 + trigger blocking UPDATE; AC #6. TASK-HR-004 ships the single recompute path.

### ISS-003 — Start_date drift after onboarding
First-pass allowed retroactive start_date amendment, breaking sabbatical math. Resolved: §1 #8 + DEC-207 + BEFORE UPDATE trigger raising `cannot_modify_locked_start_date`; AC #7.

### ISS-004 — Status FSM open-ended
First-pass had string-typed status field; ill-defined transitions. Resolved: §1 #3 + DEC-200 + 6-value enum + §1 #5 closed 11-transition matrix; AC #8 + AC #9.

### ISS-005 — CCCD access untracked
First-pass left CCCD reads silent. Resolved: §1 #13 + DEC-208 sev-1 audit row + OTel counter; AC #15.

### ISS-006 — Append-only history bypassable
First-pass relied on handler discipline. Resolved: §1 #6 + task-audit skill rule 12 + `REVOKE UPDATE, DELETE FROM cyberos_app`; AC #10.

### ISS-007 — Sabbatical computation drift
First-pass had no formula calibration test. Resolved: §1 #26 + IMMUTABLE SQL function + `sabbatical_test::accrual_curve` covering year 0/1/4/5/6/10/30/35/40 with cap at 30; AC #11–13.

### ISS-008 — Level enum unbounded
First-pass had `level INT`. Resolved: §1 #4 + DEC-206 + 7-value closed enum + `level_enum_closed_test` cross-validating Rust + SQL; AC #2.

### ISS-009 — CCCD column shape ambiguous
First-pass had `cccd_id TEXT`. Resolved: §1 #12 + DEC-202 + `cccd_encrypted BYTEA` only; raw form forbidden.

### ISS-010 — `member_active_view` predicate drift
First-pass had no canonical "currently employed" view. Resolved: §1 #20 + view `member_active_view` filtering `status IN ('probation','active','on_leave')`; AC #24.

### ISS-011 — No FK constraint to auth.subjects
First-pass let HR Members exist without an Auth subject. Resolved: §1 #15 + DEC-209 + FK REFERENCES auth.subjects(id) ON DELETE RESTRICT; AC #26.

## §3 — Resolution

All 11 mechanical concerns addressed in the first revision pass. **Score = 10/10.**

Per task-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (members table × closed status FSM × closed level enum × comp-exclusion guard × append-only history × RLS isolation × sabbatical accrual × CCCD encrypted + audit × REST + idempotency × OTel × 2 SQL views × AUTH-bound auto-create), not by line targets.

---

*End of TASK-HR-001 audit.*
