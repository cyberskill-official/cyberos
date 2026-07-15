---
title: memory - the CyberOS BRAIN · CyberOS
migrated: TASK-DOCS-002
---

memory is the platform's remembering faculty - BRAIN when you speak about it, `memory` in code. It is two layers with one truth: an append-only, audit-chained Layer-1 store on local disk, and a Layer-2 service that derives searchable structure from it.

## Layer 1 - the protocol store

The normative protocol is `AGENTS.md` (vendored to every initialised repo). Key facts:

- The store lives at `.cyberos/memory/store/` at the project root (§0.4) - under the same gitignored `.cyberos/` tree as the rest of the vendored machine. Resolution: explicit `--store`/`CYBEROS_STORE` override, then the nearest store walking up from the working directory.
- Every mutation is one of the canonical ops (`put`, `move`, `delete`, `put_if`); rows chain by hash into monthly binlog segments with an MMR cross-check, so any byte can be replayed or proved.
- `python -m cyberos doctor` walks the invariant set and reports store health; `cyberos export` produces a deterministic, portable zip.
- Stores are tenant data: always gitignored, never committed.

## Layer 2 - the brain service

`services/memory` (Rust) reads Layer 1 and derives: per-event embeddings (pgvector), rolling summaries, and hot/warm/cold tiering keyed on each interaction's occurred-at time. Recall answers over the hot index with provenance back to the exact Layer-1 rows; access is consent-gated and tenant-scoped (fail-closed RLS). The DB-backed suite runs via `scripts/local_verify.sh`.

## Day-to-day

- A fresh store is scaffolded by `cyberos-init` in every project (skip with `CYBEROS_NO_MEMORY=1`).
- The desktop app captures quick notes into the store and supervises sync.
- Agents working in a repo record decisions, audits, and plans into the BRAIN per the protocol.

## Changelog

History (protocol amendments, the store relocation, Layer-2 hardening) lives in the [changelog](./changelog.html); this page describes only the current state.
