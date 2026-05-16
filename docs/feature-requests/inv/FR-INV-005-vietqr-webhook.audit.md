---
fr_id: FR-INV-005
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

FR-INV-005 ships the VietQR/Napas247 webhook handler with HMAC-SHA256 + idempotency + memo parser + append-only ledger. Scope: 26 §1 normative clauses covering 2 closed Postgres enums (receipt_source 3, bank_code 15), per-tenant URL routing with slug, HMAC-SHA256 verification with constant-time compare + 60-second secret-rotation overlap, 5-minute replay window, idempotency on Napas TXN reference, append-only payment_receipts via SQL grant + privileged inv_cash_applier role with column-level UPDATE on invoice_id only, memo parser regex `^(HD|INV)(\d{6,12})\b` with fast-path matching, payload SHA-256 storage (not full body) for forensic replay, server-side received_at (vs Napas-supplied webhook_ts), 5 BRAIN audit row kinds (received/rejected/matched/unmatched/duplicate) with PII scrubbing of sender_account+sender_name+memo, BIGINT đồng money storage, VND-only enforcement, sev-2 alarm at > 10 rejections/h, probe-detection audit on tenant_unknown, 5-second handler ack budget, secret rotation handler with CFO role gate. 22 rationale paragraphs. §3 contains: 2 migrations (payment_receipts with closed enums + webhook_secrets with rotation history), HMAC verifier with constant-time compare, webhook handler with full 8-step flow, memo parser with unit tests. 27 ACs. 30 failure-mode rows. 20 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — HMAC bypass via env override
First-pass had a debug flag to skip HMAC. Resolved: §1 #8 + DEC-380 + disallowed_tools — no escape hatch; HMAC always enforced.

### ISS-002 — Idempotency missing
First-pass would double-credit on Napas247 retry. Resolved: §1 #10 + DEC-381 + UNIQUE constraint on (tenant_id, transaction_reference); AC #8.

### ISS-003 — Replay attack window
First-pass had no timestamp validation. Resolved: §1 #9 + DEC-389 + 5-minute window via `ts` field in body; AC #7.

### ISS-004 — Append-only not enforced at SQL grant
First-pass relied on handler discipline. Resolved: §1 #6 + DEC-382 + REVOKE UPDATE, DELETE; privileged inv_cash_applier role with column-level grant on invoice_id only; AC #14 + #15 + #16 + #17.

### ISS-005 — Cross-currency in single handler
First-pass tried to multiplex VND + USD via switch. Resolved: §1 #21 + DEC-385 + VND-only enforcement; Stripe/Wise are separate handlers.

### ISS-006 — Tenant URL inspection from body
First-pass parsed body to find tenant_id. Resolved: §1 #7 + DEC-387 + per-tenant URL slug — tenant resolved BEFORE body touch.

### ISS-007 — Secret rotation atomic-cutover (lockout risk)
First-pass had no overlap window. Resolved: §1 #16 + #20 + 60-second dual-secret window; old + new both valid during transition; AC #19 + #20.

### ISS-008 — Probe attacks invisible
First-pass returned 404 silently on bad slug. Resolved: §1 #25 + DEC-388 + emit inv.webhook_rejected with `reason=tenant_unknown` for probe detection; AC #9.

### ISS-009 — Memo parser too permissive (false positives)
First-pass used loose regex. Resolved: §1 #11 + tight `^(HD|INV)(\d{6,12})\b` regex + 6-12 digit bound; word boundary prevents false matches; unit tests AC #10 + #11.

## §3 — Resolution

All 9 mechanical concerns addressed. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (2 closed enums × HMAC-SHA256 + constant-time compare × idempotency × replay window × append-only + privileged column-level grant × memo parser × payload-hash storage × server-side timestamp × 5 BRAIN audit kinds × VND-only × per-tenant URL × 60s secret-rotation overlap × probe detection × CFO-gated rotation), not by line targets.

---

*End of FR-INV-005 audit.*
