# `cao-accounting` — Chief Accounting Officer

> Per `../../../modules/cuo/docs/module.md` §5.2 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief Accounting Officer.
- **Persona slug:** `cao-accounting`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: — · Series A: — · Scale-up: optional · Growth: common · Enterprise: ESSENTIAL pre-IPO (per §3 matrix).
- **One-sentence scope:** Controller-level, owns GAAP/IFRS, audit, SOX. Reports to CFO. Quiet but essential pre-IPO.

## §2  Information inputs
See C-Suite Reference §5.2 for full input list. Expand in next session.

## §3  Stakeholder inputs
CEO / board mandates; peer-C-suite asks; customer + regulator signals as applicable. See §5.2.

## §4  Resource inputs
Budget envelope, headcount, tooling spend authority per role profile. See §5.2.

## §5  Outputs
**Strategic:** accounting policy framework; SOX readiness roadmap; IPO accounting prep. **Operational:** monthly close; audit liaison; SOX controls; tax provision. **Communication:** audit committee chapter (board). **Team:** Controller; Accounting Managers; Tax.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.2.

## §7  KPIs
| Close cycle time | < 10 business days | close calendar |
| Audit findings | 0 | external audit |
| SOX deficiency count | 0 | internal audit |
| Account reconciliation completion | 100 % monthly | close checklist |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** missed SOX deadlines; account-reconciliation backlog; revenue-recognition errors; IPO surprise on day -90.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
NetSuite / Sage Intacct / Oracle (GL); BlackLine (close + recon); Workiva (SOX); Floqast (close); Avalara (tax).

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `monthly-close-execution` | Controller execution: accruals + recs + technical-accounting + sign-offs | monthly | monthly-close@1 (controller execution log) | shipped (1.0.0) |
| `quarterly-audit-readiness` | Control testing + evidence + walkthroughs + position papers | quarterly | compliance-program@1 (audit-readiness chapter) | shipped (1.0.0) |
| `annual-accounting-policy` | Policy manual: rev rec / leases / equity / segment / standard adoptions | annual | strategy-document@1 (accounting-policy chapter) | shipped (1.0.0) |
| `annual-budget-controllership` | Controllership support for budget cycle: actuals + classification | annual | budget@1 (controllership chapter) | shipped (1.0.0) |

All workflows chain through shipped Tier-1 skills (`monthly-close`, `compliance-program`, `strategy-doc`, `budget`). See `../../skill/MODULE.md` §3.1 + §3.2.

---

## Cross-references
- `../../../modules/cuo/docs/module.md` §5.2 — source role profile.
- `../MODULE.md` §4 — canonical persona catalog.
- `../docs/AGENTS.md` — protocol normativity.
