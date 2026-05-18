---
template: software-requirements-specification@1
title: <System feature name>
author: @<author-handle>
created_at: <ISO 8601 with timezone>
last_updated_at: <ISO 8601 with timezone>
srs_status: draft
prd_ref: ../prds/<slug>.prd.md
target_release: 2026-Q3
srs_iteration: 1
architectural_review_passed: false
# superseded_by: <path; required if srs_status: superseded>
# cseco_sign_off: "@cseco-handle 2026-MM-DDTHH:MM:SS+07:00"
---

# <System feature name>

## Background

Source PRD: [<prd-title>](<prd_ref>) (`product-requirements-document@1`, iteration <N>).

<1-2 paragraphs on technical context not in PRD.>

## System Architecture

<!-- authority: human-edited --> <Component-by-component description.>

## Data Model

<Entities, relationships, schema deltas, migrations.>

## API Surface

| Method | Path | Request schema | Response schema | Idempotent |
| --- | --- | --- | --- | --- |
| GET | /api/foo | - | { ... } | yes |

## Data Flows

### Story 1 — <PRD story title>

<End-to-end sequence; can be a sequence diagram or numbered list.>

## Non-Functional Requirements

- **Performance:** <p95 latency target> at <load>.
- **Availability:** <SLA>.
- **Durability:** <RPO/RTO>.
- **Scalability:** <horizontal limits, vertical limits>.
- **Security:** <see Security Posture below>.
- **Observability:** <see Telemetry Plan below>.

## Failure Modes

- <Failure mode 1>: <handling>.
- <Failure mode 2>: <handling>.

## Security Posture

- Auth: <mechanism>.
- Authz: <RBAC / ABAC / capability-based>.
- Secret store: <which one; rotation cadence>.
- Encryption at rest: <algorithm; key management>.
- Audit trail: <every action of kind X emits genie.action_log row of kind Y>.

## Telemetry Plan

- Events: <list>.
- Metrics: <list with type {counter|gauge|histogram} + label dimensions>.
- Logs: <log lines that MUST exist; severity threshold for alerting>.

## Open Architectural Questions

1. <!-- needs: cuo-cto --> <Question.>
2. <!-- needs: cuo-cseco --> <Question.>

<!-- ── Conditionally-required (uncomment + fill) ── -->

<!--
## AI Subsystem Spec

(Required when prd_ref's eu_ai_act_risk_class: high.)
- Model: <name; version; provider>
- Inference path: <local / hosted; latency budget>
- Oversight implementation: <how a human reviews each high-stakes decision>
- Transparency mechanism: <what the user sees about AI involvement>

## Compliance Implementation

(Required when PRD's confidentiality ∈ {client_confidential, regulated}.)

- Encryption at rest: <details>
- Audit log retention: <duration; immutability mechanism>
- BCDR: <RTO / RPO; tested cadence>

## Review Record

(Required when architectural_review_passed: true.)

| Reviewer | Role | Approved at (ISO) | SRS version hash |
| --- | --- | --- | --- |
| @<handle> | CTO | <ts> | sha256:<hash> |
| @<handle> | Engineering | <ts> | sha256:<hash> |
-->
