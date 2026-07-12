---
contract_id: project-plan
contract_version: v1
template_literal: project-plan@1
description: Canonical project-plan@1 — feasibility memo + project plan + RAID log + communication plan emitted by project-plan-author. Validated by project-plan-audit via project_plan_rubric@1.0. Implements modules/cuo/docs/appendices.md (§13 Software Development Process) §2(c).
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cpo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }

determinism: { reproducible: false, fixity_notes: "Plan content reflects scheduling + cost judgement; contract body shape is byte-stable." }

emitted_source_freshness_tier: 15
---

# `project-plan@1` — canonical project plan contract

> Frontmatter contract: see `project-plan-audit/RUBRIC.md` §2 (`FM-101..111`).
> Required body sections: `project-plan-audit/RUBRIC.md` §3 (`SEC-001..010`).
> Conditional sections by `governance_framework` (PMBOK 8 / PRINCE2 7) + `engagement_model` + regulated-domain: `COND-001..004`.

## Citations

- SDP §2(c) — Feasibility + project planning source.
- PMBOK 8th Edition — eight performance domains (May 2026 release).
- PRINCE2 7th Edition — five integrated elements (Issues replaces Change).
- Consumers: `project-plan-author`, `project-plan-audit`, downstream `stage-gate-author` (each gate references the plan).
