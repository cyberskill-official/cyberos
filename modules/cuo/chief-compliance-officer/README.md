# `cco-compliance` — Chief Compliance Officer (Compliance)

> Per `../../../modules/cuo/docs/module.md` §5.6 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Compliance Officer (Compliance).
- **Persona slug:** `cco-compliance`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: — · Series A: — · Scale-up: if regulated · Growth: common · Enterprise: ESSENTIAL (per §3 matrix).
- **One-sentence scope:** Designs compliance management system; personal liability rising. **Acronym collision** with Commercial, Customer, Communications.

## §2-§4  Inputs (information / stakeholder / resource)
See C-Suite Reference §5.6 for the full input lists. Expand in next session.

## §5  Outputs
**Strategic:** compliance management system; regulator-mapping; risk-control matrix. **Operational:** compliance training; regulator submissions; policy currency; incident response. **Communication:** board compliance chapter; regulator liaison. **Team:** Compliance Officers + Policy Managers + Training Coordinators.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.6.

## §7  KPIs
| Regulatory findings | 0 | external audit |
| Training completion | > 95 % | LMS |
| Policy currency | 100 % within review-cycle | policy register |
| Incident closure SLA | per severity tier | incident tracker |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** policy-bloat without enforcement; training-theatre (clicks without comprehension); reactive-only mode.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Workiva / ServiceNow GRC / LogicGate (GRC); KnowBe4 / SAI360 (training); contract-CLM systems for regulatory addenda.

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `annual-compliance-program` | Regulations + control framework + training + monitoring + escalations | annual | compliance-program@1 | shipped (1.0.0) |
| `quarterly-control-testing` | Control sample + test results + remediation tracking | quarterly | compliance-program@1 (testing chapter) | shipped (1.0.0) |
| `per-regulatory-filing` | Compliance-driven filing (vs litigation-driven from CLO-Legal) | per-event | regulatory-filing@1 | shipped (1.0.0) |
| `quarterly-board-compliance-chapter` | Compliance chapter for quarterly board deck | quarterly | compliance-program@1 (board chapter) | shipped (1.0.0) |

All workflows chain through shipped skills — Tier-1 (`compliance-program`) + Tier-4 (`regulatory-filing`). See `../../skill/MODULE.md` §3.1 + §3.4.

---

## Cross-references
- `../../../modules/cuo/docs/module.md` §5.6.
- `../MODULE.md` §4.
