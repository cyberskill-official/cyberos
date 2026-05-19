# MEMORY — CyberOS memory Protocol & Reference Implementation

> **Local-first, audit-chained, append-only personal memory store for AI-assisted work.** Drop the protocol files into any project; your agent (Claude / Cursor / Codex / Copilot / Cowork) loads them and starts building a project-local **memory** you can copy, audit, encrypt, and merge with teammates.

| | |
|---|---|
| **Module name** | `memory` |
| **Spec status** | Normative — see [`AGENTS.md`](AGENTS.md) (Layer-1 protocol, 246 lines, ~3.6k tokens) + [`INTEROP.md`](INTEROP.md) (cross-agent subset, ≤6,000 chars) + [`memory.schema.json`](memory.schema.json) + [`memory.invariants.yaml`](memory.invariants.yaml) |
| **Implementation** | Python 3.10+ (this directory's `cyberos/` package); `pip install -e .` exposes the `cyberos` console script |
| **Test suite** | 255 green; `cyberos doctor` passes 15/15 invariants on the live memory |
| **License** | MIT |

This README is the **single source of truth** for the MEMORY module's installation, operation, and design rationale. It consolidates what previously lived in `docs/README.md` (step-by-step install), `docs/AUTOSYNC_DESIGN.md` (background-sync design), `docs/EVOLUTION.md` (history + open questions), `docs/LAYER_2_SOURCE_OF_TRUTH.md` (Layer-2 design), and `docs/PROPOSAL.md` (open / shipped design decisions).

**The protocol artefacts remain at module root as separate files** because they are referenced by name from the spec:

- [`AGENTS.md`](AGENTS.md) — Layer-1 normative spec (symlinked from the repo root as `CLAUDE.md` + `AGENTS.md` so every agent loads it)
- [`INTEROP.md`](INTEROP.md) — non-ledger consumer subset (per AGENTS.md §14.1)
- [`memory.schema.json`](memory.schema.json) — machine schema for memory files (per §3.3 / §5.2 / §6.2)
- [`memory.invariants.yaml`](memory.invariants.yaml) — walker input invariants (per §1)
- [`CHANGELOG.md`](CHANGELOG.md) — release history (2,202 lines, separate by convention)

---

## Table of contents

1. [What it does](#1-what-it-does)
2. [Status](#2-status)
3. [Quick start](#3-quick-start)
4. [Run locally (full install)](#4-run-locally-full-install)
5. [The four supported workflows](#5-the-four-supported-workflows)
6. [Step-by-step: starting from zero](#6-step-by-step-starting-from-zero)
7. [Audit (verify, walk, doctor)](#7-audit-verify-walk-doctor)
8. [Fine-tune (encryption, sync class, classification)](#8-fine-tune-encryption-sync-class-classification)
9. [Deploy strategy](#9-deploy-strategy)
10. [Layout](#10-layout)
11. [Place in CyberOS architecture](#11-place-in-cyberos-architecture)
12. [Appendix A — Autosync design](#appendix-a--autosync-design)
13. [Appendix B — Evolution & open questions](#appendix-b--evolution--open-questions)
14. [Appendix C — Layer-2 source-of-truth](#appendix-c--layer-2-source-of-truth)
15. [Appendix D — Proposals (shipped & open)](#appendix-d--proposals-shipped--open)
16. [Cross-references](#cross-references)

---

## 1. What it does

The memory is an append-only, content-addressed, hash-chained personal memory store. It records every meaningful interaction between you and your AI agents so that:

- **Tomorrow's session knows what today's session learned.** Memory files persist across conversations, projects, and machines.
- **Every change is auditable.** A binary framed audit ledger with SHA-256 chain-sealing (Phase 1) or MMR + Ed25519-signed tree heads (Phase 2 P2) makes tampering detectable.
- **Privacy classes are enforced.** Memory files declare `meta.classification` and `meta.sync_class`; the writer rejects exports of `private` content; encryption envelopes guard `restricted` content at rest.
- **Cross-agent operation works.** The protocol is text-based and platform-agnostic: any agent that can read `AGENTS.md` (or the smaller `INTEROP.md`) can read your memory. The reference implementation lives here, but the protocol does not depend on it.

The module is the source of truth for the Layer-1 protocol. Other CyberOS modules (`skill`, `cuo`) MUST route all memory writes through `cyberos.core.writer.Writer` rather than touching `audit/`, `HEAD`, or `.lock` directly.

---

## 2. Status

| Phase | Status |
|---|---|
| Core writer + reader + walker | shipped |
| MMR + Signed Tree Heads (P2 Stage 3) | shipped (opt-in) |
| Crypto-mode (STH-only) | shipped (opt-in) |
| Cross-platform automation (launchd / systemd / Task Scheduler) | shipped |
| Semantic search | shipped (optional `sentence-transformers` dep) |
| Sync conflict awareness (iCloud / Dropbox / OneDrive) | shipped |
| All 12 audit proposals (P1–P12 + P2 Stage 3) | shipped |
| Cross-memory import (P6) | shipped |
| HTTP REST (`cyberos serve`) | shipped |
| Daily digest (`cyberos digest`) | shipped |
| Mobile publish (`cyberos publish`) | shipped |
| iOS companion app | pending (future) |
| Public anchoring of STH (transparency log) | pending (future) |

---

## 3. Quick start

```bash
# From repo root: install the module + the cyberos CLI in editable mode
cd modules/memory
pip install -e .

# Verify the local memory passes every invariant
cyberos --store ../../.cyberos-memory doctor

# Append a memory
echo "Stephen prefers brief responses" \
  | cyberos --store ../../.cyberos-memory --actor stephen put memories/preferences/communication.md -

# Inspect chain state
cyberos --store ../../.cyberos-memory state
```

Without `pip install -e .`, the package is runnable as `python -m cyberos` from this directory (or set `PYTHONPATH` to include it).

---

## 4. Run locally (full install)

### Prerequisites

- Python ≥ 3.10
- Required pip dependencies (`msgspec`, `cryptography`, `crc32c`, `rfc8785`, `pyyaml`, `jsonschema`, `zstandard`)
- (Optional) `sentence-transformers` for semantic search
- (Optional) `pandoc` if you'll convert PRD/SRS docx ↔ md

### Install

```bash
cd modules/memory
pip install -e .[test]    # adds pytest, pytest-cov

# Verify
cyberos --version
cyberos doctor --help
pytest tests/ -v          # → 255 passed
```

### CLI surface

```
cyberos put <path> <body-source>            # append a memory (body-source: - for stdin, or file path)
cyberos move <src> <dst>                    # rename within the store
cyberos delete <path> [--mode tombstone|purge] [--reason ...]
cyberos read <path>                         # render a memory file (frontmatter + body)
cyberos search <query> [--semantic]         # keyword or semantic search
cyberos state                               # show HEAD seq + chain tip
cyberos doctor [--repair]                   # walk + verify invariants
cyberos consolidate                         # Walk → Compact → Sign → Publish
cyberos export <out.zip>                    # deterministic exportable bundle
cyberos import <source.zip> [--filter ...]  # cross-memory merge (P6)
cyberos serve [--port 8088]                 # HTTP REST API
cyberos digest [--since 24h]                # daily digest
cyberos publish [--target mobile]           # mobile-ready bundle
```

### Background automation

Cross-platform scheduler integration is shipped:

```bash
# macOS (launchd)
scripts/install.sh . --with-automation

# Linux (systemd-user; cron fallback)
scripts/install.sh . --with-automation

# Windows (Task Scheduler)
powershell -ExecutionPolicy Bypass -File scripts/install.ps1 -Target . -WithAutomation
```

Nightly + weekly maintenance: walks the memory, runs `consolidate` if size/row thresholds exceeded, archives sealed monthly segments to `.binlog.zst`, refreshes the SQLite derived index.

---

## 5. The four supported workflows

| # | Workflow | Status |
|---|---|---|
| 1 | **Solo, single machine** — one person, one laptop; agent auto-builds memory | shipped |
| 2 | **Solo, multi-machine** — copy `.cyberos-memory/` between your own machines | shipped (deterministic export) |
| 3 | **Multi-person, independent memories** — each teammate has their own | shipped |
| 4 | **Multi-person, merged** — pull selected memories from a teammate's memory | shipped (v2.1 — `cyberos import`) |

---

## 6. Step-by-step: starting from zero

These eight steps take a fresh project from nothing to "my agent remembers everything I'm doing, automatically."

### Step 1 — Install the dependencies (one-time, per machine)

```bash
pip install msgspec cryptography crc32c rfc8785 pyyaml jsonschema zstandard
```

On Python ≥ 3.11 you may need `--break-system-packages` on Linux / macOS to bypass PEP 668.

### Step 2 — Copy the protocol files into your project

```bash
cd ~/Projects/my-new-project

mkdir -p docs/memory
cp ~/Projects/CyberSkill/cyberos/modules/memory/AGENTS.md             docs/memory/
cp ~/Projects/CyberSkill/cyberos/modules/memory/INTEROP.md            docs/memory/
cp ~/Projects/CyberSkill/cyberos/modules/memory/memory.schema.json    docs/memory/
cp ~/Projects/CyberSkill/cyberos/modules/memory/memory.invariants.yaml docs/memory/
```

All four normative files (~30 KB total). The rest of `modules/memory/` (this README, appendices, CHANGELOG) is informative — do not copy unless wanted.

You also need the `cyberos` Python package:

```bash
cp -r ~/Projects/CyberSkill/cyberos/modules/memory/cyberos ./cyberos
```

### Step 3 — Initialise the memory

```bash
mkdir -p .cyberos-memory/{audit,memories/{decisions,facts,people,projects,preferences,drift,refinements},meta,company,module,member,client,project,persona,conflicts,exports,index}

cat > .cyberos-memory/manifest.json <<EOF
{
  "store_version": "2.0.0",
  "agent_protocol": "AGENTS.md@2.0.0",
  "created_at_utc": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "fingerprint": "$(uuidgen | tr A-Z a-z)",
  "audit_chain_head": null,
  "imports": {}
}
EOF

printf '\x00\x00\x00\x00\x00\x00\x00\x00' > .cyberos-memory/HEAD
```

### Step 4 — Wire your agent

Symlink the protocol so your agent picks it up:

```bash
ln -sf docs/memory/AGENTS.md AGENTS.md
ln -sf docs/memory/AGENTS.md CLAUDE.md   # or .cursorrules, copilot-instructions.md
```

### Step 5 — First memory

```bash
python -m cyberos --store .cyberos-memory --actor your-name put \
  memories/facts/about-this-project.md - <<EOF
---
name: about-this-project
description: One-liner about what this project is and who's working on it.
type: project
---

This project does X. It's owned by Y. Today is the first day.
EOF
```

`HEAD` advances from `00…` to `01…`; the first audit row is sealed.

### Step 6 — Doctor

```bash
python -m cyberos --store .cyberos-memory doctor
# → READY  ✓ 15/15 invariants
```

### Step 7 — Automation (optional but recommended)

```bash
scripts/install.sh . --with-automation --with-pre-commit
```

Six-phase install (protocol files, memory init, agent wiring, scheduler registration, git pre-commit hook, verify).

### Step 8 — Use it

From now on every meaningful conversation turn that adds a fact, preference, or decision your agent learns SHOULD be put into the memory. Read the agent-side rules in [`AGENTS.md`](AGENTS.md) §1 (the pre-write checklist) and your project's `CLAUDE.md` auto-memory section for the exact when/what/how.

---

## 7. Audit (verify, walk, doctor)

```bash
# Quick health check (15 invariants)
cyberos doctor

# Walk the entire ledger + verify chain consistency
cyberos walk --verify-chain

# Repair recoverable invariants
cyberos doctor --repair

# Recovery from FROZEN_HUMAN state (catastrophic divergence)
cyberos doctor --repair --reason "manifest unparseable after disk corruption"
```

### Forbidden operations (per AGENTS.md §6.5)

Never touch these directly:

- In-place edit of a written ledger row
- Re-ordering of rows
- Deletion of rows (use `delete` with `mode=tombstone` instead)
- Rewriting the binlog tail past the last intact frame

Recovery from chain corruption is via consolidation (`cyberos consolidate`), not row mutation.

---

## 8. Fine-tune (encryption, sync class, classification)

Memory files declare protection level in their frontmatter / sidecar:

```yaml
---
name: client-roadmap-q4
description: "Acme Co's confidential Q4 product roadmap."
type: client
sync_class: shareable           # or 'private' (default)
classification: confidential    # public | internal | confidential | restricted
cipher: aes-256-gcm             # set to apply encryption envelope (§5.4)
acl: ["stephen@cyberskill.world"]   # optional explicit allow-list
---

(body — encrypted at rest if cipher != null)
```

- `sync_class: private` (default) — never leaves the local store
- `sync_class: shareable` — MAY be exported via `cyberos export` deterministic zip
- `classification: restricted` — encryption envelope REQUIRED
- `classification: confidential` — encryption envelope RECOMMENDED
- `acl: [...]` — explicit allow-list of actor IDs

### v1 compatibility

The v1 four-tier sync_class (`local-only / publishable / shared / client-visible`) is preserved in `meta.sync_class_v1` for one release cycle for tooling that has not migrated.

### Cross-memory merge (v2.1)

```bash
# Import shareable memories from a teammate's exported memory
cyberos import teammate-export.zip --filter "sync_class=shareable"
# Imported rows become fresh `put` rows on the local chain with
#   extra.imported_from = <source fingerprint>
#   extra.foreign_chain = <source row's chain hash>
# Idempotent re-import via manifest.imports.<fingerprint>.last_imported_seq
```

---

## 9. Deploy strategy

> **Production deploy:** the canonical runbook for taking MEMORY to AWS Fargate lives in the **root README §3 — MEMORY deploy** ([`../../README.md`](../../README.md#3--memory-deploy)). It covers prerequisites (Postgres 16 + pgvector + AGE + Redis 7), the Rust memory service build, migration sequence, JWK bootstrap, smoke tests, ECS deploy, rollback, observability dashboards, and secret management. Sections 9.1-9.5 below cover local/development scenarios — they are NOT the production path.

### 9.1 Local-dev (every project)

Symlink `AGENTS.md` + `CLAUDE.md` from project root → `modules/memory/AGENTS.md` (or copy if your filesystem doesn't support symlinks). memory lives at project-root `.cyberos-memory/`. Maintenance is `cyberos doctor` (manual) or automation (background).

### 9.2 Multi-machine (single user)

`cyberos export` produces a byte-identical zip across runs/platforms. Copy between machines via iCloud / Dropbox / OneDrive — the writer is sync-conflict-aware. Conflict files (`.lock conflict`, `*.icloud`) are surfaced by `cyberos doctor`.

### 9.3 Multi-tenant (per organisation)

One `.cyberos-memory/` per organisation, each with its own `manifest.json` fingerprint. Cross-memory merge via `cyberos import` with explicit `acl` / `sync_class` filters. The protocol prohibits merging foreign chains directly — every import row is a fresh `put` on the local chain.

### 9.4 HTTP REST (`cyberos serve`)

```bash
cyberos serve --port 8088 --bind 127.0.0.1
# GET    /api/v2/memories?type=facts&since=24h
# POST   /api/v2/memories
# DELETE /api/v2/memories/<path>?mode=tombstone
# GET    /api/v2/audit/chain
# GET    /api/v2/audit/sth         (latest signed tree head)
```

### 9.5 Mobile publish

```bash
cyberos publish --target mobile --out ~/Desktop/memory.mobi.zip
# → manifest + filtered memories (sync_class=shareable) + STH; loads in the
#   forthcoming iOS companion app (pending)
```

### Key operational invariants (all deployments)

1. **The memory is the source of truth.** Even with the SQLite derived index, when index ↔ filesystem disagree, the filesystem wins.
2. **Path security.** Every path argument MUST be relative, MUST resolve strictly inside `<memory-root>/`, MUST contain no `..` after normalisation.
3. **Atomic write — two-phase + parent-dir sync.** macOS uses `fcntl(F_BARRIERFSYNC)` per-batch and `fcntl(F_FULLFSYNC)` per-checkpoint. Plain `fsync()` is insufficient on Darwin.
4. **Lock leases TTL = 10s, renew = 3s.** Stale leases reaped via monotonic time comparison.
5. **Forbidden in production:** writing directly to `audit/`, `HEAD`, or `.lock`; re-ordering rows; deleting rows.

---

## 10. Layout

```
modules/memory/
├── README.md                  ← THIS FILE (single source of truth)
├── AGENTS.md                  ← Layer-1 normative spec (symlinked from repo root)
├── INTEROP.md                 ← non-ledger consumer subset
├── CHANGELOG.md               ← release history
├── memory.schema.json         ← machine schema for memory files
├── memory.invariants.yaml     ← walker input invariants
├── pyproject.toml             ← Python package + cyberos console script
├── requirements.txt           ← re-export of cyberos/requirements.txt
├── cyberos/                   ← Python package (core/, __main__.py, …)
│   ├── __init__.py
│   ├── __main__.py
│   ├── core/
│   │   ├── writer.py          ← canonical Writer class (every memory write goes through here)
│   │   ├── reader.py
│   │   ├── walker.py
│   │   ├── doctor.py
│   │   ├── consolidator.py
│   │   ├── importer.py
│   │   ├── exporter.py
│   │   ├── mmr.py
│   │   └── …
│   └── requirements.txt
├── tools/                     ← schema generator, voice linter, encrypt, benchmark
├── tests/                     ← pytest suite (255 green)
├── bench/                     ← throughput / cold-CLI / determinism benchmarks
└── scripts/                   ← install.sh / install.ps1, automation, pre-commit, helpers
```

---

## 11. Place in CyberOS architecture

| Module | Role | Lives at |
|---|---|---|
| **`memory/`** | **This module.** The memory — append-only audit-chained personal memory store | `.cyberos-memory/` per project |
| [`skill/`](../skill/) | Catalog of agentic Skills + Rust host + Bun toolchain | `modules/skill/<name>/` flat layout |
| [`cuo/`](../cuo/) | Persona-aware orchestration layer above SKILL | `modules/cuo/<persona-slug>/` flat layout |

This module interacts with:

- [`skill/`](../skill/) — skill bundles can declare `allowed_memory_scopes` (read/write) against the memory; the host enforces them via the capability broker
- [`cuo/`](../cuo/) — every CUO routing decision + workflow invocation lands in the memory audit chain via `cyberos.core.writer.Writer` (per [`AGENTS.md`](AGENTS.md) §6, §11). The CUO MUST NOT write directly to `audit/`, `HEAD`, or `.lock`. Wired in Phase 3 via `cuo/cuo/core/memory_bridge.py`

For the full interactive picture see [`../../website/docs/index.html`](../../website/docs/index.html).

---

## Appendix A — Autosync design

> Full design rationale for background sync, conflict detection, and cross-platform scheduler integration. (Was `docs/AUTOSYNC_DESIGN.md`.)

The autosync subsystem reconciles three concerns:

1. **Sync conflict awareness.** iCloud / Dropbox / OneDrive may produce `.<filename>.icloud` placeholders, `.<filename> (conflicted copy …)` files, or out-of-band lock-file mutations. The writer detects these on every operation and surfaces them via `cyberos doctor`.
2. **Background maintenance schedule.** Cross-platform scheduler integration (launchd / systemd-user / Task Scheduler) runs nightly walk + weekly consolidate.
3. **Determinism guarantee.** `cyberos export` produces byte-identical output across runs and platforms (sorted paths, fixed timestamp `2000-01-01T00:00:00Z`, fixed mode `0o644`, ZIP_DEFLATED level 6).

### Detection patterns (per OS)

| OS | Sync provider | Conflict marker |
|---|---|---|
| macOS | iCloud Drive | `.<filename>.icloud` (placeholder) or `<filename> (<conflict-date>)` |
| macOS / Linux / Windows | Dropbox | `<filename> (Stephen's conflicted copy 2026-05-17).md` |
| macOS / Linux / Windows | OneDrive | `<filename>-DESKTOP-XXXXXXX.md` |
| Windows | OneDrive | Locked-file errors on `.lock` acquisition |

The writer's `_detect_sync_conflicts()` check runs before any mutation; conflicts trigger a transition to `FROZEN_RECOVERABLE` per AGENTS.md §12.

### Background job — nightly walk

```bash
# Installed by scripts/install.sh --with-automation
*/30 * * * *  cyberos --store ~/Projects/<proj>/.cyberos-memory doctor --quiet
0    2 * * *  cyberos --store ~/Projects/<proj>/.cyberos-memory walk --verify-chain --since-ledger-tail
0    3 * * 0  cyberos --store ~/Projects/<proj>/.cyberos-memory consolidate
```

### Determinism for export

- Files sorted by path (UTF-8 NFC normalised)
- All `mtime` fields written as `2000-01-01T00:00:00Z`
- All file modes written as `0o644`
- ZIP_DEFLATED compression level 6
- Excluded: `exports/`, `__pycache__/`, `.cache/`, `.lock`, `HEAD`, `index/`

This guarantees `sha256(export.zip)` is stable across machines.

---

## Appendix B — Evolution & open questions

> History of how the protocol got here + the open questions tracker. (Was `docs/EVOLUTION.md`.)

### Stage 1 — Plain markdown files (2025 Q4)

`MEMORY/<topic>.md` files manually appended by the user. No chain, no schema, no audit. Worked for one user but immediately fragmented when used cross-agent.

### Stage 2 — JSON Lines ledger (2026 Q1)

`audit/*.jsonl` rows + `manifest.json` + memory frontmatter schema. Chain via `prev` field pointing at prior row's hash. Worked but JSON canonicalisation was fragile (RFC 8785 not enforced).

### Stage 3 — Binary framed binlog + msgspec canonical JSON (2026 Q2)

Current Phase 1 baseline. `*.binlog` with `[u32 length BE][u32 crc32c BE][u64 seq BE][u64 ts_ns BE][payload]`. Payload is RFC 8785 JCS canonical JSON. Chain via `chain = SHA-256(canonical(record_minus_chain) || prev_chain)`.

### Stage 4 — MMR + Signed Tree Heads (2026 Q2 — Proposal P2)

Merkle Mountain Range over canonical-JSON leaves. Ed25519-signed tree heads per consolidation. Activation requires resolution of EVOLUTION §4 Q1–Q3 (key custody, public anchoring, recovery semantics).

### Stage 5 — Cross-memory merge (2026 Q2 — P6 shipped)

`cyberos import <source.zip>` adds imported memories as fresh local-chain rows with `extra.imported_from` + `extra.foreign_chain` provenance.

### Stage 6 — HTTP REST + mobile publish (2026 Q2 — shipped)

`cyberos serve` + `cyberos publish --target mobile`.

### Open questions (§4)

1. **Q1 — Key custody for STH signing.** OS keychain (Keychain / KWallet / DPAPI) vs filesystem-encrypted under user passphrase vs hardware token? Currently filesystem-encrypted; key rotation procedure undocumented.
2. **Q2 — Public anchoring of STH.** Push to a Sigstore-style transparency log? Pin to git? Currently no public anchoring; STH is purely local.
3. **Q3 — Recovery semantics if a STH is lost/corrupted.** Re-derive from raw rows (requires full walk) vs trust the next STH (requires gap acknowledgement). Currently re-derive; expensive for large memories.

---

## Appendix C — Layer-2 source-of-truth

> Layer-2 design: how the memory coexists with external systems of record (calendar, CRM, email, Notion, etc.) without becoming the source of truth for things it isn't authoritative for. (Was `docs/LAYER_2_SOURCE_OF_TRUTH.md`.)

### Principle

The memory is **Layer-1: my personal AI-assist scratchpad**. It is NOT:

- The source of truth for billing (that's your accounting system)
- The source of truth for code (that's git)
- The source of truth for calendar events (that's Google Calendar / Outlook)
- The source of truth for support tickets (that's Zendesk / Intercom)

The memory's purpose is to **persist the agent's understanding** of those external sources, plus your own preferences / decisions / drift / refinements that have no external system of record.

### Reference memory pattern

When the memory needs to capture knowledge about an external resource, write a `reference` memory:

```yaml
---
name: linear-project-ingest
description: Pipeline bugs tracked in Linear project INGEST
type: reference
external_system: linear
external_id: INGEST
last_synced_at: 2026-05-18T10:00:00Z
---

Pipeline bugs are tracked in Linear project "INGEST". Filter `state:open` for current work.
Authoritative source. Do NOT re-summarize pipeline state in memory — fetch from Linear.
```

### Anti-pattern (DO NOT)

```yaml
# Don't do this — Linear is authoritative for ticket state, not memory
---
name: open-bugs-2026-05-18
type: facts
---
Open bugs:
- INGEST-101: foo
- INGEST-102: bar
- INGEST-103: baz
```

The list goes stale the moment you write it. Write the `reference` memory instead, and let the agent fetch live state when needed.

---

## Appendix D — Proposals (shipped & open)

> The proposal tracker. (Was `docs/PROPOSAL.md`.)

### Shipped proposals (P1–P12, P2 Stage 3)

| ID | Title | Status |
|---|---|---|
| P1 | Binary framed binlog with crc32c per-row | shipped 2026 Q1 |
| P2 | MMR + Signed Tree Heads (Stage 1 — local-only) | shipped 2026 Q1 |
| P2 Stage 2 | MMR + STH (cross-machine sync) | shipped 2026 Q2 |
| P2 Stage 3 | MMR + STH (public anchoring opt-in) | shipped (opt-in) |
| P3 | Encryption envelope for `restricted` classification | shipped 2026 Q1 |
| P4 | Soft tombstone vs hard purge distinction | shipped 2026 Q1 |
| P5 | Conflict-aware writer | shipped 2026 Q1 |
| P6 | Cross-memory merge via `cyberos import` | shipped 2026 Q2 |
| P7 | Doctor with `--repair` flag | shipped 2026 Q1 |
| P8 | Deterministic export | shipped 2026 Q1 |
| P9 | Semantic search (`--semantic` flag) | shipped 2026 Q2 |
| P10 | HTTP REST API | shipped 2026 Q2 |
| P11 | Daily digest | shipped 2026 Q2 |
| P12 | Mobile publish | shipped 2026 Q2 |

### Open proposals (P13+)

| ID | Title | Status |
|---|---|---|
| P13 | iOS companion app for `cyberos publish` bundles | designed; not implemented |
| P14 | Public anchoring of STH via Sigstore transparency log | designed; awaiting Q2 resolution |
| P15 | Multi-tenant `cyberos serve` (per-organisation STH chains) | designed; awaiting key-custody decision |
| P16 | Differential `cyberos import` (only-new rows since last import seq) | partially shipped; needs CLI polish |
| P17 | Real-time `cyberos watch` for memory observability | designed; not implemented |
| P18 | OpenAPI 3.1 spec for `cyberos serve` | open |
| P19 | Dreaming — `cyberos dream` out-of-band batch reflection (AGENTS.md §7.7) | **APPROVED 2026-05-19** · §7.7 amendment merged · implementation lives in [`FR-MEMORY-115`](../../docs/feature-requests/memory/FR-MEMORY-115-cyberos-dream.md) |
| P20 | Per-store ACL via `STORE.yaml` (AGENTS.md §14.4) | **APPROVED 2026-05-19** · §14.4 amendment merged · implementation lives in [`FR-MEMORY-117`](../../docs/feature-requests/memory/FR-MEMORY-117-per-store-acl.md) |
| P21 | `put_if` precondition-hash op (AGENTS.md §3.1 extension) | **APPROVED 2026-05-19** · §3.1 extended (added 4th canonical op + §3.1.5/.6/.7) · implementation lives in [`FR-MEMORY-118`](../../docs/feature-requests/memory/FR-MEMORY-118-put-if-precondition.md) |
| P22 | Session transcript ledger (AGENTS.md §18) | **APPROVED 2026-05-19** · §18 amendment merged · implementation lives in [`FR-MEMORY-119`](../../docs/feature-requests/memory/FR-MEMORY-119-session-transcript-ledger.md). Namespaced under `cyberos transcript` (the FR's `cyberos session` collides with existing P11 coordination subcommand) |

Proposals follow the §16 self-amendment grammar: `propose-now` requires `APPROVE protocol change P<n> §<section>` in the active chat; `log-deferred` appends to EVOLUTION.md §4 with a date stamp.

P19–P22 are the four protocol amendments introduced by the MEMORY Improvement Wave 2026 Q3 (FR-MEMORY-115 / 117 / 118 / 119). Per Stephen's 2026-05-19 decision, each is approved independently rather than bundled. The FR sources for the design rationale are the individual `FR-MEMORY-11{5,7,8,9}.md` spec files plus their `.audit.md` siblings.

---

## Cross-references

- [`AGENTS.md`](AGENTS.md) — Layer-1 normative spec (symlinked from repo root)
- [`INTEROP.md`](INTEROP.md) — non-ledger consumer subset
- [`CHANGELOG.md`](CHANGELOG.md) — release history (2,202 lines)
- [`memory.schema.json`](memory.schema.json) — machine schema
- [`memory.invariants.yaml`](memory.invariants.yaml) — walker input
- [`cyberos/`](cyberos/) — Python implementation
- [`tests/`](tests/) — 255 green tests
- [`../skill/README.md`](../skill/README.md) — sibling SKILL module
- [`../cuo/README.md`](../cuo/README.md) — sibling CUO module
- [`../../docs/Software Development Process.md`](../../docs/Software%20Development%20Process.md)
- [`../../tours/`](../../tours/) — operational CodeTour walkthroughs for memory audit-chain repair, frontmatter fixes, conflict resolution
- [`../../website/docs/index.html`](../../website/docs/index.html) — interactive multi-layer architecture diagram
