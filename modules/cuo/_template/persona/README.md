# `<persona-slug>` — Chief <Disambiguated Title>

> Per `../../../../modules/cuo/docs/module.md` §4 (the persona catalog / role profile); 9-block schema per `appendices.md`. Disambiguation rule per module.md §2.

## §1  Identity & scope

- **Full disambiguated title:** Chief <Disambiguated Title> (e.g. "Chief Revenue Officer" — not just "CRO").
- **Persona slug:** `<persona-slug>`
- **Acronym collision resolution:** <which acronym + which of the meanings — cite §2 row>.
- **Reports to:** <CEO / Board / other C-role>
- **Reports in:** <list of direct reports>
- **Stage prevalence:** <Seed: — / Series A: — / Scale-up: ESSENTIAL / Growth: ESSENTIAL / Enterprise: ESSENTIAL> (per §3 matrix)
- **One-sentence scope:** <"Owns X for the firm; bridges Y and Z; accountable for outcome A.">

## §2  Information inputs

What this persona consumes to make decisions.

- **Dashboards:** <named dashboards / BI tools>
- **Reports:** <FP&A, board reports, dept reports, etc.>
- **Market intel:** <named sources — analyst reports, Gartner, McKinsey, sector trackers>
- **Customer signals:** <NPS, support tickets, win/loss, churn, etc.>
- **Internal signals:** <eNPS, attrition, velocity, defect rate, etc.>

## §3  Stakeholder inputs

Who this persona listens to.

- **CEO / Board mandates:** <which directives, what cadence>
- **Peer C-suite asks:** <which peers, what asks>
- **Regulator signals:** <if applicable — e.g. SEC, EU AI Act regulator, Vietnam MIC, etc.>
- **External advisor inputs:** <board observers, fractional execs, audit firms>

## §4  Resource inputs

What this persona controls or influences.

- **Budget envelope:** <rough range as % of revenue or absolute>
- **Headcount:** <direct + indirect>
- **Tooling spend authority:** <decision rights>

## §5  Outputs

What this persona produces.

### 5.1 Strategic
- <Roadmap / plan / vision document — typically multi-quarter horizon>

### 5.2 Operational
- <Decisions, approvals, escalations — typically weekly/monthly>

### 5.3 Communication
- <Board decks, all-hands, customer-facing comms — typically per-cadence>

### 5.4 Team
- <Hires, promotions, culture-shaping, succession plans>

## §6  Cadence

| Cadence | Activity |
|---|---|
| Daily | <e.g. ops review, anomaly triage> |
| Weekly | <e.g. staff meeting, exec sync> |
| Monthly | <e.g. metrics review, forecast update, board prep> |
| Quarterly | <e.g. QBR, OKR rollout, board meeting> |
| Annual | <e.g. strategy planning, budget, comp cycle> |

## §7  KPIs

3-5 quantitative metrics this persona owns, each with target range.

| KPI | Target | Measurement source |
|---|---|---|
| <KPI 1> | <target> | <dashboard or report> |
| <KPI 2> | <target> | <…> |
| <KPI 3> | <target> | <…> |
| <KPI 4> | <target> | <…> |
| <KPI 5> | <target> | <…> |

## §8  Audit criteria

### 8.1 Quantitative gates (per C-Suite Reference §6)
- Does the output move this persona's 3 primary KPIs?
- Is forecast/plan within ±10% of actuals?
- Are leading indicators (not just lagging) being tracked?

### 8.2 Qualitative rubric (1–5 per dimension)
- **Alignment:** with CEO / board mandate.
- **Coherence:** with peer C-suite plans.
- **Customer/market grounding:** does it reflect external reality?
- **Risk-adjusted realism:** are downside scenarios planned for?
- **Communicability:** can a non-expert understand it?

### 8.3 Failure modes
- **Universal (from §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific:** <role-specific traps — cite §5 profile>

### 8.4 Commercial baseline reminders (per C-Suite Reference §8)
- Don't add a C-title to solve a process problem.
- Match the title to the funding stage.
- Disambiguate every acronym in writing.
- Beware hype-cycle titles.
- Audit outputs, not titles.

## §9  Tools & stack

- **Productivity & rhythm:** <named tools — e.g. Notion, Linear, Slack>
- **Decision support / dashboards:** <named — e.g. Looker, Mode, Tableau, Grafana>
- **Domain-specific:** <named — e.g. for CFO: NetSuite, Adaptive; for CTO: GitHub, PagerDuty>
- **AI assistance (per SDP §5 AI-use policy):** <named tools, data-perimeter rules, AI-assisted PR labelling commitment>

---

## Workflows

This persona ships the following workflows under `./workflows/`. Each workflow chains skills from the SKILL module to produce a specific output. See `cuo/_template/HOW_TO_USE.md` step-by-step for authoring new workflows.

| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `<workflow-1>` | <one-line> | <cadence> | <output artefact> | shipped \| planned |
| `<workflow-2>` | <…> | <…> | <…> | … |

---

## Cross-references

- `../../../../modules/cuo/docs/module.md` §4 — the source role profile (persona catalog).
- `../../../../modules/cuo/docs/module.md` §3 — the stage matrix that determined "stage prevalence" above.
- `../../../../modules/cuo/docs/module.md` §6 — the audit framework referenced in §8.
- `../MODULE.md` — the canonical persona catalog.
- `../docs/AGENTS.md` — protocol normativity.
- `../docs/ROUTING.md` — how the CUO routes to this persona.
