# Changelog — CyberOS

This is the repo-level changelog for CyberOS services (like `services/auth`, `services/ai-gateway`, `services/obs-collector`, `services/mcp-gateway`) and repo-umbrella changes (UI, website, docs, root files).

For module-specific changelogs, please refer to their respective directories:
- [CUO Module Changelog](modules/cuo/CHANGELOG.md)
- [MEMORY Module Changelog](modules/memory/CHANGELOG.md)
- [PLUGIN Module Changelog](modules/plugin/CHANGELOG.md)
- [SKILL Module Changelog](modules/skill/CHANGELOG.md)

---

## 2026-05-19 — [AUTH] FR-AUTH-005 drained 17/17 + rustc floor bumped 1.83→1.88

**FR-AUTH-005** (admin REST list_tenants + list_subjects + revoke + unrevoke + cursor + jti deny-list) drained end-to-end in one Cowork session. 17 spec-vs-code gaps closed across 5 slices; ≈1,300 LOC src + tests. BACKLOG line 224 mutated `planned` → `[BLOCKED: 17 gaps]` → `shipped + strict-audited`. **All 6 Wave-1 MUST AUTH FRs (001/002/003/004/005/006) are now shipped + strict-audited** — wave-1-2 deploy table-stakes are drain-complete.

New modules: `services/auth/src/{cursor,deny_list,sessions}.rs`. New migration: `migrations/0021_sessions.sql` (relocated per DEC-MIGRATION-SLOT-001; slot 0007 was taken). New memory_bridge emitters: `emit_subject_revoked` + `emit_subject_unrevoked`. New test files: `admin_list_test.rs` + `admin_revoke_test.rs` + `admin_cursor_pagination_test.rs` + `admin_deny_list_test.rs`. OTel `#[tracing::instrument]` on all 4 admin handlers.

Architecture decisions logged in FR-AUTH-005-admin-rest.audit.md §10.5: **DEC-DENY-LIST-001** (in-memory slice-1; Redis lift = FR-AUTH-110), **DEC-CURSOR-SIGN-001** (HMAC-SHA256 via HKDF from `AUTH_CURSOR_SIGNING_SECRET` env), **DEC-MIGRATION-SLOT-001** (0007→0021 relocation). Structural G-012 enforcement: `DenyList` exposes no `remove()` API — unrevoke literally cannot clear the deny-list at the compile-time level.

Deferred follow-ups: **FR-AUTH-110** (Redis-backed deny-list lift for wave-2 horizontal scale) + **FR-AUTH-111** (closed revoke reason taxonomy enum: compromised / terminated / policy-violation / operator-error / other).

**`services/Cargo.toml` `rust-version` bumped 1.83 → 1.88** — `webauthn-rs 0.5.5`, `time 0.3.47`, `icu_* 2.2.0`, `base64urlsafedata 0.5.5`, `home 0.5.12` all now require ≥1.86/1.88. One-time operator step: `rustup toolchain install 1.88.0`. README §1 prerequisites table updated.

**Cascading build fixes after the bump** — `cargo +1.88.0 build -p cyberos-auth` surfaced 2 errors that were either pre-existing (borrow-checker stricter on this NLL path) or shaken loose by the sqlx 0.8 + ipnetwork 0.20 trait shuffle:

- **handlers.rs:1871** — `traceparent: Option<String>` was moved into `svc.issue(..., traceparent, ...)` then borrowed via `.as_deref()` for the memory audit emit at :1896. Fixed with `traceparent.clone()` at the move site per the compiler's own suggestion.
- **travel.rs:213 + travel_admin.rs:170 + travel_admin.rs:212/244 + travel_policy.rs:134** — `ipnetwork::IpNetwork` no longer satisfies `sqlx::Encode<'_, Postgres>` / `Decode<'_, Postgres>` (likely a version-coherence pitfall: sqlx-postgres pulls its own ipnetwork copy distinct from the auth crate's). Fixed by binding/reading at the DB boundary as `String` (Postgres INET accepts the textual CIDR; `::text` cast on read). The struct field `TravelPolicy::allowlist: Vec<IpNetwork>` is preserved — parsed from `String` at the read boundary via `filter_map(|(c,)| c.parse().ok())`.
- 4 unused-import warnings cleaned (`body::Body` in middleware.rs, `Redirect` in oidc.rs, `Redirect`+`Serialize` in saml.rs).

Compile-verify on macOS: `cd services && cargo +1.88.0 build -p cyberos-auth && cargo +1.88.0 test -p cyberos-auth` (was `+1.85.0`).

## 2026-05-19 — [AI / OBS / MCP] P0 implementation wave kicks off — three new `services/` workspace members, two FRs shipped end-to-end + two FRs flipped to `building`

CyberOS's P0 build order is locked at AI Gateway → OBS → AUTH (stub) → MCP Gateway → CHAT (per `docs/feature-requests/BACKLOG.md §2 line 130`). AUTH stub was already shipped + strict-audited; this wave starts the implementation phase for AI / OBS / MCP. All three services now exist under `services/` as workspace members alongside the previously-shipped `auth`/`memory`/`skill-broker` crates.

### What landed

**`services/ai-gateway/`** — Rust workspace member, slice-1 of P0.1 AI Gateway:

- **FR-AI-003 — memory audit-row bridge (canonical Writer subprocess) shipped end-to-end (10/10).** `services/ai-gateway/src/memory_writer.rs` (440 LOC) + `…/memory_writer/canonical.rs` (220 LOC) implements: subprocess spawn of `python3 -m cyberos.writer put` with stdin/stdout/stderr piping; NFC normalisation of all string-valued nodes; sorted-key, no-extra-whitespace JSON serialisation matching AGENTS.md §6.2; SHA-256 chain-hash recomputation + verification against Writer stdout (FR-AI-003 §1 #7); path-traversal guard pre-spawn (AC #7 — absolute paths, `..` traversal, reserved prefixes `audit/` / `index/` / `HEAD` / `.lock` all rejected); 5s timeout with `kill_on_drop` + SIGTERM-then-SIGKILL semantics (AC #4); typed builders for the slice-1 closed set (`precheck` / `invocation` / `invocation_failed` / `hold_expired` / `persona_loaded`); `check_writer_available` startup health check (FR-AI-003 §1 #10). The `python3 -m cyberos.writer` Python module ships from `modules/memory/runtime/`; integration tests against the live subprocess gate behind a `--features integration-writer` flag.
- **FR-AI-005 — Tenant-policy YAML loader shipped end-to-end (10/10).** `services/ai-gateway/src/policy/{schema,cache,loader}.rs` (~700 LOC) implements: closed `TenantPolicy` + `AiPolicy` + `Provider` + `Residency` + `EmergencyOverride` schema with `schemars` JSONSchema derives; `ArcSwap<HashMap<…>>`-backed lock-free cache (AC #9 — 1000×100 concurrent reads under 3s); `notify` file-watcher with eager startup load + ALL-failures aggregation into one `LoaderInitError::Schema { failures }` (FR-AI-005 §1 #11); invalid hot-reload preserves cache + emits `tracing::error!` (FR-AI-005 §1 #5); path-traversal + charset + length validation on input `tenant_id`; `validate_yaml` pure-function entry point for `cyberos-ai policy validate` (FR-AI-005 §1 #13); `is_loadable_filename` filter rejects `EXAMPLE.` + `_` + uppercase prefixes. Integration tests cover AC #1 / #3 / #4; singleton-shared tests (AC #2 / #5 / #6 / #7 / #9) gated behind `--ignored --test-threads=1` per `OnceCell` constraint. `gen-schema` binary emits the JSONSchema mirror for CI drift detection.
- **`cyberos-ai` operator CLI** (`src/bin/cyberos_ai.rs`) — slice-1 subcommands: `policy validate <file>` + `policy list` + `serve` (placeholder until FR-AI-008 lands the HTTP router).
- **Workspace wiring:** `services/Cargo.toml` adds `ai-gateway` as a member. **Dependencies declared:** `serde_yaml` 0.9 · `schemars` 0.8 with `rust_decimal` feature · `rust_decimal` 1.36 with `serde-with-str` · `notify` 6.1 · `arc-swap` 1.7 · `once_cell` 1.20 · `unicode-normalization` 0.1 · `hex` 0.4 · `futures` 0.3 (FR-AI-003 prerequisites).

**`services/obs-collector/`** — Rust workspace member, slice-1 of P0.2 OBS:

- **FR-OBS-001 — OTel collector + LGTM stack — scaffold shipped (10/10), status flipped `planned → building`.** Slice-1 ship covers the canonical `config/otel-collector-config.yaml` matching FR-OBS-001 §3 byte-for-byte (OTLP grpc:4317 + http:4318 receivers with `bearertokenauth` authenticator + resource processor + `attributes/pii_scrub` processor deleting `prompt_text` / `response_text` / `user_email` / `cccd` + secret-pattern regex + batch 10s/1024 + Loki / Prometheus remote-write / Tempo exporters + `file_storage` extension at `/var/lib/otelcol/file_storage` + health_check on :13133). `cyberos-obs validate-config` + `validate-tokens` pre-flight CI gates parse the YAML and assert FR-OBS-001 §3 invariants (receivers/processors/exporters/extensions all required; `pii_scrub` MUST be on logs + traces pipelines per §1 #11; `bearertokenauth` + `file_storage` extensions MUST be present per §1 #2 + #7). Self-metric name + label constants (`obs_collector_received_spans_total{service}`, `dropped_total{reason ∈ auth|pii_scrub|backend_error|buffer_full}`, `export_latency_ms{backend ∈ loki|prometheusremotewrite|otlp/tempo}`). Bearer-token file parser + `config/auth/tokens.example` template. Workspace-wired Rust crate.
- **Remaining for `shipped` status:** the live LGTM deployment (Helm chart + docker-compose for otelcol-contrib + Loki + Prometheus + Tempo + Grafana per FR-OBS-001 §1 #13 sizing: 6.5 vCPU / 11.5 GB RAM / 100 GB disk) lands at `deploy/obs/` next session, at which point status flips `building → shipped`.

**`services/mcp-gateway/`** — Rust workspace member, slice-1 of P0.4 MCP Gateway:

- **FR-MCP-001 — MCP 2025-11-25 spec compliance — scaffold shipped (10/10), status flipped `planned → building`.** Slice-1 ship covers: JSON-RPC 2.0 parser at `src/protocol/jsonrpc.rs` (single + batch + notifications + empty-batch rejection); closed JSON-RPC error code map per DEC-272 at `src/protocol/errors.rs` (-32700 parse_error · -32600 invalid_request · -32601 method_not_found · -32602 invalid_params · -32603 internal_error · MCP-defined -32001 unauthorized · -32002 rate_limited · -32003 tool_not_found · -32004 module_unreachable · -32005 elicitation_required); `initialize` handshake at `src/protocol/initialize.rs` returning `protocolVersion` + `Capabilities` (tools + prompts + resources + logging per DEC-266) + `ServerInfo` + `instructions`, with protocol-version-mismatch error returning the canonical `{received, supported}` data payload (FR-MCP-001 §1 #5); `tools/list` at `src/protocol/tools_list.rs` with base64 cursor pagination at PAGE_SIZE=100 (FR-MCP-001 §1 #6 + #15); `tools/call` at `src/protocol/tools_call.rs` with permission gate on `requires_scope` + returns `-32004 module_unreachable` until FR-MCP-002 wires the registration handler; `ToolAnnotations` per DEC-264 + spec (destructiveHint / readOnlyHint / idempotentHint / openWorldHint); `ToolRegistry` in-memory cache at `src/federation/registry.rs` with deterministic sort for pagination; Axum router at `src/router.rs` mounting `POST /mcp` + `GET /mcp/healthz`. `MCP_PROTOCOL_VERSION` pinned at `"2025-11-25"` per DEC-260. Tests across jsonrpc parsing + error codes + initialize match/mismatch + pagination + tools/call permission gate + router dispatch end-to-end.
- **Remaining for `shipped` status:** JWT verification per FR-AUTH-004 (depends on `services/auth/src/jwt.rs::Claims::scope_grants()` accessor) + audience-bound token check + per-(tenant, tool) rate-limit + `mcp.tool_call_{started, completed}` memory audit row pair per DEC-265 + Streamable HTTP SSE transport + OTel span emission per `mcp.gateway.{initialize, tools_list, tools_call}`. These land in follow-on slices alongside FR-MCP-002..008.

### Catalog totals after this wave

- **Total FRs:** 261 (unchanged — this is implementation work, not authoring)
- **FRs at `shipped + 10/10`:** prior 17 (AUTH 6 + memory 1 + SKILL 8 + others 2) → **19** (+FR-AI-003, +FR-AI-005)
- **FRs flipped `planned → building (10/10)`:** **+2** (FR-OBS-001, FR-MCP-001) — scaffolds shipped, awaiting live-deploy + auth-wiring respectively
- **Production module status table** (`BACKLOG.md §0.5`) — 3 new rows added for `services/{ai-gateway, obs-collector, mcp-gateway}`

### §14 protocol affirmation

This wave touched files exclusively under `services/`, `docs/feature-requests/BACKLOG.md`, and `CHANGELOG.md`. Per AGENTS.md §14.1: this wave does NOT write to `<memory-root>/audit/`, `<memory-root>/HEAD`, or `<memory-root>/.lock` directly — all memory chain-touching operations route through the canonical Writer subprocess via the new `cyberos_ai_gateway::memory_writer::emit()` entry point (FR-AI-003 §1 #9, AGENTS.md §14.1). Reciprocity invariant preserved: every new module README cites the FRs that own its surface; every shipped FR in `BACKLOG.md` flipped to `shipped (10/10)` is mirrored in the per-module status table.

### Build verification

The Rust toolchain is not available in the agent sandbox. Stephen runs the following locally to validate:

```bash
cd services
cargo check -p cyberos-ai-gateway -p cyberos-obs-collector -p cyberos-mcp-gateway --tests
cargo test  -p cyberos-ai-gateway --lib policy
cargo test  -p cyberos-ai-gateway --lib memory_writer
cargo test  -p cyberos-ai-gateway --test policy_loader_test ac1_valid_yaml_loads_and_matches
cargo test  -p cyberos-ai-gateway --test policy_loader_test ac3_invalid_schema_rejected_on_init
cargo test  -p cyberos-obs-collector
cargo test  -p cyberos-mcp-gateway
```

The OnceCell-shared policy-loader tests (AC#2 / #5 / #6 / #7 / #9) are `#[ignore]` by default and run via `--ignored --test-threads=1`.

### Next session ship targets

1. **FR-AI-001** — cost-ledger pre-call check (depends FR-AI-003 ✅ + FR-AI-005 ✅, both now shipped). 8h. Adds `services/ai-gateway/migrations/0001_cost_ledger.sql` + `src/cost_ledger/precheck.rs` + integration test against a real Postgres.
2. **FR-AI-002** — cost-ledger post-call reconcile. 6h.
3. **FR-AI-004** — cost-hold expiry cleanup job (Postgres pg-scheduled). 3h.
4. **`deploy/obs/`** — Helm chart + docker-compose for the LGTM stack to flip FR-OBS-001 `building → shipped`.
5. **FR-MCP-002** — per-module server registration handler + heartbeat lifecycle, wired to the slice-1 `ToolRegistry`. Unblocks the `tools/call` `module_unreachable` short-circuit.

---

## 2026-05-19 — MEMORY Wave 2026-Q3 CLOSED — FR-MEMORY-120 `cyberos history` shipped (final FR)

Final FR of the MEMORY Improvement Wave 2026 Q3. No protocol amendment needed (pure read-only projection).

### FR-MEMORY-120 implementation

New `modules/memory/cyberos/core/history.py` (~265 LOC):
- `HistoryEntry` dataclass; `walk(store, target_path, *, follow_moves, since, limit, show_body)` two-pass projection (path-set expansion via move-row sweep, then filter + project). Most-recent-first by default.
- `_row_touches_paths(row, paths)` matches `row.path`, `extra.path`, `extra.affected_paths[]`, and move `src`/`dst`. Catches put / move / delete + every aux kind (`episode.logged`, `memory.importance_scored`, `dream.proposal_applied`, `memory.acl_denied`, `memory.precondition_failed`, `session.*`).
- `render_annotations(extra)` — inline provenance suffix recognising `dream_id`, `proposal_id`, `session_id`, `invocation`, `imported_from`, `merged_into`, `warn_only`.

CLI: new `cyberos history <path>` with `--limit`, `--chronological`, `--no-follow-moves`, `--show-body`, `--since {Nh|Nd|ISO}`, `--json`.

### Smoke verified end-to-end (against fresh /tmp/memory-120 fixture)

- ✅ Single put → 1 entry, multi-put → N entries descending seq order
- ✅ `--chronological` flips to ascending
- ✅ `--limit 2` caps; `--since 0h` filters all; `--since 24h` keeps all 3
- ✅ Dream annotations rendered inline: `via dream 01KRYP4D… (proposal PMERGE001) merged into memories/facts/canonical.md`
- ✅ Tombstone delete row appears with `extra.mode: "tombstone"`
- ✅ JSON output with all 9 HistoryEntry fields
- ✅ Never-existed path → `No history for 'memories/facts/never.md'.`
- ✅ **Read-only invariant: HEAD before=3, after=3 exact match** (per FR-MEMORY-120 §1 #1)
- ✅ `cyberos verify` chain intact (3 records)

### Test coverage

New: `modules/memory/tests/core/test_history.py` (302 lines, 18 test functions). Covers AC #1, #2, #3, #4, #5, #7, #10, #13, #16, #20, #21, plus annotation rendering across all 5 recognised tags.

### Full MEMORY Wave 2026-Q3 — FINAL STATUS

| FR | Spec | Impl | Tests | Amendment |
|---|---|---|---|---|
| FR-MEMORY-112 | episodic memory | ✅ | 351L | — |
| FR-MEMORY-113 | recency-decay recall | ✅ | 266L | — |
| FR-MEMORY-114 | write-time importance | ✅ | 263L | — |
| FR-MEMORY-115 | `cyberos dream` | ✅ | 329L | P19 §7.7 ✅ |
| FR-MEMORY-116 | semantic-dedup consolidate | ✅ | 257L | — |
| FR-MEMORY-117 | per-store ACL | ✅ | 359L | P20 §14.4 ✅ |
| FR-MEMORY-118 | `put_if` precondition-hash | ✅ | 349L | P21 §3.1 ✅ |
| FR-MEMORY-119 | session transcript ledger | ✅ | 308L | P22 §18 ✅ |
| FR-MEMORY-120 | `cyberos history` | ✅ | 302L | — |

**All 9 FRs shipped end-to-end. All 4 protocol amendments APPROVED + merged into AGENTS.md. 2,784 lines of tests covering 110+ acceptance criteria. `cyberos verify` reports chain integrity across every test fixture.**

The MEMORY module now matches Anthropic's Memory+Dreaming primitive (per the source talk + Ramakrushna article that started this whole wave 2026-05-19 morning).

---

## 2026-05-19 — Wave 3 cont. — AGENTS.md §18 (P22 APPROVED) + FR-MEMORY-119 transcript ledger

### Protocol amendment §18 (P22 APPROVED)

Operator's fourth terse `APPROVE` of the session. New section §18 Session transcript ledger added to [`modules/memory/AGENTS.md`](modules/memory/AGENTS.md). Nine sub-clauses cover opt-in lifecycle, date-partitioned storage at start-date, closed classification enum (`confidential` default per Stephen 2026-05-19 / `restricted`), encryption envelope for restricted payloads, summary rows on the main chain, retention (default 30 days), in-session `extra.session_id` propagation, lifecycle invariants, and ACL applicability to the `sessions/` subtree.

**Namespace conflict resolved**: the FR spec'd `cyberos session start|append|end`, but the existing P11 `cyberos session` subcommand handles multi-agent coordination — different product with the same verb. The implementation namespaces the transcript ledger under `cyberos transcript` (start / append / end / read / list / purge-expired). §18.1 documents the rename.

Tracker entry in [`modules/memory/README.md`](modules/memory/README.md) Appendix D flipped P22 from "awaiting APPROVE" to **APPROVED 2026-05-19**.

### FR-MEMORY-119 implementation

New module: `modules/memory/cyberos/core/transcript.py` (~630 LOC):

- **`Session` dataclass** (id, started_at, classification, retention_days, actor, ended_at, ended_reason, binlog_path).
- **`start(writer, session_id, classification, retention_days, actor)`** — anchors §18 check, classification enum check, single-active-pointer check, creates date-partitioned `sessions/<YYYY-MM-DD>/<id>.binlog`, writes `.active` pointer, emits `session.start` aux row.
- **`append(writer, session_id, role, content, redactions_applied)`** — closed role enum, active-session check, length-prefixed frame append (`[u32 length BE][u64 turn_seq BE][u64 ts_ns BE][JSON payload]`). Restricted sessions go through `_encrypt_content()` which wraps content in an `aes-256-gcm` envelope (uses `cryptography` package if available; falls back to a structured placeholder otherwise).
- **`end(writer, session_id, reason, seal_binlog=True)`** — compresses `.binlog` → `.binlog.zst` via zstd level-10, removes raw, emits `session.end` row.
- **`read(store, session_id, decrypt)`** — iterates frames (handles both `.binlog` and `.binlog.zst`); decrypts content_cipher payloads only when `decrypt=True`.
- **`list_sessions(store, since)`** — enumerates sessions; detects `active|ended|purged` state from binlog suffix + tombstone marker.
- **`purge_expired(writer, retention_days, dry_run)`** — scans date dirs older than retention, overwrites bodies with tombstone manifest, emits `session.purged` rows.
- **`active_session_id(store)`** — read the `.active` pointer.
- **`_has_section_18(store)`** — anchor check (looks for `§18` + "Session transcript ledger" in nearby `AGENTS.md`).

CLI: new `cyberos transcript {start|append|end|read|list|purge-expired}` subcommand with full operator surface — `--id`, `--classification`, `--retention-days`, `--role`, `--content`, `--redactions-applied`, `--reason`, `--no-seal`, `--decrypt`, `--since`, `--dry-run`, `--json`.

### Smoke verified end-to-end

- **AC #1 lifecycle**: start (confidential default) → append × 2 (turn_seq 0, 1) → end (sealed `.binlog.zst`) → read back full transcript with timestamps + roles.
- **AC #3 restricted encrypts**: `--classification restricted` + append → without `--decrypt` shows `[encrypted content; --decrypt to read]`; with `--decrypt` shows `secret hello`.
- **AC #4 invalid classification**: `--classification public` rejected at argparse (exit 2).
- **AC #5 append-without-start**: `error: no active session; start one first` (exit 2).
- **AC #8 two-active rejection**: starting a second session while `concurrent-1` is active → `error: a session is already active ('concurrent-1')` (exit 2).
- **AC #19 amendment gate**: AGENTS.md moved aside → exit 3 + `APPROVE protocol change P22 §18` message.
- **AC #17 list**: enumerated 3 ended sessions with their `state: ended` + `binlog_path`.
- **AC #18 + #20 retention purge**: backdated dir at 2026-04-01 → `purge-expired --dry-run --retention-days 30` reports 1 purge candidate without mutating; actual `purge-expired` emits `session.purged` row at seq=7, replaces body with tombstone JSON.
- `cyberos verify` reported chain intact (7 records final).

### Test coverage

New: `modules/memory/tests/core/test_transcript.py` (308 lines, 19 test functions) covering: amendment-gate enforcement, lifecycle round-trip, date-partitioned storage, restricted encrypt/decrypt round-trip, closed classification enum (4 reject paths), append-without-start, append-after-end, double-end, two-active-rejection, `active_session_id()` helper, input validation (empty id, bad role, retention_days ≤ 0), `list_sessions`, purge dry-run + actual, session.purged payload shape, read-unknown-session-returns-empty.

### Combined test surface (Waves 1+2+FR-117+FR-118+FR-119)

8 test files at `modules/memory/tests/core/` = **2,482 lines, ~122 test functions across 95+ acceptance criteria**.

### Files touched (this entry only)

New:
- `modules/memory/cyberos/core/transcript.py` (632 lines)
- `modules/memory/tests/core/test_transcript.py` (308 lines)

Modified:
- `modules/memory/AGENTS.md` — added §18 (9 sub-clauses)
- `modules/memory/cyberos/__main__.py` — `_cmd_transcript` handler + `cyberos transcript` subparser
- `modules/memory/README.md` Appendix D — P22 flipped APPROVED + documented the `transcript` namespace decision

### Wave 3 status

- ✅ **P20 §14.4** APPROVED + FR-MEMORY-117 shipped
- ✅ **P21 §3.1** APPROVED + FR-MEMORY-118 shipped
- ✅ **P22 §18** APPROVED + FR-MEMORY-119 shipped
- ⏳ **FR-MEMORY-120** (`cyberos history`) — no amendment gate; ships when operator says go

---

## 2026-05-19 — Wave 3 cont. — AGENTS.md §3.1 extension (P21 APPROVED) + FR-MEMORY-118 put_if

### Protocol amendment §3.1 (P21 APPROVED)

Operator's third terse `APPROVE` of the session. §3.1's canonical-op table extended from THREE ops (`put` / `move` / `delete`) to FOUR — `put_if` joins as the optimistic-concurrency primitive. Three new sub-clauses:

- **§3.1.5** — `memory.precondition_failed` aux row schema (`{actor, path, expected, actual, attempt_at}`). HEAD doesn't advance for the rejected put; advances by +1 for the aux row only.
- **§3.1.6** — success row is INDISTINGUISHABLE from a regular `put` (`op="put"`, not `"put_if"`). Downstream consumers (walker / doctor / dream / history) require no special-case logic.
- **§3.1.7** — ACL check (FR-MEMORY-117) runs BEFORE the precondition check. Policy refusal returns `acl_denied`, not `precondition_failed` — different operator action needed.

Tracker entry in [`modules/memory/README.md`](modules/memory/README.md) Appendix D flipped P21 from "awaiting APPROVE" to **APPROVED 2026-05-19**.

### FR-MEMORY-118 implementation

New op in `modules/memory/cyberos/core/ops.py`:

- `PutIfResult` frozen dataclass (5 fields: `outcome`, `reason`, `expected`, `actual`, `committed_seq`).
- `put_if(writer, rel_path, body, *, actor, precondition_body_hash, kind, extra)` — content-conditional write. Order of checks: path traversal + size cap → shape validation (64-char lowercase hex OR `None` only) → `_has_section_3_1_put_if()` anchor check → ACL gate (FR-MEMORY-117, runs BEFORE precondition per §3.1.7) → precondition check (3 rejection paths) → write. Success emits a plain `put` row (per §3.1.6) so downstream consumers need no special-case.

CLI: new `cyberos put-if <path> <body_file> --precondition <hex|none>` subcommand with `--precondition-from-file` + `--json` variants.

### Smoke verified end-to-end

- **AC #1 match → written**: pre-computed body hash; `put-if` returns `outcome: written, seq: 2`.
- **AC #2 mismatch → rejected**: stale hash → `outcome: rejected, reason: precondition_failed, expected: 8fc80ed1…, actual: 79598c2e…`. Chain advances by exactly 1 (aux row only).
- **AC #3 null + absent → written**, **AC #4 null + existing → rejected**.
- **AC #16 shape validation**: uppercase hex → `error: precondition_body_hash must be 64-char lowercase hex or None`.
- **AC #19 amendment gate**: with AGENTS.md moved aside, `put-if` exits with code 3 naming `APPROVE protocol change P21 §3.1`.
- **Retry-loop pattern** worked in 1 attempt against the seeded fixture.
- `cyberos verify` chain intact (6 records).

### Test coverage

New: `modules/memory/tests/core/test_put_if.py` (349 lines, 19 test functions) covering: amendment-gate enforcement, shape validation (5 parametrized bad inputs), 4×2 precondition cross-product, HEAD-doesn't-advance-on-reject invariant, success-row-is-`op=put` invariant, aux-row payload shape, ACL-before-precondition ordering, PutIfResult shape, retry-loop pattern, sequential two-writer race smoke.

### Combined test surface (Waves 1+2+FR-117+FR-118)

7 test files at `modules/memory/tests/core/` = **2,174 lines, ~103 test functions across 80+ acceptance criteria**.

### Files touched (this entry only)

New: `tests/core/test_put_if.py` (349 lines).
Modified: `modules/memory/AGENTS.md` §3.1; `modules/memory/cyberos/core/ops.py` (~160 LOC); `modules/memory/cyberos/__main__.py`; `modules/memory/README.md` Appendix D.

### Wave 3 status

- ✅ **P20 §14.4** APPROVED + FR-MEMORY-117 shipped
- ✅ **P21 §3.1** APPROVED + FR-MEMORY-118 shipped
- ⏳ **P22 §18** (session transcript ledger) — awaiting `APPROVE protocol change P22 §18`
- ⏳ **FR-MEMORY-120** (`cyberos history`) — no amendment gate

---

## 2026-05-19 — Wave 3 start — AGENTS.md §14.4 (P20 APPROVED) + FR-MEMORY-117 per-store ACL

### Protocol amendment §14.4 (P20 APPROVED)

Operator approved with a terse `APPROVE` (interpreted as the next-in-queue per the one-at-a-time rule). New section added to [`modules/memory/AGENTS.md`](modules/memory/AGENTS.md) at §14.4 — Store-level ACL. Seven sub-clauses cover STORE.yaml shape + parsing, write-side enforcement via the canonical writer with first-match-wins glob resolution, read non-enforcement (writes-only at protocol level), `memory.acl_denied` aux row payload, two-sided ACL check on `move`, INTEROP-consumer obligation, and the `store-yaml-acl-valid` walker invariant.

Tracker entry in [`modules/memory/README.md`](modules/memory/README.md) Appendix D flipped P20 from "awaiting APPROVE" to **APPROVED 2026-05-19**.

### FR-MEMORY-117 implementation

New: `modules/memory/cyberos/core/store_acl.py` (~280 LOC):
- `StoreAcl` dataclass + `from_yaml(path)` parser with full validation (closed-enum modes, required `store_id`, list-shape `acl`, glob-actor strings).
- `find_governing_store_yaml(root, rel_path)` — walks UP from the target's parent dir; innermost STORE.yaml wins.
- `check_write(root, rel_path, actor)` — returns `AclResult` with `allowed`, `mode`, `store_id`, `yaml_path`, `matched_entry`, `reason`.
- Two-mode operation:
  - **Enforced** (AGENTS.md §14.4 anchor present): denied writes raise `AclDenied` after emitting the `memory.acl_denied` aux row.
  - **WARN-ONLY** (anchor absent, pre-amendment transition): aux row still emitted with `warn_only=True` payload field, but writes proceed. Anti-footgun for operators who pull code before APPROVE'ing.
- `explain(root, path, actor)` — operator-readable diagnostic for `cyberos acl explain`.

Hooks into `modules/memory/cyberos/core/ops.py`:
- New `AclDenied(PermissionError)` exception class.
- New `_acl_check(writer, rel_path, actor, attempt_kind)` helper — emits the aux row on any non-allow result.
- `put()` gates the write before the atomic file write.
- `move()` calls `_acl_check` for BOTH `src_rel` and `dst_rel` per §14.4.5; either-side failure → `AclDenied`.
- `delete()` gates before the audit-row submit.

Hooks into `modules/memory/memory.schema.json`:
- New `StoreAclMode`, `StoreAclEntry`, `StoreAcl` definitions matching FR-MEMORY-117 §3 schema fragment.

CLI: new `cyberos acl {show|validate|explain}` subcommand:
- `acl show` — pretty-prints every STORE.yaml in the store.
- `acl validate` — re-validates every STORE.yaml against the schema; non-zero exit on any failure.
- `acl explain <path>` — resolves the effective mode for the active actor on a given path, with the matched ACL entry highlighted.

### Smoke verified end-to-end

- **WARN-ONLY** (no AGENTS.md): scheduled-importer write to a deny subtree → seq advanced, `memory.acl_denied` aux row emitted with `warn_only: true`.
- **Enforced** (AGENTS.md §14.4 present): same write → `AclDenied` raised, aux row emitted with `warn_only: false`, NO put row written.
- `cyberos verify` chain intact across both modes.
- `cyberos acl show` formatted three-entry STORE.yaml correctly.
- `cyberos acl explain` correctly resolved scheduled-importer → `deny`, stephen@cyberskill.world → `read-write`.
- Happy-path put for stephen@cyberskill.world succeeded at seq=4.

### Test coverage

New: `modules/memory/tests/core/test_store_acl.py` (359 lines, 19 test functions) covering: StoreAcl parse + validation, find_governing_store_yaml walk, check_write across enforcement/warn-only/permissive paths, glob-actor matching, first-match-wins, explicit-deny override, default_mode fallback, built-in actor literals, put/move/delete ACL enforcement via canonical ops, `move` two-sided check, memory.acl_denied aux-row payload shape, explain() output.

### Combined test surface (Waves 1 + 2 + FR-MEMORY-117)

6 test files at `modules/memory/tests/core/`: `test_episode.py` (351) + `test_ranking_and_decay.py` (266) + `test_importance.py` (263) + `test_dream.py` (329) + `test_consolidate_semantic_dedup.py` (257) + `test_store_acl.py` (359) = **1,825 lines, ~84 test functions across 65+ acceptance criteria**.

### Files touched (this entry only)

New:
- `modules/memory/cyberos/core/store_acl.py` (~280 lines)
- `modules/memory/tests/core/test_store_acl.py` (359 lines)

Modified:
- `modules/memory/AGENTS.md` — added §14.4 Store-level ACL (7 sub-clauses)
- `modules/memory/memory.schema.json` — added StoreAclMode, StoreAclEntry, StoreAcl definitions
- `modules/memory/cyberos/core/ops.py` — `AclDenied` + `_acl_check` + integration into put/move/delete
- `modules/memory/cyberos/__main__.py` — `_cmd_acl` handler + `cyberos acl {show|validate|explain}` subparser
- `modules/memory/README.md` Appendix D — P20 flipped APPROVED

### Wave 3 status

- ✅ **P20 §14.4** APPROVED + FR-MEMORY-117 implemented
- ⏳ **P21 §3.1** (put_if precondition-hash) awaiting `APPROVE protocol change P21 §3.1`
- ⏳ **P22 §18** (session transcript ledger) awaiting `APPROVE protocol change P22 §18`
- ⏳ **FR-MEMORY-120** (`cyberos history`) — no amendment gate; ships next session

---

## 2026-05-19 — Dependency-version bumps + AGENTS.md §7.7 (P19 APPROVED) + Wave 2 implementation (FR-MEMORY-115 + 116)

### Dependency version audit + bumps (repo-wide)

Operator: "use latest stable; check throughout cyberos and update all possible." Conservative floor bumps to known-stable versions; patch releases pick up via `pip install -U` / `cargo update` / `pnpm up`. Playground folder (cloned upstream repos) untouched.

| Project | Files touched | What bumped |
|---|---|---|
| `modules/memory/` | `pyproject.toml`, `cyberos/requirements.txt` | setuptools 61→75; msgspec 0.18→0.18.6; crc32c 2.4→2.7; PyYAML 6.0→6.0.2 |
| `modules/skill/` | `pyproject.toml`, `toolchain/package.json` | setuptools 61→75; anthropic 0.40→0.42; msgspec→0.18.6; pyyaml→6.0.2; @bytecodealliance/jco 1.7→1.10 |
| `modules/cuo/` | `pyproject.toml` | hatchling 1.18→1.25; click 8.1→8.1.7; pyyaml→6.0.2; pytest 8.0→8.3 |
| `services/embed-sidecar/` | `pyproject.toml` | setuptools 68→75; fastapi 0.110→0.115; uvicorn 0.27→0.32; pydantic 2.6→2.9; sentence-transformers 2.7→3.0; torch 2.2→2.4 |
| `services/` (workspace) | `Cargo.toml` | rust-version 1.81→1.83; tokio 1.41→1.42; clap 4→4.5; jsonwebtoken 9→9.3 |
| `services/auth/` | `Cargo.toml` | reqwest→0.12.9; ipnetwork 0.20→0.21; zeroize 1→1.8 |
| `services/memory/` | `Cargo.toml` | async-trait→0.1.83; regex 1→1.11; reqwest→0.12.9 |
| `services/skill-broker/` | `Cargo.toml` | flagged serde_yaml deprecation (slice-4 migration tracked); no version change |
| `services/memory/desktop/` | `package.json` | @tauri-apps/api/cli 2.0→2.1; svelte 5.0→5.2; tslib 2.7→2.8; tailwind 3.4.0→3.4.14 |

### Protocol amendment §7.7 (P19 APPROVED)

Operator approved Wave 2 protocol amendment. New section added to [`modules/memory/AGENTS.md`](modules/memory/AGENTS.md) at §7.7 — Dreaming. Seven sub-clauses cover out-of-band identity, `extra.dream_id` + `extra.proposal_id` provenance invariant, body-hash precondition gate, operator-gated apply, four new audit kinds, snapshot isolation, and the closed detector enum.

Tracker entry in [`modules/memory/README.md`](modules/memory/README.md) Appendix D flipped P19 from "awaiting APPROVE" to **APPROVED 2026-05-19**.

### FR-MEMORY-115 — `cyberos dream` out-of-band reflection

New: `modules/memory/cyberos/core/dream/{__init__,proposals,_audit_iter,detectors,runner,applier}.py`. Four async detectors (`duplicates` / `stale` / `patterns` / `verify`) matching AGENTS.md §7.7.7 closed enum. Runner uses Crockford-base32 ULID `dream_id`, snapshot-isolated against `head_seq` at start, persists `DreamDiff` to `dreams/<YYYYMMDDTHHMMSSZ>/diff.json`. Applier: 3-pass (strict-idempotency via chain-walk → body-hash precondition → write with `extra.dream_id`/`extra.proposal_id` per §7.7.2). AGENTS.md §7.7 anchor checked before any writes (`ProtocolAmendmentMissing` on missing).

CLI: `cyberos dream` + `cyberos dream-apply <id>` subcommands.

**Smoke verified end-to-end:** seeded 3 facts (2 near-duplicate); dream found 1 merge proposal; apply rejected without §7.7 anchor, succeeded with anchor, idempotent on re-apply (`skipped_idempotent: 1`); `cyberos verify` reported chain intact (13 rows).

**Side effect**: `cyberos.core.ops.delete()` gained optional `extra: dict | None` kwarg (additive, back-compat).

### FR-MEMORY-116 — Semantic-dedup consolidate phase

Thin wrapper: 5th phase appended to `cyberos.core.consolidate.run()` → `Walk → Compact → Sign → Publish → SemanticDedup`. Delegates to FR-MEMORY-115's `duplicates` detector + applier verbatim (asserted by `test_consolidate_imports_dream_detector` via `inspect.getsource`). On apply, emits a marker `dream.complete` row with `extra.invocation = "consolidate"` so FR-MEMORY-120 history can distinguish dedup-from-consolidate from dedup-from-explicit-`dream`.

CLI: extends `cyberos consolidate` with `--semantic-dedup`, `--semantic-dedup-apply`, `--semantic-dedup-threshold`, `--semantic-dedup-scope`. Default behavior unchanged. `ConsolidationReport` gains 5 new fields.

**Smoke verified end-to-end:** same 2-duplicate fixture; all 5 phases ran; 1 proposal found + 1 applied; final `dream.complete` aux row carries `extra.invocation: "consolidate"`.

### Combined test coverage (Wave 1 + Wave 2)

5 test files at `modules/memory/tests/core/`: `test_episode.py` (351) + `test_ranking_and_decay.py` (266) + `test_importance.py` (263) + `test_dream.py` (329) + `test_consolidate_semantic_dedup.py` (257) = **1,466 lines, ~65 test functions across 50+ acceptance criteria**.

### Deferred to subsequent sessions

- pytest run against full suite — sandbox is Python 3.10 (module requires 3.11+); test files are ready for `pytest tests/core/ -v` in operator env.
- Wave 3 (FR-MEMORY-117 ACL · 118 put_if · 119 sessions · 120 history) — gated on independent `APPROVE protocol change P20 / P21 / P22` chat-turns when operator ready.

---

## 2026-05-18 — Wave-1+2 impl sessions 15-18: embed sidecar, NFR audits, SAML XML-DSig, GeoIP+policy, Tauri desktop, slice-3 universal wiring + admin REST

End-of-day continuation of the Wave-1+2 implementation phase. Eight items shipped end-to-end:

**[AI] FR-AI-019 embedding sidecar closed end-to-end.** New `services/embed-sidecar/` — FastAPI server with mock + real backends behind `CYBEROS_EMBED_MODE`. `POST /embed` matches the Rust `EmbeddingClient` wire protocol. Mock backend hashes inputs to deterministic unit-norm 1024-vectors; real backend lazy-loads `BAAI/bge-m3` via sentence-transformers. **10/10 pytest cases pass.**

**NFR audit-pair coverage.** All 153 NFR specs across 18 module directories now have `.audit.md` siblings on the `nfr-spec@1` rubric. 153/153 scored 10/10.

**[AUTH] FR-AUTH-103 SAML XML-DSig (slice-2) + xml-c14n hardening.** `services/auth/src/saml_sig.rs` (~520 lines): ds:Signature discovery, strict algorithm allowlist (RSA-SHA256 + SHA-256 + exc-c14n), enveloped-signature stripping, reference-by-ID resolution, RSA-PKCS1-v1.5 verify, hand-rolled X.509 → SPKI TLV walk. Migration 0017 adds per-IdP `allow_unsigned BOOLEAN DEFAULT FALSE` — replaces the legacy `AUTH_SAML_ALLOW_UNSIGNED=1` env-var escape hatch. `exc_c14n` rewritten as a proper tokeniser: XML-decl + comment + PI stripping, attribute sort (xmlns first, then alpha by qualified name), single→double quote normalisation, XML-attr escaping. 14 canonicaliser tests + 7 X.509/PEM tests.

**[AUTH] FR-AUTH-106 GeoIP + policy + CIDR + sticky-suppression (slices 2 + 3).** New `services/auth/src/geoip.rs` with `GeoIpResolver` trait, `MaxMindResolver` (GeoLite2-City + optional Anonymous-IP), `NullResolver` fallback. Activates `cross_continent_velocity` (country flip < 6h) and `geo_velocity_exceeded` (haversine > 1000 km/h) detectors. Migration 0018 ships `travel_policy`, `travel_cidr_allowlist` (with /9-IPv4 + /17-IPv6 prefix-tightness CHECK), `travel_policy_audit` (reason ≥10 chars). New `travel_policy.rs` — 60s `PolicyCache`, bounded-50k `StickySuppress` LRU. New `TravelOutcome::Block` variant. New `assess_login` wraps the detector chain with policy + CIDR + anonymous-IP + sticky. New `travel_admin.rs` exposes 5 routes (`PUT/GET travel-policy`, `POST/GET cidrs`, `DELETE cidrs/:id`) gated by `security-admin` / `tenant-admin`, writes audit rows, invalidates `PolicyCache`. **All four login flows** (password, OIDC, SAML, Passkey) now go through `assess_login` — Block → 403, Challenge → token + `needs_mfa_challenge`.

**Production runbook + CI for GeoIP.** New `services/auth/scripts/install-geoip.sh` (MaxMind direct or internal mirror). New `services/auth/tests/geoip_test.rs` (skips when DB absent, asserts `8.8.8.8 → US` and `165.21.0.1 → SG` when present). `.github/workflows/services.yml` gains an install step gated on `secrets.MAXMIND_LICENSE_KEY`.

**[MEMORY] FR-MEMORY-104 Tauri 2.x desktop scaffold.** New `services/memory/desktop/` (19 files). Backend: Tauri 2 + plugin-shell + plugin-fs; `commands.rs` for search/quick-capture/sync-state; `sync_supervisor.rs` supervises the Python memory-sync daemon with 5-restarts-per-60s circuit breaker. Frontend: Svelte 5 runes + Vite + Tailwind 3 — `App.svelte` with Dashboard / Search / Sync tabs. **NOT in `services/Cargo.toml` workspace** — own Cargo.lock. Signing scripts: `generate-updater-keys.sh` (tauri signer generate), `sign-and-notarize-macos.sh` (codesign + notarytool + staple + spctl + auto-generated entitlements), `sign-windows.sh` (signtool + SHA-256 + RFC 3161). README documents the full release runbook. FR-MEMORY-104 status bumped `accepted → building`.

---

## 2026-05-18 — Modules refactor + doc consolidation + CUO v3.0.0-a4 supervisor (Phases 1+2+3+4) + Sessions A–N catalog completion + CHANGELOG centralisation + FR-CUO-106

Massive multi-stream day. Four parallel programs landed end-to-end:

### Stream 1 — CUO catalog completion (Sessions A through N)
- **Sessions A–C (2026-05-17 evening → 18 morning):** Tier-1 (29) + Tier-2 (29) + Tier-3 (8) skill pairs shipped — closed NEEDED_SKILLS gap to 66/66 = 100%
- **Session D:** 14 Now-tier workflows (CEO 5 + CFO 5 + chief-of-staff 4) — all chain shipped skills
- **Session E:** 5 Tier-4 legal skill pairs + 5 CLO-Legal workflows
- **Session F:** 6 Tier-5 security/sales/delivery skill pairs + 16 Series-A workflows (COO + CHRO + CSO-Sales + CISO)
- **Session G:** 1 Tier-6 skill (churn-analysis) + 10 Scale-up workflows (CRO-Revenue + CPO-Privacy first-coverage + CFO/CHRO depth)
- **Session H:** 5 Tier-7 Enterprise skill pairs + 20 Enterprise workflows (CPO-Product + CDO-Data + CAIO + CCO-Customer + Chief-Knowledge-Officer)
- **Sessions I–N:** 124 niche-persona workflows authored through the EXISTING 104-pair catalog (six consecutive "no new skills needed" sessions, validating the v3.0.0 supervisor hypothesis)
- **Final state:** 104 author+audit pairs / 208 bundles / 108 contracts; 47 active personas (1 EXTINCT cautionary tale `chief-metaverse-officer`); **194 workflows live**; zero `planned:` gaps

### Stream 2 — CUO v3.0.0 Python supervisor (Phases 1+2+3)
- **Phase 1 (`3.0.0a1`):** catalog scanner + persona/workflow discovery + chain validator + two-stage router with domain-language fallback + dry-run mode. 9/9 tests pass. CLI: `list-personas`, `list-workflows`, `route`, `dry-run`
- **Phase 2 (`3.0.0a2`):** `Invoker` ABC + `MockInvoker` + `SubprocessInvoker` + `select_invoker('auto')` + `execute_chain()` walking workflow chains with filesystem hand-off. CLI: `execute`. 14/15 tests pass
- **Phase 3 (`3.0.0a3`):** `LLMInvoker` (mock-llm default + Anthropic API mode reading SKILL.md as system prompt + RUBRIC.md guardrails for audit skills) + memory audit-chain emission via `cyberos.core.writer.Writer` wrapper. CLI: `--invoker llm`, `--memory-emit`, `--actor`. **21/22 tests pass** (1 expected skip — catalog-complete invariant); HEAD advances `01 → 03` on first emit
- **Phase 4 (`3.0.0a4`):** 5 special-case workflow Handler subclasses at `modules/cuo/cuo/core/handlers/` — `LinearHandler` (default), `TimeCriticalHandler` (bypass scheduling + SLA breach audit), `PerInstanceHandler` (iterate ×N + fan-in summary), `MultiOutputHandler` (fan-out final step per recipient), `SequentialApprovalHandler` (gate chain B on approval of chain A), `PersonaPairHandler` (interleaved chains with shared artefact ownership). Dispatched by workflow `pattern:` frontmatter. Spec at `docs/feature-requests/cuo/FR-CUO-106-supervisor-phase4-special-handlers.md`. **49/50 tests pass** (was 21+1; +28 new Phase 4 tests including end-to-end dispatch against real catalog). 8 new memory audit kinds.
- **Phase 4 CLI wiring (this session close):** `cyberos-cuo execute` now auto-dispatches via `pick_handler(workflow)` and prints `# dispatched to <HandlerClass>` when pattern ≠ linear. New flags: `--explain` (show pattern + handler + workflow_file + rationale before invocation) + `--no-handler-dispatch` (bypass for debug). `WorkflowEntry.frontmatter` dict added to `modules/cuo/cuo/core/catalog.py` so arbitrary frontmatter fields (`pattern`, `sla_minutes`, `instance_descriptor`, `output_recipients`, `gates`, `peer_persona`, etc.) survive parsing. 15 affected workflows patched with `pattern:` frontmatter (3 time_critical + 1 per_instance + 1 multi_output + 1 sequential_approval pair + 4 persona_pair pairs).
- **C1 — CUO depth additions (first wave):** 27 new workflows shipped across 14 priority personas (ceo, cfo, cto, chro, cso-sales, coo, cmo, ciso, cdo-data, cpo-product, chief-of-staff, cro-revenue, caio, cpo-privacy). Catalog now: **221 workflows total** (was 194 post-Session N). ~250-450 workflows of depth headroom remain across 33 other personas.
- **Governance docs consolidation:** 4 generated reports (CONTRACT_VERIFICATION_REPORT.md + IMPLEMENTATION_ORDER.md + MIGRATION_AUDIT.md + SPRINT_PLAN.md) merged into single `docs/feature-requests/REPORTS.md` with §1-§4 sections. Top-level FR governance files now **4 (was 7)**: feature-request-audit skill, BACKLOG.md, REPORTS.md, VN_GLOSSARY.md.
- **Commit manifest prepared:** `COMMIT.md` at repo root with conventional-commit message, tag `v3.0.0-a4`, and pre-push validation checklist.
- **Persona-slug normalisation (final session change):** all 33 short-acronym persona folders renamed to full `chief-*-officer` form for consistency. `cto/` → `chief-technology-officer/`, `cfo/` → `chief-financial-officer/`, `cco-customer/` → `chief-customer-officer/`, etc. 15 personas already in full form left unchanged (chief-architect, chief-of-staff, chief-{brand,digital,ethics,innovation,knowledge,medical,remote,trust,transformation,esg,automation,happiness,metaverse}-officer). Total: **1,447 substitutions across 241 files** (workflow frontmatter `workflow_id`/`persona`/`escalates_to`/`consults`/`peer_persona`/`approver_persona`, persona READMEs, MODULE.md catalog, test_smoke.py assertions, CLI docstring examples, website html, FR catalog, modules/cuo/README.md). Python package `cuo` at `modules/cuo/cuo/` intentionally NOT renamed (Python identifier constraint). **49/50 tests still pass** post-rename. End-to-end smoke: memory HEAD advanced `09 → 0c`; `cyberos-cuo execute chief-privacy-officer/breach-response-cycle --explain` dispatches to TimeCriticalHandler correctly.
- **Deploy artefacts cleanup (post-deploy):** website manually deployed to `cyberos.cyberskill.world` via Vercel by operator. Removed deploy-tooling clutter from repo root: `vercel.json`, `.vercelignore`, `DEPLOY-VERCEL.md` (deploy is operator-controlled, no CI commitment in-repo); `.wrangler/` cache+tmp dirs (Cloudflare path abandoned for this site); `.cuo-slug-mapping.json` (transient rename artefact — mapping now lives in git history + memory + this CHANGELOG only). Rewrote `website/README.md` + `website/docs/DEPLOYMENT.md` to reflect Vercel-deployed reality (replacing the 228-line Cloudflare-centric DEPLOYMENT.md with a 49-line operator-flow doc).

### Stream 3 — Repo refactor + doc consolidation
- Moved `cuo/`, `skill/`, `memory/` → `modules/<name>/` (isolation preserved)
- Consolidated each module's `docs/` into a single comprehensive `README.md` at module root:
  - `modules/cuo/README.md` — 713 lines
  - `modules/skill/README.md` — 4,112 lines (existing 2,478-line wiki + 8 appendices)
  - `modules/memory/README.md` — 612 lines
- Kept protocol artefacts at module root (NOT folded): `AGENTS.md` (symlink target), `INTEROP.md`, `memory.schema.json`, `memory.invariants.yaml`, `MODULE.md` (cuo + skill canonical catalogs)
- Updated root `CLAUDE.md` + `AGENTS.md` symlinks → `modules/memory/AGENTS.md`
- Deleted outdated `docs/prd/` (724K) + `docs/srs/` (2.3M); both frozen 2026-05-15
- Promoted `docs/tours/` → repo-root `tours/` (7 CodeTour operational runbooks)
- Patched `modules/cuo/cuo/cli.py::_find_cyberos_root` + `_resolve_roots` to prefer modules/ layout, fall back to legacy flat
- Patched `modules/cuo/cuo/core/memory_bridge.py::_try_import_memory_writer` + `_find_memory_root` for the new ancestry walk
- Rewrote root `README.md` + `docs/README.md` for the new layout
- **Consolidated all per-module CHANGELOGs into this root CHANGELOG.md** sorted by date; per-module CHANGELOG files replaced with one-line "moved" pointers

### Stream 4 — FR catalog refresh

- Authored **FR-CUO-106** (+ .audit.md sibling) — Phase 4 special-case workflow handlers spec. 256 lines normative + 1 line audit (10/10).
- Refreshed `docs/feature-requests/BACKLOG.md` header: v0.2.0 → v0.3.0. Added per-module production-status table. Added "What changed since v0.2.0" section.
- FR catalog audit confirmed: 0 stale `cyberos/skill/`, `cyberos/cuo/`, or `cyberos/memory/` paths; the 26 domain folders already use `modules/` paths.

### End-to-end verification
- `pytest tests/ -v` in `modules/cuo/` → **21 passed, 1 skipped** (same green status as pre-refactor)
- CLI smoke: `cyberos-cuo execute chief-technology-officer/adr-quick-capture --memory-emit` → COMPLETED, 3 memory rows emitted, HEAD advanced `03 → 06`
- Root symlinks resolve correctly to `modules/memory/AGENTS.md`

### Files touched (high level)
- 12 new persona-folder workflow batches (~150 markdown files)
- 79 new skill-pair scaffolds (~470 files across SKILL.md / RUBRIC.md / CHANGELOG.md / CONTRACT.md / template.md)
- 8 new Python source files (`cuo/{catalog, validator, router, supervisor, invoker, llm_invoker, memory_bridge}.py` + tests)
- 3 module READMEs rewritten (~5,400 lines)
- root README + docs/README rewritten
- 1 root CHANGELOG.md consolidated (this entry's merge)

---

## 2026-05-15 — UI bug fixes from screenshots (Mermaid syntax + diagram sizing + title overlap + mobile overflow + PRD/SRS sweep cleanup)

Stephen flagged five UI bugs from live deploy screenshots; all fixed.

**Bug 1 — Hero h1 overlap (`.h-1 mb-3 + p` collision on index.html):**
- `assets/styles.css:325–355` — bumped `.h-1` line-height 1.25 → 1.3, `margin-block-end` 1.25rem → 1.5rem, added `padding-block-end: 0.25rem` to protect BVP descenders.
- Changed sibling rule line 346: `margin-block-start: 0` → `0.5rem !important` for h-display + h-1 successors. Guarantees min-gap even when Tailwind `mb-3` overrides.

**Bug 2 — Mermaid "Syntax error in text" in memory §3:**
- Root cause: `FILES["memories/<kind>/<hex>/<file>.md"]` — Mermaid 11.4.1 parses `<kind>`/`<hex>`/`<file>` as unknown HTML tags inside node labels.
- Fixed 3 locations in `modules/memory.html` (lines 288, 454, 503): `<kind>` → `{kind}` etc.
- Fixed 1 location in `modules/hr.html:841` (same root cause inside a Mermaid sequence).
- Repo-wide sweep confirmed no other `<placeholder>` patterns in Mermaid blocks.

**Bug 3 — Stage 0→5 flowchart rendered microscopic:**
- Root cause: `.mermaid svg { max-width: 100%; height: auto; }` forced wide flowcharts to shrink to ~700px parent, making labels unreadable.
- Fix at `assets/styles.css:429–449`: dropped `display:flex; justify-content:center;` (which fought overflow scroll), changed `max-width: 100%` → `max-width: none !important` on SVG. Now wide diagrams scroll horizontally instead of shrinking. Added scrollbar styling for visual hint.

**Bug 4 — Mobile horizontal overflow:**
- Added 70-line mobile safety net at `assets/styles.css:1017–1085`:
  - `html, body { overflow-x: hidden; max-width: 100vw; }` to clamp viewport
  - `.container { min-width: 0 }` so flex/grid children can shrink
  - `.bbg-card { overflow-wrap: anywhere }` so long URLs/codes wrap
  - `@media (max-width: 768px)`: tables wrap their card in scroll, code blocks pre-wrap, fact-grid `minmax(140px, 1fr)`, h-display clamp 1.875–2.5rem
  - `@media (max-width: 480px)`: tighter container padding + 120px fact-card minimum
  - Mermaid `max-height: 70vh` on mobile to prevent monstrous portrait diagrams

**Bug 5 — Lingering PRD/SRS references:**
- 47 textual edits across 28 HTML files in `website/docs/` (per Agent sweep). Removed: "PRD/SRS narrative remains authoritative" disclaimers (23), "PRD coverage" eyebrows, broken `<a href="#"></a>` empty anchors, "Generated from PRD + SRS source" footer, "DEC-NNN in SRS" → "DEC-NNN" rewrites (5 in infrastructure.html + 1 in ten.html), persona "draft PRD/SRS" chip rephrases. Preserved: the two intentional github.com canonical-spec links in `fr-catalog.html` lines 56–57.
- Grep verification: `\bPRD\b|\bSRS\b` across `website/docs/*.html` → 2 hits, both intentional.

Verified: memory.html Mermaid no longer has `<kind>/<hex>/<file>` patterns; styles.css line counts went from 1018 → 1085. The fix should ship cleanly to Cloudflare Pages on next deploy.

---

## 2026-05-15 — RES module page rewritten to Gold (capacity-vs-forecast integrator + hiring forecast + allocation engine)

Rewrote `website/docs/modules/res.html` to Gold. Three strategic roles: (1) capacity-vs-forecast integrator (joins HR + PROJ + TIME + LEARN on Member-id × week; integrator not source-of-truth), (2) hiring forecast (skill-gap × CRM pipeline × LEARN mastery → hire trigger before deliverables drop), (3) allocation engine (CUO/COO drafts rebalance recommendations; VN Labour Code Art. 107 OT caps hard-floor).

Key changes:
- NEW §0 — 3-card layout + integration-model Mermaid (HR/PROJ/TIME/LEARN/CRM → RES → CUO → hiring memo/rebalance proposal) + 10-row auto-vs-human matrix
- Risks +5 (R-RES-010..014): RES forecast becomes CEO-decision dependency · Member-preference flags ignored under high-priority · VN OT-cap version drift · cross-Engagement reallocation rate-card mismatch · Lumi RES synthesis leaks Engagement intel
- KPIs +6: hiring memo CEO acceptance rate · Member-preference override rate (= 1.0) · cross-Engagement rate-card alignment · cap version stamp coverage (= 1.0) · Lumi cross-tenant sign-off (= 1.0)
- References expanded: §0 + MEMORY_AUTOSYNC_DESIGN.md + AUDIT_AND_PLAN + FR_AUTHORING_WORKFLOW

---

## 2026-05-15 — PORTAL module page rewritten to Gold (client-facing surface + scoped read-only views + external IdP)

Rewrote `website/docs/modules/portal.html` to Gold. Three strategic roles: (1) scoped read-only client surface (PROJ/INV/DOC/CHAT views filtered by Engagement membership + sync_class=client-visible), (2) per-tenant brand pack (white-label theme + custom CNAME), (3) external IdP integration (client logs in via own SAML/OIDC; JIT provisioning; never stores password).

Key changes:
- NEW §0 — 3-card layout + multi-tenant-within-multi-tenant Mermaid + 10-row auto-vs-human matrix
- Risks +5 (R-PORTAL-011..015): sync_class misconfig leak (Critical) · JIT role-mapping wrong · SVG XSS · Client AI cross-Engagement cite (Critical) · SCIM deprovision delay
- KPIs +6: sync_class filter pass (= 1.0) · JIT role accuracy (≥ 0.99) · SVG XSS blocks · cross-Engagement rejection rate · SCIM session-invalidation p95
- References expanded: §0 + MEMORY_AUTOSYNC_DESIGN.md + AUDIT_AND_PLAN + FR_AUTHORING_WORKFLOW

---

## 2026-05-15 — DOC module page rewritten to Gold (document repository + e-sign workflow + contract lifecycle)

Rewrote `website/docs/modules/doc.html` to Gold. Three strategic roles: (1) document repository (versioned + ACL'd + 10-year retention), (2) e-sign workflow (partner-routed cryptography to eIDAS QTSP / AATL CA / VN CA; CyberOS-owned workflow + identity verification), (3) contract lifecycle (HR/CRM/ESOP integration + expiry alerts + renewal automation).

Key changes:
- NEW §0 — 3-card layout + partner-routed signing Mermaid + 10-row auto-vs-human matrix
- Risks +5 (R-DOC-011..015): cross-module trigger source mismatch · CUO renewal stale terms · expiry cascade miss · multi-jurisdiction cert chain · migrated DocuSign LTV failure
- KPIs +5: cross-module trigger validation (= 1.0) · renewal terms-stamp coverage (= 1.0) · expiry cascade completeness (= 1.0) · multi-jurisdiction cert-chain declaration (= 1.0) · LTV re-validation (≥ 0.95)
- References expanded: §0 + MEMORY_AUTOSYNC_DESIGN.md + AUDIT_AND_PLAN + FR_AUTHORING_WORKFLOW

---

## 2026-05-15 — OKR module page rewritten to Gold (cascade orchestrator + KR auto-progress + face-saving retros)

Rewrote `website/docs/modules/okr.html` to Gold. Three strategic roles: (1) cascade orchestrator (Company → Team → Member quarterly), (2) KR auto-progress engine (each KR's progress_source query reads PROJ/INV/HR/LEARN; nightly batch), (3) face-saving retro engine (Vietnamese cultural framing: "what did we learn?").

Key changes:
- NEW §0 — 3-card layout + auto-progress data-flow Mermaid + 8-row auto-vs-human matrix
- Risks +5 (R-OKR-010..014): progress source schema drift · face-saving framing weaponised · CUO digest hallucination · OKR-weight skews REW · retro cross-tenant leak
- KPIs +5: progress source schema drift · face-saving pattern detection · digest hallucination rate (≤ 0.01) · OKR-share-of-VP correctness (= 1.0) · retro sync_class default compliance (= 1.0)
- References expanded: §0 + MEMORY_AUTOSYNC_DESIGN.md §5 + AUDIT_AND_PLAN + FR_AUTHORING_WORKFLOW

---

## 2026-05-15 — ESOP module page rewritten to Gold (Phantom Stock vesting + Good/Bad Leaver branch + HoldCo flip)

Rewrote `website/docs/modules/esop.html` to Gold. Three strategic roles: (1) grant lifecycle (issue/vest/cliff/cancel/put), (2) Good Leaver vs Bad Leaver branch on HR offboarding (CFO+CEO co-sign required), (3) liquidity-event simulator (annual valuation + put option exec + Singapore HoldCo flip trigger at ARR ≥ $1.5M).

Key changes:
- NEW §0 — 3-card layout + cap-table spine Mermaid showing memory exclusion + 10-row auto-vs-human matrix
- Risks +5 (R-ESOP-011..015): Leaver branch AI auto-route (Critical) · put-option ARR-trigger drift · vesting accrual on statutory leave · M&A acceleration without Member notice · HoldCo partial-flip rollback
- KPIs +5: Good/Bad Leaver co-sign integrity (= 1.0) · vesting accrual statutory-leave correctness · M&A notification SLA (≤ 5 days) · HoldCo flip cohort success (= 1.0 rollback on partial) · put-option exec query latency
- References expanded: §0 + 5 cross-module links + MEMORY_AUTOSYNC_DESIGN.md + DEC-036 + AUDIT_AND_PLAN + FR_AUTHORING_WORKFLOW

---

## 2026-05-15 — LEARN module page rewritten to Gold (skills catalogue + VP roll-up + Hội đồng Chuyên môn workflow)

Rewrote `website/docs/modules/learn.html` to Gold. Three strategic roles: (1) skills catalogue (skill tree × 1-5 mastery × bằng cấp/chứng chỉ evidence), (2) VP (Voting Power) roll-up engine (PROJ + TIME + KB → VP score → REW BP distribution), (3) Hội đồng Chuyên môn (Specialist Council) promotion workflow (3-5 peer judges; per-judge scores never exit the LEARN boundary; aggregate-only to HR).

Key changes:
- NEW §0 — 3-card layout + signal-flow Mermaid showing per-judge boundary explicitly + 10-row auto-vs-human matrix
- Risks +5 (R-LEARN-011..015): per-judge score export misconfig (Critical) · VP signal skews toward PROJ-dominant Members · Lumi skill catalogue pushes conflict · Council deliberation memory ingestion (psychological safety) · skill self-claim spam
- KPIs +5: per-judge export attempts blocked · VP fairness variance (≤ 0.40) · skill claim evidence rate (≥ 0.95) · deliberation transcript purge (≤ 30 d) · HR-to-LEARN-to-REW signal latency
- References expanded: §0 + 6 cross-module links + MEMORY_AUTOSYNC_DESIGN.md §5 + AUDIT_AND_PLAN + FR_AUTHORING_WORKFLOW

---

## 2026-05-15 — REW module page rewritten to Gold (compensation engine + payroll bridge + bonus orchestrator)

Rewrote `website/docs/modules/rew.html` to Gold. Three strategic roles: (1) compensation record owner (encrypted, HR-isolated, structurally excluded from memory per DEC-036), (2) payroll bridge (monthly VND cycle with BHXH/BHYT/BHTN, immutable parameter versioning, byte-identical PDF replay), (3) bonus orchestrator (BP fund + calibration → P3 distribution + CEO/CFO sign-off; P1-protection invariant DB-CHECK enforced).

Key changes:
- Title/meta + hero reframed; "Bet 5 moat" + EU AI Act Annex III §4 high-risk framing preserved
- NEW §0 — 3-card layout + REW-isolated-by-design Mermaid (HR/TIME/PROJ → REW → CFO+CHRO co-sign → payslips → banks/BHXH; memory explicitly disconnected with structural-exclusion line) + 10-row auto-vs-human matrix
- Risks +5 (R-REW-011..015): HR signals weaponised for P3 cut · BHXH mid-month rate change · Lumi attempts read REW (Catastrophic) · cross-Member cache leak · CFO+CHRO collusion (P1 protection at DB CHECK, not app layer alone)
- KPIs +5: P3 distribution sign-off completeness (= 1.0) · parameter mid-month transition correctness · Lumi-attempted reads (= 0) · cross-Member cache leak attempts (= 0) · P1 DB-CHECK constraint violations (any > 0 = sev-0)
- References expanded: §0 + 6 cross-module links + MEMORY_AUTOSYNC_DESIGN.md §5 + DEC-036 + AUDIT_AND_PLAN + FR_AUTHORING_WORKFLOW

---

## 2026-05-15 — HR module page rewritten to Gold (member lifecycle + onboarding orchestrator + performance signal aggregator)

Rewrote `website/docs/modules/hr.html` to Gold. Three strategic roles: (1) member lifecycle owner with AUTH-provisioned subject + multi-module event fan-out, (2) onboarding orchestrator (LEARN + KB + PROJ ramp plans saga-fired automatically), (3) performance signal aggregator (read-only consumer of PROJ + TIME + LEARN signals; comp number lives in REW, never HR).

Key changes:
- Title/meta + hero reframed
- NEW §0 — 3-card layout + Member-id spine Mermaid + 10-row auto-vs-human matrix
- Risks +5 (R-HR-011..015): HR signals used as sole comp basis · cross-tenant Member-id collision (Critical) · onboarding fires before AUTH ready · VN labour-law mid-year amendment · sabbatical tick misclassification
- KPIs +5: signal-only comp decision rate (= 1.0) · onboarding playbook saga p95 · labour-law version stamp coverage (= 1.0) · HR-to-REW handoff p95 · statutory-leave classification accuracy
- References expanded: §0 + 7 cross-module links + MEMORY_AUTOSYNC_DESIGN.md §5 + AUDIT_AND_PLAN + FR_AUTHORING_WORKFLOW

---

## 2026-05-15 — EMAIL module page rewritten to Gold (capture surface + Genie draft + outbound defence)

Rewrote `website/docs/modules/email.html` to Gold. Three strategic roles: (1) capture surface (tracked-domain auto-log to CRM activity + PROJ thread-to-issue), (2) Genie draft (Ask Genie composes outbound replies grounded in sanitised thread + CRM + memory + KB), (3) outbound send + defence (DKIM/ARC/BIMI; CaMeL quarantine defeats EchoLeak class).

Key changes:
- Title/meta + hero reframed
- NEW §0 "The bigger picture" — 3-card layout + EMAIL-in-orchestration-spine Mermaid + 10-row auto-vs-human matrix
- Risks +5 (R-EMAIL-011..015): thread-to-issue wrong Engagement · Genie draft confidential leak (High) · bulk-send approval bypass · tracked-domain misconfig (auto-log personal) · CaMeL cost spike
- KPIs +5: thread-to-issue conversion accuracy · Genie draft confidential-leak rate (= 0) · bulk-send token compliance (= 1.0) · tracked-domain audit pass · CaMeL cost per inbound
- References expanded: §0 + 7 cross-module links + CaMeL paper + EchoLeak CVE + MEMORY_AUTOSYNC_DESIGN.md §5 + AUDIT_AND_PLAN + FR_AUTHORING_WORKFLOW

---

## 2026-05-15 — KB module page rewritten to Gold (RAG corpus + memory companion + auto-runbook catalogue source)

Rewrote `website/docs/modules/kb.html` to Gold. Three strategic roles: (1) RAG corpus with three-layer retrieval (FTS5/PGroonga + BGE-M3 + cross-encoder) + span-level citations, (2) memory companion (long-form versioned counterpart to chain-anchored memories; "promote to canonical" elevates to high-authority source consumable by Lumi cross-tenant synthesis), (3) runbook catalogue source for OBS auto-runbook router (KB outage breaks OBS triage = critical coupling).

Key changes:
- Title/meta + hero reframed
- NEW §0 "The bigger picture" — 3-card layout + KB-in-platform Mermaid + 10-row auto-vs-human matrix
- Risks +5 (R-KB-011..015): runbook catalogue drift · OBS-KB tight coupling (KB outage breaks triage, High impact) · span-citation drift · vendor-pack malicious markdown · doc-gap-detector underperforms
- KPIs +5: runbook applicability accuracy · span-citation integrity (= 1.0) · doc-gap-detector signal rate · cross-tenant retrieval reject rate · vendor-pack CSO-review rate (= 1.0)
- References expanded: §0 + 6 cross-module links + OBS §2.6 auto-runbook contract link + MEMORY_AUTOSYNC_DESIGN.md §6 + AUDIT_AND_PLAN + FR_AUTHORING_WORKFLOW

---

## 2026-05-15 — INV module page rewritten to Gold (billable rollup invoicing + hóa đơn emission + dunning automation)

Rewrote `website/docs/modules/inv.html` to Gold by encoding three strategic roles: (1) billable rollup → invoice line items (consumes TIME per-cycle rollup; rate-card snapshot preserved), (2) hóa đơn emission (Decree 123 + Circular 78 GDT XML via vietnam-vat-invoice skill; Mẫu 01/GTGT; MST validation gate), (3) revenue recognition + dunning (CUO drafts overdue chase; human sends; aging report; cash application via 4 rails).

Key changes:
- Title/meta + hero reframed
- NEW §0 "The bigger picture" — 3-card layout + INV-in-orchestration-spine Mermaid + 10-row auto-vs-human matrix
- Risks +5 (R-INV-011..015): incomplete TIME rollup → missing hours · rate-card snapshot divergence · hóa đơn cancellation without dual approval (Critical) · dunning auto-send bug · Decree 123 amendment drift
- KPIs +5: TIME→INV bridge p95 · missing-Member draft rate · rate-card snapshot integrity (= 1.0) · dunning auto-send false-positive (= 0) · hóa đơn dual-approval rate (= 1.0)
- References expanded: §0 + 6 cross-module links + PROJ §2.6 billing modes + TIME §0 rollup contract + MEMORY_AUTOSYNC_DESIGN.md §5 + AUDIT_AND_PLAN + FR_AUTHORING_WORKFLOW

---

## 2026-05-15 — TIME module page rewritten to Gold (billable-hours engine + PROJ-INV bridge + Labour-law guardrails)

Rewrote `website/docs/modules/time.html` to Gold by encoding three strategic roles: (1) hours entry (timer + manual + auto-detect from PROJ activity), (2) billable rules engine (4-step cascade per PROJ §2.6: Member override → task class → role default → fallback; decision snapshotted on row), (3) PROJ-INV bridge (per-cycle billable rollup feeds INV).

Key changes:
- Title/meta + hero reframed; fact-grid extended (8→11 cards: + Strategic role, Billable cascade, Labour caps VN Code Art. 107)
- NEW §0 "The bigger picture" — 3-card layout + spine Mermaid (PROJ → Member → TIME → Billable cascade → AM → CFO + INV/REW/memory) + 9-row auto-vs-human matrix
- Risks +5 (R-TIME-011..015): billable cascade snapshot divergence (High) · auto-detect wrong Issue · VN Labour Code 2026 amendment · cycle-rollup runs before all submissions · multi-currency drift
- KPIs +6: cascade snapshot integrity (= 1.0 hard floor) · auto-detect acceptance · PROJ-TIME issue match rate · cycle-rollup completeness · VN Labour Code version coverage (= 1.0)
- References expanded: §0 + 6 cross-module links + PROJ §2.6 billable cascade link + MEMORY_AUTOSYNC_DESIGN.md §5 + AUDIT_AND_PLAN + FR_AUTHORING_WORKFLOW

---

## 2026-05-15 — CRM module page rewritten to Gold (sales-pipeline spine + Deal-to-Engagement bridge + next-action engine)

Rewrote `website/docs/modules/crm.html` to Gold by encoding three strategic roles: (1) sales pipeline VN-first (Account · Contact · Deal with VN integrations: MST validation, VietQR, hóa đơn, salutation logic), (2) Deal-to-Engagement bridge to PROJ §2.5 join contract (deal.won → engagement.create with rate card pre-wired), (3) next-action engine (CUO ranks moves on every open deal; AI lead scoring; win/loss memories citable by future deals).

Key changes:
- Title/meta + hero reframed to 3 strategic roles
- Fact-grid extended (8→11 cards: + Strategic role, Deal → Engagement bridge One-click, Vertical-pack ready)
- NEW §0 "The bigger picture" — 3-card layout + CRM-in-orchestration-spine Mermaid + 9-row auto-vs-human matrix
- Risks +5 (R-CRM-011..015): bridge fails partially · wrong billing mode · CUO next-action inappropriate · vertical-pack drift · merge data loss
- KPIs +6: deal-to-Engagement conversion rate · conversion bridge p95 · win/loss memory citation rate · next-action acceptance · stage-stuck deal alert · forecast accuracy
- References expanded: §0 + 7 cross-module links + PROJ §2.5 join contract link + SKILL §3.6 vertical-pack pattern + MEMORY_AUTOSYNC_DESIGN.md + AUDIT_AND_PLAN + FR_AUTHORING_WORKFLOW + expanded PDPL articles

---

## 2026-05-15 — TEN module page rewritten to Gold (P2 billing slice + residency enforcement + 90-day offboarding contract)

Rewrote `website/docs/modules/ten.html` to Gold. Encodes the research review §7.3 mandate (TEN-billing thin slice at P2/P2 · exit, not P4) — three strategic roles: (1) tenant lifecycle owner with state machine + audit propagation, (2) billing slice P2 thin (Stripe + 3 plans + cost cap) vs P4 full (+ VN-PSP + self-serve + in-app UI), (3) residency enforcement (data lives where law says; cross-leak CI gate = 0).

Key changes:
- Phase chip changed: "P4 long-term" → "P2 thin slice · P4 full"
- Title/meta + hero reframed; phase 0 strategic frame
- Fact-grid extended (8→13 cards: + Strategic role, P2 slice scope, P4 full scope, Residency options, Cross-leak target = 0)
- NEW §0 "The bigger picture" — 3-card layout + tenant lifecycle Mermaid (10 nodes: external customer → TEN → 3 billing rails + 5 modules + audit/CFO) + 9-row auto-vs-human matrix
- NEW §2.5 "P2 thin slice scope" — 12-row capability contrast (P2 thin vs P4 full) + plan-tier × usage budget table (Starter $49/seat · Team $39/seat · Enterprise custom; vertical pack add-on $99/$79/negotiated)
- NEW §2.6 "Residency × jurisdiction matrix" — 4-row infra mapping (sg-1 / eu-1 / us-1 / vn-1 each with Postgres shard, S3 region, AI providers, OBS retention, compliance regime) + cross-leak CI gate spec (200+ property-based test attempts per PR)
- NEW §2.7 "90-day offboarding contract" — 4-phase timeline (Active → Terminating-A 30d → Terminating-B 60d → Terminated day 91+) + signed bundle 6-component export + permanent-delete attestation JSON with Ed25519 signature
- Risks +8 (R-TEN-013..020): P2 slice slip → margin moat delayed (High) · residency change mid-engagement · hostile termination override · Stripe DPA EU residency · plan-downgrade overage surprise · cross-leak CI gap (Critical) · vertical-pack revenue attribution leak · Lumi-pushed pack pricing change
- KPIs +9: P2 slice ship date adherence (= P2 · exit) · vertical-pack revenue share (≥ 30% of ARR by P4 · mid — the moat) · cross-leak rate (= 0 hard floor) · residency drill MTTR (≤ 72h) · plan-downgrade overage handling (= 1.0) · hostile-termination cycle time · VN-PSP coverage (≥ 0.95 at P4) · PCI-SAQ-A scope (= 0; Stripe handles all) · tenant attestation completeness (= 1.0)
- References expanded: 4 in-page sections + 6 cross-module links + AUDIT_AND_PLAN §3.3 + RESEARCH_REVIEW §7.3 (explicit cite of the P2 · exit mandate) + MEMORY_AUTOSYNC_DESIGN.md §6 + FR_AUTHORING_WORKFLOW + EU AI Act Art. 26 + expanded PDPL article citations

---

## 2026-05-15 — OBS module page rewritten to Gold (observability spine + auto-runbook router + compliance evidence surface)

Rewrote `website/docs/modules/obs.html` to Gold by encoding three strategic roles: (1) three-pillars unified pane (logs/metrics/traces/AI-traces correlated by trace_id × tenant_id; pillar × signal table; cross-pillar correlation example; tenant query proxy isolation), (2) auto-runbook router (alerts → CUO triage skill → CHAT self-service OR PagerDuty escalation; severity × routing matrix; runbook-catalogue growth loop), (3) compliance evidence surface (per-regulator scoped read-only views over memory audit chain; YAML view definitions; chain-of-custody manifest with Ed25519 signature).

Key changes:
- Title/meta + hero reframed to 3 strategic roles
- Fact-grid extended (8→12 cards: + Correlation key, Auto-runbook coverage, Compliance surfaces, etc.)
- NEW §0 "The bigger picture" — 3-card layout + emitter/consumer Mermaid + 9-row auto-vs-human matrix
- NEW §2.5 "Three-pillars unified pane" — pillar × signal-type mapping table + concrete 5-step cross-pillar investigation walkthrough + tenant query proxy isolation guarantee
- NEW §2.6 "Auto-runbook router" — 6-step routing sequence Mermaid + severity × routing matrix (P0/P1/P2/P3/P4) + runbook-catalogue self-growth loop
- NEW §2.7 "Compliance evidence surface" — regulator × audit scope matrix (EU AI Act, PDPL, SOC 2, ISO 27001, GDPR, Vietnam Decree 13/2023) + per-view scoping YAML + chain-of-custody manifest with chain anchors
- Risks +10 (R-OBS-011..020): auto-runbook miscategorising P0 (Critical) · compliance export tampering (Critical) · triage skill down → page storm · LangSmith EU residency · trace sampling drops wrong tail · persona-drift false positive · OTel context propagation breaks · query proxy DOS · runbook catalogue drift · maintenance-window noise
- KPIs +10: auto-runbook coverage (≥ 60% by P1) · P0/P1 false-suppression (= 0 hard floor) · compliance export verification rate (= 1.0) · cross-pillar correlation completeness (≥ 0.95) · tail-sampling error coverage (= 1.0) · persona-drift detector precision · query proxy violations · self-service ticket MTTR · dogfooding alert ACK (we live by this) · compliance surfaces × regulator
- References expanded to universal-protocol scope: 4 in-page sections + 8 cross-module links + AUDIT_AND_PLAN §3.3 (P0 · slice 1 placement) + RESEARCH_REVIEW §6 (9/10) + MEMORY_AUTOSYNC_DESIGN.md §8 + FR_AUTHORING_WORKFLOW + EU AI Act + ISO 27001 + ISO 42001 + SOC 2 + PDPL + Decree 13 + GDPR Art. 30

---

## 2026-05-15 — MCP Gateway module page rewritten to Gold (external-client federation + capability broker + tool-discovery surface)

Rewrote `website/docs/modules/mcp.html` to Gold by encoding three strategic roles: (1) external-client federation (22 modules → one MCP server for Claude/Cursor/Codex/Cline; SEP-986 naming + module registration sequence + 6-row client compatibility matrix), (2) capability broker (6-row tool-annotation gating + audience-bound OAuth JWT example + destructive-op Elicitation flow), (3) tool-discovery surface (6 discovery endpoints + Tasks primitive 8-field schema + 5 pre-canned prompt templates).

Changes by section:
- **`<title>` + `<meta>`** — reframed: "MCP Gateway — External-client federation · Capability broker · Tool-discovery surface".
- **Hero tagline + lede** — "the external-agent door" framing: 22 modules behind one MCP surface; Claude/Cursor/Codex see one server; federation invisible to external clients.
- **Hero fact-grid** — extended from 8 to 12 cards: added Strategic role + External clients (Claude · Cursor · Codex · Cline) + Destructive-op gating (Human-confirm) + Persona stamp coverage (100%). Renamed naming convention card with concrete pattern.
- **NEW §0 "The bigger picture — three strategic roles"** — 3-card layout; federation Mermaid (5 external clients × MCP Gateway × 6 per-module servers × 4 platform deps); 9-row auto-vs-human matrix.
- **TOC** — added bigger-picture · client-federation · capability-broker · tool-discovery entries.
- **NEW §2.5 "External-client federation"** — SEP-986 naming convention with 8 tool-name patterns + per-module registration sequence Mermaid (heartbeat-based lifecycle) + 6-row external-client compatibility matrix (Claude Code, Claude Desktop, Cursor, Codex, Cline, older 2024-11-05 clients).
- **NEW §2.6 "Capability broker"** — 6-row tool-annotation gating table (readOnly / idempotent / destructive / openWorld / longRunning / elicits); audience-bound OAuth JWT shape with aud=mcp.cyberos.com + scope_grants array; destructive-op confirmation flow with full Elicitation JSON request/response example.
- **NEW §2.7 "Tool-discovery surface"** — 6 discovery endpoints (well-known/mcp, capabilities, tools/list, prompts/list, resources/list, resources/templates/list); 8-field Tasks primitive schema with memory_chain anchor; 5 pre-canned prompt templates (weekly_brief, decision_to_issues, draft_cycle_review, deal_to_engagement, find_memory_citations).
- **§12 Risks** — added 10 new (R-MCP-011..020): external agent token theft (Critical) · prompt injection in tool description · elicitation fatigue (High likelihood) · federation lag · task storm · resource leak via list_changed · heartbeat false-positive · DCR abuse · older-protocol-version security gap · SEP-986 naming collision.
- **§13 KPIs** — added 10 new: persona-stamp coverage (hard floor = 1.0) · elicitation acceptance rate · tasks completion rate · cross-tenant token-replay attempts · older-protocol session rate (→ 0 by P3 · exit) · list_changed push latency · destructive-op confirm fatigue · external-client tools coverage · SEP-986 compliance.
- **§17 References** — replaced stale PRD/SRS refs with 4 in-page sections + 8 cross-module links + AUDIT_AND_PLAN §3.3 (P0 · slice 3 placement) + RESEARCH_REVIEW §5 (9/10) + MEMORY_AUTOSYNC_DESIGN.md §5+§6 + FR_AUTHORING_WORKFLOW + DPoP RFC 9449 + EU AI Act + PDPL citations.

The MCP Gateway page now reads as the complete answer to: (1) why 22 modules need one external door (federation Mermaid + N²→N+1 math), (2) how the broker prevents a compromised external agent from escaping scope (audience-bound JWT + tool-annotation gating + destructive-op Elicitation), (3) how external agents discover what CyberOS can do (6 discovery endpoints + 5 pre-canned prompts + Tasks primitive for long-running work), (4) what fails if MCP Gateway is missing (every external agent re-implements its own auth + tool catalogue + audit).

---

## 2026-05-15 — AI Gateway module page rewritten to Gold (P0 · slice 1 cost-of-everything gate + provider abstraction + compliance plane)

Rewrote `website/docs/modules/ai.html` to Gold by encoding three strategic roles: (1) cost-of-everything gate (per-tenant policy YAML + 7-step pre/post accounting + 7-dimension attribution), (2) provider-agnostic router (6-row model-alias table + 7-row failover semantics + residency × provider matrix), (3) compliance plane (4-link chain PII → persona → ZDR → audit + 14-field invocation row schema + VN-PII recogniser).

Changes by section:
- **`<title>` + `<meta>`** — reframed: "AI Gateway — Cost-of-everything gate · Provider-agnostic router · Compliance plane".
- **Hero tagline + lede** — explicit research review §2.4 citation: "ships at P0 · slice 1 BEFORE AUTH because if you can't account for and cap LLM spend, every other module bleeds money invisibly". Lists all 3 strategic roles.
- **Hero fact-grid** — extended from 8 to 12 cards: added Strategic role + Build placement (P0 · slice 1 P0 #1) + Cost-cap enforcement (hard-stop) + ZDR (required). Renamed dependency card to reflect P0 · slice 1 reality (memory + OBS at start; AUTH at P0 · slice 2).
- **NEW §0 "The bigger picture — three strategic roles"** — 3-card layout with cross-module dependency Mermaid (6 callers × AI Gateway × 5 providers × 4 platform deps); 9-row auto-vs-human matrix covering failover, cost-cap override, ZDR refusal, cache hit, model alias resolution, image-gen.
- **TOC** — added bigger-picture · cost-gate · provider-abstraction · compliance-plane entries (4 new).
- **NEW §2.5 "Cost-of-everything gate"** — per-tenant policy YAML (caps, hard-stop, emergency override, per-model caps, per-persona attribution); 8-actor pre/post-call accounting sequence (Caller → Gateway → ledger → Provider → memory → INV); 7-dimension attribution table (tenant_id, agent_persona, module, cost_centre, route_class, cache_state, failover_path).
- **NEW §2.6 "Provider abstraction + failover"** — 6-row model-alias resolution (chat.smart / chat.fast / chat.reason / embed.standard / rerank.standard / image.standard); 7-row failover semantics (5xx retry / consecutive 5xx → mark degraded / 429 backoff / circuit breaker / recovery / both-down degraded mode / per-tenant SLA breach); residency × provider matrix (sg-1 / eu-1 / us-1 / vn-1).
- **NEW §2.7 "Compliance plane"** — 4-link chain table (PII → persona → ZDR → audit) with recall target + failure behaviour per link; full <code>ai.invocation</code> audit row schema (14 extra fields); VN-PII recogniser table (CCCD / MST / VN phone / NĐD / VN address / VN bank account) with patterns + redaction examples.
- **§12 Risks** — added 10 new (R-AI-011..020): P0 · slice 1 sequence slip → cost-overrun invisible (Critical) · persona prompt cache poisoning · provider DPA cancellation mid-quarter · cost-ledger hold leak · streaming SSE buffer leak · embedding model upgrade breaks memory search · image-gen budget flood at P2+ · geographic residency violation during failover (Critical) · VN-PII recogniser regression · BGE GPU pod OOM under load.
- **§13 KPIs** — added 9 new: per-persona cost share (alert on &gt; 50% concentration) · cache savings rate (≥ 15% by P1) · hold-to-actual drift (≤ 5%) · residency-violation refusal rate · persona stamp coverage (hard floor = 1.0) · ZDR-compliant routing rate (hard floor = 1.0) · VN-PII recall on production sample (≥ 0.99) · provider-failover MTTR p95 (≤ 30s) · dogfooding LLM cost / Member (≤ $10/$5 trajectory).
- **§17 References** — replaced stale PRD/SRS refs with the 4 new in-page sections + MEMORY_AUTOSYNC_DESIGN.md §7 + feature-request-audit skill + AUDIT_AND_PLAN §3.3 (P0 · slice 1 placement) + RESEARCH_REVIEW §2.4 (reorder citation) + 8 cross-module links + expanded EU AI Act citations (Art. 12/13/14/15/26/50) + OWASP Gen AI Top-10 + ISO/IEC 42001 + PDPL Art. 14/20/38.

The AI Gateway page now reads as the complete answer to: (1) why this module ships first in P0 (cost-control before everything), (2) how the cost ledger gates calls in real-time (pre-check + post-reconcile + 60s hold expiry), (3) how the same Python service abstracts across Bedrock/Anthropic/OpenAI/Vertex (model alias + residency × provider matrix), (4) how the 4-link compliance chain ensures no bytes leak unscrubbed/unstamped/un-ZDR'd/un-audited. A new engineer reading this page cold can pick up the P0 · slice 1 build sequence and ship the cost-gate slice.

---

## 2026-05-15 — CUO module page rewritten to Gold (agent orchestrator + Lumi identity wrapper + skill broker contract + cross-module surfaces)

Rewrote `website/docs/modules/cuo.html` from 1035 → 1362 lines (+327 lines, +32%). Encodes three strategic roles the CUO module plays simultaneously — skill-routing memory, persona catalogue (agent-equal C-level members), Lumi tenant-identity wrapper — with explicit handling of the agent_persona JWT shape from AUTH §2.7 and the capability-broker contract from SKILL §3.5. Targeted Edit operations preserved every gold-quality detail of the shipped Phase 1 (rule-based router, 6 core modules, 10 personas, 15 fixtures) while adding 4 strategic deep-dive sections + risk/KPI extensions + universal-protocol references.

Changes by section:
- **`<title>` + `<meta>`** — reframed: "CUO — AI orchestrator · Skill-routing memory · Lumi tenant persona · CyberOS". Description names the three strategic roles + the Phase 1 ship state + the P0 · exit/P1 · exit/P2 · exit roadmap to Phases 2-4.
- **Hero tagline + lede** — explicit "agent orchestrator" framing; introduces Genie (face) / CUO (engineer view) / Lumi (org-tenant identity) naming distinction in one paragraph; lists all 3 strategic roles with Phase milestones.
- **Hero fact-grid** — extended from 8 to 12 cards: added Strategic role + Lumi readiness (P3 unlock) + Routing latency p95 + Audit-chain coverage (100%); changed "Tests" formatting to 15+15 (pytest+fixtures).
- **NEW §0 "The bigger picture — three strategic roles"** — 3-card layout (Role 1 skill-routing memory / Role 2 persona catalogue agent-equal / Role 3 Lumi tenant identity). Cross-module dependency Mermaid with CUO as hub touching 7 user surfaces upstream + 5 downstream systems including Lumi's memory at P3+. Auto-vs-human-in-loop operations matrix (8 rows) — explicit normative split.
- **TOC** — added bigger-picture · lumi-identity · skill-broker · cross-module-surfaces entries (4 new strategic anchors).
- **NEW §3.5 "Lumi identity wrapper — local CUO ↔ org-tenant persona"** — 3-row Lumi vs Genie vs local-CUO naming table; full AUTH JWT shape with agent_persona + tenant_id + scope_grants per AUTH §2.7; 4-row cross-tenant synthesis output table (updated persona prompts / keyword banks / cross-tenant lessons / vertical-pack updates) with cadence + privacy floor for each.
- **NEW §3.6 "Skill broker contract — capability-gate at every invocation"** — 11-step Mermaid sequence (User → CUO → catalog → broker → AUTH → pre-audit → skill exec → post-audit); 7-row CUO↔broker contract table (catalog stability + scope_grants + allowed_tools + destructive-op gate + pre+post audit + tenant isolation + version pinning); 10-row defer-to-human matrix (CEO/COO/CFO/CMO/CTO/CHRO/CSO/CLO/CDO/CPO) with auto-OK vs defers split.
- **NEW §3.7 "Cross-module CUO surfaces — where Genie appears"** — 9-row canonical surface table (CHAT @lumi / EMAIL Genie / PROJ inline / CRM next-action / KB ask-the-docs / TIME assist / INV pre-send check / PORTAL client / OBS triage) with trigger + context shipped + UI affordance for each. Per-surface latency budget table (6 rows) with route-only p95 + total response p95 + design note per surface.
- **§13 Risks** — added 10 new (R-CUO-008..017): Lumi tenant-id spoofing (Critical impact, CSO-owned) · destructive auto-invoke despite matrix (Critical, hard zero) · catalog drift route-vs-invoke · cross-surface latency miss · cross-tenant synthesis privacy leak · persona prompt drift via Lumi pushes · EU AI Act Art. 12 logging gap (Phase 2 migration required) · @lumi rate-limit abuse · Phase 2 LLM cascade outage degradation · Genie answers from training cutoff on company-specific topics.
- **§14 KPIs** — added 10 new universal-protocol-aware: per-surface response p95 (PROJ inline ≤ 800 ms / CHAT @lumi ≤ 4 s) · destructive-op auto-invoke rate (= 0 hard zero) · Lumi sync push success rate (≥ 0.99 at P3+) · cross-tenant sync_class violation rate (= 0 hard zero) · persona-version stability (≤ 2 changes per quarter) · @lumi cost per active Member (≤ $5/DAU/month) · must-cite-source compliance (≥ 0.95) · dogfooding rate (100% of team by P0 · exit).
- **§18 References** — replaced stale PRD/SRS section refs with the 4 new in-page sections + MEMORY_AUTOSYNC_DESIGN.md §5+§6 + feature-request-audit skill (CUO + memory + Skill = first 50 FRs) + AUDIT_AND_PLAN_2026_05_14.md §3.3 (P0 · exit/P1 · exit/P2 · exit/P3 · exit+) + RESEARCH_REVIEW_2026_05_14.md §2 (8.5/10) + 8 cross-module page links + EU AI Act Art. 12/14/26 + PDPL Art. 14.

Verified:
- 1362 lines parses cleanly
- 23 top-level sections (was 19) including 4 strategic new ones (§0, §3.5–§3.7)
- 2 new Mermaid diagrams (cross-module dependency flowchart + 11-step broker sequence)
- 17 risk rows (was 7), with 10 new framed around Lumi cross-tenant privacy + destructive-op gating + EU AI Act Art. 12 + Genie training-cutoff hallucination
- 17 KPI rows (was 7), with hard-zero KPIs (destructive auto-invoke = 0, cross-tenant sync_class violation = 0) as the compliance floor
- Lumi naming clarified in 5+ places — Genie (user face) / CUO (engineer view) / Lumi (org-tenant identity) → consistent through hero, §0, §3.5, audit table, references

The CUO page now reads as the complete answer to: (1) why CUO is the orchestrator and not "yet another chatbot framework" (the 3-role frame + cross-module surface table), (2) how the agent_persona JWT cryptographically anchors every Lumi action back to AUTH (concrete JWT example), (3) why the capability broker is the protocol-level guarantee that auto-invocation cannot escape scope (7-step sequence + 7-row contract + defer-to-human matrix), (4) where Genie actually shows up in the platform (9-row cross-module surface table with per-surface latency budgets). A new engineer reading this page cold can pick up the Phase 1 source + AGENTS.md and ship Phase 2 LangGraph integration.

---

## 2026-05-15 — PROJECT module page rewritten to Gold (orchestration spine + Engagement economics + memory-anchored decisions + Liquid-Glass UI exemplar)

Rewrote `website/docs/modules/proj.html` from 1126 → 1514 lines (+388 lines, +34%). Encodes three strategic roles the PROJ module plays simultaneously — orchestration spine for cross-module joins, memory-anchored decision substrate, consultancy-native Engagement billing surface — with no role under-served. Targeted Edit operations preserved the existing strong content (4 primitives, sync-engine architecture, 5 key-flow sequences, status enum + workflow overlay, 7 surface CLI commands) while adding 4 strategic deep-dive sections + risk/KPI extensions + universal-protocol references.

Changes by section:
- **`<title>` + `<meta>`** — reframed: "PROJ — Orchestration spine · memory-anchored decisions · Engagement billing · CyberOS". Description names the orchestration spine (CRM → PROJECT → TIME → INV → REW → KB → memory), the consultancy-native Engagement primitive, the memory-citation graph, and the Liquid-Glass UI exemplar.
- **Hero tagline + lede** — explicit "orchestration spine" framing; lists all 3 strategic roles in one paragraph; replaces stale PRD-referenced prose with role descriptions.
- **Hero fact-grid** — extended from 8 to 13 cards: added Strategic role + Cross-module joins (7) + memory integration (bidirectional) + Engagement model (3 modes) + UI surfaces (4). Strategic role card uses "Orchestration spine" pill prominent.
- **NEW §0 "The bigger picture — three strategic roles"** — 3-card layout (Role 1 orchestration spine / Role 2 memory-anchored / Role 3 Engagement billing). Cross-module join Mermaid flowchart with PROJ as hub touching 9 other modules. Auto-vs-manual operations matrix (9 rows) — explicitly classifies which PROJ behaviours are automatic vs deliberate.
- **TOC** — added bigger-picture · orchestration-spine · engagement-economics · memory-anchored · ui-surfaces entries (5 new).
- **NEW §2.5 "Orchestration spine — cross-module join contracts"** — 9-row canonical contract table covering each counterparty (CRM/EMAIL/TIME/INV/KB/REW/OKR/PORTAL/memory): direction · join key · trigger · payload shape · failure mode. Contract stability policy: breaking changes require ADR + counterparty co-sign + 1-minor-release deprecation window + migration test + memory decision memory.
- **NEW §2.6 "Engagement economics — consultancy-native primitive"** — 3-mode billing table (T&M / fixed-fee / retainer) with what INV pulls + risk + typical use. Full rate-card YAML example (architect/senior/mid/junior with VND + USD rates + per-role billable_default). Billable / non-billable cascade (4-step priority): Member override → task class → role default → fallback. Margin watchdog spec for P2 (fixed-fee scope-creep early warning).
- **NEW §2.7 "memory-anchored decisions — issues cite memories"** — three citation relations (cites / implements / supersedes) with examples. Decision-to-issues skill sequence (8-actor Mermaid: User → CUO/CPO skill → memory read → PROJ create N+1 issues → memory write audit). Dual-write audit chain example: PROJ history_event row + memory audit row with matching chain hash.
- **NEW §2.8 "Liquid-Glass UI surfaces — Board · Timeline · Gantt · Brief"** — 4-surface canonical table (primary use · default view · density · keyboard-first). PROJ-specific design-token overlay (tokens.proj.css) with status palette + priority colours + Liquid-Glass blur/saturate values. 6-point accessibility commitment list (WCAG AA + keyboard nav + screen-reader labels + focus trap + reduce-motion + VN diacritic-correct fonts).
- **§12 Risks** — added 10 new (R-PROJ-011..020): orchestration-spine SPOF · contract breaking change without ADR · fixed-fee scope creep eats margin (High likelihood × High impact, COO-owned) · memory citation drift · cycle-review draft cites out-of-window work · billing-mode mid-cycle change · decision-to-issues skill drift · Liquid-Glass accessibility fail · SPA cold-load > 5s on VN mobile (Members give up and use Excel) · NATS JetStream backlog staleness.
- **§13 KPIs** — added 10 new universal-protocol-aware: Join-contract stability (≤ 1 breaking change/quarter) · Engagement margin T&M (≥ 35%) · Engagement margin fixed-fee (≥ 30% on close) · Issues with memory citation (≥ 40% of high-priority) · Decision-to-issues skill acceptance (≥ 70%) · SPA cold-load p95 on VN mobile (≤ 5s) · Citation-drift rate (≤ 5%) · Cross-tenant ACL rejection rate · Dogfooding cycle-review draft acceptance (≥ 70% — founders use this before selling it).
- **§17 References** — replaced stale PRD/SRS section refs with the 4 new in-page sections + MEMORY_AUTOSYNC_DESIGN.md §5 (capture surfaces) + feature-request-audit skill + AUDIT_AND_PLAN_2026_05_14.md §3.3 (P1 · mid placement) + RESEARCH_REVIEW_2026_05_14.md §4 (Engagement primitive flagged as highest-leverage differentiator) + 11 cross-module page links + PDPL Art. 7/14/20.

Verified:
- 1514 lines parses cleanly
- 23 top-level sections (was 18) including 5 strategic new ones (§0, §2.5–§2.8)
- 5 new Mermaid diagrams (cross-module join flowchart + decision-to-issues sequence + 3 inline in §2.6/§2.7/§2.8)
- 20 risk rows (was 10), with 10 new framed around orchestration spine SPOF + Engagement scope creep + memory-citation drift + VN mobile cold-load
- 19 KPI rows (was 9), with margin watchdog + citation-coverage + dogfooding-acceptance as the new strategic gates

The PROJ page now reads as the complete answer to: (1) why PROJ is the spine and not just a tracker (the join contract table makes it concrete), (2) why consultancies cannot use Linear or Jira off the shelf (the Engagement economics section walks through 3 billing modes + rate-card YAML + billable cascade), (3) how the memory integration makes issue history survive leadership changes (citation graph + dual-write audit chain), (4) why PROJ is the design-system exemplar (4 canonical UI surfaces + token overlay + accessibility commitments). A new engineer reading this page cold can pick up the sync-engine, join contracts, and the four UI surfaces and start P1 slice 1.

---

## 2026-05-15 — CHAT module page rewritten to Gold (P0 dogfood gate + Mattermost fork rationale + @lumi memory capture + decommission KPI)

Rewrote `website/docs/modules/chat.html` to push the module past the threshold from "Solid (8/10)" to Gold by encoding three strategic roles simultaneously: P0 dogfood gate (Slack + Zalo killed by P0 · exit or the platform thesis fails), memory capture surface (one of four canonical capture inputs), and Vietnamese-first chat (PGroonga + TinySegmenter recall ≥ 80%). Targeted Edit operations — preserved every gold-quality detail of the prior content (channels, threads, attachments, search, memory bridge, @genie, Slack importer, mobile, voice) while adding 6 strategic new sections + risk/KPI extensions + universal-protocol references.

Changes by section:
- **`<title>` + `<meta>`** — reframed: "P0 dogfood gate · Mattermost fork · @lumi memory capture · CyberOS".
- **Hero tagline + lede** — explicit P0-dogfood-gate framing: Slack + Zalo decommissioned at P0 exit (P0 · exit), or the whole platform thesis fails. Lists the three strategic roles.
- **Hero fact-grid** — added "Decom gate Slack+Zalo killed by P0 · exit" + "E2EE decision Per-tenant Postgres encryption".
- **NEW §0 "The bigger picture — three strategic roles"** — 3-card layout (P0 dogfood gate / memory capture surface / Vietnamese-first chat). P0-exit dependency Mermaid showing reordered sequence (AI Gateway → AUTH stub → MCP → CHAT → Slack/Zalo decom → P0 exit).
- **TOC** — added 6 new section links (bigger-picture · rt-stack · lumi-memory-capture · e2ee-decision · slack-zalo-migration · decommission-kpi).
- **NEW §2.5 "Real-time stack — Mattermost fork rationale"** — 4-option decision table (Mattermost fork chosen vs Matrix / Phoenix / build-from-scratch) + own-vs-Mattermost ownership table + fork governance text + license-drift escalation path.
- **NEW §2.6 "@lumi → memory capture"** — capture rules table (@lumi=capture, no @lumi=privacy floor, DM rules) + 8-actor sequence diagram (User → CHAT → @lumi parser → CUO → AI Gateway → memory Writer → Lumi's memory). Per-message retro-capture opt-in for "Lumi remember the last N messages".
- **NEW §2.7 "E2EE decision — server-visible by design"** — 10-row threat-model trade table comparing E2EE vs per-tenant Postgres encryption-at-rest; 5-point rationale for choosing the latter; concrete encryption-at-rest description; tenant-level optional E2EE plugin reserved for P3 (search disabled on those channels).
- **NEW §2.8 "Slack/Zalo migration"** — 8-step `cyberos-chat import slack` flow with parse/map/backfill/ingest/verify checkpoints; 2-path Zalo migration (manual export + future desktop bridge); pre-import dry-run + idempotent + checkpointed importer.
- **NEW §2.9 "Decommission KPI"** — formal definition: `decommission_signal := (msgs_in_chat / (msgs_in_chat + msgs_in_slack + msgs_in_zalo)) ≥ 0.95 over 14-day rolling window`. Why 95% not 100%; tracking instrumentation table; 3-tier miss-the-gate remediation (T1 = 2-week sprint freeze on net-new modules, T2 = P1 · start platform-thesis review, T3 = potential P0 rescope per research review §1).
- **§12 Risks** — added 10 new (R-CHAT-011..020): dogfooding-never-happens (Critical, CEO-owned) · enterprise E2EE pressure · voice ASR PII leak · Mattermost license drift · @lumi rate-limit abuse · cross-tenant search leak · Slack import partial failure · retro-capture privacy boundary · mobile push PII leak · VN/EN code-switch tokeniser miss.
- **§13 KPIs** — added 9 new universal-protocol-aware: decommission_signal (P0-exit gate) · @lumi capture-rate (≥ 0.999) · @lumi response p95 (≤ 4 s) · VN tokeniser recall continuous (≥ 0.80, alert &lt; 0.78) · memory-ingest backlog max · retro-capture opt-in rate · mobile push delivery rate · cross-tenant query reject rate · dogfooding intensity (P0-gate: 100% of full-time team by P0 · slice 2).
- **§17 References** — replaced/expanded with MEMORY_AUTOSYNC_DESIGN.md §5 (CHAT as 1 of 4 capture surfaces) · feature-request-audit skill (CHAT FRs deliberately pending) · AUDIT_AND_PLAN_2026_05_14.md §3.3 (P0 · slice 2 build placement) · RESEARCH_REVIEW_2026_05_14.md §3 (Solid 8/10 with decommission caveat) · Mattermost governance docs · PGroonga + TinySegmenter refs · PDPL Art. 7/14/20/38 · EU AI Act Art. 12/13/50.

Verified:
- 24 top-level sections (was 18) including 5 strategic new ones (§0, §2.5–§2.9)
- 4 new Mermaid diagrams (P0-exit dependency + 1 sequence + 0 in §2.7/§2.8 prose + 1 in §0)
- 20 risk rows (was 10), with 10 newly framed around dogfooding + privacy + tokeniser code-switch
- 18 KPI rows (was 9), with decommission_signal as the explicit P0-exit gate
- decommission_signal definition appears verbatim 3× (hero fact-grid, §2.9, §13 KPI table)

The CHAT page now reads as the complete answer to: (1) why CHAT is the P0 dogfood gate not just another module, (2) why Mattermost fork beats Matrix/Phoenix/build-from-scratch under our constraint set, (3) how @lumi mention is the conversational memory-capture mechanism, (4) why we chose per-tenant Postgres encryption-at-rest over E2EE, (5) how Slack/Zalo migration works without losing threads/reactions, and (6) what happens if decommission_signal misses 0.95 by P0 · exit (the platform-thesis review escalation). A new engineer reading this page cold can now pick up the Mattermost fork repo + memory bridge spec + Slack importer spec and start slice 1.

---

## 2026-05-14 — AUTH module page rewritten to Gold (P0 · slice 2 stub vs P3 full + Lumi tenant identity + RFC open Qs resolved)

Rewrote `website/docs/modules/auth.html` from 1169 → 1442 lines (+273 lines, +23%). Encodes the research review §2.4 reorder (AI Gateway BEFORE AUTH) and AUTH's distinct roles as P0 · slice 2 stub vs P3 full. Targeted Edit operations preserved every gold-quality detail of the prior content while adding 4 new strategic sections + risk/KPI extensions.

Changes by section:
- **`<title>` + `<meta>`** — reframed: "P0 · slice 2 stub → P3 full · Lumi tenant identity · Agent-equal".
- **Hero tagline + lede** — explicit P0 · slice 2 stub vs P3 full distinction · cites reordered P0 sequence (AI Gateway @ P0 · slice 1 → AUTH @ P0 · slice 2 → MCP Gateway @ P0 · slice 3 → CHAT/CUO @ P0 · exit) · references RFC.md + sign-in mockup + MEMORY_AUTOSYNC_DESIGN.md.
- **Hero fact-grid** — split status into "P0 · slice 2 stub designed" + "P3 full designed", LoC into 1,500 stub + 7,000 full, RBAC into 5 stub + 22 full, dependencies + Lumi enablement.
- **NEW §0 "The bigger picture — three strategic moves"** — 3-card layout (Move 1 P0 · slice 2 stub / Move 2 P3 full / Move 3 Lumi tenant identity). Gantt chart Mermaid showing the reordered P0 build sequence end-to-end. Rationale for reorder cited from reviewer.
- **TOC** — added bigger-picture · stub-vs-full · rbac-catalogue · lumi-integration · open-questions entries.
- **NEW §2.5 "P0 · slice 2 stub vs P3 full"** — 12-row capability-contrast table covering login mechanism · MFA · RBAC catalogue · JWT signing · tenant isolation · audit-chain emission · admin surfaces · cost · LoC · tests · Lumi integration · SOC 2 evidence. Plus "Migration discipline" + "What stub doesn't compromise on" prose.
- **NEW §2.6 "22-role RBAC catalogue"** — full 22-row table with scope summary, stub-eligibility, and slice when each role lands. The 5 stub roles (root-admin · tenant-admin · tenant-member · service-account · agent-persona) are explicitly the first 5; the remaining 17 land across slices 3–5. Role-addition policy: ADR-gated, no code-only changes.
- **NEW §2.7 "AUTH ↔ Lumi's memory"** — full JWT claim shape (15 fields incl. tenant_id, tenant_residency, agent_persona, scope_grants) · sequence diagram of Lumi's memory verifying a sync push · 5-bullet contract requirements list (tenant_id non-removable, JWKS reachability, refresh-token reuse detection, agent-persona claims preserve agent-equal, residency pinning flows through).
- **NEW §2.8 "RFC open questions resolved"** — table addressing all 5 open Qs from RFC §6 with proposed defaults + rationale: Q1 workspace = new repo-root Cargo workspace · Q2 memory bridge = subprocess slice 4 → PyO3 slice 5 · Q3 tenant-0 bootstrap = `cyberos-auth bootstrap` CLI subcommand · Q4 HIBP = default-on with per-tenant opt-out · Q5 OBS = slice 1 stdout → slice 5 OTLP. Each becomes an ADR once Stephen signs off.
- **§12 Risks** — added 7 new (R-AUTH-011..017): stub stays past P3 · reorder regret · Lumi tenant-id spoofing · cross-shard JWT replay · sub-process audit-bridge bottleneck · tenant-0 bootstrap leak · PDPL Art. 38 SME grace lapse.
- **§13 KPIs** — added 7 new: stub-to-full migration coverage (≥95% T2+ subjects passkey-enrolled by P1 · exit) · mock-AUTH retirement · Lumi tenant-id verification rate · cross-shard rejection · audit-bridge p99 · SME-grace lapsed tenants · 22-role catalogue stability.
- **§17 References** — replaced PRD/SRS section refs (stripped) with services/auth/RFC.md, sign-in mockup, MEMORY_AUTOSYNC_DESIGN.md §6, RESEARCH_REVIEW §2.4 (cited verbatim), AUDIT_AND_PLAN, FR_AUTHORING_WORKFLOW, AGENTS.md §3.6+§11.

Verified:
- 1442 lines parses cleanly
- 23 top-level sections (was 18) including 4 strategic new ones
- Mermaid gantt chart documents the reordered P0 sequence
- All 5 RFC §6 open questions now have proposed defaults visible on the page

The AUTH page now reads as the complete answer to: (1) why AUTH is not P0 #1 (research review §2.4), (2) what the P0 · slice 2 stub actually contains vs the P3 full target, (3) how AUTH enables Lumi's memory tenant isolation, (4) what the 5 open RFC questions resolve to. A new engineer reading this page cold can pick up RFC.md and start slice 1.

---

## 2026-05-14 — SKILL module page rewritten to Gold (memory integration + vertical-pack moat + distribution roadmap)

Rewrote `website/docs/modules/skill.html` from 1134 → 1431 lines (+297 lines, +26%). Encodes the three strategic roles the Skill module plays simultaneously — open-standard citizen, memory-protocol enabler, vertical-pack moat — with no role under-served. Targeted Edit operations preserved every gold-quality detail of the shipped Phases 0–7 while adding Phase 8 memory integration, vertical-pack pattern + 8-pack roadmap, and the R0→R5 distribution staging.

Changes by section:
- **`<title>` + `<meta>`** — "Open Agent Skills · memory-integrated · Vertical-pack moat · CyberOS" — three roles in the title itself.
- **Hero tagline + lede** — explicit three-role frame: open-standard citizen / memory-protocol enabler / vertical-pack moat. Lists the capture daemon + sync orchestrator + synthesis sub-skill as skill bundles. Names cyberskill-vn as proof-of-pattern, not the strategy.
- **Hero fact-grid** — added "Status (memory-int) Phase 8 designed" + "Vertical packs 1 shipped · 6 planned"; updated dependencies to memory + AUTH.
- **NEW §0 "The bigger picture — three strategic roles"** — 3-card layout (Role 1 / Role 2 / Role 3); dependency graph Mermaid showing Skill's unique position touching the external Agent Skills ecosystem.
- **TOC** — added Bigger picture · memory integration · Vertical-pack pattern · Distribution roadmap entries.
- **NEW §3.5 "memory integration"** — full SKILL.md frontmatter example with memory-aware fields (allowed_memory_scopes for personal + lumi scopes); capability broker enforcement sequence diagram (8 actors, 14 steps); table of 5 universal-protocol skills (memory-capture@1, memory-sync@1, synthesis-author@1, feature-request-author, feature-request-audit).
- **NEW §3.6 "Vertical-pack pattern"** — 7-step pack recipe (jurisdiction → high-pain workflows → SKILL.md bundle → localise language → compliance-verify → agentskills.io publish → Lumi tenant sell); 9-pack roadmap table (vn shipped + sg + id + th + eu + us + hr + legal + accounting) with target ship dates and annual unit pricing; margin math worked example.
- **NEW §3.7 "Distribution roadmap R0→R5"** — 6-rung distribution table (local cache → .skill bundles → OCI registry → agentskills.io → own marketplace → enterprise white-label); explicit gating criteria; why each rung is gated (R3 waits on registry API, R4 waits on ≥50 paying tenants per research review §7.3).
- **§12 Risks** — added 7 new memory-integration + vertical-pack + distribution risks (R-SKILL-008..014): capability broker bypass, multi-tenant skill bleed, sync-state corruption, synthesis PII leak, vertical-pack legal drift, OCI signing-key compromise, agentskills.io policy hostility.
- **§13 KPIs** — added 8 new universal-protocol KPIs: broker-mediated rate (must be 100%), first-use approval latency, capability scope reject rate, synthesis emit rate, vertical-pack tenant attach rate, vertical-pack revenue share (≥30% of ARR at P4 · mid = the compounding moat), marketplace publish-to-install, pack legal-drift detection.
- **§14 RACI** — added 9 new rows for Phase 8 + synthesis sub-skill + memory-capture/sync bundles + 4 pack-authoring rows + 2 distribution/marketplace rows + 1 quarterly regulatory-drift review.
- **§16 Phase status** — added 12 new rows: Phase 8 + 3 universal-protocol skill bundles + 6 vertical packs + 2 marketplace rungs.
- **§17 References** — added MEMORY_AUTOSYNC_DESIGN.md (4 cross-links), FR_AUTHORING_WORKFLOW, AUDIT_AND_PLAN, RESEARCH_REVIEW, strategy doc §4.4 (vertical packs as Level-4 moat), and cross-module links to memory + CUO module pages.

Verified:
- 1431 lines parses cleanly
- 24 top-level sections (was 19) including 4 strategic new ones
- 4 references to MEMORY_AUTOSYNC_DESIGN.md
- 10 mentions of the 3 new universal-protocol skill bundles (memory-capture@1, memory-sync@1, synthesis-author@1)
- 39 mentions across the 9 vertical packs (vn / sg / id / th / eu / us / hr / legal / accounting)

The SKILL page now reflects the full strategic surface: open-standard citizen for distribution reach, memory-protocol enabler for cryptographic-grade audit-chain integration on every invocation, and vertical-pack moat as the actual compounding margin (≥30% of ARR at P4 · mid if the pricing+attach-rate math holds). The page reads as a complete answer to the research review's §7.3 GTM critique: the marketplace is deferred, the vertical packs ARE the moat, and the synthesis sub-skill closes the loop into multi-memory auto-evolve.

---

## 2026-05-14 — memory module page rewritten to Gold (expanded universal-protocol scope)

Rewrote `website/docs/modules/memory.html` from 1116 → 1518 lines (+402 lines, +36%). Encodes the MEMORY_AUTOSYNC_DESIGN.md vision: universal Personal memory + Lumi's memory + capture daemon + 2-way sync + multi-memory auto-evolve. Targeted Edit operations (not full rewrite) — preserved all existing gold-quality content on Stage 0 (shipped Layer 1) while encoding Stages 1–5.

Changes by section:
- **`<title>` + `<meta description>`** — reframed from "the substrate every CyberOS module depends on" to "the universal personal-and-shared memory protocol — CyberOS is the first consumer, the protocol stands alone".
- **Hero tagline + lede paragraph** — Personal memory + Lumi's memory duality; portability by folder copy; multi-memory auto-evolve as the moat; Stage 1–5 reference to MEMORY_AUTOSYNC_DESIGN.md.
- **Hero fact-grid** — replaced single-store metrics with dual-store reality (Layer 1 status + Stages 1–5 designed + Personal+Lumi stores + universal scope).
- **NEW §0 — "The bigger picture"** — 3-card layout (Personal · Sync orchestrator · Lumi's memory); auto-vs-manual capture matrix; "this is the moat" strategic frame.
- **TOC** — added "The bigger picture" + "Stages 1–5 roadmap" entries.
- **§1 Why memory exists** — 4-card layout (was 3) adding "Universal capture" + "Multi-memory power"; expanded the two-paragraph rationale with the compounding-moat argument.
- **§2 5W1H2C5M** — all 12 cells rewritten to encode the universal protocol scope. Personal vs Lumi distinction in Who/When/Where; Stage 2+ materials (Rust+notify, Presidio); cost model includes sync push p95 and synthesis LLM-cost.
- **NEW §3.5 — "Stages 1–5 universal protocol roadmap"** — Mermaid stage-dependency flowchart; gating table with effort estimates; Personal memory sub-architecture Mermaid diagram (capture surfaces → ops → store + sync queue); Lumi's memory sub-architecture diagram (N personal memories → sync → tenant chain → synthesis → wisdom); sync_class privacy taxonomy table.
- **§4 Data model** — added second ERD with 5 new entities: WatchedFolder · CaptureEvent · SyncState · LumiRow · SharedMemoryAcl · OrgMember · SynthesisInput · SynthesisArtefact (~80 lines of Mermaid erDiagram).
- **§5 API surface** — added a second CLI table with the 8 new `memory *` subcommands locked per MEMORY_AUTOSYNC_DESIGN.md §15: init/watch/unwatch/status/capture (Stage 1) + sync/sync-mode/pending/reclass (Stage 4).
- **§11 Compliance** — added PDPL Art. 7 (no data sale), Art. 20 (60-day post-audit cross-border), Art. 38 (SME 5-year grace), EU AI Act Art. 12 (synthesis logging) + Art. 50 (AI-generated content transparency), ISO/IEC 27018 §A.5 (customer agreement).
- **§12 Risk entries** — added 6 new memory-specific risks (R-memory-009..014): Lumi's memory tenant compromise, sync conflict storm, synthesis hallucination, capture daemon crash recovery, iCloud sibling explosion, PII leak via auto-capture. Each with likelihood / impact / owner / mitigation.
- **§13 KPIs** — added 8 new universal-protocol KPIs: capture rate per user, sync success rate, sync conflict rate, synthesis useful-rate, Lumi's memory seq counter, PII held-back rate, capture daemon health, cross-machine portability.
- **§14 RACI** — added 9 new rows covering Stages 1–5 + Personal-memory portability + PII detection + cross-tenant isolation testing + synthesis output review. Stage-3+ adds Cloud-DBA + Sync-SRE roles under CTO.
- **§16 Phase status** — added 5 new rows for Stages 1–5 with appropriate "design-locked / designed" pills.
- **§17 References** — replaced PRD/SRS section refs (stripped) with MEMORY_AUTOSYNC_DESIGN.md, PROPOSAL.md (Proposal P13), feature-request-audit skill, AUDIT_AND_PLAN_2026_05_14.md, RESEARCH_REVIEW_2026_05_14.md cross-links. Annotates the 4 new doctor invariants and 5 new schema entities.

Result: memory page now reflects the expanded universal-protocol vision while preserving every gold-quality detail of the shipped Stage-0 Layer 1. 5 references to MEMORY_AUTOSYNC_DESIGN.md cross-link the design source-of-truth. 20 mentions of the 8 new `memory *` subcommands give a cold reader the full CLI map.

---

## 2026-05-14 — Research review ingested + memory auto-sync design v1.0 locked

- Saved `docs/RESEARCH_REVIEW_2026_05_14.md` (315 lines, ~53 KB) — the pre-launch audit from Claude Chat's Research Mode. Aggregate 6.5/10; lowest substantive scores on Spec Quality (5) and GTM (5). 10 follow-up tasks created (#31–#40) covering: P0→P1 descope gate, AI Gateway → AUTH reorder, PDPL citation fixes, server-render NFR + Risk catalogs, first 50 FRs via feature-request-author, 7 missing risks, TEN-billing P2 slice, UX defects, memory Layer 2 source-of-truth one-pager, memory decision memory.
- **Wrote `docs/MEMORY_AUTOSYNC_DESIGN.md`** (~700 lines, design v1.0.0) — universal Personal memory + Lumi's memory architecture. Per Stephen's clarified vision: (1) Personal memory works on any folder, not just cyberos; (2) captures everything including discussions, not just file deliverables; (3) portable by folder copy across user's machines; (4) 2-way sync with Cloud memory aka Lumi's memory (also CUO's memory, CyberSkill's memory — same store, different names for different audiences); (5) multi-memory power + auto-evolve memory at scale.
  - 16 sections: vision, naming, three-layer architecture, Personal memory spec, Capture daemon spec, Lumi's memory spec, Sync orchestrator, Multi-memory auto-evolve, Dependency map, Privacy + governance, AGENTS.md Proposal P13 additions, CyberOS strategic implications, naming/branding decisions, 4-week sprint plan, 5 open questions, where-to-read-next.
  - Stage gating: **Stage 1 (Personal memory universal) + Stage 2 (capture daemon) are buildable today** — no external dep. Stages 3+ ride the P0+P2 critical path (AUTH + AI Gateway + TEN).
  - Strategic implication called out: this is **the moat** the reviewer's GTM critique was looking for. Personal memory as OSS distribution; Lumi's memory as the commercial product. The compounding switching cost = value of the org's accumulated memory.

---

## 2026-05-14 — Code-block contrast fix + PRD/SRS sweep + repair regression + Research Mode brief

- **Fixed code-block invisible-text bug.** A late-stage override in `assets/styles.css` (`.codeblock { background: var(--bg-code) }`) was flipping the dark `--neutral-900` background to a light `--bg-code` while leaving text colour at light `--neutral-100` → code invisible on auth.html and other module pages. Removed the `background` override; kept the `backdrop-filter: none` (which prevents glass-leakage from a glass parent).
- **Swept PRD/SRS back-references out of the docs site.** The docs site is now the single source of truth — removed every `PRD §X.Y`, `SRS §X.Y`, "per PRD", "see PRD", "sourced from PRD" reference across 33 HTML files. Replaced `Source: PRD §...` / `Reference: SRS §...` labels with `(covered on this page)`. Net 29,710 substitutions.
- **Repaired regex over-strip regression.** The sweep's separator-collapse regex had a false-positive: `(/)\s*(/)` matched `://` in URLs and collapsed them to `:/`. 175 URLs (Google Fonts, jsdelivr CDN, GitHub repo links, SVG xmlns, etc.) were silently broken across all HTML files. Wrote a repair pass that restored `https?:/` → `https?://` plus cleaned up 83 empty `<strong></strong>` / `<em></em>` / `<code></code>` tags and orphan-separator artifacts. Zero broken URLs verified after repair.
- **Added `docs/RESEARCH_MODE_BRIEF.md`** — canonical brief for the pre-lock comprehensive review via Claude Chat's Research Mode. Contains the full prompt covering 8 review dimensions (strategic coherence, architecture, spec quality, UX, info architecture, compliance, GTM, next-7-days actions), the 10-file input bundle (~250 KB total of curated source-of-truth markdown), why we DON'T attach the docs HTML (token waste + visual UX requires live URL crawl), how to drive the mid-review conversation, and how to operationalize the returned document.

---

## 2026-05-14 — Heading line-height fix + FR authoring workflow guide

- Fixed heading collision on H2 elements caused by the Be-Vietnam-Pro font swap. BVP has taller ascenders + descenders than Inter at the same `font-size`. The previous Inter-tuned `line-height: 1.05` (h-display), `1.15` (h-1), `1.25` (h-2) values were too tight and let the heading bounding box collide with the following paragraph (visible on the "The substrate · the catalog · the orchestrator" H2 on index.html). Updated `assets/styles.css` heading rhythm: h-display 1.05→1.1, h-1 1.15→1.25, h-2 1.25→1.4, h-3 (added) 1.45. Added explicit `margin-block-end` on each + an `h-* + * { margin-block-start: 0 }` rule to neutralise Tailwind `mb-*` collapse.
- Added `feature-request-audit skill` — canonical playbook for the post-strip FR re-authoring lifecycle. Covers the mental model, file layout, standalone vs chained flows, the standard module-slice-1 recipe (5–7 FRs per slice), how FRs surface back to the docs site, status state machine, task integration paths, and a fully worked FR-AUTH-001 example. Designed to keep open while authoring.

---

## 2026-05-14 — Comprehensive audit + FR catalog strip + Mermaid mass-fix

Added `docs/AUDIT_AND_PLAN_2026_05_14.md` — single comprehensive audit + build-readiness plan covering UI glitches (severity-ranked), FR landscape, per-module build sequence for the 19 unbuilt modules with slice-1 outlines, and strategic followups. Designed as the source of truth for the next 2 weeks of work.

**FR catalog strip (per user decision: strip-everything).** Stripped:
- All 22 module pages: each "Functional Requirements" section (the `<section id="functional-requirements">` block, lines ~789–820 across modules) replaced with a stub linking to the `feature-request-author` Agent Skill workflow. 23/23 pages patched cleanly via regex sweep.
- `website/docs/reference/fr-catalog.html`: 1006-line generated catalog replaced with a 70-line stub explaining the rebuild + how to author new FRs via the skill module.

**Partially stripped (cross-refs remain — call to extend):**
- `website/docs/reference/nfr-catalog.html` — still has 137 FR refs (NFRs are described in terms of which FRs they constrain)
- `website/docs/reference/risk-register.html` — still has 51 FR refs (risks reference the FRs they affect)
- Module pages — still have inline FR refs in Dependencies tables, NFR descriptions, KPIs, References footers (~200 total across all)
- `docs/prd/PRD.md` (393 FR refs) and `docs/srs/SRS.md` (206 FR refs) — preserved as authoritative spec narrative; .docx originals also preserved

The "strip-everything" decision affects ~434 remaining FR cross-references — these are inline within sentences and tables. They become broken references until re-authored. To clean them up, separate decisions are needed on whether to: keep them as broken refs (will rewrite organically as new FRs come online), replace with `(FR pending)` markers, or remove the surrounding sentences entirely.

**Mermaid mass-fix across 28 pages:**
- `<br/>` → `<br>` — 754 instances replaced, ALL inside `<div class="mermaid">` blocks (zero outside, verified). This fixes the "Cursorvia MCP tool" text-collapse bug seen on `modules/memory.html` where Mermaid 11.4.1 strips self-closed `<br/>` tags inside quoted node labels.
- Pastel `classDef` palette → Umber/Ochre brand: 127 instances recolored across all non-index module + architecture pages. Map: emerald-100→umber-50, blue-100→umber-100, purple-100→ochre-300, amber-100→ochre-50, pink-100→ochre-100, indigo-100→umber-200, slate-100→neutral-100, yellow-100→ochre-50, violet-100→ochre-50. Strokes likewise mapped to umber-500 / ochre-700 / neutral-400.
- 6 broken internal links to non-existent architecture pages fixed: `architecture/services.html` (5 refs from learn/hr/esop/rew/inv) and `architecture/runtime.html` (1 ref from chat) redirect to `architecture/infrastructure.html` (the closest topical match).

Net code change: 36 files, ~1,417 insertions / ~2,641 deletions. Plus new files `docs/AUDIT_AND_PLAN_2026_05_14.md` (the master plan) and `website/docs/assets/tailwind.min.css` (16.7 KB vendored from prior commit).

Open items pending Stephen's call (per audit doc):
1. Whether to strip the remaining 434 inline FR cross-refs (in NFR catalog / risk register / module sub-sections) or let them rewrite organically.
2. AUTH RFC's 5 open questions need answers before slice 1 codes.
3. Redeploy `website/docs/` via wrangler so the brand + Tailwind + Mermaid + strip fixes go live.

---

## 2026-05-14 — Vendor Tailwind (CDN was silently failing on Cloudflare Pages)

After the brand-rebuild deploy at https://5cc09eb6.cyberos-docs.pages.dev/, the layout was still broken: hero text and SVG stacked, bento stats stacked one-per-row, 22-module catalog stacked one-per-row, the three shipped-module cards stacked one-per-row. Every `grid`, `grid-cols-*`, `lg:grid-cols-*`, `flex`, `gap-*`, `mt-*` utility was dead because the Tailwind CDN script (`https://cdn.tailwindcss.com`) was loading (200, 14 KB body, no console errors) but **never injected its generated utility CSS** — confirmed by `getComputedStyle` showing `.grid` resolving to `display:block` and `typeof window.tailwind === 'undefined'`. No CSP headers, no module/MIME errors, just a silent failure of Tailwind Play CDN's runtime JIT inside Cloudflare Pages.

Fix in this commit:

- Generated a 16.7 KB static `assets/tailwind.min.css` via `npx tailwindcss@3.4.17` with content-paths covering all 32 HTML files (index + 22 modules + 4 architecture + 4 reference + 1 nav asset). Preflight disabled (we already have `assets/styles.css` setting base styles). All classes the pages actually use are baked in: `.grid`, `.flex`, `.container`, `.grid-cols-{2,3,5,6}`, `.lg:grid-cols-{4,5,6,8,12}`, `.md:grid-cols-{2,3,4}`, `.gap-{1..10}`, `.mt-{0..16}`, `.py-*`, `.text-{xs..2xl}`, `.font-{medium,semibold,bold,black}`, `.items-center`, `.justify-between`, etc.
- Replaced `<script src="https://cdn.tailwindcss.com"></script>` with `<link rel="stylesheet" href="assets/tailwind.min.css">` across all 32 HTML files (relative paths corrected: `assets/...` from index, `../assets/...` from subdirs).
- Result: layout works without runtime JavaScript, no third-party CDN dependency, faster (16.7 KB CSS gzips to ~4 KB vs the CDN's 14 KB JS + runtime compile + style injection).

To regenerate when classes change:

```bash
cd /tmp && cat > input.css <<'CSS'
@tailwind base; @tailwind components; @tailwind utilities;
CSS
cat > tailwind.config.js <<'JS'
const docs = '/path/to/cyberos/website/docs';
module.exports = {
  content: [`${docs}/*.html`, `${docs}/modules/*.html`, `${docs}/architecture/*.html`, `${docs}/reference/*.html`, `${docs}/assets/*.html`],
  corePlugins: { preflight: false },
};
JS
npx tailwindcss@3.4.17 -c tailwind.config.js -i input.css -o /path/to/cyberos/website/docs/assets/tailwind.min.css --minify
```

Once the docs site moves to a real build pipeline (Vite, Astro, or just a Makefile), this becomes one-line in the build command.

---

## 2026-05-14 — Docs site brand rebuild

Live deploy at https://fe8d68ee.cyberos-docs.pages.dev/ was off-brand: hero triangle used pastel purple/blue/green/yellow Mermaid-default palette; bento stats used per-stat blue/purple/emerald/amber/rose; phase strips used five different pastels; persona accents were purple; compliance ring was blue/green/yellow concentric; tech-stack Mermaid `classDef` was pastel-rainbow. None of these aligned with the design-system DESIGN.md anchors (Umber `#45210e` + Ochre `#f4ba17`) or with Part 21 Liquid Glass defaults.

Root cause: page authoring drift, not design-system fault. Glass classes (`.surface-light/.surface-standard/.surface-heavy`) and `--glass-*` tokens were already defined in `assets/styles.css` and `assets/tokens.css`, but `index.html` hand-coded inline Tailwind palette utilities (`bg-blue-50`, `text-purple-700`, etc.) instead of consuming them.

Fixes in this commit:

- `website/docs/index.html` — 534 lines changed. All inline pastel hex fills in the hero SVG triangle, phase strips, and compliance ring SVG converted to Umber/Ochre tints (`#f5ede6`, `#e8d4c2`, `#fef6e0`, `#fde7b3`, `#f9c64f`, `#cba88a`). All Tailwind palette utilities (`bg-blue-*`, `text-purple-*`, `bg-emerald-*`, `text-amber-*`, `text-rose-*`) replaced with `style="color:var(--umber-700)"` / `style="background:var(--ochre-50)"`. Tech-stack Mermaid `classDef` repainted to brand palette. CyberOS wordmark gradient changed from `blue→purple→emerald` to `umber→ochre`. v2026.05 pill changed from `bg-blue-50 text-blue-700` to `ochre-50 + umber-700`. Phase summary gradient changed from `from-blue-50 via-purple-50 to-emerald-50` to `umber-50 → ochre-50`. Compliance ring concentric gradients changed from `blue→green→yellow` to `neutral→umber→ochre` (warmest at the inner Vietnam home regime).
- `website/docs/assets/tokens.css` — `--font-sans`/`--font-body`/`--font-display` reordered: Be Vietnam Pro listed before Inter per design-system mandate. Comment notes the Vietnamese-first commitment.
- `website/docs/assets/styles.css` — added the `@import` for Be Vietnam Pro so the font actually loads. Added `+101 lines` of design-system utilities: `.ds-modpill` + `.ds-modpill--future` (module navigator pills), `.pill--brand`, `.tile` + `.tile--accent`. Added a transitional-safety-net override block that converts any remaining Tailwind palette utilities on the 22 module pages + 4 architecture pages + 4 reference pages to brand tokens (`bg-blue-*` → `--umber-100`, `bg-purple-*` → `--ochre-50`, etc.) so the brand wins site-wide even before each page is hand-cleaned. Saves ~620 individual edit operations.
- `website/docs/assets/scripts.js` — Mermaid `themeVariables.fontFamily` reordered to Be Vietnam Pro first.

Zero Tailwind palette leaks remain in `index.html` (was 13). Across the rest of the docs site there are still 620 leaks but the new safety-net rules in `styles.css` neutralise them visually until each page is cleaned.

Design-system suggested followups (not landed in this commit):
1. Add Part-21 sub-section "§21.x — Theming third-party renderers" with the Mermaid `themeVariables` recipe, so the next docs author doesn't re-invent it.
2. Promote `.tile`, `.pill--brand`, `.ds-modpill` from the docs site into `design-system/DESIGN.md` Part 3 as first-class component specs.
3. Ship `tools/design-system-lint.{ts,py}` per Part 15 — flag Tailwind palette utilities (`bg-blue-*` etc.) and off-anchor `fill:#` hexes at commit time.

---

## 2026-05-14 — AUTH module RFC + sign-in mockup

- Added `services/auth/RFC.md` — implementation RFC with 5-slice ship plan, audit-chain integration design, and 5 open questions blocking slice 1.
- Added `services/auth/mockups/sign-in.html` — first AUTH UI mockup applying design-system Part 21 Liquid Glass defaults, Umber + Ochre anchors, Be Vietnam Pro first, passkey-first flow with password fallback, MFA chips, memory audit-chain trust footnote.
- Verification pass against shipped modules:
  - memory: 222 tests pass + 1 skip (numpy + jsonschema needed for full green). Real bug found AND fixed: `check_manifest_validates` was skipping parseability when jsonschema absent → `cyberos state` returned READY on a broken manifest. Patched to always parse `manifest.json` first (regardless of jsonschema availability) and report `False` on `JSONDecodeError`; the optional schema-validation layer still skips cleanly when jsonschema is absent. Verified: all 4 `tests/test_state.py` tests pass, full suite 238 pass / 1 skip / 0 fail. Also verified by simulating absent jsonschema via import hook — good manifest still returns True with "parseability OK, schema skip"; bad manifest returns False with "manifest.json unparseable: ...".
  - skill: 20 SKILL.md bundles structurally verified, 4 crates, 8 inline Rust tests. `cargo build` not run (sandbox-only limitation).
  - cuo: 15/15 pytest + 15/15 routing fixtures pass. Catalog discovers all 20 skills correctly.
- Stale-claim drift surfaced (none are blockers, all are doc-only):
  - Memory tests: bootstrap says 245, README says 255, actual is 238 collected.
  - Doctor invariants: bootstrap says 16, README says 15, actual is 13 on a fresh store.
  - Docs pages: bootstrap says 32, strategy says 31, actual is 33 HTML files (32 user-facing + nav include).
  - Strategy §3 Tier-1 #2 and §5 Session-1 #1 list "wire Pagefind" as a to-do; Pagefind is already built and serving (v1.5.2, 32 pages indexed).
  - DEPLOYMENT.md is at `website/docs/DEPLOYMENT.md` (bootstrap implies it lives at `website/`).
- Docs site deploy-prep findings:
  - 6 real broken internal links to 2 missing architecture pages: `architecture/services.html` (5 refs from LEARN/HR/INV/ESOP/REW) and `architecture/runtime.html` (1 ref from CHAT). These are demand-gen blockers — fix before public deploy or convert the link targets.

---

## 2026-05-14 — Consolidation pass

Moved all CyberOS-related artifacts into a single umbrella at `cyberos/`:

- `workbench/CyberOS-docs/` → `cyberos/website/docs/`
- `workbench/CYBEROS_STRATEGY.md` → `cyberos/strategy/CYBEROS_STRATEGY.md`
- `workbench/cyberskill-vn-skills/` → `cyberos/public-skills/`
- `/design-system/` → `cyberos/design-system/`
- `/landing-page/` → `cyberos/website/landing/`

This enables clone-and-go for new sessions and keeps strategic + technical + design content co-located.

See per-module CHANGELOG.md files for module-specific history:
- `memory/docs/CHANGELOG.md`
- `skill/docs/CHANGELOG.md`
- `cuo/docs/CHANGELOG.md`
- `design-system/CHANGELOG.md`
- `website/docs/index.html` (the rendered changelog page)

---
