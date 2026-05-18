---
contract_id: runbook
contract_version: v1
template_literal: runbook@1
description: Canonical runbook@1 — operational runbook covering SLOs / on-call / common alerts / DR / observability. Authored by runbook-author; validated by runbook-audit via runbook_rubric@1.0.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cto
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Runbook content evolves with the service; structure is stable." }
emitted_source_freshness_tier: 9
---

# `runbook@1` — canonical Operational Runbook contract

> Frontmatter: `runbook-audit/RUBRIC.md` §2. Body: §3 (`SEC-001..010`) — service overview / SLOs+SLAs / error-budget policy / on-call rota / arch quick-ref / common alerts / common operations / observability / DR / vendor contacts. Conditional: §4 — personal data / multi-region / public API / payment-related.

## Citations

- SDP §2(j) — Operations stage source.
- Google SRE Book — SLOs, error budgets.
- OpenTelemetry — observability conventions.
- Consumers: `runbook-author`, `runbook-audit`, `postmortem-audit` (XCHAIN-003 — every services_affected has a runbook), `deployment-checklist-audit` (XCHAIN-005).
