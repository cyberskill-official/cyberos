---
contract_id: threat-model
contract_version: v1
template_literal: threat-model@1
description: Canonical threat-model@1 — STRIDE threat model with OWASP Top 10:2025 + ASVS mapping. Authored by threat-model-author; validated by threat-model-audit via threat_model_rubric@1.0. Implements modules/cuo/README.md#software-development-process §2(d) security review.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cseco
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }

determinism: { reproducible: false, fixity_notes: "Threat enumeration is judgement-heavy; STRIDE categories + ASVS controls are reproducible." }

emitted_source_freshness_tier: 10
---

# `threat-model@1` — canonical STRIDE threat-model contract

> Frontmatter: `threat-model-audit/RUBRIC.md` §2. Required sections: §3 (`SEC-001..014`) — system overview, trust boundaries, DFD, threats by STRIDE category (six H3s), OWASP Top 10:2025 coverage, ASVS controls, residual risk, mitigations + linked ADRs. Conditional: §4 (`COND-001..004`) — personal data (LINDDUN), AI/ML, public API, ASVS L3.

## Citations

- STRIDE (Microsoft) — primary threat-modelling framework.
- LINDDUN (KU Leuven) — privacy threat-modelling for COND-001.
- OWASP Top 10:2025, OWASP ASVS — mapping sources.
- Consumers: `threat-model-author`, `threat-model-audit`, `architecture-decision-record-audit` (XCHAIN-004), `code-review-audit` (COND-002 references threat-model entries).
