# `cio-information` — Chief Information Officer (Information)

> Per `../../../modules/cuo/README.md` §5.3 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Information Officer (Information).
- **Persona slug:** `cio-information`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: — · Series A: — · Scale-up: if complex IT · Growth: common · Enterprise: ESSENTIAL (per §3 matrix).
- **One-sentence scope:** Inward-facing: enterprise IT, internal tooling, data infra, cybersecurity oversight. Often co-owns AI investments with CTO/CFO (Deloitte).

## §2-§4  Inputs (information / stakeholder / resource)
See C-Suite Reference §5.3 for the full input lists. Expand in next session.

## §5  Outputs
**Strategic:** IT strategy + roadmap; tool consolidation; AI internal-deployment portfolio. **Operational:** enterprise system uptime; internal NPS; incident MTTR. **Communication:** board IT chapter; all-hands tool announcements. **Team:** Heads of Enterprise Apps / Infra / IT Security / Service Desk.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.3.

## §7  KPIs
| System uptime | > 99.9 % | monitoring |
| IT spend % revenue | per industry benchmark | finance |
| Internal NPS | ≥ 30 | quarterly survey |
| Incident MTTR | < 4 h | ITSM |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** shadow-IT proliferation; tool bloat; over-rotating on cost vs internal satisfaction; AI deployments without governance.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
ServiceNow / Jira Service Mgmt (ITSM); Okta / Azure AD (IAM); Datadog / Splunk (observability); Tanium / Crowdstrike (endpoint); Microsoft 365 / Google Workspace.

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `annual-it-strategy` | Service catalog + infra roadmap + vendor stack + cost optimization | annual | strategy-document@1 | shipped (1.0.0) |
| `quarterly-it-operating-review` | Uptime + SLA + MTTR + change-success + spend trends | quarterly | operating-model@1 (IT quarterly chapter) | shipped (1.0.0) |
| `quarterly-it-vendor-scorecard` | IT vendor scoring: SLA + security + sustainability + renewal | quarterly | vendor-scorecard@1 | shipped (1.0.0) |
| `annual-it-security-strategy` | IT-lens input to security strategy: endpoint + IAM + network + BCP/DR | annual | security-strategy@1 (IT chapter) | shipped (1.0.0) |

All workflows chain through shipped Tier-1/Tier-2/Tier-5 skills (`strategy-doc`, `operating-model`, `vendor-scorecard`, `security-strategy`). See `../../skill/MODULE.md` §3.1 + §3.2 + §3.5.

---

## Cross-references
- `../../../modules/cuo/README.md` §5.3 — source role profile.
- `../MODULE.md` §4.
