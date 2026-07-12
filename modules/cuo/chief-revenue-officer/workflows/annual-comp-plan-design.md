---
workflow_id: chief-revenue-officer/annual-comp-plan-design
workflow_version: 1.0.0
purpose: Annual sales-comp plan design — quota model, accelerators, SPIFFs, ramp policy, OTE recalibration.
persona: cuo/chief-revenue-officer
cadence: annual
status: shipped

inputs:
  - { name: brief, source: persona-specific operator input (drive doc, calendar event, ticket), format: markdown }
  - { name: prior_artefact, source: memory search for prior `annual-comp-plan-design` outputs, format: markdown }

outputs:
  - { name: annual_comp_plan_design_artefact, format: ad-hoc-md, recipient: persona owner + downstream consumers }

skill_chain:
  - { step: 1, skill: compensation-plan-author, inputs_from: brief, outputs_to: scp_draft }
  - { step: 2, skill: compensation-plan-audit, inputs_from: scp_draft, outputs_to: scp_final }

escalates_to: []

consults: []

audit_hooks:
  - each step's output is logged to memory audit chain via memory module (per modules/memory/cyberos/data/AGENTS.md §6)
  - workflow completion emits a single `workflow_complete` row with the full chain summary
  - HITL pauses halt the chain; resumption requires the operator's reply to be parsed and applied
---

# Annual Comp Plan Design — `cro-revenue` workflow

> Annual sales-comp plan design — quota model, accelerators, SPIFFs, ramp policy, OTE recalibration.

## When to invoke

CUO routes here when the user says things like:

- "Run the CRO REVENUE annual comp plan design"
- "Annual annual comp plan design"
- "Run the annual-comp-plan-design workflow"

## How to invoke

```bash
cyberos-cuo execute chief-revenue-officer/annual-comp-plan-design \
    --output-dir <dir> \
    --invoker llm \
    --memory-emit
```

## Expected duration

- Mock invoker: <1s (sandbox smoke).
- Subprocess invoker against the Rust SKILL host: depends on per-step skill execution.
- LLM invoker (real Anthropic API): ~30s-3min depending on prompt complexity.

## Skill-chain step-by-step

**Step 1 — `compensation-plan-author`**: produces the artefact body in the shape declared by `modules/skill/compensation-plan-author/SKILL.md`. Consumes the `brief` input (workflow-supplied) plus any prior-step hand-off.

**Step 2 — `compensation-plan-audit`**: validates the upstream artefact against the per-skill RUBRIC. Returns `rubric_outcome: {score, pass, fixes}` plus a verdict (pass / needs_human / fail / exhausted).

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

- Skill chain anchors: see `modules/skill/compensation-plan-author/` + `modules/skill/compensation-plan-audit/`
- Persona spec: see `modules/cuo/chief-revenue-officer/README.md` (9-block schema, §8 Audit criteria)
- memory protocol: see `modules/memory/cyberos/data/AGENTS.md` §6 (audit chain semantics)
