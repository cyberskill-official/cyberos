# `cso-sustainability` — Chief Sustainability Officer (Sustainability)

> Per `../../docs/The C-Suite Reference.md` §5.7 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Sustainability Officer (Sustainability).
- **Persona slug:** `cso-sustainability`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: — · Growth: optional · Enterprise: common (EU/AU mandatory under CSRD) (per §3 matrix).
- **One-sentence scope:** ESG reporting (CSRD/ESRS in EU), Scope 1/2/3 emissions, sustainable supply chain.

## §2-§4  Inputs
See C-Suite Reference §5.7. Expand in next session.

## §5  Outputs
**Strategic:** sustainability strategy; net-zero roadmap; ESG-disclosure framework. **Operational:** emissions inventory; supplier sustainability scorecards; CSRD/ESRS / ISSB submissions. **Communication:** sustainability report (annual + integrated). **Team:** Sustainability Analysts + Emissions Engineers.

## §6  Cadence
Per role profile in §5.7.

## §7  KPIs
| Scope 1/2/3 emissions | per net-zero trajectory | Watershed / Persefoni |
| ESG rating | per agency (MSCI, Sustainalytics) | rating agency |
| Regulatory disclosure clean | yes | external audit |
| Supplier sustainability coverage | % of spend | supplier register |

## §8  Audit criteria
- **Quantitative:** moves KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** greenwashing (overclaiming); emission-accounting errors; Scope 3 blind spots; regulator-deadline misses.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Watershed / Persefoni / Sweep (carbon mgmt); Workiva (ESG reporting); EcoVadis (supplier scoring).

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `annual-emissions-inventory` | Scope 1/2/3 calculation + base-year + assurance readiness | annual | emissions-inventory@1 | shipped (1.0.0) |
| `annual-sustainability-report` | Env-specific report: emissions + water + waste + supply chain | annual | sustainability-report@1 | shipped (1.0.0) |
| `quarterly-target-tracking` | Net-zero + SBTi + renewable + circular targets vs baseline | quarterly | emissions-inventory@1 (quarterly chapter) | shipped (1.0.0) |
| `annual-sustainability-strategy` | Climate ambition + supply-chain + circularity + regenerative | annual | strategy-doc@1 | shipped (1.0.0) |

All workflows chain through shipped Tier-2 skills (`emissions-inventory`, `sustainability-report`, `strategy-doc`). See `../../skill/MODULE.md` §3.2.

---

## Cross-references
- `../../docs/The C-Suite Reference.md` §5.7.
- `../MODULE.md` §4.
