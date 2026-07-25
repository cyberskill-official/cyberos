---
id: TASK-IMP-044
title: "Automated dependency updates (Dependabot)"
template: task@1
type: improvement
module: improvement
status: done
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-08T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-IMP-001]
blocks: []
related_tasks: [TASK-IMP-001, TASK-IMP-043]
routed_back_count: 0
awh: N/A
verify: T
phase: "Wave 4 - hardening"
owner: Stephen Cheng (CTO)
created: 2026-07-08
effort_hours: 2
draft_reason: authoring
entered_via: audit
service: .github
new_files:
  - .github/dependabot.yml
  - tools/install/tests/test_dependabot.sh
modified_files: []
source_pages:
  - "docs/strategy/cyberos-deep-audit-and-auto-evolution-plan-2026-07-06.md R26"
source_decisions:
  - "2026-07-25 session operator (batch/12b): Dependabot npm+actions core, cargo secondary; merge requires existing CI — no invented awh merge conditions."
---

# TASK-IMP-044: Automated dependency updates (Dependabot)

## Summary

Weekly Dependabot for npm install tooling, GitHub Actions, and secondary cargo under /services. Merge remains existing CI + human acceptance.

## Problem

Without automated update PRs, npm and Actions drift. Inventing a fake awh merge condition would be dishonest.

## Proposed Solution

.github/dependabot.yml with weekly npm (two dirs), github-actions, and cargo (/services), with groups. Tests refuse fabricated awh merge keys.

## Alternatives Considered

- Renovate. Deferred; Dependabot sufficient for 1.x.
- Auto-merge on green. Rejected for this batch.

## Success Metrics

- Primary: dependabot.yml present; tests green.
- Guardrail: no fabricated awh merge predicate.

## Scope

In scope: dependabot.yml + test + merge-honesty note.

### Out of scope / Non-Goals

- Auto-merge.
- Inventing tools/docs-site package.json for Dependabot.

## Dependencies

TASK-IMP-001.

## AI Authorship Disclosure

- **Tools used:** Composer (Cursor agent) under batch/12b operator mandate.
- **Scope:** Dependabot weekly npm + actions (+ cargo secondary).
  **re-derived and CONFIRMED:** no dependabot.yml at HEAD; R26 names Dependabot/Renovate.
  **re-derived and CORRECTED:** do not invent awh-only merge condition.
  **measured and ADDED:** docs-tools becomes a Dependabot target via IMP-001 package.json.
- **Human review:** Operator session HITL override for batch/12b; evidence under docs/batches/.

## 1. Description (normative)

- 1.1 .github/dependabot.yml MUST declare weekly npm updates for /tools/install/mcp and /tools/install/docs-tools.
- 1.2 The same file MUST declare weekly github-actions updates for /.
- 1.3 The same file MUST declare weekly cargo updates for /services (secondary; may use a lower open-pull-requests-limit).
- 1.4 npm and github-actions update PRs MUST use Dependabot groups where supported.
- 1.5 Documentation MUST state that merge still requires existing CI gates and human acceptance — Dependabot MUST NOT be configured with an invented awh-only merge condition.

## 2. Acceptance criteria

- [x] AC 1 (traces_to: #1.1) - npm dirs — test: `tools/install/tests/test_dependabot.sh::t_npm_dirs`
- [x] AC 2 (traces_to: #1.2) - actions — test: `tools/install/tests/test_dependabot.sh::t_actions_ecosystem`
- [x] AC 3 (traces_to: #1.3) - cargo — test: `tools/install/tests/test_dependabot.sh::t_cargo_ecosystem`
- [x] AC 4 (traces_to: #1.4) - groups — test: `tools/install/tests/test_dependabot.sh::t_groups_present`
- [x] AC 5 (traces_to: #1.5) - no fake awh — test: `tools/install/tests/test_dependabot.sh::t_no_fake_awh_merge`

## 3. Edge cases

- Empty dependency trees still benefit (Actions bumps still flow).
