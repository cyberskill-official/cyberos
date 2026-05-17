---
contract_id: rtm
contract_version: v1
template_literal: rtm@1
description: Canonical rtm@1 — Requirements Traceability Matrix per SDP Template §4.4. Authored by rtm-author; validated by rtm-audit via rtm_rubric@1.0. Auto-regenerated continuously.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cpo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: true, fixity_notes: "Matrix is fully derivable from source_set; byte-stable given identical source set + hashes." }
emitted_source_freshness_tier: 8
---

# `rtm@1` — canonical Requirements Traceability Matrix

> Frontmatter: `rtm-audit/RUBRIC.md` §2. Body: §3 (`SEC-001..006`) — summary / matrix (REQ-ID, Description, Source, Priority, Linked Design, Linked Code/PR, Linked Test, Status, Release) / orphans / untested / untraceable code / coverage stats.

## Citations

- SDP §3 + Template §4.4 — RTM source.
- Consumers: `rtm-author`, `rtm-audit`, downstream audit/governance reviews.
