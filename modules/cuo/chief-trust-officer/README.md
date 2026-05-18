# `chief-trust-officer` — Chief Trust Officer

> Per `../../docs/The C-Suite Reference.md` §5.6 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Trust Officer.
- **Persona slug:** `chief-trust-officer`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: — · Series A: — · Scale-up: emerging · Growth: emerging · Enterprise: emerging (SaaS-heavy) (per §3 matrix).
- **One-sentence scope:** Emerging meta-role spanning security + privacy + ethics + transparency, especially in SaaS where customer trust is the product.

## §2-§4  Inputs (information / stakeholder / resource)
See C-Suite Reference §5.6 for the full input lists. Expand in next session.

## §5  Outputs
**Strategic:** trust strategy; transparency-report framework; customer-trust portal. **Operational:** trust-portal updates; SOC 2 / ISO 27001 readiness; customer-trust deal-support. **Communication:** transparency reports; trust portal; customer-trust briefings. **Team:** small (typically 2-4 people coordinating across security + privacy + compliance + comms).

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.6.

## §7  KPIs
| Trust portal NPS | > 30 | survey |
| Transparency report cadence | quarterly | report log |
| Security-due-diligence deal-impact | acceleration trend | deal-cycle data |
| SOC 2 / ISO 27001 / cert currency | maintained | audit |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** trust-portal-as-marketing-only (without substance); role-overlap confusion with CISO + CPO-Privacy + CCO-Compliance.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Vanta / Drata / SecureFrame (audit auto); Trust portal (custom or Drata Trust); transparency-report tooling.

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `quarterly-trust-portal-update` | Refresh customer-facing trust portal: certs / privacy / subprocessors | quarterly | trust-portal-update@1 | shipped (1.0.0) |
| `annual-transparency-report` | Govt-data-requests + moderation + abuse metrics + model decisions | annual | transparency-report@1 | shipped (1.0.0) |
| `per-trust-incident-update` | Time-sensitive incident disclosure on public trust portal | per-event | trust-portal-update@1 (incident-augmented) | shipped (1.0.0) |
| `annual-trust-strategy` | Trust posture vision + cert roadmap + transparency program + customer-trust metrics | annual | strategy-document@1 | shipped (1.0.0) |

All workflows chain through shipped skills — Tier-2 (`trust-portal-update`, `transparency-report`, `strategy-doc`). See `../../skill/MODULE.md` §3.2.

---

## Cross-references
- `../../docs/The C-Suite Reference.md` §5.6.
- `../MODULE.md` §4.
