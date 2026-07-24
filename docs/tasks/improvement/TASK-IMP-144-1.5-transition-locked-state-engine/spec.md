---
id: TASK-IMP-144
title: "1.5.0 — transition-locked state engine (close frontmatter bypass)"
template: task@1
type: improvement
module: improvement
status: draft
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-23T18:40:00+00:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-CUO-303, TASK-IMP-143]
blocks: []
related_tasks: [TASK-IMP-120]
routed_back_count: 0
owner: Stephen Cheng (CTO)
created: 2026-07-23
---

# TASK-IMP-144: 1.5.0 transition-locked state engine (draft)

CyberOS stays on the **1.x** line. This milestone follows 1.4.x (formerly mislabeled "v4.0" in early roadmap notes).

## Problem

TASK-CUO-303 mechanically locks `backlog-mutate` for the two HITL transitions, but an agent can still edit `spec.md` frontmatter `status:` and regenerate BACKLOG — R-EXT-01 residual. ship-tasks.md records this as the accepted residual until a transition-locked engine lands.

## Scope (draft)

1. Single state-engine API owns every status cell mutation (frontmatter + index) with the same transition table and HITL gates.
2. Regenerators refuse to invent transition edges; they only reflect engine-committed state.
3. Capability-scoped agent identities (memory ACL + backlog writes) as a follow-on milestone inside 1.5.x.

## Non-goals

- Replacing ship-tasks skill chain.
- Cross-tool conformance kit (separate 1.5.x track).
