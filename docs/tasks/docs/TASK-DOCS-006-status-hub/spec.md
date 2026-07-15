---
id: TASK-DOCS-006
title: "Status hub - one status.html: command deck + Roadmap | Backlog | Changelog tabs (supersedes roadmap.html)"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-07-12T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: docs
priority: p0
status: done
verify: T
phase: Wave D - visual deliverables
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: 2026-07-12
memory_chain_hash: null
related_tasks: [TASK-TPL-001, TASK-DOCS-003, TASK-DOCS-005]
depends_on: [TASK-TPL-001]
blocks: []
source_pages:
  - tools/docs-site/render-roadmap.mjs
  - modules/templates/html/status-hub.html
source_decisions:
  - "2026-07-12 operator decision: backlog + changelog + roadmap merge into one comprehensive-status HTML; display = command deck + 3 hash-routed tabs (chosen over plain tabs and single-scroll dashboard)."
language: javascript (node stdlib) + html
service: tools/docs-site/
new_files:
  - tools/docs-site/render-status-hub.mjs
  - tools/docs-site/tests/test_render_status_hub.sh
modified_files:
  - tools/docs-site/build.sh
  - tools/docs-site/render-docs.mjs
  - tools/docs-site/tests/test_render_roadmap.sh
---

# TASK-DOCS-006: Status hub

## §1 - Description

One page answers "where is the project" completely: a persistent deck of headline numbers over three deep-linkable tabs.

Normative clauses:

1. A builder `tools/docs-site/render-status-hub.mjs` MUST emit `dist/website/reference/status.html` through the `status-hub@1` template from the SAME three inputs as TASK-DOCS-003 (task frontmatter, CHANGELOG sections, VERSION) - node stdlib only, deterministic stamp (VERSION + commit, no wall clock).
2. The command deck MUST always show: VERSION, built-from commit, total tasks, per-status counts (10-value enum order), latest release (version + date), and module count - visible regardless of active tab.
3. Three tabs MUST render: **Roadmap** (default; the TASK-DOCS-003 board + module rollups, rows linking to task pages when TASK-DOCS-005 pages exist), **Backlog** (filterable table: id, title, module, class, priority, status - module/class/status/priority facets; BACKLOG.md stays the agents' write-path index, this tab is the human view), **Changelog** (the release timeline). Tab routing via URL hash (`#roadmap`, `#backlog`, `#changelog`) with hash-change handling; no JS -> all three panels present in the DOM and the tab bar degrades to in-page anchor links.
4. `roadmap.html` MUST become a redirect stub to `status.html#roadmap` (meta refresh + link, no JS required); the shared nav's entry MUST become "Status" pointing at status.html; TASK-DOCS-003's suite is repointed at the hub builder with its assertions preserved (board counts, determinism, honest failures, token-clean) - the superseding is recorded on TASK-DOCS-003 as a post-ship amendment.
5. Inline vanilla JS only, CDS tokens only (template rule), same honest failures as TASK-DOCS-003 §1 #5 (unparseable task names file; zero CHANGELOG sections fails).

## §2 - Why this design

The deck answers 90% of "how are we doing" without a click; tabs keep each dense surface uncrowded; hash routing makes every view shareable. Superseding roadmap.html instead of keeping two pages avoids a split status story.

## §3 - Contract

Summary stdout: `status-hub: N tasks, R releases, VERSION X (deck+3 tabs)`. Page: `reference/status.html`, template `status-hub@1`.

## §4 - Acceptance criteria

1. **Deck totals true** (§1 #2) - fixture corpus: deck numbers equal frontmatter/CHANGELOG/VERSION-derived truth.
2. **Three tabs + routing + degrade** (§1 #3) - emitted HTML carries three panels in DOM; hash script present; anchors work JS-free (structural asserts).
3. **Backlog tab facets** (§1 #3) - rows carry data attrs for all four facets; filter script wired.
4. **Supersession clean** (§1 #4) - roadmap.html is a stub pointing at status.html#roadmap; nav says Status; repointed legacy suite passes.
5. **Determinism + honesty + tokens** (§1 #1, #5) - byte-identical rebuilds; failure fixtures fail loud; no hex outside token block.
6. **task-page links** (§1 #3) - board/backlog rows href `frs/<module>/<STEM>/` when TASK-DOCS-005 is present in the build (fixture with pages asserts hrefs).

## §5 - Verification

`tools/docs-site/tests/test_render_status_hub.sh`: t01_deck_true, t02_tabs_routing_degrade, t03_backlog_facets, t04_supersession, t05_deterministic_honest_tokens, t06_fr_links. (AC 1-6.)

## §6 - Implementation skeleton

Port render-roadmap's parsers; add backlog-table + deck renderers; substitute status-hub.html slots; stub roadmap.html; nav refItems swap; repoint old suite.

## §7 - Dependencies

TASK-TPL-001 (shell). Soft: TASK-DOCS-005 (links activate when pages exist; builder feature-detects the frs/ output dir).

## §8 - Example payloads

`status-hub: 486 tasks, 18 releases, VERSION 0.1.0 (deck+3 tabs)`

## §9 - Open questions

None blocking. A fourth "Releases assets" tab is future scope.

## §10 - Failure modes inventory

1. Tab state lost on reload - hash IS the state; default applied only when hash absent.
2. Deck/tab count drift - both derive from one parsed corpus object; single source in code.
3. roadmap.html bookmarks break - stub redirect keeps them alive indefinitely.
4. Backlog tab tempts editing-by-hand expectations - tab header states it is a generated view; BACKLOG.md remains the write path.
5. JS-off filtering absent - full corpus visible unfiltered; documented degrade per §1 #3.

## §11 - Implementation notes

Keep TASK-DOCS-003's greppable prefixes alive in the new summary line; deploy logs already watch them.

*End of TASK-DOCS-006.*
