---
id: TASK-IMP-041
title: "Secrets inventory and rotation runbook"
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
phase: Wave 4 - hardening
refs: [R22]
depends_on: []
created: 2026-07-08
verify: T
service: docs/runbooks
owner: Stephen Cheng (CTO)
---
# TASK-IMP-041: Secrets inventory and rotation runbook

## Summary

Author a secrets inventory (class, location, owner, blast radius) plus
rotate-on-leak steps (R22). Never document secret values.

## Problem

The gam updater-key leak showed missing standing process. CI and deploy use
many GitHub Actions secrets without a single inventory.

## Proposed Solution

`docs/runbooks/secrets-inventory-and-rotation.md` derived from workflow secret
names and known VPS/npm classes.

## Alternatives Considered

- Close as platform won't-do — rejected: inventory is docs and payload-safe.
- Dump actual values into an encrypted store in-repo — rejected: out of scope /
  dangerous.

## Success Metrics

- Table covers Actions secrets referenced in `.github/workflows/*` plus VPS /
  npm / BRAIN classes.
- Rotate-on-leak procedure is copy-pasteable.

## Scope

Runbook only. No secret creation/rotation execution in this task.

## AI Authorship Disclosure

- Session agent; HITL Stephen Cheng (session operator).

## 1. Description (normative)

- 1.1 MUST add `docs/runbooks/secrets-inventory-and-rotation.md` with a class
  inventory table (no values).
- 1.2 MUST include a rotate-on-leak procedure (contain → inventory → replace →
  verify → record).
- 1.3 MUST cover at least: GitHub Actions deploy/release secrets, VPS SSH,
  version-bump deploy key, store signing classes, npm token class.

## 2. Acceptance criteria

- [x] AC 1 (traces_to: #1.1–#1.3) - runbook present; names real workflow secret
      classes without values; rotation steps present

## 3. Edge cases

- If a class is unused, still list it with owner "confirm before use".
