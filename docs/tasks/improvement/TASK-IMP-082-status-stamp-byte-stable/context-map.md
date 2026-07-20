---
artefact: repo-context-map@1
task_id: TASK-IMP-082
created: 2026-07-16
verdict: pass (repo-context-map-audit: patterns pinned to file:line, outside-domain count stated, ADR trigger evaluated)
---
# Repo context map - TASK-IMP-082

## Baseline patterns the new code must follow
- renderer style: node stdlib only, no child processes, no wall clock; honest failures via `die()` - pinned_in: tools/docs-site/render-status-hub.mjs:8 (header contract), :36 (die)
- input discovery: docs/tasks/<module>/<TASK-*>/spec.md, existsSync-guarded, module/task dirs sorted - pinned_in: tools/docs-site/render-status-hub.mjs:122-129
- optional-input guards: CHANGELOG.md and VERSION read only `when present`, lenient mode degrades - pinned_in: tools/docs-site/render-status-hub.mjs:216-218 (clPath), :291-293 (VERSION)
- stamp surfaces (all read the one COMMIT const): cs-data JSON `commit` field - pinned_in: tools/docs-site/render-status-hub.mjs:400; header meta `built from` - :454; footer `generated at ... (COMMIT)` - :463
- explanatory comment voice: prose paragraphs that name the failure the code prevents - pinned_in: tools/docs-site/render-status-hub.mjs:294-300 (the new stamp comment follows the voice of the old :304-306 one it replaces)
- test harness: self-contained bash, `set -uo pipefail`, PASS/FAIL counters, ok/fail helpers, `pass=N fail=N` summary, exit non-zero on fail, mktemp fixture with trap cleanup - pinned_in: tools/docs-site/tests/test_render_status_hub.sh:6-12, scripts/tests/test_task_layout.sh:3-8
- fixture shape: scratch corpus with modules/templates copied from the repo, printf'd spec.md/CHANGELOG.md/VERSION - pinned_in: tools/docs-site/tests/test_render_status_hub.sh:14-25 (mkfix)
- production invocation contract: `node render-status-hub.mjs <root> <out>` with CYBEROS_HUB_LENIENT=1, CYBEROS_PAGE_ASSETS=1, CYBEROS_TEMPLATES - pinned_in: tools/install/lib/task-migrate.sh:55-61, reached via tools/install/lib/status-page.sh:28
- explicit-pin consumer that must keep working: audit-fleet freshness re-render pins CYBEROS_COMMIT to the page's own recorded stamp and compares content - pinned_in: tools/install/audit-fleet.sh:174-181

## Schemas / interfaces in scope
- Stamp value grammar: default `fp-` + first 12 lowercase hex of sha256 (spec §1.1); `CYBEROS_COMMIT` verbatim when set and non-empty (§1.2). The `fp-` prefix keeps fingerprints distinguishable from git shas.
- Hash input order (normative): every discovered spec.md's raw bytes in bytewise-sorted repo-relative path order, then CHANGELOG.md raw bytes when present, then VERSION raw bytes when present. The rendered page is never an input.
- run_all discovery: `scripts/tests/test_*.sh` glob - pinned_in: scripts/tests/run_all.sh:47 - the new suite is picked up with zero wiring.

## Files outside the immediate domain (tools/docs-site/ + scripts/tests/)
1. tools/docs-site/tests/test_render_status_hub.sh (modified - one assertion; it pinned the fake-HEAD sha the fixture plants, i.e. the exact behavior this task removes)
2. tools/docs-site/tests/test_render_roadmap.sh (modified - same one-assertion update)

files_outside_immediate_domain: 2 (<= 3 -> no ADR trigger; both are test-only, same-tool assertions updated to the new stamp grammar, no production surface outside the renderer is touched). Consumer repos inherit through the existing payload vendoring (tools/install/build.sh:171) with no call-site changes - the spec's "one changed default beats N remembered call sites" decision.

## Blast radius
file_count: 4 (1 renderer, 1 new suite, 2 peer-suite assertion updates) | module_count: 2 (tools/docs-site, scripts/tests) | cross_module_edges: scripts/tests suite -> tools/docs-site renderer + modules/templates fixtures; renderer -> none new (node:crypto is stdlib) module_placement_warning: null (task declares service tools/docs-site; the suite location scripts/tests/test_render_stamp.sh is fixed by spec §1.8) Behavioral radius: every status-page render everywhere (cyberos repo, hook flows, consumer repos via payload). The only observable change is the stamp VALUE on the three surfaces; layout, lenses, chunks, KPI math untouched (spec Out of scope). audit-fleet's pinned re-render is unaffected because the pin path is preserved verbatim.
