---
workflow_id: chief-ai-officer/annual-ai-strategy-refresh
workflow_version: 1.0.0
purpose: Annual AI strategy refresh — capability investments, model governance, talent plan, vendor mix.
persona: cuo/chief-ai-officer
cadence: annual
status: shipped

inputs:
  - { name: brief, source: persona-specific operator input (drive doc, calendar event, ticket), format: markdown }
  - { name: prior_artefact, source: BRAIN search for prior `annual-ai-strategy-refresh` outputs, format: markdown }

outputs:
  - { name: annual_ai_strategy_refresh_artefact, format: ad-hoc-md, recipient: persona owner + downstream consumers }

skill_chain:
  - { step: 1, skill: ai-strategy-author, inputs_from: brief, outputs_to: ais_draft }
  - { step: 2, skill: ai-strategy-audit, inputs_from: ais_draft, outputs_to: ais_final }

escalates_to: []

consults: []

audit_hooks:
  - each step's output is logged to BRAIN audit chain via memory module (per modules/memory/AGENTS.md §6)
  - workflow completion emits a single `workflow_complete` row with the full chain summary
  - HITL pauses halt the chain; resumption requires the operator's reply to be parsed and applied
---

# Annual Ai Strategy Refresh — `caio` workflow

> Annual AI strategy refresh — capability investments, model governance, talent plan, vendor mix.

## When to invoke

CUO routes here when the user says things like:

- "Run the CAIO annual ai strategy refresh"
- "Annual annual ai strategy refresh"
- "Run the annual-ai-strategy-refresh workflow"

## How to invoke

```bash
cyberos-cuo execute chief-ai-officer/annual-ai-strategy-refresh \
    --output-dir <dir> \
    --invoker llm \
    --brain-emit
```

## Expected duration

- Mock invoker: <1s (sandbox smoke).
- Subprocess invoker against the Rust SKILL host: depends on per-step skill execution.
- LLM invoker (real Anthropic API): ~30s-3min depending on prompt complexity.

## Skill-chain step-by-step

**Step 1 — `ai-strategy-author`**: produces the artefact body in the shape declared by `modules/skill/ai-strategy-author/CONTRACT.md`. Consumes the `brief` input (workflow-supplied) plus any prior-step hand-off.

**Step 2 — `ai-strategy-audit`**: validates the upstream artefact against the per-skill RUBRIC. Returns `rubric_outcome: {score, pass, fixes}` plus a verdict (pass / needs_human / fail / exhausted).

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

- Skill chain anchors: see `modules/skill/ai-strategy-author/` + `modules/skill/ai-strategy-audit/`
- Persona spec: see `modules/cuo/chief-ai-officer/README.md` (9-block schema, §8 Audit criteria)
- BRAIN protocol: see `modules/memory/AGENTS.md` §6 (audit chain semantics)
