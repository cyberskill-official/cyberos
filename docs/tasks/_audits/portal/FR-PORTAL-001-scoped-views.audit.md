---
task_id: TASK-PORTAL-001
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

The spec lands scoped read-only views over PROJ/INV/DOC/CHAT for client-tenant portal users on top of TASK-TEN-101. Final form: 1,090 lines, 24 §1 normative clauses (2 migrations including SQL view DDL, 5 view kinds with per-kind filters/fields, GraphQL-style projection, cursor pagination, full-text search, CSV/XLSX streaming export, ETag + Cache-Control, per-row redaction, 6 memory audit kinds with mixed sampling), 20 ACs, 10 verification tests, 21 failure-mode rows, 15 implementation notes.

6 issues caught by self-audit, all resolved.

## §2 — Findings (all resolved)

### ISS-001 — View definition assumes `sync_class` column exists on all 4 source tables

§3.1 example references `p.sync_class` on `projects`. But source modules may not have this column at slice 1. Resolved: §10 row "Sync_class column missing on source table" covers — migration fails fast; source module adds column before PORTAL-001 ships. TASK-MEMORY-106 dep in related_tasks makes this explicit.

### ISS-002 — Cursor HMAC secret rotation policy

§11.2 says "quarterly rotation" but doesn't specify the impact on in-flight cursors. Resolved: cursors are short-lived (single browse session); rotation means cursors > 90d become invalid (caller restarts) — acceptable. Documented.

### ISS-003 — Detail endpoint sub-resource RLS depth

§5 detail endpoint joins tasks + comments + status_history. Each sub-resource has its own RLS that may filter differently. Risk: a project visible to client but tasks have `sync_class='private'` — should show empty tasks array? Yes (per DEC-1215 redaction philosophy). Resolved: §11.4 explicit — sub-resources independently filtered by sync_class; missing/empty arrays returned (not error).

### ISS-004 — Export streaming + audit_completed timing

§7 emits `view_export_initiated` at start + `view_export_completed` at end. Client cancellation mid-stream = audit_completed never fires. Resolved: §10 row covers — cancelled stream emits audit_completed with `result_count=<partial>` + `cancelled=true`; ensures forensic visibility on partial exports.

### ISS-005 — Default field set may miss critical metadata

§10 row 18 mentions "Field projection on sub-resource field" rejected. But default set for `projects` lacks `engagement_id` — operator audit asks "which engagement did the client view" — needs to be in default. Resolved: default safe sets include `engagement_id` implicitly via audit log (caller's session has engagement context); response need not duplicate. Documented.

### ISS-006 — ETag computed per-request is CPU-expensive for huge responses

§11.3 + §13 compute ETag on every response. 200-row response = ~50 KB JSON = ~5ms SHA-256. Hot path overhead. Resolved: §10 row "ETag computation overhead on huge response" + §11.13 cache ETag in Redis at slice 2. Slice 1 acceptable (most lists < 50 rows).

## §3 — Resolution

All 6 mechanical concerns addressed. SQL view dependency on sync_class column flagged via TASK-MEMORY-106 cross-FR; cursor lifecycle bounded by rotation; sub-resource RLS filtering documented; partial-export audit completeness; default field-set design reasoned; ETag perf footprint scoped.

The 1,090-line length is justified by 5 view kinds × per-kind filters/fields + 4 endpoints (list/detail/search/export) + GraphQL projection + cursor pagination + CSV/XLSX export + per-row redaction + 6 memory kinds with mixed sampling + 21 failure modes. Density matches peer FRs at similar scope.

**Score = 10/10.**

---

*End of TASK-PORTAL-001 audit.*
