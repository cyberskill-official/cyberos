---
fr_id: FR-INV-006
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

FR-INV-006 ships the 4-step closed cash-application cascade with partial allocation, atomic over-allocation defense, append-only reversal, and CFO-only manual matching. Scope: 27 §1 normative clauses covering closed 5-value `allocation_source` enum (memo_parser, amount_date, fuzzy, manual, reversal), append-only `payment_allocations` table via SQL grant (REVOKE UPDATE/DELETE from `cyberos_app` + privileged `inv_cash_applier` role), 4-step matching cascade (Step 1 memo regex `INV[-_]?(\d{4,12})` extraction + invoice_number lookup; Step 2 SUM(unallocated) = receipt amount + ±3 day window; Step 3 fuzzy ≤ N% diff against single oldest unpaid invoice with per-tenant threshold; Step 4 CFO-only manual), atomic over-allocation block via DB trigger (P0100 + P0101 SQLSTATE), partial M:N receipt-to-invoice mapping (`invoice_outstanding_view` centralises math), reversal pattern with negative-amount rows + `reverses_allocation_id` self-FK + reciprocity trigger, 5-min scheduled job with `pg_advisory_xact_lock` per receipt (no double-processing), per-tenant `cash_app_fuzzy_threshold_pct` override [1, 20]% default 5%, CFO-only manual matching with sev-2 audit + reason required, 9 memory audit kinds with PII scrubbing (cash_app_match_attempted, matched_step1-3, matched_manual, no_match, over_allocation_blocked, partial_allocated, allocation_reversed), auto-mark-paid trigger when SUM(allocations) ≥ invoice.amount_minor, currency-mismatch hard skip at every step, dry-run handler for CFO preview, idempotent re-processing via receipt-FK + advisory lock. 22 rationale paragraphs. §3 contains: 3 migrations (payment_allocations + reverses_allocation_id self-FK + grants, over-allocation trigger function with FOR UPDATE lock, auto-mark-paid trigger, invoice_outstanding_view), 4-step matcher in pure SQL (memo parser regex + amount_date join + fuzzy ranking + manual handler), reversal handler with reciprocity check, dry-run query. 30 ACs. 32 failure-mode rows. 22 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Over-allocation race window (concurrent receipts double-allocate same invoice)
First-pass had no locking — two concurrent jobs could both see `outstanding = $1000` and allocate $1000 each → over-allocation. Resolved: §1 #8 + DEC-620 + DB trigger holds `FOR UPDATE` on invoice row + SQLSTATE P0100 over_allocation_blocked; AC #6 + #7.

### ISS-002 — Manual override unaudited (CFO writes match silently)
First-pass let CFO manually allocate without trace. Resolved: §1 #16 + DEC-628 + cfo role gate + reason required + sev-2 audit + `matched_manual` memory row; AC #12 + #13.

### ISS-003 — Reversal asymmetric (negative row without reciprocity check)
First-pass allowed orphan negative-amount rows. Resolved: §1 #18 + DEC-630 + reverses_allocation_id self-FK NOT NULL on reversals + trigger checks sign + reciprocity SUM = 0; AC #18 + #19.

### ISS-004 — Currency-mismatch silent allocation
First-pass joined receipt-to-invoice on amount alone. Resolved: §1 #20 + DEC-632 + WHERE receipt.currency = invoice.currency at every step + skip + sev-3 audit; AC #21.

### ISS-005 — Fuzzy threshold unbounded (tenant could set 100% → match any invoice)
Resolved: §1 #14 + DEC-625 + CHECK constraint [1, 20] on cash_app_fuzzy_threshold_pct + default 5% + sev-2 audit on change; AC #14.

### ISS-006 — Append-only not enforced at SQL layer (handler bug could UPDATE history)
First-pass relied on handler discipline. Resolved: §1 #4 + DEC-617 + REVOKE UPDATE, DELETE FROM cyberos_app + privileged inv_cash_applier role + reversal-only mutation path; AC #2 + #3.

### ISS-007 — Auto-mark-paid drift (handler computed sum, trigger didn't fire)
Resolved: §1 #22 + DEC-633 + AFTER INSERT trigger on payment_allocations + invoice_outstanding_view centralises math + status transition to paid atomic with allocation; AC #24.

### ISS-008 — Scheduled job double-processing (5-min cron overlap)
First-pass had no lock. Resolved: §1 #10 + DEC-622 + pg_advisory_xact_lock(receipt_id) per receipt + skip if locked + sev-3 skipped audit; AC #9.

### ISS-009 — Dry-run could mutate state (CFO preview wrote rows)
Resolved: §1 #21 + DEC-631 + dry_run flag + ROLLBACK at end + zero-row INSERT proof + response shape parallel to live handler; AC #25.

## §3 — Resolution

All 9 mechanical concerns addressed. **Score = 10/10.**

Per feature-request-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (4-step closed cascade × append-only at SQL grant × atomic over-allocation defense × partial M:N allocation × reversal reciprocity × per-tenant fuzzy threshold with cap × CFO-only manual + sev-2 audit × 9 memory audit kinds × currency-mismatch hard skip × 5-min job advisory-lock × auto-mark-paid trigger × invoice_outstanding_view × dry-run handler × idempotent re-processing), not by line targets.

---

*End of FR-INV-006 audit.*
