---
artefact: edge-case-matrix@1
task_id: TASK-IMP-092
total_rows: 12
created: 2026-07-17
verdict: pass (edge-case-matrix-audit: every category >=1 row, covered-by names real test functions, SECURITY rows point at code+test, DEGRADATION rows carry detection+recovery)
---
# Edge-case matrix - TASK-IMP-092

All test functions live in tools/install/tests/test_workflow_helpers.sh.

| # | category | trigger | expected behavior | covered by |
|---|---|---|---|---|
| 1 | null/empty | placeholder-only section under a phantom count (`## omega  (9 draft)` over `- (nothing remaining)`) receives its first insert | header drops to the sole real status `(1 done)` - never a zero entry, never a negative, the phantom is corrected not decremented | t10_retally_corrects_lying_header (omega arm) |
| 2 | null/empty | a counted header whose section tallies to ZERO rows (hypothetical removal - this tool never removes rows, so unreachable post-mutation) | retallyHeader returns null and the header stays untouched: never `## name  ()` (defensive guard, code-pinned `if (tally.size === 0) return null`) | inspection (backlog-mutate.mjs retallyHeader) + t10 omega pre-insert state exercises the same section shape |
| 3 | bounds | inherited count far from truth (`34 done` vs real 2 - the incident preserved a 14-off header through six mutations) | ONE mutation of any kind rewrites the full truth; correction cost is O(1) mutations, not O(drift) | t10_retally_corrects_lying_header (flip AND insert arms) |
| 4 | bounds | section rows in statuses the header never listed (the incident's exact shape: unindexed statuses masked by inherited counts) | retally introduces them in lifecycle order (draft, ready_to_implement, implementing, ready_to_review, reviewing, ready_to_test, testing, done, on_hold, closed, cannot_reproduce, duplicate) | t10 (flip arm: implementing + ready_to_review join ordered) + t06 (lifecycle order across flips/inserts) |
| 5 | malformed | bare header `## gamma` (no counts) above the mutated row | untouched - parseCountsHeader gates the retally exactly as it gated the incremental adjust; the mutation's diff is the one row | t05 (gamma insert, header asserted bare) + t11 (bare-header flip footprint 1/1) |
| 6 | malformed | header with prose parens or single-space `## name (1 draft)` (not the regen grammar) | not a counts header (regex demands two spaces + enum statuses + `$`); never rewritten - the tool cannot "adopt" a header the regenerator did not write | inspection (parseCountsHeader :125-138, unchanged by this task) + t05 no-counts arm |
| 7 | malformed | row with a status token outside the enum (`- [bogus] ...`) inside the section | not counted (STATUS_ORDER filter), matching regen_backlog() which only ever emits enum statuses; known-status counts stay true | inspection (retallyHeader filter) - non-enum rows are regenerator-impossible; flip/insert cannot create one (status args validated, exit 2) |
| 8 | concurrency/order | two identical runs on identical input (text and --json) | byte-identical files and envelopes - the retally reads only the post-mutation lines array, no clock/randomness | t07_json_and_determinism (c1/c2 cmp, f1/f2 cmp) |
| 9 | concurrency/order | the incident class itself: a second WRITER on another filesystem view | out of the tool's reach by design - doctrine closes it: one writer through one view per run (§11a), evidence on the committed object (§9); the retally bounds the damage by making the next single-writer mutation self-correct the index | t12_doctrine_view_rules_vendored (both passages, source + payload) + t10 (self-correction) |
| 10 | SECURITY | title bytes shaped like counts or headers (`... - Title (3 done)` or a smuggled `## ` via title) trying to steer the retally | impossible by grammar: rows are only read via parseRow (whole-line anchored), headers only rewritten when the exact header regex matches the LINE; newline injection into titles is already exit 2 (row-injection guard), so no title can become a line | t05 (row-injection guard) + inspection (parseRow/parseCountsHeader are ^...$ anchored) |
| 11 | DEGRADATION | CRLF backlog: the corrected header must not drop or gain a CR | crOf(lines[h]) carried onto the rewritten header; whole-file CR count unchanged after flip; inserted rows inherit the section ending | t07 (CRLF arms: count preserved, flipped + inserted rows keep \r) |
| 12 | DEGRADATION | a large header correction bloating the mutation's diff past the declared footprint | diff stays exactly 1 row + at most 1 header line, asserted line-by-line (removed = {old header, old row}, added = {new header, new row}); detection: t11 on every suite run; recovery: any footprint regression fails the gate before it can ship | t11_footprint_holds_with_retally |

Documented-by-design: the file-top `Totals:` line is never touched (t06 asserts it verbatim - repo-wide totals remain regen_backlog()'s job); the retally scans to the next `## ` header rather than stopping at the first row-block gap, which in the regenerated layout is the same contiguous block and in a degenerate layout counts ALL of the section's rows instead of silently missing trailing ones (matches the 086 repair's own scan, gate-log E-SPLICE/E3). SECURITY-class: no new execution surface - same imports, same write path, prose + tally arithmetic only.
