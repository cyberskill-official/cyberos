---
contract_id: monthly-close
contract_version: v1
template_literal: monthly-close@1
description: Canonical monthly-close@1 — monthly-close artefact authored by cfo / cao-accounting; cycle-time < 10 business days.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `monthly-close@1` — canonical Monthly Close Package contract

> Frontmatter: `monthly-close-audit/RUBRIC.md` §2.
> Required body sections: §3 (close calendar adherence / GL completeness / reconciliation status / journal entries posted / variance commentary / audit-trail completeness).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves close cycle-time + audit findings KPIs.

## Citations

- C-Suite Reference §5.2 (CFO close output; CAO-Accounting controller-level)
- BlackLine / Floqast close discipline
- Consumers: `monthly-close-author`, `monthly-close-audit`.
