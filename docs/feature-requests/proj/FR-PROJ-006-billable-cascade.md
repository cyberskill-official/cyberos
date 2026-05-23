---
id: FR-PROJ-006
title: "Billable cascade — Member-override → task-class → role-default → fallback; resolution snapshot at time-entry write"
module: PROJ
priority: MUST
status: done
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_frs: [FR-PROJ-001, FR-PROJ-005, FR-PROJ-007, FR-TIME-005, FR-AUTH-003]
depends_on: [FR-PROJ-005]
blocks: [FR-PROJ-007, FR-TIME-005]

source_pages:
  - website/docs/modules/proj.html#billable-cascade
source_decisions:
  - DEC-270 (billable flag computed via 4-tier cascade with explicit precedence)
  - DEC-271 (resolution snapshot at write-time; later override changes don't retro-edit entries)
  - DEC-272 (fallback default is `false` — never silently bill the customer)

language: rust 1.81
service: cyberos/services/proj-sync/
new_files:
  - services/proj-sync/migrations/0006_member_billable_overrides.sql
  - services/proj-sync/migrations/0006_task_class_billable.sql
  - services/proj-sync/src/billable/mod.rs
  - services/proj-sync/src/billable/cascade.rs
  - services/proj-sync/tests/billable_cascade_test.rs
modified_files:
  - services/proj-sync/src/types.rs                  # TaskClass enum
allowed_tools:
  - file_read: services/proj-sync/**
  - file_write: services/proj-sync/{src,tests,migrations}/**
  - bash: cd services/proj-sync && cargo test billable
disallowed_tools:
  - default-fallback to `true` (per DEC-272 — would silently bill)
  - resolve cascade after time-entry insert (per DEC-271 — snapshot at write-time)

effort_hours: 6
sub_tasks:
  - "0.5h: 0006_member_billable_overrides.sql migration"
  - "0.5h: 0006_task_class_billable.sql migration"
  - "0.5h: TaskClass enum (feature_work | bug_fix | meeting | review | research | sales_call | admin | training)"
  - "0.5h: billable/mod.rs — BillableResolution struct {value, source, tier_consulted, snapshot_at}"
  - "1.5h: cascade.rs — resolve_billable(member_id, task_class, role, engagement_id, at_date) -> BillableResolution"
  - "0.5h: snapshot embedded in time entry row (already in FR-TIME-005)"
  - "0.5h: memory audit row 'proj.billable_resolved' per resolution"
  - "1.5h: billable_cascade_test.rs — all 4 tiers + tier-2 task_class override + member override + fallback"
risk_if_skipped: "Without explicit cascade, every time-entry write reinvents 'is this billable?' — drift between codepaths means same task billed differently from kanban vs. timeline views. Without snapshot, raising a contractor's hourly rate retroactively re-bills past entries (illegal in many jurisdictions). Without 'never default-bill', accidentally-tracked work gets invoiced to the client (customer churn)."
---

## §1 — Description (BCP-14 normative)

The billable-cascade resolver **MUST** compute `billable: bool` for each time-entry write by consulting 4 sources in strict precedence order:

1. **Tier 1 — Member-override**: explicit per-member rule on this engagement (e.g. "Alice is non-billable on this engagement"). Schema: `member_billable_overrides(member_id, engagement_id, billable, reason, created_at, tenant_id)`.
2. **Tier 2 — Task-class override**: explicit per-engagement per-task-class rule (e.g. "research is non-billable on engagement X"). Schema: `task_class_billable(engagement_id, task_class, billable, tenant_id)`.
3. **Tier 3 — Role-default**: the matching rate card's `billable_default` (FR-PROJ-005).
4. **Tier 4 — Fallback**: `false`. Never silently bill.

The resolver:

1. **MUST** return `BillableResolution { value: bool, source: Tier, tier_consulted: u8, snapshot_at: i64 }` — every resolution carries provenance.
2. **MUST** stop at the FIRST applicable tier (no further consultation once a rule matches).
3. **MUST** snapshot the resolved value AT WRITE TIME of the time-entry. Later changes to any tier do NOT retroactively modify existing entries.
4. **MUST** emit `proj.billable_resolved` memory audit row per call with payload `{time_entry_id, member_id, engagement_id, task_class, role, billable_value, source_tier, snapshot_at_ns, trace_id}`.
5. **MUST** validate `member_id` belongs to the engagement (cross-engagement override invalid).
6. **MUST** be deterministic per `(member_id, task_class, role, engagement_id, at_date)` snapshot at AT-DATE; cascade reads rate card via `FR-PROJ-005::lookup_at(at_date)` so the snapshot is reproducible.
7. **MUST** emit OTel metrics:
    - `proj_billable_resolutions_total{value, source}` (counter; cardinality 2 × 4 = 8).
    - `proj_billable_cascade_depth` (histogram).
8. **MUST** expose REST: `POST /api/proj/engagements/:eng/billable-cascade/resolve` with body `{member_id, task_class, role, at_date}` → 200 `BillableResolution`. Used by clients (Kanban, timeline) for preview before writing.
9. **MUST** expose admin CRUD for Tier-1 + Tier-2 overrides:
    - `POST/DELETE /api/proj/engagements/:eng/member-overrides/:member_id`
    - `POST/DELETE /api/proj/engagements/:eng/task-class/:class`
10. **MUST** emit memory audit on override CRUD: `proj.member_billable_override_set` / `proj.task_class_billable_set`.
11. **MUST** RLS-enforce per tenant (FR-AUTH-003).
12. **MUST** validate the resolver inputs at handler boundary: `member_id` exists in tenant; `engagement_id` exists in tenant; `at_date` is not more than 5 years in the past; `currency` is in the rate-card's enum. Invalid inputs → 400 with structured error.
13. **MUST** support a "bulk-resolve" endpoint: `POST /api/proj/engagements/:eng/billable-cascade/bulk` with `[{member_id, task_class, role, at_date}]` array (max 1000 items) → returns array of resolutions in input order. Used by timesheet-import flows.
14. **MUST** mark resolved billable values as IMMUTABLE on the time entry row: any PATCH attempting to change `billable_snapshot` is rejected with 405 (force re-resolution via separate time-entry-recompute admin endpoint, which itself is audit-trail-heavy).
15. **MUST** support per-engagement DEFAULT TIER-2 task-class billable: `cyberos_proj_engagement_settings.default_task_class_billable = {feature_work: true, research: false, ...}` applied to engagement creation if no per-class override exists. Tenant-admin sets the default.
16. **MUST** track `proj_billable_resolution_latency_ms` histogram per call; p95 budget < 20ms (cascade is in the time-entry hot path).
17. **MUST** support querying historical resolutions: `GET /api/proj/engagements/:eng/billable-cascade/history?member=:id&from=&to=` returns memory audit rows. Useful for auditing "why was this hour billed."
18. **MUST** include `effective_overrides_applied` field in `BillableResolution`: even if Tier 3 matched, the response carries metadata about which other tiers existed but didn't match (e.g. "rate card was used; member-override exists but matches the same value"). Operator transparency.
19. **MUST** support a tenant-level "billable by default" policy override of the §1 Tier-4 fallback: `cyberos_proj_tenant_settings.cascade_fallback_billable = false` (default false). Tenants who explicitly want default-billable can set true (e.g. internal-billing tenants). The default remains conservative.
20. **MUST** validate `engagement_id` is not archived: resolutions on archived engagements are forbidden (would create new time entries on closed engagements). 409 `engagement_archived`.
21. **MUST** support an "explain" mode on the resolve endpoint: `?explain=true` returns the full decision tree showing what each tier returned (matched/not-matched), not just the winner. Used by operator UI for debugging.

---

## §2 — Why this design (rationale for humans)

**Why 4 tiers (DEC-270, §1)?** Real-world billable rules need granularity: org-level role defaults are too coarse (engineer is "usually" billable but specific engineers on specific eng aren't); per-member always-overridable is too granular (no defaults; every member needs explicit). Four tiers cover the cases without combinatorial explosion.

**Why first-match wins (§1 #2)?** Layered overrides need clear precedence. The pattern matches CSS specificity / IAM policy precedence — specific overrides general.

**Why snapshot at write-time (DEC-271, §1 #3)?** Retroactive rule changes break invoices already sent. A client paid for 100 hours billed at one rate; later flipping a member to non-billable would retroactively zero out a paid invoice. Snapshot = bill = invariant.

**Why fallback `false` not `true` (DEC-272, §1)?** Asymmetric risk: defaulting to billable means a tracking mistake bills a client (loss of trust + refund + admin overhead). Defaulting to non-billable means a missed bill (caught at invoice review). The latter is recoverable.

**Why audit per resolution (§1 #4)?** Auditors investigating "why was this hour billed" need the full chain: which tier matched, which rule, when. The audit row encodes the decision tree at the time of decision.

**Why preview endpoint (§1 #8)?** UX: when an engineer logs time on a task, the UI shows "Billable: ✓ (via task_class override)" so they know what they're committing to. Without preview, the UX shows nothing → surprise at invoice time.

**Why input validation at handler (§1 #12)?** Defence in depth: resolver assumes valid inputs (per the contract); handler validates so downstream resolver code is small + fast. Bad inputs returning 400 with structured errors is far better than silent fallthrough.

**Why bulk-resolve (§1 #13)?** Timesheet imports (CSV from external tools) have hundreds of entries; per-entry HTTP call = thousands of network round-trips. Bulk amortises the cost.

**Why immutability on time entry (§1 #14)?** Snapshot is the bill-truth; mutating it post-write retroactively re-bills. Force-re-resolution path is admin-only and audit-heavy because it's an exceptional operator workflow.

**Why engagement-level default task-class billable (§1 #15)?** Common case: every engagement has the same "feature_work=billable, research=non-billable" pattern. Without engagement defaults, operators set 8 task_class rows per engagement. Defaults = one set per tenant.

**Why p95 < 20ms (§1 #16)?** Time-entry write is in the operator's UI critical path; >20ms feels laggy. Cascade is 4 SQL queries worst-case; budget is generous.

**Why historical resolution query (§1 #17)?** Auditors investigating an invoice line item ask "why was this billable?" — answer requires the resolution that was made at write time. memory audit query gives the trace.

**Why effective_overrides_applied metadata (§1 #18)?** Operator transparency: "Tier 3 matched (rate card default = true); a member override exists but also returns true, so no change." Without this, operators can't tell if the result is from the priority tier or a deeper one happening to agree.

**Why per-tenant fallback override (§1 #19)?** Internal-billing tenants (cost-center accounting) want default-billable; SaaS-revenue tenants want default-not-billable. One global default doesn't fit; per-tenant policy does. Default remains conservative.

**Why archived-engagement rejection (§1 #20)?** Closed engagements shouldn't accept new time entries (would re-open billing). Rejecting at the cascade layer catches operators who don't see the archived status in their UI.

**Why explain mode (§1 #21)?** Debugging cascade decisions in production: operator wants to know "why did this resolve to false?" — explain mode shows each tier's match/no-match status. Default mode is summary; explain is for diagnostics.

---

## §3 — API contract

### Migrations

```sql
-- 0006_member_billable_overrides.sql
CREATE TABLE member_billable_overrides (
    member_id      UUID NOT NULL,
    engagement_id  UUID NOT NULL REFERENCES engagements(id),
    billable       BOOLEAN NOT NULL,
    reason         TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by     UUID NOT NULL,
    tenant_id      UUID NOT NULL,
    PRIMARY KEY (member_id, engagement_id)
);
CREATE POLICY mbo_tenant_isolation ON member_billable_overrides
    USING (tenant_id = current_setting('app.tenant_id')::uuid);

-- 0006_task_class_billable.sql
CREATE TABLE task_class_billable (
    engagement_id  UUID NOT NULL REFERENCES engagements(id),
    task_class     TEXT NOT NULL CHECK (task_class IN
                   ('feature_work','bug_fix','meeting','review','research','sales_call','admin','training')),
    billable       BOOLEAN NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by     UUID NOT NULL,
    tenant_id      UUID NOT NULL,
    PRIMARY KEY (engagement_id, task_class)
);
CREATE POLICY tcb_tenant_isolation ON task_class_billable
    USING (tenant_id = current_setting('app.tenant_id')::uuid);
```

### Cascade resolver

```rust
// services/proj-sync/src/billable/mod.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum TaskClass {
    FeatureWork, BugFix, Meeting, Review, Research, SalesCall, Admin, Training,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Tier { MemberOverride, TaskClassOverride, RoleDefault, Fallback }

#[derive(Clone, Debug, Serialize)]
pub struct BillableResolution {
    pub value:           bool,
    pub source:          Tier,
    pub tier_consulted:  u8,             // 1..=4 (matches §1 #2 first-match-wins; this is the matching tier)
    pub snapshot_at_ns:  i64,
}

pub async fn resolve(
    pool: &sqlx::PgPool,
    member_id: uuid::Uuid,
    task_class: TaskClass,
    role: crate::rate_card::Role,
    engagement_id: uuid::Uuid,
    at_date: chrono::NaiveDate,
    currency: crate::rate_card::Currency,
) -> Result<BillableResolution, BillableError> {
    let now_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap();

    // Tier 1: member-override
    if let Some(row) = sqlx::query!(
        "SELECT billable FROM member_billable_overrides
         WHERE member_id = $1 AND engagement_id = $2",
        member_id, engagement_id
    ).fetch_optional(pool).await.map_err(map_db)? {
        return Ok(BillableResolution {
            value: row.billable, source: Tier::MemberOverride,
            tier_consulted: 1, snapshot_at_ns: now_ns,
        });
    }

    // Tier 2: task-class override
    if let Some(row) = sqlx::query!(
        "SELECT billable FROM task_class_billable
         WHERE engagement_id = $1 AND task_class = $2",
        engagement_id, task_class as TaskClass
    ).fetch_optional(pool).await.map_err(map_db)? {
        return Ok(BillableResolution {
            value: row.billable, source: Tier::TaskClassOverride,
            tier_consulted: 2, snapshot_at_ns: now_ns,
        });
    }

    // Tier 3: role-default (rate card)
    if let Ok(card) = crate::rate_card::lookup_at(pool, engagement_id, role, currency, at_date).await {
        return Ok(BillableResolution {
            value: card.billable_default, source: Tier::RoleDefault,
            tier_consulted: 3, snapshot_at_ns: now_ns,
        });
    }

    // Tier 4: fallback
    Ok(BillableResolution {
        value: false, source: Tier::Fallback,
        tier_consulted: 4, snapshot_at_ns: now_ns,
    })
}
```

---

## §4 — Acceptance criteria

1. **Tier 1 hits first** — member-override = true, all else default → resolution.source = MemberOverride.
2. **Tier 1 false stops cascade** — member-override = false → resolution.value = false, source = MemberOverride (does not fall through).
3. **Tier 2 hits when no Tier 1** — no override + task_class_billable = true → source = TaskClassOverride.
4. **Tier 3 hits when no T1/T2** — rate card default = true → source = RoleDefault.
5. **Tier 4 fallback** — none of T1/T2/T3 match (e.g. no rate card for that role/date) → source = Fallback, value = false.
6. **Snapshot preserved on later override change** — write entry at T1 with member-override = true; at T2 flip override to false; original entry's billable stays true (snapshotted).
7. **Cross-engagement override invalid** — override row for (member, engagement_A) does NOT affect engagement_B.
8. **Preview endpoint** — `POST /resolve` returns same value as actual write would.
9. **Audit row emitted** — every resolve → `proj.billable_resolved` row with source_tier.
10. **Audit on override CRUD** — `proj.member_billable_override_set` on POST.
11. **Counter increments** — 100 resolves → counter `proj_billable_resolutions_total` sums to 100 across labels.
12. **RLS enforces** — tenant A's override invisible to tenant B.
13. **Cascade depth metric** — Tier 4 fallback → depth = 4; Tier 1 hit → depth = 1.
14. **Idempotent override POST** — same Idempotency-Key → returns prior; same key + diff body → 409.
15. **Handler input validation** — POST with `at_date=2010-01-01` (>5y past) → 400 `at_date_too_old`; invalid member/engagement → 400 (AC for §1 #12).
16. **Bulk resolve handles 1000 items** — POST /bulk with 1000-element array → 200 with 1000-element response in input order; > 1000 → 413 (AC for §1 #13).
17. **Bulk preserves per-item independence** — bulk with mix of valid + invalid → each item's status independent (AC for §1 #13).
18. **billable_snapshot PATCH rejected** — direct PATCH of time entry's billable_snapshot → 405 (AC for §1 #14).
19. **Engagement-default task-class billable applied** — tenant default `feature_work: true`; new engagement → has tier-2 row matching default; resolve returns true via Tier 2 (AC for §1 #15).
20. **Latency p95 < 20ms** — load test 1000 resolves → histogram p95 < 20ms (AC for §1 #16).
21. **Historical query returns memory rows** — GET /history → audit rows for member+time range (AC for §1 #17).
22. **effective_overrides_applied in response** — resolve where Tier 1 + Tier 3 both exist and agree → metadata shows both (AC for §1 #18).
23. **Tenant fallback override honoured** — set `cascade_fallback_billable=true`; Tier 4 fallback → value=true (AC for §1 #19).
24. **Archived engagement rejected** — resolve on archived eng → 409 `engagement_archived` (AC for §1 #20).
25. **Explain mode returns full tree** — POST /resolve?explain=true → response includes per-tier match/value; default mode returns summary only (AC for §1 #21).

---

## §5 — Verification

```rust
#[tokio::test]
async fn tier_1_member_override_first_match() {
    let env = TestEnv::new().await;
    let (eng, alice) = env.bootstrap().await;
    env.set_member_override(alice, eng, true).await;
    env.set_task_class(eng, TaskClass::Research, false).await;
    let res = resolve(&env.pool, alice, TaskClass::Research, Role::Engineer, eng,
                      "2026-05-16".parse().unwrap(), Currency::VND).await.unwrap();
    assert_eq!(res.value, true);
    assert_eq!(res.source, Tier::MemberOverride);
    assert_eq!(res.tier_consulted, 1);
}

#[tokio::test]
async fn tier_2_task_class_when_no_member() {
    let env = TestEnv::new().await;
    let (eng, alice) = env.bootstrap().await;
    env.set_task_class(eng, TaskClass::Meeting, false).await;
    env.create_rate_card_default(eng, Role::Engineer, true).await;
    let res = resolve(&env.pool, alice, TaskClass::Meeting, Role::Engineer, eng,
                      "2026-05-16".parse().unwrap(), Currency::VND).await.unwrap();
    assert_eq!(res.source, Tier::TaskClassOverride);
    assert_eq!(res.value, false);
}

#[tokio::test]
async fn tier_4_fallback_is_false() {
    let env = TestEnv::new().await;
    let (eng, alice) = env.bootstrap().await;
    // No overrides; no rate card
    let res = resolve(&env.pool, alice, TaskClass::Research, Role::Engineer, eng,
                      "2026-05-16".parse().unwrap(), Currency::VND).await.unwrap();
    assert_eq!(res.source, Tier::Fallback);
    assert_eq!(res.value, false);
    assert_eq!(res.tier_consulted, 4);
}

#[tokio::test]
async fn snapshot_immutable_after_override_change() {
    let env = TestEnv::new().await;
    let (eng, alice) = env.bootstrap().await;
    env.set_member_override(alice, eng, true).await;
    let entry = env.create_time_entry(alice, eng, TaskClass::FeatureWork).await;
    assert_eq!(entry.billable_snapshot, true);

    env.set_member_override(alice, eng, false).await;
    let refetched = env.read_time_entry(entry.id).await;
    assert_eq!(refetched.billable_snapshot, true);   // snapshot preserved
}
```

---

## §6 — Implementation skeleton

(API + DB schema above.)

---

## §7 — Dependencies

- **FR-PROJ-005** — rate card lookup at Tier 3.
- **FR-PROJ-007 (downstream)** — billing modes consume resolved billable flag.
- **FR-TIME-005** — time-entry snapshot field stores this resolution.
- **FR-AUTH-003** — RLS.

---

## §8 — Example payloads

```json
{
  "kind": "proj.billable_resolved",
  "payload": {
    "time_entry_id":    "te-...",
    "member_id":        "mb-...",
    "engagement_id":    "eng-...",
    "task_class":       "feature_work",
    "role":             "engineer",
    "billable_value":   true,
    "source_tier":      "role_default",
    "tier_consulted":   3,
    "snapshot_at_ns":   1747407137483000000,
    "trace_id":         "0af..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Per-member-per-task-class override (5th tier) — slice 4+; current 4 tiers cover empirical cases.
- Time-of-day rules (after 6pm = non-billable) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| No rate card for role+date | Tier 3 lookup_at Err | Falls through to Tier 4 (false) | Operator adds card OR accepts non-billable |
| Duplicate override on PK | unique constraint | 409 on POST | Caller checks first |
| Cross-engagement override | row scoped to (member, engagement) | Other engagements unaffected | None |
| Race: resolve mid-override-update | each resolve uses current snapshot | Snapshot at resolve-time is consistent | None |
| Tier 1 override deleted | next resolve falls to Tier 2 | New entries get new value | By design (snapshot protects historical) |
| Audit emit fails | Resolved value still returned; audit lost | sev-2 | Operator restores memory |
| Currency mismatch in rate-card lookup | lookup returns None → fallback to Tier 4 | False | Operator adds rate card |
| Tenant isolation broken (RLS bug) | property test catches | CI blocked | Author fixes |
| Idempotency-Key reuse with different body | handler check | 409 | Caller uses fresh key |
| Member not in engagement | should reject upstream | resolver doesn't validate; Tier 1 returns no row → falls through | Upstream caller validates |
| Storage Err mid-cascade | sqlx Err | 500 | Operator restores DB |
| Two tier-2 rules same engagement+class | PK prevents | 409 | None |
| Resolution at date in past | lookup_at uses historical card | Correct | None |
| TaskClass enum drift | CHECK constraint | 422 | Caller uses valid enum |
| Bulk-resolve > 1000 items | bounded | 413 | Caller batches |
| Bulk-resolve item validation failure | per-item status | partial results | Caller |
| Engagement archived mid-resolve | check at top of resolver | 409 | Operator |
| Tenant fallback override = true | Tier 4 returns true | by config | None |
| Explain mode response > 100KB | bounded by tier count (4) | None | None |
| Currency mismatch in rate-card lookup_at | Tier 3 returns None → Tier 4 | falls through | Operator adds card |
| at_date in past after engagement archive | resolver returns archived error | 409 | None |
| Bulk-resolve includes deleted member | per-item 400 | proceeds with others | None |
| Same task_class enum value different storage (case drift) | strict enum matching | 422 | Caller normalises |
| Resolution latency > 20ms | OBS histogram | SEV-3 if sustained | Operator investigates |
| Resolver called > tenant rate-limit | upstream limiter | 429 | Caller backs off |
| Tier 1 override row with `billable=null` | NOT NULL constraint | rejected at insert | None |
| Audit row dedup (same time_entry_id twice) | none enforced | duplicates possible; downstream dedups | None |
| Concurrent override edit + resolve | resolver reads snapshot at time | both consistent | None |

---

## §11 — Implementation notes

- `BillableResolution.tier_consulted` is the matching tier (1..=4), not the depth searched. Both interpretations are useful; the field name reflects "which tier produced the value."
- The lookup query order matches §1: Tier 1 → 2 → 3 → 4. Each is a separate SQL — could be one big LEFT JOIN but readability wins for the 4-table case.
- The fallback case is its own enum variant rather than a sentinel value — callers pattern-match cleanly.
- Snapshot field lives on FR-TIME-005's time entry row; this FR's responsibility ends at returning `BillableResolution`.
- The `proj.billable_resolved` audit row's `time_entry_id` is the downstream entry being created; if resolver is called for PREVIEW (without writing), `time_entry_id` is null.
- Metrics cardinality bounded: 2 (value) × 4 (source) = 8 label combinations — safe.
- Input validation at the handler boundary is per feature-request-audit skill §3.4 rule 13 (RLS WITH CHECK + handler validation). Defence in depth.
- Bulk-resolve cap of 1000 chosen against: (a) Postgres prepared-statement parameter limit ≈ 65k; bulk of 1000 × 4 params = 4000 well under; (b) operator workflow envelopes (CSV imports typically < 500 rows).
- The "force re-resolution" admin endpoint for §1 #14 is intentionally hidden in admin namespace because: re-resolution invalidates prior bill amounts; operator must explicitly choose.
- Engagement-default task-class billable is an array of (task_class, billable) pairs in tenant settings; engagement creation iterates and inserts.
- 20ms p95 budget breaks down: ~5ms per query × 4 queries worst-case + ~0ms response build = 20ms. Tier-1 hit is ~5ms total (single query).
- Historical query uses the memory_outbox partial index `WHERE kind='proj.billable_resolved'` for fast retrieval.
- `effective_overrides_applied` adds ~3 SQL queries (one per non-matching tier); we run them in parallel via `tokio::join!` to keep latency bounded.
- The per-tenant fallback override defaults to `false` (conservative); only internal-billing tenants flip it.
- Archived-engagement check is at the top of resolver (before any tier query) so we fail fast on the common error.
- Explain mode is a UI affordance, not a contract guarantee — the structured response shape may evolve as new tiers are added (slice 4+).
- The `billable_snapshot` field on time entry is the bill-truth; cascading downstream (FR-PROJ-007 billing modes) trusts it without re-resolving.
- We considered caching cascade resolutions but rejected: cache invalidation on override changes is complex, and the resolver is fast (~20ms p95). Cache adds complexity without justified gain.
- The cascade order (Member → TaskClass → RoleDefault → Fallback) was chosen because: Member is the most specific (per-person decisions), TaskClass is mid (per-work-type), Role is general (per-job-function), Fallback is universal.
- We rejected a 5th tier (per-member-per-task-class) because the 4 tiers cover ~99% of empirical cases; a 5th tier adds combinatorial complexity for the rare case.

---

*End of FR-PROJ-006.*
