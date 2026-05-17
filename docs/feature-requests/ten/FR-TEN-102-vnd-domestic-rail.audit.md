---
fr_id: FR-TEN-102
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

## §1 — Verdict summary

The spec lands VND domestic billing rail (VnPay + Momo + ZaloPay) symmetric to FR-TEN-003 Stripe rail. Final form: 1,210 lines, 27 §1 normative clauses (closed PSP enum, token-bind flow + KMS-wrapped tokens, monthly recurring charge with async webhook resolution via LISTEN/NOTIFY, period_close overage one-off, dunning state machine parallel to Stripe, CFO-gated refund + compensating hóa đơn, Decree 123 hóa đơn issuance with annual gap-free numbering + eHĐĐT signing, per-PSP webhook signature verification via 3 distinct HMAC schemes, NATS-bridged dispatch from INV layer, cross-rail block, founder skip, residency-isolated RLS, billing_contact_phone capture, 12 BRAIN audit kinds), 20 acceptance criteria, 10 verification tests, 22 failure-mode rows, 22 implementation notes. Net-new VND adapter trait + 3 PSP implementations + Decree 123 invoice store + 5 migrations.

The audit caught 7 issues across the FR-TEN-003 symmetry vs VND-specific divergence, hóa đơn legal-compliance details, async webhook resolution semantics, and PSP-credential rotation hygiene. All resolved.

## §2 — Findings (all resolved)

### ISS-001 — Async charge resolution lacked LISTEN/NOTIFY timeout policy

§1 #10 said "PSP responds 200 + processing; real outcome arrives via webhook within 5 min" but didn't specify what happens if the webhook never arrives. A 5-min Postgres LISTEN/NOTIFY wait that never wakes = handler hangs indefinitely. Resolved: §6.2 skeleton uses `tokio::time::timeout(Duration::from_secs(300), listener.recv())`; §10 failure-mode row covers PSP timeout → charge marked `processing_timeout` + reconciliation sweep; AC #5 verifies the async resolution path.

### ISS-002 — Gap-free hóa đơn semantics: "no duplicate" vs "no skipped" confusion

§1 #17 originally said "gap-free" without acknowledging Decree 123 §10's actual semantics — skipped numbers ARE permitted as long as the skip reason is logged. The first draft implied any rollback = compliance violation. Resolved: §1 #17 + §11.4 now explicit that "skipped numbers are auditable with explanation per Decree 123 §10"; `vnd_invoice_sequence.notes` JSONB stores skip reasons; AC #14 verifies "no duplicates" rather than "no gaps".

### ISS-003 — eHĐĐT signing failure mode under-specified

§1 #16 mandated signing via VN tax authority's eHĐĐT API but didn't define behaviour when the eHĐĐT API is down. Without clarity, an implementer might block the charge handler waiting for sign success. Resolved: §10 failure-mode row covers eHĐĐT down → invoice persisted as `status='issued'` with `signed_xml NULL`; retry job re-signs every 5min; max 24h before sev-1 (Decree 123 §13 grace period); §3.1 schema's status enum already includes the `'issued'` (unsigned) state.

### ISS-004 — Per-PSP idempotency key length differences not handled

§1 #18 said adapter maps canonical key → per-PSP shape but didn't address length constraints. VnPay caps at 100 chars, ZaloPay at 40. The canonical format `vnd.<tenant_uuid>.<operation>.<period_ts>` ~80 chars fits VnPay but exceeds ZaloPay. Resolved: §11.5 explicit length constraints per PSP + SHA-1-shortening strategy when canonical exceeds PSP limit; entropy preserved at ~2^60.

### ISS-005 — Cross-PSP charge attempt scenario undefined

A tenant binds a VnPay token at signup; later an operator (manually via SQL) inserts a Momo charge attempt against the same tenant. The active token is VnPay, but the charge code path might try Momo. Resolved: §11.14 clarifies single-source-of-truth — `vnd_payment_tokens` query returns the one active token; its `psp` column dictates which adapter to invoke. No cross-PSP confusion possible because there is no Momo token bound for that tenant.

### ISS-006 — RLS double-condition `tenant_id + residency='vn-1'` could regress

§3.1 VND tables enforce `tenant_id = current_setting('auth.tenant_id') AND current_setting('auth.residency') = 'vn-1'`. If a future FR forgets the residency check in session setup, the AND degrades to single-condition. Defense-in-depth review: FR-TEN-103 §1 #9 mandates setting `auth.residency` at every handler entry; the trip-wire on writes catches if missing. Marked resolved as documented + cross-FR enforced.

### ISS-007 — Billing_contact_phone capture point — FR-TEN-101 modification scope

§1 #23 says "FR-TEN-101's `SignupCompleteReq` body extended with optional `billing_contact_phone`". But FR-TEN-101 already shipped at 10/10 — modifying it post-hoc breaks the "no edits after accepted" convention. Resolved: `modified_files` build_envelope explicitly lists `services/ten/src/handlers/tenant_create.rs` — the field is captured at the TEN-001 provisioning layer, not in FR-TEN-101's signup flow. FR-TEN-101's signup orchestrator passes `billing_contact_email` to TEN-001; this FR extends the TEN-001 request shape to also accept phone for VND tenants. No retro-edit of TEN-101 needed.

## §3 — Resolution

All 7 mechanical concerns addressed. Hóa đơn legal semantics correct per Decree 123 §10 + §13; async charge resolution bounded by timeout; PSP-key length adapter handles all 3 PSPs; cross-PSP confusion impossible by data model; cross-FR modification scope respects "accepted = frozen" rule.

The 1,210-line length is justified by 3 PSPs × full lifecycle (token-bind + charge + refund + revoke + webhook + signature verify) + Decree 123 hóa đơn machinery + dunning + cross-rail guards. Density comparable to FR-TEN-003 (1,054) + FR-TEN-103 (1,205); VND rail has more PSP-specific surface than Stripe's single-rail model.

**Score = 10/10.**

---

*End of FR-TEN-102 audit.*
