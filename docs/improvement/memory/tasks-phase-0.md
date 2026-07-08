# Phase 0 tasks - safety and truth

Source: report sections 5-6 (docs/strategy/memory-enterprise-grade-and-auto-evolution-plan-2026-07-06.md). Status lives in `backlog.yaml`, not here. Order within the phase: MEM-001/002/003 first (everything else assumes a trustworthy recall path); the rest may interleave.

---

## MEM-001 - JWT auth on /v1/memory endpoints

refs R73, F15 | est 6h | deps none | priority critical

Why: `recall_handler` and `search` resolve tenant and viewer from `x-tenant-id` / `x-subject-id` headers (`src/brain/handler.rs:28-32`, `src/search.rs:267`), so any network caller can claim founder-grade visibility in any tenant.

Files: `services/memory/src/main.rs`, `src/brain/handler.rs`, `src/search.rs`, new `src/auth_mw.rs`; reference implementation: the chat service's FR-AUTH-004 JWT middleware (JWKS verify against the auth module).

Steps:
1. Add an axum middleware that verifies the bearer JWT against the auth module's JWKS (cache keys, handle rotation the way chat does) and injects a typed `Caller { tenant_id, viewer_subject_id }` extension from claims.
2. Replace header parsing in both handlers with the extension; reject requests without a valid token (401) or with tenant claims mismatching any residual header (400).
3. Keep `/healthz` and `/metrics` unauthenticated; everything under `/v1/memory` requires the token.
4. Env: `AUTH_JWKS_URL`, `AUTH_ISSUER`, `AUTH_AUDIENCE` with sane dev defaults; document in the service README.

Accept: a forged `x-tenant-id` header changes nothing; tenant and subject derive only from verified claims; expired/garbage tokens get 401.

Tests: integration test with a locally-signed JWKS pair proving (a) valid token recalls, (b) forged header ignored, (c) cross-tenant token cannot read another tenant, (d) expired token 401.

Review (human): confirm the JWKS URL points at the auth service in every deploy env; probe prod-like stack with `curl` yourself using a stale token; check no route under `/v1/memory` escapes the middleware.

---

## MEM-002 - Fail-closed RLS on brain tables

refs R74, F16 | est 6h | deps none | priority critical

Why: every brain policy passes when `current_setting('app.tenant_id', true) IS NULL` and treats the nil UUID as a bypass (`migrations/0006-0008`), so one code path that forgets `tenant_tx` reads across tenants silently.

Files: new `migrations/0009_rls_fail_closed.sql`, `src/brain/mod.rs`, `src/bin/admin.rs`, `src/brain/backfill.rs`.

Steps:
1. Migration 0009: recreate the policies on `brain_event_embedding`, `brain_summary`, `brain_ingest_cursor`, `brain_tier_watermark` with only `tenant_id::text = current_setting('app.tenant_id', true)`; no NULL arm, no nil-uuid arm.
2. Create role `cyberos_memory_admin` for rebuild/backfill; grant it and give it either BYPASSRLS or an explicit admin policy; wire `admin.rs` and `backfill.rs` to require it (connection string env `MEMORY_ADMIN_DATABASE_URL`).
3. Remove the `ADMIN_TENANT` nil-uuid constant usage from runtime paths; keep a doc note in `mod.rs` explaining the fail-closed contract.

Accept: any query without the GUC set returns zero rows (proven by test); daemon and handlers still work because they already use `tenant_tx`; admin flows work only under the admin role.

Tests: extend `interaction_event_rls_test.rs` with a no-GUC query asserting zero rows on all four tables; a rebuild smoke under the admin role.

Review (human): run the no-GUC probe against staging yourself; verify the admin connection string is absent from the normal service env in deploy configs.

---

## MEM-003 - RLS with FORCE on l1_audit_log and l2_* + cross-tenant probe

refs R75, R76, F17 | est 8h | deps MEM-002 | priority critical

Why: `l1_audit_log`, `l2_memory`, `l2_entity`, `l2_edge` have no RLS at all (migrations 0001/0003); isolation rests on WHERE clauses.

Files: new `migrations/0010_rls_l1_l2.sql`, `src/layer2/*.rs` and `src/search.rs` call sites (ensure every query path runs inside `tenant_tx`), tests.

Steps:
1. Migration 0010: ENABLE + FORCE RLS with the fail-closed policy (MEM-002 form) on the four tables; grants unchanged.
2. Audit every query in `layer2/`, `search.rs`, `rebuild.rs`, `brain/provenance.rs` for direct pool usage without the GUC; wrap in `tenant_tx` or route through the admin role where genuinely cross-tenant (tenant discovery, metrics endpoint).
3. `main.rs::discover_tenants` and `metrics` become admin-role reads (they are legitimately cross-tenant).
4. Add composite-index sanity: EXPLAIN the hot recall/search/ingest queries under RLS and record plans in the PR; add missing `(tenant_id, ...)` leading indexes if any plan degraded (R76).
5. Extend the repo's RLS property gate to cover memory tables and add a scheduled cross-tenant probe test (two tenants seeded, each queries the other, expects zero).

Accept: all memory tables FORCE RLS fail-closed; probe reads zero cross-tenant rows; no hot query lost its index.

Tests: property test over the four tables; probe test; plan-regression assertions for recall and ingest queries.

Review (human): read the EXPLAIN outputs in the PR; run the cross-tenant probe against staging; confirm deep-audit R15 tracking references this task.

---

## MEM-004 - Rate limits on recall/search

refs R77, F18 | est 4h | deps MEM-001 | priority high

Why: recall is an extraction oracle without per-principal limits (`main.rs` router has none).

Files: `services/memory/src/main.rs`, new `src/rate_limit.rs` (or tower-governor), config env.

Steps: token-bucket per (tenant, subject) and per tenant on `/v1/memory/*`; defaults e.g. 60/min subject, 600/min tenant, burst 10; 429 with `retry-after`; counters into OTel; env-overridable per tenant later via config table (leave hook).

Accept: sustained burst gets 429s; limits configurable; healthz unaffected.

Tests: integration test hammering recall past the bucket; assert 429 + header; assert counter increments.

Review (human): sanity-check the default numbers against expected Lumi/console traffic; confirm 429s visible on the obs dashboard.

---

## MEM-005 - Fix the dead recall confidence floor

refs R9, F7 | est 3h | deps none | priority high

Why: `recall.rs:102` sets `best_summary = 1.0` for any summary hit, so the `recall_confidence_floor` (0.30) drill trigger can never fire on quality grounds.

Files: `src/brain/recall.rs`.

Steps: return cosine similarity (1 - distance) from `summary_search` per row (`embedding <=> $2` is already computed; select it), set `best_summary` to the top hit's real similarity, and compare against the floor. Full-text fallback path: map `ts_rank_cd` onto [0,1] conservatively or treat as below-floor (drill unavailable without a vector anyway; keep current behavior there).

Accept: weak best-summary similarity triggers drill with `drill=false`; strong match does not; behavior documented in the module header.

Tests: unit/integration with the stub embed client: seed a summary far from the query vector, assert the drill path label in `explain`; seed a near one, assert summary-only.

Review (human): eyeball the floor value against real data once golden cases exist (MEM-009); it may need tuning.

---

## MEM-006 - Batched candidate pipeline (snippets, verify, access)

refs R10, R18, R19, F8, F9 | est 8h | deps none | priority high

Why: event hits return empty snippets (`hot_event_search` never filled, `enrich_snippet` never called), and the loop does up to 3 queries per candidate x 50 candidates.

Files: `src/brain/recall.rs`, `src/brain/access_scope.rs`, `src/brain/provenance.rs`.

Steps:
1. After fusion, collect candidate seqs and fetch their L1 rows in one `WHERE tenant_id=$1 AND seq = ANY($2)` query; fill snippets (`snippet_from_body`) and verify anchors in Rust from that single result set.
2. Replace per-candidate `caller_may_see` with one visible-set computation: self + founder-grant check (one query) + granted targets list (one query); filter candidates in memory. Keep `deny_reason` metrics by classifying the filtered-out subjects in one grouped query.
3. Keep the fail-closed semantics identical: tampered candidates dropped before access filtering; deny-by-default for unknown subjects.
4. Record recall p95 before/after in the PR (local bench with seeded data is fine).

Accept: non-empty snippets on event hits; fixed query count per recall (embed + 2 retrievers + 1 L1 batch + 2 access) regardless of candidate count; identical access outcomes on the existing access-scope test.

Tests: extend `brain_recall_access_scope_test.rs` to assert snippet presence and query-count ceiling (via a counting executor or pg stat snapshot); tamper test still green.

Review (human): check the access refactor against `services/eval/src/access.rs` semantics line by line; this is the intra-tenant boundary, treat it like auth code.

---

## MEM-007 - ai-gateway /v1/embeddings route + contract test

refs R44, F34 | est 8h | deps none | priority critical

Why: `embed_client.rs` targets a documented-but-unwired gateway route; until it exists the brain only ever ran against stubs.

Files: `services/ai-gateway` (router: embeddings handler using the existing `EmbedRequest`/`EmbedResponse` stubs, model alias `bge-m3` -> embed-sidecar), `services/memory/tests/` contract test, `services/dev` compose wiring.

Steps:
1. Implement `POST /v1/embeddings` in the gateway: resolve TenantPolicy from `x-tenant-id`, route the alias to the in-region embedding backend (embed-sidecar bge-m3), charge the spend cap, return `{embeddings, model, embed_model_version}` and 402 on cap exhaustion, matching `embed_client.rs` expectations exactly (dim 1024).
2. Support batch `input: []` from day one (MEM-026 depends on it).
3. Add a memory-side contract test that spins a stub gateway binary (or wiremock) asserting request/response shape and the 402 path.
4. Wire `services/dev` compose so `cargo test -p cyberos-memory` can exercise the real route locally when the sidecar is up; keep the deterministic stub for unit tests.

Accept: real embeddings flow end to end in dev; 402 degrades to `pending_embed_retry`; contract test pins the shape from the memory side.

Tests: gateway unit tests (routing, cap, batch); memory contract test; one end-to-end ingest-and-recall smoke in dev compose.

Review (human): verify residency + ZDR policy resolution applies to the embeddings route the same as chat; confirm spend shows up on the gateway's tenant ledger.

---

## MEM-008 - Metrics correctness + per-leg latency

refs R100, F31 | est 2h | deps none | priority medium

Files: `src/brain/ingest_worker.rs:68` (label `embed_malformed`), `src/brain/recall.rs` (span or histogram per leg: summary_search, event_search, l1_batch, access_filter, rerank later), `src/brain/metrics.rs`.

Accept: Malformed counted under its own label; traces show per-leg timings.

Tests: metrics unit assertions where the shim allows; otherwise label constants pinned by test.

Review (human): confirm the obs dashboards pick up the new labels without breaking existing panels.

---

## MEM-009 - Golden recall eval runner + seed set

refs R45, F32 | est 10h | deps none | priority high

Why: zero measurement today; every later phase gates on this runner.

Files: new `services/eval/src/memory_golden/` (or `services/memory/tests/golden/`), `docs/improvement/memory/golden/seed-cases.yaml` (25 cases), CI workflow hook.

Steps:
1. Define the case schema: `{id, query, caller_subject, expect_audit_row_ids[] (any-of), expect_kind, notes}`.
2. Write 25 seed cases against a deterministic fixture corpus (seeded through the emit path with the stub embed client so CI needs no gateway).
3. `memory-eval` binary: loads fixtures, runs recall per case, scores recall@10 and MRR, writes a JSON report; exit nonzero on >3% regression against the checked-in baseline.
4. CI: run on PRs touching `services/memory/**` or the golden dir; store the baseline in-repo, updated deliberately.

Accept: runner green on main; a deliberate ranking sabotage (test) trips the gate.

Tests: the runner is the test; add one meta-test that a shuffled ranking fails.

Review (human): read the 25 cases; they encode what "good recall" means for CyberSkill, which is a product judgment, not an engineering one.

---

## MEM-010 - Architecture ADRs (single plane, Rust sync, no CRDTs)

refs R1, R59, R64, F25 | est 4h | deps none | priority high

Why: three direction-setting defaults from the report need recording so later tasks do not relitigate them; EXECUTION-DISCIPLINE says pick the default, record it, continue.

Files: `docs/adrs/ADR-MEM-001-single-memory-plane.md`, `ADR-MEM-002-sync-rust-in-tauri.md`, `ADR-MEM-003-no-crdt-memory-rows.md`; `services/memory/desktop/src-tauri/src/sync_supervisor.rs`.

Steps: write the three ADRs (context, decision, consequences, revisit triggers), each two-thirds a page, citing report sections 6.A/6.F/10. In the desktop app, remove the python daemon spawn or gate it behind `MEMORY_SYNC_LEGACY_PY=1` defaulting off, so the tray no longer implies syncing that does not happen; log a clear "sync not yet shipped, see MEM-032" message.

Accept: ADRs merged; desktop no longer spawn-loops a nonexistent module by default.

Review (human): these are the operator-fork items; approve or veto each ADR explicitly. Veto reroutes MEM-032/033 scope before any sync code is written.

---

## MEM-011 - Pin pooled-connection GUC behavior; one GUC name

refs R95, F35 | est 4h | deps MEM-002 | priority medium

Files: `src/brain/mod.rs`, `src/brain/access_scope.rs` (drop the set-both workaround once eval aligns), integration test, `docs/deploy` note.

Steps: integration test against PgBouncer/Supavisor transaction mode (dev compose service) proving `set_config(..., true)` never leaks across pooled transactions; standardize on `app.tenant_id` in memory + eval (coordinate the eval migration or keep a compat shim with a removal note); document the pooling requirement in the deploy docs.

Accept: leak test green under transaction pooling; one GUC name (or a dated shim) across services.

Review (human): confirm the Supabase/pool mode used in prod matches the tested mode.
