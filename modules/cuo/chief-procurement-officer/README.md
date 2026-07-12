# `cpo-procurement` — Chief Procurement Officer (Procurement)

> Per `../../../modules/cuo/docs/module.md` §5.7 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Procurement Officer (Procurement).
- **Persona slug:** `cpo-procurement`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: — · Growth: optional · Enterprise: common (procurement-heavy industries) (per §3 matrix).
- **One-sentence scope:** Supply chain + vendor management at scale. **Acronym collision** with People, Product, Privacy.

## §2-§4  Inputs
See C-Suite Reference §5.7. Expand in next session.

## §5  Outputs
**Strategic:** procurement strategy; vendor-consolidation thesis; sustainable-procurement framework. **Operational:** vendor scorecards; spend reviews; contract negotiations. **Communication:** spend dashboard to CFO. **Team:** Sourcing + Vendor Mgmt + Procurement Ops.

## §6  Cadence
Per role profile in §5.7.

## §7  KPIs
| Cost-savings vs prior year | % | finance |
| Vendor consolidation count | trending down | vendor registry |
| Supplier sustainability coverage | % of spend | supplier scorecards |
| Contract cycle time | per category | CLM |

## §8  Audit criteria
- **Quantitative:** moves KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** single-vendor lock-in; aggressive cost cuts impacting quality; procurement-as-blocker.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Coupa / Ariba / Ivalua (S2P); Vendr / Tropic (SaaS-spend); EcoVadis (supplier sustainability).

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `annual-procurement-strategy` | Spend taxonomy + category playbooks + supplier base + savings targets | annual | procurement-strategy@1 | shipped (1.0.0) |
| `quarterly-supplier-scorecard` | Strategic supplier scoring: performance + cost + risk + sustainability | quarterly | vendor-scorecard@1 | shipped (1.0.0) |
| `per-category-sourcing-event` | Major category RFx charter | per-event | program-charter@1 | shipped (1.0.0) |
| `quarterly-savings-tracker` | Realized vs target savings by category and savings type | quarterly | procurement-strategy@1 (quarterly chapter) | shipped (1.0.0) |

All workflows chain through shipped Tier-1/Tier-2 skills (`procurement-strategy`, `vendor-scorecard`, `program-charter`). See `../../skill/MODULE.md` §3.1 + §3.2.

---

## Cross-references
- `../../../modules/cuo/docs/module.md` §5.7.
- `../MODULE.md` §4.
