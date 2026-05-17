---
workflow_id: chief-product-officer/annual-product-vision-refresh
workflow_version: 1.0.0
purpose: Annual product-vision refresh — 3-year horizon, big-bet portfolio, OKR cascade alignment.
persona: cuo/chief-product-officer
cadence: annual
status: shipped

inputs:
  - { name: brief, source: persona-specific operator input (drive doc, calendar event, ticket), format: markdown }
  - { name: prior_artefact, source: BRAIN search for prior `annual-product-vision-refresh` outputs, format: markdown }

outputs:
  - { name: annual_product_vision_refresh_artefact, format: ad-hoc-md, recipient: persona owner + downstream consumers }

skill_chain:
  - { step: 1, skill: product-roadmap-author, inputs_from: brief, outputs_to: vision_draft }
  - { step: 2, skill: product-roadmap-audit, inputs_from: vision_draft, outputs_to: vision_final }

escalates_to: []

consults: []

audit_hooks:
  - each step's output is logged to BRAIN audit chain via memory module (per modules/memory/AGENTS.md §6)
  - workflow completion emits a single `workflow_complete` row with the full chain summary
  - HITL pauses halt the chain; resumption requires the operator's reply to be parsed and applied
---

# Annual Product Vision Refresh — `cpo-product` workflow

> Annual product-vision refresh — 3-year horizon, big-bet portfolio, OKR cascade alignment.

## When to invoke

CUO routes here when the user says things like:

- "Run the CPO PRODUCT annual product vision refresh"
- "Annual annual product vision refresh"
- "Run the annual-product-vision-refresh workflow"

## How to invoke

```bash
cyberos-cuo execute chief-product-officer/annual-product-vision-refresh \
    --output-dir <dir> \
    --invoker llm \
    --brain-emit
```

## Expected duration

- Mock invoker: <1s (sandbox smoke).
- Subprocess invoker against the Rust SKILL host: depends on per-step skill execution.
- LLM invoker (real Anthropic API): ~30s-3min depending on prompt complexity.

## Skill-chain step-by-step

**Step 1 — `product-roadmap-author`**: produces the artefact body in the shape declared by `modules/skill/product-roadmap-author/CONTRACT.md`. Consumes the `brief` input (workflow-supplied) plus any prior-step hand-off.

**Step 2 — `product-roadmap-audit`**: validates the upstream artefact against the per-skill RUBRIC. Returns `rubric_outcome: {score, pass, fixes}` plus a verdict (pass / needs_human / fail / exhausted).

## Failure modes

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Author skill returns malformed JSON | Audit skill flags `validation_failed` | Chain halts at step 2 with FAILED outcome | operator reviews, re-runs with corrected input |
| Audit skill returns `rubric_outcome.pass=false` | Supervisor reads rubric_outcome | Chain returns PARTIAL; output quarantined | author re-runs with corrections; re-audit |
| LLM API rate limit | Anthropic SDK raises 429 | StepResult.status=FAILED with stderr | retry with backoff; or switch --invoker mock for sandbox |

## Operator-side decisions

- **Approve the audit verdict before downstream consumption.** Audit `pass` does NOT mean operator-accepted; it means rubric-satisfied.
- **Reuse last quarter's prior_artefact** unless context has materially changed; the BRAIN search input handles this automatically when present.
- **Tag the BRAIN row** with explicit business context if this is a one-off vs the recurring cadence.

## Cross-references

- Skill chain anchors: see `modules/skill/product-roadmap-author/` + `modules/skill/product-roadmap-audit/`
- Persona spec: see `modules/cuo/chief-product-officer/README.md` (9-block schema, §8 Audit criteria)
- BRAIN protocol: see `modules/memory/AGENTS.md` §6 (audit chain semantics)
