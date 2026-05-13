# EVOLUTION.md — CyberOS Memory Protocol: History, Bundles, Audit Reports

**Status:** Informative. Not loaded by agents per session. Lives alongside the
normative `AGENTS.md` and the machine-validatable `memory.schema.json`.

This file is the **history file** the Deep Optimization Audit §4.1 calls for:
"AGENTS.md MUST NOT contain history". Stages 1–6 narrative, Bundles A–Q
refinement bundles, prior audit reports, and the rationale-trail for every
decision belong here, not in the per-session-loaded normative document.

> Editorial rule: this file's content may grow; AGENTS.md's content shrinks
> in inverse proportion. Anything that ends "we learned…" or "the reason
> was…" goes here.

---

## 0. Document inventory

The current protocol lives in five files under `docs/memory/`:

| File | Status | Loaded per session? |
|---|---|---|
| `AGENTS.md` | Normative | yes |
| `memory.schema.json` | Machine-validatable | no (referenced by tools) |
| `memory.invariants.yaml` | Walker input | no (referenced by self-audit) |
| `INTEROP.md` | Normative (subset) | optionally, by external agents |
| `EVOLUTION.md` | Informative (this file) | **no** |
| `README.md` | Pedagogical | no |

`AGENTS.md` SHOULD remain ≤ 6 000 tokens; `EVOLUTION.md` is unbounded.

---

## 1. Stage history (placeholder — to be populated)

The protocol has accreted through six numbered stages plus a series of
refinement bundles. Each stage represents a coherent set of changes shipped
together; bundles are tactical refinements between stages.

Stages — fill in from your existing notes when next consolidating:

- **Stage 1** — Bootstrap (2026-05-04 genesis). Initial six-op vocabulary,
  YAML frontmatter, per-row Merkle chain.
- **Stage 2** — _placeholder_
- **Stage 3** — _placeholder_
- **Stage 4** — _placeholder_
- **Stage 5** — Encryption envelope (Stage 5 at-rest encryption,
  REF-030). _Move full prose here._
- **Stage 6** — Long-term health (REF-029). _Move full prose here._

To migrate prose from `AGENTS.md` and `docs/memory/README.md` Parts 25–31
into this file: copy the section, replace its location in the source with
a one-line reference (`See EVOLUTION.md §<n>.<m>`), and commit. Per the
Deep Audit, README Parts 25–31 absorbed Layer-1 manual content that
belongs here, not in the protocol document or the pedagogical readme.

---

## 2. Bundle index

Refinement bundles flagged in `.cyberos-memory/memories/refinements/`:

- REF-001 through REF-037 — _placeholder, populate from refinements/_.

When `cyberos consolidate` (or equivalent) is implemented, it should auto-
populate this index from the refinements directory; for now this is a
hand-curated list.

---

## 3. Prior audit reports

Major external audits the protocol has absorbed:

### 3.1 Layer-1 Optimization Audit — May 2026

**Scope:** Filesystem store, audit ledger, writer/reader. Reference impl
under `cyberos/core/`.

**Headline findings:**

1. Per-row fsync on the JSONL audit ledger — replaced by group commit
   (5 ms window or 16 rows) + binary framed segments.
2. PyYAML frontmatter — replaced by msgspec JSON (~250× faster).
3. 63 standalone tool scripts — collapsed into a single `cyberos` CLI
   with lazy imports (cold `--help` < 30 ms).
4. SQLite index in DELETE journal mode — switched to WAL + tuned PRAGMA.
5. **macOS `fsync()` data-loss bug** — fixed by routing per-batch syncs
   through `F_BARRIERFSYNC`, checkpoint syncs through `F_FULLFSYNC`.

**Outcome:** Schema v2 with chain-bridge migration. `legacy_last_chain`
in `manifest.json:migration` pins the v1 chain tip into v2 so the
cumulative Merkle chain is continuous across the boundary. Implementation
in `cyberos/core/` (12 modules); migration in `runtime/tools/cyberos_migrate_v2.py`;
38 regression tests under `tests/core/`.

**Status:** Shipped. Cutover complete on user's BRAIN; legacy
`brain_writer.py` now delegates to v2 via `runtime/lib/brain_writer_shim.py`
when `schema_version >= 2`.

### 3.2 Deep Optimization Audit — May 2026 (Protocol Document)

**Scope:** `docs/memory/AGENTS.md` itself — the protocol document, not the
implementation.

**Headline findings:**

1. AGENTS.md is **over-specified at ~13–18k tokens** — should split into a
   ~5–6k normative core + schema + invariants + history + interop subset.
2. Six file ops should collapse to three (`put`, `move`, `delete(mode)`)
   with content hashing replacing `insert`/`str_replace`'s semantic.
3. Per-row Merkle chain should be replaced by Merkle Mountain Range (MMR)
   + Signed Tree Heads (STH) per consolidation — the modern transparency-log
   primitive (Sigstore Rekor v2, Trillian-Tessera, IACR 2025/234).
4. YAML frontmatter inside Markdown bodies should move to `<file>.meta.json`
   sidecars validated against `memory.schema.json` — Norway Problem,
   octal ambiguity, YAML 1.1↔1.2 split.
5. `§0.4` TIER 1/2/3 self-amendment priority is scaffolding from the
   growth phase — collapse to binary `propose-now`/`log-deferred`.
6. Add `delete(path, "purge")` for GDPR Article 17 compliance.

**Status of recommendations (final, 2026-05-14):**

The user waived `§0.2` for the rebuild session ("i approve all since this
is the rebuild"). Every recommendation except P2 Stage 3 has shipped.

| Recommendation | Status | Date |
|---|---|---|
| `memory.schema.json` (generated from msgspec types) | **shipped (W0)** | 2026-05-13 |
| `memory.invariants.yaml` (declarative invariant set, 15 invariants) | **shipped (W0)** | 2026-05-13 |
| `INTEROP.md` (Cursor-compatible subset, ≤6,000 chars) | **shipped (W0)** | 2026-05-13 |
| `EVOLUTION.md` skeleton (this file) | **shipped (W0)** | 2026-05-13 |
| AGENTS.md RFC rewrite (BCP 14, ~373 lines, 75 % token reduction) | **shipped (P5)** | 2026-05-13 |
| Six → three canonical ops (`put`/`move`/`delete(mode)`) + v1 aliases | **shipped (P1)** | 2026-05-13 |
| Sidecar `*.meta.json` migration tool + reader support | **shipped (P3)** | 2026-05-13 |
| GDPR `delete(path, "purge")` with magic-phrase gate | **shipped (P4)** | 2026-05-13 |
| MMR + Ed25519 STH (additive alongside chain) | **shipped (P2 Stage 1)** | 2026-05-14 |
| Passphrase-wrapped signing key (scrypt + ChaCha20-Poly1305) | **shipped (P2 Stage 2)** | 2026-05-14 |
| Chain primitive swap (MMR + STH become *the* integrity primitive) | **DEFERRED (P2 Stage 3)** | gated on 2-week soak + fresh approval |

The single outstanding item — P2 Stage 3 — is documented in
`docs/memory/PROPOSAL.md` with the gate criteria and the approval phrase.

---

## 4. Open questions

Status after 2026-05-14:

- **Q1 (Deep Audit §7 R3) — RESOLVED:** Pure-Python MMR shipped at
  `cyberos/core/mmr.py` (peak-stack representation, ~340 lines, zero
  external deps). See `P2_RESOLUTION.md` for the analysis behind picking
  this over the Rust crate.
- **Q2 (Deep Audit §7 R2) — RESOLVED:** Passphrase-wrapped storage via
  scrypt + ChaCha20-Poly1305 shipped at `cyberos/core/sth.py`. Key file
  is backward-compatible: stage-1 raw seeds still load. CLI subcommand
  `cyberos sth-wrap` migrates in place; public key preserved so existing
  STHs remain verifiable.
- **Q3 (Layer-1 audit §B11) — DEFAULTED:** Local-only STH publication
  (`P2_RESOLUTION.md` Mode 1) is the default; Mode 2 (paired-host
  cross-anchor) and Mode 3 (public Rekor/Trillian) are documented but
  not active. Toggle via future config; not blocking.
- **Q4 (Bundle M flagging) — RESOLVED:** §4.10/§4.11 merge, §17
  sync_class forward-references, §8 heading simplification all adopted
  in `AGENTS.md` v2.

There are no outstanding open questions from the Layer-1 or Deep audits
at the time of writing. The single remaining proposal (P2 Stage 3 chain
primitive swap) is gated on a 2-week soak, not on an open question.

---

## 5. Editorial procedure (moved from AGENTS.md §0.6)

When implementation files in `cyberos/core/` change behaviour that the
protocol promises, update `AGENTS.md` in the same commit. When the
*reasoning* behind a protocol decision changes, update `EVOLUTION.md`.

This file is append-mostly: add a dated section under "## N. <topic>"
when something substantive happens; don't rewrite history.
