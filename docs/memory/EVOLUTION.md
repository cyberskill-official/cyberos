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

The protocol lives in `docs/memory/`:

| File | Status | Loaded per session? |
|---|---|---|
| `AGENTS.md` | Normative | yes |
| `memory.schema.json` | Machine-validatable | no (referenced by tools) |
| `memory.invariants.yaml` | Walker input | no (referenced by self-audit) |
| `INTEROP.md` | Normative (subset) | optionally, by external agents |
| `EVOLUTION.md` | Informative (this file) | **no** |
| `PROPOSAL.md` | Informative (outstanding proposals + resolutions) | **no** |
| `README.md` | Pedagogical (newcomer guide) | no |
| `CHANGELOG.md` | Informative (dated history) | **no** |

`AGENTS.md` SHOULD remain ≤ 6 000 tokens; `EVOLUTION.md` is unbounded.

§5 below records the cleanup of legacy script tooling that
previously lived in `LEGACY_SCRIPTS.md` (folded in 2026-05-14).

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
  external deps). See `PROPOSAL.md` Appendix for the analysis behind
  picking this over the Rust crate.
- **Q2 (Deep Audit §7 R2) — RESOLVED:** Passphrase-wrapped storage via
  scrypt + ChaCha20-Poly1305 shipped at `cyberos/core/sth.py`. Key file
  is backward-compatible: stage-1 raw seeds still load. CLI subcommand
  `cyberos sth-wrap` migrates in place; public key preserved so existing
  STHs remain verifiable.
- **Q3 (Layer-1 audit §B11) — DEFAULTED:** Local-only STH publication
  (`PROPOSAL.md` Appendix Mode 1) is the default; Mode 2 (paired-host
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

---

# §5. Cleanup record — retired legacy scripts

*Previously docs/memory/LEGACY_SCRIPTS.md (folded in 2026-05-14).*

**Scope:** 59 scripts under `runtime/tools/` (the Deep Audit's "63+" was a
nearby estimate; today's count is 59).
**Companion:** `cyberos/README.md` lists v2 subcommands; `EVOLUTION.md`
records Stage history.

Each script is graded one of:

* **A. Delete** — fully subsumed by a v2 subcommand or core module. Safe
  to remove after a 7-day soak window (Layer-1 audit §6 Phase 6).
* **B. Merge** — substantive logic that should fold into a v2 subcommand
  but hasn't yet. Keep the file; track the merge as a follow-up.
* **C. Keep (operational)** — orthogonal to the writer; not duplicated
  by v2. UX, monitoring, integration, or experimental scope.
* **D. Keep (proposal scope)** — folds into an active `PROPOSAL.md` item
  (P1–P5). Resolve at proposal-approval time, not now.

Hard rule the §0.2 immutability clause imposes: **no deletions in this
session.** This file is a deletion *plan*, not a deletion log.

---

## A. Delete (after 7-day soak, ~11 scripts)

**Status (2026-05-14):** Survey complete. The legacy `runtime/tools/cyberos`
bash wrapper still routes 9 of the 11 commands to their underlying scripts.
Until that wrapper is itself retired (separate, focused project), only the
2 scripts with zero callers are retired.

* ✅ `cyberos_lazy.py` — replaced with deprecation stub; exits 2 on run.
  Zero importers, zero bash refs. Safe to `git rm` whenever convenient.
* ✅ `cyberos_index_hook.py` — replaced with deprecation stub. Same.
* ⏸️ The other 9 stay alive until the bash wrapper migrates to
  `python -m cyberos` itself.

These are directly replaced by v2 functionality. Keep until the nightly
soak has run cleanly for a week post-cutover; then drop.

| Script | Replacement | Status |
|---|---|---|
| `cyberos_doctor.py` | `python -m cyberos doctor` | ⏸️ retained (bash wrapper) — 4 refs in `runtime/tools/cyberos` |
| `cyberos_export.py` | `python -m cyberos export` | ⏸️ retained (bash wrapper) — 4 refs |
| `cyberos_validate.py` | `python -m cyberos validate` | ⏸️ retained (bash wrapper) — 7 refs |
| `cyberos_lock.py` | `cyberos.core.lock.StoreLock` | ⏸️ retained (bash wrapper) — 1 ref |
| `cyberos_lazy.py` | `cyberos.core.reader` mmap + seqlock | ✅ **deprecation stub installed 2026-05-14** |
| `cyberos_index_hook.py` | `cyberos.core.index.replay_from_binlog` | ✅ **deprecation stub installed 2026-05-14** |
| `cyberos_compact_stats.py` | `cyberos.core.walker.MmapWalker` + manifest | ⏸️ retained (bash wrapper) — 1 ref |
| `cyberos_index.py` | `cyberos.core.index.open_index` | ⏸️ retained (bash wrapper) — 3 refs |
| `cyberos_show.py` | `python -m cyberos audit dump` + index search | ⏸️ retained (bash wrapper) — 2 refs |
| `cyberos_migrate.py` | `cyberos_migrate_v2.py` | ⏸️ retained (bash wrapper) — 1 ref |
| `canonical_sha.py` | `cyberos.core.writer._canonical` | ⏸️ retained (bash wrapper) — 2 refs |

**Pre-delete checklist** (do not skip):
1. Grep the repo for `runtime/tools/cyberos_<name>` to find callers.
2. Replace each call site with the v2 equivalent.
3. Run the full pytest suite + `cyberos doctor`.
4. Run the nightly soak for 7 days.
5. Then delete + commit with `git rm` (preserves history).

---

## B. Merge into a v2 subcommand (~12 scripts)

Substantive logic; the right shape is a new `cyberos <verb>` subcommand
that lazy-imports the legacy implementation. No data-mutation rewrites.

| Script | Proposed v2 subcommand | Effort |
|---|---|---|
| `cyberos_dedup.py` | `cyberos dedup` | Wrap as-is; preserves content-fingerprint logic |
| `cyberos_prune.py` | `cyberos prune` | Wrap as-is; staleness + contradiction detection |
| `cyberos_graph.py` | `cyberos graph` | Wrap; outputs DOT/JSON |
| `cyberos_autorepair.py` | `cyberos doctor --repair` | Folds into doctor's repair mode |
| `cyberos_cleanup.py` | `cyberos doctor --cleanup` | Same — gated leftover-detection |
| `cyberos_bulk.py` | `cyberos bulk` | Wrap; needs write-path delegation through ops.put |
| `cyberos_add.py` | `cyberos add` | Interactive wizard; ops.create under the hood |
| `cyberos_edit.py` | `cyberos edit` | $EDITOR wrapper; ops.str_replace under the hood |
| `cyberos_history.py` | `cyberos history` | Diff + time-travel; reads binlog via walker |
| `cyberos_hybrid_search.py` | `cyberos search --hybrid` | Already have `cyberos search`; flag-gated mode |
| `cyberos_semantic_search.py` | `cyberos search --semantic` | Same; pluggable backend |
| `cyberos_repl.py` | `cyberos repl` | Wrap; uses the v2 op functions in-process |

**Merge order suggested:** doctor extensions first (autorepair, cleanup),
then search modes (semantic, hybrid), then write-path wrappers (add, edit,
bulk). Each merge is a 1-day task in isolation.

---

## C. Keep (operational, orthogonal — ~22 scripts)

Not duplicated by v2; serve UX, monitoring, or integration roles.

| Script | Why keep |
|---|---|
| `cyberos_migrate_v2.py` | The migration itself |
| `cyberos_generate_schema.py` | Generates `docs/memory/memory.schema.json` |
| `cyberos_encrypt.py` | Stage 5 at-rest encryption + Shamir 3-of-5 escrow |
| `cyberos_hooks.py` | Claude Code hook installation; orthogonal to writer |
| `cyberos_cold_storage.py` | Cold-tier `.zip` archives with Merkle anchors |
| `cyberos_onboard.py` | Interactive new-contributor bootstrap |
| `cyberos_serve.py` | Local web dashboard |
| `cyberos_tui.py` | curses live dashboard |
| `cyberos_static.py` | Static HTML render of the BRAIN |
| `cyberos_analytics.py` | Local-only usage telemetry |
| `cyberos_authoring.py` | Stage 3 authoring quality amplifiers |
| `cyberos_skill.py` | Skill registry loader |
| `cyberos_skill_bench.py` | Skill cost + accuracy benchmarks |
| `cyberos_skill_quality.py` | Stage 6 quality + trust amplifiers |
| `cyberos_cross_skill.py` | Cross-skill consistency validation |
| `cyberos_fr.py` | Feature-request browser + task-graph |
| `cyberos_fr_migrate.py` | Legacy `feature_request@1` migration |
| `cyberos_fr_parser.py` | Shared FR artefact parser |
| `cyberos_proj.py` | Project-tracker sync |
| `cyberos_project_index.py` | Auto-refresh `project-index.md` |
| `cyberos_chain.py` | FR → tasks chain orchestrator |
| `cyberos_refinements.py` | §0.4 refinement candidate dashboard |
| `cyberos_ref_from_drift.py` | Pre-fill a REF from a drift candidate |
| `cyberos_council.py` | Opt-in council-mode synthesis for ambiguous REFs |
| `benchmark.py` | Legacy benchmark suite; redundant with `bench/` but no behaviour overlap |
| `voice_check.py` | Tone / voice check (orthogonal) |
| `extract_agents_core.py` | Ad-hoc extractor; standalone utility |
| `cyberos_advanced.py` | Stage 8 future-state scaffolds |
| `cyberos_stream.py` | Audit-row streaming + webhooks |

Most of these don't touch the audit ledger and are safe alongside v2. The
ones that do (e.g. `cyberos_encrypt`, `cyberos_cold_storage`) write
schema-v1-shaped audit rows via the legacy `brain_writer`; the shim
intercepts and refuses with a deferral message under v2. Each of those
needs a v2-specific update before they can run again — but that's
implementation work, not deletion work.

---

## D. Keep (proposal scope — defer to P1–P5 approval, ~5 scripts)

These map directly onto staged `PROPOSAL.md` items. Decide their fate
when the corresponding proposal is approved or rejected.

| Script | Linked proposal | What happens on approval |
|---|---|---|
| `cyberos_sign.py` | P2 (MMR + STH) | Replaced by STH-signing inside the writer; standalone CLI deleted |
| `cyberos_replicate.py` | P2 + AT Protocol inductive-firehose model | Folds into a `cyberos replicate` mode bound to STH inclusion proofs |
| `cyberos_sync.py` | P2 (sync via STH cross-anchor) | Replaced by the STH-based protocol |
| `cyberos_branch.py` | (no current proposal — future R&D) | Defer; not currently in scope |
| `cyberos_crdt.py` | (no current proposal) | Defer; experimental |
| `cyberos_tenant.py` | (no current proposal — multi-tenant story TBD) | Defer |
| `cyberos_parallel_validate.py` | Implicit in P2 (auditor model) | Defer |

---

## Summary

* **A. Delete after soak:** 11 scripts (≈ 19% of the surface).
* **B. Merge into v2 subcommands:** 12 scripts (≈ 20%).
* **C. Keep — operational:** 29 scripts (≈ 49%).
* **D. Keep — proposal scope:** 7 scripts (≈ 12%).

The Layer-1 audit's "delete ~40, merge ~20" target was optimistic; the
real surface is mostly C — orthogonal UX / integration tools that never
needed to be part of the writer. The right number of *writer-overlapping*
scripts to retire is ~11, not ~40.

When you're ready to enact group A, the smallest reversible commit
sequence is one script per commit, each with: grep callers → replace with
v2 equivalent → run tests → `git rm`. Trying to delete the whole group
in one commit will produce a noisy diff and a long bisect window if
something regresses.
