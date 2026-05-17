---
contract_id: deploy-checklist
contract_version: v1
template_literal: deploy-checklist@1
description: Canonical deploy-checklist@1 — Deployment Readiness Checklist per SDP Template §4.7. Authored by deploy-checklist-author; validated by deploy-checklist-audit via deploy_checklist_rubric@1.0.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cto
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Per-release artefact; DORA baseline values change every deploy." }
emitted_source_freshness_tier: 10
---

# `deploy-checklist@1` — canonical Deployment Readiness Checklist

> Frontmatter: `deploy-checklist-audit/RUBRIC.md` §2. Body: §3 (`DEP-001..012`) — DoDs / release notes / rollback / feature flags / migrations / monitoring / on-call / scans / change ticket / SBOM / DORA baseline / signed artefacts. Conditional: §4 (`COND-001..006`) — production / breaking / large migration / regulated / canary / AI-model update.

## Citations

- SDP §2(i) + Template §4.7.
- DORA four key metrics — captured at deploy time for `DEP-011`.
- OWASP Top 10:2025 A08 Software & Data Integrity Failures — drives `DEP-012`.
- Consumers: `deploy-checklist-author`, `deploy-checklist-audit`.
