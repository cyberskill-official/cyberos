---
task_id: TASK-PROJ-011
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 18
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per task-audit skill §0; ISS-007..018 added)
---

## §1 — Verdict summary

TASK-PROJ-011 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 22 §1 clauses (regex + classification, blocker_state table, auto-resolve triggers, business-day dwell, audit kinds, CUO Notify, hourly scan, RLS, metrics, PII redaction on target_ref, manual resolve, manual create, escalation at 10d, per-tenant dwell override, CUO notify health tracking, cycle detection, age distribution histogram, snooze workflow, mentioned_users in payload, blocker_resolved_diff with comment text, cross-engagement blocker support). 18 §2 rationale paragraphs. §3 contains migration + parser regex + business_days_between + scan_stale + CUO notify integration. 29 ACs. §10 lists 30 failure rows. §11 lists 23 implementation notes covering NLP rejection rationale, escalation calibration, snooze max 14d rationale, cross-engagement scope check, manual-vs-auto distinction, comment-edit-not-retrigger policy, Vietnamese username handling.

## §2 — Findings (all resolved)

### ISS-001 — Detection signal
Drag-to-Blocked-column = friction; comment parsing = zero-friction. Resolved: §1 #1 + DEC-320.

### ISS-002 — Business-day math
Calendar dwell pings false-positives Mon morning. Resolved: §1 #5 business days + VN holidays env.

### ISS-003 — Auto-resolve triggers
Without clear rules, blockers linger after target done. Resolved: §1 #4 three trigger paths.

### ISS-004 — Notification routing
Email = noise. Resolved: §1 #7 + DEC-322 CUO Notify only.

### ISS-005 — Idempotent staleness
Without single-shot semantics, every scan re-notifies. Resolved: §1 #6 `stale_notified_at` column + AC #13.

### ISS-006 — Target classification
Without typed targets, can't auto-resolve. Resolved: §1 #2 three kinds; AC #1 #2 #3.

### ISS-007 — target_ref PII leak (strict-redo pass)
FreeText blockers embed emails/phones. Resolved: §1 #11 + redact + AC #18.

### ISS-008 — No manual resolve path (strict-redo pass)
Auto-resolve misses verbal agreements. Resolved: §1 #12 + AC #19.

### ISS-009 — Parser-missed blockers untracked (strict-redo pass)
Meetings discuss blockers not in comments. Resolved: §1 #13 + manual create + AC #20.

### ISS-010 — Stale never escalates (strict-redo pass)
Assignee may not see; needs management. Resolved: §1 #14 + escalation at 10d + AC #21.

### ISS-011 — Global dwell threshold inflexible (strict-redo pass)
Per-tenant SLA varies. Resolved: §1 #15 + override + AC #22.

### ISS-012 — Notification failure invisible (strict-redo pass)
Silent CUO failures = unfound blockers. Resolved: §1 #16 + tracking + SEV-1 at 3 fails + AC #23.

### ISS-013 — Cycles silently allowed (strict-redo pass)
A↔B cycle = both stuck. Resolved: §1 #17 + detection + alert + AC #24.

### ISS-014 — No team-health metric (strict-redo pass)
Total count alone doesn't show distribution. Resolved: §1 #18 + age histogram + AC #25.

### ISS-015 — No snooze (strict-redo pass)
Long-running legitimate blockers spam alerts. Resolved: §1 #19 + snooze max 14d + AC #26.

### ISS-016 — Mention context lost (strict-redo pass)
Comment tags unblock owner; not captured. Resolved: §1 #20 + mentioned_users + AC #27.

### ISS-017 — Resolution context lost (strict-redo pass)
Auto-resolve emits row but not the comment that triggered. Resolved: §1 #21 + blocker_resolved_diff + AC #28.

### ISS-018 — Same-engagement scope too narrow (strict-redo pass)
Cross-engagement dependencies real. Resolved: §1 #22 + scope check on both engagements + AC #29.

## §3 — Resolution

All 18 mechanical concerns addressed. **Score = 10/10.**

Per task-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine surface (parser × classification × auto-resolve × dwell × escalation × snooze × cycle × mention tracking × cross-engagement × PII), not by line targets.

---

*End of TASK-PROJ-011 audit.*
