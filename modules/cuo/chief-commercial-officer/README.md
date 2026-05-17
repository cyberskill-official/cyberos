# `cco-commercial` — Chief Commercial Officer (Commercial)

> Per `../../docs/The C-Suite Reference.md` §5.4 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Commercial Officer (Commercial).
- **Persona slug:** `cco-commercial`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: — · Series A: — · Scale-up: common in B2B services · Growth: common · Enterprise: common (per §3 matrix).
- **One-sentence scope:** Sales + partnerships + sometimes marketing; common in B2B services. **Acronym collision** with Compliance, Customer, Communications.

## §2-§4  Inputs (information / stakeholder / resource)
See C-Suite Reference §5.4 for the full input lists. Expand in next session.

## §5  Outputs
**Strategic:** GTM strategy; partnership portfolio; pricing for services. **Operational:** deal escalations; partner-program management. **Communication:** board commercial chapter; partner QBRs. **Team:** Sales + Partnerships + Sales-Enablement.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.4.

## §7  KPIs
| Bookings | per quota | CRM |
| Win rate | per segment | CRM |
| Deal size avg | per segment | CRM |
| Partner-sourced revenue | per partner-program goal | CRM + PRM |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** partner-program over-investment without partner revenue; sales-process bloat; pricing-without-data.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Salesforce / HubSpot (CRM); Crossbeam / Reveal (PRM); Outreach / Salesloft (engagement); Pricing tools (PROS, Vendavo).

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `annual-partner-program` | Partner taxonomy + tiering + enablement + economics + joint GTM | annual | partner-program@1 | shipped (1.0.0) |
| `per-strategic-partnership` | Per-partner charter: profile + value + joint-investment + governance | per-event | program-charter@1 | shipped (1.0.0) |
| `quarterly-partner-scorecard` | Pipeline + attribution + tier promotion/demotion | quarterly | partner-program@1 (quarterly chapter) | shipped (1.0.0) |
| `annual-channel-strategy` | Direct vs partner vs marketplace mix + conflict policy | annual | strategy-doc@1 | shipped (1.0.0) |

All workflows chain through shipped Tier-2 skills (`partner-program`, `program-charter`, `strategy-doc`). See `../../skill/MODULE.md` §3.2.

---

## Cross-references
- `../../docs/The C-Suite Reference.md` §5.4 — source role profile.
- `../MODULE.md` §4.
