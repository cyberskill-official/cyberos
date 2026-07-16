---
artefact: repo-context-map@1
task_id: TASK-IMP-092
created: 2026-07-17
verdict: pass (repo-context-map-audit: patterns pinned to file:line, outside-domain count stated, ADR trigger evaluated)
---
# Repo context map - TASK-IMP-092

## Baseline patterns the new code must follow
- byte-discipline executor conventions: line model split/join on '\n' with CR carried per line (stripCR/crOf, backlog-mutate.mjs:93-94), row grammar `- [<status>] <stem> - <title>` (parseRow, :97-100), two-phase atomic writes, node stdlib only, deterministic output - the retally must live INSIDE this model, not beside it
- counts-header grammar authority: parseCountsHeader accepts exactly `## <name>  (<N status>(, <N status>)*)` - two spaces before the paren, statuses from the STATUS_ORDER enum, no duplicates (backlog-mutate.mjs:125-138); anything else is not a counts header and is never edited. The retally reuses this parser unchanged for both the "is this header counted" decision and the prefix
- rendering convention: `${prefix}  (${rendered})` with zero-count statuses omitted and statuses in STATUS_ORDER (the first ten are regen_backlog()'s order; backlog-mutate.mjs:60-63) - the incremental adjust already rendered this way, so the retally changes WHERE the numbers come from, not how they print
- whole-file discipline: one row plus at most one section-header line per mutation (file header doc + t07 diff proofs) - the retally may only rewrite the header the mutation was already allowed to touch (flip: nearestHeaderAbove; insert: target.header)
- the incident's own repair as prior art: TASK-IMP-086 gate-log CORRECTIVE ADDENDUM + E-SPLICE recompute the header "from ALL rows in the section in the file's own STATUS_ORDER" (gate-log-draft.md:106-113) - the retally is that repair promoted into the tool so it happens on every mutation instead of once in an emergency
- suite harness shape: emit_backlog fixtures, ok/fail counters, `pass=N fail=N` summary, scenario selection via `want`, ensure_payload builds once into $TMP/payload (test_workflow_helpers.sh:40-102, 390-393); run_all.sh discovers `tools/install/tests/test_*.sh` with zero wiring
- doctrine placement: §11a swarm sub-bullets carry operational learnings with "(Learned on the 2026-07-16 ...)" attributions; §9 testing-phase prose carries evidence rules; version-tagged passages "(v2.6.x, TASK-...)"; workflow_version in frontmatter is suite-pinned (t09)
- payload propagation: build.sh copies ship-tasks.md to payload cuo/ and plugin/skills/ship-tasks/cuo/; docs-tools/*.mjs vendored byte-identically (t08 cmp) - editing modules/ + tools/install/ sources is the ONLY plumbing needed

## Schemas / interfaces in scope
- retallyHeader(lines, h): parse header at h via parseCountsHeader (null -> untouched); tally parseRow hits from h+1 to the next `## ` line or EOF (in the regenerated layout that is exactly the section's contiguous row block; the `- (nothing remaining)` placeholder parses as no row); statuses outside STATUS_ORDER are not counted (regen_backlog() alignment); empty tally -> null (never `()`); render in lifecycle order, zero counts omitted, CR preserved via crOf
- call sites: cmdFlip (nearest header above the flipped row) and cmdInsert (the target section's header); JSON envelope keeps header_line/old_header/new_header; message verb becomes "header retallied at line N"
- doctrine passages: §11a one-writer-one-view sub-bullet under Swarm execution; §9 committed-object evidence paragraph after the TRACE-004 prose; workflow_version 2.6.2 -> 2.6.3

## Files outside the immediate domain (tools/install/docs-tools/ + tools/install/tests/)
1. modules/cuo/chief-technology-officer/workflows/ship-tasks.md (modified, +2 passages +version bump - spec-declared in `modified_files`)

files_outside_immediate_domain: 1 (<= 3 -> no ADR trigger; the one file is spec-declared doctrine prose).

## Blast radius
file_count: 3 modified, 0 new (backlog-mutate.mjs ~63 line-diff, test_workflow_helpers.sh +~150/-~20, ship-tasks.md +3/-1) | module_count: 2 (tools/install docs-tools+tests, modules/cuo workflow) | cross_module_edges: suite -> build.sh payload (t08/t09/t12, read-only use); tool -> regen_backlog() grammar (encoded, never redefined)
module_placement_warning: null (spec declares `service: tools/install/docs-tools`; every touched file is spec-declared)
Behavioral radius: every future flip/insert on a counted section rewrites that header to the section's true tally - a pre-existing header lie is corrected by the FIRST mutation instead of preserved forever (the 086 incident's 34-vs-20 could not survive one flip now). Bare headers, uncounted sections, the Totals line, and all refusal paths (exit 2/6/7) are byte-for-byte unchanged. Consumer repos inherit tool + doctrine through the payload on their next install; t06's expectations moved from incremental to retally semantics (disclosed in code-review.md).
