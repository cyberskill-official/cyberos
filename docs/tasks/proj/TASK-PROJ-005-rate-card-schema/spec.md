---
id: TASK-PROJ-005
title: "Rate-card schema per Engagement — (role × currency × hourly_rate × billable_default) with effective-date versioning and TASK-AUTH-003 RLS"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: PROJ
priority: p0
status: done
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: 2026-05-23
memory_chain_hash: null
related_tasks: [TASK-PROJ-001, TASK-PROJ-006, TASK-PROJ-007, TASK-AUTH-003, TASK-MEMORY-101]
depends_on: [TASK-PROJ-001]
blocks: [TASK-PROJ-006, TASK-PROJ-007, TASK-CRM-004]

source_pages:
  - website/docs/modules/proj.html#rate-cards
  - website/docs/legal/vn-time-billing.html
source_decisions:
  - DEC-260 (rate cards are versioned by effective_from; supersession via new row, not UPDATE)
  - DEC-261 (currency is per-row, not per-engagement; multi-currency engagements legal)
  - DEC-262 (billable_default is the rate-card's recommendation; overridable per TASK-PROJ-006 cascade)

language: rust 1.81
service: cyberos/services/proj-sync/
new_files:
  - services/proj-sync/migrations/0005_rate_cards.sql
  - services/proj-sync/src/rate_card/mod.rs
  - services/proj-sync/src/rate_card/handlers.rs
  - services/proj-sync/src/rate_card/effective.rs
  - services/proj/tests/status_fsm_test.rs
modified_files:
  - services/proj-sync/src/types.rs                  # Role enum + Currency enum
allowed_tools:
  - file_read: services/proj-sync/**
  - file_write: services/proj-sync/{src,tests,migrations}/**
  - bash: cd services/proj-sync && cargo test rate_card
disallowed_tools:
  - UPDATE existing rate_cards rows (per DEC-260 — supersession only)
  - hardcode currency to VND (per DEC-261 — multi-currency support is first-class)

effort_hours: 4
subtasks:
  - "0.5h: 0005_rate_cards.sql migration (rate_cards table + UNIQUE constraint per (engagement_id, role, currency, effective_from))"
  - "0.5h: Role enum (engineer | designer | pm | qa | analyst | exec)"
  - "0.5h: Currency enum (VND | USD | SGD | EUR | JPY)"
  - "0.5h: rate_card/mod.rs — RateCard struct + DTOs"
  - "1.0h: handlers.rs — POST/GET/PATCH endpoints with idempotency + RLS"
  - "0.5h: effective.rs — lookup_at(engagement, role, currency, ts) returns effective rate at time"
  - "0.5h: memory audit row 'proj.rate_card_*' on create/supersede"
  - "1.0h: rate_card_test.rs — happy + supersession + effective-date lookup + RLS"
risk_if_skipped: "Without rate cards, every billable engagement loses traceability (what was the hourly rate at the time of work X?). Without versioning, rate changes corrupt historical entries (raising rates → old timesheets retroactively repriced). Without per-engagement scoping, all-or-nothing pricing across the org. Without currency, cross-border engagements (SGD client paying USD contractor) impossible. TASK-PROJ-006 cascade + TASK-PROJ-007 billing modes both require this schema."
---

## §1 — Description (BCP-14 normative)

The rate-card layer **MUST** persist per-engagement billing rates with effective-date versioning. The contract:

1. **MUST** define `rate_cards` table with columns: `id UUID PK`, `engagement_id UUID FK`, `role` (enum), `currency` (enum), `hourly_rate_minor` (BIGINT — amount in minor units: VND has 0 decimals = whole VND; USD has 2 = cents), `billable_default BOOLEAN`, `effective_from DATE`, `effective_to DATE` (nullable; null = current), `created_at TIMESTAMPTZ`, `created_by_subject_id UUID`, `tenant_id UUID`.
2. **MUST** support 6 canonical roles: `Engineer`, `Designer`, `Pm`, `Qa`, `Analyst`, `Exec`. v1 frozen; new roles via v2 migration.
3. **MUST** support 5 currencies: `VND` (0 decimals), `USD` (2), `SGD` (2), `EUR` (2), `JPY` (0).
4. **MUST** treat rate cards as APPEND-ONLY. `UPDATE rate_cards SET hourly_rate_minor = ...` is forbidden by RLS+role policy. Supersession = INSERT new row with `effective_from = today + 1` AND `UPDATE prior.effective_to = today`.
5. **MUST** enforce uniqueness: only ONE active rate card per `(engagement_id, role, currency)` tuple AT ANY GIVEN TIME. Overlapping intervals → 409 CONFLICT.
6. **MUST** expose `lookup_at(engagement_id, role, currency, at_date)` returning the rate card row whose `effective_from <= at_date AND (effective_to IS NULL OR effective_to > at_date)`. None → 404.
7. **MUST** expose REST endpoints:
    - `POST /api/proj/engagements/:eng/rate-cards` — create or supersede. Idempotent on `Idempotency-Key`.
    - `GET /api/proj/engagements/:eng/rate-cards?at=YYYY-MM-DD&role=engineer&currency=VND` — lookup.
    - `GET /api/proj/engagements/:eng/rate-cards/history` — full versioned history.
    - `PATCH /api/proj/engagements/:eng/rate-cards/:id` — ONLY `billable_default` mutable (rate fields immutable per #4); other PATCH fields rejected.
8. **MUST** emit memory audit rows:
    - `proj.rate_card_created` on new row.
    - `proj.rate_card_superseded` on supersession with payload `{old_card_id, new_card_id, old_rate_minor, new_rate_minor, currency, role, effective_from}`.
    - `proj.rate_card_billable_default_changed` on PATCH.
9. **MUST** enforce RLS per TASK-AUTH-003 — operators can only see/write rate cards for their tenant's engagements.
10. **MUST** validate amounts non-negative (`hourly_rate_minor >= 0`); zero allowed (pro bono work).
11. **MUST** emit OTel metric `proj_rate_cards_active{currency, role}` (gauge — count of currently-active cards per cell).
12. **SHOULD** support `cyberos rate-cards export <engagement_id>` CSV dump for finance reconciliation.
13. **MUST** support `POST /api/proj/engagements/:eng/rate-cards/preview` — dry-run that returns what would happen (new card + supersede chain) without persisting. Used by operators before committing rate changes.
14. **MUST** support retroactive corrections via a separate `proj.rate_card_corrected` audit row kind. Corrections require admin role AND `correction_reason` text. The corrected row is a NEW row with `effective_from < today`; the old row's `effective_to` is set to the new `effective_from`; the audit references both. Corrections are rare and audited heavily.
15. **MUST** expose a "default rate card pack" at engagement creation: tenant-admin defines `cyberos_proj_tenant_settings.default_rate_card_pack` (JSONB with role × currency × rate). New engagements get these as their initial rate cards. Operators can override per-engagement.
16. **MUST** support role aliasing for migrations: `cyberos_proj_role_aliases` table maps deprecated names (e.g. `developer` → `engineer`). Aliases resolve at the handler layer; storage always uses canonical roles.
17. **MUST** validate that `effective_from <= today + 365 days`: rate cards more than a year in the future suggest typo. Reject with 422 `effective_from_too_far`.
18. **MUST** record `archived_at TIMESTAMPTZ` separate from supersession: an engagement-archived state (TASK-PROJ-001) sets all the engagement's active rate cards' `archived_at = NOW()`; they remain queryable for historical reference but are excluded from `proj_rate_cards_active` gauge.
19. **MUST** support per-engagement rate-card lock: `cyberos_proj_engagement_settings.rate_cards_locked = true` rejects all create/supersede operations for that engagement (compliance reason: rate change requires legal review). Lock is operator-toggled with audit trail.
20. **MUST** validate currency consistency with engagement: if engagement has `default_currency = VND`, rate cards in other currencies emit a SEV-3 warning `proj.rate_card_currency_mismatch` (informational; doesn't reject; operators may legitimately have multi-currency engagements).
21. **MUST** support `?include_archived=true` query parameter on GET history endpoint; default behaviour excludes archived cards.
22. **MUST** include `created_via_ip + created_via_user_agent` in audit payload (NOT in the rate_cards row itself — too sensitive for general reads) for forensic traceability of rate changes.

---

## §2 — Why this design (rationale for humans)

**Why versioning via supersession (§1 #4, DEC-260)?** UPDATE of `hourly_rate_minor` retroactively re-prices old timesheets (a $100/h entry from January suddenly worth $150/h after a March raise). Append-only with `effective_from` means: timesheets reference the rate card that was active at the time of work — historically immutable. The auditor's question "what was the rate on March 15?" has one answer.

**Why per-row currency (§1 #3, DEC-261)?** Real engagements span currencies — a SG-based client pays the engineer in USD but the local PM in VND. Per-engagement currency would force collapse to one; per-row supports the actual case.

**Why minor units in BIGINT (§1 #1)?** Floats are unsafe for money (0.1 + 0.2 ≠ 0.3). VND/JPY have 0 decimals → minor = whole. USD/SGD/EUR have 2 → minor = cents. Storage as BIGINT covers up to 9.2 quintillion minor — far beyond any conceivable hourly rate.

**Why 6 canonical roles (§1 #2)?** Empirical from VN consultancy market: engineer + designer + PM + QA + analyst covers ~95% of billable roles; exec for senior advisory. More granular roles (senior-vs-junior engineer) belong in `Member.tier` (different task), not rate cards.

**Why only `billable_default` mutable (§1 #7)?** The rate is the contractual artifact — once a rate-card is referenced by a billable entry, changing it would be retroactive fraud. The default flag is a UX hint ("when creating a new task in this engagement, default-bill it") — changing it doesn't change historical entries.

**Why audit supersede separately (§1 #8)?** Auditors investigating "did this client get billed correctly through the rate change" follow the supersede chain. Generic `rate_card_created` audit rows lose the linkage; a dedicated kind with `old → new` payload makes the chain queryable.

**Why preview endpoint (§1 #13)?** Rate-card changes are high-stakes (affects billing for hundreds of timesheets). Preview lets operators verify the resulting supersede chain before committing. Drying-run prevents accidental retro overlaps.

**Why retroactive correction support (§1 #14)?** Real-world: rate was misconfigured at engagement start; discovered weeks later after billing. Need to correct the historical record AND audit the correction. A separate audit row kind makes this surgical.

**Why default rate-card pack (§1 #15)?** Tenants engaging multiple clients with similar rate structures (a consultancy with standard engineering/PM/QA tiers) shouldn't re-enter the same 6 rates per engagement. Pack at engagement creation = one-line setup.

**Why role aliasing (§1 #16)?** When migrating from another tool (or renaming roles internally), operators have legacy data referencing `developer` etc. Aliasing at the handler layer lets data migrate without database churn.

**Why 365-day-future cap (§1 #17)?** A rate card with `effective_from = 2030-01-01` is almost certainly a typo (operator meant 2026). The cap catches it before the rate ships.

**Why archived_at separate from supersede (§1 #18)?** Supersession means "this rate was replaced"; archival means "this engagement is no longer active." They're different states with different operator implications. Archived cards should disappear from active counts but remain queryable.

**Why rate-cards lock (§1 #19)?** Enterprise contracts often require legal sign-off for rate changes; the lock enforces that workflow at the data layer.

**Why currency-consistency warning (§1 #20)?** Operator setting a USD rate on a VND-default engagement is probably intentional (cross-border contractor) but worth surfacing. SEV-3 informational doesn't reject; just visible in operator dashboards.

**Why IP/UA in audit only (§1 #22)?** Rate-card rows are read frequently (lookup_at on every billable entry); embedding IP/UA in the table bloats the hot path. Audit rows are read rarely (forensics); fine to be richer.

---

## §3 — API contract

### Migration

```sql
-- services/proj-sync/migrations/0005_rate_cards.sql

CREATE TABLE rate_cards (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    engagement_id      UUID NOT NULL REFERENCES engagements(id),
    role               TEXT NOT NULL CHECK (role IN ('engineer','designer','pm','qa','analyst','exec')),
    currency           TEXT NOT NULL CHECK (currency IN ('VND','USD','SGD','EUR','JPY')),
    hourly_rate_minor  BIGINT NOT NULL CHECK (hourly_rate_minor >= 0),
    billable_default   BOOLEAN NOT NULL DEFAULT true,
    effective_from     DATE NOT NULL,
    effective_to       DATE,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by_subject_id UUID NOT NULL,
    tenant_id          UUID NOT NULL,
    UNIQUE (engagement_id, role, currency, effective_from)
);

-- Partial-unique: at most one ACTIVE card per (engagement, role, currency)
CREATE UNIQUE INDEX uniq_active_rate_card
    ON rate_cards (engagement_id, role, currency)
    WHERE effective_to IS NULL;

CREATE INDEX idx_rate_cards_lookup ON rate_cards (engagement_id, role, currency, effective_from, effective_to);

ALTER TABLE rate_cards ENABLE ROW LEVEL SECURITY;
CREATE POLICY rate_cards_tenant_isolation ON rate_cards
    USING (tenant_id = current_setting('app.tenant_id')::uuid);
```

### Rust types + handlers

```rust
// services/proj-sync/src/rate_card/mod.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum Role { Engineer, Designer, Pm, Qa, Analyst, Exec }

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash)]
#[sqlx(type_name = "TEXT")]
pub enum Currency { VND, USD, SGD, EUR, JPY }

impl Currency {
    pub fn decimals(self) -> u8 {
        match self { Currency::VND | Currency::JPY => 0, _ => 2 }
    }
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct RateCard {
    pub id:                    uuid::Uuid,
    pub engagement_id:         uuid::Uuid,
    pub role:                  Role,
    pub currency:              Currency,
    pub hourly_rate_minor:     i64,
    pub billable_default:      bool,
    pub effective_from:        chrono::NaiveDate,
    pub effective_to:          Option<chrono::NaiveDate>,
    pub created_at:            chrono::DateTime<chrono::Utc>,
    pub created_by_subject_id: uuid::Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum RateCardError {
    #[error("overlap with existing active card")] OverlapConflict,
    #[error("rate fields immutable; only billable_default can be patched")] ImmutableField,
    #[error("not found")] NotFound,
    #[error("negative amount: {0}")] NegativeAmount(i64),
    #[error("db: {0}")] Db(String),
}
```

```rust
// services/proj-sync/src/rate_card/handlers.rs
pub async fn create_or_supersede(
    pool: &sqlx::PgPool,
    engagement_id: uuid::Uuid,
    req: CreateReq,
    subject: uuid::Uuid,
) -> Result<RateCard, RateCardError> {
    if req.hourly_rate_minor < 0 { return Err(RateCardError::NegativeAmount(req.hourly_rate_minor)); }
    let mut tx = pool.begin().await.map_err(|e| RateCardError::Db(e.to_string()))?;

    // Close prior active card (effective_to = effective_from - 1 day)
    let prior: Option<RateCard> = sqlx::query_as(
        "SELECT * FROM rate_cards
         WHERE engagement_id = $1 AND role = $2 AND currency = $3 AND effective_to IS NULL
         FOR UPDATE"
    ).bind(engagement_id).bind(req.role).bind(req.currency).fetch_optional(&mut *tx).await
        .map_err(|e| RateCardError::Db(e.to_string()))?;

    if let Some(p) = &prior {
        if p.effective_from >= req.effective_from { return Err(RateCardError::OverlapConflict); }
        sqlx::query("UPDATE rate_cards SET effective_to = $1 WHERE id = $2")
            .bind(req.effective_from).bind(p.id)
            .execute(&mut *tx).await.map_err(|e| RateCardError::Db(e.to_string()))?;
    }

    // Insert new card
    let new_card: RateCard = sqlx::query_as(
        "INSERT INTO rate_cards (engagement_id, role, currency, hourly_rate_minor, billable_default,
                                  effective_from, created_by_subject_id, tenant_id)
         VALUES ($1,$2,$3,$4,$5,$6,$7, current_setting('app.tenant_id')::uuid)
         RETURNING *"
    ).bind(engagement_id).bind(req.role).bind(req.currency).bind(req.hourly_rate_minor)
     .bind(req.billable_default).bind(req.effective_from).bind(subject)
     .fetch_one(&mut *tx).await.map_err(|e| RateCardError::Db(e.to_string()))?;

    tx.commit().await.map_err(|e| RateCardError::Db(e.to_string()))?;

    if let Some(p) = prior {
        emit_memory_row("proj.rate_card_superseded", serde_json::json!({
            "old_card_id": p.id, "new_card_id": new_card.id,
            "old_rate_minor": p.hourly_rate_minor, "new_rate_minor": new_card.hourly_rate_minor,
            "currency": req.currency, "role": req.role,
            "effective_from": req.effective_from,
        })).await;
    } else {
        emit_memory_row("proj.rate_card_created", serde_json::json!({
            "card_id": new_card.id, "engagement_id": engagement_id,
            "role": req.role, "currency": req.currency,
            "hourly_rate_minor": new_card.hourly_rate_minor,
        })).await;
    }
    Ok(new_card)
}

pub async fn lookup_at(
    pool: &sqlx::PgPool,
    engagement_id: uuid::Uuid,
    role: Role,
    currency: Currency,
    at: chrono::NaiveDate,
) -> Result<RateCard, RateCardError> {
    sqlx::query_as(
        "SELECT * FROM rate_cards
         WHERE engagement_id = $1 AND role = $2 AND currency = $3
           AND effective_from <= $4 AND (effective_to IS NULL OR effective_to > $4)
         LIMIT 1"
    ).bind(engagement_id).bind(role).bind(currency).bind(at)
    .fetch_optional(pool).await
    .map_err(|e| RateCardError::Db(e.to_string()))?
    .ok_or(RateCardError::NotFound)
}
```

---

## §4 — Acceptance criteria

1. **Create rate card** — POST → 201; row in `rate_cards`; `effective_to` is null.
2. **Supersede on second create** — second POST same role/currency with later `effective_from` → prior row's `effective_to` set; new row inserted; only one active.
3. **Overlap rejected** — second POST with `effective_from <= prior.effective_from` → 409.
4. **lookup_at picks active card** — three cards across time; lookup at midpoint date returns the right one.
5. **lookup_at returns 404 outside range** — no active card on date → 404.
6. **Negative amount rejected** — `hourly_rate_minor: -1` → 422.
7. **PATCH rate fields rejected** — PATCH `hourly_rate_minor` → 422 `immutable_field`.
8. **PATCH billable_default succeeds** — PATCH `billable_default: false` → 200; original other fields preserved.
9. **History endpoint returns all versions** — 3 supersessions → 3 rows ordered by `effective_from` ASC.
10. **memory audit `rate_card_created`** — first create → row.
11. **memory audit `rate_card_superseded`** — supersede → row with old + new card IDs + delta.
12. **memory audit `rate_card_billable_default_changed`** — PATCH default → row.
13. **RLS isolates tenants** — tenant A cannot read tenant B's cards.
14. **Multi-currency on same engagement** — eng X with VND + USD cards simultaneously → both active.
15. **Idempotency-Key honoured** — same key + same body → returns prior outcome; same key + diff body → 409.
16. **OTel gauge `proj_rate_cards_active`** — 3 cards across 3 cells → gauge values match.
17. **CSV export** — `cyberos rate-cards export <eng>` → ordered CSV with all fields.
18. **Preview returns supersede chain without persist** — POST /preview → returns new+prior with effective_to update simulated; no DB change (AC for §1 #13).
19. **Retroactive correction emits dedicated audit** — admin POST with `correction_reason` + past `effective_from` → `proj.rate_card_corrected` row with both old + new IDs + reason (AC for §1 #14).
20. **Retroactive correction by non-admin rejected** — non-admin POST with past effective_from → 403 (AC for §1 #14).
21. **Default rate-card pack applied to new engagement** — engagement creation with `tenant.default_rate_card_pack` set → engagement has those cards (AC for §1 #15).
22. **Role alias resolves to canonical** — POST with `role=developer` (alias for engineer) → stored as `engineer`; audit + GET return canonical (AC for §1 #16).
23. **effective_from > today+365 rejected** — POST with `effective_from=2030-01-01` → 422 `effective_from_too_far` (AC for §1 #17).
24. **Engagement archive sets archived_at on cards** — archive engagement → all active cards' archived_at populated (AC for §1 #18).
25. **Archived cards excluded from active gauge** — gauge value drops after engagement archive (AC for §1 #18).
26. **Locked engagement rejects supersede** — set `rate_cards_locked=true`; POST → 409 `rate_cards_locked` (AC for §1 #19).
27. **Currency mismatch emits warning** — POST USD card on VND-default engagement → SEV-3 audit `proj.rate_card_currency_mismatch`; card still created (AC for §1 #20).
28. **History include_archived flag** — GET ?include_archived=true returns archived cards; default excludes them (AC for §1 #21).
29. **Audit captures IP + UA** — POST sets `created_via_ip` + `created_via_user_agent` in audit payload; absent in rate_cards row (AC for §1 #22).
30. **VND/JPY 0-decimal storage** — POST with `hourly_rate_minor=500000` for VND stores as 500000 (== 500,000 VND); USD 5000 = $50 (AC for §1 #1).
31. **Idempotency same key same body returns same outcome** — POST with same `Idempotency-Key` + identical body → same response (AC for §1 #15).
32. **Idempotency same key different body returns 409** — same key + modified body → 409 `idempotency_body_mismatch` (AC for §1 #15).

---

## §5 — Verification

```rust
// services/proj/tests/status_fsm_test.rs

#[tokio::test]
async fn create_and_supersede() {
    let env = TestEnv::new().await;
    let eng = env.create_engagement().await;
    let card1 = env.create_rate_card(eng, Role::Engineer, Currency::VND, 500_000, "2026-01-01").await.unwrap();
    let card2 = env.create_rate_card(eng, Role::Engineer, Currency::VND, 600_000, "2026-04-01").await.unwrap();

    let card1_refreshed: RateCard = sqlx::query_as("SELECT * FROM rate_cards WHERE id = $1")
        .bind(card1.id).fetch_one(&env.pool).await.unwrap();
    assert_eq!(card1_refreshed.effective_to, Some(chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap()));
    assert!(card2.effective_to.is_none());

    let supersede_row = env.memory.latest("proj.rate_card_superseded").await;
    assert_eq!(supersede_row["payload"]["old_rate_minor"], 500_000);
    assert_eq!(supersede_row["payload"]["new_rate_minor"], 600_000);
}

#[tokio::test]
async fn lookup_at_picks_active() {
    let env = TestEnv::new().await;
    let eng = env.create_engagement().await;
    env.create_rate_card(eng, Role::Engineer, Currency::VND, 500_000, "2026-01-01").await.unwrap();
    env.create_rate_card(eng, Role::Engineer, Currency::VND, 600_000, "2026-04-01").await.unwrap();

    let feb = lookup_at(&env.pool, eng, Role::Engineer, Currency::VND, "2026-02-15".parse().unwrap()).await.unwrap();
    assert_eq!(feb.hourly_rate_minor, 500_000);
    let may = lookup_at(&env.pool, eng, Role::Engineer, Currency::VND, "2026-05-15".parse().unwrap()).await.unwrap();
    assert_eq!(may.hourly_rate_minor, 600_000);
}

#[tokio::test]
async fn rate_field_immutable() {
    let env = TestEnv::new().await;
    let card = env.create_rate_card_default().await.unwrap();
    let err = patch_rate_card(&env.pool, card.id, json!({"hourly_rate_minor": 999})).await.unwrap_err();
    assert!(matches!(err, RateCardError::ImmutableField));
}

#[tokio::test]
async fn multi_currency_same_engagement() {
    let env = TestEnv::new().await;
    let eng = env.create_engagement().await;
    let vnd = env.create_rate_card(eng, Role::Engineer, Currency::VND, 500_000, "2026-01-01").await.unwrap();
    let usd = env.create_rate_card(eng, Role::Engineer, Currency::USD, 5000, "2026-01-01").await.unwrap();
    assert!(vnd.effective_to.is_none() && usd.effective_to.is_none());
}

#[tokio::test]
async fn rls_isolates_tenants() {
    let env_a = TestEnv::for_tenant("A").await;
    let env_b = TestEnv::for_tenant("B").await;
    let eng_a = env_a.create_engagement().await;
    env_a.create_rate_card(eng_a, Role::Engineer, Currency::VND, 500_000, "2026-01-01").await.unwrap();

    let cross: Result<RateCard, _> = sqlx::query_as("SELECT * FROM rate_cards WHERE engagement_id = $1")
        .bind(eng_a).fetch_one(&env_b.pool).await;
    assert!(cross.is_err());
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton.)

---

## §7 — Dependencies

- **TASK-PROJ-001** — `engagements` table FK target.
- **TASK-PROJ-006 (downstream)** — billable cascade uses `lookup_at` to resolve rates.
- **TASK-PROJ-007 (downstream)** — billing modes consume rate cards.
- **TASK-AUTH-003** — RLS on `rate_cards`.

---

## §8 — Example payloads

```json
{
  "kind": "proj.rate_card_superseded",
  "payload": {
    "old_card_id":    "rc-...",
    "new_card_id":    "rc-...",
    "old_rate_minor": 500000,
    "new_rate_minor": 600000,
    "currency":       "VND",
    "role":           "engineer",
    "effective_from": "2026-04-01"
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Per-member rate overrides (Alice's senior engineer rate ≠ junior default) — slice 3+; lives in TASK-PROJ-006 cascade.
- Tiered rates (first 40h/week at X, overtime at Y) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Concurrent supersede same cell | `FOR UPDATE` lock | Second caller sees prior's updated effective_to | Retries succeed |
| Overlap detected | effective_from check | 409 | Caller adjusts date |
| Negative amount | CHECK constraint | 422 | Caller fixes |
| PATCH non-billable_default field | handler whitelist | 422 | Caller uses correct field |
| Engagement deleted (cascade?) | FK ON DELETE RESTRICT | Cannot delete eng with cards | Operator archives engagement |
| Currency drift (insert "USDT") | CHECK constraint | 422 | Caller uses valid enum |
| Role drift | CHECK constraint | 422 | Caller uses valid enum |
| Effective_from in past with active card | overlap check | 409 if overlap | Caller adjusts |
| Active card without effective_to (orphan) | partial unique enforces ≤1 | None | None |
| RLS bypass | RLS policy | 0 rows / 404 | None |
| audit emit fails | MemoryWriter Err | Card created; audit lost; sev-2 | Operator restores memory |
| Lookup date is null | type system rejects | 422 | Caller provides date |
| BIGINT overflow | i64 ≈ 9.2 quintillion minor | unrealistic | None |
| ENG belongs to different tenant | RLS catches | 404 | Caller uses correct tenant context |
| Preview includes side effect (bug) | invariant test ensures no DB change | None | Author fixes |
| Retroactive correction without correction_reason | handler check | 400 `correction_reason_required` | Caller adds reason |
| Retroactive correction by non-admin | RBAC | 403 | Operator escalates |
| Default rate-card pack with invalid role | validate at engagement creation | 422 | Operator fixes tenant settings |
| Default pack updated mid-creation | engagement uses snapshot at start | None | None |
| Role alias maps to retired canonical | alias config flagged | 422 | Operator updates aliases |
| Role alias cycles (a → b → a) | resolver detects | 500 + SEV-1 | Operator |
| effective_from = today's date with active card | overlap check | 409 if overlap; OK if no prior | Caller |
| effective_from > today + 365d | range check | 422 | Caller |
| Engagement archived while rate change in flight | tx ordering: archive blocks supersede | 409 | Operator |
| Rate-card lock toggled during in-flight POST | lock check at top of handler | mid-flight may succeed; next rejects | None |
| Currency mismatch SEV-3 firing too often | dedup at OBS layer | None | Operator tunes |
| include_archived with large engagement | bounded by tenant limit (10k cards) | pagination | None |
| Audit IP captured incorrectly (proxy IP, not client) | use X-Forwarded-For if trusted | None | Operator validates trust chain |
| Audit UA truncated at 1KB | bounded; tail discarded | None | None |
| Concurrent supersede + correction | tx serialises via FOR UPDATE | one wins | None |
| Correction with effective_from after correction date | invalid: corrections are retroactive | 400 | Caller |
| Correction overlaps with already-superseded chain | resolver fans through chain | 409 if overlap | Caller |
| Default pack with all currencies for one role | engagement gets N cards (one per currency) | as designed | None |
| Tenant deletes default pack mid-creation | engagement defaults to no pack | None | None |
| Concurrent engagement creation with same default pack | each engagement independent | None | None |
| OTel gauge update fails | counter not updated; logged | None | None |
| Rate-card row > 10KB (huge metadata in audit) | metadata not in row; only audit | None | None |
| CSV export contains comma in display fields | RFC 4180 quoting | None | None |

---

## §11 — Implementation notes

- `hourly_rate_minor` is BIGINT (signed i64) — supports up to ~9.2 quintillion minor units. JPY at 0 decimals + i64 covers any conceivable rate.
- The partial-unique index on `(engagement_id, role, currency) WHERE effective_to IS NULL` enforces "≤ 1 active card per cell" without preventing historical rows.
- `effective_to = effective_from + 1 day` would be cleaner (closed-open interval) but Postgres DATE arithmetic makes the half-open `effective_to > $at` query natural; we use closed-open.
- The `Currency::decimals()` helper is used by TASK-PROJ-007 + task-INV when rendering amounts; minor unit display formula = `value / 10^decimals`.
- Idempotency uses the same `admin_idempotency_keys` table pattern as TASK-AUTH-001.
- Concurrency: `FOR UPDATE` lock on prior card row prevents two callers from both reading "no prior" and both inserting.
- Audit row currency stored as enum string (matches CHECK constraint).
- CSV export is the operator escape hatch; finance teams sometimes prefer Excel pivot over SQL.
- The preview endpoint is implemented as the same handler with a `commit=false` branch; this guarantees preview output exactly matches what commit would produce. No drift between paths.
- Retroactive correction is intentionally heavier (admin-only, requires reason, separate audit kind) because retroactive billing changes touch invoiced periods. The friction is the feature.
- Default rate-card pack at engagement creation deliberately copies into per-engagement rows (not references). Tenant changes to the pack don't affect already-created engagements.
- Role aliases are runtime-resolved (not DB-rewritten) because DB rewrite would break audit trails. Audit shows the canonical role; UI shows the alias if configured.
- The 365-day-future cap is a sanity check, not a contract enforcement; if a tenant genuinely needs further-out rates, an operator unlocks via tenant settings.
- `archived_at` vs supersession: supersede means "replaced by a newer rate"; archive means "engagement is done." A card can be both superseded AND archived. Both states are preserved.
- The rate-cards-locked flag is a per-engagement compliance hold; we considered a per-card lock but operators wanted engagement-level (atomic).
- Currency-mismatch is SEV-3 (informational) because: (a) multi-currency engagements are legitimate; (b) operators reading dashboards see it and decide; (c) hard-reject would block valid use cases.
- IP/UA in audit only (not in row) keeps the hot read path lean; only forensic queries need these fields.
- We considered storing rates as `pg_numeric(20,8)` instead of BIGINT minor units; BIGINT is faster + matches the canonical pattern in TASK-AUTH-001 + TASK-PROJ-001.
- `FOR UPDATE` lock is on the prior active row, not all rows. Lock scope is bounded.
- We don't support rate_card deletion — even archive doesn't delete; data is forever-queryable for audit reconstruction.
- Engagement deletion (TASK-PROJ-001) is `ON DELETE RESTRICT` for rate cards: operators must explicitly archive cards before engagement deletion.
- Per-engagement rate-card export (CSV) is the only export path; cross-engagement export deferred (slice 4+).
- The `proj_rate_cards_active` gauge updates on every create/supersede/archive; computation = `SELECT COUNT(*) FROM rate_cards WHERE effective_to IS NULL AND archived_at IS NULL`.
- We chose 6 canonical roles based on Vietnamese consultancy market survey; tenants outside that set can use the role-aliases mechanism rather than custom roles.
- The `effective_from` is DATE (not TIMESTAMPTZ); rate changes apply at midnight tenant-local time. Sub-day precision adds complexity without operational value.
- For correction workflows, the previously-active card's `effective_to` is set to the new card's `effective_from`. This means a correction inserted at `effective_from = 2026-03-15` closes the prior at `2026-03-14`; new card opens `2026-03-15`. Half-open interval semantics.
- Default rate-card pack JSONB schema is open (per-tenant extension); tooling validates structure at engagement creation.
- We chose to keep the role enum closed (no operator-defined roles) to prevent rate-card sprawl; the alias mechanism handles compat naming.

---

*End of TASK-PROJ-005.*
