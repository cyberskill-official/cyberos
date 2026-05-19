---
workflow_id: chief-technology-officer/quarterly-tech-debt-review
workflow_version: 1.0.0
purpose: Quarterly tech-debt audit + prioritised paydown plan — what slows shipping, what risks security, what to bury.
persona: cuo/chief-technology-officer
cadence: quarterly
status: shipped

inputs:
  - { name: brief, source: persona-specific operator input (drive doc, calendar event, ticket), format: markdown }
  - { name: prior_artefact, source: memory search for prior `quarterly-tech-debt-review` outputs, format: markdown }

outputs:
  - { name: quarterly_tech_debt_review_artefact, format: ad-hoc-md, recipient: persona owner + downstream consumers }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: brief, outputs_to: debt_strategy_draft }
  - { step: 2, skill: strategy-document-audit, inputs_from: debt_strategy_draft, outputs_to: tech_debt_plan }

escalates_to: []

consults: []

audit_hooks:
  - each step's output is logged to memory audit chain via memory module (per modules/memory/AGENTS.md §6)
  - workflow completion emits a single `workflow_complete` row with the full chain summary
  - HITL pauses halt the chain; resumption requires the operator's reply to be parsed and applied
---

# Quarterly Tech Debt Review — `cto` workflow

> Quarterly tech-debt audit + prioritised paydown plan — what slows shipping, what risks security, what to bury.

## When to invoke

CUO routes here when the user says things like:

- "Run the CTO quarterly tech debt review"
- "Quarterly quarterly tech debt review"
- "Run the quarterly-tech-debt-review workflow"

## How to invoke

```bash
cyberos-cuo execute chief-technology-officer/quarterly-tech-debt-review \
    --output-dir <dir> \
    --invoker llm \
    --memory-emit
```

## Expected duration

- Mock invoker: <1s (sandbox smoke).
- Subprocess invoker against the Rust SKILL host: depends on per-step skill execution.
- LLM invoker (real Anthropic API): ~30s-3min depending on prompt complexity.

## Skill-chain step-by-step

**Step 1 — `strategy-document-author`**: produces the artefact body in the shape declared by `modules/skill/strategy-document-author/CONTRACT.md`. Consumes the `brief` input (workflow-supplied) plus any prior-step hand-off.

**Step 2 — `strategy-document-audit`**: validates the upstream artefact against the per-skill RUBRIC. Returns `rubric_outcome: {score, pass, fixes}` plus a verdict (pass / needs_human / fail / exhausted).

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

- Skill chain anchors: see `modules/skill/strategy-document-author/` + `modules/skill/strategy-document-audit/`
- Persona spec: see `modules/cuo/chief-technology-officer/README.md` (9-block schema, §8 Audit criteria)
- memory protocol: see `modules/memory/AGENTS.md` §6 (audit chain semantics)
