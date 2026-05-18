---
contract_id: decomm
contract_version: v1
template_literal: decommissioning@1
description: Canonical decommissioning@1 — decommissioning / retirement package (data export + destruction certificate + DNS retirement + license cancellation + source archive). Authored by decommissioning-author; validated by decommissioning-audit via decomm_rubric@1.0.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cseco
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Per-decommission package; structural sections + destruction logs are stable." }
emitted_source_freshness_tier: 8
---

# `decommissioning@1` — canonical Decommissioning contract

> Frontmatter: `decommissioning-audit/RUBRIC.md` §2. Body: §3 (`SEC-001..012`) — decision + rationale / stakeholders / comms timeline / retention plan / export plan / destruction certificate / DNS retirement / license cancellation / source archive / final backup / runbook decomm / sign-off. Conditional: §4 — GDPR Art. 17 + Vietnam Decree 13/2023 / PCI-DSS / HIPAA / partner notifications / successor migration / refund policy.

## Citations

- SDP §2(m) — Decommissioning stage source.
- GDPR Article 17 — Right to Erasure.
- Vietnam Decree 13/2023 PDPD.
- PCI-DSS Requirement 9.8 — media destruction.
- HIPAA 45 CFR § 164.310(d)(2) — health-data disposal.
- Consumers: `decommissioning-author`, `decommissioning-audit`.
