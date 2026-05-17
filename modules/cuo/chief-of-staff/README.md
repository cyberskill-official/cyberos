# `chief-of-staff` — Chief of Staff

> Per `../../docs/The C-Suite Reference.md` §5.7 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief of Staff.
- **Persona slug:** `chief-of-staff`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Often first key hire (seed) · Series A onward: ESSENTIAL · Enterprise: Office-of-CEO (per §3 matrix).
- **One-sentence scope:** Not strictly C-level but operates at it. CEO leverage multiplier; rhythm-of-business owner; OKR/decision tracker; special-projects lead. Visible.vc + First Round consensus: **one of the most important early hires.**

## §2-§4  Inputs
See C-Suite Reference §5.7. Expand in next session.

## §5  Outputs
**Strategic:** OKR-cascade governance; rhythm-of-business calendar; cross-functional special-projects. **Operational:** decision log; meeting hygiene; CEO time mgmt. **Communication:** CEO-comms drafting; cross-functional briefings. **Team:** typically 0-2 reports.

## §6  Cadence
Per role profile in §5.7.

## §7  KPIs
| CEO time reclaimed | per quarter | calendar audit |
| On-time decision closure | > 90 % | decision log |
| OKR adoption % across functions | > 95 % | OKR tool |
| Cross-functional initiative delivery | per program plan | PMO |

## §8  Audit criteria
- **Quantitative:** moves KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** role-ambiguity vs CEO; becoming a gatekeeper; OKR-process-bureaucracy.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Linear / Asana / ClickUp (special projects); Lattice / Ally / Workboard (OKRs); Notion (decision log + briefing).

---

## Workflows

| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `quarterly-okr-cascade` | Govern the OKR cascade with decision-log trail | quarterly | okr-set@1 + decision-log@1 | shipped (1.0.0) |
| `weekly-rhythm-of-business` | Refresh the operating rhythm + CEO calendar | weekly | rhythm-of-business@1 | shipped (1.0.0) |
| `decision-log-keeping` | Capture a single decision as a structured log entry | on-demand | decision-log@1 | shipped (1.0.0) |
| `special-project-charter` | Charter a CEO-sponsored special project | per-event | program-charter@1 | shipped (1.0.0) |

All workflows chain through shipped skills — Tier-1 (`okr-set`, `decision-log`, `rhythm-of-business`) + Tier-2 (`program-charter`). See `../../skill/MODULE.md` §3.1 + §3.2.

---

## Cross-references
- `../../docs/The C-Suite Reference.md` §5.7.
- `../MODULE.md` §4.
