---
template: project_brief@1
title: <Project name — 3-80 chars>
author: @<author-handle>
created_at: <ISO 8601 with timezone, e.g. 2026-05-06T10:00:00+07:00>
last_updated_at: <ISO 8601 with timezone>
project_kind: software_product   # software_product | software_consulting_engagement | internal_tooling | marketing_campaign | hiring_plan | partnership | research_spike | other
triage_verdict: proceed          # proceed | revise | reject
# triage_reason: <required if triage_verdict ∈ {revise, reject}; 1-500 chars>
target_release: 2026-Q3          # SemVer | quarter (YYYY-Qn) | unspecified
client_visible: false            # true if a specific client commissioned this
# client_id: <required if client_visible is true>
eu_ai_act_risk_class: not_ai     # not_ai | minimal | limited | high
confidentiality: internal        # public | internal | client_confidential | regulated
budget_band: undisclosed         # none | under_5k | 5k_to_25k | 25k_to_100k | over_100k | undisclosed
team_capacity_check_passed: true
discovery_iteration: 1
chain_profile: standard          # lean | standard | full — see CONTRACT.md §Chain profile
---

# <Project name>

## Background

<!-- 2-5 paragraphs explaining WHY this project is being considered. What signal triggered it? Citations to memory entries (memories/projects/*, memories/decisions/*) are encouraged. -->

## Goals

1. <!-- authority: human-edited | human-confirmed | llm-explicit | llm-implicit --> <One ≤2-sentence outcome statement.>
2. ...

## Audience

<!-- WHO benefits? Be specific. "Users" is rejected — name the persona, segment, or specific client. -->

## Success Metrics

- **Primary metric:** <metric name> — baseline <X>; target <Y by date>; measurement source <where the data comes from>.
- **Guardrail metric (optional):** <metric name> — must not degrade beyond <threshold>.

## Constraints

- <Timeline / budget / regulatory / technical / headcount constraint as a bullet.>
- ...

## Kill Criteria

- <"We'd kill this if [observable signal X]">
- ...

## Stakeholder Map

| Role | Person | Responsibility |
| --- | --- | --- |
| Decider | <name/handle> | final approve/reject |
| Reviewer | <name/handle> | reviews drafts, doesn't gate |
| Informed | <name/handle> | needs to know, not asked to act |

## Prior Art (memory)

<!-- What does the memory tell us we already tried, decided, or learned? Cite memories/decisions/DEC-NNN-*.md, memories/projects/<project>.md, company/locked-decisions.md paths. If nothing relevant, write the explicit statement: "No relevant prior art found in memory as of <ISO date>." -->

## Open Questions

<!-- Each question gets a marker indicating who should answer. -->

1. <!-- needs: cuo-clo --> <Question for legal review.>
2. <!-- needs: human --> <Question that requires user decision.>

<!-- Or, if no open questions: --> <!-- "No open questions — all required intake answered as of <ISO date>." -->

<!-- ── Conditionally-required sections (uncomment + fill as needed) ── -->

<!--
## Client Context

(Required when client_visible: true.)

- Client: <name>
- Signed agreements: NDA <date>; MSA <date>; SOW <id>
- Known sensitivities: <list>

## AI Risk Snapshot

(Required when eu_ai_act_risk_class ∈ {limited, high}. Preliminary; full assessment in PRD.)

### Data Sources
- ...

### Human Oversight
- ...

### Failure Modes
- ...

## Compliance Constraints

(Required when confidentiality ∈ {client_confidential, regulated}.)

- Frameworks in scope: <GDPR / HIPAA / SOC 2 / ...>
- Required controls: <list>

## Triage Reasoning

(Required when triage_verdict ∈ {revise, reject}.)

- Why the verdict was set: <reason>
- What would change it: <observable signal or threshold> -->
