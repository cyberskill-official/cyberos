# `cpo-privacy` — Chief Privacy Officer (Privacy)

> Per `../../docs/The C-Suite Reference.md` §5.6 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Privacy Officer (Privacy).
- **Persona slug:** `cpo-privacy`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: — · Series A: external · Scale-up: part-time legal counsel · Growth onward: ESSENTIAL (CyberSkill-specific: required under Vietnam Decree 13/2023) (per §3 matrix).
- **One-sentence scope:** GDPR / CCPA / Vietnam Decree 13 owner. **Acronym collision** with People, Product, Procurement.

## §2-§4  Inputs (information / stakeholder / resource)
See C-Suite Reference §5.6 for the full input lists. Expand in next session.

## §5  Outputs
**Strategic:** privacy program; PIA framework; cross-border-transfer policy. **Operational:** PIA reviews; DSR fulfilment; breach notification; sub-processor mgmt. **Communication:** privacy notice updates; DPO regulator liaison. **Team:** Privacy Engineers + DPO Coordinators.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.6.

## §7  KPIs
| PIA coverage | 100 % new features | PIA tracker |
| Breach notification timeliness | within GDPR Art. 33 72h | incident tracker |
| DSR response time | < statutory window | DSR tracker |
| Privacy training completion | > 95 % | LMS |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** missed Art. 33 timelines; PIA-skip on AI features; sub-processor list stale; consent-flow dark-pattern accusations.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
OneTrust / TrustArc / DataGrail (privacy mgmt); Privado / Securiti (privacy engineering); LMS for training.

---

## Workflows

| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `data-subject-request-cycle` | Handle a single DSR within statutory window (GDPR/PDPD 30d / CCPA 45d) | on-demand | dsr-runbook@1 | shipped (1.0.0) |
| `privacy-impact-assessment` | Per-feature PIA (escalates to DPIA per GDPR Art. 35) | per-event | pia@1 | shipped (1.0.0) |
| `breach-response-cycle` | 72-hour GDPR Art. 33 / PDPD breach notification cycle | per-event | breach-notification@1 | shipped (1.0.0) |
| `annual-privacy-program` | ROPA + DPIA inventory + DSR metrics + breach lookback + regulator status | annual | compliance-program@1 | shipped (1.0.0) |

All workflows chain through shipped Tier-1 skills (`dsr-runbook`, `pia`, `breach-notification`, `compliance-program`) — zero new skills required. See `../../skill/MODULE.md` §3.1.

---

## Cross-references
- `../../docs/The C-Suite Reference.md` §5.6.
- `../MODULE.md` §4.
