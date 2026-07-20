---
artefact: repo-context-map@1
task_id: TASK-IMP-089
created: 2026-07-17
verdict: pass (repo-context-map-audit: patterns pinned to file:line, outside-domain count stated, ADR trigger evaluated)
---
# Repo context map - TASK-IMP-089

## Baseline patterns the new code must follow
- template split: TASK-TEMPLATE.md is the ENGINEERING half only - frontmatter + `## 1. Description` / `## 2. Acceptance criteria` / `## 3. Edge cases` / invariants; it carries no `## Scope`. The PRD half (Summary ... Scope ... Dependencies) lives in the per-type templates - pinned_in: tools/install/templates/TASK-TEMPLATE.md:34-61 (post-change), modules/skill/contracts/task/templates/feature.md:36-94 (whose `## Scope` is prose guidance, no H3 - so an "exactly one `### Out of scope / Non-Goals`" assert would be FALSE against the actual files; the truthful assert is: no out-of-scope H2 in TASK-TEMPLATE.md at all)
- single out-of-scope home: `## Scope > ### Out of scope / Non-Goals` is the rubric's home (SEC-006/QA-006 never required section 4) - pinned_in: spec.md Summary + IMP-07 decision rows in `source_pages`/`source_decisions`
- suite style: self-contained bash, `set -uo pipefail`, repo-root cd, shared PASS/FAIL counters via ok()/fail(), scenario-named labels, `pass=N fail=N` summary, non-zero exit on fail - pinned_in: scripts/tests/test_template_schema.sh:23-28 and the runner lines at the bottom
- WHY-comment doctrine: every scenario carries the incident that motivates it - pinned_in: test_template_schema.sh:1-22 (header), t07's named-path lesson (:97-110): a filter that silently matches nothing is indistinguishable from a real absence
- scratch-build contract: `build.sh <out>` accepts an output dir (`out="${1:-$repo/dist/cyberos}"`, build.sh:16), rm -rf's it, and vendors `tools/install/templates/.` verbatim into `<out>/cuo/templates/` (`cp -R`, build.sh:36) - so byte-parity source<->payload is the correct post-build assert
- payload-sync doctrine: dist/ is rebuilt by the batch parent before commit; task-level suites build to SCRATCH and never mutate dist/ - pinned_in: TASK-IMP-084 code-review.md "Diff size" note + t07's dist-optional guard (test_template_schema.sh t07 "payload not built here; not this test's job")
- tmp hygiene: `mktemp -d` + `trap 'rm -rf "$TMP"' EXIT` - pinned_in: tools/install/tests/test_rubrics_vendored.sh harness shape (via IMP-084 context-map row)

## Schemas / interfaces in scope
- Target TASK-TEMPLATE.md tail: `## 3. Edge cases` -> `## 4. Protected invariants this task must not weaken` (body byte-unchanged from old section 5); zero out-of-scope H2s (numbered or not); nothing at `## 5.`
- shape oracle `shape_why(file)`: echoes reason tokens (`duplicate-out-of-scope-H2`, `invariants-not-at-##4`, `stray-##5-heading`); empty = conforming. Shared by all three t08 arms so live template, reintroduction fixture and payload copy are judged by ONE rule set. `^## ` cannot match `###`, so the PRD H3 home stays out of reach by construction.
- consumer entry point: install.sh:742 quickstart tells every new repo `cp .cyberos/cuo/templates/TASK-TEMPLATE.md .../spec.md` - the file is the FIRST artifact a new project touches (t06's original incident).

## Files outside the immediate domain (tools/install/templates/ + scripts/tests/)
none. The reference sweep (grep for "Protected invariants", `## 4./## 5.` numbered headings and "out of scope" H2s across tools/, modules/, scripts/, README.md) hits ONLY TASK-TEMPLATE.md itself. Near-misses ruled out: `tools/install/tests/test_*.sh:2` headers cite "TASK-XXX-NNN §5 suite" - those are the citing tasks' own historical spec sections (docs/tasks corpus, excluded by spec §3); `tools/install/README.md:122/135` "### 4./### 5." number distribution channels, not template sections.

files_outside_immediate_domain: 0 (<= 3 -> no ADR trigger).

## Blast radius
file_count: 2 modified, 0 new (TASK-TEMPLATE.md +1/-5; test_template_schema.sh +61/-0) | module_count: 2 (tools/install templates, scripts/tests) | cross_module_edges: suite -> build.sh CLI contract (read-only scratch use); template -> every future consumer-repo first task via the payload module_placement_warning: null (spec declares `service: tools/install/templates`; the touched files are exactly the spec's `modified_files`) Behavioral radius: prompt-text only - no executable path changes. Existing specs keep both shapes (the rubric accepts both; t08 targets the TEMPLATE, never the corpus - spec §3). Consumer repos inherit the single-home template through the payload on their next install; dist/ refresh is the batch parent's rebuild step and until then dist/ carries the pre-change copy (t07 checks per-type presence only, so it stays green).
