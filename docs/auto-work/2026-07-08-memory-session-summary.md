# AUTO_WORK session summary - memory improvement program - 2026-07-08

Branch: `auto/memory-enterprise` (from `main` @ 76fb90a). Executor: Claude (Cowork). Author on the mount; gate + git on the operator's Mac via Desktop Commander (sandbox has no cargo/docker and cannot git-write the mount). Docker Desktop was started this session for the DB gates (dev Postgres = container `cyberos-postgres`, pgvector pg16).

## Delivered - 6 commits, all fully gated, all `in_review`

Per-task ledgers: `2026-07-08-memory-1.md` .. `-5.md`. Only the human reviewer sets `done` (PROMPT.md prompt B).

1. **MEM-001** (32e69cf) - JWT auth on `/v1/memory`. Header-trust identity (`x-tenant-id`/`x-subject-id`) replaced by a verified CyberOS access token (ports `chat/src/auth.rs`; `authenticate_claims` + `require_auth` middleware; `AppState` fails boot without a verifier). 10 new auth tests.
2. **MEM-002** (98bfda6) - fail-closed brain-table RLS (migration 0009 drops the unset-GUC fail-open + nil-uuid bypass arms). Proven at the DB by `brain_rls_test.rs` under a non-superuser probe role.
3. **MEM-059** (4d8cae4, discovered) - fixed the `brain_common` test harness so the brain DB suite RUNS for the first time (it applied multi-statement migrations with `sqlx::query` -> setup error; + a concurrent-apply race). raw_sql + once-per-process `OnceCell` migrate.
4. **MEM-060** (7929a4e, discovered) - fixed TWO serious latent product bugs the now-runnable suite exposed:
   - `summarize::scope_event_filter` hardcoded `$3`, reused where `$2` was needed -> **rolling summaries had NEVER built for subject/channel scopes** (bind mismatch).
   - `access_scope` decoded `SELECT 1` (INT4) as `Option<i64>` -> the **founder/manager ALLOW path of the access boundary ERRORED whenever a grant matched**.
5. **MEM-005** (33c1597) - recall confidence floor now uses the REAL top summary cosine similarity, not a hardcoded `1.0` (the floor -> drill decision was dead code). `should_drill` helper + `best_summary` in explain + DB regression test.

Plus the two critical security holes from the report (auth, RLS) are closed.

## Brain DB suite status (was 0 tests running; now)

`brain_ingest 3/3, brain_provenance 3/3, brain_recall_access_scope 4/4, brain_summaries 3/3, brain_rls 1/1, brain_confidence_floor 1/1` — all green. `brain_tiering 0/2` — fails on a distinct pre-existing bug, filed as MEM-061.

## Filed during execution (new backlog tasks)

- **MEM-059** (done above) and **MEM-060** (done above) - discovered + fixed.
- **MEM-061** (P0, high, ready) - the interaction `emit` path drops `occurred_at_ns`: `cyberos_audit_chain::emit_genesis_with_op` sets `l1_audit_log.ts_ns` to INGESTION time, not occurrence time, so age-based tiering never fires and recency ranking (MEM-021) is undermined. Fix in the emit/audit-chain path. This is the root cause of the 2 `brain_tiering_test` failures.

## Still not started - blockers recorded on each backlog entry

- **MEM-007** (critical) - INFRA. The ai-gateway `/v1/embeddings` route is ALREADY wired (`server/mod.rs:360`); it lacks a concrete bge-m3 backend + tenant policy (memory sends model `bge-m3`; `call_embed_provider` has no `Bge` adapter). Full acceptance needs a model backend, not stand-up-able headlessly. The memory-side contract test (vs a stub gateway) can proceed independently.
- **MEM-006** (high) - batched recall pipeline (snippets + one-query verify + set-based access). Now gateable (brain suite works), but a substantial recall.rs refactor touching the security-sensitive access path; left for focused work.
- **MEM-009** (high) - golden eval runner. Now gateable; not started.
- **MEM-010** (high) - the three architecture ADRs (operator-decision forks) + retire the phantom Tauri python-daemon spawn. Needs the human's architectural sign-off.
- **MEM-008** (medium) - metrics label fix + per-leg latency. Small; not started.
- **MEM-003 / MEM-004** - correctly `blocked` on MEM-001/002 being `done` (review-gated).

## For the reviewer

- 5 tasks in `in_review` (MEM-001, 002, 059, 060, 005). Review with prompt B; approving MEM-001/002 unblocks MEM-003/004.
- MEM-060 is a notable finding: two core features (rolling summaries; founder/manager recall access) were silently broken and had never been exercised because the brain DB test suite never ran. Worth a look at how "proven green" was claimed for tests that could not execute.
- Operator actions still pending: `git push` (branch not pushed); before enabling `DEPLOY_MEMORY`, set `MEMORY_AUTH_JWKS_URL` (or HS256 secret) in the memory env (MEM-001 makes boot fail-closed).

## Environment notes for the next session

- Start Docker Desktop for DB gates (`open -a Docker`); `DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos`.
- The dev `cyberos` DB user is a SUPERUSER and bypasses RLS - RLS assertions must run under a non-superuser role (see `brain_rls_test.rs`).
- Cargo workspace root: `~/Projects/CyberSkill/cyberos/services`.
