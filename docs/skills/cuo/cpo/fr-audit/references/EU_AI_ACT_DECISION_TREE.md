# EU AI Act decision tree (audit-side)

> Same tree as `cuo/cpo/fr-author/references/EU_AI_ACT_DECISION_TREE.md`. Both skills consult the identical tree; `fr-author` uses it to populate `tentative_eu_ai_act_risk_class`, `fr-audit` uses it to check QA-001 / QA-002 / QA-003 violations. Sourced from `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §8.

When the FR involves an AI system (broadly: ML inference, LLM generation, automated scoring, generative content), classify per:

1. **Article 5 — Prohibited practices.** Social scoring of natural persons, untargeted facial-image scraping, emotion inference in workplace/education, real-time public-space biometric ID for law enforcement, subliminal manipulation. → If matched in the FR body, QA-003 fires; mark `needs_human` with `hitl_reason: legal_compliance`.

2. **Annex III — High-risk domains.** Biometrics, education grading, hiring/recruitment scoring, credit scoring, emergency dispatch triage, law-enforcement profiling, migration/border decisions, critical-infrastructure control, judicial decisions. → If matched in the FR body AND `eu_ai_act_risk_class < high`, QA-002 fires; mark `needs_human` with `hitl_reason: ai_act_risk_boundary`.

3. **Article 50 — Transparency.** Chatbots, AI-generated text/audio/image/video, deepfakes, emotion recognition in non-Annex-III contexts. → Disclosure obligation surfaces as COND-004's required `## AI Authorship Disclosure` section when `ai_authorship != none`. Absence with `ai_authorship != none` → COND-004 error.

4. **None of the above** → `eu_ai_act_risk_class: minimal` is acceptable if AI is involved but the use is purely internal/non-disclosed; `not_ai` if no AI is involved.

## Audit-side specifics

The audit MUST NOT auto-promote `eu_ai_act_risk_class` (per `references/ANTI_FABRICATION.md`). When QA-001 / QA-002 / QA-003 fire, the issue is `needs_human` and the FR is not modified.

The audit's HITL question, in each case, is structured:

- **QA-001** (dodged risk class): "FR claims `eu_ai_act_risk_class: <minimal|not_ai>` but body contains AI-generation cues + the FR is user-facing or client-visible. Options: A) Promote class to `limited` (with COND-004 disclosure section). B) Confirm class as-is and explain in the FR body why the cues are not AI-generation. C) Mark FR `wontfix` (rare; requires CLO sign-off)."
- **QA-002** (high-risk indicator without `high`): "FR body mentions Annex III domain `<domain>` while class is `<current>`. Options: A) Promote class to `high` (with full COND-003 AI Risk Assessment). B) Clarify in FR body that the Annex III domain is mentioned only as context, not the FR's own behaviour. C) Defer to CLO via the `escalation.to_persona_on_legal` path."
- **QA-003** (Article 5 prohibited practice): "FR body describes `<prohibited practice>`. Article 5 prohibits this regardless of risk class. Options: A) Redesign — describe how the FR avoids the prohibited behaviour. B) Mark FR `wontfix`; the feature cannot ship as described. C) Defer to CLO via the `escalation.to_persona_on_legal` path."

## Citations

- EU AI Act in force August 2025; obligations phased to August 2026 → CyberOS-PRD §12.2.2; SRS DEC-064.
- CUO defer-to-human triggers — "Legal or compliance assertion" → PRD §6.4.1.
- CPO escalation graph routing this to `cuo-clo` → `cuo/cpo/SKILL.md` `escalation.to_persona_on_legal`.
