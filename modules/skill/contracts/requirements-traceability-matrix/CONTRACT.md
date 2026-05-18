---
contract_id: rtm
contract_version: v1
template_literal: requirements-traceability-matrix@1
description: Canonical requirements-traceability-matrix@1 — Requirements Traceability Matrix per SDP Template §4.4. Authored by requirements-traceability-matrix-author; validated by requirements-traceability-matrix-audit via rtm_rubric@1.0. Auto-regenerated continuously.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cpo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: true, fixity_notes: "Matrix is fully derivable from source_set; byte-stable given identical source set + hashes." }
emitted_source_freshness_tier: 8
---

# `requirements-traceability-matrix@1` — canonical Requirements Traceability Matrix

> Frontmatter: `requirements-traceability-matrix-audit/RUBRIC.md` §2. Body: §3 (`SEC-001..006`) — summary / matrix (REQ-ID, Description, Source, Priority, Linked Design, Linked Code/PR, Linked Test, Status, Release) / orphans / untested / untraceable code / coverage stats.

## Citations

- SDP §3 + Template §4.4 — RTM source.
- Consumers: `requirements-traceability-matrix-author`, `requirements-traceability-matrix-audit`, downstream audit/governance reviews.
