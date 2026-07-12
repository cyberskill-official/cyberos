# `cmo` — Chief Marketing Officer

> Per `../../../modules/cuo/docs/module.md` §5.4 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Marketing Officer.
- **Persona slug:** `cmo`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: — · Series A: Head of Marketing · Scale-up: common · Growth: ESSENTIAL or absorbed by CRO · Enterprise: often reports to CRO (per §3 matrix).
- **One-sentence scope:** Pressure: many CMO roles absorbed into CRO or Chief Customer Officer; the 'death of the CMO' narrative is overstated but role is narrowing toward brand+demand.

## §2-§4  Inputs (information / stakeholder / resource)
See C-Suite Reference §5.4 for the full input lists. Expand in next session.

## §5  Outputs
**Strategic:** brand strategy; demand-gen plan; campaign portfolio. **Operational:** campaign launches; MQL→SQL handoff. **Communication:** brand book; PR coordination; analyst-relations. **Team:** Brand + Demand-Gen + Content + PMM.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.4.

## §7  KPIs
| Brand awareness | per market research | survey |
| MQL→SQL conversion | > 15 % | CRM |
| CAC by channel | per channel benchmark | finance |
| Share of voice | trending up | media tracker |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** vanity-metric trap (followers without pipeline); brand vs demand silo; over-rotating on one channel; PR-only orientation.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
HubSpot / Marketo / Pardot (automation); Webflow / WordPress (web); Sprout Social (social); Mention / Brand24 (monitoring); 6sense (intent).

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `quarterly-brand-strategy` | Positioning + narrative + messaging architecture | quarterly | brand-strategy@1 | shipped (1.0.0) |
| `per-campaign-plan` | Per-launch / theme / program campaign with channel + creative + measurement | per-event | campaign-plan@1 | shipped (1.0.0) |
| `quarterly-analyst-briefing` | Gartner / Forrester / IDC / IDG AR narrative + supporting evidence | quarterly | analyst-briefing@1 | shipped (1.0.0) |
| `per-press-release` | Per-announcement press release (content side) | per-event | press-release@1 | shipped (1.0.0) |

All workflows chain through shipped Tier-2 skills (`brand-strategy`, `campaign-plan`, `analyst-briefing`, `press-release`). See `../../skill/MODULE.md` §3.2.

---

## Cross-references
- `../../../modules/cuo/docs/module.md` §5.4 — source role profile.
- `../MODULE.md` §4.
