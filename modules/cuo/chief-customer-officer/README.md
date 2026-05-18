# `cco-customer` — Chief Customer Officer (Customer)

> Per `../../docs/The C-Suite Reference.md` §5.4 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Customer Officer (Customer).
- **Persona slug:** `cco-customer`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: — · Series A: — · Scale-up: optional · Growth: common · Enterprise: common (consulting-firm-specific high-ROI) (per §3 matrix).
- **One-sentence scope:** Owns post-sale CX, success, support. Critical for consulting + SaaS firms (NRR is the moat).

## §2-§4  Inputs (information / stakeholder / resource)
See C-Suite Reference §5.4 for the full input lists. Expand in next session.

## §5  Outputs
**Strategic:** customer success strategy; expansion playbook; churn-prevention plan. **Operational:** NPS program; QBR coordination; account expansion. **Communication:** customer advisory board. **Team:** CSMs + Support + Customer Marketing.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.4.

## §7  KPIs
| NRR | > 110 % | CRM |
| Churn rate | < industry benchmark | CRM |
| CSAT | > 85 % | survey |
| NPS | > 30 | survey |
| Expansion revenue | per account-mgmt target | CRM |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** reactive-only customer success; account-management without health-score visibility; QBR-theatre (going through motions without expansion plan).
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Gainsight / ChurnZero / Catalyst (CS platform); Zendesk / Intercom / Front (support); Pendo / Appcues (in-product); Salesforce (CRM).

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `quarterly-customer-health-review` | Health distribution + at-risk + expansion + advocacy + CSM utilization | quarterly | customer-health-review@1 | shipped (1.0.0) |
| `per-account-cs-engagement` | Per-account relationship map + success criteria + cadence + expansion thesis | per-event | customer-success-engagement@1 | shipped (1.0.0) |
| `quarterly-cab-cycle` | CAB agenda + curated attendees + synthesis + commitments | quarterly | customer-advisory-board@1 | shipped (1.0.0) |
| `quarterly-churn-collaboration` | CS-side churn root-cause partner workflow to CRO-Revenue churn analysis | quarterly | churn-analysis@1 (CS-augmented) | shipped (1.0.0) |

All workflows chain through shipped skills — Tier-1 (`cs-engagement`) + Tier-2 (`customer-advisory-board`) + Tier-6 (`churn-analysis`) + Tier-7 (`customer-health-review`). See `../../skill/MODULE.md` §3.1 + §3.2 + §3.6 + §3.7.

---

## Cross-references
- `../../docs/The C-Suite Reference.md` §5.4 — source role profile.
- `../MODULE.md` §4.
