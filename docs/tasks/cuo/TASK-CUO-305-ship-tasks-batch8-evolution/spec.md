---
id: TASK-CUO-305
title: "ship-tasks evolution — batch/8 HITL + sub-batch doctrine"
template: task@1
type: improvement
module: cuo
status: done
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-23T18:40:00+00:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-CUO-303]
blocks: []
related_tasks: [TASK-IMP-084, TASK-IMP-120, TASK-IMP-141]
routed_back_count: 0
owner: Stephen Cheng (CTO)
created: 2026-07-23
---

# TASK-CUO-305: ship-tasks evolution from batch/8 friction

## Problem

Shipping batch/8 surfaced repeated HITL/ops friction (documented in `docs/batches/batch-8a-ship-notes.md` … `8c`):

- Verdict evidence under `docs/batches/` must carry YAML frontmatter or the status hub breaks.
- Sub-batch vs parent ledger doctrine was undefined.
- Shared "approve all" evidence for N flips worked but was undocumented.
- Truth-precedes-index flip order was easy to miss mid-flight.

## Fix

Fold those candidates into `modules/cuo/chief-technology-officer/workflows/ship-tasks.md` HITL section: gated-flip checklist, batch evidence sanction, sub-batch ledger rules.

## Acceptance

1. ship-tasks.md documents the four-step gated-flip checklist.
2. Shared evidence for batch "approve all" is explicitly sanctioned.
3. Sub-batch / parent ledger relationship is stated.
4. No behavior change to backlog-mutate exit codes.
