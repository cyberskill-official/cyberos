# AUTO_WORK session summary - memory improvement program - 2026-07-08

Branch: `auto/memory-enterprise` (from `main` @ 76fb90a). Executor: Claude (Cowork). Author on the mount; gate + git on the operator's Mac via Desktop Commander (sandbox has no cargo/docker and cannot git-write the mount).

## Delivered (fully gated, committed, in_review)

- **MEM-001** (commit 32e69cf) - JWT auth on `/v1/memory`. Header-trust identity replaced by a verified CyberOS access token (ports `chat/src/auth.rs`; `authenticate_claims` + `require_auth` middleware; `AppState` holds an `Arc<Authenticator>` and fails boot without a verifier). Gates green: fmt, clippy `-D warnings`, `cargo test` (82 lib tests incl. 10 new auth tests). Ledger: `2026-07-08-memory-1.md`.
- **MEM-002** (commit 98bfda6) - fail-closed brain-table RLS. Migration 0009 removes the unset-GUC (fail-open) and nil-uuid bypass arms from all 4 brain-table policies; proven at the DB by `tests/brain_rls_test.rs` under a non-superuser probe role. Ledger: `2026-07-08-memory-2.md`.

Both are the program's two P0 **critical** items (the header-trust auth hole and the fail-open RLS hole). They await human review (only the reviewer sets `done`); use PROMPT.md prompt B.

## Not started this session - blockers recorded on each backlog entry

- **MEM-007** (critical) - INFRA. Key finding: the ai-gateway `/v1/embeddings` route is ALREADY wired (`server/mod.rs:360`); `call_embed_provider` dispatches to real local adapters but has no `Bge`/`Vertex` adapter, and memory sends model `bge-m3`. Full acceptance ("real gateway serves bge-m3 under tenant policy") needs a bge-m3 embedding backend + tenant policy - not stand-up-able headlessly. The memory-side contract test (against a stub gateway) is DB-free and can proceed independently.
- **MEM-005 / MEM-006 / MEM-009** (high) - HARNESS. All verify via DB-backed recall over `tests/brain_common.rs`, which is **pre-existing-broken** (discovered this session): it applies multi-statement migrations with `sqlx::query` (extended protocol) -> "cannot insert multiple commands", and fixing that unmasks a concurrent-migration race on `pg_proc`. The whole DB-backed brain suite is red at setup, independent of my changes.
- **MEM-003 / MEM-004** - correctly `blocked` on MEM-001/002 being `done` (review-gated).
- **MEM-008** (medium) and **MEM-010** (high, ADRs) are unblocked and fully completable next.

## Recommended next priority (keystone unblocker)

Fix the `brain_common.rs` test harness (change `sqlx::query` -> `sqlx::raw_sql` for multi-statement files AND serialize migration application, e.g. a process-wide `tokio::sync::OnceCell`/`Mutex` or a single migrate-once step). This one fix unblocks MEM-005, MEM-006, and MEM-009 (and makes the entire brain DB suite runnable for the first time). `tests/brain_rls_test.rs` already demonstrates the working pattern (raw_sql + self-contained setup). Suggest formalizing as a new P0 task (e.g. MEM-011a) or folding into MEM-038; it is the critical path for the rest of P0's DB-backed verification.

## Environment notes for the next session

- Docker Desktop must be running for DB gates (`open -a Docker`; dev Postgres = container `cyberos-postgres`, pgvector pg16; `DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos`).
- The dev `cyberos` DB user is a SUPERUSER and bypasses RLS - any RLS assertion must run under a non-superuser role (see `brain_rls_test.rs`).
- Cargo workspace root is `~/Projects/CyberSkill/cyberos/services`.
