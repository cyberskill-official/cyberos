# `cpo-people` — Chief People Officer (People)

> Per `../../../modules/cuo/README.md` §5.5 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief People Officer (People).
- **Persona slug:** `cpo-people`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Same as CHRO (per §3 matrix).
- **One-sentence scope:** Synonym of CHRO at many firms. **Acronym collision** with Product, Privacy, Procurement.

## §2-§4  Inputs (information / stakeholder / resource)
See C-Suite Reference §5.5 for the full input lists. Expand in next session.

## §5  Outputs
Same as CHRO. Use this slug if the firm prefers 'CPO-People' nomenclature; otherwise use `chro`.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.5.

## §7  KPIs
Same as CHRO.

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** naming-confusion with CPO-Product.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
Same as CHRO.

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `synonym-pointer-readme` | Pointer: CPO-People is synonym of CHRO; use CHRO workflows for actual HR work | on-demand | decision-log@1 | shipped (1.0.0) |
| `annual-people-strategy` | Annual people strategy (synonym variant) | annual | strategy-document@1 | shipped (1.0.0) |
| `quarterly-people-review` | Consolidated quarterly people review (workforce + engagement + DEI) | quarterly | rhythm-of-business@1 (people chapter) | shipped (1.0.0) |
| `annual-employee-value-proposition` | EVP: culture + total rewards + career growth + mission alignment | annual | strategy-document@1 (EVP chapter) | shipped (1.0.0) |

**CPO-People is a synonym of CHRO** at firms that prefer "People" nomenclature. All workflows chain through shipped Tier-1/Tier-2 skills (`decision-log`, `strategy-doc`, `rhythm-of-business`). See `../chro/` for canonical implementation and `../../skill/MODULE.md` §3.1 + §3.2.

---

## Cross-references
- `../../../modules/cuo/README.md` §5.5 — source role profile.
- `../MODULE.md` §4.
