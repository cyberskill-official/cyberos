---
fr_id: FR-INV-003
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 9
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per feature-request-audit skill §0)
---

## §1 — Verdict summary

FR-INV-003 ships the Stripe webhook handler — Stripe-Signature v1 + closed 8-event allowlist + event.id idempotency + multi-currency + append-only ledger. Scope: 26 §1 normative clauses covering Stripe v1 signing scheme (HMAC-SHA256 over `{t}.{body}` with constant-time compare), closed StripeEventKind enum (8 values mapping to Stripe event_type strings), idempotency on event.id with 200 OK ack on duplicate, 5-minute replay window per Stripe spec, per-tenant URL routing with KMS-encrypted secret + 60s rotation overlap, multi-currency support (USD/EUR/SGD/GBP) with BIGINT minor units, append-only stripe_event_log via SQL grant, 6 memory audit row kinds with PII-scrubbed customer email + name, metadata.invoice_id linking to FR-INV-001 invoices, charge.refunded as negative payment_receipts row, unknown event_type → 200 OK + log (Stripe must not retry), livemode validation (prod vs test), subscription events logged for FR-TEN-003 consumption, sev-2 alarm at > 10 rejections/h, PCI SAQ-A scope (Stripe holds card data). 19 rationale paragraphs. §3 contains: migration 0012 (stripe_event_log + bank_code ALTER TYPE for STRIPE), migration 0013 (stripe_webhook_secrets with rotation), Stripe v1 signature verifier with v0-deprecation + replay window, StripeEventKind closed enum with FromStr, webhook handler with full 8-step flow, event dispatcher routing by enum. 27 ACs. 31 failure-mode rows. 21 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Stripe v0 signing scheme accepted (deprecated)
First-pass parsed both v0 and v1. Resolved: §1 #7 + DEC-468 + parser ignores v0; only v1 hex extracted; AC #5.

### ISS-002 — Idempotency missing
First-pass would double-credit on Stripe retry. Resolved: §1 #10 + DEC-462 + UNIQUE(tenant_id, stripe_event_id) + handler lookup; AC #8.

### ISS-003 — Unknown event_type returned 500 → infinite retry
First-pass 500'd on novel events. Resolved: §1 #9 + DEC-461 + 200 OK ack + log row with `outcome='unknown_event_type'`; AC #9.

### ISS-004 — Replay window missing
First-pass had no `t` timestamp check. Resolved: §1 #8 + DEC-464 + 5-min window matching Stripe spec; AC #7.

### ISS-005 — Currency unbounded
First-pass accepted any 3-letter currency. Resolved: §1 #12 + DEC-466 + closed allowlist USD/EUR/SGD/GBP; AC #11.

### ISS-006 — Livemode unvalidated (test events on prod endpoint)
First-pass didn't check livemode. Resolved: §1 #23 + DEC-461 + env-driven `expect_livemode` config; AC #12.

### ISS-007 — Append-only not enforced at SQL grant
First-pass relied on handler discipline. Resolved: §1 #5 + DEC-463 + REVOKE UPDATE, DELETE; AC #16.

### ISS-008 — Metadata.invoice_id ignored
First-pass left receipts unlinked. Resolved: §1 #25 + DEC-472 + extract metadata.invoice_id + populate receipt's invoice_id column; AC #14.

### ISS-009 — Customer PII raw in memory
First-pass logged full email + name in audit chain. Resolved: §1 #13 + FR-MEMORY-111 scrubbing; hashed forms in chain.

## §3 — Resolution

All 9 mechanical concerns addressed. **Score = 10/10.**

Per feature-request-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (Stripe v1 signature × closed 8-event allowlist × event.id idempotency × replay window × multi-currency BIGINT × append-only via SQL grant × per-tenant URL + secret rotation × 6 memory audit kinds × metadata.invoice_id linking × refund-as-negative-amount × livemode validation × PCI SAQ-A scope × sev-2 alarm), not by line targets.

---

*End of FR-INV-003 audit.*
