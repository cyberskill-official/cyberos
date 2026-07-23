---
id: TASK-IMP-142
title: "Schedule MCP/OBS resume wave after IMP-139 triage"
template: task@1
type: improvement
module: improvement
status: done
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-23T18:40:00+00:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-IMP-139]
blocks: []
related_tasks: [TASK-MCP-003, TASK-MCP-005, TASK-MCP-006, TASK-MCP-007, TASK-MCP-008, TASK-OBS-001, TASK-OBS-003, TASK-OBS-005, TASK-OBS-007, TASK-OBS-008, TASK-OBS-009, TASK-APP-001]
routed_back_count: 0
owner: Stephen Cheng (CTO)
created: 2026-07-23
---

# TASK-IMP-142: MCP/OBS + APP-001 resume schedule

## Context

IMP-139 Gate-2 applied dossier recommendations: **11 route_back** (MCP-003/005/006/007/008, OBS-001/003/005/007/008/009) → `ready_to_implement`, and **TASK-APP-001 resume** → `implementing`. Leaving them idle recreates the stuck-WIP class G13 detects.

## Deliverable (this task)

Author the schedule only (no implementation of the MCP/OBS work itself):

1. Record the three sub-batches in [`docs/batches/batch-9-post-120-followups.md`](../../batches/batch-9-post-120-followups.md) Wave 3 section (already drafted).
2. Preconditions: IMP-141 + MEMORY-302 `done` before starting 9a.
3. Per-cluster intent: re-spec/adopt against measured tree paths (`services/mcp-gateway/`, `services/shared/`), not abandoned claimed layouts.

## Acceptance

1. batch-9 ledger lists 9a-mcp / 9b-obs / 9c-app with member IDs.
2. This task's status can advance once the schedule is linked from the parent batch ledger (no code changes required beyond docs).
