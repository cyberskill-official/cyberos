---
id: TASK-TEN-002
title: "3 plan tiers (Starter / Team / Enterprise) hardcoded with per-tier caps"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: TEN
priority: p0
status: ready_to_implement
new_files:
  - docs/tasks/ten/PLAN_CAPS.md
accepted_at: 2026-05-16
accepted_by: Stephen Cheng
verify: T
phase: P2
milestone: P2 · billing-substrate
slice: 1
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-TEN-001, TASK-TEN-003, TASK-TEN-004, TASK-TEN-005]
depends_on: [TASK-TEN-001]
blocks: [TASK-TEN-003, TASK-TEN-005, TASK-TEN-101]

source_pages:
  - website/docs/modules/ten.html#plans
source_decisions:
  - DEC-770 2026-05-16 — Exactly 3 tiers: Starter, Team, Enterprise — no Free, no Trial-as-plan
  - DEC-771 2026-05-16 — Closed `plan_tier` Postgres enum; CI cardinality test asserts 3
  - DEC-772 2026-05-16 — Plan caps hardcoded as compile-time Rust constants — not DB-mutable
  - DEC-773 2026-05-16 — Mid-period upgrade prorates; mid-period downgrade defers to next period boundary
  - DEC-774 2026-05-16 — Downgrade requires confirmation (cap violation if current usage exceeds target tier)
  - DEC-775 2026-05-16 — Plan change is a single atomic operation — both new tier + effective_at written in one COMMIT
  - DEC-776 2026-05-16 — Tier history is append-only via `tenant_plan_history` table (no UPDATE to tenants.plan_tier without history row)
  - DEC-777 2026-05-16 — Founder (CyberSkill operators) hardcoded Enterprise — cannot be downgraded via API
  - DEC-778 2026-05-16 — Plan-tier seat caps: Starter=3, Team=25, Enterprise=∞ (NULL meaning unlimited)
  - DEC-779 2026-05-16 — Plan-tier api_call caps: Starter=10k/mo, Team=500k/mo, Enterprise=∞
  - DEC-780 2026-05-16 — Plan-tier ai_token caps: Starter=100k/mo, Team=5M/mo, Enterprise=50M/mo (Enterprise still has cap; unlimited tokens too expensive)
  - DEC-781 2026-05-16 — Plan-tier storage caps: Starter=1GiB, Team=100GiB, Enterprise=1TiB (then per-GB billing)
  - DEC-782 2026-05-16 — Per-tenant override allowed only by CyberSkill founder role; sev-2 audit
  - DEC-783 2026-05-16 — Plan-change handler issues a TASK-TEN-004 metering event tagged with `plan_change` for audit linkage

build_envelope:
  language: rust 1.81
  service: cyberos/services/ten/
  new_files:
    - services/ten/src/plans/mod.rs
    - services/ten/src/plans/tiers.rs
    - services/ten/src/plans/caps.rs
    - services/ten/src/handlers/plan_change.rs
    - services/ten/src/handlers/plan_show.rs
    - services/ten/migrations/0004_plan_tier.sql
    - services/ten/migrations/0005_plan_history.sql
    - services/ten/tests/plan_change_test.rs
    - services/ten/tests/plan_downgrade_violation_test.rs
    - services/ten/tests/plan_founder_test.rs
  modified_files:
    - services/ten/src/handlers/tenant_create.rs (default plan_tier=Starter)
    - services/metering/src/policy.rs (consume plan-tier caps when no per-tenant override)
  allowed_tools:
    - file_read: services/ten/**
    - file_write: services/ten/{src,tests,migrations}/**
    - bash: cargo test -p cyberos-ten
  disallowed_tools:
    - hardcode plan caps in any other crate (single source of truth = services/ten/src/plans/caps.rs)
    - UPDATE tenants.plan_tier without a tenant_plan_history INSERT in same TX

effort_hours: 4
subtasks:
  - "0.5h: tier_enum migration + plan_tier column on tenants"
  - "0.5h: plans/caps.rs compile-time constants with const-fn cap getters"
  - "0.5h: tenant_plan_history append-only migration"
  - "0.5h: plan_change handler (proration + downgrade-defer logic)"
  - "0.5h: downgrade-violation check (current usage vs target caps)"
  - "0.5h: founder-only override path + sev-2 audit"
  - "0.5h: metering policy consumer (plan-tier caps default; per-tenant override stronger)"
  - "0.5h: integration tests (upgrade + downgrade + founder + violation)"
risk_if_skipped: "Without explicit plan tiers, every tenant gets the same caps; either too generous (we leak money) or too restrictive (we lose customers). Plan tiers are the primary commercial differentiator and the substrate for TASK-TEN-003 Stripe billing."
---

## §1 — Description (BCP-14 normative)

The TEN service **MUST** define exactly three hardcoded plan tiers — Starter, Team, Enterprise — with per-tier caps on the four TASK-TEN-004 metering axes, expose a plan-change API, and persist tier transitions as append-only history.

1. **MUST** define the closed 3-value Postgres enum `plan_tier = ('starter', 'team', 'enterprise')` with a CI cardinality test asserting exactly 3 (DEC-771). Adding a fourth tier requires a schema migration + DEC entry.

2. **MUST** maintain the per-tier caps as compile-time Rust constants in `services/ten/src/plans/caps.rs` (DEC-772). The constants:

| tier | seats | api_calls/mo | ai_tokens/mo | storage |
|---|---|---|---|---|
| starter | 3 | 10_000 | 100_000 | 1 GiB |
| team | 25 | 500_000 | 5_000_000 | 100 GiB |
| enterprise | NULL (∞) | NULL (∞) | 50_000_000 | 1 TiB |

   NULL means unlimited for that axis. Enterprise has a finite ai_tokens cap because tokens map to provider pass-through costs and "unlimited" would expose us to cost explosions (DEC-780). Storage above 1 TiB on Enterprise is billed per-GB (out of scope here; see TASK-TEN-003).

3. **MUST** default new tenants to `starter` at TASK-TEN-001 provisioning (DEC-783 ref). The default is wired in `services/ten/src/handlers/tenant_create.rs` — no other code path may write `plan_tier` at insert.

4. **MUST** expose `POST /v1/admin/tenants/{id}/plan` to change a tenant's plan. The handler requires:
   - The caller's role is `tenant_admin` (per TASK-AUTH-101) for upgrade/downgrade of their own tenant, OR `cyberskill_founder` for any tenant.
   - The body `{ "target_tier": "team", "effective": "immediate" | "next_period" }` is valid.
   - For downgrades, the handler checks current period usage against target-tier caps (DEC-774). If usage exceeds target caps on any axis, the handler returns `409 CONFLICT` with `{error: "downgrade_violation", axis, current, target_cap}` and refuses unless the body includes `acknowledge_data_loss: true` AND the target action implies a permitted resolution (e.g., the tenant has deactivated seats to fit).

5. **MUST** treat upgrades as immediate (proration applies; see DEC-773). Downgrades default to `effective: "next_period"` (deferred to billing-period boundary) per DEC-773. Setting `effective: "immediate"` on a downgrade is allowed but emits a sev-2 audit row noting the unusual choice.

6. **MUST** write a `tenant_plan_history` row in the same Postgres transaction that UPDATEs `tenants.plan_tier` (DEC-775 + DEC-776). The history row captures `(tenant_id, from_tier, to_tier, actor_id, occurred_at, effective_at, proration_amount_cents, reason)`. The `tenant_plan_history` table is append-only via SQL grant (REVOKE UPDATE, DELETE FROM cyberos_app). A trigger on `tenants` rejects UPDATEs to `plan_tier` not bracketed by a same-TX `tenant_plan_history` INSERT.

7. **MUST** define a closed 3-value `plan_change_effective` Postgres enum (`immediate`, `next_period`, `defer_billing_only`). CI cardinality test asserts 3.

8. **MUST** prevent any non-founder API path from changing the plan of a tenant flagged `is_founder_tenant = true` (DEC-777). The handler short-circuits with `403 FORBIDDEN` + `{error: "founder_tenant_plan_immutable"}`. The founder tenant is the CyberSkill operator's own tenant — hardcoded to Enterprise + cannot be moved.

9. **MUST** consume the per-tier caps as the metering default when `tenants.metering_caps_yaml IS NULL`. The metering policy resolver order (DEC-781 ref): (1) per-tenant explicit `metering_caps_yaml` (founder-set), (2) plan-tier caps (this task), (3) platform absolute maximums (TASK-TEN-004 §11.10). Per-tenant overrides are stronger than plan caps; plan caps are stronger than platform defaults.

10. **MUST** emit one memory audit row per plan change at sev-2 with kind `ten.plan_changed`. The row carries `(tenant_id, actor_id, from_tier, to_tier, effective_at, proration_amount_cents)`. The reason field is scrubbed via TASK-MEMORY-111 before chain emission.

11. **MUST** compute proration on upgrade as `((target_tier_price - current_tier_price) * days_remaining_in_period) / days_in_period`, integer math in cents (no floating-point). The result is positive for upgrade (tenant owes prorated diff). The proration handler is invoked by TASK-TEN-003 Stripe billing; this task emits the proration_amount_cents in the history row but does NOT mutate Stripe state.

12. **MUST** snapshot the `from_tier` caps at the moment of change into the history row so future audits can reconstruct what the tenant had access to. The snapshot is the JSONB `from_tier_caps_snapshot` column. This avoids "the tier caps changed, now history rows are ambiguous" — even if we add a fourth tier later, old history rows show the caps as they were.

13. **MUST** validate `target_tier` is one of the 3 enum values; invalid value returns `400 BAD_REQUEST`. Same tier as current returns `409 CONFLICT { error: "no_change" }` (no audit row).

14. **MUST** rate-limit plan changes to at most 1 per tenant per 24h (DEC-773 implies this — flip-flop prevention). Second change within 24h returns `429 TOO_MANY_REQUESTS` with `{error: "plan_change_rate_limited", next_allowed_at}`. The founder-override path bypasses this rate limit (founder operator may need to make multiple corrections).

15. **MUST** record the plan change in the TASK-TEN-004 metering event stream with a synthetic event of axis-independent kind `plan_change` carrying `idempotency_key = "plan_change_<history_id>"` (DEC-783). This enables billing-side reconciliation between plan history and metering events.

16. **MUST** expose `GET /v1/tenants/{id}/plan` returning the current tier + per-axis caps + effective_since. The response:
    ```json
    {
      "tenant_id": "ten_abc",
      "tier": "team",
      "effective_since": "2026-05-01T00:00:00Z",
      "caps": {
        "seats": 25,
        "api_calls_per_month": 500000,
        "ai_tokens_per_month": 5000000,
        "storage_bytes": 107374182400
      },
      "next_scheduled_change": null
    }
    ```

17. **MUST** expose `GET /v1/tenants/{id}/plan/history` returning paginated history rows for the tenant. Role gate: `tenant_admin` for self, `cyberskill_founder` for any.

18. **MUST** maintain a `tenants.next_scheduled_change` nullable column for deferred downgrades. A row is set when a downgrade is requested with `effective: "next_period"`; the period-close job (TASK-TEN-004 #22) applies the change as part of period freeze. After application, the column is cleared.

19. **MUST** reject the plan change if `tenants.next_scheduled_change IS NOT NULL` AND the new request would create a second deferred change. Returns `409 CONFLICT { error: "deferred_change_already_pending", scheduled: <ts>, requested_new: <body> }`. The handler also exposes `DELETE /v1/admin/tenants/{id}/plan/scheduled` to cancel a pending change (tenant_admin or founder).

20. **MUST** keep all of this PII-free at the chain layer. The history row's `reason` text passes through TASK-MEMORY-111; coordinates, names, emails are not in this domain (it's plan tiers).

21. **MUST** validate plan-change reason length [10, 1000] characters when present (optional field; required for downgrades and for founder overrides). Empty + downgrade returns `400 reason_required_for_downgrade`. Empty + founder-override returns `400 reason_required_for_founder_override`.

22. **MUST** expose a dry-run mode: `POST /v1/admin/tenants/{id}/plan?dry_run=true` returns the would-apply preview (proration cents, current vs target caps diff, downgrade-violation detection) without persisting and without emitting audit rows. Dry-run does NOT consume the 24h rate-limit slot.

23. **MUST** wire the founder-override path through a distinct handler (`POST /v1/admin/founder/tenants/{id}/plan/override`) that requires the `cyberskill_founder` role explicitly and emits a sev-2 audit row with kind `ten.plan_founder_override`. This separation prevents accidental founder-grade changes from the regular plan handler path.

24. **MUST** support per-tenant `is_founder_tenant` boolean (DEC-777) seeded TRUE at provisioning time for the CyberSkill operator's tenant (via TASK-TEN-001 founder flag). A trigger rejects mutation of `is_founder_tenant` after insert (one-way set at creation).

25. **MUST** emit 4 closed memory audit kinds:
    - `ten.plan_changed` (sev-2, every plan transition)
    - `ten.plan_founder_override` (sev-2, founder-only override path)
    - `ten.plan_change_rejected_violation` (sev-2, downgrade violation)
    - `ten.plan_change_rejected_rate_limit` (sev-3, 24h rate limit)

---

## §2 — Rationale (informative — preserve all 22 paragraphs)

**§2.1  Why exactly 3 tiers.** DEC-770 + clause #1. More tiers create choice paralysis at signup ("what's the difference between Standard and Plus?"); fewer leave commercial holes (Starter is too small for some, Team is too small for others). 3 tiers is the standard SaaS shape — small/medium/large — and it covers the buyer-persona spectrum without overwhelming the signup flow.

**§2.2  Why hardcoded constants and not DB rows.** DEC-772 + clause #2. Plan caps are part of the product contract. A DB-mutable "caps" table would let an operator silently change a tenant's plan caps (compliance + customer-trust violation). Hardcoded constants mean cap changes require a code release + a coordinated customer notification. The cost is that adding a new tier or changing caps requires a deploy; the benefit is that "what's my cap?" has a single answer reachable by reading source.

**§2.3  Why no Free tier.** DEC-770 implicit. Free tiers attract abuse (spam, mass account creation) without producing revenue. Our model is paid-from-day-1; trial periods are time-limited per-tenant flags (out of scope for this task) but not first-class tiers.

**§2.4  Why proration on upgrade but defer-to-next-period on downgrade.** DEC-773. Upgrade is a positive cashflow event for us and the tenant gets immediate access to the upgraded caps; proration matches the cash to the access. Downgrade is a negative cashflow event for us — deferring to next-period keeps the tenant on the current tier for the rest of the paid-for period (no refund accounting) and gives them time to "pull back" before the downgrade hits. The override (immediate downgrade with audit) exists for true exit cases.

**§2.5  Why founder-tenant hardcoded.** DEC-777 + clause #8 + #24. The CyberSkill operator's own tenant runs the platform — it cannot suddenly be downgraded by a UI accident or a bug in the plan-change handler. The is_founder_tenant boolean + the API short-circuit + the immutability trigger combine to make it accident-proof.

**§2.6  Why append-only history.** DEC-776 + clause #6. Plan history is the foundation for billing reconciliation, audit responses to "what did I have on date X?", and compliance answers. An UPDATE-able plan history could be silently rewritten; an append-only one preserves the truth. The trigger that requires a same-TX INSERT to history makes "UPDATE tenants without writing history" structurally impossible.

**§2.7  Why snapshot from_tier_caps in history.** Clause #12. If we add a fourth tier or change caps for an existing tier (rare but possible), historical rows that reference the old tier need to still answer "what did this tenant have access to on that date?". Snapshot in JSONB lets us reconstruct without keeping ancient tier definitions in code.

**§2.8  Why per-axis caps and not a single "plan limit".** Customers buy AI tokens vs API calls vs seats at different rates. A single combined limit (e.g., "credits") obscures which axis is the cost driver and makes upgrade discussions vague. Per-axis caps make the upsell conversation concrete: "you're at seat-cap; the next tier doubles seats".

**§2.9  Why Enterprise has finite ai_tokens cap.** DEC-780. Tokens map directly to provider pass-through cost (OpenAI, Anthropic, etc.). An "unlimited" Enterprise tenant who plugs in a bot that burns 1B tokens/day would lose us a lot of money. 50M tokens/month is generous (most Enterprise tenants stay well below) and the per-tenant override path lets us lift the cap for individual customers via contract.

**§2.10  Why per-tenant override stronger than plan caps.** Clause #9. Sales sometimes needs to negotiate a Team customer with a 50-seat cap before they upgrade to Enterprise. The per-tenant override (founder-only) lets us write that bespoke contract without violating the plan-cap model. The audit row makes every override visible.

**§2.11  Why 24h rate limit on plan changes.** Clause #14. Without the limit, a tenant could flip between Starter and Team every minute, generating proration accounting noise and burning audit chain bandwidth. 24h is long enough that legitimate "I made a mistake" reversals can happen on the founder path; the regular path is rate-limited.

**§2.12  Why deferred downgrades go through period-close.** Clause #18 + TASK-TEN-004 #22. The metering period-close handler is already the moment of accounting truth (seats + storage snapshot). Applying the plan change at that same boundary keeps all billing-relevant changes atomic — the tenant's bill for the new period uses the new tier.

**§2.13  Why downgrade-violation check is current-period not month-to-date snapshot.** Clause #4 + DEC-774. The relevant question for a downgrade is "will this tenant exceed the target tier caps right now?". Snapshot of MTD usage answers that. Future projection (e.g., extrapolating to month-end) would add complexity without changing the answer for most cases. We accept the slight inaccuracy of "tenant is at 24 seats today and would be 26 by month-end" — the period-close enforcement catches it.

**§2.14  Why a separate handler for founder override.** Clause #23. The regular plan-change handler has rate limits + violation checks + caller-role checks; the founder-override handler bypasses some of those (it can change founder tenants, exceed rate limits, etc.). Mixing the two would require complex if/else inside one handler; a separate handler keeps the regular path's invariants pure.

**§2.15  Why we wire plan-change into the metering event stream.** Clause #15 + DEC-783. Billing reconciliation needs to know "the tenant changed from Team to Enterprise on day 15 of the period; usage from day 15-30 should bill at Enterprise rates". The metering event tagged `plan_change` is the marker. Without it, billing would have to join `tenant_plan_history` to every metering event — expensive.

**§2.16  Why reasons are required for downgrades and founder overrides.** Clause #21. Both actions have higher-impact consequences (downgrade may reduce service; founder override is the most-privileged path). Forcing a written reason creates accountability and helps post-hoc investigation. Upgrades don't require a reason (the tenant chose to spend more; the reason is implicit).

**§2.17  Why dry-run doesn't consume the rate-limit slot.** Clause #22. Operators want to preview changes before committing — making dry-run consume a slot would prevent the live change from happening. The dry-run is purely read-side.

**§2.18  Why next_scheduled_change is on the tenants table not history.** Clause #18. The pending change is a current-state property of the tenant ("you're scheduled to downgrade at next period"). It belongs on the tenants row for fast lookup. The history table records the schedule + the eventual execution as two rows; the tenants row is the pointer to the upcoming.

**§2.19  Why we reject second deferred changes.** Clause #19. Allowing a stack of pending changes (downgrade to Team at period N, then upgrade back to Enterprise at period N+1) would confuse both the tenant and the billing system. One pending change at a time; cancel via DELETE before scheduling another.

**§2.20  Why audit kinds are 4 not 1.** Clause #25. Each kind reflects a different operational signal: normal change, founder override, downgrade violation, rate limit. Operators dashboard-query on kind; collapsing them would force free-text parsing.

**§2.21  Why we don't ship a plan-comparison UI as part of this task.** Out of scope. The marketing site + the tenant-admin SPA (TASK-TEN-107) own the UI; this task is the data + API substrate.

**§2.22  Why proration math is integer cents.** Clause #11. Floating-point in money is forbidden — accumulated rounding errors create invoice mysteries. Integer cents (i64 max = ~$92 quadrillion, sufficient for billing) is the correct primitive. The proration formula is integer division with floor semantics; the rounding goes in the tenant's favor (we under-bill by up to 1 cent per change).

---

## §3 — API & schema

### §3.1 — Migration 0004: plan_tier enum + tenants column

```sql
-- services/ten/migrations/0004_plan_tier.sql

CREATE TYPE plan_tier AS ENUM ('starter', 'team', 'enterprise');
CREATE TYPE plan_change_effective AS ENUM ('immediate', 'next_period', 'defer_billing_only');

ALTER TABLE tenants
    ADD COLUMN IF NOT EXISTS plan_tier plan_tier NOT NULL DEFAULT 'starter',
    ADD COLUMN IF NOT EXISTS plan_effective_since TIMESTAMPTZ NOT NULL DEFAULT now(),
    ADD COLUMN IF NOT EXISTS is_founder_tenant BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS next_scheduled_change JSONB;  -- {target_tier, effective_at, history_id}

-- One-way set: is_founder_tenant cannot flip after first insert
CREATE OR REPLACE FUNCTION reject_founder_flip() RETURNS TRIGGER AS $$
BEGIN
    IF OLD.is_founder_tenant IS DISTINCT FROM NEW.is_founder_tenant THEN
        RAISE EXCEPTION 'is_founder_tenant_immutable' USING ERRCODE = 'P0300';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER founder_flip_trg BEFORE UPDATE ON tenants
    FOR EACH ROW EXECUTE FUNCTION reject_founder_flip();
```

### §3.2 — Migration 0005: tenant_plan_history

```sql
-- services/ten/migrations/0005_plan_history.sql

CREATE TABLE tenant_plan_history (
    id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id                 UUID NOT NULL REFERENCES tenants(id),
    actor_id                  UUID NOT NULL REFERENCES subjects(id),
    occurred_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    effective_at              TIMESTAMPTZ NOT NULL,
    from_tier                 plan_tier,        -- NULL for first record
    to_tier                   plan_tier NOT NULL,
    effective_kind            plan_change_effective NOT NULL,
    proration_amount_cents    BIGINT NOT NULL DEFAULT 0,
    from_tier_caps_snapshot   JSONB NOT NULL,
    to_tier_caps_snapshot     JSONB NOT NULL,
    reason                    TEXT CHECK (reason IS NULL OR (length(reason) BETWEEN 10 AND 1000)),
    memory_chain_hash          CHAR(64) NOT NULL CHECK (memory_chain_hash ~ '^[0-9a-f]{64}$')
);

CREATE INDEX plan_history_tenant_time ON tenant_plan_history (tenant_id, occurred_at DESC);

REVOKE UPDATE, DELETE ON tenant_plan_history FROM cyberos_app;
GRANT INSERT, SELECT ON tenant_plan_history TO ten_writer;
GRANT SELECT ON tenant_plan_history TO security_admin, cyberskill_founder;

ALTER TABLE tenant_plan_history ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON tenant_plan_history
    USING (tenant_id = current_setting('cyberos.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('cyberos.tenant_id')::uuid);

-- Trigger: UPDATE on tenants.plan_tier requires a same-TX history INSERT
CREATE OR REPLACE FUNCTION require_plan_history() RETURNS TRIGGER AS $$
DECLARE rows_in_tx INTEGER;
BEGIN
    IF OLD.plan_tier IS DISTINCT FROM NEW.plan_tier THEN
        SELECT COUNT(*) INTO rows_in_tx FROM tenant_plan_history
            WHERE tenant_id = NEW.id AND occurred_at >= now() - INTERVAL '1 second';
        IF rows_in_tx = 0 THEN
            RAISE EXCEPTION 'plan_tier_update_requires_history_row' USING ERRCODE = 'P0301';
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER plan_history_trg BEFORE UPDATE ON tenants
    FOR EACH ROW EXECUTE FUNCTION require_plan_history();
```

### §3.3 — Caps constants

```rust
// services/ten/src/plans/caps.rs

#[derive(Debug, Clone, Copy)]
pub struct TierCaps {
    pub seats: Option<i64>,                  // None = unlimited
    pub api_calls_per_month: Option<i64>,
    pub ai_tokens_per_month: Option<i64>,
    pub storage_bytes: Option<i64>,
}

pub const STARTER: TierCaps = TierCaps {
    seats: Some(3),
    api_calls_per_month: Some(10_000),
    ai_tokens_per_month: Some(100_000),
    storage_bytes: Some(1 * 1024 * 1024 * 1024),  // 1 GiB
};

pub const TEAM: TierCaps = TierCaps {
    seats: Some(25),
    api_calls_per_month: Some(500_000),
    ai_tokens_per_month: Some(5_000_000),
    storage_bytes: Some(100 * 1024 * 1024 * 1024),  // 100 GiB
};

pub const ENTERPRISE: TierCaps = TierCaps {
    seats: None,                                       // unlimited
    api_calls_per_month: None,                         // unlimited
    ai_tokens_per_month: Some(50_000_000),             // capped — provider pass-through
    storage_bytes: Some(1024 * 1024 * 1024 * 1024),    // 1 TiB
};

pub fn caps_for(tier: PlanTier) -> &'static TierCaps {
    match tier {
        PlanTier::Starter => &STARTER,
        PlanTier::Team => &TEAM,
        PlanTier::Enterprise => &ENTERPRISE,
    }
}

pub const TIER_PRICE_CENTS_MONTHLY: [(PlanTier, i64); 3] = [
    (PlanTier::Starter, 4_900),     // $49
    (PlanTier::Team, 24_900),       // $249
    (PlanTier::Enterprise, 99_900), // $999 base; usage-based on top
];
```

### §3.4 — Plan-change handler

```rust
// services/ten/src/handlers/plan_change.rs

pub async fn change_plan(
    pool: &PgPool,
    actor_id: Uuid,
    tenant_id: Uuid,
    target: PlanTier,
    effective: PlanChangeEffective,
    reason: Option<String>,
    dry_run: bool,
) -> Result<PlanChangeResult, PlanError> {
    let mut tx = pool.begin().await?;

    // Load current state
    let current: TenantPlanRow = sqlx::query_as!(
        TenantPlanRow,
        r#"SELECT plan_tier AS "plan_tier: PlanTier",
                  plan_effective_since,
                  is_founder_tenant,
                  next_scheduled_change
           FROM tenants WHERE id = $1 FOR UPDATE"#,
        tenant_id
    ).fetch_one(&mut *tx).await?;

    if current.is_founder_tenant {
        return Err(PlanError::FounderImmutable);
    }
    if current.plan_tier == target {
        return Err(PlanError::NoChange);
    }
    if current.next_scheduled_change.is_some() {
        return Err(PlanError::DeferredAlreadyPending);
    }

    // Downgrade-violation check
    let is_downgrade = is_downgrade(current.plan_tier, target);
    if is_downgrade {
        let viol = downgrade_violation_check(pool, tenant_id, target).await?;
        if let Some(v) = viol {
            return Err(PlanError::DowngradeViolation(v));
        }
        if reason.is_none() {
            return Err(PlanError::ReasonRequiredForDowngrade);
        }
    }

    // Rate-limit (skip on founder-override path — handled by separate route)
    let last_change_at: Option<DateTime<Utc>> = sqlx::query_scalar!(
        "SELECT MAX(occurred_at) FROM tenant_plan_history WHERE tenant_id = $1",
        tenant_id
    ).fetch_one(&mut *tx).await?;
    if let Some(t) = last_change_at {
        if (Utc::now() - t).num_hours() < 24 {
            return Err(PlanError::RateLimited { next_allowed_at: t + Duration::hours(24) });
        }
    }

    let effective_at = match effective {
        PlanChangeEffective::Immediate => Utc::now(),
        PlanChangeEffective::NextPeriod => period_end(tenant_id, &mut tx).await?,
        PlanChangeEffective::DeferBillingOnly => Utc::now(),
    };
    let proration = compute_proration_cents(current.plan_tier, target, &current.plan_effective_since, &effective_at)?;

    if dry_run {
        tx.rollback().await?;
        return Ok(PlanChangeResult::DryRun {
            from: current.plan_tier, to: target,
            proration_amount_cents: proration,
            effective_at,
        });
    }

    let memory_hash = emit_memory_plan_changed(tenant_id, actor_id, current.plan_tier, target, effective_at, proration, &reason).await?;

    let history_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO tenant_plan_history
              (tenant_id, actor_id, effective_at, from_tier, to_tier,
               effective_kind, proration_amount_cents,
               from_tier_caps_snapshot, to_tier_caps_snapshot, reason, memory_chain_hash)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
           RETURNING id"#,
        tenant_id, actor_id, effective_at,
        current.plan_tier as _, target as _,
        effective as _, proration,
        caps_snapshot_json(current.plan_tier), caps_snapshot_json(target),
        reason, &memory_hash
    ).fetch_one(&mut *tx).await?;

    if effective == PlanChangeEffective::Immediate {
        sqlx::query!(
            "UPDATE tenants SET plan_tier = $2, plan_effective_since = $3 WHERE id = $1",
            tenant_id, target as _, effective_at
        ).execute(&mut *tx).await?;
    } else {
        sqlx::query!(
            r#"UPDATE tenants SET next_scheduled_change = jsonb_build_object(
                'target_tier', $2::text,
                'effective_at', $3::timestamptz,
                'history_id', $4::uuid
            ) WHERE id = $1"#,
            tenant_id, target.to_string(), effective_at, history_id
        ).execute(&mut *tx).await?;
    }

    // §1 #15 metering event linkage
    metering::record_plan_change(pool, tenant_id, history_id).await?;

    tx.commit().await?;
    Ok(PlanChangeResult::Applied { history_id, effective_at, proration_amount_cents: proration })
}
```

---

## §4 — Acceptance criteria

1. `plan_tier` enum exactly 3 values (CI cardinality test).
2. `plan_change_effective` enum exactly 3 values.
3. New tenant default `plan_tier = starter` after TASK-TEN-001 provisioning.
4. `caps_for(Starter)` returns seats=3, api=10k, ai=100k, storage=1GiB.
5. `caps_for(Team)` returns seats=25, api=500k, ai=5M, storage=100GiB.
6. `caps_for(Enterprise)` returns seats=NULL, api=NULL, ai=50M, storage=1TiB.
7. Upgrade Starter→Team applies immediately + proration computed in integer cents.
8. Downgrade Team→Starter defaults to `effective: next_period`; tenants.next_scheduled_change populated.
9. Downgrade with current_seats > target_cap returns `409 downgrade_violation` (without `acknowledge_data_loss`).
10. Downgrade with no reason returns `400 reason_required_for_downgrade`.
11. Same-tier change returns `409 no_change`; no audit row, no history row.
12. Invalid tier value returns `400`.
13. Two changes within 24h: second returns `429 plan_change_rate_limited`.
14. Founder tenant attempt to change via regular handler returns `403 founder_tenant_plan_immutable`.
15. Founder-override handler (separate route) succeeds for founder tenant; bypasses rate limit; sev-2 `ten.plan_founder_override` audit.
16. UPDATE on `tenants.plan_tier` without history INSERT in same TX → P0301 trigger error.
17. UPDATE on `tenants.is_founder_tenant` → P0300 trigger error.
18. `tenant_plan_history` REVOKE UPDATE/DELETE confirmed at `\dp`.
19. RLS prevents cross-tenant SELECT on `tenant_plan_history`.
20. Plan-change memory audit row at sev-2 with kind `ten.plan_changed`.
21. `GET /v1/tenants/{id}/plan` returns current tier + caps + effective_since.
22. `GET /v1/tenants/{id}/plan/history` returns paginated rows scoped to caller's tenant (or any tenant for founder).
23. `next_scheduled_change` is set on deferred downgrade; cleared by period-close job.
24. Second deferred change while one pending returns `409 deferred_change_already_pending`.
25. `DELETE /v1/admin/tenants/{id}/plan/scheduled` clears the pending change.
26. Per-tenant `metering_caps_yaml` override stronger than plan caps (TASK-TEN-004 integration).
27. `dry_run=true` returns preview without DB write, audit row, or rate-limit consumption.
28. Plan change emits a TASK-TEN-004 metering event with kind=plan_change linked by history_id.
29. Proration formula uses integer division (no floating point); rounding favors tenant.
30. from_tier_caps_snapshot JSONB captures the caps at change time for audit-time reconstruction.

---

## §5 — Verification (CI tests)

- `cardinality_test_tier` — 3.
- `cardinality_test_effective` — 3.
- `tier_default_test` — new tenant created → plan_tier='starter'.
- `caps_starter_test` — caps_for(Starter) matches table.
- `caps_team_test`, `caps_enterprise_test` — same.
- `upgrade_immediate_test` — Starter→Team applies + proration > 0.
- `downgrade_defer_test` — Team→Starter sets next_scheduled_change.
- `downgrade_violation_test` — current seats > target → 409.
- `downgrade_reason_required_test` — no reason → 400.
- `same_tier_test` — 409 no_change.
- `invalid_tier_test` — body `target_tier: 'gold'` → 400.
- `rate_limit_test` — two changes in 24h → second 429.
- `founder_immutable_regular_test` — founder tenant via regular API → 403.
- `founder_override_test` — separate handler succeeds + sev-2 audit.
- `plan_history_trigger_test` — UPDATE plan_tier without history INSERT → P0301.
- `founder_flip_trigger_test` — UPDATE is_founder_tenant → P0300.
- `append_only_test` — REVOKE inspection.
- `rls_isolation_test` — two tenants, cross-query empty.
- `metering_caps_override_test` — per-tenant override stronger than plan default.
- `dry_run_test` — no DB write, no audit, no rate-limit slot consumed.
- `scheduled_change_cancel_test` — DELETE clears next_scheduled_change.
- `second_deferred_test` — second deferred → 409.
- `metering_event_link_test` — plan change emits TASK-TEN-004 event with history_id.
- `proration_integer_test` — proration formula returns i64; no floats.
- `from_tier_snapshot_test` — JSONB snapshot present in history row.

---

## §6 — File skeleton

```
services/ten/
├── src/
│   ├── plans/
│   │   ├── mod.rs          # pub re-exports
│   │   ├── tiers.rs        # PlanTier enum + PlanChangeEffective
│   │   ├── caps.rs         # §3.3 constants + caps_for()
│   │   ├── proration.rs    # integer-cents proration math
│   │   └── violation.rs    # downgrade_violation_check
│   └── handlers/
│       ├── tenant_create.rs       # MODIFIED: default plan_tier=Starter
│       ├── plan_show.rs           # GET /v1/tenants/{id}/plan
│       ├── plan_history.rs        # GET /v1/tenants/{id}/plan/history
│       ├── plan_change.rs         # POST /v1/admin/tenants/{id}/plan
│       ├── plan_change_founder.rs # POST /v1/admin/founder/.../plan/override
│       └── plan_scheduled_cancel.rs # DELETE /v1/admin/tenants/{id}/plan/scheduled
├── migrations/
│   ├── 0004_plan_tier.sql
│   └── 0005_plan_history.sql
└── tests/
    ├── plan_change_test.rs
    ├── plan_downgrade_violation_test.rs
    └── plan_founder_test.rs
```

---

## §7 — Dependencies & blast-radius

**Depends on**: TASK-TEN-001 (provisioning CLI + tenants table).

**Blocks**: TASK-TEN-005 (vertical-pack pricing add-on).

**Blast radius if broken**:
- **Wrong default tier**: new tenants get under- or over-provisioned; bounded by manual correction.
- **Cap mismatch with metering**: tenants hit caps unexpectedly; defense-in-depth via per-tenant override.
- **Founder-tenant downgrade**: catastrophic (platform itself loses access); two layers of protection (handler + trigger).
- **History row missing**: forensic gap; trigger prevents.

---

## §8 — Payload examples

### §8.1 — Upgrade

```
POST /v1/admin/tenants/{id}/plan
Authorization: Bearer <tenant_admin>
{ "target_tier": "team", "effective": "immediate" }

200 OK
{
  "applied": true,
  "history_id": "...",
  "effective_at": "2026-05-16T10:30:00Z",
  "proration_amount_cents": 13200
}
```

### §8.2 — Downgrade with violation

```
POST /v1/admin/tenants/{id}/plan
{ "target_tier": "starter", "effective": "next_period", "reason": "Cost reduction Q3" }

409 Conflict
{
  "error": "downgrade_violation",
  "axis": "seats",
  "current": 12,
  "target_cap": 3
}
```

### §8.3 — Get plan

```
GET /v1/tenants/{id}/plan

200 OK
{
  "tenant_id": "ten_abc",
  "tier": "team",
  "effective_since": "2026-05-01T00:00:00Z",
  "caps": { "seats": 25, "api_calls_per_month": 500000, "ai_tokens_per_month": 5000000, "storage_bytes": 107374182400 },
  "next_scheduled_change": null
}
```

### §8.4 — Founder override

```
POST /v1/admin/founder/tenants/{id}/plan/override
Authorization: Bearer <cyberskill_founder>
{ "target_tier": "enterprise", "reason": "Customer contract — 12mo Enterprise floor" }

200 OK
{ "applied": true, "history_id": "..." }
```

---

## §9 — Open questions

- **OQ-1** (closed by DEC-770): 3 tiers — Starter, Team, Enterprise. Confirmed.
- **OQ-2** (closed by DEC-772): hardcoded constants.
- **OQ-3** (open): should mid-year price changes trigger automatic notifications to current tenants? Out of scope; emails are task-EMAIL-* domain. Currently we'd handle via marketing.
- **OQ-4** (open): vertical-pack pricing (TASK-TEN-005) layers add-ons on top of base plan; the interaction model needs TASK-TEN-005 + TASK-TEN-003 to be finalized.

---

## §10 — Failure modes (32 rows)

| # | Failure | Detection | Sev | Handler |
|---|---------|-----------|-----|---------|
| 1 | UPDATE plan_tier without history INSERT | trigger P0301 | 1 | Reject; abort TX |
| 2 | UPDATE is_founder_tenant | trigger P0300 | 1 | Reject |
| 3 | Founder tenant via regular handler | role check | 2 | 403 + sev-2 |
| 4 | Two concurrent plan changes (race) | FOR UPDATE lock | 3 | Second sees updated state; one wins |
| 5 | Rate-limit bypass attempt | last-change query | 3 | 429 + sev-3 |
| 6 | Downgrade with active over-cap usage | downgrade_violation_check | 2 | 409 + sev-2 |
| 7 | No reason on downgrade | length check | 3 | 400 |
| 8 | No reason on founder override | length check | 3 | 400 |
| 9 | Invalid tier in body | enum cast fail | 3 | 400 |
| 10 | Invalid effective_kind | enum cast fail | 3 | 400 |
| 11 | Plan history INSERT permission denied | GRANT misconfig | 1 | Abort TX; sev-1 |
| 12 | memory audit emission fails | subprocess error | 1 | Retry via WAL; if exhausted, sev-1 |
| 13 | Cross-tenant RLS leak | rls_isolation_test | 1 | CI blocks |
| 14 | Proration overflow i64 | overflow_op_panic in debug; saturating in release | 2 | Saturate at i64::MAX; sev-2 audit |
| 15 | next_scheduled_change set while another pending | application check | 3 | 409 |
| 16 | Period-close job fails to apply deferred change | job failure log | 2 | Retry next tick; sev-2 |
| 17 | Cancel pending change with no pending | application check | 3 | 404 |
| 18 | Metering event link fails | recorder error | 2 | History row written; metering retry via WAL |
| 19 | Same target_tier as current | application check | 3 | 409 no_change |
| 20 | dry_run mutates state | dry_run_test | 2 | CI blocks |
| 21 | Caps constant drift between code and table | code review | 2 | CI test compares constants to documented table |
| 22 | Per-tenant override silently ignored | metering_caps_override_test | 1 | CI blocks |
| 23 | from_tier_caps_snapshot missing | NOT NULL constraint | 1 | INSERT rejected |
| 24 | memory_chain_hash regex fails | CHECK | 1 | INSERT rejected |
| 25 | Reason > 1000 chars | CHECK | 3 | 400 |
| 26 | Reason < 10 chars on downgrade | application check | 3 | 400 |
| 27 | Plan change for terminated tenant | RLS + state check | 2 | 404; sev-2 audit |
| 28 | Plan history pagination unbounded | limit clause | 3 | Default LIMIT 50 |
| 29 | Founder tenant flag flipped via raw SQL | trigger | 1 | DB rejects |
| 30 | Caps for newly-added tier missing | match exhaustiveness | 1 | Rust compile error |
| 31 | Period boundary computation drifts on DST change | UTC math | 3 | All periods in UTC |
| 32 | Plan change recorded but session token still holds old role-bundle | session refresh | 3 | Next token refresh picks up new tier |

---

## §11 — Implementation notes

**§11.1** `caps.rs` constants are `const fn`-accessible so other crates can do `const STARTER_SEATS: i64 = caps::STARTER.seats.unwrap_or(i64::MAX);` if needed.

**§11.2** The trigger that requires same-TX history INSERT uses a `now() - INTERVAL '1 second'` window. The 1-second window is generous for "same transaction" — Postgres TX runs within microseconds typically.

**§11.3** Proration formula: `((target_price - current_price) * days_remaining) / days_in_period`, integer math, days computed as `EXTRACT(EPOCH FROM (period_end - now())) / 86400`.

**§11.4** The period_end helper queries the tenant's billing timezone + cycle to compute the next period boundary. Mirrors TASK-TEN-004 #18 pattern.

**§11.5** The downgrade-violation check queries the materialized `metering_current_period` view (TASK-TEN-004 #20) for each axis and compares to target tier's cap.

**§11.6** The `is_downgrade` helper compares tier ordinals: `enterprise > team > starter`.

**§11.7** The founder-override handler is mounted at a separate URL path with stricter middleware (founder role required at request entry, not handler).

**§11.8** Tests use the same testcontainers Postgres pattern as other AUTH/TEN tests. Founder tenant is seeded via fixture; rate-limit test uses time-mocking.

**§11.9** The `tenant_plan_history.from_tier` is nullable for the very first record (the auto-default at provisioning). Subsequent records always have a non-NULL from_tier.

**§11.10** The metering event recorded by clause #15 carries `axis = plan_change` — which is NOT in the closed 4-axis enum. This is intentional: the synthetic event is recorded directly in memory chain only, not in the metering_events table. The `idempotency_key = "plan_change_<history_id>"` plus the absence of a metering_events row is the disambiguation.

**§11.11** The 24h rate limit is per tenant, not per actor. A tenant_admin and the founder cannot both change the same tenant within 24h via regular paths; the founder-override path bypasses.

**§11.12** Plan changes for terminated tenants (TASK-TEN-104) are rejected at the handler (404). The trigger would catch it too via RLS, but the handler's explicit check is more user-friendly.

**§11.13** The `next_scheduled_change` JSONB schema is fixed: `{target_tier, effective_at, history_id}`. The history_id back-link enables the cancellation handler to mark the history row's effective_at = null without losing the original record.

**§11.14** The CI test `caps_drift_test` reads constants from `caps.rs` via cargo-expand and compares to a documented table in `docs/tasks/ten/PLAN_CAPS.md` (one-source-of-truth audit).

**§11.15** The from_tier_caps_snapshot uses `caps_snapshot_json(tier)` which is `serde_json::to_value(caps_for(tier))`. The JSONB shape matches `TierCaps` exactly.

**§11.16** Plan-history pagination uses keyset pagination on `(occurred_at DESC, id DESC)` to avoid OFFSET drift.

**§11.17** The reason field is text; TASK-MEMORY-111 PII scrubbing applies before chain emission. Operators see the unscrubbed reason in Postgres (RLS-scoped); chain holds scrubbed.

**§11.18** The handler computes proration cents using the `TIER_PRICE_CENTS_MONTHLY` constants. The constants are the source of truth for billing-pricing; TASK-TEN-003 Stripe integration reads the same constants for invoice line items.

**§11.19** The `plan_change` synthetic metering audit kind helps billing reconcile when proration crosses a period boundary (e.g., upgrade on day 30 of month → some days at old tier, some at new). The chain row carries both tiers + the effective_at, so billing can split correctly.

**§11.20** The founder-tenant detection is via the `is_founder_tenant` boolean; the role `cyberskill_founder` is a subject role (per TASK-AUTH-101). A subject with role `cyberskill_founder` can operate on any tenant via the founder-override handler; a non-founder cannot change a founder tenant via any path.

**§11.21** All API responses use snake_case JSON; the OpenAPI doc reflects exactly the field shape shown in §8.

**§11.22** The downgrade-defer mechanism's "effective at next period end" is computed at request time; if the tenant's billing cycle changes between request and execution (rare), the next period_close job uses the THEN-current period boundary, not the request-time one.

---

*End of TASK-TEN-002 spec.*
