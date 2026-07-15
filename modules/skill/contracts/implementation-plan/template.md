---
template: impl_plan@1
title: <Implementation plan name>
author: @<author-handle>
created_at: <ISO 8601 with timezone>
last_updated_at: <ISO 8601 with timezone>
tech_spec_ref: ../tech-specs/TS-NNN-task-MMM-<slug>.md
target_release: 2026-Q3
proj_backend: linear            # linear | jira | github | none
tickets_created: false
total_tickets: 0
total_estimated_engineer_days: 0
chain_profile: standard
---

# <Implementation plan name>

## Background

Source tech-spec: [<title>](<tech_spec_ref>).

## Tickets

| # | Title | Sizing | Dependencies | PROJ ticket | Acceptance criteria ref |
| --- | --- | --- | --- | --- | --- |
| 1 | <ticket title> | M | — | (pending) | tech-spec §"Test plan" item 1 |
| 2 | <ticket title> | S | T1 | (pending) | tech-spec §"Test plan" item 2 |

## Sprint Suggestion

- **Sprint 1:** Tickets #1, #2, #3 (~6 engineer-days; no external blockers)
- **Sprint 2:** Tickets #4, #5 (~5 engineer-days; depends on Sprint 1 #2)

## Risks

| Risk | Mitigation |
| --- | --- |
| <risk> | <mitigation> |

## Open Questions

1. <question with `<!-- needs: <persona|human> -->` marker>

<!-- Or: "No open questions — all tickets ready for sprint planning." -->

<!--
## Ticket Index (auto-generated when tickets_created: true)
| # | PROJ ticket ID | URL |
| --- | --- | --- |
| 1 | LIN-1234 | https://linear.app/cyberskill/issue/LIN-1234 |

## Architecture Note (required when chain_profile: lean)
Lean profile skipped tech-spec authoring. Architectural assumptions:
- ...
- ...
-->
