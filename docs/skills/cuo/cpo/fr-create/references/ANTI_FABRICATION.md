# Anti-fabrication rules (both fr-create and fr-audit)

> Sourced verbatim from `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §9. The audit rubric in `cuo/cpo/fr-audit/RUBRIC.md` enforces these via `QA-007` (unsourced numeric target) and `QA-008` (cross-team dependency claim).

The skill MUST NEVER:

- Invent customer quotes, attributions, or dates.
- Invent metric baselines or numeric targets — use `TBD-HUMAN: <reason>` and surface via QA-007.
- Invent named entities (people, vendors, regulations) not present in the requirements or in the FR being audited.
- Assert an external team has agreed to a dependency without a referenced ticket / owner / commitment.
- Change `eu_ai_act_risk_class` or `ai_authorship` autonomously.
- Set `ai_authorship: none` on output the skill itself produced.

The skill SHOULD prefer `TBD-HUMAN` over a plausible-but-unsourced placeholder.

## Why this matters in CyberOS

Per PRD §6.7's "Hallucination defence" rule:

> RAG forces source citation; if BRAIN has no relevant chunks for a question, CUO says "I don't know — your wish requires more context" rather than inventing.

The anti-fabrication discipline is the FR-domain manifestation of that global rule. A fabricated customer quote in an FR is exactly the same class of failure as CUO inventing a fact in a strategy memo.

Audit rule QA-007 escalates an unsourced target to `needs_human` (category: `success_metric_targets`). The Question primitive surfaces:

- The metric line as the skill saw it.
- The 2–3 most-relevant BRAIN search results (or "no candidates").
- Options:  A) Provide the source.  B) Mark TBD-HUMAN until source found. C) Drop this metric.

The same Question primitive applies to QA-008 (cross-team dependency without an owner): A) Provide ticket/owner.  B) Soften the dependency language (e.g. "if available").  C) Remove the dependency.
