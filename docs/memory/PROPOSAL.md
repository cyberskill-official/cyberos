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

---

# Appendix: P2 Stage 3 — Resolved design questions

*Previously docs/memory/P2_RESOLUTION.md (folded in 2026-05-14).*

(per-row chain → MMR + STH) requires user approval after these three
open questions are answered.

The Deep Optimization Audit §7 R3 explicitly flags: "MMR implementation
bug → wrong root" as Medium-likelihood / High-impact. P2 is the largest
change in the proposal stack and the only one whose corruption mode is
silent (a wrong root looks like a correct root until an inclusion proof
is requested). The three questions below must be answered concretely
before the swap is safe to ship.

The companion stub at ``cyberos/core/sth.py`` is **additive** — it
produces Signed Tree Heads alongside the current per-row chain, without
replacing it. The swap to STH-as-primitive is gated on resolving these
questions AND on a separate chat-turn approval.

---

## Q1 — Which MMR implementation?

Three candidate paths, ranked by safety + maintenance burden:

### Option A — pure Python reference implementation (RECOMMENDED for v2.1)

* Vendor a minimal MMR (≤ 300 lines) into ``cyberos/core/mmr.py``.
* Algorithm from IACR eprint 2025/234 §3 (Bonneau-Christ-Hafezi).
* Property-tested against the DataTrails Go reference + Grin's Rust
  implementation using shared test vectors.
* Zero runtime deps; works in the sandbox; reviewable in one sitting.
* Trade-off: slower than the Rust crate (negligible for our scale —
  thousands of records, not billions).

### Option B — ``merkle-mountain-range`` Rust crate via PyO3

* Production-grade, used by Grin / Mina / Beam.
* Trade-off: PyO3 build chain (Rust toolchain dep, cross-compile for
  macOS arm64 + Linux x86_64), CI cost, and a wheel rebuild on every
  upstream release.
* Defer until v2.2 if A's performance is insufficient.

### Option C — wrap DataTrails' Go reference via subprocess

* Mature, audited, includes the "height-14 massif" partitioning.
* Trade-off: Go binary as a dep; subprocess overhead per leaf.
* Not recommended for a personal store — adds operational complexity
  for negligible benefit.

**Recommendation: A.** Concrete plan:

1. Vendor ``cyberos/core/mmr.py``: pure Python, SHA-256 nodes, peak-list
   maintained on disk at ``audit/mmr/peaks-<seq>.bin``.
2. Test corpus: 1k random leaf sequences, cross-checked against a Python
   port of the DataTrails reference vectors (one-time validation).
3. Property tests: inclusion proof verifies; consistency proof between
   any two tree sizes verifies; concatenation of two trees produces a
   root matching the canonical "combine" operation.

---

## Q2 — Key management for the Ed25519 signing key

Three layers to design:

### 2.1 Key storage

* **Recommended:** ``age``-style passphrase-wrapped storage at
  ``~/.config/cyberos/sth_signing_key.age``. The passphrase is provided
  by the user (cached in agent or OS keychain for the active session;
  never stored alongside the key).
* Rejected: storing in the OS keychain only. Cross-machine sync via
  iCloud Keychain / Bitwarden / etc. is out of scope for v2.1; the key
  is per-host, and STHs from different hosts are independently valid.

### 2.2 Key rotation

* New key → sign a "rotation STH" whose ``previous_signer`` field
  identifies the old key's last STH by hash. Old key is destroyed.
* Auditor verifies: every STH back to genesis chains via the
  ``previous_signer`` field; no chain break across rotations.
* Default rotation interval: annual or on key compromise.

### 2.3 Key compromise recovery

* If the active key is suspected compromised: generate a new key, sign
  a single STH that pins the old key's last legitimate STH plus a
  ``revocation: true`` field, and immediately rotate.
* Audit consumers MUST treat STHs signed AFTER the revocation timestamp
  by the old key as invalid.

**Risk register entry:** loss of the signing key (passphrase forgotten)
is fatal for *future* STHs but does not corrupt history — every prior
STH remains independently verifiable by its embedded public key. The
user can sign new STHs with a new key as long as the chain is anchored
to the last legitimate STH.

---

## Q3 — Publish STHs to a public transparency log?

Three modes, listed least-to-most-public:

### Mode 1 — Local only (default, RECOMMENDED)

* STHs live under ``audit/sth/`` inside the store.
* Auditor verifies by reading the local store. No third-party trust
  required.
* CyberSkill's "single-actor sovereignty over personal memory" remains
  intact.

### Mode 2 — Cross-anchor to another local repo

* Each STH's root hash is recorded in a paired log (e.g. another
  ``.cyberos-memory`` on a different machine).
* Gives cross-host forgery detection without third-party trust.
* Requires user to set up the second repo. Opt-in.

### Mode 3 — Publish to Sigstore Rekor v2 / Trillian-Tessera

* Best-in-class third-party verifiability.
* Trade-off: every STH leaks (a) the user's signing key fingerprint and
  (b) the timestamp of every consolidation to a public log. Privacy
  trade-off the user must consciously accept.
* Recommended only for shared/client-visible scope. NOT default.

**Recommendation: Mode 1 default; provide hooks for 2 and 3.** Mode 2
is a 2-line config change ("here's my paired repo"). Mode 3 is a
``cyberos publish-sth --target rekor`` flag, off by default.

---

## Implementation sequencing (post-approval)

If you approve Q1=A / Q2 (3-layer key model) / Q3=Mode 1:

1. **Stage 1 (W1 shadow):** Land ``cyberos/core/mmr.py`` +
   ``cyberos/core/sth.py``. Writer produces a per-batch leaf into the MMR
   alongside the existing chain. Consolidation produces an STH AND keeps
   the chain. Both invariants checked at every doctor run.
2. **Stage 2 (post-W1 soak, ~2 weeks):** Verify the MMR root and chain
   tip agree on every record under load. If divergence: P0; halt.
3. **Stage 3 (W2 cutover):** STH becomes the primary integrity primitive.
   New writes omit ``prev_chain`` / ``chain`` from the row payload; the
   record's MMR leaf is its identity. The legacy chain stays in
   ``audit/legacy_chain_tail.json`` for forensic continuity.
4. **Stage 4 (cleanup, ~1 month after W2):** Drop the legacy chain
   verification path. Chain fields in old rows remain valid history.

Each stage is independently reversible until the next is taken.

---

## How to approve

Same as the other proposals — cite the section and the proposal id:

> APPROVE protocol change P2 §6 (MMR + STH); resolutions Q1=A, Q2=3-layer-key, Q3=Mode-1

If you want a smaller commit, you can also approve only Stage 1 (additive
MMR + STH alongside the chain, no primitive swap yet):

> APPROVE protocol change P2 §6 Stage 1 only (additive MMR + STH, chain unchanged)
