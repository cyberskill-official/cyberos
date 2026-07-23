# TASK-MEMORY-303 — implementation evidence

Worker: T6 (memory hardening), batch/8-audit-hardening, 2026-07-23.
Scope executed per the batch carve-outs: run-gates wiring → gates worker;
build.sh edits → install worker; live-store mutation → operator-gated
(plan authored, proof on a /tmp copy). Everything below ran with the live
store opened read-only; its `.lock` was never acquired.

## 1. What changed and why (file by file)

### Modified — modules/memory/** (this worker's exclusive set)

| file | why |
|---|---|
| `tools/cyberos_generate_schema.py` | Generator is the single source (§1.1): now emits the three `StoreAcl*` definitions (§14.4.7), the `episode` kind + its description, and the package-data copy's top-level description. Generator output is semantically identical to the pre-existing package-data copy (proven by JSON-equality before regeneration) — nothing was rolled back. |
| `memory.schema.json` | Regenerated from the generator. Now carries StoreAcl/StoreAclEntry/StoreAclMode + episode kind. Byte-identical to the data copy (both sha256 `7712b8dc…7548`). |
| `cyberos/data/memory.schema.json` | Regenerated from the same generator run — formatting normalized (the old file was hand-edited with non-generator JSON layout), content JSON-equal to before. This is the canonical copy build.sh now vendors (install worker's change, verified here by test). |
| `tests/test_schema_drift.py` | §1.2: `_COMMITTED` pointed at nonexistent `modules/memory/docs/memory.schema.json` → every test silently skipped. Now points at the real root copy, missing schema is `pytest.fail` (never skip), docstring regen command fixed to regenerate BOTH copies, and the required-definitions check includes the three ACL keys. NOTE: the file was mode 0400 on disk; chmod'd to 0644 to edit. |
| `cyberos/core/invariants.py` | §1.4: `_CANONICAL_TOP_LEVEL_DIRS` += `sessions`, `dreams` (with a comment pinning the ISS-006 non-interference semantics vs `_SANDBOX_FRAGMENTS`). Three new checks + registry entries + `__all__`: `check_dream_applied_provenance` (§7.7.2), `check_store_yaml_acl_valid` (§14.4.7 — structural transcription of the schema's StoreAcl definition, no optional jsonschema dep), `check_session_lifecycle` (§18.8 — pairing, duplicate starts/ends, strict turn_seq from 0, no turn before start / after end, tombstone-exempt, orphan-binlog detection). Plus `_iter_chain_rows` helper (all segments, oldest first). All three pass trivially on minimal stores (no chain / no sessions / no STORE.yaml). |
| `memory.invariants.yaml` + `cyberos/data/memory.invariants.yaml` | The three invariants declared (16 total now). Both copies byte-identical (the walker loads the package-data copy first). |
| `cyberos/core/writer.py` | §1.7: `Writer.submit` stamps `extra.session_id` on put/move/delete rows while `sessions/.active` names a session (direct file read — writer stays import-light). Caller-supplied session_id wins; aux/summary rows exempt; stale marker trusted by design (documented + tested). |
| `cyberos/core/dream/detectors.py` | **Exposed latent defect, fixed** (see §3 below): `_enumerate_memories` now excludes tombstoned paths (`_tombstoned_paths` — replays delete/put/move rows), so dream detection never proposes ops on §3.5-deleted memories and SemanticDedup re-apply is genuinely idempotent (TASK-MEMORY-116 AC #11). |
| `cyberos/__main__.py` | Doctor subparser accepts `--store` after the subcommand (`default=argparse.SUPPRESS` so it never clobbers the root-level flag). Standalone-gate ergonomics for §1.6. |
| `CHANGELOG.md` (module) | §1.8: top entry names the four deliverable groups. See "deviations" for the root-CHANGELOG call. |

### New files

| file | what |
|---|---|
| `modules/memory/INTEROP.md` | §1.3 / §14.1 consumer subset, **5,230 chars** (≤ 6,000). Five mandated anchors: read paths; MUST NOT write `audit/`/`HEAD`/`.lock`; canonical-writer routing; §14.4.6 STORE.yaml honor-for-writes; §14.3 sync_class semantics. |
| `modules/memory/tests/test_schema_single_source.py` | AC 1 + AC 2. The vendored leg resolves every `memory.schema.json` path referenced in build.sh and asserts byte-identity with the canonical copy — robust to the install worker's edits and fails if any copy forks again. |
| `modules/memory/tests/test_interop_doc.py` | AC 3 (presence, bound, anchors, build.sh vendors it) + AC 8 (changelog groups). |
| `modules/memory/tests/test_walker_sessions_dreams.py` | AC 4 (layout pass with `sessions/`+`dreams/`; three violating fixtures each fail exactly their own invariant; clean store passes all three; ISS-006 sandbox non-interference both directions) + AC 5 fixture leg (relocation on a live-layout clone: red → green, chain intact, 2 move rows verified). |
| `modules/memory/tests/test_session_id_stamping.py` | AC 7 (stamp present iff active, across put/move/delete) + stale-marker edge + caller-override. |
| `docs/tasks/memory/TASK-MEMORY-303-memory-contract-hardening/store-repair-plan.md` | Operator-gated §1.5 repair: exact `move` ops, body-hash preconditions, expected post-state, rollback, before/after doctor output, volatility + re-contamination warnings. |

## 2. Verbatim verification output

New + fixed tests (13/13):

```text
tests/test_schema_drift.py::test_committed_schema_matches_generator_output PASSED
tests/test_schema_drift.py::test_schema_has_required_definitions PASSED
tests/test_schema_drift.py::test_schema_audit_record_op_is_permissive_string PASSED
tests/test_schema_single_source.py::test_all_copies_identical_and_acl_bearing PASSED
tests/test_schema_single_source.py::test_drift_test_cannot_skip PASSED
tests/test_interop_doc.py::test_interop_present_bounded_vendored PASSED
tests/test_interop_doc.py::test_changelog_records_hardening PASSED
tests/test_walker_sessions_dreams.py::test_new_invariants_pass_and_fail_correctly PASSED
tests/test_walker_sessions_dreams.py::test_sessions_allowlist_does_not_touch_sandbox_check PASSED
tests/test_walker_sessions_dreams.py::test_repair_fixture_relocation_preserves_chain PASSED
tests/test_session_id_stamping.py::test_stamp_present_iff_active PASSED
tests/test_session_id_stamping.py::test_stale_marker_still_stamps PASSED
tests/test_session_id_stamping.py::test_caller_supplied_session_id_wins PASSED
============================== 13 passed in 0.68s ==============================
```

Full memory suite (`python3 -m pytest modules/memory/tests -q`):

```text
519 passed, 5 skipped, 11 warnings in 21.56s
```

(The 5 skips are pre-existing, platform-conditional: `tests/core/test_crash_safety.py` "fork-and-kill crash safety regression runs on Linux only". Baseline before this task's changes: 518 passed / **3 silent drift-test skips**; the suite is +14 tests net with zero drift skips.)

Drift-injection proof (delete `StoreAcl` from the root copy → run → restore):

```text
FAILED tests/test_schema_drift.py::test_committed_schema_matches_generator_output
FAILED tests/test_schema_drift.py::test_schema_has_required_definitions
FAILED tests/test_schema_single_source.py::test_all_copies_identical_and_acl_bearing
FAILED tests/test_schema_single_source.py::test_drift_test_cannot_skip
4 failed, 1 passed in 0.49s
--- restored ---
5 passed in 0.46s
```

Schema copy hashes after unification (byte-identical, `--check` exit 0 on both):

```text
7712b8dcd553dfc9bc3ef07e665fec7177ae276b520aa70618fe6e9c55767548  modules/memory/memory.schema.json
7712b8dcd553dfc9bc3ef07e665fec7177ae276b520aa70618fe6e9c55767548  modules/memory/cyberos/data/memory.schema.json
```

Doctor smoke against a full COPY of the live store (`cp -R .cyberos/memory/store /tmp/…`, sandbox exempted via `CYBEROS_HOST_MOUNT_PREFIX=/tmp`):

- BEFORE repair: `total: 16 pass: 15 warn: 0 error: 1` — the one error is
  `layout-root-canonical  unexpected top-level entries: ['impl-plans/', 'adrs/']` → exit 1.
  All three new invariants run and PASS trivially on this store.
- AFTER executing the repair plan on the copy: `total: 16 pass: 16 … overall: OK` → exit 0;
  `cyberos verify` → `verified 2 records across 1 segment(s); chain intact` → exit 0.
  Full transcripts embedded in `store-repair-plan.md` §6.

## 3. Exposed latent defect (why detectors.py is in the diff)

Legalising `dreams/` (normative, §1.4) un-blocked a code path: at HEAD,
the FIRST SemanticDedup run wrote `dreams/<ts>/diff.json`, which made
`layout-root-canonical` fail, which made every SUBSEQUENT consolidate
Walk phase red, which silently skipped every subsequent dedup run —
`test_reapply_is_idempotent` (TASK-MEMORY-116 AC #11) was green only
because the second run never executed. With `dreams/` legal the second
run really runs, and the duplicates detector re-found tombstoned bodies
(§3.5 tombstones keep bytes on disk) and re-applied a merge: 1 → 2 rows,
deterministic failure. Proven by bisect: pristine-HEAD extraction passes;
HEAD + only-my-invariants.py fails; the allowlist line is the trigger.
Fix (in-scope, `modules/memory/**`): `_enumerate_memories` excludes paths
whose latest chain row tombstoned them (state follows `move`, `put`
resurrects). Dream detection over deleted memories was wrong regardless;
re-apply idempotency now holds for the right reason. Dedup + dream suites:
28/28 pass.

## 4. Doctor invocation contract (for the gates worker)

The entrypoint is standalone-ready; wire the gate as:

```sh
# presence probe (skip with provenance line when absent):
[ -d "$repo/.cyberos/memory/store" ]
# importability probe — module identity, not a $PATH binary (ISS-008 / TASK-IMP-130):
python3 -c "import cyberos.core" 2>/dev/null
# the gate command (both --store positions accepted; CYBEROS_STORE env also works):
python3 -m cyberos doctor --store "$repo/.cyberos/memory/store"
# optional: --json for machine-readable output ({"ok": bool, "results": [...16 entries]})
```

Exit contract: **0** = every error-level invariant passed (warning-level
failures, e.g. a missing `crc32c` wheel, do NOT fail the run — matches
"green is necessary, never sufficient"); **1** = ≥1 error-level failure;
**2** = argparse/usage error. The walker is read-only against the store
(the only writing mode is the separate `--repair` flag — do not use it in
the gate). Runtime on the live-store copy: ~0.5–0.6 s (16 invariants;
`manifest-validates-against-schema` dominates at ~0.5 s).

**Ordering hazard (spec edge case, normative):** on THIS repo the gate
stays RED until the operator executes the store repair (live store
currently fails `layout-root-canonical` — that is correct behavior).
Repair-before-gate-wiring is the mandated order; use scratch stores for
the gate's own three-state test, with `CYBEROS_HOST_MOUNT_PREFIX` set
when seeding them under `/tmp`.

## 5. Deviations from the spec (all deliberate, all carve-out-driven)

1. **`run-gates.sh` doctor gate (§1.6, AC 6) — not implemented here.**
   Gates worker's slice. My side of the contract: standalone entrypoint +
   `--store`-after-subcommand + the §4 contract above. `tools/install/tests/test_doctor_gate.sh`
   (spec `new_files`) belongs with that slice.
2. **`build.sh` (§1.1 vendoring arrow, §1.3 INTEROP vendoring) — not edited here.**
   Install worker's slice; observed already landed in the shared tree
   (schema vendored from `cyberos/data/`, INTEROP.md vendored beside it).
   My tests pin both from the reading side and stay valid whichever copy
   the script references, as long as content is canonical.
3. **Live-store repair (§1.5, AC 5) — not executed.** Operator-gated by
   the spec itself; `store-repair-plan.md` + the /tmp-copy proof + the
   fixture regression test are the implementation-side deliverables. The
   plan also runs TASK-MEMORY-261's decision step (disposition table) for
   ratification at the HITL gate, per the spec's "261 unshipped" branch.
4. **Root `CHANGELOG.md` untouched; module `modules/memory/CHANGELOG.md`
   carries the §1.8 entry.** The root file is outside this worker's
   ownership set and is release-versioned (top entry `[1.1.0]`, a shipped
   release — a post-1.1.0 task entry needs an Unreleased section the
   final pass should own). AC 8's test pins the module changelog.
5. **Scratch-payload build legs of AC 1 / AC 3 asserted at the source**
   (build.sh reference resolution + content identity) rather than by
   executing `build.sh` — running the install worker's script mid-batch
   from this slice would race their edits. Final pass: one payload build
   closes the loop end-to-end.

## 6. Open items for the HITL reviewer

1. **Approve + execute `store-repair-plan.md`** (this folder). Step-2
   hashes are volatile while TASK-MEMORY-302 is unfixed — re-measure at
   execution (the plan aborts on mismatch). After execution, re-run
   `cyberos doctor` + `cyberos verify` on the live store and record both
   outputs at the gate (AC 5's live half).
2. **Ratify the 261 disposition** embedded in the plan (ADR → `memories/decisions/`,
   impl-plan → `memories/projects/`), or reject and have the plan re-issued —
   the two-dir disposition is deliberately the narrowest possible slice of 261.
3. **Re-contamination risk:** the 302 applier re-wrote the stray files
   at 10:59 today (measured mid-implementation — hashes churned within
   the hour). Landing TASK-MEMORY-302 promptly after the repair keeps
   the store green; until then a re-run of the affected appliers will
   re-freeze the store.
4. **Detector tombstone fix** (§3): correctness fix inside my ownership,
   but it grazes TASK-MEMORY-115/116 scope — flag to the reviewer in case
   they want it recorded as a footnote on those tasks.
5. **Final-pass verification:** one scratch payload build (AC 1/AC 3
   payload legs) + the gates worker's AC 6 three-state test + full
   `run_all.sh`, per the batch's final sequential pass.
6. **Installed-tree staleness:** `.cyberos/memory/` (gitignored, vendored)
   still carries the pre-unification schema until the next
   install/update run — expected per §0.4's refresh rule, noting for
   completeness.
