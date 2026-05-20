# `cro-risk` — Chief Risk Officer (Risk)

> Per `../../../modules/cuo/README.md` §5.6 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Risk Officer (Risk).
- **Persona slug:** `cro-risk`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: — · Series A: — · Scale-up: if FS/regulated · Growth: common · Enterprise: ESSENTIAL (per §3 matrix).
- **One-sentence scope:** Enterprise risk framework; risk appetite vs tolerance. **Different from CCO-Compliance:** CRO-Risk sets *how much* risk; CCO-Compliance ensures *legality*.

## §2-§4  Inputs (information / stakeholder / resource)
See C-Suite Reference §5.6 for the full input lists. Expand in next session.

## §5  Outputs
**Strategic:** enterprise risk framework; risk-appetite statement; scenario library. **Operational:** KRI dashboards; risk-incident review; stress-tests. **Communication:** board risk chapter. **Team:** Risk Analysts + Stress-Test Owners.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.6.

## §7  KPIs
| KRI breach count | trending 0 | risk dashboard |
| Capital adequacy (FS) | per regulator | finance |
| Scenario-test coverage | 100 % per cycle | risk register |
| Risk-incident MTTR | per severity tier | incident tracker |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** risk-register-without-action; siloed from operational reality; over-conservative (paralyses growth) or under-conservative (paralyses operations).
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Workiva / Archer / Riskonnect (ERM); BlackRock Aladdin / Numerix (FS scenario engines); custom Looker dashboards.

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `annual-erm-framework` | Risk taxonomy + appetite + risk-and-control matrix + governance | annual | enterprise-risk-framework@1 | shipped (1.0.0) |
| `quarterly-kri-dashboard` | KRI thresholds + breach analysis + escalations + trend | quarterly | key-risk-indicator-dashboard@1 | shipped (1.0.0) |
| `per-incident-postmortem` | Risk-lens postmortem with control-failure attribution | per-event | postmortem@1 (risk lens) | shipped (1.0.0) |
| `quarterly-board-risk-chapter` | Risk chapter for quarterly board deck | quarterly | board-deck@1 chapter | shipped (1.0.0) |

All workflows chain through shipped skills — Tier-2 (`enterprise-risk-framework`, `kri-dashboard`) + SDP-original (`postmortem`) + Tier-1 (`board-deck`). See `../../skill/MODULE.md` §3 + §3.1 + §3.2.

---

## Cross-references
- `../../../modules/cuo/README.md` §5.6.
- `../MODULE.md` §4.
