---
id: TASK-IMP-055
title: "Spec-only module manifest"
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
refs: [R9]
depends_on: []
created: 2026-07-08
verify: T
service: docs/modules
owner: Stephen Cheng (CTO)
---
# TASK-IMP-055: Spec-only module manifest

## Summary

Publish an honest machine-readable + human table of which `modules/*` trees
ship code vs docs/skills-only (deep-audit R9).

## Problem

Many module folders look like packages but only carry CHANGELOG/audit stubs.
Agents invent import paths that do not exist.

## Proposed Solution

`modules/manifest.yaml` + `docs/modules/MANIFEST.md` listing every top-level
module with `ships_code` or `docs_skills_only`.

## Alternatives Considered

- Delete docs-only module folders — rejected: they still anchor task modules.
- Generate-only with no human table — rejected: agents need a readable index.

## Success Metrics

- Manifest lists every `modules/*` directory exactly once with a kind.
- Kinds match on-disk reality for cuo/memory/skill/templates vs the rest.

## Scope

In scope: yaml + markdown manifest. Out of scope: rewriting module READMEs.

## AI Authorship Disclosure

- Session agent; HITL Stephen Cheng (session operator).

## 1. Description (normative)

- 1.1 MUST add `modules/manifest.yaml` with `version`, per-module `id` + `kind`.
- 1.2 MUST add `docs/modules/MANIFEST.md` human table cross-linking the yaml.
- 1.3 MUST classify `cuo`, `memory`, `skill`, `templates` as `ships_code` and
  the remaining top-level modules as `docs_skills_only` for CyberOS 1.x.

## 2. Acceptance criteria

- [x] AC 1 (traces_to: #1.1–#1.3) - yaml + md present; module set matches `modules/*`
- [x] AC 2 (traces_to: #1.3) - four ships_code ids are exactly cuo/memory/skill/templates

## 3. Edge cases

- `tools/install/mcp/` is not a `modules/mcp` code tree — note in manifest.
