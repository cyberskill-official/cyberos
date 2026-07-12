# `cdo-data` — Chief Data Officer (Data)

> Per `../../../modules/cuo/docs/module.md` §5.3 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Data Officer (Data).
- **Persona slug:** `cdo-data`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: — · Series A: — · Scale-up: Head of Data · Growth: common · Enterprise: ESSENTIAL (per §3 matrix).
- **One-sentence scope:** Data strategy, governance, analytics, increasingly AI. **Acronym collision:** CDO can also mean Digital or Diversity — this folder is the Data CDO.

## §2-§4  Inputs (information / stakeholder / resource)
See C-Suite Reference §5.3 for the full input lists. Expand in next session.

## §5  Outputs
**Strategic:** data strategy + data-product roadmap; data-governance framework; ML platform vision. **Operational:** data-quality dashboards; data-product launches; model deployment pipeline. **Communication:** board data chapter; data-product RFCs. **Team:** Data Engineering; Analytics; Data Science; MLOps.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.3.

## §7  KPIs
| Data-quality score | > 0.95 per critical dataset | data observability |
| Time-to-insight | < 24 h for standard analyst Q | analytics platform |
| Model deployment count | per quarter target | MLOps tool |
| Data-product revenue | per business plan | finance attribution |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** data swamp (lake without governance); model-deploy cycle stuck at 6+ months; data-products without paying customers; LLM hallucination liability.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Snowflake / BigQuery / Databricks (warehouse); dbt / Fivetran / Airbyte (ELT); Looker / Mode / Hex (BI); MLflow / Weights & Biases (MLOps); Monte Carlo / Bigeye (data observability); Collibra / Atlan (governance).

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `annual-data-strategy` | Domains + data products + governance + infrastructure + team operating model | annual | data-strategy@1 | shipped (1.0.0) |
| `per-data-product-release` | Schema + SLA + lineage + consumer onboarding + deprecation policy | per-event | data-product@1 | shipped (1.0.0) |
| `quarterly-data-governance-review` | Quality + access + lineage + MDM + policy adherence | quarterly | data-governance@1 | shipped (1.0.0) |
| `annual-customer-360-architecture` | Identity resolution + master entity + activation surfaces + consent | annual | customer-360@1 | shipped (1.0.0) |

All workflows chain through shipped skills — Tier-2 (`data-strategy`, `data-product`, `data-governance`) + Tier-7 (`customer-360`). See `../../skill/MODULE.md` §3.2 + §3.7.

---

## Cross-references
- `../../../modules/cuo/docs/module.md` §5.3 — source role profile.
- `../MODULE.md` §4.
