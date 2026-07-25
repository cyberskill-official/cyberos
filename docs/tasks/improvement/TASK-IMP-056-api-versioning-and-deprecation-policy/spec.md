---
id: TASK-IMP-056
title: "API versioning and deprecation policy"
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
refs: [R10]
depends_on: []
created: 2026-07-08
verify: T
service: docs/governance
owner: Stephen Cheng (CTO)
---
# TASK-IMP-056: API versioning and deprecation policy

## Summary

Write a short versioning + deprecation policy for CyberOS 1.x **payload**
HTTP/MCP/CLI surfaces (R10), not platform `/v1` service routes.

## Problem

Silent breaking changes on MCP tools or CLI verbs become incidents once a
second consumer appears.

## Proposed Solution

`docs/governance/api-versioning.md` covering MCP tools, install CLI verbs,
docs-tools exit contracts, and workflow enums — with a deprecation window.

## Alternatives Considered

- Policy for all monorepo `/v1` routes — rejected: platform scope (ADR-003).
- SemVer tooling automation — rejected: doc policy first.

## Success Metrics

- Policy names in-scope surfaces and a deprecation checklist.

## Scope

Governance doc only.

## AI Authorship Disclosure

- Session agent; HITL Stephen Cheng (session operator).

## 1. Description (normative)

- 1.1 MUST add `docs/governance/api-versioning.md` scoped to payload MCP/CLI/
  docs-tools/workflow surfaces.
- 1.2 MUST define additive-vs-breaking rules and a minimum deprecation window.
- 1.3 MUST explicitly mark platform service `/v1` routes out of scope.

## 2. Acceptance criteria

- [x] AC 1 (traces_to: #1.1–#1.3) - policy file exists with scope table + deprecation checklist

## 3. Edge cases

- Security emergency may shorten the window only with an ADR + release note.
