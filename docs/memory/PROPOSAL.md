# PROPOSAL.md — Deep Optimization Audit changes (shipped state)

**Status:** Most proposals are shipped. P2 Stage 3 is the only outstanding item.

This file used to stage the five W1/W2 changes for §0.2 chat-turn approval.
After the May 2026 rebuild session the user approved everything in a single
chat-turn waiver ("i approve all since this is the rebuild"), and the
proposals shipped sequentially.

The Deep Audit text itself remains the canonical reference. EVOLUTION.md §3.2
records the audit's headline findings and which session shipped each one.

---

## Shipped status

| Proposal | Status | Date | Notes |
|---|---|---|---|
| **P1** — six → three canonical ops (`put`, `move`, `delete(mode)`) | **shipped** | 2026-05-13 | V1 aliases preserved for one release cycle; canonical names emit `op="put"` / `op="move"` in audit rows |
| **P2 Stage 1** — additive MMR + Ed25519 Signed Tree Heads | **shipped** | 2026-05-14 | MMR runs alongside the chain; doctor cross-check catches divergence; chain remains source of truth |
| **P2 Stage 2** — passphrase-wrapped signing key (scrypt + ChaCha20-Poly1305) | **shipped** | 2026-05-14 | Backward-compatible with stage-1 raw keys; `cyberos sth-wrap` for in-place migration |
| **P2 Stage 3** — chain primitive swap (MMR + STH become the only integrity primitive) | **DEFERRED** | n/a | Gated on 2-week soak under Stage 1; requires fresh chat-turn approval |
| **P3** — sidecar `*.meta.json` migration tool + parser support | **shipped** | 2026-05-13 | `cyberos_migrate_sidecar.py` idempotent + reversible; reader auto-detects format |
| **P4** — GDPR Article 17 `delete(path, "purge")` mode | **shipped** | 2026-05-13 | Magic-phrase gate; body bytes redacted with visible marker; audit fact unerasable |
| **P5** — AGENTS.md RFC-style rewrite | **shipped** | 2026-05-13 | 1,241 → 373 lines (~75 % reduction); old version frozen as `AGENTS.v1.md` |

## P6 — `cyberos import <other-store>` for cross-BRAIN team merge

**Status:** Proposal. Not yet implemented. The single capability gap
between the v2 protocol and the user's stated workflow #4 (combining
multiple people's BRAINs into one project).

### Problem

The protocol supports independent BRAINs per person (each gets their
own `.cyberos-memory/` with its own chain + signing key). What's missing
is a *first-class merge tool* that lets person A pull selected memories
from person B's BRAIN into their own.

Manual workaround today: A unzips B's `cyberos export` bundle, picks
which `.md` files to keep, and re-applies them via `cyberos put`. Works,
but tedious and easy to get wrong.

### Proposed design

```
cyberos import <other-store-zip-or-dir> [options]

  --filter <expr>            limit to memories matching the predicate
                             (e.g. `kind=decision`, `sync_class=shareable`,
                             `actor=human:stephen`)
  --since <iso-date>         only memories created after this date
  --map-actor <from>:<to>    rewrite actor field on import (e.g.
                             `human:alice:human:alice@cyberskill.world`)
  --dry-run                  report what would import; no writes
  --on-conflict {skip,overwrite,branch}
                             how to handle path collisions; default skip
```

### Invariants the implementation MUST preserve

1. **Chain continuity** — every imported memory becomes a NEW audit
   row in the local chain with op=`put` (or `move` / `delete`). The
   foreign chain doesn't merge directly; it's *replicated* via fresh
   rows that record the foreign chain hash in `extra.imported_from_sth`.
2. **Provenance** — `actor` field preserves the foreign actor name by
   default; `--map-actor` lets the importer canonicalise.
3. **Privacy** — by default, only memories with
   `meta.sync_class=shareable` import. `--filter classification=public`
   widens it explicitly.
4. **Idempotence** — re-running an import after the source has new
   rows only imports the delta. Tracked via
   `manifest.imports[<source-fingerprint>].last_seq`.
5. **Auditability** — each import emits an `op="session.start"` row
   followed by the imported memories, then a `session.end`, so the
   audit log records the boundary.

### Effort estimate

* `cyberos/core/import_.py` — ~200 lines (binlog walker + filter pipeline
  + path-collision policy).
* `cyberos import` CLI subcommand — ~50 lines.
* Tests — ~150 lines (synthetic source store, multiple filter
  scenarios, conflict resolution, idempotence).
* Documentation — short section in `AGENTS.md` v2 §14 (cross-agent
  interop).

Total: ~1–2 focused sessions of work.

### Approval phrase

```
APPROVE protocol change P6 §14 (cross-BRAIN import tool)
```

No gating dependencies. Can land before or after P2 Stage 3.

---

## P2 Stage 3 — the only outstanding proposal

Stage 3 promotes Signed Tree Heads to the primary integrity primitive. The
per-row chain (`prev_chain` + `chain` fields on every audit row) becomes
optional and is no longer required for new writes — every record's identity
is established by its MMR leaf and the most recent STH.

**Why deferred:**

* The Deep Audit's risk register (R3) marks MMR-implementation-bug as
  Medium-likelihood / High-impact. The failure mode is *silent* — a wrong
  root looks correct until an inclusion proof is requested.
* `P2_RESOLUTION.md` requires a 2-week soak under Stage 1 (the additive
  MMR running alongside the chain) so the cross-check invariant has time
  to surface any drift under real load. The nightly soak task already
  exercises this.
* Once Stage 3 lands the rollback path tightens — the chain can be
  reconstructed from MMR leaves but it's mechanical work, not a single
  flag flip.

**Approval phrase:** when ready, cite

> APPROVE protocol change P2 §6 Stage 3 (chain primitive swap to MMR + STH)

A smaller commit unit is the optional Stage 2.5 — passphrase-wrap rotation
plus public-STH cross-anchoring (`P2_RESOLUTION.md` Q3 Mode 2). That's
useful operationally without committing to the primitive swap; ask if
interested.

## Other items considered and decided against

The Deep Audit also discussed:

* **`sync_class` four-way enum → `private` + `shareable` + ACL.** The v1
  four-tier values are preserved in `meta.sync_class_v1` for one release
  cycle (see AGENTS.md v2 §15). No active migration; legacy values keep
  working.
* **Tier-based source-priority table → five-row table.** Implemented in
  AGENTS.md v2 §8.1.
* **`§0.4` TIER 1/2/3 self-amendment grammar → binary
  `propose-now`/`log-deferred`.** Implemented in AGENTS.md v2 §16.
* **EVOLUTION.md, INTEROP.md split from AGENTS.md.** Done at W0.

---

## How to approve P2 Stage 3 (the only remaining)

```
APPROVE protocol change P2 §6 Stage 3 (chain primitive swap to MMR + STH)
```

Suggested gate: wait for at least 14 consecutive nightly soak runs (already
scheduled at 01:09 daily) with the `ledger-mmr-cross-check` invariant green
and no MMR-related warnings in the doctor output. The cyberos-nightly-soak
scheduled task tracks this.
