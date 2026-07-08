# Phase 2 tasks - online-offline sync + compliance

Gate to enter: P0 done; MEM-012/013/014 at least in_review (erasure needs the fact layer to cascade over). Two independent tracks: sync (MEM-032..038) and compliance (MEM-039..044); they can interleave.

---

## MEM-032 - Sync server: shape stream down, outbox up

refs R60, R65, F25, F26 | est 16h | deps MEM-001, MEM-010 | priority critical

Why: no first-party sync exists; chat-core already proved the pattern (server-authoritative seq/pos stream + idempotent outbox).

Files: new `src/sync/mod.rs` + `src/sync/shapes.rs` + `src/sync/outbox.rs`, routes in `main.rs`, contract doc `docs/feature-requests/memory/FR-MEMORY-129-first-party-sync.md` (new FR, drafted by this task).

Steps:
1. Down: `GET /v1/memory/sync?cursor=<seq>&shapes=...` streams (long-poll first, SSE later) chain rows and derived current rows the device may hold: own-subject rows, granted-scope rows, `sync_class` shareable only, per-shape WHERE definitions server-side (never client-supplied SQL).
2. Up: `POST /v1/memory/sync/outbox` accepts batches of client ops (puts, quick captures, interaction events); dedup on the existing per-tenant unique `event_id` (migration 0005); returns per-op accept/conflict(409+server row)/reject.
3. Enforce `sync_class` and access grants server-side at the shape layer (R65) with tests; RLS remains the backstop.
4. Cursor semantics documented: resume from last acked seq; snapshots for cold start delegated to MEM-036.

Accept: two simulated devices converge through the server; a private row never appears in another device's stream (test); replayed outbox batches are no-ops.

Tests: shape-scoping tests (grants, sync_class); outbox idempotency + conflict tests; resume-from-cursor test.

Review (human): read the FR draft + shape definitions; shapes are the data-exposure policy for devices.

---

## MEM-033 - Tauri SQLite client core

refs R61, R63 | est 20h | deps MEM-032 | priority critical

Files: new `desktop/src-tauri/src/sync/` (Rust worker, rusqlite), SQLite schema `memory_synced_*` / `memory_local_*` / views / `changes` outbox, `sync_supervisor.rs` rewired to spawn the internal worker.

Steps:
1. Schema per the report's through-the-DB pattern: immutable `*_synced` tables written only by the stream consumer; `*_local` shadow tables for pending writes; combining views; INSTEAD OF triggers appending to `changes`.
2. Worker loop: drain `changes` to the outbox endpoint (batch, retry with the existing backoff/circuit breaker), consume the stream into `*_synced`, clear local rows on ack, rebase on 409 (server row wins per column LWW + revision; local diff re-queued or surfaced).
3. Tray states wired: syncing / offline / error / up-to-date.
4. Remove the python daemon path entirely (MEM-010 flagged it off; this deletes it).

Accept: create/edit offline, restart app, reconnect: converges with zero dupes; 409 path exercised in a test; tray reflects truth.

Tests: Rust integration tests against a local server instance; kill-mid-drain crash test; view-merge unit tests.

Review (human): run the desktop app through an offline day yourself (airplane-mode test) before sign-off.

---

## MEM-034 - Derived-data policy (recompute on server)

refs R62 | est 8h | deps MEM-033 | priority high

Files: `src/sync/shapes.rs` (exclude derived tables from up-sync; optional embeddings-down shape), desktop store (optional sqlite-vec table keyed by embed_model_version), docs.

Steps: assert by construction that clients never upload embeddings/summaries/facts (outbox schema simply has no such op); optional read-only embeddings-down shape + sqlite-vec local index for on-device search, gated by a setting, dropped and re-pulled when `embed_model_version` changes.

Accept: derived rows regenerate server-side after an offline edit; on-device search (if enabled) returns parity results for hot-window content.

Tests: recompute-after-sync test; model-version invalidation test.

Review (human): decide whether on-device semantic search ships now or stays dark (battery/size cost vs value).

---

## MEM-035 - Desktop at-rest encryption

refs R66, F27 | est 8h | deps MEM-033 | priority high

Files: desktop store init (SQLCipher via rusqlite bundled-sqlcipher), key management via `keyring` crate (macOS Keychain / Windows DPAPI / Secret Service), perms hardening, recovery doc.

Steps: AES-256 full-file encryption with a per-device random key stored in the OS keystore; WAL mode; 0600 on db/wal/journal; key-loss recovery = wipe + re-hydrate from server (documented + menu action); migration path for existing unencrypted dev stores.

Accept: db files unreadable without keystore; recovery flow tested; startup fails loud (not silently unencrypted) if keystore unavailable.

Tests: encryption round-trip; perms assertions; recovery integration test.

Review (human): verify on your Mac that the key lands in Keychain and the recovery action works.

---

## MEM-036 - Offline capture + hydration

refs R67, R68, R69 | est 8h | deps MEM-033 | priority high

Files: desktop capture paths (quick capture, hooks) writing to the local outbox with client UUIDv7 event_ids; `src/sync/` snapshot endpoint; docs.

Steps: snapshot for cold start = hot window (30d) + current summaries + profiles, then cursor streaming; cold history stays server-side, fetched on demand by `audit_row_id`; document the exactly-once contract for module authors; mark `conflicts.py` sibling detection legacy (doctor still reports, banner says first-party sync active).

Accept: fresh device usable in under a minute on dev data; offline capture chains exactly once after reconnect.

Tests: hydration timing test; offline-capture replay test.

Review (human): none beyond gate.

---

## MEM-037 - Device identity + transport hardening

refs R70, R71 | est 6h | deps MEM-032 | priority medium

Files: migration adding `device_id` to pushed chain rows (nullable, aux column; anchors unaffected), auth module device-token issuance (short expiry + refresh), per-device rate limits on sync routes.

Accept: rows attributable per device in the console viewer; expired device token refreshes cleanly; abusive device throttled independently of the user.

Tests: token refresh integration; per-device limit test; attribution query test.

Review (human): confirm device revocation story (kick-by-revoke parity with the auth module's session revoke).

---

## MEM-038 - Sync test matrix + chaos + ingest idempotency properties

refs R72, R98 | est 10h | deps MEM-033, MEM-036 | priority high

Files: `desktop/src-tauri/tests/` matrix suite, `services/memory/tests/` property tests, CI wiring.

Steps: matrix = {offline create, edit, delete} x {concurrent server change, none} x {crash mid-drain, clean} x {replay, no replay}; chaos rig style ported from `modules/memory/runtime/tests/chaos`; property tests for the ingest worker (crash between UPSERT and cursor, duplicate seq delivery, out-of-order seq, gateway flapping).

Accept: matrix green in CI; convergence and zero duplicate chain rows proven under chaos; properties pin the idempotency envelope.

Tests: this task is tests.

Review (human): read the matrix table for missing cells; approve as the sync regression bar.

---

## MEM-039 - Retention policy engine + reaper

refs R81, R32, F22 | est 10h | deps MEM-012 | priority high

Files: migration `retention_policy(tenant_id, memory_kind|event_class, ttl_days, action CHECK IN ('archive','none'), version, created_at)` (RLS'd, versioned, every change chained), new `src/brain/reaper.rs` job on the maintenance tick, cold-storage archiver (object store write per deploy scaffolding).

Steps: defaults per report (presence/view events weeks, content events years, facts until invalidated); reaper archives below-threshold or expired rows out of serving tables/indexes into cold storage with a manifest; Layer 1 untouched within its legal window; every reap emits counts to the ops metrics and a chained job row.

Accept: policy-driven aging demonstrable on dev with shortened TTLs; archived rows restorable by manifest; L1 intact.

Tests: TTL aging integration; archive/restore round trip; policy-versioning audit test.

Review (human): set the real TTL numbers (legal + product call, PDPL retention-minimization applies).

---

## MEM-040 - Erasure: crypto-shredding + lineage cascade

refs R82, R84, R89, F21 | est 20h | deps MEM-039, MEM-014 | priority critical

Why: append-only chain with inline bodies vs PDPL/GDPR erasure; crypto-shredding is the resolution and must land before content capture scales.

Files: migration for `subject_dek(tenant_id, subject_id, wrapped_dek, created_at, destroyed_at)` + `derived_artifact(source_audit_row_id, artifact_type, artifact_id)` lineage; envelope-encryption in the emit path for content-bearing bodies (AES-GCM, DEK per subject, KEK per tenant via KMS/env for dev); `src/brain/erasure.rs` job; `admin.rs` `erase-subject` command; backup re-deletion ledger; field-level crypto option for sensitive kinds (R84).

Steps:
1. New content-bearing chain writes encrypt `body` with the subject DEK (envelope format versioned); chain anchors hash the ciphertext, so verification survives erasure.
2. Lineage rows written by ingest/fact/summary pipelines (artifact_type in embedding|fact|summary|edge|cache).
3. Erasure job: destroy DEK (set destroyed_at, delete key material), delete the subject's vectors + facts + snippets, re-summarize affected windows, REINDEX the partial HNSW indexes (ghost-vector caveat), chain an erasure event with counts.
4. Backups: erasure ledger replayed on restore (documented + drill).
5. Legacy plaintext rows: one-time migration encrypting existing content-bearing bodies (or archiving them under the retention engine) - scoped decision recorded in the PR.

Accept: erasure drill on a test subject removes all plaintext and vectors, chain still verifies end to end, restore-from-backup replays the erasure; drill documented as a runbook.

Tests: end-to-end erasure drill in CI (small corpus); ghost-vector test (nearest-neighbor of erased vector absent after reindex); DEK-destruction unit tests.

Review (human): security review of the envelope format and KMS wiring; run the erasure drill yourself on staging; decide the legacy-rows scoping question in the PR.

---

## MEM-041 - PDPL 91/2025 pack

refs R83, R90 | est 10h | deps MEM-025 | priority critical

Files: `docs/legal/memory-dpia-2026.md`, `docs/legal/memory-ctia-2026.md` (drafts for operator filing), consent-mapping note linking the acknowledgment ledger to PDPL consent requirements, `docs/legal/memory-compliance-cadence.md` (quarterly evidence checklist per R90).

Steps: draft the DPIA over the monitoring program (data categories, purposes, subjects' rights, safeguards incl. MEM-040); draft the CTIA covering Supabase SG + Vultr SG hosting of VN employees' data (60-day filing clock runs from first transfer); map notice/acknowledgment flow to consent rules; define the quarterly evidence bundle (access reviews, RLS probes, PII recall scores, erasure drill results).

Accept: both filings drafted to submission quality with placeholders only for operator signatures/dates; cadence checklist merged.

Tests: none (documents); cross-references validated.

Review (human): legal review + filing are operator actions; this task only prepares them. Filing itself is a §2.2 manual action.

---

## MEM-042 - Recall read-audit rows

refs R85, F23 | est 5h | deps MEM-001 | priority high

Files: `src/brain/handler.rs` / `recall.rs` (post-response emit), event kind `memory.recall_performed` with caller subject, query hash (never raw query text - it may contain PII), hit count, distinct subjects touched; sampling knob if volume demands.

Accept: every recall chains an audit row; overhead measured (<5ms budget, async emit); console viewer filters it.

Tests: emit-on-recall integration; PII-absence test (raw query never chained).

Review (human): confirm query-hash-only satisfies the evaluation program's audit needs (trade-off recorded).

---

## MEM-043 - External chain anchoring + nightly walker

refs R86 | est 10h | deps none | priority high

Files: new `src/bin/chain_walker.rs` (or extend admin), scheduled job (deploy cron/compose), anchor publisher (signed git tag or write-once object-store key per deploy scaffolding), alerting via obs.

Steps: nightly walk verifies `l1_audit_log` anchors end to end per tenant (batched, resumable); publish the signed chain head externally on schedule; alert on divergence with the failing seq; wire the walker's result into MEM-052's ops tile later.

Accept: corruption injected in a dev copy is detected and alerted with the exact row; anchor visible outside the database.

Tests: corruption-detection test; publisher dry-run test.

Review (human): choose the anchor destination (git tag vs bucket) and hold the signing key (operator secret, §2.2).

---

## MEM-044 - Rust denylist + admin hardening

refs R87, R88 | est 6h | deps MEM-002 | priority medium

Files: `src/interaction/emit.rs` (denylist check in validate), denylist patterns shared with `modules/memory/runtime/tests/denylist` (extract to a shared spec file both suites read), `src/bin/admin.rs` (admin role required, `--break-glass` flag for prod URLs, chained invocation row with operator identity).

Accept: secret-shaped attributes rejected at validate with a metric; admin invocations audited; prod runs demand the flag.

Tests: denylist table tests (shared vectors with the Python suite); admin audit-row test.

Review (human): skim the shared denylist spec for gaps (API key shapes used by CyberSkill vendors).
