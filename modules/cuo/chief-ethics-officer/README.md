# `chief-ethics-officer` — Chief Ethics Officer / Chief AI Ethics Officer

> Per `../../../modules/cuo/README.md` §5.6 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Ethics Officer / Chief AI Ethics Officer.
- **Persona slug:** `chief-ethics-officer`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: — · Series A: — · Scale-up: — · Growth: emerging · Enterprise: emerging (per §3 matrix).
- **One-sentence scope:** Emerging; oversees responsible-AI policy, bias audits, model cards. Often the same person owns AI ethics + broader business ethics in mid-market firms.

## §2-§4  Inputs (information / stakeholder / resource)
See C-Suite Reference §5.6 for the full input lists. Expand in next session.

## §5  Outputs
**Strategic:** ethics framework; AI-ethics policy; ethics-review-board charter. **Operational:** ethics review reviews; bias-test reviews; model-card audits; whistleblower line. **Communication:** ethics report (annual); model-card library publication. **Team:** Ethics Reviewers + Model-Card Authors + Compliance crossovers.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.6.

## §7  KPIs
| Ethics-review coverage | 100 % high-risk AI features | review tracker |
| Bias-test pass rate | per fairness threshold | bias-test framework |
| AI incident count | 0 | incident tracker |
| Whistleblower-line response time | per policy SLA | whistleblower platform |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** ethics-theatre without action; bias-test-without-remediation; model-card debt; failed engagement with AI engineers.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Aporia / Fiddler (bias monitoring); Hugging Face model cards; ETHX (research); LawRoom / NAVEX (ethics training + whistleblower).

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `per-use-case-ethics-review` | Stakeholder + harm + fairness review with PROCEED/MODIFY/DECLINE recommendation | per-event | ethics-review@1 | shipped (1.0.0) |
| `quarterly-bias-portfolio-audit` | Aggregate disparate-impact pattern across all production models | quarterly | bias-audit@1 (portfolio) | shipped (1.0.0) |
| `per-model-card-ethics-sign-off` | Ethics sign-off on model-card limits / fairness / scope sections | per-event | model-card@1 (with ethics signoff) | shipped (1.0.0) |
| `annual-ethics-program` | Values + decision rights + training + transparency commitments | annual | compliance-program@1 (ethics chapter) | shipped (1.0.0) |

All workflows chain through shipped skills — Tier-2 (`ethics-review`) + Tier-1 (`bias-audit`, `model-card`, `compliance-program`). See `../../skill/MODULE.md` §3.1 + §3.2.

---

## Cross-references
- `../../../modules/cuo/README.md` §5.6.
- `../MODULE.md` §4.
