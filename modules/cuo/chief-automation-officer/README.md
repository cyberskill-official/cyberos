# `chief-automation-officer` — Chief Automation Officer

> Per `../../docs/The C-Suite Reference.md` §5.7 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Automation Officer.
- **Persona slug:** `chief-automation-officer`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Ops-heavy firms; usually absorbed by CIO/CTO (per §3 matrix).
- **One-sentence scope:** Emerging in ops-heavy firms; usually absorbed by CIO/CTO.

## §2-§4  Inputs
See C-Suite Reference §5.7. Expand in next session.

## §5  Outputs
**Strategic:** automation roadmap (RPA + AI + process); ROI-tracking framework. **Operational:** bot-deploy reviews; ROI tracking. **Communication:** automation savings dashboard. **Team:** RPA Devs + Process Mining Analysts + AI Engineers.

## §6  Cadence
Per role profile in §5.7.

## §7  KPIs
| Automated hours / quarter | per ROI target | RPA platform |
| Automation ROI | > 3x | finance |
| Process-mining coverage | % of major processes | process-mining tool |

## §8  Audit criteria
- **Quantitative:** moves KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** automation-theatre; bot-sprawl without governance; process-mining without action.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
UiPath / Automation Anywhere / Microsoft Power Automate (RPA); Celonis / SAP Signavio (process mining).

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `annual-automation-roadmap` | RPA pipeline + AI-augmented automation + hyper-automation + ROI | annual | automation-roadmap@1 | shipped (1.0.0) |
| `per-automation-charter` | Per-process: process-mining + design + ROI + change-mgmt | per-event | program-charter@1 | shipped (1.0.0) |
| `quarterly-automation-portfolio-review` | Bot health + ROI realization + failure modes | quarterly | automation-roadmap@1 (quarterly chapter) | shipped (1.0.0) |
| `annual-operating-model-impact` | Annual assessment of automation impact on operating model | annual | operating-model@1 (automation chapter) | shipped (1.0.0) |

All workflows chain through shipped Tier-1/Tier-2 skills (`automation-roadmap`, `program-charter`, `operating-model`). See `../../skill/MODULE.md` §3.1 + §3.2.

---

## Cross-references
- `../../docs/The C-Suite Reference.md` §5.7.
- `../MODULE.md` §4.
