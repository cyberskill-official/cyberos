---
workflow_id: <persona-slug>/<workflow-slug>
workflow_version: 1.0.0
purpose: <one-line purpose statement>
persona: cuo/<persona-slug>
cadence: on-demand    # daily | weekly | monthly | quarterly | annual | on-demand | per-event
status: planned       # planned | shipped | retired

inputs:
  - { name: <input-name>, source: <where>, format: <markdown | json | dashboard | verbal-brief | meeting-notes> }

outputs:
  - { name: <output-name>, format: <artifact type — e.g. product-requirements-document@1 | statement-of-work@1 | runbook@1 | ad-hoc-md>, recipient: <persona-slug or external> }

skill_chain:
  - { step: 1, skill: <skill-name>, inputs_from: <workflow input name OR prior-step output>, outputs_to: <next-step input OR workflow output> }
  - { step: 2, skill: <skill-name>, inputs_from: ..., outputs_to: ... }
  # Use `planned:<skill-name>` for skills that don't yet ship in the SKILL module
  # (then add the gap to cuo/docs/NEEDED_SKILLS.md).

escalates_to:
  - { persona: cuo/<persona-slug>, when: "<trigger condition>" }

consults:
  - { persona: cuo/<persona-slug>, when: "<trigger condition>" }

audit_hooks:
  - each step's output is logged to memory audit chain via memory module (per memory/docs/AGENTS.md §6)
  - workflow completion emits a single `workflow_complete` row with the full chain summary
  - HITL pauses halt the chain; resumption requires the operator's reply to be parsed and applied
---

# <Workflow Title> — `<persona-slug>` workflow

> One-paragraph operator-facing description. What problem does this workflow solve? Which deliverable does it produce? When should the persona reach for it?

## When to invoke

CUO routes here when the user says things like:

- "<example natural-language trigger 1>"
- "<example natural-language trigger 2>"
- "<example natural-language trigger 3>"

## How to invoke (CLI / CUO supervisor)

```bash
# Direct invocation (skip routing; useful for scripting)
cyberos-cuo run cuo/<persona-slug>/<workflow-slug> \
  --input '<json or path to input>' \
  --output-dir ./outputs/
```

## Expected duration

- **Happy path:** <N> minutes / hours / days
- **With HITL pauses:** add <N> hours per pause for operator response
- **Worst case (chain exhausts max_iterations):** <N>

## Skill chain — step by step

### Step 1: `<skill-name>`
- **What it does:** <one-line summary of the SKILL module skill's purpose>
- **Inputs from this workflow:** <named workflow inputs or prior-step outputs>
- **Outputs:** <named artefacts the SKILL produces>
- **Pause point:** <if applicable — operator decision required>

### Step 2: `<skill-name>`
- **What it does:** ...
- **Inputs:** ...
- **Outputs:** ...

(Repeat per step.)

## Failure modes — per step

| Step | Code | What happens | Recovery |
|---|---|---|---|
| 1 | BOOT-001 | Required input file missing | Operator supplies; resume |
| 1 | HITL | Step skill pauses for human input | Operator answers; chain resumes |
| 2 | EXHAUSTED | Audit loop hit max_iterations without converging | Operator escalates to <persona-slug> for manual revision |

## Operator-side decisions

Operators are pulled into this workflow at the following pause points:

1. **<Pause point 1>** — e.g. PLAN approval before WORKER phase starts.
2. **<Pause point 2>** — e.g. HITL on numeric targets without source.
3. **<Pause point 3>** — e.g. compliance-boundary escalation per `escalates_to:` declaration.

## Cross-references

- `../README.md` — the persona's 9-block spec (this workflow renders block 5's strategic / operational / communication outputs).
- `../../../../modules/cuo/docs/module.md` §4 — the source role profile (persona catalog).
- `../../docs/AGENTS.md` — protocol normativity.
- `../../docs/ROUTING.md` — how the CUO supervisor reaches this workflow.
- `../../docs/NEEDED_SKILLS.md` — if any step references a `planned:` skill, the gap is enumerated there.
- `../../../skill/<chain-step-skill>/SKILL.md` — the per-skill spec for each chain step.
