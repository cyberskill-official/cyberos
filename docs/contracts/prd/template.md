---
template: prd@1
title: <Product feature name — 3-100 chars>
author: @<author-handle>
created_at: <ISO 8601 with timezone>
last_updated_at: <ISO 8601 with timezone>
prd_status: draft               # draft | in_review | approved | superseded
project_brief_ref: ../briefs/<slug>.brief.md   # path or memory_id of the source project_brief@1
target_release: 2026-Q3
client_visible: false
# client_id: <required if client_visible is true>
eu_ai_act_risk_class: not_ai
confidentiality: internal
prd_iteration: 1
chain_profile: standard          # inherited from project_brief.chain_profile; do not change here
# superseded_by: <path; required if prd_status: superseded>
# cl_sign_off: "@clo-handle 2026-MM-DDTHH:MM:SS+07:00"
# cseco_sign_off: "@cseco-handle 2026-MM-DDTHH:MM:SS+07:00"
---

# <Product feature name>

## Background

<!-- Link to the project brief and add 2-3 paragraphs of additional context not in the brief. -->

Source brief: [<brief-title>](<project_brief_ref>) (`project_brief@1`, iteration <N>).

## Goals

1. <!-- authority: human-edited --> <Outcome statement.>
2. <!-- authority: llm-explicit --> <Outcome statement; cite source in trailing HTML comment.> <!-- source: memories/projects/2026-04-19-pilot-csm-feedback.md -->
3. ...

## Non-goals

- <Explicit out-of-scope item.>
- ...

## User Stories

### Story 1 — <One-line title>

> <!-- authority: human-confirmed --> "<Verbatim quote from a user / customer / stakeholder OR a synthesised user-voice statement.>"

**Acceptance criteria:**
- <!-- authority: llm-explicit --> <Measurable, testable criterion.>
- ...

### Story 2 — ...

## Quality Bars

- **Performance:** <metric> baseline <X>; threshold <Y>; measurement source <where>.
- **Availability:** <e.g. 99.9% target>.
- **Privacy:** <data classes touched, retention rules>.
- **Accessibility:** <e.g. WCAG 2.2 AA>.
- **Security:** <e.g. all writes go through scope-checked MCP; no direct DB access>.

## Open Questions

1. <!-- needs: cuo-clo --> <Question.>
2. <!-- needs: human --> <Question.>

<!-- Or, if no open questions: --> <!-- "No open questions — all decisions captured in this PRD." -->

## EU AI Act Considerations

<!-- Required if eu_ai_act_risk_class ∈ {limited, high}. --> <!-- For not_ai / minimal: state explicitly: --> <!-- "Not in scope of EU AI Act — feature involves no AI/ML inference / biometric data / Annex III activity. Reviewed: <ISO date> by <persona|human>." -->

## Compliance and Privacy

- Frameworks in scope: <GDPR / HIPAA / SOC 2 / ...>
- PII touched: <list>
- Consent required: <yes / no, and what kind>
- Audit logging: <every <action> emits genie.action_log row of kind <X>>

## Rough Sizing

This document feels like ~<N> engineer-months at the FR-create granularity:

- <Story 1> — <S/M/L/XL>.
- <Story 2> — <S/M/L/XL>.

Total: ~<N> engineer-months. Hint for `srs-author`; tech spec will refine.

## Success Definition

12 weeks post-launch:

- <!-- authority: llm-explicit --> ≥<X>% of <segment> have <observable behaviour>.
- <!-- authority: human-edited --> Median time to <outcome> drops from <baseline> to ≤<target>.
- Zero p1 incidents attributable to the feature in the first 30 days.
- <Quality bar metric> stays within budget.

## Research Signals

- **<Source 1>:** <date range>; <what was observed>.
- **<Source 2>:** <date>; <verbatim quote or summary>; cited in `memories/projects/<slug>.md`.
- ...

<!-- ── Conditionally-required sections (uncomment + fill as needed) ── -->

<!--
## Client Context

(Required when client_visible: true.)

- Client: <name>
- Deliverables in scope: <list>
- Milestones: <list with target dates>
- Acceptance criteria: <how the client signs off>

## High-Risk AI Risk Assessment

(Required when eu_ai_act_risk_class: high.)

### Annex III mapping
- ...

### Oversight mechanism
- ...

### Transparency obligations
- ...

### Post-market monitoring
- ...

## Compliance Implementation Plan

(Required when confidentiality ∈ {client_confidential, regulated}.)

- Encryption at rest: <details>
- Audit logging: <details>
- Retention rules: <details>
- BCDR: <details>

## Approval Record

(Required when prd_status: approved.)

| Role | Person | Approved at (ISO) | PRD version hash (sha256) |
| --- | --- | --- | --- |
| Product owner | @<handle> | <ts> | sha256:<hash> |
| Engineering lead | @<handle> | <ts> | sha256:<hash> |
| CLO (if applicable) | @<handle> | <ts> | sha256:<hash> |
-->
