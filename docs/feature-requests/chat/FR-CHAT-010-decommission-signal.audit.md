---
fr_id: FR-CHAT-010
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 17
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (no-line-cap expansion per AUTHORING.md §0; ISS-007..017 added)
---

## §1 — Verdict summary

FR-CHAT-010 expanded 2026-05-16 P.M. via strict-redo no-line-cap pass. Current scope: 21 §1 clauses (per-source counts, ratio, threshold, 14-day window, info-only, audit, nightly cron, CLI, transition notify, metrics, active-users breakdown, 3-consecutive Ready gate, persisted decommission_state, per-tenant threshold override, Regression detection, snooze CLI, recommended_action, last_legacy_message_at, source-weight extension, state-changed audit on every change, 30-day ratio_history in payload). 16 §2 rationale paragraphs. §3 contains: 6-variant Status enum, full DecommissionSignal struct with all fields, single-query check_tenant with snooze guard + active-user counts + last-legacy ts, derive_status as pure function with comprehensive table coverage, persist_state with upsert+first_ready_at preservation, fetch_ratio_history 30-day query, snooze.rs CLI integration, schema with state table + threshold/snooze/weights columns. 25 ACs. §5 contains 13 named test bodies covering all status transitions + per-tenant threshold + Regression + Snooze + recommended_action parameterised + last-legacy-ts + state-changed audit + 30-day trend + no-spam-on-stable + pure-function table. §6 deepens with 10 wiring subsections (nightly cron, tenant ordering, state-vs-derive choice, transition timing, snooze interaction, recommended-action evolution, history performance, CLI surface, failure routing, source-weight extension). §8 lists 5 example payloads (Ready + Regression + state-changed + snoozed + SEV-2 alert). §10 lists 42 failure rows. §11 lists 25 implementation notes covering source-flag canonical name, cron piggyback rationale, persisted-state vs derive-from-BRAIN choice, 3-consecutive-checks calibration, asymmetric 2-vs-3 Regression vs Ready threshold, Approaching status rationale, snooze-not-per-status, FILTER-with-partial-index Postgres pattern, DISTINCT ON history dedup, pure-function recommend(), 14-day window calibration, snooze --until date vs duration choice.

## §2 — Findings (all resolved)

### ISS-001 — Window length
1-week catches spikes. Resolved: §1 + DEC-510 14 days.

### ISS-002 — Threshold value
100% impossible. Resolved: §1 #3 + DEC-510 0.95.

### ISS-003 — Auto vs info-only
Auto-disable = mid-import corruption. Resolved: §1 #5 + DEC-511.

### ISS-004 — Sample-size protection
< 100 = noise. Resolved: §1 #2 + AC #3 InsufficientData.

### ISS-005 — Notification spam
Sev-3 every check = noise. Resolved: §1 #9 transition-only.

### ISS-006 — Source flag taxonomy
Without convention, ambiguous. Resolved: `imported_slack | imported_zalo` per props.

### ISS-007 — Active-user concentration unsurfaced (strict-redo pass)
Operators investigating "remaining 5%" need to know whether it's 5 heavy users or 50 occasional users. Ratio alone doesn't say. Resolved: §1 #11 + active-user counts in payload; AC #15 + test body verify.

### ISS-008 — Single-day spike could flip status (strict-redo pass)
Original threshold was instantaneous: one good day = Ready. False-positives on conference days / vacation lulls. Resolved: §1 #12 + 3-consecutive-checks gate + Approaching status as intermediate; AC #16 + test body + derive_status table verify.

### ISS-009 — No persisted decommission state (strict-redo pass)
Without persisted state, streak math required re-computing from BRAIN history on every check — coupling check latency to BRAIN. Resolved: §1 #13 + `cyberos_chat_decommission_state` table + persist_state upsert preserving first_ready_at; AC #17 verify.

### ISS-010 — Threshold was global only (strict-redo pass)
Compliance-heavy tenants want stricter; SMB tenants migrating from heavy Slack accept lower. Global 0.95 doesn't fit all. Resolved: §1 #14 + `decommission_threshold` column with default 0.95 + per-tenant override; AC #18 + test body verify.

### ISS-011 — Regression unsignaled (strict-redo pass)
A previously-Ready tenant dropping below threshold should alert (legacy traffic returning); original spec only signaled forward transitions. Resolved: §1 #15 + Regression status + 2-check gate + SEV-2 alert; AC #19 + test body verify.

### ISS-012 — No operator snooze (strict-redo pass)
Tenants in known "not decommissioning" state (M&A, vendor lock) generate noise as they cross thresholds. Resolved: §1 #16 + snooze CLI + Snoozed status + `chat.decommission_snoozed` audit; AC #20 + test body verify.

### ISS-013 — Recommendation reduced to numeric (strict-redo pass)
Operators reading the JSON shouldn't memorise a decision tree of (status, ratio) → action. Resolved: §1 #17 + `recommended_action` field + pure recommend() function with parameterised test coverage; AC #21.

### ISS-014 — last_legacy_message_at missing (strict-redo pass)
Operators communicate to stakeholders with concrete dates ("Slack hasn't been used in 9 days") rather than ratios. Resolved: §1 #18 + MAX-FILTER query in single round-trip; AC #22 + test body verify.

### ISS-015 — No granular state-change audit (strict-redo pass)
Original audited only Ready transitions; intermediate state changes (NotReady → Approaching → Ready) were invisible. Operators tracking adoption need every state change. Resolved: §1 #20 + `chat.decommission_state_changed` audit on every transition; AC #23 + test body verify.

### ISS-016 — No trend visualisation data (strict-redo pass)
Operators asking "is adoption accelerating or stalling?" need the curve, not just the point. Resolved: §1 #21 + 30-day `ratio_history` in payload + DISTINCT ON daily aggregate; AC #24 + test body verify.

### ISS-017 — Source-weight extensibility unspecified (strict-redo pass)
VN-market tenants find Zalo more business-critical; future operators may want to weight per-source. Original spec had no extension hook. Resolved: §1 #19 + `decommission_source_weights` JSONB column reserved for slice 4+; current impl defaults to 1.0 all sources; documented in §6.10.

## §3 — Resolution

All 17 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine surface (6-state machine × per-tenant threshold × per-source weights × snooze × consecutive-check gates × regression detection × concrete-date signal × 30-day trend × operator-facing recommendation), not by line targets.

---

*End of FR-CHAT-010 audit.*
