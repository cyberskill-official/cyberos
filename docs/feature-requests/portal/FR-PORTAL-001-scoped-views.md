---
id: FR-PORTAL-001
title: "PORTAL scoped read-only views — PROJ/INV/DOC/CHAT filtered by Engagement membership + sync_class=client-visible + per-row redaction + GraphQL-style projection"
module: PORTAL
priority: MUST
status: draft
verify: T
phase: P4
milestone: P4 · slice 1
slice: 1
owner: Stephen Cheng (CCO)
created: 2026-05-17
shipped: null
brain_chain_hash: null
related_frs: [FR-PORTAL-002, FR-PORTAL-003, FR-PORTAL-004, FR-PORTAL-005, FR-PORTAL-006, FR-PORTAL-008, FR-TEN-101, FR-AUTH-004, FR-AUTH-101, FR-PROJ-001, FR-INV-001, FR-DOC-001, FR-CHAT-005, FR-AI-003, FR-BRAIN-111, FR-BRAIN-106, FR-OBS-005]
depends_on: [FR-TEN-101]
blocks: [FR-PORTAL-007, FR-PORTAL-008]

source_pages:
  - website/docs/modules/portal.html#scoped-views
  - https://gdpr.eu/article-25-data-protection-by-design/
  - https://datatracker.ietf.org/doc/html/rfc7235  # HTTP auth framework

source_decisions:
  - DEC-1200 2026-05-17 — Client tenants see read-only projection of 4 module surfaces (PROJ, INV, DOC, CHAT) filtered by Engagement membership AND sync_class='client-visible' per FR-BRAIN-106; never write through PORTAL views (writes go through dedicated FR-PORTAL-006 workflows)
  - DEC-1201 2026-05-17 — Closed enum `portal_view_kind` = {projects, invoices, documents, channels, calendar}; CI cardinality test asserts 5
  - DEC-1202 2026-05-17 — View materialisation: live SQL views with RLS predicate joining `engagement_memberships` + per-row `sync_class` check; NO denormalised cache (correctness > performance at slice 1; cache at slice 2)
  - DEC-1203 2026-05-17 — Per-row field redaction: rows of class `client-visible-redacted` return with internal fields (notes, internal_status, assignee_internal) blanked
  - DEC-1204 2026-05-17 — GraphQL-style projection: client requests `?fields=id,title,status` returns only those fields; defaults to predefined safe set per view kind
  - DEC-1205 2026-05-17 — Endpoint shape: `GET /v1/portal/views/{view_kind}?engagement_id=...&filters=...&fields=...&cursor=...`; cursor-based pagination at 50/page
  - DEC-1206 2026-05-17 — Search within view: full-text search via PostgreSQL `tsvector` on per-view searchable columns; scoped by RLS
  - DEC-1207 2026-05-17 — Single-row detail endpoint: `GET /v1/portal/views/{view_kind}/{id}` returns the row PLUS related sub-resources (e.g., project detail includes child tasks, comments) filtered by sync_class
  - DEC-1208 2026-05-17 — All view-read events emit `portal.view_read` audit row at sev-3 (sampled at 1% via FR-OBS-006); detail-row reads at sev-2 (always emitted — caller saw specific resource)
  - DEC-1209 2026-05-17 — Cross-Engagement access: caller member of multiple Engagements MUST switch context via `engagement_id` query param; views never auto-aggregate across Engagements (UX confusion + audit unclear)
  - DEC-1210 2026-05-17 — Rate limit 600 read/min/caller (10/sec sufficient for browsing); excess → 429
  - DEC-1211 2026-05-17 — Per-view kind-specific filters: `projects` supports status/assignee/date_range; `invoices` supports status/date_range/amount_range; `documents` supports tag/type/date_range; `channels` supports tag/last_message_date
  - DEC-1212 2026-05-17 — Stale-read tolerance: views read against the canonical Postgres tables (no eventual consistency); reads MAY observe in-flight writes that haven't yet emitted audit (rare; documented)
  - DEC-1213 2026-05-17 — Exported view (CSV/Excel) via `GET /v1/portal/views/{view_kind}/export?format=csv|xlsx` per FR-PORTAL-008-derivative streaming; ≤ 10k rows per export; over-limit → use DSAR
  - DEC-1214 2026-05-17 — BRAIN audit kinds: portal.view_read, portal.view_detail_read, portal.view_search_executed, portal.view_export_initiated, portal.view_export_completed, portal.view_redaction_applied
  - DEC-1215 2026-05-17 — Per-row redaction is field-set-replacement (NOT row-omission); redacted rows STILL appear in list but with masked fields per DEC-1203
  - DEC-1216 2026-05-17 — Tenant default sync_class for new rows: 'private' (default); promotion to 'client-visible' is explicit user action in each source module
  - DEC-1217 2026-05-17 — Cache-Control: response carries `Cache-Control: private, max-age=30` (30-sec client cache OK; per-user RLS makes shared cache impossible)
  - DEC-1218 2026-05-17 — ETag: SHA-256-16 of canonical-JSON response body; clients use `If-None-Match` for 304 responses (reduces bandwidth)
  - DEC-1219 2026-05-17 — Cursor pagination uses base64(JSON{last_id, last_sort_value}); opaque to clients; stable across requests

build_envelope:
  language: rust 1.81
  service: cyberos/services/portal/
  new_files:
    - services/portal/migrations/0014_portal_view_definitions.sql      # SQL view DDL per view_kind
    - services/portal/migrations/0015_portal_view_read_log.sql         # detail-row read audit log
    - services/portal/src/views/mod.rs                                 # orchestrator
    - services/portal/src/views/projects.rs                            # /views/projects + detail
    - services/portal/src/views/invoices.rs                            # /views/invoices + detail
    - services/portal/src/views/documents.rs                           # /views/documents + detail
    - services/portal/src/views/channels.rs                            # /views/channels + detail
    - services/portal/src/views/calendar.rs                            # /views/calendar (slice-2 stub)
    - services/portal/src/views/projection.rs                          # field-set projection (GraphQL-style)
    - services/portal/src/views/redaction.rs                           # per-row redaction logic
    - services/portal/src/views/search.rs                              # tsvector search
    - services/portal/src/views/pagination.rs                          # cursor encode/decode
    - services/portal/src/views/export.rs                              # CSV + XLSX export
    - services/portal/src/audit/view_events.rs                         # 6 BRAIN row builders
    - services/portal/src/handlers/view_routes.rs                      # REST routes
    - services/portal/tests/view_projects_list_test.rs
    - services/portal/tests/view_projects_filtered_by_sync_class_test.rs
    - services/portal/tests/view_engagement_isolation_test.rs
    - services/portal/tests/view_field_projection_test.rs
    - services/portal/tests/view_per_row_redaction_test.rs
    - services/portal/tests/view_pagination_test.rs
    - services/portal/tests/view_search_test.rs
    - services/portal/tests/view_export_csv_test.rs
    - services/portal/tests/view_export_size_cap_test.rs
    - services/portal/tests/view_detail_with_subresources_test.rs
    - services/portal/tests/view_etag_caching_test.rs
    - services/portal/tests/view_rate_limit_test.rs
    - services/portal/tests/view_kind_enum_cardinality_test.rs
    - services/portal/tests/view_cross_engagement_blocked_test.rs
    - services/portal/tests/view_audit_emission_test.rs

  modified_files:
    - services/portal/src/lib.rs                                       # mount view routes
    - services/portal/Cargo.toml                                       # +rust_xlsxwriter for export

  allowed_tools:
    - file_read: services/portal/**
    - file_read: services/{proj,inv,doc,chat}/src/**
    - file_write: services/portal/{src,tests,migrations}/**
    - bash: cd services/portal && cargo test views

  disallowed_tools:
    - allow writes through view endpoints (per DEC-1200 — read-only)
    - return rows with sync_class!='client-visible' or 'client-visible-redacted' (per DEC-1200)
    - auto-aggregate across Engagements (per DEC-1209)
    - cache responses across users (per DEC-1217 — per-user only)
    - export > 10k rows (per DEC-1213 — use DSAR)

effort_hours: 12
sub_tasks:
  - "0.5h: 0014_portal_view_definitions.sql + RLS predicates joining engagement_memberships + sync_class filter"
  - "0.4h: 0015_portal_view_read_log.sql + RLS"
  - "0.5h: views/mod.rs + closed enum + dispatcher"
  - "0.8h: views/projects.rs + detail with sub-resources"
  - "0.7h: views/invoices.rs"
  - "0.7h: views/documents.rs"
  - "0.7h: views/channels.rs"
  - "0.3h: views/calendar.rs (slice-2 stub)"
  - "0.5h: views/projection.rs (field-set filter)"
  - "0.5h: views/redaction.rs (per-row mask)"
  - "0.6h: views/search.rs (tsvector + RLS-aware)"
  - "0.4h: views/pagination.rs (cursor encode/decode)"
  - "0.7h: views/export.rs (CSV + XLSX streaming + 10k cap)"
  - "0.4h: audit/view_events.rs (6 builders)"
  - "0.5h: handlers/view_routes.rs"
  - "2.5h: tests — 15 test files"
  - "0.5h: integration smoke against seeded multi-tenant fixture"
  - "0.3h: wire-up lib.rs"

risk_if_skipped: "Without scoped views, PORTAL is a brand pack + SSO + Genie chat with nothing to browse — clients can sign in but see a blank portal. The 4 view surfaces (projects/invoices/documents/channels) ARE the client experience. Without DEC-1200's read-only constraint, write paths bypass FR-PORTAL-006 workflow audit. Without DEC-1202's RLS-based filtering, client sees all tenant data including cross-Engagement leaks. Without DEC-1203's per-row redaction, internal fields (notes, internal-status) leak to client. Without DEC-1209's per-Engagement context switch, multi-Engagement users see ambiguous aggregations. Without DEC-1213's export size cap, 1M-row exports DOS the portal. Without DEC-1216's default sync_class='private', every internal row becomes client-visible by default → catastrophic leak. The 12h effort lands the client-portal data surface that, combined with PORTAL-002 brand + PORTAL-003 IdP + PORTAL-005 Genie, completes the white-label SaaS experience."
---

## §1 — Description (BCP-14 normative)

The PORTAL service **MUST** ship scoped read-only views at `services/portal/src/views/` over the PROJ/INV/DOC/CHAT modules, filtered by Engagement membership AND sync_class='client-visible' per FR-BRAIN-106, with per-row redaction, field projection, cursor pagination, full-text search, CSV/XLSX export, and 6 BRAIN audit kinds.

1. **MUST** define the closed `portal_view_kind` enum at migration `0014`: `('projects','invoices','documents','channels','calendar')` per DEC-1201. CI cardinality test asserts 5. The `calendar` view is a slice-2 stub returning 501.

2. **MUST** create SQL views per `portal_view_kind` via migration `0014`. Each view is a `CREATE VIEW portal_view_<kind> AS SELECT ... FROM <source_table> WHERE sync_class IN ('client-visible','client-visible-redacted') AND tenant_id = current_setting('auth.tenant_id')::uuid AND engagement_id IN (SELECT engagement_id FROM engagement_memberships WHERE subject_id = current_setting('auth.subject_id')::uuid)`. RLS predicate inherits from each source table's RLS plus the engagement-membership join.

3. **MUST** define `portal_view_read_log` at migration `0015`: `(id BIGSERIAL PRIMARY KEY, tenant_id UUID NOT NULL, engagement_id UUID NOT NULL, caller_subject_id UUID NOT NULL, view_kind portal_view_kind NOT NULL, resource_id UUID, action TEXT NOT NULL CHECK (action IN ('list','detail','search','export_initiated','export_completed')), filter_hash16 TEXT, result_count INT, trace_id CHAR(32), occurred_at TIMESTAMPTZ NOT NULL DEFAULT now())`. Append-only per AUTHORING.md rule 12. RLS-scoped.

4. **MUST** expose list endpoint `GET /v1/portal/views/{view_kind}?engagement_id=...&filters=...&fields=...&cursor=...&limit=...` per DEC-1205. Handler:
    - Validates JWT + extracts caller_subject_id + tenant_id.
    - Validates engagement_id is in caller's `engagement_memberships`; else 403 + `engagement_access_denied`.
    - Validates `view_kind` is in closed enum.
    - Applies per-view-kind filters per DEC-1211.
    - Applies field projection per §1 #10.
    - Returns rows from the SQL view (RLS auto-applies sync_class + engagement filter).
    - Paginate via cursor (default limit 50; max 200).
    - Emit `portal.view_read` (sev-3 sampled 1%).
    - ETag + Cache-Control per §1 #13.

5. **MUST** expose detail endpoint `GET /v1/portal/views/{view_kind}/{id}` per DEC-1207. Handler:
    - Same auth + engagement check.
    - Returns single row PLUS related sub-resources:
      - `projects/{id}` → includes tasks, comments, status_history (all filtered by sync_class).
      - `invoices/{id}` → includes line_items, payment_history.
      - `documents/{id}` → includes versions, comments.
      - `channels/{id}` → includes recent messages (last 50, also sync_class filtered).
    - Emit `portal.view_detail_read` (sev-2 — always, not sampled).

6. **MUST** expose search endpoint `POST /v1/portal/views/{view_kind}/search` per DEC-1206 with body `{ query: <string>, filters?: {...}, fields?: [...], cursor?: ... }`. Handler:
    - Uses PostgreSQL `tsvector` index on per-view searchable columns (e.g. project name + description).
    - Combines with view's existing RLS filters.
    - Returns ranked results.
    - Emit `portal.view_search_executed` (sev-3 — sampled).

7. **MUST** expose export endpoint `GET /v1/portal/views/{view_kind}/export?format=csv|xlsx&filters=...&fields=...` per DEC-1213. Handler:
    - Same auth + filter.
    - Streams response (chunked transfer encoding); rows fetched + serialised in 100-row batches.
    - Caps at 10,000 rows; over-cap returns 413 + `export_too_large; use_dsar`.
    - Content-Type `text/csv` or `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet`.
    - Emit `portal.view_export_initiated` at start + `portal.view_export_completed` at end (both sev-2 — material data movement).

8. **MUST** apply per-row redaction per DEC-1203 + DEC-1215. Rows with `sync_class='client-visible-redacted'` returned with internal-only fields blanked:
    - `projects`: blank `internal_notes`, `assignee_internal_id`, `internal_priority`.
    - `invoices`: blank `internal_notes`, `cost_breakdown`.
    - `documents`: blank `internal_summary`, `internal_classification`.
    - `channels`: blank `internal_pin_count`, `internal_archive_reason`.
   Redaction tracked: each redacted row emits sampled `portal.view_redaction_applied` informational row (sev-3, sampled 5%) so operators can monitor what's being redacted.

9. **MUST** support per-view kind-specific filters per DEC-1211. Filter param format `?filters=<base64-JSON>` decodes to:
    - `projects`: `{ status: ["active","pending"], assignee_external_id: "...", date_range: {from: "...", to: "..."} }`.
    - `invoices`: `{ status: ["sent","paid","overdue"], date_range: {...}, amount_range: {min: ..., max: ...} }`.
    - `documents`: `{ tag: ["contract","report"], type: ["pdf","docx"], date_range: {...} }`.
    - `channels`: `{ tag: ["project-X"], last_message_date_range: {...} }`.
   Invalid filter keys for the view kind → 400 + `invalid_filter_key` + valid_keys array.

10. **MUST** apply GraphQL-style field projection per DEC-1204. The `fields` query param decodes to a comma-separated list (or JSON array via base64). Handler:
    - If absent: returns the default safe set per view kind.
    - If present: validates each field is in the allowed set; rejects unknown fields with 400 + `unknown_field`.
    - SELECT only those fields from the view (avoids SELECT * cost on wide tables).
    - Default safe sets:
      - `projects`: `id, title, status, owner_external_id, created_at, last_activity_at`.
      - `invoices`: `id, invoice_number, status, amount, currency, issued_at, due_at`.
      - `documents`: `id, title, type, size_bytes, tag, created_at, updated_at`.
      - `channels`: `id, name, tag, last_message_at, member_count`.

11. **MUST** enforce per-Engagement context per DEC-1209. `engagement_id` query param REQUIRED on every endpoint. Without it → 400 + `engagement_id_required`. Caller member of N Engagements switches via UI engagement-picker; views never aggregate.

12. **MUST** apply cursor pagination per DEC-1219. The cursor is base64(JSON `{last_id: uuid, last_sort_value: <value>}`). Pagination handler:
    - Validates cursor signature (HMAC-signed to prevent forgery).
    - Applies `WHERE (sort_value, id) > (cursor.last_sort_value, cursor.last_id)` (keyset pagination per AUTHORING.md rule 16 derivative).
    - Returns `{ rows: [...], next_cursor: <base64> | null }`.

13. **MUST** include ETag + Cache-Control headers per DEC-1217 + DEC-1218. ETag = SHA-256 truncated 16 hex of canonical-JSON response body; `If-None-Match` match → 304. Cache-Control `private, max-age=30` allows browser-side cache; per-user RLS prevents shared-cache leak.

14. **MUST** rate-limit at 600 reads/min/caller per DEC-1210 + AUTHORING.md §8.2d derivative. Exceeded → 429 + Retry-After.

15. **MUST** emit 6 BRAIN audit row kinds per DEC-1214:
    - `portal.view_read` (sev-3 — high-volume; sampled 1%)
    - `portal.view_detail_read` (sev-2 — material; always emitted)
    - `portal.view_search_executed` (sev-3 — sampled 5%)
    - `portal.view_export_initiated` (sev-2 — material data movement)
    - `portal.view_export_completed` (sev-2 — paired with initiated)
    - `portal.view_redaction_applied` (sev-3 — sampled 5%)

16. **MUST** PII-scrub per AUTHORING.md rule 18. Audit rows carry `filter_hash16` + `resource_id` (UUID; non-PII per FR-PORTAL-004 §1 #18 rationale); raw filter values + result content NOT in chain.

17. **MUST** thread trace_id end-to-end per AUTHORING.md rule 22-24.

18. **MUST** stream exports in chunked-transfer encoding per §1 #7. Memory footprint bounded — no full-result-set buffering.

19. **MUST** enforce read-only — view endpoints never accept POST/PUT/PATCH/DELETE (returns 405). Writes route through FR-PORTAL-006 workflows.

20. **MUST** validate `sync_class` filter is enforced at the SQL view level per DEC-1202 + FR-BRAIN-106. Handler does NOT trust client-supplied sync_class filter; the view's predicate is the gate.

21. **MUST NOT** auto-aggregate across Engagements per DEC-1209. Each request scoped to one engagement_id.

22. **MUST NOT** allow shared HTTP cache (proxies, CDNs) per DEC-1217. `Cache-Control: private` mandatory; CDN edges MUST NOT cache.

23. **MUST NOT** return rows with `sync_class IN ('private','team-internal')` per FR-BRAIN-106. View definition enforces; handler-side check defense-in-depth.

24. **SHOULD** observe per-Engagement read volume via OTel histogram `portal_view_read_total{engagement_id, view_kind}`.

---

## §2 — Why this design (rationale for humans)

**Why read-only at slice 1 (§1 #19, DEC-1200)?** Read-only is simpler to make correct: no transaction complexity, no validation, no cascade revocation. Writes via PORTAL-006 workflows give us audit-trail control (client-initiated request → CHAT thread → CyberOS-side action). Read-only views + write workflows = clean separation of read scale from write trust.

**Why sync_class='client-visible' enforced at SQL view (§1 #2, DEC-1202)?** Defense-in-depth — handler-only enforcement leaks if handler bug. SQL view enforcement is the floor; handler is the ceiling. Both must agree; either alone catches bugs in the other.

**Why per-row redaction (vs row omission) (§1 #8, DEC-1215)?** Omitting a row "blinds" the client — they don't know the row exists. Showing a redacted row is honest: "we have data on X but you can't see all of it". Better UX + same data security.

**Why per-Engagement context required (§1 #11, DEC-1209)?** Multi-Engagement caller seeing one mega-list is confusing UX (which client is which?) + auditor's "who saw what" becomes unclear (one query → 50 Engagement rows = which one was the user actually looking at?). Forcing per-Engagement scope makes the audit answerable.

**Why CSV + XLSX export but capped 10k (§1 #7, DEC-1213)?** Clients legitimately want offline copies of their data (compliance backup, accounting reconciliation). CSV/XLSX are universal formats. 10k cap balances UX (covers 95% of legitimate needs) vs DOS prevention. DSAR (FR-PORTAL-008) handles full-archive exports.

**Why GraphQL-style field projection (§1 #10, DEC-1204)?** Mobile clients on metered connections want minimal payload. Full row = 5-20 KB; projected row = 200-500 bytes. 90%+ bandwidth saving. Standard pattern (GraphQL, Sparse Fieldsets in JSON:API).

**Why cursor pagination over offset (§1 #12)?** Offset breaks at scale (slow scan past `LIMIT offset N`); cursor uses keyset (index lookup). Standard pattern for large datasets.

**Why per-Engagement filter at SQL view + RLS (§1 #2)?** Combined enforcement: tenant_id from current_setting (RLS) + engagement_id from join. Single-condition would leak if either piece is misconfigured.

**Why default sync_class='private' (DEC-1216)?** Default-deny is the security default. A new row appearing in a source module is invisible to clients until explicitly promoted. Default-allow would leak every internal note.

**Why sampled view-read audit (§1 #15, DEC-1208)?** Browsing = high-volume reads (every page render = 10s of view calls). Sampling at 1% gives statistical observability without exploding the chain. Detail reads (always emitted) + search (sampled 5%) provide the audit signal for specific resource access.

**Why detail reads always emitted (§1 #15)?** Detail = "user saw row X". Forensic: "did the client view document 12345 before suing?" must be answerable. Always-emit guarantees.

---

## §3 — API contract

### 3.1 Postgres schema

```sql
-- 0014_portal_view_definitions.sql
CREATE TYPE portal_view_kind AS ENUM ('projects','invoices','documents','channels','calendar');

-- Example: projects view
CREATE VIEW portal_view_projects AS
SELECT p.id, p.tenant_id, p.engagement_id, p.title, p.status,
       p.owner_external_id, p.created_at, p.last_activity_at,
       CASE WHEN p.sync_class = 'client-visible' THEN p.internal_notes ELSE NULL END AS internal_notes,
       CASE WHEN p.sync_class = 'client-visible' THEN p.assignee_internal_id ELSE NULL END AS assignee_internal_id,
       p.sync_class
FROM projects p
WHERE p.sync_class IN ('client-visible','client-visible-redacted')
  AND p.tenant_id = current_setting('auth.tenant_id')::uuid
  AND p.engagement_id IN (
    SELECT engagement_id FROM engagement_memberships
    WHERE subject_id = current_setting('auth.subject_id')::uuid
  );

-- Analogous CREATE VIEW for invoices, documents, channels.
-- All views are RLS-aware via the WHERE clauses (no separate RLS policy needed; the view IS the policy).

-- 0015_portal_view_read_log.sql
CREATE TABLE portal_view_read_log (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  engagement_id UUID NOT NULL,
  caller_subject_id UUID NOT NULL,
  view_kind portal_view_kind NOT NULL,
  resource_id UUID,
  action TEXT NOT NULL CHECK (action IN ('list','detail','search','export_initiated','export_completed')),
  filter_hash16 TEXT,
  result_count INT,
  trace_id CHAR(32),
  occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_view_read_caller ON portal_view_read_log(caller_subject_id, occurred_at DESC);
ALTER TABLE portal_view_read_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_view_read_log_rls ON portal_view_read_log
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON portal_view_read_log FROM cyberos_app;
```

### 3.2 REST endpoints

```text
GET    /v1/portal/views/{view_kind}                              (list; cursor pagination)
GET    /v1/portal/views/{view_kind}/{id}                          (detail + sub-resources)
POST   /v1/portal/views/{view_kind}/search                        (search; cursor pagination)
GET    /v1/portal/views/{view_kind}/export?format=csv|xlsx        (streaming; 10k cap)
```

All require `engagement_id` query param.

---

## §4 — Acceptance criteria

1. **portal_view_kind cardinality** — enum = exactly `{projects, invoices, documents, channels, calendar}`.
2. **List filtered by sync_class** — projects with sync_class='private' NOT in response; only 'client-visible' or 'client-visible-redacted'.
3. **Engagement isolation** — caller member of Eng X + Y; list query for X returns ONLY X's rows; Y rows absent.
4. **Field projection** — `?fields=id,title` returns only those fields; `?fields=internal_notes` rejected with 400 if not in safe set.
5. **Per-row redaction** — row with `sync_class='client-visible-redacted'` returns with `internal_notes=NULL`.
6. **Cursor pagination** — first page returns next_cursor; cursor returns next page; round-trip stable.
7. **Search** — POST search returns ranked results scoped by RLS.
8. **CSV export** — `?format=csv` returns text/csv stream with headers row.
9. **XLSX export** — `?format=xlsx` returns valid xlsx binary.
10. **Export 10k cap** — 10001 rows → 413 + `export_too_large`.
11. **Detail with sub-resources** — `projects/{id}` returns row + tasks + comments + status_history.
12. **engagement_id required** — list without `engagement_id` → 400 + `engagement_id_required`.
13. **Cross-Engagement engagement_id** — list with engagement_id NOT in membership → 403.
14. **ETag 304** — second list with `If-None-Match` → 304.
15. **Cache-Control private** — header on every 200 response.
16. **Rate limit 600/min** — 601st read → 429.
17. **Read-only** — POST/PUT/DELETE on view endpoint → 405.
18. **Calendar slice-2 stub** — `/v1/portal/views/calendar` → 501.
19. **6 BRAIN audit kinds emitted** — full browse + detail + search + export + redaction lifecycle covers all 6.
20. **Filter hash in audit** — filter_hash16 = SHA256-16 of canonical filter JSON; raw filter NOT in chain.

---

## §5 — Verification

### 5.1 `view_projects_list_test.rs`

```rust
#[tokio::test]
async fn list_returns_only_client_visible() {
    let ctx = TestContext::with_engagement_subject().await;
    ctx.seed_project(SyncClass::Private, "private-project").await;
    ctx.seed_project(SyncClass::ClientVisible, "shared-project").await;

    let r = ctx.get_view("projects", ctx.eng_id).await;
    let body: serde_json::Value = r.json().await.unwrap();
    let titles: Vec<&str> = body["rows"].as_array().unwrap().iter().filter_map(|r| r["title"].as_str()).collect();
    assert!(titles.contains(&"shared-project"));
    assert!(!titles.contains(&"private-project"));
}
```

### 5.2 `view_engagement_isolation_test.rs`

```rust
#[tokio::test]
async fn engagement_y_rows_not_returned_when_filtering_x() {
    let ctx = TestContext::with_subject_in_two_engagements().await;
    ctx.seed_project_in_eng(ctx.eng_x, SyncClass::ClientVisible, "x-proj").await;
    ctx.seed_project_in_eng(ctx.eng_y, SyncClass::ClientVisible, "y-proj").await;

    let r = ctx.get_view("projects", ctx.eng_x).await;
    let titles: Vec<String> = ctx.extract_titles(r).await;
    assert!(titles.contains(&"x-proj".into()));
    assert!(!titles.contains(&"y-proj".into()));
}
```

### 5.3 `view_per_row_redaction_test.rs`

```rust
#[tokio::test]
async fn redacted_rows_have_internal_fields_nulled() {
    let ctx = TestContext::with_engagement_subject().await;
    ctx.seed_project_with_redacted_class("redacted-proj", "secret note").await;
    let r = ctx.get_view("projects", ctx.eng_id).await;
    let row = ctx.find_row(r, "title", "redacted-proj").await;
    assert!(row["internal_notes"].is_null());
    assert_eq!(row["title"], "redacted-proj");

    let audit = ctx.brain_rows().await;
    assert!(audit.iter().any(|r| r.kind == "portal.view_redaction_applied"));
}
```

### 5.4 `view_field_projection_test.rs`

```rust
#[tokio::test]
async fn fields_param_limits_response_shape() {
    let ctx = TestContext::with_engagement_subject().await;
    ctx.seed_project(SyncClass::ClientVisible, "p1").await;
    let r = ctx.get_view_with_fields("projects", ctx.eng_id, "id,title").await;
    let row = ctx.first_row(r).await;
    assert!(row.get("id").is_some());
    assert!(row.get("title").is_some());
    assert!(row.get("status").is_none());

    let r2 = ctx.get_view_with_fields("projects", ctx.eng_id, "internal_notes").await;
    assert_eq!(r2.status(), 400);
}
```

### 5.5 `view_pagination_test.rs`

```rust
#[tokio::test]
async fn cursor_round_trip_stable() {
    let ctx = TestContext::with_engagement_subject().await;
    for i in 0..150 {
        ctx.seed_project(SyncClass::ClientVisible, &format!("p{i}")).await;
    }
    let r1 = ctx.get_view_limited("projects", ctx.eng_id, 50).await;
    let body1: serde_json::Value = r1.json().await.unwrap();
    assert_eq!(body1["rows"].as_array().unwrap().len(), 50);
    let cursor = body1["next_cursor"].as_str().unwrap();

    let r2 = ctx.get_view_cursor("projects", ctx.eng_id, cursor).await;
    let body2: serde_json::Value = r2.json().await.unwrap();
    assert_eq!(body2["rows"].as_array().unwrap().len(), 50);
}
```

### 5.6 `view_export_csv_test.rs`

```rust
#[tokio::test]
async fn csv_export_streams_with_headers() {
    let ctx = TestContext::with_engagement_subject().await;
    for i in 0..100 { ctx.seed_project(SyncClass::ClientVisible, &format!("p{i}")).await; }
    let r = ctx.get_export("projects", ctx.eng_id, "csv").await;
    assert_eq!(r.headers()["content-type"].to_str().unwrap(), "text/csv");
    let body = r.text().await.unwrap();
    let lines: Vec<&str> = body.lines().collect();
    assert!(lines[0].contains("id,title,status"));
    assert_eq!(lines.len(), 101);  // header + 100 rows
}
```

### 5.7 `view_export_size_cap_test.rs`

```rust
#[tokio::test]
async fn export_over_10k_rejected() {
    let ctx = TestContext::with_engagement_subject().await;
    for i in 0..10001 { ctx.seed_project_fast(SyncClass::ClientVisible, &format!("p{i}")).await; }
    let r = ctx.get_export("projects", ctx.eng_id, "csv").await;
    assert_eq!(r.status(), 413);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["error"], "export_too_large");
    assert_eq!(body["use_dsar"], true);
}
```

### 5.8 `view_detail_with_subresources_test.rs`

```rust
#[tokio::test]
async fn detail_includes_tasks_and_comments() {
    let ctx = TestContext::with_engagement_subject().await;
    let proj_id = ctx.seed_project(SyncClass::ClientVisible, "p1").await;
    ctx.seed_task_in_project(proj_id, SyncClass::ClientVisible, "t1").await;
    ctx.seed_comment_in_project(proj_id, SyncClass::ClientVisible, "c1").await;

    let r = ctx.get_view_detail("projects", proj_id, ctx.eng_id).await;
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["title"], "p1");
    assert_eq!(body["tasks"].as_array().unwrap().len(), 1);
    assert_eq!(body["comments"].as_array().unwrap().len(), 1);

    let audit = ctx.brain_rows().await;
    assert!(audit.iter().any(|r| r.kind == "portal.view_detail_read" && r.severity == 2));
}
```

### 5.9 `view_kind_enum_cardinality_test.rs`

```rust
#[tokio::test]
async fn view_kind_has_5_values() {
    let ctx = TestContext::new().await;
    let labels: Vec<String> = sqlx::query_scalar(
        "SELECT unnest(enum_range(NULL::portal_view_kind))::text"
    ).fetch_all(&ctx.pool).await.unwrap();
    let mut labels = labels; labels.sort();
    assert_eq!(labels, vec!["calendar","channels","documents","invoices","projects"]);
}
```

### 5.10 `view_etag_caching_test.rs`

```rust
#[tokio::test]
async fn etag_304_on_match() {
    let ctx = TestContext::with_engagement_subject().await;
    ctx.seed_project(SyncClass::ClientVisible, "p1").await;
    let r1 = ctx.get_view("projects", ctx.eng_id).await;
    let etag = r1.headers()["etag"].to_str().unwrap().to_owned();
    let r2 = ctx.get_view_with_if_none_match("projects", ctx.eng_id, &etag).await;
    assert_eq!(r2.status(), 304);
}
```

---

## §6 — Implementation skeleton

```rust
// services/portal/src/views/mod.rs
pub async fn list_handler(ctx: AppCtx, jwt: JwtClaims, view_kind: PortalViewKind,
                          engagement_id: Uuid, filters: Filters, fields: FieldSet,
                          cursor: Option<Cursor>, limit: u32) -> Result<ListResp, ViewError> {
    // Validations
    require_engagement_membership(&ctx, jwt.subject_id, engagement_id).await?;
    rate_limit_check(&ctx, jwt.subject_id).await?;
    filters.validate_for_kind(view_kind)?;
    fields.validate_for_kind(view_kind)?;

    // Set session vars for RLS
    let mut conn = ctx.pool.acquire().await?;
    sqlx::query("SET LOCAL auth.subject_id = $1").bind(jwt.subject_id).execute(&mut *conn).await?;
    sqlx::query("SET LOCAL auth.tenant_id = $1").bind(jwt.tenant_id).execute(&mut *conn).await?;

    let (rows, next_cursor) = match view_kind {
        PortalViewKind::Projects => projects::list(&mut conn, engagement_id, &filters, &fields, cursor, limit).await?,
        PortalViewKind::Invoices => invoices::list(/* ... */).await?,
        /* ... */
        PortalViewKind::Calendar => return Err(ViewError::SliceUnavailable),
    };

    // Apply per-row redaction
    let redacted_rows: Vec<_> = rows.into_iter()
        .map(|r| redaction::apply(view_kind, r))
        .collect();
    if redacted_rows.iter().any(|r| r.was_redacted) {
        emit_audit(&ctx, "portal.view_redaction_applied", json!({/*sampled*/})).await;
    }

    // Audit + ETag
    emit_audit(&ctx, "portal.view_read", json!({
        "engagement_id": engagement_id,
        "view_kind": view_kind,
        "filter_hash16": filters.hash16(),
        "result_count": redacted_rows.len(),
    })).await;

    let resp = ListResp { rows: redacted_rows, next_cursor };
    let etag = sha256_hex(&serde_json::to_vec(&resp)?)[..16].to_owned();
    Ok(ListResp { /* ... */ }.with_etag(etag).with_cache_control("private, max-age=30"))
}
```

---

## §7 — Dependencies

**Upstream (depends_on):**
- **FR-TEN-101** Self-serve signup — tenant + Engagement exists before views.

**Cross-module (related_frs):**
- **FR-PORTAL-002** Brand pack — view UI inherits brand.
- **FR-PORTAL-003** External IdP — caller_subject_id from IdP-auth JWT.
- **FR-PORTAL-004** SCIM deprovision — revoked sessions return 401.
- **FR-PORTAL-005** Branded Genie — uses same scope_grants pattern.
- **FR-PORTAL-006** Client-initiated workflows — write counterpart to read views.
- **FR-PORTAL-008** DSAR — full data export when > 10k rows needed.
- **FR-PROJ-001** Projects — source schema; sync_class filter joins here.
- **FR-INV-001** Invoices — source schema.
- **FR-DOC-001** Documents — source schema.
- **FR-CHAT-005** Channels — source schema; messages too.
- **FR-AUTH-101** RBAC — engagement_memberships table consumed.
- **FR-BRAIN-106** sync_class enforcement — view definitions consume.
- **FR-AI-003** BRAIN audit — 6 new kinds.
- **FR-BRAIN-111** PII scrubbing — filter hash only in chain.

**Downstream (blocks):**
- **FR-PORTAL-007** PWA — needs views to display.
- **FR-PORTAL-008** DSAR — references view shape for export.

---

## §8 — Example payloads

### 8.1 List response

```json
{
  "rows": [
    { "id": "0190f7c0-...", "title": "Q2 audit", "status": "in_progress",
      "owner_external_id": "alice@acme.com", "created_at": "2026-05-01T...",
      "last_activity_at": "2026-05-17T..." }
  ],
  "next_cursor": "eyJsYXN0X2lkIjoiMDE5MGY3YzAtLi4uIiwibGFzdF9zb3J0X3ZhbHVlIjoiMjAyNi0wNS0xNyJ9"
}
```

### 8.2 `portal.view_detail_read` BRAIN row

```json
{
  "kind": "portal.view_detail_read",
  "severity": 2,
  "tenant_id": "8a2f...",
  "actor_id": "user.subject.456",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T09:14:32.847Z",
  "payload": {
    "engagement_id": "0190...",
    "view_kind": "projects",
    "resource_id": "0190f7c0-..."
  }
}
```

---

## §9 — Open questions

All resolved for slice 1. Deferred:

- **Deferred:** Calendar view (slice 2).
- **Deferred:** Real-time view updates via SSE (slice 2).
- **Deferred:** Saved filter presets per user (slice 2).
- **Deferred:** Custom field configuration per tenant (slice 3).
- **Deferred:** Bulk operations (multi-row actions) — slice 3.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Engagement not in caller membership | join returns 0 | 403 + `engagement_access_denied` | Caller switches engagement |
| Unknown view_kind | enum validation | 400 + `unknown_view_kind` | Caller fixes URL |
| Unknown filter key | per-kind validation | 400 + `invalid_filter_key` + valid_keys | Caller fixes filter |
| Unknown field in projection | safe-set check | 400 + `unknown_field` | Caller fixes fields param |
| Cursor signature invalid (tampered) | HMAC verify | 400 + `cursor_invalid` | Caller starts fresh |
| Export > 10k rows | count pre-check | 413 + `export_too_large; use_dsar` | DSAR via FR-PORTAL-008 |
| Rate limit hit | counter | 429 + Retry-After | Caller backs off |
| Sync_class column missing on source table | view DDL fails at migration | Migration error; rollback | Source module adds sync_class column |
| RLS bypassed (auth.subject_id not set) | view returns 0 rows | Empty list response | Handler bug; fix session var |
| Calendar view requested at slice 1 | enum match → SliceUnavailable | 501 + `slice_2_coming` | Wait for slice 2 |
| Detail row not found | view returns 0 | 404 + `not_found` | Inherent |
| ETag collision (rare) | client sees stale 304 | Up to 30s staleness | Cache TTL eventual consistency |
| Slow query on large filter | OBS p95 latency alarm | sev-3; investigate index | Add index per filter pattern |
| Export stream cancelled mid-way | client disconnect | Partial CSV written + audit `export_completed` with partial count | Client retries |
| Cross-tenant view via session var hack | RLS rejects via current_setting | 0 rows; appears as empty list | Inherent isolation |
| Sub-resource missing for detail row | source table doesn't have row | Sub-resource field is empty array | Inherent |
| Search returns no results | tsvector miss | 200 + empty rows + next_cursor null | Inherent |
| Field projection on sub-resource field | not in safe set | Rejected upfront | Caller uses detail endpoint |
| Filter date_range invalid format | parse error | 400 + `invalid_date_format` | Caller fixes |
| Postgres view definition out of sync with source table schema | migration test | CI fails | Update view + re-migrate |
| ETag computation overhead on huge response | OTel latency | sev-3 if p95 > 500ms | Cache ETag in Redis (slice 2) |

---

## §11 — Implementation notes

**§11.1** SQL views chosen over materialised views at slice 1 — correctness > performance; slice 2 may add materialised view + refresh-on-source-change pattern.

**§11.2** Cursor HMAC uses server-side secret rotated quarterly; key in KMS.

**§11.3** ETag computation = SHA-256 truncated 16 hex; matches FR-MCP-005 PRM pattern.

**§11.4** Sub-resource queries (detail endpoint) hit additional tables; each requires its own RLS path. Slice 1 = sequential queries; slice 2 = parallel via `tokio::join!`.

**§11.5** Field projection at SELECT level (not post-fetch filter) reduces row size + network bandwidth.

**§11.6** Export streaming via `axum::body::Body::from_stream`; row batches of 100 to bound memory.

**§11.7** XLSX export uses `rust_xlsxwriter` crate; streaming write to bytes buffer.

**§11.8** Rate limit per-caller via Redis sliding-window (consistent with all other PORTAL FRs).

**§11.9** SQL view RLS via WHERE clause + current_setting — not separate POLICY because views inherit base-table policies but adding view-specific RLS predicate is clearer.

**§11.10** Cache-Control: private + max-age=30 is intentional — 30s allows fast navigation between pages without re-querying; per-user privacy maintained.

**§11.11** The `portal.view_redaction_applied` audit is sampled 5% (higher than view_read's 1%) because redaction is forensically relevant — operators may want to see "is the tenant correctly classifying sync_class" patterns.

**§11.12** ETag is computed over the canonical-JSON of the response (sorted keys) for determinism — same data on same request returns same ETag.

**§11.13** The view definition aliases internal fields with CASE WHEN to NULL-out for redacted rows; client always sees the same JSON shape (avoids "this row has fewer fields" UX confusion).

**§11.14** Calendar view stub at slice 1 returns 501 immediately; logged but no audit row (informational only).

**§11.15** Export 10k cap is per-export; multiple exports can accumulate. Rate limit + DSAR alternative.

---

*End of FR-PORTAL-001 spec.*
