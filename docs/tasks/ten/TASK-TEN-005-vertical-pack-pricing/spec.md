---
id: TASK-TEN-005
title: "TEN vertical-pack pricing add-on — per-pack monthly fee (not per-seat) on top of base plan tier; multi-currency; prorate on install/uninstall"
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
module: TEN
priority: p0
status: draft
verify: T
phase: P4
milestone: P4 · marketplace
slice: 2
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-TEN-002, TASK-TEN-003, TASK-TEN-102, TASK-TEN-004, TASK-SKILL-107, TASK-INV-001, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-111, TASK-OBS-007]
depends_on: [TASK-TEN-002, TASK-SKILL-107]
blocks: []

source_pages:
  - website/docs/modules/ten.html#vertical-packs
  - website/docs/modules/skill.html#vertical-packs

source_decisions:
  - DEC-1300 2026-05-17 — Vertical packs (e.g. "Legal Pack", "Construction Pack" per TASK-SKILL-107) priced as flat per-pack-per-month add-on; NOT per-seat (consultancies often have 1 partner managing 50 client engagements through the same pack — per-seat would double-bill)
  - DEC-1301 2026-05-17 — Per-pack base price set by CyberSkill (the pack publisher) + per-tenant override allowed for sales-led discounting
  - DEC-1302 2026-05-17 — Multi-currency: pack prices in PRICE_CATALOG-style matrix per (pack_id × currency × tier-uplift); 12 base entries per pack × N currencies × 3 tiers (some packs only available on Team+/Enterprise)
  - DEC-1303 2026-05-17 — Installation triggers prorated invoice via TASK-TEN-003 (stripe) or TASK-TEN-102 (vnd); uninstall triggers prorated credit (Stripe) or skip-next-billing (vnd)
  - DEC-1304 2026-05-17 — Closed enum `pack_install_status` = {pending, active, uninstalled, suspended_billing_failed, blocked_tier}; CI cardinality 5
  - DEC-1305 2026-05-17 — Pack uninstall is 14-day soft (data + skill access retained, can re-install without re-onboarding); after 14d hard-removed (skill bundle deactivated, data flagged for retention per tenant policy)
  - DEC-1306 2026-05-17 — Tier gating: per-pack `min_tier` constraint (e.g. compliance packs Team+ only); install attempt on lower tier → 403 `tier_upgrade_required`
  - DEC-1307 2026-05-17 — Per-tenant install limit: 50 packs/tenant (Enterprise can request override); prevent skill-bundle bloat
  - DEC-1308 2026-05-17 — memory audit kinds: ten.pack_installed, ten.pack_uninstalled, ten.pack_billing_failed, ten.pack_price_overridden, ten.pack_tier_blocked
  - DEC-1309 2026-05-17 — Pack price changes by publisher (CyberSkill) apply at next billing period; existing tenants grandfathered for 90 days

build_envelope:
  language: rust 1.81
  service: cyberos/services/ten/
  new_files:
    - services/ten/migrations/0023_vertical_pack_installs.sql
    - services/ten/migrations/0024_vertical_pack_price_catalog.sql
    - services/ten/migrations/0025_vertical_pack_overrides.sql
    - services/ten/src/packs/mod.rs
    - services/ten/src/packs/install.rs
    - services/ten/src/packs/uninstall.rs
    - services/ten/src/packs/price_resolver.rs
    - services/ten/src/packs/billing_push.rs
    - services/ten/src/packs/tier_gate.rs
    - services/ten/src/audit/pack_events.rs
    - services/ten/src/handlers/pack_routes.rs
    - services/ten/tests/pack_install_test.rs
    - services/ten/tests/pack_uninstall_prorate_test.rs
    - services/ten/tests/pack_tier_gate_test.rs
    - services/ten/tests/pack_per_tenant_override_test.rs
    - services/ten/tests/pack_install_status_enum_test.rs
    - services/ten/tests/pack_50_install_limit_test.rs
    - services/ten/tests/pack_soft_uninstall_14d_test.rs
    - services/ten/tests/pack_billing_failed_suspends_test.rs
    - services/ten/tests/pack_audit_emission_test.rs
    - services/ten/tests/pack_grandfathered_pricing_test.rs

  modified_files:
    - services/ten/src/lib.rs
    # add active-pack lookup
    - services/skill/src/registry.rs

  allowed_tools:
    - file_read: services/{ten,skill,inv}/**
    - file_write: services/ten/{src,tests,migrations}/**
    - file_write: services/skill/src/registry.rs
    - bash: cd services/ten && cargo test packs

  disallowed_tools:
    - per-seat pricing on packs (per DEC-1300 — flat-fee only)
    - install pack below min_tier (per DEC-1306)
    - exceed 50-pack limit without Enterprise override (per DEC-1307)
    - apply publisher price change retroactively (per DEC-1309 — 90d grandfather)

effort_hours: 5
subtasks:
  - "0.4h: 0023 + 0024 + 0025 migrations"
  - "0.4h: packs/mod.rs + closed enum"
  - "0.5h: packs/install.rs (tier-gate + 50-cap + provision skill bundle)"
  - "0.4h: packs/uninstall.rs (14d soft → hard)"
  - "0.5h: packs/price_resolver.rs (catalog + override resolution)"
  - "0.5h: packs/billing_push.rs (stripe + vnd dispatch)"
  - "0.3h: packs/tier_gate.rs"
  - "0.3h: audit/pack_events.rs (5 builders)"
  - "0.3h: handlers/pack_routes.rs"
  - "1.0h: tests — 10 test files"
  - "0.4h: skill registry integration"

risk_if_skipped: "Without per-pack pricing, the SKILL marketplace (TASK-SKILL-107) has nothing to bill — packs are free, no revenue from the marketplace. TASK-TEN-002 plan tiers monetise the platform; TASK-TEN-005 monetises the marketplace. Without DEC-1300 flat-fee model, consultancies churn (per-seat pricing penalises their core workflow). Without DEC-1306 tier gating, free-tier tenants install expensive Enterprise-only packs. Without DEC-1305 14d soft-uninstall, accidental uninstalls force costly re-onboarding. The 5h effort lands the marketplace revenue primitive."
---

## §1 — Description (BCP-14 normative)

The TEN service **MUST** ship vertical-pack pricing add-on at `services/ten/src/packs/` with flat per-pack-per-month pricing, multi-currency catalog, per-tenant override, tier-gating, 14-day soft uninstall, install limit, prorated billing via TASK-TEN-003/102, and 5 memory audit kinds.

1. **MUST** define closed `pack_install_status` enum: `('pending','active','uninstalled','suspended_billing_failed','blocked_tier')` per DEC-1304. Cardinality asserts 5.

2. **MUST** define `vertical_pack_installs` at migration `0023`: `(install_id UUID PRIMARY KEY, tenant_id UUID NOT NULL, pack_id TEXT NOT NULL, status pack_install_status NOT NULL DEFAULT 'pending', installed_at TIMESTAMPTZ NOT NULL DEFAULT now(), activated_at TIMESTAMPTZ, soft_uninstalled_at TIMESTAMPTZ, hard_uninstalled_at TIMESTAMPTZ, current_period_start TIMESTAMPTZ, current_period_end TIMESTAMPTZ, billing_currency billing_currency_enum NOT NULL, last_charge_amount_minor BIGINT, last_charge_at TIMESTAMPTZ, trace_id CHAR(32))`. Partial unique `(tenant_id, pack_id) WHERE status NOT IN ('uninstalled','hard_uninstalled')`.

3. **MUST** define `vertical_pack_price_catalog` at migration `0024`: `(pack_id TEXT NOT NULL, currency billing_currency_enum NOT NULL, min_tier plan_tier NOT NULL, monthly_price_minor BIGINT NOT NULL CHECK (monthly_price_minor > 0), version INT NOT NULL, effective_from TIMESTAMPTZ NOT NULL DEFAULT now(), PRIMARY KEY (pack_id, currency, version))`. Versioned for price-change grandfathering.

4. **MUST** define `vertical_pack_overrides` at migration `0025`: `(id BIGSERIAL PRIMARY KEY, tenant_id UUID NOT NULL, pack_id TEXT NOT NULL, currency billing_currency_enum NOT NULL, monthly_price_minor BIGINT NOT NULL, justification TEXT NOT NULL, set_by_subject_id UUID NOT NULL, set_at TIMESTAMPTZ NOT NULL DEFAULT now(), expires_at TIMESTAMPTZ)`. Partial unique `(tenant_id, pack_id, currency) WHERE expires_at IS NULL OR expires_at > now()`.

5. **MUST** enforce RLS on all 3 tables scoped to tenant_id; `vertical_pack_overrides` requires `cfo` role to insert.

6. **MUST** expose `POST /v1/admin/tenants/{tid}/packs/install` body `{ pack_id }`. Handler:
   - Validates pack exists in TASK-SKILL-107 registry.
   - Tier-gate check per §1 #7.
   - 50-pack-limit check per DEC-1307.
   - Duplicate check (partial unique).
   - Resolve price per §1 #8.
   - Prorate invoice via TASK-TEN-003 (stripe-rail) or TASK-TEN-102 (vnd-rail).
   - INSERT install row with status='pending' → transition to 'active' on billing confirmation.
   - Emit `ten.pack_installed`.

7. **MUST** tier-gate per DEC-1306. Lookup pack's `min_tier`; compare with `tenants.plan_tier`. If insufficient → 403 + `tier_upgrade_required` + emit `ten.pack_tier_blocked` sev-2.

8. **MUST** resolve price per `price_resolver.rs::resolve(tenant, pack, currency)`:
   - Check `vertical_pack_overrides` first (per-tenant, currency-matched, not expired) → use override price.
   - Else lookup `vertical_pack_price_catalog` latest version effective at `tenants.installed_at` per DEC-1309 grandfathering (within 90d use install-time version).
   - Else error `no_price_for_pack_currency_combo`.

9. **MUST** expose `POST /v1/admin/tenants/{tid}/packs/{install_id}/uninstall` per DEC-1305:
   - Transition status='uninstalled', `soft_uninstalled_at=now()`.
   - Skill bundle remains accessible 14 days (TASK-SKILL-107 reads our status).
   - At T+14d hard-uninstall job: `hard_uninstalled_at=now()` + skill bundle deactivated.
   - Stripe: prorate credit at uninstall; VND: skip next monthly charge.
   - Emit `ten.pack_uninstalled`.

10. **MUST** expose `POST /v1/admin/tenants/{tid}/packs/{install_id}/reinstall` within 14d window:
    - Reverses soft-uninstall: status='active' + `soft_uninstalled_at=NULL`.
    - No new billing (period still active).
    - Emit `ten.pack_installed` with `via_reinstall=true`.

11. **MUST** monthly recurring charge at billing-cycle-anchor (consistent with TASK-TEN-003 DEC-788). Per active install, dispatch charge via configured rail. On failure: increment retry; after TASK-TEN-003-style 3 retries → status='suspended_billing_failed' + emit `ten.pack_billing_failed` sev-1.

12. **MUST** support per-tenant override via `POST /v1/admin/tenants/{tid}/packs/{pack_id}/override` (cfo role) body `{ currency, monthly_price_minor, justification, expires_at? }`. Emit `ten.pack_price_overridden` sev-1.

13. **MUST** enforce 50-pack-install limit per DEC-1307. `COUNT(*) WHERE tenant_id=$1 AND status IN ('pending','active') ≥ 50` → 403 + `install_limit_exceeded` (Enterprise tier can request via support).

14. **MUST** grandfather pack pricing per DEC-1309. When catalog version changes:
    - Existing installs continue paying their install-time price for 90 days.
    - At T+90d: next billing uses new price + emit `ten.pack_price_changed` (informational not in 5-core).
    - New installs immediately use new price.

15. **MUST** emit 5 memory audit kinds per DEC-1308:
    - `ten.pack_installed` (sev-2)
    - `ten.pack_uninstalled` (sev-2)
    - `ten.pack_billing_failed` (sev-1)
    - `ten.pack_price_overridden` (sev-1)
    - `ten.pack_tier_blocked` (sev-2)

16. **MUST** PII-scrub: justification_sha256 only in chain; raw in DB.

17. **MUST** thread trace_id end-to-end.

18. **MUST NOT** allow per-seat pack pricing per DEC-1300.

19. **MUST NOT** install pack below min_tier per DEC-1306.

20. **MUST NOT** charge VND tenant via Stripe rail or vice versa (consistent with TASK-TEN-003 DEC-784 + TASK-TEN-102 DEC-973).

---

## §2 — Why this design (rationale)

**Why flat per-pack pricing (§1 #1, DEC-1300)?** Consultancies operate 1-to-many: one partner with the Legal Pack covers 50 clients. Per-seat would multiply the cost by 50× → packs become uneconomic for the core target segment.

**Why 90-day grandfather (§1 #14, DEC-1309)?** Mid-period price hikes feel like bait-and-switch. 90 days = one quarterly review cycle for the tenant to budget the new price.

**Why 50-pack limit (§1 #13, DEC-1307)?** Skill bundle loading at ~200KB per pack × 50 = 10MB per tenant — tolerable. Beyond 50 = bundle bloat + UX confusion (which pack does what?).

**Why 14d soft uninstall (§1 #9, DEC-1305)?** Accidental uninstall is real (admin click error; sales demo cleanup). 14d grace = users can reinstall without re-onboarding skill state.

---

## §3 — API contract

```sql
-- 0023_vertical_pack_installs.sql
CREATE TYPE pack_install_status AS ENUM ('pending','active','uninstalled','suspended_billing_failed','blocked_tier');

CREATE TABLE vertical_pack_installs (
  install_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  pack_id TEXT NOT NULL,
  status pack_install_status NOT NULL DEFAULT 'pending',
  installed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  activated_at TIMESTAMPTZ,
  soft_uninstalled_at TIMESTAMPTZ,
  hard_uninstalled_at TIMESTAMPTZ,
  current_period_start TIMESTAMPTZ,
  current_period_end TIMESTAMPTZ,
  billing_currency billing_currency_enum NOT NULL,
  install_time_price_version INT NOT NULL,
  last_charge_amount_minor BIGINT,
  last_charge_at TIMESTAMPTZ,
  trace_id CHAR(32)
);
CREATE UNIQUE INDEX uniq_active_pack_install
  ON vertical_pack_installs(tenant_id, pack_id)
  WHERE status NOT IN ('uninstalled');
ALTER TABLE vertical_pack_installs ENABLE ROW LEVEL SECURITY;
CREATE POLICY vertical_pack_installs_rls ON vertical_pack_installs
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE DELETE ON vertical_pack_installs FROM cyberos_app;
GRANT UPDATE (status, activated_at, soft_uninstalled_at, hard_uninstalled_at,
              current_period_start, current_period_end, last_charge_amount_minor, last_charge_at)
  ON vertical_pack_installs TO cyberos_app;

-- 0024_vertical_pack_price_catalog.sql
CREATE TABLE vertical_pack_price_catalog (
  pack_id TEXT NOT NULL,
  currency billing_currency_enum NOT NULL,
  min_tier plan_tier NOT NULL,
  monthly_price_minor BIGINT NOT NULL CHECK (monthly_price_minor > 0),
  version INT NOT NULL,
  effective_from TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (pack_id, currency, version)
);
REVOKE UPDATE, DELETE ON vertical_pack_price_catalog FROM cyberos_app;

-- 0025_vertical_pack_overrides.sql
CREATE TABLE vertical_pack_overrides (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  pack_id TEXT NOT NULL,
  currency billing_currency_enum NOT NULL,
  monthly_price_minor BIGINT NOT NULL,
  justification TEXT NOT NULL,
  set_by_subject_id UUID NOT NULL,
  set_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ
);
CREATE UNIQUE INDEX uniq_active_pack_override
  ON vertical_pack_overrides(tenant_id, pack_id, currency)
  WHERE expires_at IS NULL OR expires_at > now();
ALTER TABLE vertical_pack_overrides ENABLE ROW LEVEL SECURITY;
CREATE POLICY vertical_pack_overrides_rls ON vertical_pack_overrides
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON vertical_pack_overrides FROM cyberos_app;
```

Endpoints:
```text
POST   /v1/admin/tenants/{tid}/packs/install
POST   /v1/admin/tenants/{tid}/packs/{install_id}/uninstall
POST   /v1/admin/tenants/{tid}/packs/{install_id}/reinstall
POST   /v1/admin/tenants/{tid}/packs/{pack_id}/override     (cfo)
GET    /v1/admin/tenants/{tid}/packs
GET    /v1/admin/packs/catalog                              (public-read)
```

---

## §4 — Acceptance criteria

1. **pack_install_status cardinality 5**.
2. **Install creates row + prorated invoice** — pack installed mid-period charges prorated amount.
3. **Tier gate** — install below min_tier → 403 + `tier_upgrade_required` + audit.
4. **50-pack limit** — 51st install → 403 + `install_limit_exceeded`.
5. **Per-tenant override** — cfo override → resolver uses override price.
6. **Override expires** — past `expires_at` → resolver falls back to catalog.
7. **Soft uninstall 14d** — uninstalled pack still in skill registry until T+14d.
8. **Hard uninstall T+14d** — job removes pack from registry; skill bundle deactivated.
9. **Reinstall within 14d** — status returns 'active' without re-billing.
10. **Billing failure 3 retries** — third failure → status='suspended_billing_failed' + sev-1 audit.
11. **Grandfathered pricing** — pack price changed; existing installs charged old price for 90d.
12. **New install uses new price** — after catalog version bump, new install charged new price immediately.
13. **VND tenant uses TASK-TEN-102 rail** — pack billing routes through VND PSPs.
14. **USD tenant uses Stripe rail**.
15. **Cross-rail rejected** — VND tenant cannot install pack priced only in USD → 400.
16. **Per-seat attempt rejected** — API endpoint has no seat_count field; pricing is flat.
17. **5 memory audit kinds emitted** — full lifecycle.
18. **Cfo-only override** — non-cfo override attempt → 403.
19. **RLS isolation** — tenant A's installs invisible to tenant B.
20. **PII scrub** — justification_sha256 in chain only.

---

## §5 — Verification

```rust
#[tokio::test]
async fn install_charges_prorated() {
    let ctx = TestContext::with_stripe_tenant("acme", PlanTier::Team).await;
    ctx.seed_pack_catalog("legal-pack", BillingCurrency::Usd, PlanTier::Team, 9900).await;
    ctx.travel_to_mid_period().await;
    let r = ctx.install_pack(ctx.tenant_id, "legal-pack").await;
    assert_eq!(r.status(), 201);
    let charge: i64 = ctx.last_stripe_invoice_amount().await;
    assert!(charge < 9900 && charge > 4000); // prorated half-month
}

#[tokio::test]
async fn tier_gate_blocks_starter() {
    let ctx = TestContext::with_stripe_tenant("starter-co", PlanTier::Starter).await;
    ctx.seed_pack_catalog("enterprise-pack", BillingCurrency::Usd, PlanTier::Enterprise, 99900).await;
    let r = ctx.install_pack(ctx.tenant_id, "enterprise-pack").await;
    assert_eq!(r.status(), 403);
    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "ten.pack_tier_blocked"));
}

#[tokio::test]
async fn soft_uninstall_then_reinstall() {
    let ctx = TestContext::new().await;
    let install_id = ctx.install_pack_for_test().await;
    ctx.uninstall_pack(install_id).await;
    assert_eq!(ctx.load_status(install_id).await, "uninstalled");
    let r = ctx.reinstall_pack(install_id).await;
    assert_eq!(r.status(), 200);
    assert_eq!(ctx.load_status(install_id).await, "active");
}

#[tokio::test]
async fn grandfathered_for_90d() {
    let ctx = TestContext::new().await;
    ctx.seed_pack_catalog_v1("legal-pack", BillingCurrency::Usd, PlanTier::Team, 9900).await;
    let install_id = ctx.install_pack_for_test().await;
    ctx.bump_catalog_version("legal-pack", 14900).await;  // price up 50%

    ctx.travel_clock_forward(Duration::from_days(60)).await;
    let charge1 = ctx.run_monthly_billing(install_id).await;
    assert_eq!(charge1, 9900);  // grandfathered

    ctx.travel_clock_forward(Duration::from_days(40)).await;  // now T+100d
    let charge2 = ctx.run_monthly_billing(install_id).await;
    assert_eq!(charge2, 14900);  // new price
}

// 5.5 cfo-only override
// 5.6 50-pack limit
// 5.7 billing failure suspends
// 5.8 cross-rail rejection
// 5.9 cardinality enum
// 5.10 audit emission
```

---

## §7 — Dependencies

**Upstream:** TASK-TEN-002 (plan tiers — min_tier semantics), TASK-SKILL-107 (pack registry — pack_id source).
**Cross-module:** TASK-TEN-003 (stripe rail), TASK-TEN-102 (vnd rail), TASK-TEN-004 (metering integration), TASK-INV-001 (invoice line items), TASK-AUTH-101 (cfo role), TASK-AI-003, TASK-MEMORY-111.
**Downstream:** None.

---

## §8 — Example payload

`ten.pack_installed`:
```json
{
  "kind": "ten.pack_installed",
  "severity": 2,
  "tenant_id": "8a2f...",
  "actor_id": "user.tenant_admin.456",
  "trace_id": "...",
  "occurred_at": "2026-05-17T...",
  "payload": {
    "install_id": "0190...",
    "pack_id": "cyberos.packs.legal",
    "billing_currency": "USD",
    "monthly_price_minor": 9900,
    "prorated_charge_minor": 4523,
    "via_reinstall": false,
    "price_version": 3
  }
}
```

---

## §9 — Open questions

Deferred:
- **Deferred:** Annual pack pricing — slice 3.
- **Deferred:** Pack bundles ("Legal + Compliance + Contracts" discount) — slice 3.
- **Deferred:** Free-trial periods on packs — slice 3.
- **Deferred:** Pack revenue-share with external publishers (slice 4 marketplace).

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Pack not in registry | TASK-SKILL-107 lookup | 404 | Caller fixes pack_id |
| Tier insufficient | check | 403 + tier_blocked audit | Caller upgrades plan |
| 50-pack limit hit | count | 403 + enterprise_override_required | Enterprise tenant contacts sales |
| Duplicate install | partial unique | 409 + already_installed | Inherent |
| Billing failure | rail returns error | Retries; status=suspended_billing_failed after 3 | CFO investigates |
| Override expires mid-period | resolver fallback | Next billing uses catalog price | Inherent |
| Cross-rail attempt | rail guard | 400 + wrong_billing_rail | Caller uses correct rail |
| Pack catalog price missing for currency | resolver miss | 400 + no_price_for_combo | Publisher seeds price |
| Hard-uninstall job stuck | watchdog | sev-2 alert | Manual cleanup |
| Override above plan-cap | validation | 400 + override_invalid | CFO uses sane price |
| Soft-uninstall reinstall after 14d | window check | 410 + window_expired; new install required | Fresh install |
| Catalog version effective_from in future | filter | Version not applied yet | Inherent |
| Grandfather window race (T+89d vs T+90d) | clock-based | Inherent boundary; alert if billing event near boundary | Inherent |
| Cfo override without justification | validation | 400 + justification_required | Inherent |
| Pack uninstalled while billing in flight | tx isolation | Last writer wins; pro-rate credit calculated | Inherent |
| Currency change attempt on existing install | billing_currency immutable per TASK-TEN-003 | Schema rejection | Inherent |

---

## §11 — Implementation notes

**§11.1** Pack install creates a row in `vertical_pack_installs` AND notifies TASK-SKILL-107 to activate the bundle.

**§11.2** Per-currency price catalog typically has 4-5 entries per pack (USD/EUR/SGD/GBP/VND).

**§11.3** Override expires_at supports time-bound discounts ("50% off for 6 months").

**§11.4** Billing-cycle anchor for pack matches tenant's main subscription anchor.

**§11.5** Hard-uninstall job runs hourly; sweeps `soft_uninstalled_at < now() - 14d`.

**§11.6** Grandfather window per-install: `install_time_price_version` column locks the version for 90d.

**§11.7** Cross-rail check uses TASK-TEN-003 + TASK-TEN-102 guards directly.

**§11.8** Per-pack revenue tracked separately for publisher revenue-share (slice 4).

**§11.9** Skill registry consumes install status via API; TASK-SKILL-107 derived.

**§11.10** Override CHECK constraint: cannot exceed 2× catalog price (defensive — typos shouldn't 10× the bill).

---

*End of TASK-TEN-005 spec.*
