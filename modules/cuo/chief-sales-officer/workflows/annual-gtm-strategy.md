---
workflow_id: chief-sales-officer/annual-gtm-strategy
workflow_version: 1.0.0
purpose: Annual GTM strategy doc — ICP refinement, motion model, channel mix, capacity plan, comp model alignment.
persona: cuo/chief-sales-officer
cadence: annual
status: shipped

inputs:
  - { name: brief, source: persona-specific operator input (drive doc, calendar event, ticket), format: markdown }
  - { name: prior_artefact, source: memory search for prior `annual-gtm-strategy` outputs, format: markdown }

outputs:
  - { name: annual_gtm_strategy_artefact, format: ad-hoc-md, recipient: persona owner + downstream consumers }

skill_chain:
  - { step: 1, skill: go-to-market-plan-author, inputs_from: brief, outputs_to: gtm_draft }
  - { step: 2, skill: go-to-market-plan-audit, inputs_from: gtm_draft, outputs_to: gtm_final }

escalates_to: []

consults: []

audit_hooks:
  - each step's output is logged to memory audit chain via memory module (per modules/memory/cyberos/data/AGENTS.md §6)
  - workflow completion emits a single `workflow_complete` row with the full chain summary
  - HITL pauses halt the chain; resumption requires the operator's reply to be parsed and applied
---

# Annual Gtm Strategy — `cso-sales` workflow

> Annual GTM strategy doc — ICP refinement, motion model, channel mix, capacity plan, comp model alignment.

## When to invoke

CUO routes here when the user says things like:

- "Run the CSO SALES annual gtm strategy"
- "Annual annual gtm strategy"
- "Run the annual-gtm-strategy workflow"

## How to invoke

```bash
cyberos-cuo execute chief-sales-officer/annual-gtm-strategy \
    --output-dir <dir> \
    --invoker llm \
    --memory-emit
```

## Expected duration

- Mock invoker: <1s (sandbox smoke).
- Subprocess invoker against the Rust SKILL host: depends on per-step skill execution.
- LLM invoker (real Anthropic API): ~30s-3min depending on prompt complexity.

## Skill-chain step-by-step

**Step 1 — `go-to-market-plan-author`**: produces the artefact body in the shape declared by `modules/skill/go-to-market-plan-author/SKILL.md`. Consumes the `brief` input (workflow-supplied) plus any prior-step hand-off.

**Step 2 — `go-to-market-plan-audit`**: validates the upstream artefact against the per-skill RUBRIC. Returns `rubric_outcome: {score, pass, fixes}` plus a verdict (pass / needs_human / fail / exhausted).

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

- Skill chain anchors: see `modules/skill/go-to-market-plan-author/` + `modules/skill/go-to-market-plan-audit/`
- Persona spec: see `modules/cuo/chief-sales-officer/README.md` (9-block schema, §8 Audit criteria)
- memory protocol: see `modules/memory/cyberos/data/AGENTS.md` §6 (audit chain semantics)
