# `chief-esg-officer` — Chief ESG Officer

> Per `../../../modules/cuo/docs/module.md` §5.7 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief ESG Officer.
- **Persona slug:** `chief-esg-officer`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Same as CSO-Sustainability (per §3 matrix).
- **One-sentence scope:** Variant overlapping CSO-Sustainability. Use this slug if firm prefers ESG framing (capital-market-facing); use cso-sustainability if operations-facing.

## §2-§4  Inputs
See C-Suite Reference §5.7. Expand in next session.

## §5  Outputs
Same as CSO-Sustainability with stronger capital-markets / investor-relations tilt.

## §6  Cadence
Per role profile in §5.7.

## §7  KPIs
Same as CSO-Sustainability.

## §8  Audit criteria
- **Quantitative:** moves KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** role-confusion with CSO-Sustainability; over-IR-pivot away from operational reality.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Same as CSO-Sustainability + IR portals (Q4).

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `annual-esg-report` | ESG strategy + performance + stakeholder + regulatory disclosures | annual | sustainability-report@1 | shipped (1.0.0) |
| `annual-esg-strategy` | Materiality + targets + investment + stakeholder engagement | annual | strategy-document@1 | shipped (1.0.0) |
| `quarterly-esg-compliance` | CSRD/ESRS + SEC climate + ISSB readiness | quarterly | compliance-program@1 (ESG chapter) | shipped (1.0.0) |
| `per-stakeholder-engagement` | Charter investor/NGO/community/ERG engagement programs | per-event | program-charter@1 | shipped (1.0.0) |

All workflows chain through shipped Tier-1/Tier-2 skills (`sustainability-report`, `strategy-doc`, `compliance-program`, `program-charter`). See `../../skill/MODULE.md` §3.1 + §3.2.

---

## Cross-references
- `../../../modules/cuo/docs/module.md` §5.7.
- `../MODULE.md` §4.
