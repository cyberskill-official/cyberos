# `chief-medical-officer` — Chief Medical Officer

> Per `../../../modules/cuo/README.md` §5.7 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Medical Officer.
- **Persona slug:** `chief-medical-officer`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Healthtech / insurtech firms only (per §3 matrix).
- **One-sentence scope:** Healthtech / insurtech only.

## §2-§4  Inputs
See C-Suite Reference §5.7. Expand in next session.

## §5  Outputs
**Strategic:** clinical strategy; medical-affairs framework; clinical-evidence policy. **Operational:** clinical-trial oversight; medical-affairs liaison; safety reporting. **Communication:** clinical-affairs to regulators; medical advisory board. **Team:** Medical Affairs + Clinical Operations + Safety.

## §6  Cadence
Per role profile in §5.7.

## §7  KPIs
| Clinical-trial completion on schedule | per protocol | trial mgmt |
| Adverse-event reporting timeliness | per regulator | safety system |
| Medical-advisory-board engagement | quarterly | board mgmt |

## §8  Audit criteria
- **Quantitative:** moves KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** regulator-deadline misses (FDA, EMA, Vietnam MoH); safety-reporting gaps; clinical-evidence weak.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Veeva Vault (clinical); Medidata Rave (EDC); regulator portals.

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `per-clinical-protocol` | Per-study clinical trial protocol per ICH-GCP E6(R3) | per-event | clinical-protocol@1 | shipped (1.0.0) |
| `quarterly-safety-report` | Adverse-event review + signal detection (PSUR/DSUR) | quarterly | safety-report@1 | shipped (1.0.0) |
| `annual-medical-strategy` | Therapeutic-area focus + evidence-gen + RWE + KOL engagement | annual | strategy-document@1 | shipped (1.0.0) |
| `per-medical-affairs-charter` | MSL / KOL / evidence / medical-ed program charter | per-event | program-charter@1 | shipped (1.0.0) |

All workflows chain through shipped Tier-2/Tier-3 skills (`clinical-protocol`, `safety-report`, `strategy-doc`, `program-charter`). See `../../skill/MODULE.md` §3.2 + §3.3.

---

## Cross-references
- `../../../modules/cuo/README.md` §5.7.
- `../MODULE.md` §4.
