---
workflow_id: chief-of-staff/monthly-leadership-onboarding
workflow_version: 1.0.0
purpose: Monthly onboarding cadence for new exec hires — context pack, intro plan, 30/60/90 alignment.
persona: cuo/chief-of-staff
cadence: monthly
status: shipped

inputs:
  - { name: brief, source: persona-specific operator input (drive doc, calendar event, ticket), format: markdown }
  - { name: prior_artefact, source: BRAIN search for prior `monthly-leadership-onboarding` outputs, format: markdown }

outputs:
  - { name: monthly_leadership_onboarding_artefact, format: ad-hoc-md, recipient: persona owner + downstream consumers }

skill_chain:
  - { step: 1, skill: onboarding-pack-author, inputs_from: brief, outputs_to: onb_draft }
  - { step: 2, skill: onboarding-pack-audit, inputs_from: onb_draft, outputs_to: onb_final }

escalates_to: []

consults: []

audit_hooks:
  - each step's output is logged to BRAIN audit chain via memory module (per modules/memory/AGENTS.md §6)
  - workflow completion emits a single `workflow_complete` row with the full chain summary
  - HITL pauses halt the chain; resumption requires the operator's reply to be parsed and applied
---

# Monthly Leadership Onboarding — `chief-of-staff` workflow

> Monthly onboarding cadence for new exec hires — context pack, intro plan, 30/60/90 alignment.

## When to invoke

CUO routes here when the user says things like:

- "Run the CHIEF OF STAFF monthly leadership onboarding"
- "Monthly monthly leadership onboarding"
- "Run the monthly-leadership-onboarding workflow"

## How to invoke

```bash
cyberos-cuo execute chief-of-staff/monthly-leadership-onboarding \
    --output-dir <dir> \
    --invoker llm \
    --brain-emit
```

## Expected duration

- Mock invoker: <1s (sandbox smoke).
- Subprocess invoker against the Rust SKILL host: depends on per-step skill execution.
- LLM invoker (real Anthropic API): ~30s-3min depending on prompt complexity.

## Skill-chain step-by-step

**Step 1 — `onboarding-pack-author`**: produces the artefact body in the shape declared by `modules/skill/onboarding-pack-author/CONTRACT.md`. Consumes the `brief` input (workflow-supplied) plus any prior-step hand-off.

**Step 2 — `onboarding-pack-audit`**: validates the upstream artefact against the per-skill RUBRIC. Returns `rubric_outcome: {score, pass, fixes}` plus a verdict (pass / needs_human / fail / exhausted).

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

- Skill chain anchors: see `modules/skill/onboarding-pack-author/` + `modules/skill/onboarding-pack-audit/`
- Persona spec: see `modules/cuo/chief-of-staff/README.md` (9-block schema, §8 Audit criteria)
- BRAIN protocol: see `modules/memory/AGENTS.md` §6 (audit chain semantics)
