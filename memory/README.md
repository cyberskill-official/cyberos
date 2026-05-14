# memory — CyberOS memory module

Local-first, audit-chained personal memory store. Append-only ledger, six file ops, MMR inclusion proofs, signed tree heads, deterministic export. Every read and write that touches the BRAIN goes through this module.

## Status

| Phase | Status |
|---|---|
| Core writer + reader + walker | shipped |
| MMR + STH | shipped |
| Crypto-mode (STH-only) | shipped (opt-in) |
| Cross-platform automation (launchd / systemd / Task Scheduler) | shipped |
| Semantic search | shipped (optional `sentence-transformers` dep) |
| Sync conflict awareness (iCloud / Dropbox / OneDrive / etc.) | shipped |
| All 12 audit proposals (P1-P12 + P2 Stage 3) | shipped |
| Cross-BRAIN import (P6) | shipped |
| HTTP REST (`cyberos serve`) | shipped |
| Daily digest (`cyberos digest`) | shipped |
| Mobile publish (`cyberos publish`) | shipped |
| iOS companion app | pending — future |
| Public anchoring of STH (transparency log) | pending — future |

Test suite: 255 green. `cyberos doctor` passes 15/15 invariants on the live BRAIN.

## Quick start

```bash
# From repo root: install module + the cyberos CLI in editable mode.
cd memory
pip install -e .

# Verify a store passes every invariant.
cyberos --store ../.cyberos-memory doctor

# Append a memory.
cyberos --store ../.cyberos-memory --actor stephen put memories/facts/example.md -

# Inspect chain state.
cyberos --store ../.cyberos-memory state
```

Without `pip install -e .`, the package is still runnable as
`python -m cyberos` provided you run it from this directory (or set
`PYTHONPATH` to include it).

## Layout

```
memory/
├── README.md              ← you are here
├── pyproject.toml         ← registers the `cyberos` console script
├── requirements.txt       ← re-export of cyberos/requirements.txt
├── cyberos/               ← Python package (core/, __main__.py, requirements.txt)
├── docs/                  ← AGENTS.md (protocol RFC), schema, invariants, CHANGELOG
├── tools/                 ← schema generator, voice linter, encrypt, benchmark
├── tests/                 ← pytest suite (regression tests for core + CLI)
├── bench/                 ← throughput / cold-CLI / determinism benchmarks
└── scripts/               ← install.sh, automation (launchd / systemd / Task Scheduler),
                              pre-commit hook, unwrap-md helper
```

## Place in the CyberOS architecture

CyberOS has three modules today:

| Module | Role | Lives at |
|---|---|---|
| `memory/` | The BRAIN — append-only audit-chained personal memory store | `~/.cyberos-memory/` per project |
| `skill/` | Catalog of agentic Skills + Rust host + Bun toolchain | `skill/skills/` + Rust crates |
| `cuo/` | Router — natural-language → skill chain → memory record | Python package |

This module is **memory**. It interacts with:
- `skill/` — skill bundles can declare `allowed_brain_scopes` (read/write) against the BRAIN; the host enforces them via the capability broker.
- `cuo/` — every routing decision the router makes is appended to the BRAIN's audit chain as a memory record (today via flat-file bridge; Phase-2 will go through the canonical `Writer`).

For the full picture see `../website/docs/index.html` (interactive multi-layer architecture doc, 31 pages).

## Where to read next

* [`docs/AGENTS.md`](docs/AGENTS.md) — the full RFC: data model, audit chain, ops, source tiers, sync classes.
* [`docs/CHANGELOG.md`](docs/CHANGELOG.md) — release history.
* [`docs/PROPOSAL.md`](docs/PROPOSAL.md) — open / shipped design decisions.
* [`cyberos/README.md`](cyberos/README.md) — implementation-level notes (writer, walker, lock).
