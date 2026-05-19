---
id: FR-INV-002
title: "INV multi-currency support — VND/USD/SGD/EUR/GBP with daily SBV FX snapshot + per-invoice currency lock + cross-currency reporting"
module: INV
priority: MUST
status: draft
verify: T
phase: P2
milestone: P2 · slice 1
slice: 1
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-INV-001, FR-INV-011, FR-TEN-003, FR-TEN-102, FR-AI-003, FR-MEMORY-111]
depends_on: [FR-INV-001]
blocks: []

source_pages:
  - website/docs/modules/inv.html#fx
  - https://sbv.gov.vn/  # State Bank of Vietnam reference rates

source_decisions:
  - DEC-1510 2026-05-17 — Per-invoice currency locked at creation (matches engagement.billing_currency); cross-currency conversion ONLY for reporting (not invoice mutation)
  - DEC-1511 2026-05-17 — Daily SBV FX rate snapshot at 09:00 UTC (2pm Vietnam) — official VN reference for VND conversions; USD/SGD/EUR/GBP cross-rates derived
  - DEC-1512 2026-05-17 — Closed enum `fx_source` = {sbv_daily, ecb_daily, manual_override}; cardinality 3
  - DEC-1513 2026-05-17 — Cross-currency reporting: report at "as-of" date pulls FX from that day's snapshot (deterministic — same query same date = same result)
  - DEC-1514 2026-05-17 — Per-tenant base currency for financial reports (default = engagement's currency; tenant may set tenant-wide base)
  - DEC-1515 2026-05-17 — memory audit kinds: inv.fx_snapshot_recorded, inv.fx_snapshot_failed, inv.fx_manual_override, inv.report_currency_converted

build_envelope:
  language: rust 1.81
  service: cyberos/services/inv/
  new_files:
    - services/inv/migrations/0006_fx_rates.sql
    - services/inv/src/fx/mod.rs
    - services/inv/src/fx/sbv_fetcher.rs
    - services/inv/src/fx/ecb_fetcher.rs
    - services/inv/src/fx/snapshot_job.rs
    - services/inv/src/fx/converter.rs
    - services/inv/src/fx/manual_override.rs
    - services/inv/src/audit/fx_events.rs
    - services/inv/src/handlers/fx_routes.rs
    - services/inv/tests/fx_sbv_fetch_test.rs
    - services/inv/tests/fx_daily_snapshot_test.rs
    - services/inv/tests/fx_as_of_deterministic_test.rs
    - services/inv/tests/fx_manual_override_test.rs
    - services/inv/tests/fx_source_enum_test.rs
    - services/inv/tests/fx_source_unavailable_test.rs
    - services/inv/tests/fx_audit_emission_test.rs

  modified_files:
    - services/inv/src/lib.rs

  allowed_tools:
    - file_read: services/inv/**
    - file_write: services/inv/{src,tests,migrations}/**
    - bash: cd services/inv && cargo test fx

  disallowed_tools:
    - mutate invoice currency post-creation (per DEC-1510)
    - skip daily snapshot (per DEC-1511)
    - allow non-CFO manual override (per DEC-1512)

effort_hours: 6
sub_tasks:
  - "0.4h: 0006_fx_rates.sql + closed enum"
  - "0.3h: fx/mod.rs"
  - "0.5h: sbv_fetcher.rs (SBV API scraping; fallback to PDF parser)"
  - "0.5h: ecb_fetcher.rs (ECB API)"
  - "0.4h: snapshot_job.rs (daily 09:00 UTC)"
  - "0.4h: converter.rs (per-date FX lookup)"
  - "0.3h: manual_override.rs (CFO-gated)"
  - "0.3h: audit/fx_events.rs"
  - "0.3h: handlers/fx_routes.rs"
  - "1.5h: tests — 7 test files"
  - "0.6h: integration with FR-INV-001 reporting layer"

risk_if_skipped: "Without FX, multi-currency tenants (USD invoices + VND tenants per FR-TEN-102) cannot produce consolidated financial reports → CFO unable to answer 'total revenue this quarter in VND'. Without DEC-1511 daily snapshot, report numbers shift retroactively as rates change. Without DEC-1513 deterministic as-of, same report two days apart returns different numbers → trust break."
---

## §1 — Description (BCP-14 normative)

The INV service **MUST** ship multi-currency FX support at `services/inv/src/fx/` with daily SBV+ECB snapshots, per-date deterministic conversion, CFO-gated manual override, and 4 memory audit kinds.

1. **MUST** define closed `fx_source` enum: `('sbv_daily','ecb_daily','manual_override')` per DEC-1512. Cardinality 3.

2. **MUST** define `fx_rates` table at migration `0006`: `(snapshot_date DATE NOT NULL, base_currency billing_currency_enum NOT NULL, quote_currency billing_currency_enum NOT NULL, rate NUMERIC(18,8) NOT NULL CHECK (rate > 0), source fx_source NOT NULL, recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(), recorded_by_subject_id UUID, override_reason TEXT, PRIMARY KEY (snapshot_date, base_currency, quote_currency))`.

3. **MUST** invoice currency immutable per DEC-1510 — already enforced by FR-INV-001 + engagement.billing_currency immutability per FR-TEN-003.

4. **MUST** snapshot SBV daily at 09:00 UTC per DEC-1511 via `snapshot_job.rs`:
   - Fetch SBV reference rates (VND base; USD/EUR/JPY/CNY/etc cross-rates).
   - Persist all (VND, X) pairs.
   - Fetch ECB rates as cross-validation + non-SBV pairs.
   - Failure → emit `inv.fx_snapshot_failed` sev-2; retry hourly.

5. **MUST** support deterministic as-of conversion per DEC-1513 via `converter.rs::convert(amount_minor, from, to, as_of_date)`:
   - Lookup `fx_rates(as_of_date, from, to)`.
   - If direct rate missing, compute via VND base: `from→VND→to`.
   - Returns same value on every call with same inputs.
   - If no rate available for date → 412 + `no_fx_for_date`.

6. **MUST** support CFO manual override via `POST /v1/admin/inv/fx/override` body `{ snapshot_date, base, quote, rate, reason }`. Caller has `cfo`. Inserts row with source='manual_override' + emits `inv.fx_manual_override` sev-1.

7. **MUST** consume by INV-011 (revenue recognition) + reporting endpoints for cross-currency rollup.

8. **MUST** emit 4 memory audit kinds per DEC-1515.

9. **MUST** thread trace_id end-to-end.

10. **MUST NOT** mutate invoice currency post-creation (DEC-1510).

11. **MUST NOT** allow non-CFO override (DEC-1512).

---

## §2 — Why this design

**Why SBV official rate (DEC-1511)?** VN tax law references SBV daily rate for VAT/CIT calculations; using SBV ensures regulatory alignment.

**Why per-date snapshot (DEC-1513)?** Reports referencing past dates must reproduce. Snapshot-based vs live FX = deterministic.

**Why CFO manual override (DEC-1512)?** Edge cases (weekend dates, missing SBV publication days, treaty rates) need a manual path with audit trail.

---

## §3 — API contract

```sql
CREATE TYPE fx_source AS ENUM ('sbv_daily','ecb_daily','manual_override');

CREATE TABLE fx_rates (
  snapshot_date DATE NOT NULL,
  base_currency billing_currency_enum NOT NULL,
  quote_currency billing_currency_enum NOT NULL,
  rate NUMERIC(18,8) NOT NULL CHECK (rate > 0),
  source fx_source NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  recorded_by_subject_id UUID,
  override_reason TEXT,
  trace_id CHAR(32),
  PRIMARY KEY (snapshot_date, base_currency, quote_currency)
);
CREATE INDEX idx_fx_date ON fx_rates(snapshot_date DESC);
REVOKE UPDATE, DELETE ON fx_rates FROM cyberos_app;
```

Endpoints:
```text
GET    /v1/inv/fx/rates?as_of=...&base=VND
POST   /v1/admin/inv/fx/override                  (cfo)
GET    /v1/inv/fx/convert?amount=...&from=USD&to=VND&as_of=...
```

---

## §4 — Acceptance criteria
1. **fx_source cardinality 3**. 2. **Daily SBV snapshot persists VND pairs**. 3. **ECB fallback for non-VND pairs**. 4. **As-of conversion deterministic** — same inputs same output. 5. **Direct pair preferred over via-VND**. 6. **Via-VND cross-rate computation** — USD→EUR via VND. 7. **Missing date → 412**. 8. **Manual override CFO-only**. 9. **Override reason required**. 10. **4 memory audit kinds emitted**. 11. **Snapshot retry on failure**. 12. **Invoice currency immutable**. 13. **Trace_id end-to-end**. 14. **PII scrub override reason**. 15. **Concurrent snapshot race-safe** (PRIMARY KEY). 16. **Cross-currency report uses as-of correctly**. 17. **Rate CHECK > 0**. 18. **Weekend SBV gap handled** — falls forward to prior Friday rate. 19. **Per-tenant base currency for reports**. 20. **All 5 currencies supported** (VND/USD/SGD/EUR/GBP).

---

## §5 — Verification

```rust
#[tokio::test]
async fn daily_snapshot_persists() {
    let ctx = TestContext::with_mocked_sbv().await;
    ctx.run_snapshot_job(today()).await;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM fx_rates WHERE snapshot_date=$1")
        .bind(today()).fetch_one(&ctx.pool).await.unwrap();
    assert!(count >= 4);  // VND-USD, VND-EUR, VND-SGD, VND-GBP minimum
}

#[tokio::test]
async fn as_of_deterministic() {
    let ctx = TestContext::with_fx_snapshot(date!(2026-05-15)).await;
    let r1 = ctx.convert(100_00, "USD", "VND", date!(2026-05-15)).await;
    let r2 = ctx.convert(100_00, "USD", "VND", date!(2026-05-15)).await;
    assert_eq!(r1, r2);
}

#[tokio::test]
async fn manual_override_cfo_only() {
    let ctx = TestContext::new().await;
    let r = ctx.as_engagement_admin().fx_override(date!(2026-05-15), "VND", "USD", 24500).await;
    assert_eq!(r.status(), 403);
    let r2 = ctx.as_cfo().fx_override(date!(2026-05-15), "VND", "USD", 24500).await;
    assert_eq!(r2.status(), 200);
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-INV-001.
**Cross-module:** FR-AUTH-101 (cfo role), FR-AI-003, FR-MEMORY-111. **Consumed by:** FR-INV-011, FR-TEN-003, FR-TEN-102 reporting.

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| SBV API down | timeout | Failed snapshot; retry hourly | ECB fallback temporary |
| ECB API down | timeout | Sev-2 alert; manual override path | Inherent |
| Weekend gap | date-not-found | Falls forward to prior weekday rate | Inherent |
| Future date conversion | check | 412 | Inherent |
| Cross-rate inconsistency (SBV vs ECB) | drift > 1% | Sev-3 alert | Manual review |
| Rate precision overflow | NUMERIC(18,8) | INSERT fail | Use BigDecimal lib |
| Snapshot race | PRIMARY KEY | One wins | Inherent |
| Manual override conflicts with snapshot | source enum | Override row coexists; converter prefers manual | Inherent |
| Concurrent override | tx isolation | Last-writer-wins | Inherent |
| Currency not in enum | billing_currency_enum check | 400 | Inherent |
| SBV publishes different rate retroactively | doesn't happen per SBV policy | N/A | Inherent |
| Tenant base currency change | grandfather past reports | Inherent | Reports re-render with current base |

## §11 — Implementation notes
- §11.1 SBV scraping via `sbv.gov.vn` daily PDF; backup ECB JSON API.
- §11.2 Snapshot at 09:00 UTC = 2pm Vietnam time; SBV publishes by then.
- §11.3 Cross-rate via VND base when direct missing; documented as canonical path.
- §11.4 NUMERIC(18,8) gives 10 decimals + 8 fractional — sufficient for any FX precision.
- §11.5 Weekend gap rule: use Friday's rate for Sat/Sun queries.

---

*End of FR-INV-002 spec.*
