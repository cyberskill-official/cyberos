# `cso-security` — Chief Security Officer (Security)

> Per `../../docs/The C-Suite Reference.md` §5.7 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Security Officer (Security).
- **Persona slug:** `cso-security`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Mostly enterprise; physical-sec heavy industries (per §3 matrix).
- **One-sentence scope:** Physical + info-sec super-set of CISO. **Acronym collision** with Strategy, Sustainability, Sales.

## §2-§4  Inputs
See C-Suite Reference §5.7. Expand in next session.

## §5  Outputs
**Strategic:** corporate-security strategy spanning physical + info-sec; executive-protection program. **Operational:** physical-security ops; cyber-physical convergence; insider-threat program. **Communication:** board security chapter. **Team:** Corporate Security + InfoSec (often dual-reporting with CISO).

## §6  Cadence
Per role profile in §5.7.

## §7  KPIs
| Physical-incident MTTR | per facility | security ops |
| Cyber-physical incident count | trending 0 | unified incident tracker |
| Executive-protection coverage | per protectee | EP roster |

## §8  Audit criteria
- **Quantitative:** moves KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** physical-vs-info-sec silo (the role exists to bridge them); insider-threat blind spots; EP-without-intel.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Genetec / Milestone (VMS); LenelS2 (access control); Cyware (CTI); FusionCenter.

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `annual-converged-security-strategy` | Physical + info-sec + supply-chain + insider-threat + exec protection | annual | security-strategy@1 (converged) | shipped (1.0.0) |
| `per-physical-security-charter` | Physical-security program: facility / exec-protection / supply-chain / insider | per-event | program-charter@1 | shipped (1.0.0) |
| `per-converged-incident-postmortem` | Converged postmortem: physical + info-sec + supply-chain root cause | per-event | postmortem@1 | shipped (1.0.0) |
| `quarterly-vuln-management` | Converged VM: physical-facility + info-sec + supply-chain + insider-threat | quarterly | vulnerability-management-report@1 (converged) | shipped (1.0.0) |

All workflows chain through shipped Tier-2/Tier-5 + SDP-original skills (`security-strategy`, `program-charter`, `postmortem`, `vulnerability-mgmt-report`). See `../../skill/MODULE.md` §3 + §3.2 + §3.5.

---

## Cross-references
- `../../docs/The C-Suite Reference.md` §5.7.
- `../MODULE.md` §4.
