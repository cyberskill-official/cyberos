---
fr_id: FR-INV-004
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 8
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per feature-request-audit skill §0)
---

## §1 — Verdict summary

FR-INV-004 ships Wise webhook handler for multi-currency receipts (USD/EUR/GBP/SGD/JPY) with RSA-SHA256 signature verification, per-profile public key cache + rotation, idempotency, dead-letter queue, and currency-mismatch hold for CFO review. Scope: 25 §1 normative clauses covering closed 3-value `wise_event_type` enum (transfers_state_change, balances_credit, balances_update — exhaustive at Wise 2025-Q1 API), closed 5-value `wise_receipt_state` (received, matched, currency_mismatch, dead_lettered, manually_resolved), RSA-SHA256 signature verification via ring crate (not HMAC — Wise asymmetric) + per-profile public key fetched once + cached 24h + force-refresh on verification failure + accept new key OR persist failure as 401 + sev-2 audit, 5-day staleness rejection (matches Wise retry window), URL-vs-body profile_id cross-check to defeat URL-forgery confusion, idempotency via UNIQUE (profile_id, event_id) + INSERT ON CONFLICT DO NOTHING + duplicate returns 200 with no second audit, append-only via SQL grant (REVOKE UPDATE/DELETE FROM cyberos_app; privileged inv_wise_writer role for state mutations), fast 200 response (≤5s Wise SLA) via signature verify + persist + WAL push then return; heavy processing offloaded to background processor consuming WAL channel with pg_advisory_xact_lock(event_id) per event, currency-mismatch (e.g., USD receipt to VND invoice) NEVER auto-converted at receipt time (DEC-846 + FX is FR-INV-002 SBV authoritative) — held in shared `unmatched_receipts` table for CFO review + sev-2 audit + resolve handler requires CFO role + reason ≥10 chars, dead-letter at 3 processing failures + sev-1 audit + ops alarm + CFO restore handler resets retry_count and state, per-profile rate limit 100/s + 429 Retry-After:1, TLS 1.3 enforced at edge layer, 7-day profile deprecation window then 410 GONE, RLS isolation on wise_webhook_events + unmatched_receipts, 8 closed memory audit kinds (wise_received sev-3, wise_matched sev-3, wise_signature_invalid sev-2, wise_stale_event sev-2, wise_currency_mismatch sev-2, wise_dead_lettered sev-1, wise_profile_unknown sev-1, wise_key_rotated sev-2), all reason/body text scrubbed via FR-MEMORY-111 before chain emission, body retention 2 years then NULL-out cleanup while preserving metadata + chain row stays forever, CFO raw-body view emits sev-3 introspection audit. 22 rationale paragraphs. §3 contains: 2 migrations (wise_webhook_events with closed enums + UNIQUE idempotency + grants + RLS + per-tenant wise_profile_id + 7-day deprecation column; shared unmatched_receipts with source enum CHECK + currency CHAR(3) + amount_minor positive + resolution_notes ≥10 chars + RLS), signature verifier using ring RSA_PKCS1_2048_8192_SHA256, handler with key refresh + staleness check + URL/body cross-check + ON CONFLICT DO NOTHING + WAL push + SLA monitoring. 30 ACs. 32 failure-mode rows. 22 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Hand-rolled RSA verification or HMAC mistake
First-pass had HMAC like Stripe/Napas247. Resolved: §1 #1 + DEC-840 + RSA-SHA256 via ring crate + disallowed_tools forbids hand-rolling; AC #3.

### ISS-002 — Public key fetched per webhook (latency + Wise rate-limit)
Resolved: §1 #2 + DEC-841 + 24h cache + force-refresh on signature failure path; AC #10.

### ISS-003 — Public key rotation breaks signature verification mid-flight
Resolved: §1 #3 + DEC-848 + retry once with re-fetched key + accept new key OR persist failure; AC #11.

### ISS-004 — Replay window unbounded
Resolved: §1 #6 + DEC-844 + 5-day staleness rejection (matches Wise retry window) + sev-2 audit; AC #6.

### ISS-005 — Auto-convert FX at receipt time (revenue recognition controversy)
Resolved: §1 #10 + DEC-846 + DEC-852 + currency-mismatch hold in unmatched_receipts + CFO resolves with reason; AC #13 + #14.

### ISS-006 — Synchronous processing risks 5-second SLA
Resolved: §1 #8 + DEC-850 + fast 200 + WAL push + background processor with advisory lock; AC #4 + #22 + #29.

### ISS-007 — URL vs body profile_id confusion (forged URLs)
Resolved: §1 #22 + cross-check + 400 + sev-2 audit; AC #8.

### ISS-008 — Dead-letter accumulation unbounded
Resolved: §1 #12 + DEC-851 + 3-fail threshold + ops alarm + CFO restore handler; AC #15 + #16.

## §3 — Resolution

All 8 mechanical concerns addressed. **Score = 10/10.**

Per feature-request-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (RSA-SHA256 asymmetric verification via ring × per-profile public key 24h cache + rotation force-refresh × 5-day staleness reject × URL-vs-body profile_id cross-check × idempotency UNIQUE (profile_id, event_id) ON CONFLICT × append-only SQL grant × fast 200 within 5s Wise SLA × background processor with pg_advisory_xact_lock × currency-mismatch hold for CFO (no auto-convert) × dead-letter at 3 failures + restore handler × per-profile rate limit 100/s × 7-day profile deprecation + 410 GONE × TLS 1.3 edge enforcement × RLS isolation × 8 closed memory audit kinds × FR-MEMORY-111 PII scrubbing × body retention 2y NULL-out + chain forever × CFO raw-body view with sev-3 introspection audit × shared unmatched_receipts table across sources × wise_profile_id 12-digit format validation at provisioning), not by line targets.

---

*End of FR-INV-004 audit.*
