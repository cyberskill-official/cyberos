---
fr_id: FR-DOCS-003
audited: 2026-07-12
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

# FR-DOCS-003 audit

## §1 - Verdict summary

Audited for doctrine fit with FR-DOCS-002 (generated, deterministic, dependency-free, never committed) and for the integrity of a page that visualizes the FR corpus itself. The timestamp/determinism conflict and the silent-drop of invalid statuses were the two real defects; both closed. Traceability closes over t01-t08 in tools/docs-site/tests/test_render_roadmap.sh.

## §2 - Findings (all resolved)

### ISS-001 timestamp broke byte-identical rebuilds
A generated-at wall-clock stamp violates the FR-DOCS-002 determinism property the page inherits. Resolved: §1 #5 stamp = VERSION + commit only, AC 5 double-build byte equality.

### ISS-002 counting from BACKLOG.md inherited a stale index
The backlog's hand-maintained totals were the session's OWN discovered defect; a roadmap built on them repeats it. Resolved: §1 #2 counts derived from frontmatter only, AC 2 fixture tally.

### ISS-003 invalid statuses were silently dropped
The first cut's board simply omitted rows with out-of-enum statuses (the FR-EVAL-001 class), hiding data-quality problems. Resolved: visible `invalid` bucket + stderr warning while parse failures still fail the build (§10 #2) - monitoring without enforcement theater.

### ISS-004 JS-off users got a blank board
Filters implemented as render-time exclusion would empty the page without JS. Resolved: §1 #3 all content in the DOM, filters hide, AC 3 no-JS assertion.

### ISS-005 CHANGELOG structure drift would render an empty timeline
Zero parsed sections looked like a quiet release history. Resolved: §1 #5 empty parse fails the build, AC 6.

### ISS-006 "trigger on every release/deploy" was under-wired
Only the deploy path was hooked in the first cut; a release tag would not refresh the public page. Resolved: §1 #4 wires BOTH workflows to the same publish target, AC 4 structural asserts on all three files.

## §3 - Resolution

All six findings addressed as cited. Depends on FR-DOCS-002 (reviewing) as declared - this queues immediately behind it. **Score = 10/10.**

*End of FR-DOCS-003 audit.*

## §4 - Ship record (2026-07-12)

- Implementation: render-roadmap.mjs (stdlib, 3 inputs, 4 blocks, deterministic stamp, inline vanilla
  filtering, token styling) + build.sh step + render-docs nav hook + release.yml docs job (same VPS
  target as deploy.yml); commit 3747f4c. Phase artefacts: docs/feature-requests/.workflow/FR-DOCS-003/.
- Review: human verdict at gate 1 APPROVE + pre-authorize done (Stephen Cheng, in-chat).
- Testing: test_render_roadmap.sh 8/8 (one per AC), 8/8 repo suites, live build green
  (486 FRs / 18 releases / VERSION 0.1.0, page in nav). Gate 2 recorded per pre-authorization.
- Field finding queued: regen_backlog read_fm silently skips 42 yaml-invalid FRs (444 vs 486) -
  next-batch FR to make the skip loud and repair the files. The roadmap's invalid-status bucket is
  live as the corpus data-quality monitor (§11).

Verdict unchanged: PASS, Score = 10/10.
