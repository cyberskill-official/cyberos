---
workflow_id: chief-human-resources-officer/quarterly-enps-pulse
workflow_version: 1.0.0
purpose: Run the quarterly eNPS pulse — survey distribution, results analysis, manager-distribution insights, action plan.
persona: cuo/chief-human-resources-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_enps,         source: last quarter's enps-program@1,                  format: employee-net-promoter-score-program@1 }
  - { name: survey_results,     source: Culture Amp / Lattice / Officevibe / TINYpulse, format: csv export }
  - { name: open_actions,       source: prior-quarter action plan + status,             format: markdown }

outputs:
  - { name: enps_pulse,         format: enps-program@1, recipient: cuo/chro + cuo/ceo + all managers (their team detail) }

skill_chain:
  - { step: 1, skill: employee-net-promoter-score-program-author, inputs_from: { prior_enps: prior_enps, survey_results: survey_results, open_actions: open_actions }, outputs_to: pulse_draft }
  - { step: 2, skill: employee-net-promoter-score-program-audit,  inputs_from: pulse_draft, outputs_to: enps_pulse }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "company eNPS drops >10 pts QoQ OR any team eNPS < -10" }
  - { persona: cuo/chief-happiness-officer, when: "specific theme (workload / management / growth) flagged in >3 teams" }

consults:
  - { persona: cuo/chief-communications-officer, when: "results need internal-comms rollout (good news or bad)" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with enps_pulse hash + company-eNPS + per-team breakdown
  - HITL pause at step 2 on QA-ACTION-001 (no action plan for low scorers)
---

# Quarterly eNPS pulse — `chief-human-resources-officer/quarterly-enps-pulse`

CHRO's quarterly eNPS pulse workflow. Distributes survey via Culture Amp / Lattice / Officevibe / TINYpulse, analyzes results, generates per-manager distribution insights, drafts action plan. Standard target eNPS ≥30 per First Round / a16z benchmarks for high-growth companies.

## When to invoke

- "Run the Q<n> eNPS pulse"
- "Quarterly engagement survey"
- "Employee NPS results"

## How to invoke

```bash
cyberos-cuo run cuo/chief-human-resources-officer/quarterly-enps-pulse \
  --input prior_enps=./hr/2026-Q1/enps.md \
  --input survey_results=./hr/2026-Q2/culture-amp.csv \
  --input open_actions=./hr/2026-Q1/enps-actions.md \
  --output-dir ./hr/2026-Q2/
```

## Expected duration

- **Happy path:** 1-2 hours runtime + 2 weeks for survey window + analysis
- **Worst case:** company-eNPS drop triggers 1-quarter intervention program

## Skill chain

- **Step 1 `employee-net-promoter-score-program-author`** — drafts per Officevibe / Culture Amp + First Round benchmarks.
- **Step 2 `employee-net-promoter-score-program-audit`** — validates per `enps_program_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-ACTION-001 | No action plan for low scorers | Operator drafts |
| 2 | QA-COMP-PCT-001 | Response rate < 60% | Operator extends window |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.5 — CHRO role profile
- `../../chief-happiness-officer/README.md` — escalation peer
- `../../../skill/enps-program-{author,audit}/SKILL.md`
