# `clo-legal` — Chief Legal Officer / General Counsel

> Per `../../docs/The C-Suite Reference.md` §5.2 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Legal Officer / General Counsel.
- **Persona slug:** `clo-legal`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: external · Series A: external · Scale-up: fractional · Growth onward: ESSENTIAL (per §3 matrix).
- **One-sentence scope:** Owns legal, regulatory, contracts, litigation, governance. **Fastest-growing NEO role 2021-2025.** Often absorbs Compliance in mid-market.

## §2  Information inputs
See C-Suite Reference §5.2 for full input list. Expand in next session.

## §3  Stakeholder inputs
CEO / board mandates; peer-C-suite asks; customer + regulator signals as applicable. See §5.2.

## §4  Resource inputs
Budget envelope, headcount, tooling spend authority per role profile. See §5.2.

## §5  Outputs
**Strategic:** legal strategy; M&A legal close-out; IP portfolio strategy. **Operational:** contract review + signing; litigation mgmt; regulatory submissions. **Communication:** board legal chapter; client legal liaison. **Team:** AGCs; paralegals; outside-counsel coordination.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.2.

## §7  KPIs
| Litigation exposure | trending down | legal mgmt |
| Contract cycle time | < target (e.g. 7 days for standard MSA) | CLM |
| Regulatory finding count | 0 | audit |
| M&A close quality | qualitative + close-rate | deal post-mortem |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** contract-cycle-time blowout; missed regulatory deadlines; IP filings lapsing; reactive-only mode (always firefighting, never proactive).
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Ironclad / Juro / DocuSign CLM (contracts); LexisNexis / Westlaw (research); Litify / iManage (matter mgmt); Spotdraft / Pavago (AI contract review).

---

## Workflows

| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `msa-contract-review` | Review an incoming MSA against the playbook | on-demand | contract-review@1 | shipped (1.0.0) |
| `incoming-nda-triage` | Triage an incoming NDA (GREEN/YELLOW/RED) | on-demand | non-disclosure-agreement-triage@1 | shipped (1.0.0) |
| `quarterly-regulatory-cycle` | Author the quarter's regulatory filings | quarterly | regulatory-filing@1 (multiple) | shipped (1.0.0) |
| `annual-ip-strategy` | Author the annual IP portfolio strategy | annual | intellectual-property-strategy@1 | shipped (1.0.0) |
| `quarterly-board-legal-chapter` | Author the legal chapter of the board deck | quarterly | litigation-management-update@1 + board-deck@1 chapter | shipped (1.0.0) |

All workflows chain through Tier-4 legal skills (`contract-review`, `nda-triage`, `regulatory-filing`, `ip-strategy`, `litigation-mgmt-update`) — shipped Session E 2026-05-17 — plus shipped Tier-1 (`board-deck`, `compliance-program`). See `../../skill/MODULE.md` §3.4.

---

## Cross-references
- `../../docs/The C-Suite Reference.md` §5.2 — source role profile.
- `../MODULE.md` §4 — canonical persona catalog.
- `../docs/AGENTS.md` — protocol normativity.
