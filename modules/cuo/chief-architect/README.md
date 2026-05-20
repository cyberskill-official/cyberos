# `chief-architect` — Chief Architect / Chief Software Architect

> Per `../../../modules/cuo/README.md` §5.3 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Architect / Chief Software Architect.
- **Persona slug:** `chief-architect`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: — · Series A: — · Scale-up: — · Growth: optional · Enterprise: common (infra-heavy) (per §3 matrix).
- **One-sentence scope:** Below CTO in most firms; rarely C-level except at infrastructure-heavy enterprises.

## §2-§4  Inputs (information / stakeholder / resource)
See C-Suite Reference §5.3 for the full input lists. Expand in next session.

## §5  Outputs
**Strategic:** target architecture; reference-architecture library; tech-radar curation. **Operational:** architecture review board; ADR adjudication. **Communication:** architecture decisions to engineering. **Team:** Principal Engineers; Staff Architects.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.3.

## §7  KPIs
| Architecture review pass rate | trending up | ARB log |
| Reference-architecture adoption | per major component | adoption tracking |
| Tech-radar moves per quarter | bounded (avoid churn) | radar |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** ivory-tower (architecture detached from implementer reality); rubber-stamp ARB; tech-radar fashion-chasing.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
C4 model + structurizr (diagrams); ThoughtWorks Tech Radar pattern; archi (TOGAF); ADR tools.

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `per-architecture-decision` | ADR per Nygard format with threat-model + SDD chain | per-event | architecture-decision-record@1 | shipped (1.0.0) |
| `annual-architecture-vision` | Reference architecture + principles + tech-radar + debt portfolio | annual | strategy-document@1 | shipped (1.0.0) |
| `per-threat-model-review` | Per-system STRIDE + ASVS + mitigations review | per-event | threat-model@1 | shipped (1.0.0) |
| `quarterly-design-review` | Cross-SDD review: design coherence + ADR alignment + NFR coverage | quarterly | software-design-document@1 (review chapter) | shipped (1.0.0) |

All workflows chain through shipped SDP-original + Tier-2 skills (`adr`, `threat-model`, `sdd`, `strategy-doc`). See `../../skill/MODULE.md` §3 + §3.2.

---

## Cross-references
- `../../../modules/cuo/README.md` §5.3 — source role profile.
- `../MODULE.md` §4.
