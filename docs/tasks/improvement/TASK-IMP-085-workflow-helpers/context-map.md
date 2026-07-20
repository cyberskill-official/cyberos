---
artefact: repo-context-map@1
task_id: TASK-IMP-085
created: 2026-07-16
verdict: pass (repo-context-map-audit: patterns pinned to file:line, outside-domain count stated, ADR trigger evaluated)
---
# Repo context map - TASK-IMP-085

## Baseline patterns the new code must follow
- docs-tools convention: node stdlib only, ESM, single self-contained .mjs, honest loud failures, `--json` as data - pinned_in: tools/install/docs-tools/task-lint.mjs (the batch-1 peer this task sits beside), tools/docs-site/md.mjs. The spec's Alternatives row rejects python siblings explicitly (consumer repos are guaranteed node by the MCP/status tooling, not python3 versions).
- contract source 1 (ship-manifest@1): modules/cuo/chief-technology-officer/workflows/ship-tasks.md "Resume semantics" (staleness order, two-phase writes, pins, the mandated resume line, terminal handling) + modules/skill/contracts/task/SHIP-MANIFEST.md (field table, 1..31 step indices, hitl shape). Python peer: modules/cuo/cuo/ship_manifest.py - `validate`/`write_atomic`/`resume_plan`/`select_next`/`finalize`; its `write_atomic` (tempfile + os.replace, fsync) is the two-phase idiom the .mjs mirrors, and its `resume_plan` fixes the semantics of "done|skipped-conditional counts as resolved" that the .mjs `walk()` reuses.
- contract source 2 (backlog-state-update@2): modules/skill/backlog-state-update-author/SKILL.md §2 (old_line pre-image, mutation_kind enum, insert payload with `expected_absent`), §2b (regenerator-identical grammar + stem-ascending placement + whole-file discipline), §3 (byte-for-byte optimistic concurrency, "never moves rows"). Byte-authority for row grammar and header counts: `regen_backlog()` in scripts/migrate_improvement_to_task.py:168-217 - row `- [<status>] <stem> - <title>` with ` (improvement)` tag (line 205), header `## {mod}  ({N status, ...})` in STATUS_ORDER (lines 198-199), placeholder `- (nothing remaining)` (line 207), stem-primary sort (line 203).
- payload vendoring shape: per-file `[ -f ... ] && cp` guarded lines inside build.sh's docs-tools block - pinned_in: tools/install/build.sh:165-181; the two new lines sit at build.sh:175-177, directly under the task-lint copy (the 084 lesson: this block copies NAMED files, nothing globs the source dir, so a new tool that skips build.sh never reaches any consumer).
- install lay-down: the whole payload `docs-tools/` dir is copied verbatim into `.cyberos/docs-tools/` - pinned_in: tools/install/install.sh:61 - so the guarded build copies are the ONLY new plumbing.
- workflow-doc propagation: modules/.../ship-tasks.md -> payload `cuo/ship-tasks.md` (build.sh:28) -> plugin copy `plugin/skills/ship-tasks/cuo/ship-tasks.md` (build.sh:203-204); editing the modules/ source reaches both payload copies on rebuild (t09 gates all three).
- test harness shape: self-contained bash, `set -uo pipefail`, here/repo resolution, mktemp TMP + trap cleanup, ok/fail counters, `pass=N fail=N` summary, per-scenario `want` filter, cached `ensure_payload` build-then-scratch-install - pinned_in: tools/install/tests/test_task_lint.sh:24-32,243-246.
- suite discovery: scripts/tests/run_all.sh:43 globs `tools/install/tests/test_*.sh` - the new suite is picked up with zero wiring (AC 10's ops check).
- determinism doctrine: byte-identical output on identical input; where a contract REQUIRES timestamps (ship-manifest step entries), the clock is injectable (env `CYBEROS_NOW` / `--now`) rather than the doctrine weakened - precedent: task-lint's no-clock rule + the memory protocol's pinned-time test fixtures.

## Schemas / interfaces in scope
- `node ship-manifest.mjs [--json] [--root <dir>] init|record|verify|resume-line|delete ...` on `docs/tasks/.workflow/<task-id>.ship.json`; exits 0/2/3/4/5 (staleness order); resume line format is the workflow's mandated string verbatim; step entries `{index, skill, status, artefact_path, artefact_sha256, verdict, completed_at}`.
- `node backlog-mutate.mjs [--json] [--root <dir>] flip|insert ...` on `docs/tasks/BACKLOG.md` (or `--backlog`); exits 0/2/6/7; a mutation is one row plus at most one section-header line; CRLF and every byte outside the mutation round-trip.
- Deployment paths: `tools/install/docs-tools/*.mjs` in the platform repo; `.cyberos/docs-tools/*.mjs` in installed repos (both named by the two new doctrine sentences in ship-tasks.md).

## Files outside the immediate domain (tools/install/docs-tools/ + tools/install/tests/)
1. tools/install/build.sh (modified, +3 lines - the guarded vendor copies required by §1 #1.7; spec-declared in `modified_files`)
2. modules/cuo/chief-technology-officer/workflows/ship-tasks.md (modified, +9/-1 - the two §1 #1.8 pointer passages plus the workflow_version 2.6.1 -> 2.6.2 bump; spec-declared in `modified_files`; the bump is disclosed in code-review.md)

files_outside_immediate_domain: 2 (<= 3 -> no ADR trigger; both are spec-declared).

## Blast radius
file_count: 5 (3 new: two tools + suite; 2 modified: build.sh +3, ship-tasks.md +9/-1) | module_count: 3 (tools/install docs-tools+tests, tools/install build, modules/cuo workflow doc) | cross_module_edges: suite -> build.sh + install.sh + payload cuo/plugin ship-tasks.md copies (t08/t09); tools -> the two contracts at authoring time (semantics are encoded, never redefined - spec §5) plus a RUNTIME parse of the workflow doc's frontmatter for version auto-discovery and the step->skill chain (doc-driven by construction: the doc changes, the tool follows). module_placement_warning: null (spec declares `service: tools/install/docs-tools`; all three new files sit where §1 #1.1/#1.9 fix them). Behavioral radius: zero on existing production paths - both tools run only when invoked. The workflow_version bump means a manifest pinned at 2.6.1 verifies as exit 3 (needs_human) on next resume - which is the contract working as written, not a regression (mixed-version runs are exactly what the pin exists to stop); no manifests are committed (gitignored session state). Cone-disjoint from batch siblings: the suite mutates only fixture backlogs under mktemp TMP, never docs/tasks/BACKLOG.md (TASK-IMP-086's file) or docs/release/ (TASK-IMP-087's).
