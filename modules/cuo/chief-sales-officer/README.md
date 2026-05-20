# `cso-sales` — Chief Sales Officer (Sales)

> Per `../../../modules/cuo/README.md` §5.4 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Sales Officer (Sales).
- **Persona slug:** `cso-sales`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: VP Sales · Series A: VP Sales · Scale-up: optional · Growth: common (sometimes absorbed by CRO) · Enterprise: under CRO (per §3 matrix).
- **One-sentence scope:** Pure sales leadership; often reports to CRO. **Acronym collision** with Strategy, Security, Sustainability.

## §2-§4  Inputs (information / stakeholder / resource)
See C-Suite Reference §5.4 for the full input lists. Expand in next session.

## §5  Outputs
**Strategic:** territory design; comp-plan calibration; sales-methodology adoption. **Operational:** pipeline reviews; deal escalations; forecast calls; sales-rep performance. **Communication:** sales kickoff; QBR for top accounts. **Team:** AE / SDR / Sales-Engineering / Sales-Ops.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.4.

## §7  KPIs
| Quota attainment % of reps | > 70 % | CRM |
| Sales-cycle length | per segment target | CRM |
| Pipeline coverage | 3-4× quota | CRM |
| Win rate | per segment | CRM |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** hero-rep dependence; sales process not adopted; territory disputes; sandbagging-then-pull-forward.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Salesforce / HubSpot (CRM); Gong / Chorus (call intel); Clari / Aviso (forecast); Spiff / CaptivateIQ (comp).

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `weekly-pipeline-review` | Stage health + slippage + commit-vs-quota forecast | weekly | pipeline-report@1 | shipped (1.0.0) |
| `quarterly-account-plan` | Stakeholder map + white-space + growth thesis (per top account) | quarterly | account-plan@1 | shipped (1.0.0) |
| `annual-gtm-plan` | ICP + segmentation + channel strategy + quota model | annual | go-to-market-plan@1 | shipped (1.0.0) |
| `quarterly-nps-program` | Customer NPS survey + root-cause + action plan | quarterly | net-promoter-score-program@1 | shipped (1.0.0) |

All workflows chain through shipped skills — Tier-1 (`pipeline-report`, `account-plan`, `nps-program`) + Tier-5 (`gtm-plan`). See `../../skill/MODULE.md` §3.1 + §3.5.

---

## Cross-references
- `../../../modules/cuo/README.md` §5.4 — source role profile.
- `../MODULE.md` §4.
