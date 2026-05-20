# `caio` — Chief AI Officer

> Per `../../../modules/cuo/README.md` §5.3 + §4 (9-block schema).

## §1  Identity & scope
- **Full disambiguated title:** Chief AI Officer.
- **Persona slug:** `caio`
- **Reports to:** CEO (typically).
- **Stage prevalence:** Seed: — · Series A: — · Scale-up: optional · Growth: increasingly common · Enterprise: ESSENTIAL (transitional) (per §3 matrix).
- **One-sentence scope:** 76% prevalence in 2026 (IBM CEO Study). Owns AI strategy, governance, scaling. **Transitional role — likely absorbed by CTO/CDO/CIO within 3 years.**

## §2-§4  Inputs (information / stakeholder / resource)
See C-Suite Reference §5.3 for the full input lists. Expand in next session.

## §5  Outputs
**Strategic:** AI strategy + use-case portfolio; AI risk inventory; AI governance framework. **Operational:** AI use-case prioritisation; model card library; bias-test pass-rate tracking. **Communication:** board AI chapter; AI ethics committee. **Team:** AI Engineers; ML Researchers; AI Product; AI Ethics.

## §6  Cadence
Daily / weekly / monthly / quarterly / annual rhythms per role profile. See §5.3.

## §7  KPIs
| AI use cases in production | per quarter | AI portfolio tracker |
| % workforce regularly using AI | ≥ 25 % (industry baseline) | survey |
| AI-driven cost reduction | per business plan | finance attribution |
| AI risk incidents | 0 | incident tracker |

## §8  Audit criteria
- **Quantitative:** moves the persona's 3-5 KPIs above; forecast within ±10 %.
- **Qualitative rubric:** alignment / coherence / customer-grounding / risk-realism / communicability.
- **Universal failure modes (per §6):** playbook transplant; activity over outcomes; silo-ing; forecast drift without narrative; hero dependence; AI-washing.
- **Role-specific failure modes:** AI-washing (claims without measurable production use); EU AI Act blind spots (high-risk classification missed); model card debt; governance-vs-velocity tension unresolved.
- **Commercial baseline reminders (per §8):** don't add a C-title for a process problem; match title to stage; disambiguate acronyms; beware hype-cycle titles; audit outputs not titles.

## §9  Tools & stack
OpenAI / Anthropic / Cohere (LLM); LangChain / LlamaIndex (orchestration); MLflow / W&B (MLOps); Aporia / Fiddler (model monitoring); model-card registries (Hugging Face).

---

## Workflows
| Workflow | Purpose | Cadence | Output | Status |
|---|---|---|---|---|
| `annual-ai-strategy` | Portfolio + build/buy/partner + governance + MLOps + EU AI Act risk classification | annual | ai-strategy@1 | shipped (1.0.0) |
| `quarterly-use-case-portfolio-review` | Pipeline + value × feasibility × risk + sunset | quarterly | ai-use-case-portfolio@1 | shipped (1.0.0) |
| `per-model-card-release` | Per-model documentation: use / data / perf / limits / ethics | per-event | model-card@1 | shipped (1.0.0) |
| `per-model-bias-audit` | Disparate-impact + 4/5ths analysis + mitigation | per-event | bias-audit@1 | shipped (1.0.0) |

All workflows chain through shipped skills — Tier-1 (`ai-use-case-portfolio`, `model-card`, `bias-audit`) + Tier-7 (`ai-strategy`). See `../../skill/MODULE.md` §3.1 + §3.7.

---

## Cross-references
- `../../../modules/cuo/README.md` §5.3 — source role profile.
- `../MODULE.md` §4.
