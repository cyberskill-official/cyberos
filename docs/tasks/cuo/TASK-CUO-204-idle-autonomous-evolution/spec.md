---
template: task@1
id: TASK-CUO-204
title: "Idle-time autonomous evolution - the dream loop under the AWH gate"
author: "@stephen"
department: engineering
status: done
priority: p2
created_at: "2026-06-22T10:30:00+07:00"
ai_authorship: assisted
feature_type: internal_tooling
eu_ai_act_risk_class: high
target_release: 2026-Q4
client_visible: false
module: cuo
new_files:
  - modules/cuo/cuo/core/dream_loop.py
  - modules/cuo/cuo/core/evolution_envelope.py
  - modules/cuo/tests/test_dream_loop.py
  - modules/cuo/tests/test_evolution_envelope.py
  - modules/cuo/config/dream.yaml
depends_on: [TASK-CUO-200, TASK-CUO-201, TASK-CUO-202, TASK-CUO-203]
---

# Task

> Turn Your Will Into Real.

## Summary

CyberOS should keep improving itself while no one is using it, without ever risking an unsafe self-modification. This adds an idle-time loop - the "dream loop" - that runs the existing self-evolution cycle (TASK-CUO-201/202/203) on its own when no human is active. The loop proposes refinements to skill prompts, workflow step ordering, and thresholds, runs them through the AWH gate (author, test, gate), and applies a change only if it passes the gate, falls inside an explicit evolution envelope, and is classified low-risk. Anything that touches a security invariant halts and waits for a human. Every applied change is reversible, lands on a dedicated branch for review, and the whole loop has a kill switch.

## Problem

The self-evolution harness (TASK-CUO-200 through 203) can already spot that a skill fails too often or that a workflow's shape causes repeat route-backs, and it can propose and apply low-risk fixes. But today a human triggers each run and approves each apply. Stephen wants the system to improve while he is offline - to use idle time productively - while being certain it cannot quietly change something that matters for safety (cross-tenant isolation, the audit chain, auth, PII recall, cost math).

Without a bounded autonomous loop, idle time is wasted and every improvement waits on Stephen's attention. Without a hard envelope and a human checkpoint, an autonomous loop is unsafe. The two needs have to be met together: autonomy inside a fence, never outside it.

## Proposed Solution

A scheduled idle-time loop that reuses the TASK-CUO-201/202/203 machinery, bounded by an explicit envelope and a human checkpoint. User-visible behaviour: when the machine has been idle for a set period, the loop wakes, proposes and tests refinements against the golden sets, applies the ones that pass and are in-envelope and low-risk, and leaves a branch plus an audit trail for Stephen to review in the morning. Anything risky is left as a halted proposal, not applied.

### Section 1 - normative requirements (BCP-14)

1. The loop MUST start only when an idle detector reports no human input and no active session for at least the configured window (default 30 minutes), and MUST stop the moment activity resumes.

2. The loop MUST reuse the TASK-CUO-201/202/203 propose cycle and MUST run it against the golden sets only. It MUST NOT read or touch production tenant data.

3. A proposed change MUST be applied only if all three hold: (a) it passes the AWH gate (author then test then gate, green); (b) it falls inside the evolution envelope of clause 4; (c) TASK-CUO-202 classifies it low-risk. If any condition fails, the change MUST NOT be applied.

4. The system MUST define an explicit evolution envelope in `config/dream.yaml` with an allowlist of self-modifiable artifacts (skill prompt bodies, workflow step ordering, numeric thresholds, retry counts) and a denylist of security invariants that the loop MUST NEVER auto-modify: cross-tenant isolation logic, audit-chain definitions, auth and RBAC, PII recall gates, cost-ledger math, and anything under a path marked protected.

5. Any proposed change that touches a denylist invariant MUST halt the loop and emit a human-in-the-loop proposal. It MUST NOT be applied autonomously under any circumstance.

6. Every applied change MUST be reversible: the loop MUST record the pre-change content hash and a rollback ref, and MUST auto-revert a change if a follow-up gate run regresses.

7. The loop MUST be bounded: a maximum number of applied changes per idle window, a maximum wall-clock per window, and a kill switch (env var plus a flag in `config/dream.yaml`) that disables dreaming entirely. The kill switch MUST take effect without a redeploy.

8. The loop MUST emit memory audit kinds through the TASK-CUO-201 audit path: `cuo.dream_started`, `cuo.dream_proposal`, `cuo.dream_applied`, `cuo.dream_halted_hitl`, `cuo.dream_reverted`, each carrying the run id, the target artifact, and the outcome.

9. Applied changes MUST land as commits on a dedicated `auto/dream` branch, never on main and never deployed. A human reviews and merges.

10. The loop MUST NOT touch secrets, MUST NOT call external networks beyond what the gate already allows, and MUST NOT deploy or push to any remote.

## Alternatives Considered

Continuous online learning or weight-level fine-tuning of a model while idle. Rejected: CyberOS has no training infrastructure, weight updates are opaque and hard to gate or revert, and an autonomously retrained model is exactly the unsafe self-modification Stephen wants to avoid. The dream loop edits prompts, workflow shape, and thresholds - all human-readable, gate-testable, and revertible - not weights.

Keep evolution human-triggered only (the TASK-CUO-200..203 status quo). Rejected: it does not meet the "improve while offline" ask. Idle time stays unused and every refinement waits on attention.

Let the loop apply any gate-passing change, trusting the gate alone. Rejected: a green gate proves tests pass, not that the change is safe to make autonomously. A prompt edit could pass tests yet weaken a security posture the tests do not cover. The envelope plus the denylist halt is the second fence the gate cannot provide.

## Success Metrics

Primary metric - net safe refinements per idle window.
- Definition: number of changes applied by the dream loop in one idle window that still pass the gate on the next full run (no regression), minus any auto-reverted changes.
- Baseline: 0 (no autonomous loop exists; all evolution is human-triggered).
- Target: greater than 0 per active idle window, with zero net regressions carried into the morning review.
- Measurement method: the `cuo.dream_applied` minus `cuo.dream_reverted` audit counts, cross-checked against the next morning's gate run.
- Source: memory audit rows (TASK-CUO-201 path) and the AWH gate log.

Guardrail metric - denylist auto-applies.
- Definition: number of changes touching a denylist security invariant that were applied without a human approving them.
- Baseline: not applicable (no loop today).
- Target: exactly zero. Every denylist-touching change must appear as a `cuo.dream_halted_hitl` proposal, never as `cuo.dream_applied`.
- Measurement method: audit-log assertion that no `cuo.dream_applied` row has a target inside the denylist set; a CI test enforces this on the envelope.
- Source: memory audit rows plus the envelope unit test.

## Scope

In scope: the idle detector, the dream loop driving the existing propose cycle against the golden sets, the evolution envelope (allowlist plus denylist) and its enforcement, the low-risk-only auto-apply gate, reversibility and auto-revert, the bounded-run limits and kill switch, the five audit kinds, and the dedicated-branch landing.

### Out of scope

- Weight-level fine-tuning or any online learning on a model. The loop never trains weights.
- Auto-merge to main or any deploy. The loop stops at a reviewable branch.
- Any autonomous change to a denylist security invariant. Those always halt for a human.
- Multi-day unattended runs beyond the bounded window, or running while a human is active.
- Reading or using production tenant data. The loop sees only the golden sets.

## Dependencies

- TASK-CUO-200 harness read-only report - the outcome-distribution metrics the loop reads.
- TASK-CUO-201 refinement proposals - the proposal shape and the audit path the loop reuses.
- TASK-CUO-202 auto-bump low-risk - the risk classifier that gates auto-apply.
- TASK-CUO-203 workflow-level evolution - the workflow-edit proposals the loop can apply.
- The AWH gate (author, test, gate) and the per-module golden sets, which the loop runs against.
- The scheduled-task runner that wakes the loop on idle.

## AI Risk Assessment

### Data sources

The loop reads only the golden sets and the harness's own outcome distributions. It does not read production tenant data, secrets, or PII. Proposed edits are to human-readable artifacts (prompts, workflow YAML, thresholds), so every change is inspectable. No model weights are trained or altered.

### Human oversight

The denylist halts any security-invariant change for a human decision (EU AI Act Article 14). Every applied change lands on a review branch that a human merges; nothing reaches main or production autonomously. A kill switch disables the loop without a redeploy. Stephen reviews the morning audit trail of what the loop did overnight, and can revert any change from its recorded rollback ref.

### Failure modes

A bad proposal that passes tests but is wrong is caught when the follow-up gate regresses, and is auto-reverted (clause 6). A change that tries to touch a denylist invariant is halted before apply (clause 5). A runaway loop is bounded by the per-window change and time limits and the kill switch (clause 7). A loop that wakes during human activity is prevented by the idle detector (clause 1). In every case the safe state is "no change applied", reached by halting, not by guessing.

## AI Authorship Disclosure

- Tools used: Claude (Cowork), authoring this FR from Stephen's capability request and the existing TASK-CUO-200..203 harness specs.
- Scope: full draft of this specification, including the normative clauses, the envelope design, and the acceptance metrics.
- Human review: Stephen reviews and approves before status moves past draft; this is a high-risk FR, so the envelope and the denylist need his explicit sign-off, and the paired audit plus the CAF gate validate before any implementation merges.
