---
contract_id: closure
contract_version: v1
template_literal: closure@1
description: Canonical closure@1 — project closure package (sign-off + lessons + KT + asset handover). Authored by closure-author; validated by closure-audit via closure_rubric@1.0. Implements SDP §2(l).
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cpo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Closure content compiles from many sources; structural set + sign-off table are stable." }
emitted_source_freshness_tier: 10
---

# `closure@1` — canonical Project Closure contract

> Frontmatter: `closure-audit/RUBRIC.md` §2. Body: §3 (`SEC-001..012`) — sign-off cert / deliverables accepted / lessons / KT / source-code handover / runbook handover / credentials rotation / asset handover / closure metrics / NPS / surviving obligations / next-steps. Conditional: §4 — people offboarding / data disposition / warranty notice / managed-services disengagement.

## Citations

- SDP §2(l) — Closure stage source.
- SDP §6 — Offboarding pack source.
- Consumers: `closure-author`, `closure-audit`, downstream `decomm-author` (if system is being retired).
