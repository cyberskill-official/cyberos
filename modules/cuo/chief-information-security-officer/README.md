# `ciso` — Chief Information Security Officer

> Per `../../docs/The C-Suite Reference.md` §5.3 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Information Security Officer.
- **Persona slug:** `ciso`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: outsourced · Series A: outsourced · Scale-up: often outsourced · Growth: ESSENTIAL (or vCISO) · Enterprise: ESSENTIAL (per §3 matrix).
- **One-sentence scope:** Cyber risk owner. Often reports to CIO / CEO / CRO-Risk. Vietnam context: increasingly required under Decree 13/2023 PDPD.

## §2-§4  Inputs (information / stakeholder / resource)
See C-Suite Reference §5.3 for the full input lists. Expand in next session.

## §5  Outputs
**Strategic:** security strategy + ISMS; security architecture review board; vCISO advisory if outsourced. **Operational:** vuln mgmt; incident response; security audit; pen-test program. **Communication:** board security chapter; customer-trust portal contributions; regulator submissions. **Team:** SecOps; AppSec; GRC; Identity.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.3.

## §7  KPIs
| MTTD (detect) | < 1 h | SIEM |
| MTTR (respond) | < 4 h sev1 | SIEM + IR |
| Vuln closure SLA | per severity | vuln-mgmt tool |
| Audit findings closed | trending 0 | audit tracker |
| Phishing-test pass rate | > 95 % | security awareness platform |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** missed regulator deadlines (Vietnam Decree 13/2023, GDPR Art. 33); CVE backlog growth; pen-test findings unresolved; security-team burnout.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Crowdstrike / SentinelOne (EDR); Splunk / Sentinel (SIEM); Snyk / GitGuardian (DevSecOps); Wiz / Lacework (cloud security); KnowBe4 (awareness); ServiceNow GRC.

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `annual-security-strategy` | Threat landscape + NIST CSF posture + initiatives + OKRs | annual | security-strategy@1 | shipped (1.0.0) |
| `monthly-vuln-management` | Open vulns + SLA + exceptions + remediation roadmap | monthly | vulnerability-mgmt-report@1 | shipped (1.0.0) |
| `annual-pen-test-cycle` | Scope + engagement + findings + retest | annual | pen-test-report@1 | shipped (1.0.0) |
| `soc2-audit-readiness` | TSC evidence + gap analysis + remediation | annual | soc2-evidence@1 | shipped (1.0.0) |

All workflows chain through shipped Tier-5 skills (`security-strategy`, `vulnerability-mgmt-report`, `pen-test-report`, `soc2-evidence`). See `../../skill/MODULE.md` §3.5.

---

## Cross-references
- `../../docs/The C-Suite Reference.md` §5.3 — source role profile.
- `../MODULE.md` §4.
