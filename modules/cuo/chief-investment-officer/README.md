# `cio-investment` — Chief Investment Officer (Investment)

> Per `../../../modules/cuo/docs/module.md` §5.7 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Investment Officer (Investment).
- **Persona slug:** `cio-investment`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Asset-management firms only (per §3 matrix).
- **One-sentence scope:** Asset-management firms; portfolio strategy. **Acronym collision** with Information.

## §2-§4  Inputs
See C-Suite Reference §5.7. Expand in next session.

## §5  Outputs
**Strategic:** investment thesis + portfolio strategy. **Operational:** position sizing; risk-budget allocation. **Communication:** LP letters; investment committee. **Team:** PMs + Analysts + Quant Research.

## §6  Cadence
Per role profile in §5.7.

## §7  KPIs
| Portfolio IRR / TWR | per benchmark | portfolio system |
| Sharpe ratio | per benchmark | portfolio system |
| Drawdown | bounded per risk-budget | portfolio system |
| LP NPS | > 30 | survey |

## §8  Audit criteria
- **Quantitative:** moves KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** style-drift; over-leverage; LP-comms-gaps in drawdowns; benchmark-hugging.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Bloomberg / Refinitiv (data); Aladdin / Charles River (PMS); Tableau / custom (LP reporting).

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `per-investment-thesis` | Per-position: hypothesis + valuation + catalysts + risks + sizing | per-event | investment-thesis@1 | shipped (1.0.0) |
| `quarterly-lp-letter` | LP letter: performance + positioning + market commentary | quarterly | limited-partner-letter@1 | shipped (1.0.0) |
| `annual-investment-strategy` | Asset allocation + sector tilts + risk budget + IPS alignment | annual | strategy-document@1 | shipped (1.0.0) |
| `quarterly-portfolio-review` | Attribution + risk exposures + thesis re-validation + rebalance | quarterly | strategy-document@1 (quarterly chapter) | shipped (1.0.0) |

All workflows chain through shipped Tier-2/Tier-3 skills (`investment-thesis`, `lp-letter`, `strategy-doc`). See `../../skill/MODULE.md` §3.2 + §3.3.

---

## Cross-references
- `../../../modules/cuo/docs/module.md` §5.7.
- `../MODULE.md` §4.
