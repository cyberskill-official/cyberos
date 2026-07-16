---
artefact: implementation-plan@1
task_id: TASK-IMP-092
created: 2026-07-17
estimate_pts: 2
verdict: pass (implementation-plan-audit: every matrix row addressed by a slice, context-map patterns respected, estimate sane vs spec effort_hours 4)
---
# Implementation plan - TASK-IMP-092

Slices (each maps to §1 clauses and edge-case-matrix rows):
1. retallyHeader in tools/install/docs-tools/backlog-mutate.mjs - replaces
   updateHeaderCounts(raw, deltas). Gate on parseCountsHeader (bare/prose headers
   untouched), tally parseRow hits from the header to the next `## ` line (the
   section's contiguous row block in the regenerated layout; placeholder parses as
   no row; non-enum statuses uncounted, regen-aligned), empty tally -> null (never
   `()`), render `${prefix}  (${rendered})` in STATUS_ORDER with zero counts
   omitted and CR carried. Call sites: cmdFlip nearestHeaderAbove, cmdInsert
   target.header - the only header the mutation was ever allowed to touch. The
   `cur + d < 0` refuse-to-go-negative guard dies with the deltas: the retally
   cannot go negative because it counts, and a header that "never counted this
   row" is exactly the lie the task exists to correct. Message verb becomes
   "header retallied"; envelope fields unchanged. Header docs + --help updated to
   describe the retally (t07's help asserts preserved). (§1 #1.1, #1.3; rows
   2-7, 10-11.)
2. Suite updates for the retally expectation - t06's four header asserts move
   from incremental arithmetic to true tallies (the fixture's alpha '2 done' and
   beta '17 done' are phantoms the first mutation now corrects - faithful update,
   disclosed in code-review.md); t09's workflow_version pin 2.6.2 -> 2.6.3; suite
   header comments extended for t06's new meaning and t10-t12. (§1 #1.1 guardrail,
   AC 3.)
3. t10_retally_corrects_lying_header + t11_footprint_holds_with_retally over a new
   emit_lying_backlog fixture (alpha `(34 done)` vs rows 1 draft/1 implementing/
   2 done - the incident in miniature; omega `(9 draft)` over a placeholder):
   flip AND insert each rewrite the header to the true tally, statuses the header
   never listed join in lifecycle order, the placeholder insert drops phantom
   counts to the sole real status, the --json envelope names old/new header; the
   diff footprint is asserted line-by-line (removed = old header + old row, added
   = new header + new row; insert 1+2; bare header exactly 1+1). (§1 #1.2, #1.3;
   rows 1, 3-5, 12.)
4. Doctrine + vendor proof - ship-tasks.md workflow_version 2.6.3; §11a swarm
   sub-bullet: shared files owned by ONE writer through ONE filesystem view per
   run, cone-independence includes view-independence, lost-update mechanics named,
   TASK-IMP-086 attribution; §9 testing-phase paragraph: acceptance evidence for
   content deliverables measured on the committed object (`git show
   <commit>:<path>`), never a working view. t12 greps both passages + the version
   in the SOURCE and in the scratch payload's cuo/ship-tasks.md (ensure_payload =
   `bash tools/install/build.sh "$TMP/payload"`). (§1 #1.4, #1.5; row 9.)

Pattern conformance (context-map): the retally lives inside the existing line
model (stripCR/crOf/parseRow/parseCountsHeader all reused unchanged), rendering
identical to the incremental path's, node stdlib only, determinism preserved;
suite grows in the file's own idiom (emit_* fixture, want-gated scenarios,
ensure_payload reuse). Out of scope honored: no post-commit parity guard, no
filesystem changes, no Totals-line maintenance, no new sections.

Estimate: 2 pts (~4 h) - matches spec effort_hours: 4. Actual landed surface:
3 modified files, 0 new (backlog-mutate.mjs ~63-line diff net +27, suite +~150
lines incl. 3 scenarios + fixture, ship-tasks.md +2 passages +version), suite
12/12 in ~2.8 s including two payload builds and a scratch install.
