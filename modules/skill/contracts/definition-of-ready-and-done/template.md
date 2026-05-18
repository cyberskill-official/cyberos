---
template: definition-of-ready-and-done@1
title: <Project> — Definition of Ready + Definition of Done
author: @<engagement-manager>
project: <Project name>
engagement_model: fixed_price    # fixed_price | time_and_materials | dedicated_team | staff_augmentation | managed_services
effective_date: 2026-MM-DD
dor_dod_version: 1.0.0
provenance:
  source_path: ./project-plan.md
  source_hash: sha256:<hash>
approved_by:
  - { handle: "@<po-handle>",  role: "PO" }
  - { handle: "@<tl-handle>",  role: "TL" }
  - { handle: "@<em-handle>",  role: "EM" }
  - { handle: "@<qa-handle>",  role: "QA" }
---

# <Project> — Definition of Ready + Definition of Done

## 1. Definition of Ready

A backlog item is READY for sprint pickup only when ALL of the following are true:

- **DOR-001** Clear user value statement (who benefits, how).
- **DOR-002** Acceptance criteria explicit (Given/When/Then).
- **DOR-003** Dependencies identified (teams, vendors, hardware, external APIs).
- **DOR-004** NFRs noted (perf, security, accessibility) — even "none beyond defaults".
- **DOR-005** Security/privacy implications flagged (data class, PII, regulatory scope).
- **DOR-006** Designs attached (Figma / wireframe / mock-up) OR N/A with reason.
- **DOR-007** Estimable in one sprint (or marked "spike" with timeboxed budget).
- **DOR-008** Demoable (success shown to a stakeholder in <5 min).

## 2. Definition of Done

A backlog item is DONE only when ALL of the following are true:

- **DOD-001** Code merged to main.
- **DOD-002** Unit tests passing.
- **DOD-003** Integration tests passing.
- **DOD-004** Code coverage ≥ **<threshold>%**.
- **DOD-005** SAST scan clean (no new high-severity findings).
- **DOD-006** SCA scan clean (no new high-severity dependency vulns).
- **DOD-007** Documentation updated (API docs / user docs / ADR if applicable).
- **DOD-008** Deployed to staging.
- **DOD-009** Product owner accepted (UAT or async sign-off).
- **DOD-010** Observability hooks present (logs / metrics / traces).

## 3. Scope of Application

| Item type | DoR applies? | DoD applies? | Notes |
|---|---|---|---|
| story | yes | yes | default |
| spike | partial (DOR-001/002/003/007 only) | n/a | timeboxed |
| bug | yes | yes | DOR-006 design optional |
| epic | yes (rolled-up) | applies to constituent stories | — |

## 4. Waivers and Exceptions

Waivers are allowed in extremis. Each waiver MUST record:

- `waived_by: @<operator-handle>`
- `reason: <free text — usually a regulatory / customer-pressure / incident-recovery rationale>`
- `expires_at: <ISO date — waivers are not permanent>`

## 5. Review Cadence

This document is reviewed at: `next_review_date: 2026-MM-DD` (default: quarterly). Review triggers re-confirmation by the four `approved_by` roles.

<!-- ── Conditionally-required additions (uncomment + fill per engagement) ── -->
<!-- COND-001 fixed_price → add "stage-gate sign-off captured" to DoD -->
<!-- COND-002 personal data → DOD must include "privacy review passed"; DOR must include "data class flagged" -->
<!-- COND-003 AI-driven → DOD must include "AI-use disclosure label on PR"; DOR must include "EU AI Act class assessed" -->
<!-- COND-005 safety-critical → DOD must include "hazard analysis updated"; DOR must include "safety case applies" -->
