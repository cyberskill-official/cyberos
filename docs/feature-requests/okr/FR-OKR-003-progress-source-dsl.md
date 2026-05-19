---
id: FR-OKR-003
title: "OKR KR progress_source DSL — declarative query against PROJ / INV / HR / LEARN modules for auto-progress feed"
module: OKR
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng (CEO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-OKR-002, FR-OKR-004, FR-PROJ-013, FR-INV-009, FR-HR-008, FR-LEARN-001, FR-MEMORY-111]
depends_on: [FR-OKR-001]
blocks: [FR-OKR-004]

source_pages:
  - website/docs/modules/okr.html#progress-source-dsl

source_decisions:
  - DEC-1980 2026-05-17 — Declarative DSL: progress_source: {module, metric, filter, agg} → typed value; resolver per module
  - DEC-1981 2026-05-17 — Closed enum `dsl_module` = {proj, inv, hr, learn, custom_sql}; cardinality 5
  - DEC-1982 2026-05-17 — Closed enum `dsl_agg` = {sum, count, avg, max, min, latest}; cardinality 6
  - DEC-1983 2026-05-17 — Per-module metric whitelist — only safe SELECTs; custom_sql REQUIRES CFO+CEO co-sign
  - DEC-1984 2026-05-17 — DSL stored as JSONB on KR; resolved at progress compute time
  - DEC-1985 2026-05-17 — memory audit kinds: okr.progress_source_set, okr.progress_source_resolved, okr.progress_source_resolution_failed, okr.custom_sql_dual_signed

build_envelope:
  language: rust 1.81
  service: cyberos/services/okr/
  new_files:
    - services/okr/migrations/0003_progress_source.sql
    - services/okr/src/dsl/mod.rs
    - services/okr/src/dsl/parser.rs
    - services/okr/src/dsl/resolvers/mod.rs
    - services/okr/src/dsl/resolvers/proj_resolver.rs
    - services/okr/src/dsl/resolvers/inv_resolver.rs
    - services/okr/src/dsl/resolvers/hr_resolver.rs
    - services/okr/src/dsl/resolvers/learn_resolver.rs
    - services/okr/src/dsl/resolvers/custom_sql_resolver.rs
    - services/okr/src/dsl/metric_whitelist.rs
    - services/okr/src/handlers/dsl_routes.rs
    - services/okr/src/audit/dsl_events.rs
    - services/okr/tests/dsl_module_enum_cardinality_test.rs
    - services/okr/tests/dsl_agg_enum_cardinality_test.rs
    - services/okr/tests/dsl_proj_resolver_test.rs
    - services/okr/tests/dsl_metric_whitelist_test.rs
    - services/okr/tests/dsl_custom_sql_dual_sign_test.rs
    - services/okr/tests/dsl_audit_emission_test.rs

  modified_files:
    - services/okr/src/krs.rs

  allowed_tools:
    - file_read: services/{okr,proj,inv,hr,learn}/**
    - file_write: services/okr/{src,tests,migrations}/**
    - bash: cd services/okr && cargo test dsl

  disallowed_tools:
    - bypass metric whitelist (per DEC-1983)
    - custom_sql without dual-sign (per DEC-1983)

effort_hours: 10
sub_tasks:
  - "0.4h: 0003_progress_source.sql"
  - "0.4h: dsl/mod.rs"
  - "0.7h: parser.rs"
  - "0.5h: resolvers/mod.rs"
  - "0.6h: proj_resolver.rs"
  - "0.6h: inv_resolver.rs"
  - "0.6h: hr_resolver.rs"
  - "0.5h: learn_resolver.rs"
  - "0.8h: custom_sql_resolver.rs (dual-sign gate)"
  - "0.5h: metric_whitelist.rs"
  - "0.5h: handlers/dsl_routes.rs"
  - "0.4h: audit/dsl_events.rs"
  - "3.0h: tests — 6 test files"
  - "0.5h: docs"

risk_if_skipped: "Without DSL, all KR progress entered manually → check-in fatigue. Without DEC-1983 whitelist, KR DSL can SELECT anything (RLS leak). Without DEC-1985 custom_sql dual-sign, junior staff embeds arbitrary SQL."
---

## §1 — Description (BCP-14 normative)

The OKR service **MUST** ship progress_source DSL at `services/okr/src/dsl/` parsing JSONB queries against 5 modules with metric whitelist + custom_sql dual-sign gate, 4 memory audit kinds.

1. **MUST** validate `dsl_module` against closed enum per DEC-1981, `dsl_agg` per DEC-1982.

2. **MUST** define schema at migration `0003`:
   ```sql
   ALTER TABLE okr_krs ADD COLUMN progress_source_jsonb JSONB;
   ALTER TABLE okr_krs ADD COLUMN progress_source_last_resolved_at TIMESTAMPTZ;

   CREATE TABLE okr_custom_sql_approvals (
     kr_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     sql_text TEXT NOT NULL,
     cfo_signed_by UUID NOT NULL,
     cfo_signed_at TIMESTAMPTZ NOT NULL,
     ceo_signed_by UUID NOT NULL,
     ceo_signed_at TIMESTAMPTZ NOT NULL,
     approved_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE okr_custom_sql_approvals ENABLE ROW LEVEL SECURITY;
   CREATE POLICY custom_sql_rls ON okr_custom_sql_approvals
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON okr_custom_sql_approvals FROM cyberos_app;
   ```

3. **MUST** parse DSL at `parser.rs::parse(jsonb) → DslQuery`:
   - Required fields: module, metric, agg
   - Optional: filter (key-value pairs), date_range
   - Reject unknown enum values.

4. **MUST** dispatch per module at `resolvers/mod.rs::resolve(dsl, tenant)`:
   - proj → proj_resolver (e.g. metric=issues_closed, agg=count, filter={status:done})
   - inv → inv_resolver (e.g. metric=invoice_amount_paid, agg=sum)
   - hr → hr_resolver (e.g. metric=member_count, agg=count, filter={status:active})
   - learn → learn_resolver (e.g. metric=courses_completed, agg=sum)
   - custom_sql → custom_sql_resolver (gated)

5. **MUST** enforce metric whitelist per DEC-1983 at `metric_whitelist.rs::is_allowed(module, metric)` — only pre-approved metrics; reject all others.

6. **MUST** gate custom_sql per DEC-1983 — execution requires `okr_custom_sql_approvals` row with both CFO + CEO signatures; same person can't sign both.

7. **MUST** emit 4 memory audit kinds per DEC-1985. PII per FR-MEMORY-111: resolved value SHA-256 hashed; module/metric ok.

8. **MUST** thread trace_id from set → resolve → audit.

9. **MUST NOT** bypass whitelist per DEC-1983.

10. **MUST NOT** execute custom_sql without dual-sign per DEC-1983.

---

## §2 — Why this design

**Why DSL not raw SQL (DEC-1980)?** Safety + portability — module-aware resolvers respect RLS automatically.

**Why whitelist (DEC-1983)?** Without whitelist, KR DSL becomes a side-channel for reading any tenant data (PII risk).

**Why custom_sql gate (DEC-1983)?** Escape hatch for edge cases but requires governance — dual sign is the discipline.

**Why JSONB storage (DEC-1984)?** Flexible schema; easy to extend with new agg / filter shapes.

---

## §3 — API contract

Sample DSL:
```json
{
  "module": "proj",
  "metric": "issues_closed",
  "agg": "count",
  "filter": {"status": "done", "label": "p0-customer"},
  "date_range": {"from": "2026-01-01", "to": "2026-03-31"}
}
```

Resolver returns: `{value: 47, resolved_at: "2026-05-17T02:00:00Z", source: "proj.issues_closed"}`.

Custom SQL approval flow:
```text
POST /v1/okr/krs/{id}/custom-sql/request   body: {sql_text}
POST /v1/okr/krs/{id}/custom-sql/cfo-sign
POST /v1/okr/krs/{id}/custom-sql/ceo-sign
```

---

## §4 — Acceptance criteria
1. **dsl_module enum cardinality 5**. 2. **dsl_agg enum cardinality 6**. 3. **Parser rejects unknown enums**. 4. **proj resolver works**. 5. **inv resolver works**. 6. **hr resolver works**. 7. **learn resolver works**. 8. **Metric whitelist enforced**. 9. **custom_sql requires dual-sign**. 10. **Same-person dual-sign rejected**. 11. **4 memory audit kinds emitted**. 12. **PII scrubbed (resolved value SHA256)**. 13. **RLS denies cross-tenant**. 14. **Trace_id preserved**. 15. **Resolvers respect module RLS**. 16. **Append-only approvals via REVOKE**. 17. **Filter key-value validated per module schema**. 18. **date_range respects timezone**. 19. **Resolution failure → sev-2 + null value**. 20. **last_resolved_at updated on each compute**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn proj_resolver_counts_closed_issues() {
    let ctx = TestContext::with_5_closed_issues().await;
    let dsl = json!({"module":"proj","metric":"issues_closed","agg":"count","filter":{"status":"done"}});
    let val = resolve(dsl, ctx.tenant).await;
    assert_eq!(val, 5);
}

#[tokio::test]
async fn whitelist_rejects_unknown_metric() {
    let dsl = json!({"module":"proj","metric":"private_field","agg":"count"});
    let r = resolve(dsl, ctx.tenant).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn custom_sql_requires_dual_sign() {
    let dsl = json!({"module":"custom_sql","sql":"SELECT count(*) FROM users"});
    let r = try_resolve(dsl, ctx.tenant).await;
    assert!(r.is_err());  // not approved
    ctx.approve_cfo(ctx.kr_id).await;
    let r2 = try_resolve(dsl, ctx.tenant).await;
    assert!(r2.is_err());  // CEO missing
    ctx.approve_ceo(ctx.kr_id).await;
    let r3 = try_resolve(dsl, ctx.tenant).await;
    assert!(r3.is_ok());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-OKR-001.
**Downstream:** FR-OKR-004 (auto-progress cron uses resolver).
**Cross-module:** FR-PROJ-013, FR-INV-009, FR-HR-008, FR-LEARN-001 (data sources), FR-AUTH-101 (CFO/CEO roles), FR-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| DSL parse error | parser | 400 | fix JSON |
| Unknown module | enum | 400 | use valid |
| Unknown metric | whitelist | 400 | use whitelisted |
| Custom SQL without approval | gate | 403 | get dual-sign |
| Resolution timeout | retry 1x | null + sev-2 | inherent |
| Module data unavailable | catch | null + sev-2 | inherent |
| Cross-tenant resolution | RLS | 0 rows | inherent |
| date_range invalid | validate | 400 | fix |
| Filter key not in schema | reject | 400 | fix filter |
| Same-person dual-sign | validator | 403 | different signer |

## §11 — Implementation notes
- §11.1 Each resolver implements trait `Resolver { async fn resolve(&self, dsl) -> Result<Value> }`.
- §11.2 Metric whitelist per-module is hardcoded constants; reviewed in PR.
- §11.3 Custom SQL approved sql_text is immutable (REVOKE UPDATE); change = new approval.
- §11.4 memory audit body: kr_id, module, metric, agg; resolved_value SHA256.
- §11.5 Auto-progress cron (FR-OKR-004) calls resolve() and writes to current_value.

---

*End of FR-OKR-003 spec.*
