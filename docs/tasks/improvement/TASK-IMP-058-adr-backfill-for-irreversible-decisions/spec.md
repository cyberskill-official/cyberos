---
id: TASK-IMP-058
title: "ADR backfill for irreversible decisions"
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
refs: [R50]
depends_on: []
created: 2026-07-08
verify: T
service: docs/adrs
owner: Stephen Cheng (CTO)
---
# TASK-IMP-058: ADR backfill for irreversible decisions

## Summary

Backfill ADRs for irreversible calls agents keep re-litigating: own-chat, AGE
removal, payload vs platform, rules_sha cone, HITL gates (R50 + batch/12d).

## Problem

Strategy docs bury decisions; future agents re-propose rejected paths.

## Proposed Solution

Five accepted ADRs under `docs/adrs/` plus an index README; thin `docs/adr/`
alias pointer. Cross-link DEC rows when present; do not invent a fake ledger.

## Alternatives Considered

- Only update strategy essays — rejected: not citable as ADR artefacts.
- Include RouterBackend/SSO in the same five — deferred; own-chat/AGE/payload/
  rules_sha/HITL were the operator-selected set for this batch.

## Success Metrics

- Index lists ≥5 new ADRs as accepted with stable filenames.

## Scope

In scope: ADR bodies + index + `docs/adr/` pointer. Out of scope: full DEC
ledger migration.

## AI Authorship Disclosure

- Session agent; HITL Stephen Cheng (session operator).

## 1. Description (normative)

- 1.1 MUST add ADRs for: own-chat-from-scratch; AGE removal; payload vs
  platform; rules_sha cone; HITL gates.
- 1.2 MUST add/update `docs/adrs/README.md` index linking them.
- 1.3 MUST provide `docs/adr/README.md` pointing at `docs/adrs/` (singular path).

## 2. Acceptance criteria

- [x] AC 1 (traces_to: #1.1) - five ADR files exist with `status: accepted`
- [x] AC 2 (traces_to: #1.2, #1.3) - both README indexes present and linked

## 3. Edge cases

- Existing ADR-IMP-068 / ADR-OBS-003 remain; index includes them.
