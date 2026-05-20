# `chief-innovation-officer` — Chief Innovation Officer

> Per `../../../modules/cuo/README.md` §5.7 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Innovation Officer.
- **Persona slug:** `chief-innovation-officer`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: — · Growth onward: optional · Enterprise: common (innovation-led firms) (per §3 matrix).
- **One-sentence scope:** R&D portfolio, new-venture incubation.

## §2-§4  Inputs
See C-Suite Reference §5.7. Expand in next session.

## §5  Outputs
**Strategic:** innovation portfolio; new-venture incubation pipeline; corporate-VC theses (if applicable). **Operational:** kill/scale decisions per venture. **Communication:** innovation pipeline value to board. **Team:** Innovation Scouts + Venture Builders + Incubator PMs.

## §6  Cadence
Per role profile in §5.7.

## §7  KPIs
| Innovation pipeline value | per portfolio target | portfolio mgmt |
| % revenue from new products | per growth target | finance |
| Venture-success rate | per portfolio benchmark | venture tracker |

## §8  Audit criteria
- **Quantitative:** moves KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** innovation-theatre; failure-to-kill (zombie projects); pipeline-without-paying-customers.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Pipefy / Wellspring (innovation portfolio); Brightidea / IdeaScale (ideation); custom venture-tracking.

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `annual-innovation-portfolio` | Horizons 1/2/3 + investment thesis per bet + stage-gates + kill criteria | annual | innovation-portfolio@1 | shipped (1.0.0) |
| `per-innovation-charter` | Per-bet (H2/H3) charter with hypothesis + experiment + stage-gates | per-event | program-charter@1 | shipped (1.0.0) |
| `quarterly-portfolio-review` | Stage-gate decisions + kill / graduate / pivot recommendations | quarterly | innovation-portfolio@1 chapter | shipped (1.0.0) |
| `annual-innovation-strategy` | Moonshot vision + operating model + partnership + innovation OKRs | annual | strategy-document@1 | shipped (1.0.0) |

All workflows chain through shipped Tier-2 skills (`innovation-portfolio`, `program-charter`, `strategy-doc`). See `../../skill/MODULE.md` §3.2.

---

## Cross-references
- `../../../modules/cuo/README.md` §5.7.
- `../MODULE.md` §4.
