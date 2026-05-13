# LEGACY_SCRIPTS.md — survey of `runtime/tools/*.py`, mapped to v2 equivalents

**Status:** Survey only. No deletions until you approve per-script.
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
