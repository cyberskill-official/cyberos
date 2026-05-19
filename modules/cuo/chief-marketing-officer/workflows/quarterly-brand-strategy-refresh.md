---
workflow_id: chief-marketing-officer/quarterly-brand-strategy-refresh
workflow_version: 1.0.0
purpose: Quarterly brand-strategy refresh — positioning, messaging arc, narrative themes, channel mix recalibration.
persona: cuo/chief-marketing-officer
cadence: quarterly
status: shipped

inputs:
  - { name: brief, source: persona-specific operator input (drive doc, calendar event, ticket), format: markdown }
  - { name: prior_artefact, source: memory search for prior `quarterly-brand-strategy-refresh` outputs, format: markdown }

outputs:
  - { name: quarterly_brand_strategy_refresh_artefact, format: ad-hoc-md, recipient: persona owner + downstream consumers }

skill_chain:
  - { step: 1, skill: brand-strategy-author, inputs_from: brief, outputs_to: brand_draft }
  - { step: 2, skill: brand-strategy-audit, inputs_from: brand_draft, outputs_to: brand_final }

escalates_to: []

consults: []

audit_hooks:
  - each step's output is logged to memory audit chain via memory module (per modules/memory/AGENTS.md §6)
  - workflow completion emits a single `workflow_complete` row with the full chain summary
  - HITL pauses halt the chain; resumption requires the operator's reply to be parsed and applied
---

# Quarterly Brand Strategy Refresh — `cmo` workflow

> Quarterly brand-strategy refresh — positioning, messaging arc, narrative themes, channel mix recalibration.

## When to invoke

CUO routes here when the user says things like:

- "Run the CMO quarterly brand strategy refresh"
- "Quarterly quarterly brand strategy refresh"
- "Run the quarterly-brand-strategy-refresh workflow"

## How to invoke

```bash
cyberos-cuo execute chief-marketing-officer/quarterly-brand-strategy-refresh \
    --output-dir <dir> \
    --invoker llm \
    --memory-emit
```

## Expected duration

- Mock invoker: <1s (sandbox smoke).
- Subprocess invoker against the Rust SKILL host: depends on per-step skill execution.
- LLM invoker (real Anthropic API): ~30s-3min depending on prompt complexity.

## Skill-chain step-by-step

**Step 1 — `brand-strategy-author`**: produces the artefact body in the shape declared by `modules/skill/brand-strategy-author/CONTRACT.md`. Consumes the `brief` input (workflow-supplied) plus any prior-step hand-off.

**Step 2 — `brand-strategy-audit`**: validates the upstream artefact against the per-skill RUBRIC. Returns `rubric_outcome: {score, pass, fixes}` plus a verdict (pass / needs_human / fail / exhausted).

## Failure modes

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Author skill returns malformed JSON | Audit skill flags `validation_failed` | Chain halts at step 2 with FAILED outcome | operator reviews, re-runs with corrected input |
| Audit skill returns `rubric_outcome.pass=false` | Supervisor reads rubric_outcome | Chain returns PARTIAL; output quarantined | author re-runs with corrections; re-audit |
| LLM API rate limit | Anthropic SDK raises 429 | StepResult.status=FAILED with stderr | retry with backoff; or switch --invoker mock for sandbox |

## Operator-side decisions

- **Approve the audit verdict before downstream consumption.** Audit `pass` does NOT mean operator-accepted; it means rubric-satisfied.
- **Reuse last quarter's prior_artefact** unless context has materially changed; the memory search input handles this automatically when present.
- **Tag the memory row** with explicit business context if this is a one-off vs the recurring cadence.

## Cross-references

- Skill chain anchors: see `modules/skill/brand-strategy-author/` + `modules/skill/brand-strategy-audit/`
- Persona spec: see `modules/cuo/chief-marketing-officer/README.md` (9-block schema, §8 Audit criteria)
- memory protocol: see `modules/memory/AGENTS.md` §6 (audit chain semantics)
