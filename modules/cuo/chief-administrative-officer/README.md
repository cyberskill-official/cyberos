# `cao-admin` — Chief Administrative Officer (Administrative)

> Per `../../../modules/cuo/docs/module.md` §5.1 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Administrative Officer (Administrative).
- **Persona slug:** `cao-admin`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Rare; declining; mostly enterprise legacy (per §3 matrix).
- **One-sentence scope:** Back-office aggregator: facilities, admin, sometimes HR/IT in older firms. Increasingly merged or eliminated.

## §2  Information inputs
See C-Suite Reference §5.1 for full input list (dashboards, reports, market intel, customer signals, internal signals). Expand in next session.

## §3  Stakeholder inputs
CEO / board mandates; peer-C-suite asks; customer + regulator signals as applicable. See §5.1.

## §4  Resource inputs
Budget envelope, headcount, tooling spend authority per role profile. See §5.1.

## §5  Outputs
**Strategic:** back-office consolidation roadmap. **Operational:** facilities / admin / records mgmt. **Team:** admin function leaders.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.1.

## §7  KPIs
| Admin cost as % revenue | declining trend | finance |
| Records compliance | 100 % | audit |
| Facilities utilisation | per workplace model | facilities mgmt |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** role-bloat (absorbs everything no one else wants); irrelevance in remote-first orgs.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
ERP + workplace mgmt + records mgmt systems.

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `annual-administrative-operating-model` | Back-office org + processes + governance + shared-services | annual | operating-model@1 | shipped (1.0.0) |
| `weekly-back-office-cadence` | Function-head sync + ticket queues + escalation + vendor coordination | weekly | rhythm-of-business@1 | shipped (1.0.0) |
| `annual-vendor-consolidation` | Overlap analysis + contract rationalization + savings + transition | annual | vendor-scorecard@1 (consolidation chapter) | shipped (1.0.0) |
| `annual-ga-strategy` | G&A function priorities + automation + shared-services + cost targets | annual | strategy-document@1 | shipped (1.0.0) |

All workflows chain through shipped Tier-1/Tier-2 skills (`operating-model`, `rhythm-of-business`, `vendor-scorecard`, `strategy-doc`). See `../../skill/MODULE.md` §3.1 + §3.2.

---

## Cross-references
- `../../../modules/cuo/docs/module.md` §5.1 — source role profile.
- `../MODULE.md` §4 — canonical persona catalog.
- `../docs/AGENTS.md` — protocol normativity.
