# `cro-restructuring` — Chief Restructuring Officer (Restructuring)

> Per `../../../modules/cuo/README.md` §5.7 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Restructuring Officer (Restructuring).
- **Persona slug:** `cro-restructuring`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Crisis only — at every stage when distressed (per §3 matrix).
- **One-sentence scope:** Crisis-only; appointed during distress / turnaround, often interim. **Acronym collision** with Revenue, Risk.

## §2-§4  Inputs
See C-Suite Reference §5.7. Expand in next session.

## §5  Outputs
**Strategic:** turnaround plan; capital restructuring; non-core divestiture roadmap. **Operational:** cost-out execution; vendor renegotiation; cash conservation. **Communication:** creditor + board comms; employee comms in restructuring. **Team:** typically embeds with existing CFO/COO; brings 2-5 turnaround specialists.

## §6  Cadence
Per role profile in §5.7.

## §7  KPIs
| Cash burn reduction | per restructuring plan | finance |
| Debt-restructure close | by deadline | legal / finance |
| Employee retention through restructuring | per retention plan | HR |

## §8  Audit criteria
- **Quantitative:** moves KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** cost-cuts-into-the-bone (kills the business); creditor-comms breakdown; key-employee flight.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
AlixPartners / FTI / A&M / Alvarez (turnaround toolkits); 13-week-cash-flow models.

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `per-turnaround-plan` | Situation + stabilization + value creation + exit | per-event | turnaround-plan@1 | shipped (1.0.0) |
| `weekly-cash-flow` | Distress-mode TWCF with same-day actuals reconciliation | weekly | thirteen-week-cash-flow@1 | shipped (1.0.0) |
| `per-stakeholder-communication` | Distress stakeholder comms (lenders/employees/customers/suppliers/board) | per-event | decision-log@1 | shipped (1.0.0) |
| `monthly-restructuring-update` | Monthly status: plan progress + milestones + covenant + stakeholders | monthly | turnaround-plan@1 (monthly chapter) | shipped (1.0.0) |

All workflows chain through shipped Tier-1/Tier-3 skills (`turnaround-plan`, `13-week-cash-flow`, `decision-log`). See `../../skill/MODULE.md` §3.1 + §3.3.

---

## Cross-references
- `../../../modules/cuo/README.md` §5.7.
- `../MODULE.md` §4.
