# EU AI Act decision tree

> Sourced verbatim from `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §8. Used by both `fr-create` (during PLAN to populate `tentative_eu_ai_act_risk_class`) and by `fr-audit` (during the rubric run to check QA-001 / QA-002 / QA-003 violations).

When the FR involves an AI system (broadly: ML inference, LLM generation, automated scoring, generative content), classify per:

1. **Article 5 — Prohibited practices.** Social scoring of natural persons, untargeted facial-image scraping, emotion inference in workplace/education, real-time public-space biometric ID for law enforcement, subliminal manipulation. → If matched, the FR cannot ship as-described regardless of risk class. Mark `needs_human` with category `legal_compliance` and rule_id `QA-003`.

2. **Annex III — High-risk domains.** Biometrics, education grading, hiring/recruitment scoring, credit scoring, emergency dispatch triage, law-enforcement profiling, migration/border decisions, critical- infrastructure control, judicial decisions. → If matched, default to `eu_ai_act_risk_class: high` unless Article 6(3) preparatory carve-out clearly applies. Always escalate (`needs_human`, category `ai_act_risk_boundary`, rule_id `QA-002`) — never auto-classify.

3. **Article 50 — Transparency.** Chatbots, AI-generated text/audio/image/ video, deepfakes, emotion recognition in non-Annex-III contexts. → `eu_ai_act_risk_class: limited` is the floor. Disclosure obligation surfaces as a required `## AI Authorship Disclosure` section.

4. **None of the above** → `eu_ai_act_risk_class: minimal` if AI is involved but the use is purely internal/non-disclosed; `not_ai` if no AI is involved.

When in doubt, escalate (return `escalate` in PLAN; surface as `needs_human` in WORKER). The cost of a false `pass` exceeds the cost of an extra HITL round. The skill MUST NEVER auto-classify `eu_ai_act_risk_class` to `minimal` when a determining fact is missing.

## Cross-persona escalation

Any `needs_human` raised by this decision tree triggers the persona-card's `escalation.to_persona_on_legal` (= `cuo-clo`). The CLO Question is appended to the HITL_BATCH_REQUEST with category `legal_compliance` and includes a CC of the original requestor on the audit row.

## Citations

- EU AI Act in force August 2025; obligations phased to August 2026 → CyberOS-PRD §12.2.2; SRS DEC-064 (high-risk classification posture).
- CUO defer-to-human triggers — "Legal or compliance assertion" → PRD §6.4.1.
