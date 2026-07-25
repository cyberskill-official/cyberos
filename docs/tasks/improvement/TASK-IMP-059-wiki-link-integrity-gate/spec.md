---
id: TASK-IMP-059
title: "Wiki link-integrity gate"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: improvement
created_at: 2026-07-08T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: improvement
priority: p1
status: done
phase: Wave 5 - platform and process
refs: [R51]
depends_on: []
related_tasks: []
created: 2026-07-08
verify: T
service: tools/install/docs-tools
owner: Stephen Cheng (CTO)
---
# TASK-IMP-059: Wiki link-integrity gate

## Summary

Add a docs gate that fails on broken relative markdown links and missing
TASK- cross-refs in frontmatter, so the corpus agents plan from cannot rot
silently (deep-audit R51).

## Problem

Task READMEs and specs accumulate dead relative links and `depends_on` edges
pointing at tasks that never existed or were renumbered. Agents treat those
paths as real.

## Proposed Solution

Ship `tools/install/docs-tools/wiki-link-check.mjs` (+ allowlist), wire it into
the install test suite and payload vendor copy, and keep the live `docs/` scan
green via allowlist for known historical index debt.

## Alternatives Considered

- Only warn (non-zero never) — rejected: green theatre.
- Fail on every bare prose `TASK-` mention — rejected: specs cite future work;
  scope is links + frontmatter edges.

## Success Metrics

- Suite `test_wiki_link_check.sh` pass=6 fail=0.
- Live `docs/` scan exits 0 with committed allowlist.

## Scope

In scope: checker, allowlist, test suite, `build.sh` vendor lines.

### Out of scope / Non-Goals

- Rewriting every historical module README index (allowlisted; follow-up sweep).
- HTML docs-site link checking.

## AI Authorship Disclosure

- **Tools used:** Composer session agent.
- **Human review:** session HITL with Stephen Cheng (session operator).

## 1. Description (normative)

- 1.1 MUST provide `wiki-link-check.mjs` that scans `docs/**/*.md` (excluding
  `_archive/`), fails on broken relative non-http file links, skips `.html`
  targets, and supports an allowlist file.
- 1.2 MUST fail when `depends_on` / `blocks` / `related_tasks` frontmatter on
  `spec.md` cites a `TASK-XXX` id with no matching `docs/tasks/**/TASK-XXX*`
  folder, unless allowlisted.
- 1.3 MUST ship `tools/install/tests/test_wiki_link_check.sh` covering help,
  pass/fail fixtures, allowlist, live scan, and build vendor grep.
- 1.4 MUST vendor the checker (+ allowlist) from `tools/install/build.sh`.

## 2. Acceptance criteria

- [x] AC 1 (traces_to: #1.1, #1.2) - checker exits 1 on broken link / missing TASK; 0 when clean
- [x] AC 2 (traces_to: #1.3) - `test_wiki_link_check.sh` pass=6 fail=0
- [x] AC 3 (traces_to: #1.4) - `build.sh` copies `wiki-link-check.mjs` and allowlist

## 3. Edge cases

- Pure `#anchor` links are not file targets.
- Allowlist keys: `file -> target`, bare `TASK-ID`, or whole-file path.
