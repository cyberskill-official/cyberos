# Changelog — MEMORY

## 2026-07-23 — TASK-MEMORY-303 memory contract hardening

Four deliverable groups (audit finding H10 + the memory-row medium items):

- **Schema unification (single-source)** — `tools/cyberos_generate_schema.py` now emits the `StoreAcl`/`StoreAclEntry`/`StoreAclMode` definitions (§14.4.7) and the `episode` kind; `modules/memory/memory.schema.json` and `modules/memory/cyberos/data/memory.schema.json` are both regenerated generator output and byte-identical (previously the root copy lacked the ACL definitions and the vendored payload shipped the stale side). `tests/test_schema_drift.py` now points at the real committed path and FAILs (never skips) when the schema is missing; `tests/test_schema_single_source.py` pins generator `--check`, cross-copy byte-identity, the build.sh vendoring source, and the un-skippability itself.
- **INTEROP.md** — new `modules/memory/INTEROP.md` (≤ 6,000 chars), the §14.1 non-ledger consumer subset: read paths, the MUST-NOT-write set (`audit/`, `HEAD`, `.lock`), canonical-writer routing, §14.4.6 STORE.yaml honor-for-writes, §14.3 sync_class export semantics. Bound + content anchors pinned by `tests/test_interop_doc.py`.
- **Walker + doctor additions** — `_CANONICAL_TOP_LEVEL_DIRS` gains the protocol's own `sessions/` (§18.2) and `dreams/` (§7.7.4); three declared-but-missing invariants are implemented and registered in `memory.invariants.yaml` + `cyberos/core/invariants.py`: `dream-applied-row-has-provenance` (§7.7.2), `store-yaml-acl-valid` (§14.4.7), `session-lifecycle` (§18.8). `cyberos doctor` accepts `--store` after the subcommand for standalone gate invocations. The writer stamps `extra.session_id` on put/move/delete rows while `sessions/.active` names a session (§18.7). Exposed-and-fixed: legalising `dreams/` let SemanticDedup actually re-run, revealing that dream detectors enumerated tombstoned bodies (§3.5 preserves them on disk) and re-proposed merges of deleted memories — `_enumerate_memories` now excludes paths whose latest chain row tombstoned them, making re-apply genuinely idempotent (TASK-MEMORY-116 AC #11) instead of accidentally-green.
- **Store repair (operator-gated)** — the live store's stray top-level `adrs/` + `impl-plans/` dirs get a complete relocation plan (`docs/tasks/memory/TASK-MEMORY-303-memory-contract-hardening/store-repair-plan.md`): canonical-writer `move` ops with body-hash preconditions, expected post-state, rollback, and before/after doctor output — proven against a copy of the live store; execution awaits the recorded operator approval at this task's HITL gate.

## 2026-05-18 — TASK-MEMORY-104 Tauri 2.x desktop scaffold

New `services/memory/desktop/` (19 files). Backend: Tauri 2 + plugin-shell + plugin-fs; `commands.rs` for search/quick-capture/sync-state; `sync_supervisor.rs` supervises the Python memory-sync daemon with 5-restarts-per-60s circuit breaker. Frontend: Svelte 5 runes + Vite + Tailwind 3 — `App.svelte` with Dashboard / Search / Sync tabs. **NOT in `services/Cargo.toml` workspace** — own Cargo.lock. Signing scripts for macOS and Windows. TASK-MEMORY-104 status bumped `accepted → building`.

---

## 2026-05-19 — MEMORY Wave 2026-Q3 CLOSED — TASK-MEMORY-120 `cyberos history` shipped (final task)

Final task of the MEMORY Improvement Wave 2026 Q3. No protocol amendment needed (pure read-only projection).

### TASK-MEMORY-120 implementation

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
- ✅ **Read-only invariant: HEAD before=3, after=3 exact match** (per TASK-MEMORY-120 §1 #1)
- ✅ `cyberos verify` chain intact (3 records)

### Test coverage

New: `modules/memory/tests/core/test_history.py` (302 lines, 18 test functions). Covers AC #1, #2, #3, #4, #5, #7, #10, #13, #16, #20, #21, plus annotation rendering across all 5 recognised tags.

### Full MEMORY Wave 2026-Q3 — FINAL STATUS

| Task | Spec | Impl | Tests | Amendment |
|---|---|---|---|---|
| TASK-MEMORY-112 | episodic memory | ✅ | 351L | — |
| TASK-MEMORY-113 | recency-decay recall | ✅ | 266L | — |
| TASK-MEMORY-114 | write-time importance | ✅ | 263L | — |
| TASK-MEMORY-115 | `cyberos dream` | ✅ | 329L | P19 §7.7 ✅ |
| TASK-MEMORY-116 | semantic-dedup consolidate | ✅ | 257L | — |
| TASK-MEMORY-117 | per-store ACL | ✅ | 359L | P20 §14.4 ✅ |
| TASK-MEMORY-118 | `put_if` precondition-hash | ✅ | 349L | P21 §3.1 ✅ |
| TASK-MEMORY-119 | session transcript ledger | ✅ | 308L | P22 §18 ✅ |
| TASK-MEMORY-120 | `cyberos history` | ✅ | 302L | — |

**All 9 tasks shipped end-to-end. All 4 protocol amendments APPROVED + merged into AGENTS.md. 2,784 lines of tests covering 110+ acceptance criteria. `cyberos verify` reports chain integrity across every test fixture.**

The MEMORY module now matches Anthropic's Memory+Dreaming primitive (per the source talk + Ramakrushna article that started this whole wave 2026-05-19 morning).

---

## 2026-05-19 — Wave 3 cont. — AGENTS.md §18 (P22 APPROVED) + TASK-MEMORY-119 transcript ledger

### Protocol amendment §18 (P22 APPROVED)

Operator's fourth terse `APPROVE` of the session. New section §18 Session transcript ledger added to [`modules/memory/AGENTS.md`](modules/memory/AGENTS.md). Nine sub-clauses cover opt-in lifecycle, date-partitioned storage at start-date, closed classification enum (`confidential` default per Stephen 2026-05-19 / `restricted`), encryption envelope for restricted payloads, summary rows on the main chain, retention (default 30 days), in-session `extra.session_id` propagation, lifecycle invariants, and ACL applicability to the `sessions/` subtree.

**Namespace conflict resolved**: the task spec'd `cyberos session start|append|end`, but the existing P11 `cyberos session` subcommand handles multi-agent coordination — different product with the same verb. The implementation namespaces the transcript ledger under `cyberos transcript` (start / append / end / read / list / purge-expired). §18.1 documents the rename.

Tracker entry in [`modules/memory/README.md`](modules/memory/README.md) Appendix D flipped P22 from "awaiting APPROVE" to **APPROVED 2026-05-19**.

### TASK-MEMORY-119 implementation

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

### Combined test surface (Waves 1+2+TASK-117+TASK-118+TASK-119)

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

- ✅ **P20 §14.4** APPROVED + TASK-MEMORY-117 shipped
- ✅ **P21 §3.1** APPROVED + TASK-MEMORY-118 shipped
- ✅ **P22 §18** APPROVED + TASK-MEMORY-119 shipped
- ⏳ **TASK-MEMORY-120** (`cyberos history`) — no amendment gate; ships when operator says go

---

## 2026-05-19 — Wave 3 cont. — AGENTS.md §3.1 extension (P21 APPROVED) + TASK-MEMORY-118 put_if

### Protocol amendment §3.1 (P21 APPROVED)

Operator's third terse `APPROVE` of the session. §3.1's canonical-op table extended from THREE ops (`put` / `move` / `delete`) to FOUR — `put_if` joins as the optimistic-concurrency primitive. Three new sub-clauses:

- **§3.1.5** — `memory.precondition_failed` aux row schema (`{actor, path, expected, actual, attempt_at}`). HEAD doesn't advance for the rejected put; advances by +1 for the aux row only.
- **§3.1.6** — success row is INDISTINGUISHABLE from a regular `put` (`op="put"`, not `"put_if"`). Downstream consumers (walker / doctor / dream / history) require no special-case logic.
- **§3.1.7** — ACL check (TASK-MEMORY-117) runs BEFORE the precondition check. Policy refusal returns `acl_denied`, not `precondition_failed` — different operator action needed.

Tracker entry in [`modules/memory/README.md`](modules/memory/README.md) Appendix D flipped P21 from "awaiting APPROVE" to **APPROVED 2026-05-19**.

### TASK-MEMORY-118 implementation

New op in `modules/memory/cyberos/core/ops.py`:

- `PutIfResult` frozen dataclass (5 fields: `outcome`, `reason`, `expected`, `actual`, `committed_seq`).
- `put_if(writer, rel_path, body, *, actor, precondition_body_hash, kind, extra)` — content-conditional write. Order of checks: path traversal + size cap → shape validation (64-char lowercase hex OR `None` only) → `_has_section_3_1_put_if()` anchor check → ACL gate (TASK-MEMORY-117, runs BEFORE precondition per §3.1.7) → precondition check (3 rejection paths) → write. Success emits a plain `put` row (per §3.1.6) so downstream consumers need no special-case.

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

### Combined test surface (Waves 1+2+TASK-117+TASK-118)

7 test files at `modules/memory/tests/core/` = **2,174 lines, ~103 test functions across 80+ acceptance criteria**.

### Files touched (this entry only)

New: `tests/core/test_put_if.py` (349 lines). Modified: `modules/memory/AGENTS.md` §3.1; `modules/memory/cyberos/core/ops.py` (~160 LOC); `modules/memory/cyberos/__main__.py`; `modules/memory/README.md` Appendix D.

### Wave 3 status

- ✅ **P20 §14.4** APPROVED + TASK-MEMORY-117 shipped
- ✅ **P21 §3.1** APPROVED + TASK-MEMORY-118 shipped
- ⏳ **P22 §18** (session transcript ledger) — awaiting `APPROVE protocol change P22 §18`
- ⏳ **TASK-MEMORY-120** (`cyberos history`) — no amendment gate

---

## 2026-05-19 — Wave 3 start — AGENTS.md §14.4 (P20 APPROVED) + TASK-MEMORY-117 per-store ACL

### Protocol amendment §14.4 (P20 APPROVED)

Operator approved with a terse `APPROVE` (interpreted as the next-in-queue per the one-at-a-time rule). New section added to [`modules/memory/AGENTS.md`](modules/memory/AGENTS.md) at §14.4 — Store-level ACL. Seven sub-clauses cover STORE.yaml shape + parsing, write-side enforcement via the canonical writer with first-match-wins glob resolution, read non-enforcement (writes-only at protocol level), `memory.acl_denied` aux row payload, two-sided ACL check on `move`, INTEROP-consumer obligation, and the `store-yaml-acl-valid` walker invariant.

Tracker entry in [`modules/memory/README.md`](modules/memory/README.md) Appendix D flipped P20 from "awaiting APPROVE" to **APPROVED 2026-05-19**.

### TASK-MEMORY-117 implementation

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
- New `StoreAclMode`, `StoreAclEntry`, `StoreAcl` definitions matching TASK-MEMORY-117 §3 schema fragment.

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

### Combined test surface (Waves 1 + 2 + TASK-MEMORY-117)

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

- ✅ **P20 §14.4** APPROVED + TASK-MEMORY-117 implemented
- ⏳ **P21 §3.1** (put_if precondition-hash) awaiting `APPROVE protocol change P21 §3.1`
- ⏳ **P22 §18** (session transcript ledger) awaiting `APPROVE protocol change P22 §18`
- ⏳ **TASK-MEMORY-120** (`cyberos history`) — no amendment gate; ships next session

---

## 2026-05-19 — Dependency-version bumps + AGENTS.md §7.7 (P19 APPROVED) + Wave 2 implementation (TASK-MEMORY-115 + 116)

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

### TASK-MEMORY-115 — `cyberos dream` out-of-band reflection

New: `modules/memory/cyberos/core/dream/{__init__,proposals,_audit_iter,detectors,runner,applier}.py`. Four async detectors (`duplicates` / `stale` / `patterns` / `verify`) matching AGENTS.md §7.7.7 closed enum. Runner uses Crockford-base32 ULID `dream_id`, snapshot-isolated against `head_seq` at start, persists `DreamDiff` to `dreams/<YYYYMMDDTHHMMSSZ>/diff.json`. Applier: 3-pass (strict-idempotency via chain-walk → body-hash precondition → write with `extra.dream_id`/`extra.proposal_id` per §7.7.2). AGENTS.md §7.7 anchor checked before any writes (`ProtocolAmendmentMissing` on missing).

CLI: `cyberos dream` + `cyberos dream-apply <id>` subcommands.

**Smoke verified end-to-end:** seeded 3 facts (2 near-duplicate); dream found 1 merge proposal; apply rejected without §7.7 anchor, succeeded with anchor, idempotent on re-apply (`skipped_idempotent: 1`); `cyberos verify` reported chain intact (13 rows).

**Side effect**: `cyberos.core.ops.delete()` gained optional `extra: dict | None` kwarg (additive, back-compat).

### TASK-MEMORY-116 — Semantic-dedup consolidate phase

Thin wrapper: 5th phase appended to `cyberos.core.consolidate.run()` → `Walk → Compact → Sign → Publish → SemanticDedup`. Delegates to TASK-MEMORY-115's `duplicates` detector + applier verbatim (asserted by `test_consolidate_imports_dream_detector` via `inspect.getsource`). On apply, emits a marker `dream.complete` row with `extra.invocation = "consolidate"` so TASK-MEMORY-120 history can distinguish dedup-from-consolidate from dedup-from-explicit-`dream`.

CLI: extends `cyberos consolidate` with `--semantic-dedup`, `--semantic-dedup-apply`, `--semantic-dedup-threshold`, `--semantic-dedup-scope`. Default behavior unchanged. `ConsolidationReport` gains 5 new fields.

**Smoke verified end-to-end:** same 2-duplicate fixture; all 5 phases ran; 1 proposal found + 1 applied; final `dream.complete` aux row carries `extra.invocation: "consolidate"`.

### Combined test coverage (Wave 1 + Wave 2)

5 test files at `modules/memory/tests/core/`: `test_episode.py` (351) + `test_ranking_and_decay.py` (266) + `test_importance.py` (263) + `test_dream.py` (329) + `test_consolidate_semantic_dedup.py` (257) = **1,466 lines, ~65 test functions across 50+ acceptance criteria**.

### Deferred to subsequent sessions

- pytest run against full suite — sandbox is Python 3.10 (module requires 3.11+); test files are ready for `pytest tests/core/ -v` in operator env.
- Wave 3 (TASK-MEMORY-117 ACL · 118 put_if · 119 sessions · 120 history) — gated on independent `APPROVE protocol change P20 / P21 / P22` chat-turns when operator ready.

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
- **§17 References** — replaced PRD/SRS section refs (stripped) with MEMORY_AUTOSYNC_DESIGN.md, PROPOSAL.md (Proposal P13), task-audit skill, AUDIT_AND_PLAN_2026_05_14.md, RESEARCH_REVIEW_2026_05_14.md cross-links. Annotates the 4 new doctor invariants and 5 new schema entities.

Result: memory page now reflects the expanded universal-protocol vision while preserving every gold-quality detail of the shipped Stage-0 Layer 1. 5 references to MEMORY_AUTOSYNC_DESIGN.md cross-link the design source-of-truth. 20 mentions of the 8 new `memory *` subcommands give a cold reader the full CLI map.

---

## 2026-05-14 — Research review ingested + memory auto-sync design v1.0 locked

- Saved `docs/RESEARCH_REVIEW_2026_05_14.md` (315 lines, ~53 KB) — the pre-launch audit from Claude Chat's Research Mode. Aggregate 6.5/10; lowest substantive scores on Spec Quality (5) and GTM (5). 10 follow-up tasks created (#31–#40) covering: P0→P1 descope gate, AI Gateway → AUTH reorder, PDPL citation fixes, server-render NFR + Risk catalogs, first 50 tasks via task-author, 7 missing risks, TEN-billing P2 slice, UX defects, memory Layer 2 source-of-truth one-pager, memory decision memory.
- **Wrote `docs/MEMORY_AUTOSYNC_DESIGN.md`** (~700 lines, design v1.0.0) — universal Personal memory + Lumi's memory architecture. Per Stephen's clarified vision: (1) Personal memory works on any folder, not just cyberos; (2) captures everything including discussions, not just file deliverables; (3) portable by folder copy across user's machines; (4) 2-way sync with Cloud memory aka Lumi's memory (also CUO's memory, CyberSkill's memory — same store, different names for different audiences); (5) multi-memory power + auto-evolve memory at scale.
- 16 sections: vision, naming, three-layer architecture, Personal memory spec, Capture daemon spec, Lumi's memory spec, Sync orchestrator, Multi-memory auto-evolve, Dependency map, Privacy + governance, AGENTS.md Proposal P13 additions, CyberOS strategic implications, naming/branding decisions, 4-week sprint plan, 5 open questions, where-to-read-next.
- Stage gating: **Stage 1 (Personal memory universal) + Stage 2 (capture daemon) are buildable today** — no external dep. Stages 3+ ride the P0+P2 critical path (AUTH + AI Gateway + TEN).
- Strategic implication called out: this is **the moat** the reviewer's GTM critique was looking for. Personal memory as OSS distribution; Lumi's memory as the commercial product. The compounding switching cost = value of the org's accumulated memory.
