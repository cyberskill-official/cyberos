---
id: TASK-TIME-005
title: "TIME billable flag cascade — 4-step resolver (entry override → project default → engagement policy → tenant default) with snapshot on row"
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
module: TIME
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CCO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-TIME-001, TASK-PROJ-006, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-TIME-001, TASK-PROJ-006]
blocks: [TASK-TIME-009]

source_pages:
  - website/docs/modules/time.html#billable-cascade

source_decisions:
  - DEC-1410 2026-05-17 — Billable flag determined at entry-write time via 4-step cascade; snapshotted on row (entry's `is_billable` column populated immediately); never re-resolved live
  - DEC-1411 2026-05-17 — Closed enum `billable_source` = {entry_override, project_default, engagement_policy, tenant_default}; CI cardinality 4
  - DEC-1412 2026-05-17 — Cascade order: entry.override > project.default > engagement.policy > tenant.default; first non-NULL wins
  - DEC-1413 2026-05-17 — Member CAN override per-entry (UI checkbox); engagement_admin can override project default; cfo can override engagement policy
  - DEC-1414 2026-05-17 — Snapshot principle: invoice-time queries use `entries.is_billable` directly; no live re-cascade; ensures invoice immutability
  - DEC-1415 2026-05-17 — memory audit kinds: time.billable_resolved, time.billable_overridden_at_entry, time.project_default_changed, time.engagement_policy_changed

language: rust 1.81
service: cyberos/services/time/
new_files:
  - services/time/migrations/0004_billable_defaults.sql
  - services/time/src/billable/mod.rs
  - services/time/src/billable/cascade.rs
  - services/time/src/audit/billable_events.rs
  - services/time/tests/billable_cascade_test.rs
  - services/time/tests/billable_entry_override_test.rs
  - services/time/tests/billable_snapshot_immutable_test.rs
  - services/time/tests/billable_source_enum_test.rs
  - services/time/tests/billable_engagement_default_test.rs
  - services/time/tests/billable_audit_emission_test.rs

modified_files:
  - services/time/src/entry/create.rs

allowed_tools:
  - file_read: services/time/**
  - file_write: services/time/{src,tests,migrations}/**
  - bash: cd services/time && cargo test billable

disallowed_tools:
  - re-resolve billable flag post-entry-write (per DEC-1414 snapshot principle)
  - skip cascade and assume tenant default (per DEC-1410)

effort_hours: 5
subtasks:
  - "0.4h: 0004 migration (project/engagement/tenant billable_default + entry.is_billable column)"
  - "0.4h: billable/mod.rs + closed enum"
  - "0.6h: billable/cascade.rs (4-step resolver)"
  - "0.3h: integration into entry/create.rs"
  - "0.3h: audit/billable_events.rs"
  - "1.5h: tests — 6 test files"
  - "1.5h: integration smoke"

risk_if_skipped: "Without cascade, every TIME entry needs explicit billable flag from Member → UX nightmare + error-prone + non-billable hours leak to invoices (revenue inflation = fraud) OR billable hours marked non-billable (revenue loss). Without DEC-1414 snapshot, retroactive engagement_policy changes silently flip past entries' billability → invoice corruption. Without DEC-1411 closed enum, ad-hoc source labels obscure audit. The 5h effort lands the auto-classification primitive."
---

## §1 — Description (BCP-14 normative)

The TIME service **MUST** ship 4-step billable-flag cascade at `services/time/src/billable/` with closed source enum, snapshot at entry-write, override-at-each-level controls, and 4 memory audit kinds.

1. **MUST** define closed `billable_source` enum: `('entry_override','project_default','engagement_policy','tenant_default')` per DEC-1411. Cardinality 4.

2. **MUST** add columns to TIME entries: `is_billable BOOLEAN NOT NULL`, `billable_source billable_source NOT NULL`. Snapshotted at entry creation per DEC-1410.

3. **MUST** add `billable_default BOOLEAN` columns to: `projects` (NULL = inherit), `engagements` (NULL = inherit), `tenants` (NOT NULL — required).

4. **MUST** resolve billable flag per `cascade.rs::resolve(entry_override, project_id, engagement_id, tenant_id)` per DEC-1412:
   - If `entry_override IS NOT NULL` → return (override, 'entry_override').
   - Else SELECT projects.billable_default; if NOT NULL → return (project_default, 'project_default').
   - Else SELECT engagements.billable_default; if NOT NULL → return (engagement_policy, 'engagement_policy').
   - Else SELECT tenants.billable_default → return (tenant_default, 'tenant_default').
   - Sets `entries.is_billable` + `entries.billable_source`.

5. **MUST** snapshot at write per DEC-1414. Post-write `is_billable` is IMMUTABLE — never re-cascaded.

6. **MUST** support per-level override per DEC-1413:
   - Member: `entry.billable_override BOOLEAN` at create-time.
   - engagement_admin: `PATCH /v1/projects/{id}` body `{ billable_default }` (project-level).
   - cfo: `PATCH /v1/engagements/{id}` body `{ billable_default }` (engagement-level).
   - cfo: `PATCH /v1/admin/tenants/{id}` body `{ billable_default }` (tenant-level).
   - Each level change emits respective memory audit row.

7. **MUST** emit 4 memory audit kinds per DEC-1415:
   - `time.billable_resolved` (sev-3 — informational; sampled 1%)
   - `time.billable_overridden_at_entry` (sev-3 — when Member uses checkbox)
   - `time.project_default_changed` (sev-2)
   - `time.engagement_policy_changed` (sev-2)

8. **MUST** thread trace_id end-to-end.

9. **MUST NOT** re-resolve post-write (per DEC-1414).

10. **MUST NOT** allow tenant_default NULL (per DEC-1410 — must have base).

---

## §2 — Why this design (rationale)

**Why 4-step cascade vs simpler N-step (§1 #4, DEC-1412)?** Industry convention from consulting tools — billability rules naturally hierarchical (tenant baseline → engagement contract → project type → entry exception).

**Why snapshot at write (§1 #5, DEC-1414)?** Without snapshot, retroactive policy changes silently rewrite history. With snapshot, invoice integrity preserved across policy evolution.

**Why per-level override (§1 #6, DEC-1413)?** Different roles legitimately need different override scopes. Tenant admin sets defaults; engagement_admin tunes per-engagement; cfo overrides specific engagements; Member flags exceptions.

---

## §3 — API contract

```sql
-- 0004_billable_defaults.sql
CREATE TYPE billable_source AS ENUM ('entry_override','project_default','engagement_policy','tenant_default');

ALTER TABLE time_entries
  ADD COLUMN is_billable BOOLEAN NOT NULL,
  ADD COLUMN billable_source billable_source NOT NULL,
  ADD COLUMN billable_override BOOLEAN;

ALTER TABLE projects ADD COLUMN billable_default BOOLEAN;
ALTER TABLE engagements ADD COLUMN billable_default BOOLEAN;
ALTER TABLE tenants ADD COLUMN billable_default BOOLEAN NOT NULL DEFAULT true;
```

Endpoints:
```text
PATCH  /v1/projects/{id}        { billable_default }     (engagement_admin)
PATCH  /v1/engagements/{id}     { billable_default }     (cfo)
PATCH  /v1/admin/tenants/{id}   { billable_default }     (cfo)
```

---

## §4 — Acceptance criteria

1. **billable_source cardinality 4**.
2. **Tenant default applied** — entry with no project/engagement override → source=tenant_default.
3. **Engagement override** — engagement.billable_default=false → entry billable=false; source=engagement_policy.
4. **Project override** — project.billable_default=false (engagement=true) → entry billable=false; source=project_default.
5. **Entry override** — Member sets billable_override=true (project=false) → entry billable=true; source=entry_override.
6. **Snapshot immutable** — entry created billable=true; subsequent engagement policy change to false → entry remains billable=true.
7. **4 memory audit kinds emitted**.
8. **Tenant default required** — INSERT tenants without billable_default fails NOT NULL.
9. **Engagement_admin updates project** — PATCH succeeds + audit.
10. **CFO updates engagement** — PATCH succeeds + audit.
11. **Non-admin cannot update project default** — 403.
12. **Member entry override at form** — UI checkbox flips per-entry.
13. **Cascade order verified** — fixture with all 4 levels populated → entry_override wins.
14. **Cascade order skip NULL** — project.billable_default=NULL → falls through to engagement.
15. **PII scrub** — audit row carries no description text.
16. **Trace_id end-to-end**.
17. **Cross-tenant RLS** — engagement default change scoped.
18. **Audit sampled correctly** — `billable_resolved` at 1% per TASK-OBS-006 tail.
19. **Member override doesn't change project default** — entry-level only.
20. **Concurrent updates race-safe** — last-write-wins on policy fields.

---

## §5 — Verification

```rust
#[tokio::test]
async fn cascade_4_step() {
    let ctx = TestContext::with_tenant_default(true).await;
    ctx.set_engagement_default(ctx.eng_id, Some(false)).await;
    ctx.set_project_default(ctx.proj_id, Some(true)).await;

    let entry_id = ctx.create_entry(ctx.member_id, ctx.proj_id, None).await;
    let row: (bool, String) = sqlx::query_as("SELECT is_billable, billable_source::text FROM time_entries WHERE entry_id=$1")
        .bind(entry_id).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(row.0, true);
    assert_eq!(row.1, "project_default");
}

#[tokio::test]
async fn entry_override_wins() {
    let ctx = TestContext::with_engagement_default(false).await;
    let entry_id = ctx.create_entry_with_override(ctx.member_id, ctx.eng_id, true).await;
    let row: (bool, String) = sqlx::query_as("SELECT is_billable, billable_source::text FROM time_entries WHERE entry_id=$1")
        .bind(entry_id).fetch_one(&ctx.pool).await.unwrap();
    assert!(row.0);
    assert_eq!(row.1, "entry_override");
}

#[tokio::test]
async fn snapshot_immutable() {
    let ctx = TestContext::with_engagement_default(true).await;
    let entry_id = ctx.create_entry(ctx.member_id, ctx.proj_id, None).await;
    ctx.set_engagement_default(ctx.eng_id, Some(false)).await;
    let row: (bool,) = sqlx::query_as("SELECT is_billable FROM time_entries WHERE entry_id=$1")
        .bind(entry_id).fetch_one(&ctx.pool).await.unwrap();
    assert!(row.0);  // unchanged
}

// 5.4..5.6: enum cardinality, role checks, audit emission
```

---

## §7 — Dependencies

**Upstream:** TASK-TIME-001 (entry write), TASK-PROJ-006 (project billable_default column).
**Cross-module:** TASK-AUTH-101 (engagement_admin + cfo roles), TASK-AI-003, TASK-MEMORY-111.

---

## §8 — Example payload

`time.engagement_policy_changed`:
```json
{
  "kind": "time.engagement_policy_changed",
  "severity": 2,
  "tenant_id": "8a2f...",
  "actor_id": "user.cfo.789",
  "trace_id": "...",
  "payload": {
    "engagement_id": "0190...",
    "from_default": true,
    "to_default": false
  }
}
```

---

## §9 — Open questions

Deferred:
- **Deferred:** Per-task billable default (5th cascade level) — slice 2.
- **Deferred:** Per-role billable rules (e.g. senior consultants always billable) — slice 2.
- **Deferred:** Time-of-day billable rules (weekend/night premium) — slice 3.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Tenant default NULL | NOT NULL constraint | INSERT fails | Tenant config must set |
| Project/engagement deleted mid-entry-write | FK soft | Cascade falls through to next level | Inherent |
| Policy change race | tx isolation | Later writes use new policy; older entries snapshotted | Inherent |
| Override checkbox UI bug sends NULL | handler treats as no-override | Falls through cascade | UI fix |
| Cross-tenant policy update | RLS | 403 | Inherent |
| Project from different engagement | engagement check at entry | 400 | Inherent |
| Member without engagement membership | RLS at write | 403 | Inherent |
| Cascade query slow at high volume | OBS latency | sev-3; index ensures O(1) | Index check |
| Source enum extended without migration | CI cardinality test | CI fails | Migration first |
| Snapshot field accidentally mutated | REVOKE UPDATE | DB rejects | Inherent |
| Audit row sampling drops critical event | TASK-OBS-006 tail | Entry-override always emitted (not sampled) per AC #18 | Inherent |
| Engagement_admin updates project in other engagement | role scope check | 403 | Inherent |
| Tenant default change retroactively wanted | snapshot principle | Past entries unchanged | Manual re-issue if needed |
| Concurrent project default updates | tx isolation | Last writer wins | Inherent |
| Member overrides for non-billable engagement | allowed but flagged | Audit notes deviation | Engagement_admin review |

---

## §11 — Implementation notes

**§11.1** Cascade resolver does 1 SELECT per entry write; ~1ms with proper indexes.

**§11.2** Snapshot fields: REVOKE UPDATE on (`is_billable`, `billable_source`) per task-audit skill rule 12.

**§11.3** Override boolean explicitly nullable — distinguishes "no override" (NULL) from "false override" (FALSE).

**§11.4** Audit sampling at 1% via TASK-OBS-006 except entry_override always emitted (Member intent).

**§11.5** UI form pre-fetches cascade preview ("This will be billable based on engagement policy").

**§11.6** Policy change endpoints emit memory before commit (atomic).

**§11.7** Tenant default defaults to TRUE on tenant create (most consultancies bill by default).

**§11.8** Project/engagement defaults default to NULL (inherit).

**§11.9** PII: no description in audit; only IDs.

**§11.10** Cross-tenant via RLS.

---

*End of TASK-TIME-005 spec.*
