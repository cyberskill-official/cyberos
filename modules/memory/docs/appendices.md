---
title: Memory - Appendices & Extended Reference
source: website/docs/modules/memory/appendices.html
migrated: FR-DOCS-002
---

# MEMORY appendices - detailed reference

> Extended design rationale and proposal tracker. For the concise module overview, see [README.md](README.md).

## Appendix A - autosync design

The autosync subsystem reconciles three concerns:

1. **Sync conflict awareness.** iCloud / Dropbox / OneDrive may produce conflict markers. The writer detects these on every operation and surfaces them via `cyberos doctor`.
2. **Background maintenance.** Cross-platform scheduler integration (launchd / systemd-user / Task Scheduler) runs a nightly walk plus a weekly consolidate.
3. **Determinism.** `cyberos export` produces byte-identical output across runs and platforms.

### Detection patterns

| OS | Sync provider | Conflict marker |
|---|---|---|
| macOS | iCloud Drive | `.<filename>.icloud` or `<filename> (<conflict-date>)` |
| macOS / Linux / Windows | Dropbox | `<filename> (Stephen's conflicted copy 2026-05-17).md` |
| macOS / Linux / Windows | OneDrive | `<filename>-DESKTOP-XXXXXXX.md` |
| Windows | OneDrive | Locked-file errors on `.lock` acquisition |

### Background jobs

```
# Installed by scripts/install.sh --with-automation
*/30 * * * *  cyberos --store .cyberos/memory/store doctor --quiet
0    2 * * *  cyberos --store .cyberos/memory/store walk --verify-chain --since-ledger-tail
0    3 * * 0  cyberos --store .cyberos/memory/store consolidate
```

### Export determinism

- Files sorted by path (UTF-8 NFC)
- All `mtime` = `2000-01-01T00:00:00Z`
- All file modes = `0o644`
- ZIP_DEFLATED level 6
- Excluded: `exports/`, `__pycache__/`, `.cache/`, `.lock`, `HEAD`, `index/`

## Appendix B - evolution

Design rationale for the current store format, by stage.

### Stage 1 - plain markdown (2025 Q4)

`MEMORY/<topic>.md` files manually appended. No chain, no schema, no audit.

### Stage 2 - JSON Lines ledger (2026 Q1)

`audit/*.jsonl` rows + `manifest.json` + frontmatter schema. Chain via `prev` field. Worked, but JSON canonicalisation was fragile.

### Stage 3 - binary framed binlog + msgspec (2026 Q2)

Current Phase 1 baseline. `*.binlog` with `[u32 length BE][u32 crc32c BE][u64 seq BE][u64 ts_ns BE][payload]`. Payload is RFC 8785 JCS canonical JSON. Chain via `chain = SHA-256(canonical(record_minus_chain) || prev_chain)`.

### Stage 4 - MMR + Signed Tree Heads (2026 Q2)

Merkle Mountain Range over canonical-JSON leaves. Ed25519-signed tree heads per consolidation. Activation pending.

### Stage 5 - cross-memory merge (2026 Q2)

`cyberos import <source.zip>` adds imported memories as fresh local-chain rows with provenance.

### Stage 6 - HTTP REST + mobile publish (2026 Q2)

`cyberos serve` + `cyberos publish --target mobile`.

### Open questions

1. **Q1 - key custody for STH signing.** OS keychain vs filesystem-encrypted vs hardware token? Currently filesystem-encrypted.
2. **Q2 - public anchoring of STH.** Push to a transparency log? Pin to git? Currently local-only.
3. **Q3 - recovery if a STH is lost or corrupted.** Re-derive from raw rows (expensive) vs trust the next STH (requires gap acknowledgement).

## Appendix C - layer-2 source-of-truth

The memory is **Layer-1: personal AI-assist scratchpad**. It is NOT the source of truth for billing (accounting), code (git), calendar (Google/Outlook), or tickets (Zendesk/Linear). Its purpose is to persist the agent's understanding of those external sources.

### Reference memory pattern

```
---
name: linear-project-ingest
description: Pipeline bugs tracked in Linear project INGEST
type: reference
external_system: linear
external_id: INGEST
last_synced_at: 2026-05-18T10:00:00Z
---

Pipeline bugs are tracked in Linear project "INGEST". Filter `state:open` for current work.
Authoritative source. Do NOT re-summarize pipeline state in memory.
```

## Appendix D - proposals

### Shipped

| ID | Title |
|---|---|
| P1 | Binary framed binlog with crc32c per-row |
| P2 | MMR + Signed Tree Heads (Stage 1-3) |
| P3 | Encryption envelope for `restricted` classification |
| P4 | Soft tombstone vs hard purge |
| P5 | Conflict-aware writer |
| P6 | Cross-memory merge via `cyberos import` |
| P7 | Doctor with `--repair` flag |
| P8 | Deterministic export |
| P9 | Semantic search |
| P10 | HTTP REST API |
| P11 | Daily digest |
| P12 | Mobile publish |
| P13-P18 | iOS app, public anchoring, multi-tenant, differential import, watch, OpenAPI |
| P19 | Dreaming - `cyberos dream` (AGENTS.md §7.7) |
| P20 | Per-store ACL via `STORE.yaml` (AGENTS.md §14.4) |
| P21 | `put_if` precondition-hash op (AGENTS.md §3.1) |
| P22 | Session transcript ledger (AGENTS.md §18) |

Proposals follow §16 self-amendment: `propose-now` requires `APPROVE protocol change P<n> §<section>` in the active chat; `log-deferred` records for future consideration.

## Changelog

History lives in the [changelog](./changelog.html); this page describes only the current state.
