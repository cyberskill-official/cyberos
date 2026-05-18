# `cco-communications` — Chief Communications Officer (Communications)

> Per `../../docs/The C-Suite Reference.md` §5.4 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Communications Officer (Communications).
- **Persona slug:** `cco-communications`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: — · Series A: — · Scale-up: — · Growth: common · Enterprise: ESSENTIAL (per §3 matrix).
- **One-sentence scope:** PR, IR, internal comms, crisis. Common at enterprise. **Acronym collision** with Commercial, Compliance, Customer.

## §2-§4  Inputs (information / stakeholder / resource)
See C-Suite Reference §5.4 for the full input lists. Expand in next session.

## §5  Outputs
**Strategic:** comms strategy; crisis-comms playbook; IR cadence. **Operational:** press releases; analyst briefings; internal newsletters; crisis comms execution. **Communication:** is the deliverable. **Team:** PR + IR + Internal Comms + Speechwriting.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.4.

## §7  KPIs
| Share of voice | trending up | media tracker |
| IR perception (sell-side feedback) | improving | analyst survey |
| Internal comms eNPS | > 30 | survey |
| Crisis-response time | per playbook SLA | crisis log |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** press-release-only mindset; IR confusion (mixed messages to analysts); crisis-comms paralysis.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Cision / Meltwater (PR + monitoring); Notified (IR portal); Staffbase / Firstup (internal); Q4 Inc (IR data).

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `annual-crisis-playbook` | Scenario inventory + holding statements + escalation + stakeholder maps | annual | crisis-communications-playbook@1 | shipped (1.0.0) |
| `per-press-release` | Distribution + wire + media-list + earned-media follow-through | per-event | press-release@1 (with distribution log) | shipped (1.0.0) |
| `monthly-internal-newsletter` | All-hands newsletter: wins / decisions / OKRs / people moves / asks | monthly | internal-newsletter@1 | shipped (1.0.0) |
| `per-crisis-response` | Invoke playbook + draft/distribute statements + stakeholder coordination | per-event | crisis-communications-playbook@1 (incident-augmented) | shipped (1.0.0) |

All workflows chain through shipped Tier-2 skills (`crisis-comms-playbook`, `press-release`, `internal-newsletter`). See `../../skill/MODULE.md` §3.2.

---

## Cross-references
- `../../docs/The C-Suite Reference.md` §5.4 — source role profile.
- `../MODULE.md` §4.
