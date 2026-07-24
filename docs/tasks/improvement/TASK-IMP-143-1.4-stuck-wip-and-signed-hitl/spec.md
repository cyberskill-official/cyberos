---
id: TASK-IMP-143
title: "1.4.x — stuck-WIP hub sentinel + signed HITL verdict artifacts"
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
depends_on: [TASK-IMP-140, TASK-CUO-303]
blocks: []
related_tasks: [TASK-IMP-139, TASK-IMP-144]
routed_back_count: 0
owner: Stephen Cheng (CTO)
created: 2026-07-23
---

# TASK-IMP-143: 1.4.x stuck-WIP hub + signed HITL (draft)

CyberOS stays on the **1.x** line. This milestone is the next minor wave after 1.3.0 (formerly mislabeled "v3.x" in early roadmap notes).

## Scope (draft — author full ACs before promoting)

1. **Stuck-WIP status-hub sentinel** — promote G13's report-only detector (`scripts/tests/test_benchmark_gates.sh::t_g13`) into a visible status-hub surface (threshold N=30 days) with operator triage links.
2. **Signed/attributed HITL verdict artifacts** — replace honor-system `--verdict-by` actor strings with attributed verdict files (signature or at least content-addressed actor+timestamp+evidence hash). Named Non-Goal of TASK-CUO-303.

## Non-goals

- Transition-locked state engine (TASK-IMP-144 / 1.5.0).
- Changing the two HITL gate transitions themselves.
