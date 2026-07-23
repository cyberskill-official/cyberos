---
id: TASK-IMP-141
title: "MMR sync for memory-append — doctor stays READY after gated flips"
template: task@1
type: improvement
module: improvement
status: done
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-23T18:40:00+00:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-CUO-303, TASK-MEMORY-303, TASK-IMP-140]
routed_back_count: 0
owner: Stephen Cheng (CTO)
created: 2026-07-23
---

# TASK-IMP-141: MMR sync for memory-append

## Problem

Every gated HITL flip (`reviewing→ready_to_test`, `testing→done`) appends a `status_overridden` row via `tools/install/docs-tools/memory-append.mjs`. That tool advanced the binlog + HEAD but did **not** update `audit/mmr/peaks.bin`. Doctor `ledger-mmr-cross-check` then went RED, and with CUO-302's doctor gate fail-closed, `run-gates.sh` failed until a human rebuilt peaks from the binlog (batch-8b/8c ship notes).

## Fix

After every successful append, rebuild `audit/mmr/peaks.bin` from every on-disk payload using the same peak-stack algorithm as `modules/memory/cyberos/core/mmr.py` (Node stdlib only — docs-tools convention). Catch-up heals a previously stale peaks file.

## Acceptance

1. `bash tools/install/tests/test_memory_append.sh` includes `t05_mmr_peaks_stay_in_sync` and passes.
2. After N appends, peaks `leaf_count` equals HEAD.
3. A deliberately stale peaks.bin is healed by the next append.
4. CHANGELOG Unreleased notes the fix.
