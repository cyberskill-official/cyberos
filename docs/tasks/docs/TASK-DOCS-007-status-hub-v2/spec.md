---
id: TASK-DOCS-007
title: "Status hub v2 - dashboard UI (module cards, progress bars, chips) + zero-touch HTML regeneration on every task/backlog/version change"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: improvement
created_at: 2026-07-12T00:00:00+07:00
department: engineering
author: "@stephencheng"
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
related_tasks: [TASK-DOCS-006, TASK-TPL-001, TASK-IMP-068]
depends_on: [TASK-DOCS-006]
blocks: []
source_pages:
  - modules/templates/html/status-hub.html
  - tools/docs-site/render-status-hub.mjs
  - .githooks/pre-commit
source_decisions:
  - "2026-07-12 operator feedback with reference HTML: status page too simple / wall of text - adopt the reference's dashboard language (segmented progress bars, task chips, module cards, callout, tick phases) on CDS tokens."
  - "2026-07-12 operator requirement: task/backlog/version changes MUST regenerate the HTML automatically - no manual trigger anywhere."
language: html + javascript (node stdlib) + bash + yaml
service: tools/docs-site/
new_files: []
modified_files:
  - modules/templates/html/status-hub.html
  - modules/templates/contracts/TEMPLATE.md
  - tools/docs-site/render-status-hub.mjs
  - tools/docs-site/tests/test_render_status_hub.sh
  - .githooks/pre-commit
  - .github/workflows/deploy.yml
  - .github/workflows/version.yml
---

# TASK-DOCS-007: Status hub v2 + zero-touch regeneration

## §1 - Description

Normative clauses:

1. The status-hub@1 shell MUST adopt the operator-reference dashboard language on CDS tokens: umber header band with ochre rule (title/subtitle/meta), an overall-progress card with a SEGMENTED bar (done=success green, in-flight=ochre, on-hold=muted) + count chips, a generated "Now shipping" callout listing in-flight tasks as chips, and a color legend. New slots (subtitle, meta:html, now:html, legend:html) are additive to the contract table.
2. The Roadmap tab MUST replace the 10-column board with MODULE CARDS: one card per module carrying a per-module segmented minibar, % done, and one chip per task (short id, status-colored, title+status tooltip, linked to its page); in-flight modules get the ochre card accent; the invalid-status bucket stays visible as its own card; the rollup table remains below.
3. The Changelog tab MUST render releases as tick-circle rows (newest = ochre "now", older = green done). The Backlog tab keeps its four facets with status rendered as chips.
4. Zero-touch regeneration MUST hold on all three surfaces: (a) local - the pre-commit hook regenerates dist/website whenever staged changes touch docs/tasks/**, docs/**, module docs, modules/templates/**, tools/docs-site/**, CHANGELOG.md, or VERSION (warn-not-block on build failure, skip with warning when node is absent); (b) published on push - deploy.yml's docs job path filters gain VERSION and modules/templates/**; (c) published on version bump - version.yml dispatches deploy.yml after pushing the [skip ci] bump commit, so the hub's VERSION stamp never goes stale (warn on dispatch failure).
5. Styling stays token-only (hex solely in vendored cds/*.css) and the page self-contained; suites keep the TASK-DOCS-006 assertions updated to the v2 structure.

## §2 - Why this design

The reference page communicates in bars and chips where v1 used columns of text; mapping it onto CDS tokens keeps one visual language. The three regeneration surfaces close every staleness path: the author's machine (hook), content pushes (paths), and the one commit class that suppresses CI ([skip ci] bumps -> explicit dispatch).

## §3 - Contract

Slot table (status-hub@1, additive): title, subtitle, meta:html, deck:html, now:html, legend:html, tab_*:html, footer. Chip grammar: span.chip.(done|active|hold|todo) with data-status, wrapped in a.chip-link when the task page exists.

## §4 - Acceptance criteria

1. **Dashboard structure renders** (§1 #1, #2) - fixture: overall bar with 50.0% done segment + count chips; module card grid with minibars and linked chips; callout appears when in-flight tasks exist.
2. **Changelog ticks + backlog chips** (§1 #3) - newest release row carries the "now" tick; the status column renders chips.
3. **Zero-touch surfaces wired** (§1 #4) - the hook contains the docs trigger + build call; deploy.yml paths carry VERSION + modules/templates/**; version.yml dispatches deploy.yml after a successful bump push.
4. **Token-clean + suites green** (§1 #5) - no hex outside the token block; hub + legacy + templates suites pass on the v2 structure.

## §5 - Verification

test_render_status_hub.sh (t01/t06 updated) + test_render_roadmap.sh (legacy asserts preserved) + test_templates_module.sh + wiring greps recorded in the ship record.

## §6 - Implementation skeleton

Shell v2 + builder fragment rewrite (single corpus object unchanged); hook block appended; two workflow edits.

## §7 - Dependencies

TASK-DOCS-006 (the hub it restyles). TASK-IMP-068's hook infrastructure hosts the new trigger.

## §8 - Example payloads

Summary line unchanged: `status-hub: 491 tasks (...), 20 releases, VERSION 0.2.0 (deck+3 tabs)`.

## §9 - Open questions

None blocking. Per-task history sparklines need event data the corpus does not carry - future scope.

## §10 - Failure modes inventory

1. Hook slows commits - the site build is seconds-scale; warn-not-block keeps commits unblocked on failure.
2. Dispatch loop risk - version.yml dispatches deploy.yml only (deploy never bumps VERSION); acyclic.
3. node absent on an operator machine - hook warns and skips; published surfaces unaffected.
4. Chip flood on giant modules (memory: 85) - 11.5px pills in narrow grid columns; acceptable past 100/module, pagination is future scope.
5. Reference hexes leaking - mapped to tokens at authoring; the hex greps enforce.

## §11 - Implementation notes

Chips show short ids (task- prefix stripped) for density; full id + title + status live in the tooltip.

**Post-ship amendment (2026-07-12, TASK-IMP-071):** §1 #4c's deploy-dispatch workaround is retired -
with [skip ci] gone from bump commits, deploy.yml's VERSION path filter fires natively on the bump
push. Surfaces (a) and (b) unchanged.

*End of TASK-DOCS-007.*
