# `cfo` — Chief Financial Officer

> Per `../../../modules/cuo/docs/module.md` §5.2 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Financial Officer.
- **Persona slug:** `cfo`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: fractional · Series A: fractional/FT · Scale-up onward: ESSENTIAL (per §3 matrix).
- **One-sentence scope:** Owns FP&A, capital, controls, investor relations; modern CFOs co-own data + tech investment decisions (Deloitte 2025). Earns 37-39% of CEO comp.

## §2  Information inputs
See C-Suite Reference §5.2 for full input list. Expand in next session.

## §3  Stakeholder inputs
CEO / board mandates; peer-C-suite asks; customer + regulator signals as applicable. See §5.2.

## §4  Resource inputs
Budget envelope, headcount, tooling spend authority per role profile. See §5.2.

## §5  Outputs
**Strategic:** budget; forecast; capital plan; IR narrative. **Operational:** monthly close; board financials; cash mgmt. **Communication:** board financials; investor updates; bank/audit liaison. **Team:** Controller; FP&A; Treasury; AR/AP.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.2.

## §7  KPIs
| Forecast accuracy | ±5 % | finance |
| Cash runway | per stage | treasury |
| Gross / net margin | per business model | finance |
| Working capital | per industry | finance |
| Audit clean | yes | external audit |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** forecast drift >10 %; control weaknesses; slow close (>10 days); investor-communication blackouts; founder-mode resistance to financial discipline.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
NetSuite / SAP / Oracle (ERP); Adaptive / Anaplan / Pigment (FP&A); Stripe data (revenue); BI (Looker, Mode); Carta (cap table); Tipalti / Bill.com (AP).

---

## Workflows

| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `monthly-close` | Close the books for the month | monthly | monthly-close@1 | shipped (1.0.0) |
| `quarterly-forecast` | Build the rolling 4Q driver-based forecast | quarterly | forecast@1 | shipped (1.0.0) |
| `annual-budget` | Build the annual operating budget | annual | budget@1 | shipped (1.0.0) |
| `quarterly-board-financials` | Author the financial chapter of the board deck | quarterly | board-deck@1 chapter | shipped (1.0.0) |
| `monthly-cash-management` | Refresh the 13-week cash forecast (TWCF) | monthly | thirteen-week-cash-flow@1 | shipped (1.0.0) |
| `annual-strategic-plan` | Capital structure + M&A capacity + hurdle rates + investor narrative | annual | strategy-document@1 | shipped (1.0.0) |

All workflows chain through shipped skills — Tier-1 (`monthly-close`, `forecast`, `budget`, `board-deck`) + Tier-2 (`strategy-doc`) + Tier-3 (`13-week-cash-flow`). See `../../skill/MODULE.md` §3.1 + §3.2 + §3.3.

---

## Cross-references
- `../../../modules/cuo/docs/module.md` §5.2 — source role profile.
- `../MODULE.md` §4 — canonical persona catalog.
- `../docs/AGENTS.md` — protocol normativity.
