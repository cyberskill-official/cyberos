# P2_RESOLUTION.md — proposed answers to EVOLUTION.md Q1–Q3

**Status:** Proposal. Not enacted. The crypto-primitive swap itself
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
