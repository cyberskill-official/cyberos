---
fr_id: FR-CRM-001
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 9
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per AUTHORING.md §0)
---

## §1 — Verdict summary

FR-CRM-001 ships the Account/Contact/Deal Postgres schema with custom pipelines + closed FSM. Scope: 26 §1 normative clauses covering 3 closed Postgres enums (deal_status 4, pipeline_shape 4, account_type stub 1), many-to-many contact_account_membership, append-only deal_status_history + deal_stage_history at SQL grant, deal status FSM trigger with stage-gate (open→won requires is_won stage), 4-default-pipeline seed function for FR-TEN-001 hook, BIGINT-minor money storage, RLS isolation across 8 tables, 8 memory audit kinds with PII scrubbing of email/phone/full_name, contact-membership-≥1 invariant, expected_close future-date enforcement, REST surface (15 handlers), OTel emission, mutual-exclusion stage classification (is_open xor is_won xor is_lost), default-pipeline-uniqueness via partial index, probability override semantics. 19 rationale paragraphs. §3 contains: 6 migrations (accounts + contacts + pipelines/stages + deals with FSM trigger + history + seed function), Rust types, FSM validator. 27 ACs. 30 failure-mode rows. 23 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Contact orphaned without account
First-pass let contacts exist with zero memberships. Resolved: §1 #16 + DEC-348 + handler check + AC #6 + #7.

### ISS-002 — Deal status drift across pipelines
First-pass had per-pipeline status enum (every tenant's stages duplicating won/lost). Resolved: §1 #8 + DEC-344 + universal closed `deal_status` (4 values) + per-pipeline stage with `is_won/is_lost` classification; AC #1 + #11.

### ISS-003 — Mutual exclusion of stage classification
First-pass allowed `is_open=true + is_won=true` simultaneously, breaking FSM. Resolved: §1 #6 + DB CHECK; AC #21.

### ISS-004 — Money as FLOAT
First-pass used `DECIMAL(15,2)` (better) but allowed bound-field flexibility. Resolved: §1 #13 + AUTHORING.md rule 11 + BIGINT minor + CHAR(3) currency; AC #18.

### ISS-005 — Won/lost without reason
First-pass allowed silent close. Resolved: §1 #21 + DB CHECK reason length + trigger required; AC #13 + #14.

### ISS-006 — Status FSM at handler only
First-pass had no DB-side check; direct SQL bypassed. Resolved: §1 #9 + `enforce_deal_status_fsm` trigger + AC #16.

### ISS-007 — Default-pipeline seed function deferred
First-pass left "operator creates pipelines manually". Resolved: §1 #14 + seed_default_pipelines SQL function + FR-TEN-001 hook integration; AC #23.

### ISS-008 — PII in memory audit (raw email, phone, full_name)
First-pass logged contact email in audit chain. Resolved: §1 #18 + FR-MEMORY-111 scrubbing; hashed forms only in chain.

### ISS-009 — Append-only history not enforced
First-pass relied on handler discipline. Resolved: §1 #10 + #11 + DEC-346 + `REVOKE UPDATE, DELETE FROM cyberos_app` on both history tables; AC #19 + #20.

## §3 — Resolution

All 9 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (3 closed enums × many-to-many contacts × append-only history × FSM with stage-gate × default-pipeline seed × BIGINT-minor money × RLS isolation × 8 memory audit kinds × probability override × expected-close future-date × mutual-exclusion stage classification), not by line targets.

---

*End of FR-CRM-001 audit.*
