---
id: TASK-INV-009
title: "INV AR aging report — current/30/60/90/120+ bucket rollup per customer + per engagement with as-of date determinism"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: INV
priority: p0
status: draft
verify: T
phase: P2
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-INV-001, TASK-INV-002, TASK-INV-010, TASK-INV-011, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-INV-001]
blocks: [TASK-INV-010]

source_pages:
  - website/docs/modules/inv.html#aging

source_decisions:
  - DEC-1540 2026-05-17 — Aging buckets: current (≤0d overdue), 1-30d, 31-60d, 61-90d, 91-120d, 120+d (6 buckets); industry-standard
  - DEC-1541 2026-05-17 — As-of date deterministic — same as_of_date + same data = identical report (no clock drift)
  - DEC-1542 2026-05-17 — Closed enum `aging_status` = {current, overdue_30, overdue_60, overdue_90, overdue_120, overdue_120plus}; cardinality 6
  - DEC-1543 2026-05-17 — Partial-paid invoices: bucket on REMAINING balance; not full invoice amount
  - DEC-1544 2026-05-17 — Multi-currency support via TASK-INV-002 snapshot at as_of_date (cross-currency rollup uses report base currency)
  - DEC-1545 2026-05-17 — Per-customer + per-engagement + tenant-wide rollup variants
  - DEC-1546 2026-05-17 — memory audit kinds: inv.aging_report_generated (no PII in chain, only count + total)

build_envelope:
  language: rust 1.81
  service: cyberos/services/invoicing/
  new_files:
    - services/invoicing/src/reports/aging.rs
    - services/invoicing/src/reports/aging_bucketer.rs
    - services/invoicing/src/handlers/aging_routes.rs
    - services/invoicing/src/audit/aging_events.rs
    - services/invoicing/tests/aging_bucket_test.rs
    - services/invoicing/tests/aging_as_of_determinism_test.rs
    - services/invoicing/tests/aging_partial_paid_test.rs
    - services/invoicing/tests/aging_multi_currency_test.rs
    - services/invoicing/tests/aging_status_enum_cardinality_test.rs
    - services/invoicing/tests/aging_audit_emission_test.rs

  modified_files:
    - services/invoicing/src/lib.rs

  allowed_tools:
    - file_read: services/invoicing/**
    - file_write: services/invoicing/{src,tests}/**
    - bash: cd services/invoicing && cargo test aging

  disallowed_tools:
    - bucket on full invoice when partial-paid (per DEC-1543)
    - use now() in bucket calc (per DEC-1541 — as_of only)

effort_hours: 4
subtasks:
  - "0.4h: reports/aging.rs"
  - "0.5h: aging_bucketer.rs"
  - "0.4h: handlers/aging_routes.rs"
  - "0.3h: audit/aging_events.rs"
  - "1.6h: tests — 6 test files"
  - "0.8h: AR aging dashboard UI hook (TASK-CUO-101)"

risk_if_skipped: "Without AR aging, CFO cannot prioritize collection — late-stage receivables become bad debt. Without DEC-1541 determinism, monthly reports vary by minute (audit pain). Without DEC-1543 partial-paid handling, paid-down invoices skew old buckets (false signal)."
---

## §1 — Description (BCP-14 normative)

The INV service **MUST** ship AR aging at `services/invoicing/src/reports/aging.rs` returning 6-bucket overdue rollup per as_of_date with per-customer / per-engagement / tenant-wide variants, multi-currency conversion via TASK-INV-002, 1 memory audit kind.

1. **MUST** expose `POST /v1/inv/reports/aging` body `{ as_of_date, group_by?: 'customer'|'engagement'|'tenant', base_currency? }`. Auth via TASK-AUTH-101 (CFO + accountant roles).

2. **MUST** bucket via `aging_bucketer.rs::bucket(invoice, as_of_date)`:
   - Compute `days_overdue = as_of_date - invoice.due_date`.
   - Map to enum per DEC-1542: `current` (≤0), `overdue_30` (1-30), `overdue_60` (31-60), `overdue_90` (61-90), `overdue_120` (91-120), `overdue_120plus` (>120).
   - Use `invoice.outstanding_balance` (not total) per DEC-1543.

3. **MUST** use as_of_date for ALL calculations per DEC-1541 — never `now()`. Same params → same SQL → same result.

4. **MUST** support multi-currency per DEC-1544: when `base_currency` differs from invoice currency, convert via TASK-INV-002 `fx_snapshot(currency_pair, as_of_date)`. Missing FX → fall back to nearest-prior with sev-2 audit.

5. **MUST** return rollup per group_by:
   - `customer`: `[{customer_id, current, overdue_30, ..., total_outstanding, currency}]`
   - `engagement`: `[{engagement_id, ..., total_outstanding}]`
   - `tenant`: single row with bucket sums

6. **MUST** exclude `cancelled` and `paid` invoices from buckets; include `sent`, `partial_paid`, `overdue` statuses.

7. **MUST** emit `inv.aging_report_generated` per DEC-1546 with `{as_of_date, group_by, customer_count, invoice_count, total_outstanding_hash}` — total amount SHA-256 hashed per TASK-MEMORY-111 (treat AR totals as confidential).

8. **MUST** thread trace_id from CFO action through bucketer + FX lookup + audit emission.

9. **MUST NOT** use `now()` per DEC-1541; reject if `as_of_date` missing.

10. **MUST NOT** bucket on full invoice when partial-paid (use outstanding_balance per DEC-1543).

---

## §2 — Why this design

**Why 6 buckets (DEC-1540)?** Industry standard (current/30/60/90/120+); CFO conditioning + tax/audit templates.

**Why as-of determinism (DEC-1541)?** Monthly close reports must be re-runnable years later with same input → same output. Without this, audits fail.

**Why outstanding_balance bucketing (DEC-1543)?** A 90-day-old invoice partially paid to $0 should be `current`, not `overdue_90`. Bucketing on full amount mis-prioritizes collection.

**Why FX at as-of (DEC-1544)?** Aging report = financial snapshot at date X; FX must match. Otherwise dashboard shows wrong USD totals.

---

## §3 — API contract

```text
POST   /v1/inv/reports/aging              (CFO/accountant)
GET    /v1/inv/reports/aging?as_of=...    (cached if recent)
```

Sample request:
```json
{
  "as_of_date": "2026-05-31",
  "group_by": "customer",
  "base_currency": "USD"
}
```

Sample response:
```json
{
  "as_of_date": "2026-05-31",
  "base_currency": "USD",
  "buckets": [
    {
      "customer_id": "uuid",
      "customer_name": "Acme Corp",
      "current": 5000.00,
      "overdue_30": 2500.00,
      "overdue_60": 1200.00,
      "overdue_90": 0,
      "overdue_120": 0,
      "overdue_120plus": 800.00,
      "total_outstanding": 9500.00,
      "invoice_count": 7,
      "currency": "USD",
      "fx_converted": false
    }
  ],
  "summary": {
    "total_outstanding": 9500.00,
    "customer_count": 1,
    "invoice_count": 7
  }
}
```

---

## §4 — Acceptance criteria
1. **6 bucket categories**. 2. **Closed enum cardinality 6**. 3. **As_of_date required (400 if missing)**. 4. **Determinism: same params = same result**. 5. **Outstanding_balance used (not total)**. 6. **Multi-currency conversion via TASK-INV-002**. 7. **FX at as_of_date (not now)**. 8. **Cancelled + paid excluded**. 9. **Sent/partial_paid/overdue included**. 10. **Group_by customer/engagement/tenant**. 11. **1 memory audit kind emitted**. 12. **PII scrubbed (total hashed)**. 13. **RLS denies cross-tenant**. 14. **CFO + accountant roles only**. 15. **Trace_id preserved**. 16. **FX missing → nearest-prior + sev-2 audit**. 17. **Empty result returns empty array (not 404)**. 18. **Days_overdue boundary edges (0,1,30,31,etc) correct per DEC-1540**. 19. **Pagination supported for >1000 customers**. 20. **JSON output deterministic ordering by customer_id**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn six_buckets_correct() {
    let ctx = TestContext::vn_tenant_with_invoices_at_various_ages().await;
    let report = ctx.aging_report("2026-05-31", "customer", "VND").await;
    assert_eq!(report.buckets[0].current, dec!(5000));
    assert_eq!(report.buckets[0].overdue_30, dec!(2500));
    // ... etc
}

#[tokio::test]
async fn as_of_determinism() {
    let ctx = TestContext::vn_tenant_with_invoices().await;
    let r1 = ctx.aging_report("2026-05-31", "tenant", "VND").await;
    let r2 = ctx.aging_report("2026-05-31", "tenant", "VND").await;
    assert_eq!(r1, r2);
}

#[tokio::test]
async fn partial_paid_uses_outstanding() {
    let ctx = TestContext::invoice_paid_50pct(1000, 90).await;  // 90d old, 50% paid
    let report = ctx.aging_report("2026-05-31", "tenant", "VND").await;
    assert_eq!(report.buckets[0].overdue_90, dec!(500));  // not 1000
}

// 5.4..5.10 — FX, audit, enum cardinality, cancelled excluded
```

---

## §6 — Skeleton

```rust
pub async fn generate(req: AgingRequest, db: &Db) -> Result<AgingReport> {
    if req.as_of_date.is_none() { return Err(400.into()); }
    let invoices = db.fetch_invoices_for_aging(req.as_of_date, req.group_by).await?;
    let buckets = invoices.into_iter()
        .filter(|i| !matches!(i.status, "cancelled" | "paid"))
        .map(|i| {
            let days_overdue = (req.as_of_date - i.due_date).num_days();
            let bucket = aging_bucketer::bucket(days_overdue);
            let amount = if let Some(base) = &req.base_currency {
                fx::convert(i.outstanding_balance, i.currency, base, req.as_of_date).await?
            } else { i.outstanding_balance };
            (i.customer_id, bucket, amount)
        })
        .collect();
    audit::emit("inv.aging_report_generated", json!({
        "as_of_date": req.as_of_date, "customer_count": ..., "total_outstanding_hash": sha256(total)
    }), trace).await?;
    Ok(roll_up(buckets, req.group_by))
}
```

---

## §7 — Dependencies
**Upstream:** TASK-INV-001, TASK-INV-002.
**Downstream:** TASK-INV-010 (dunning uses aging output).
**Cross-module:** TASK-AUTH-101 (role check), TASK-MEMORY-111 (PII).

## §8 — Sample payloads (see §3)

## §9 — Open questions
None blocking.

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| as_of_date missing | validate | 400 | provide date |
| FX rate missing for date | snapshot lookup | nearest-prior + sev-2 audit | per TASK-INV-002 |
| Customer has no invoices | empty group | omit from response | inherent |
| Invoice with NULL due_date | filter | excluded + warning audit | data fix |
| Currency conversion fail (no FX) | hard fail | sev-1 + report fails | manual FX entry |
| Massive tenant (>10k invoices) | pagination | cursor-based | inherent |
| Concurrent aging while invoice updated | snapshot SQL | uses snapshot row state | inherent |
| Decimal precision drift | use rust_decimal | 4 decimal places preserved | inherent |
| Bucket edge (1d vs 0d) | strict `<=` per spec | per DEC-1540 | tests verify |
| Cross-tenant query | RLS | 0 rows | inherent |

## §11 — Implementation notes
- §11.1 Bucketer is pure function; deterministic input → output.
- §11.2 SQL uses `WHERE due_date < as_of_date AND status NOT IN ('cancelled','paid')`.
- §11.3 FX conversion at row level (not aggregate) — preserves per-invoice currency context.
- §11.4 memory audit total_outstanding hashed (SHA256(amount.to_string()) per TASK-MEMORY-111).
- §11.5 Aging report is read-only; no row mutations; no .lock needed.

---

*End of TASK-INV-009 spec.*
