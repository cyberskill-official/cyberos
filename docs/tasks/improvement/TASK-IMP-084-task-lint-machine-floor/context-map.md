---
artefact: repo-context-map@1
task_id: TASK-IMP-084
created: 2026-07-16
verdict: pass (repo-context-map-audit: patterns pinned to file:line, outside-domain count stated, ADR trigger evaluated)
---
# Repo context map - TASK-IMP-084

## Baseline patterns the new code must follow
- docs-tools convention: node stdlib only, ESM, single self-contained .mjs, honest loud failures - pinned_in: tools/docs-site/md.mjs and tools/docs-site/render-status-hub.mjs (the two peers already vendored into the payload's docs-tools/), spec Alternatives (js-yaml explicitly rejected)
- payload vendoring shape: per-file `[ -f ... ] && cp` guarded lines inside the docs-tools block - pinned_in: tools/install/build.sh:165-178; the new lint's line follows that exact shape at build.sh:173-174
- install lay-down: the whole payload `docs-tools/` dir is copied verbatim into `.cyberos/docs-tools/` - pinned_in: tools/install/install.sh:59-61 - so vendoring into the payload is the ONLY new plumbing needed
- skill propagation: modules/skill/task-audit -> payload cuo/skills/task-audit (build.sh:128-131) -> plugin/skills/task-audit (build.sh:211-217); editing the modules/ source propagates to both payload copies on rebuild
- rule source of truth: modules/skill/task-audit/RUBRIC.md §1-§2 (FM-001..114 with the exact enums), §3 (SEC-001..009), §4 (COND-001..004), §9 (TRACE-001..003 structural halves), §10 (template detection -> `template_ambiguous`); rule_ids appear verbatim in findings per RUBRIC.md:3 ("Rule IDs MUST appear verbatim in audit reports so reports are diffable")
- FM-104 status enum: the 12-value set including `cannot_reproduce` and `duplicate` per modules/skill/contracts/task/STATUS-REFERENCE.md §1 (RUBRIC §2's FM-104 row points there; FM-113 exists precisely because `duplicate` carries a link)
- test harness shape: self-contained bash, `set -uo pipefail`, here/repo resolution, mktemp TMP with trap cleanup, ok/fail counters, `pass=N fail=N` summary, non-zero exit on fail - pinned_in: tools/install/tests/test_rubrics_vendored.sh:12-17; payload-build-then-scratch-install pattern - :19-20 and :37-44
- suite discovery: scripts/tests/run_all.sh:43 globs `tools/install/tests/test_*.sh` - the new suite is picked up with zero wiring
- determinism doctrine: byte-identical output on identical input, no wall clock, no env text, sorted iteration - pinned_in: task-audit SKILL.md determinism block (`reproducible: true`, `deterministic_drift` anomaly signal) and the batch's TASK-IMP-082 byte-stability precedent

## Schemas / interfaces in scope
- CLI: `node task-lint.mjs [--json] <spec.md|dir ...>`; directories recurse to `*/spec.md` in bytewise path order.
- Finding grammar (normative, spec §1.6): `SEVERITY rule_id file:line message`, one per line, bytewise sorted; severities `error|warning|info`; `--json` emits the same findings, same order, as an array of `{severity, rule_id, file, line, message}`. Exit 0 iff zero error-severity findings, else 2 (§1.7); `info` never flips the exit code.
- Strict YAML subset FM-001 accepts: flat `key: value`, single/double-quoted strings (with `\"` escapes - TASK-IMP-083's spec uses them), inline `[a, b]` lists, block `- item` lists, own-line comments, blank lines. Anchors, aliases, block scalars, nested/flow collections, trailing comments -> FM-001 naming the line.
- Deployment paths (the two the skill wiring names): `tools/install/docs-tools/task-lint.mjs` in the platform repo; `.cyberos/docs-tools/task-lint.mjs` in installed repos.

## Files outside the immediate domain (tools/install/docs-tools/ + tools/install/tests/)
1. modules/skill/task-audit/SKILL.md (modified, +4 lines - the §1 #1.8 normative lint-first wiring; spec-declared in `modified_files`)
2. tools/install/build.sh (modified, +2 lines - the guarded vendor copy; required by §1 #1.9's payload gate and NOT listed in the spec's `modified_files`, disclosed in code-review.md - the spec's own source_pages row cites build.sh:165-171 as the vendoring point, but that block copies named files only, so "lands in the payload automatically" needed one line to become true)

files_outside_immediate_domain: 2 (<= 3 -> no ADR trigger; one is spec-declared skill prose, the other a two-line vendoring ripple inside the exact block the spec's source_pages already anchors).

## Blast radius
file_count: 4 (2 new: lint + suite; 2 modified: SKILL.md +4, build.sh +2) | module_count: 3 (tools/install docs-tools+tests, tools/install build, modules/skill/task-audit) | cross_module_edges: suite -> build.sh + install.sh + payload skill copies (t07/t08); lint -> RUBRIC.md at authoring time only (rules are compiled in; no runtime read of the rubric) module_placement_warning: null (spec declares `service: tools/install/docs-tools`; both new files sit where §1 #1.1/#1.9 fix them) Behavioral radius: zero on existing production paths - the lint runs only when invoked (task-audit loop step, manual CLI, or a future per-repo gate; run-gates wiring is explicitly out of scope). The SKILL.md passage binds every FUTURE task-audit invocation to run the lint first when present; consumer repos inherit tool + skill text through the payload on their next install. The three batch-1 specs lint clean today (t06), so introducing the floor breaks no green state.
