# `ceo` — Chief Executive Officer

> Per `../../docs/The C-Suite Reference.md` §5.1 + §4 (9-block schema).

## §1  Identity & scope

- **Full disambiguated title:** Chief Executive Officer (CEO).
- **Persona slug:** `ceo`
- **Acronym collision resolution:** none — CEO is unambiguous.
- **Reports to:** Board of Directors.
- **Reports in:** entire C-suite + Chief of Staff.
- **Stage prevalence:** ESSENTIAL at every stage (per §3 matrix). Median tenure dropped to record-low 6.8 yrs in 2025.
- **One-sentence scope:** Highest authority; sets vision; allocates capital; owns the board relationship and the external narrative.

## §2  Information inputs

- **Dashboards:** company OKR tracker, exec dashboard, financial summary, market intel feed.
- **Reports:** board mandates, market intel, financials, exec team status, customer signals (esp. enterprise + at-risk accounts).
- **Market intel:** sector analyst reports, competitor disclosures, macro trackers.

## §3  Stakeholder inputs

- **Board:** strategic direction, governance asks, M&A theses, capital-allocation envelope.
- **Investors:** capital availability, milestone expectations.
- **Top customers:** churn signals + expansion asks (filtered by CCO-Customer / CRO).
- **C-suite peers:** weekly + monthly readouts; escalations.

## §4  Resource inputs

- **Budget envelope:** total company P&L authority.
- **Headcount:** every hire ≥VP level usually requires CEO sign-off.
- **Tooling spend authority:** unlimited (subject to board governance).

## §5  Outputs

### 5.1 Strategic
- Company strategy + 3-5yr vision; OKRs cascade; capital allocation; M&A theses; major-hire decisions.

### 5.2 Operational
- Cross-functional escalation resolution; budget reallocation; crisis-mode interim leadership.

### 5.3 Communication
- Board decks (quarterly); investor updates (quarterly); all-hands (monthly); external narrative (press, conferences, customer execs).

### 5.4 Team
- C-suite hires + fires; succession planning; culture stewardship at founding-team scale.

## §6  Cadence

| Cadence | Activity |
|---|---|
| Daily | exec one-on-ones (rotating); critical-customer touchpoints |
| Weekly | exec staff meeting; investor / board chair check-ins |
| Monthly | board prep; all-hands; deep-dive review with one function |
| Quarterly | board meeting; OKR roll; QBR; investor update |
| Annual | strategy planning; budget approval; comp cycle for C-suite |

## §7  KPIs

| KPI | Target | Source |
|---|---|---|
| Revenue growth | per stage benchmark (e.g. T2D3 for SaaS) | finance |
| EBITDA / cash runway | per business model | finance |
| Market cap / valuation | board-set | finance / IR |
| Employee NPS (eNPS) | ≥30 | CHRO survey |
| Board confidence (qualitative) | high | board self-assessment |

## §8  Audit criteria

### 8.1 Quantitative
- Do CEO outputs move the 5 KPIs above? Forecast within ±10 %?

### 8.2 Qualitative rubric
- Alignment with board mandate / coherence across functions / customer-grounding / risk-adjusted realism / communicability.

### 8.3 Failure modes
- **Universal:** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific:** missed quarters with no narrative; exec churn; capital misallocation; founder-mode lock-in past 100 employees.

### 8.4 Commercial baseline reminders (per §8 of source)
- Don't add a C-title to solve a process problem; match title to stage; disambiguate every acronym; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack

- Board portals (Diligent, Boardable, Nasdaq Boardvantage); OKR systems (Lattice, Ally, Workboard); exec dashboards (Looker, Mode, Power BI); investor-update CRMs (Visible, Carta).

---

## Workflows

| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `quarterly-board-update` | Author the quarterly board deck | quarterly | board-deck@1 | shipped (1.0.0) |
| `okr-cascade` | Drive the company-OKR cascade from vision | quarterly | okr-set@1 | shipped (1.0.0) |
| `monthly-investor-update` | Author the investor monthly update | monthly | investor-update@1 | shipped (1.0.0) |
| `c-suite-hire-decision` | Drive a C-suite hire decision (interview loop + offer) | per-event | hire-decision-record@1 | shipped (1.0.0) |
| `capital-allocation-memo` | Author a capital-allocation memo for the board | per-event | cap-alloc-memo@1 | shipped (1.0.0) |

All workflows chain through shipped Tier-1 skills (board-deck-author/audit, okr-set-author/audit, investor-update-author/audit, hire-decision-author/audit, cap-alloc-memo-author/audit) — see `../../skill/MODULE.md` §3.1.

---

## Cross-references

- `../../docs/The C-Suite Reference.md` §5.1, §3, §6 — source role profile + stage matrix + audit framework.
- `../MODULE.md` — canonical persona catalog.
- `../cto/README.md` — peer persona; closest collaboration partner for technical strategy.
- `../chief-of-staff/README.md` — operating-rhythm partner; OKR/decision tracker.
