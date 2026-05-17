---
contract_id: release-notes
contract_version: v1
template_literal: release-notes@1
description: Canonical release-notes@1 — customer-facing release notes in Keep-a-Changelog 1.1.0 + SemVer 2.0.0 format. Authored by release-notes-author; validated by release-notes-audit via release_notes_rubric@1.0.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cpo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Notes content is judgement; ordering + section set are reproducible." }
emitted_source_freshness_tier: 12
---

# `release-notes@1` — canonical Release Notes contract

> Frontmatter: `release-notes-audit/RUBRIC.md` §2. Body: §3 (Keep-a-Changelog ordering — Highlights / Added / Changed / Deprecated / Removed / Fixed / Security / Upgrade Notes / Known Issues). Conditional: §4 — breaking change, CVE patched, audience scope.

## Citations

- Keep-a-Changelog 1.1.0.
- SemVer 2.0.0.
- NVD / MITRE CVE format — prevents fabricated CVE IDs per `QA-CVE-001`.
- Consumers: `release-notes-author`, `release-notes-audit`, downstream `deploy-checklist-author` (DEP-002 references the audited release notes).
