---
id: TASK-IMP-062
title: "Quarterly envelope review ritual"
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
refs: [Stage, 5]
depends_on: [TASK-IMP-027]
created: 2026-07-08
verify: T
service: docs/runbooks
owner: Stephen Cheng (CTO)
---
# TASK-IMP-062: Quarterly envelope review ritual

## Summary

Author a quarterly checklist + calendar reminder stub for reviewing the
docs/skills envelope (Stage 5). Remains manual — IMP-027 auto-mode is closed
won't-do for 1.x; dependency retained historically and allowlisted for the
wiki-link gate.

## Problem

Without a ritual, allow/deny edges and judge anchors drift until an incident.

## Proposed Solution

`docs/runbooks/quarterly-envelope-review.md` with checklist + calendar stub +
BACKLOG pointer.

## Alternatives Considered

- Automate dream replay now — rejected: 1.x payload scope; doc-first.
- Drop because IMP-027 closed — rejected: ritual still valuable manually.

## Success Metrics

- Runbook exists with checklist + calendar fields + ADR cross-links.

## Scope

Doc + backlog pointer only. No cron/automation.

## AI Authorship Disclosure

- Session agent; HITL Stephen Cheng (session operator).

## 1. Description (normative)

- 1.1 MUST add `docs/runbooks/quarterly-envelope-review.md` with purpose,
  calendar reminder stub, and operator checklist.
- 1.2 MUST point BACKLOG or batch notes at the runbook so agents can find it.

## 2. Acceptance criteria

- [x] AC 1 (traces_to: #1.1) - runbook present with checklist + calendar stub
- [x] AC 2 (traces_to: #1.2) - BACKLOG carries a pointer line to the runbook

## 3. Edge cases

- `depends_on: TASK-IMP-027` remains even though 027 is closed (historical edge).
