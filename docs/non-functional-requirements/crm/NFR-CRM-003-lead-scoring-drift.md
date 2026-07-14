---
id: NFR-CRM-003
title: "CRM lead-scoring drift — score model MUST be re-evaluated quarterly against actual outcomes"
module: CRM
category: reliability
priority: SHOULD
verification: T
phase: P1
slo: "Quarterly: score AUC vs actual won/lost outcomes ≥ 0.70; score drift trigger ≤ 5pp"
owner: CSO-Sales
created: 2026-05-18
related_tasks: [TASK-CRM-006]
---

## §1 — Statement (BCP-14 normative)

1. The lead-scoring model (`TASK-CRM-006`) **MUST** be re-evaluated quarterly against actual won/lost outcomes; AUC ≥ 0.70 to remain in production.
2. Score drift > 5pp between quarters triggers model retraining.
3. Model versions **MUST** be tagged; production deals carry the score-model-version that generated their score.
4. Outcome events (won/lost) **MUST** feed back into the training set; the model is not static.
5. The model **MUST NOT** use PII features (name, email, CCCD); only behavioural + firmographic features.

## §2 — Why this constraint

A lead-scoring model that drifts from reality misroutes sales effort. AUC ≥ 0.70 is a reasonable floor for "better than random + actually useful." Version-tagging scores enables retroactive comparison. PII exclusion is the privacy guardrail — a model that "learns" emails leaks data via inversion attacks.

## §3 — Measurement

- Quarterly AUC against held-out outcomes.
- Counter `crm_lead_score_drift_pct` per quarter.
- CI gate: model training input feature list has no PII.

## §4 — Verification

- Quarterly benchmark (T).
- Feature audit (T) — no PII.
- A/B harness for new model versions.

## §5 — Failure handling

- AUC < 0.70 → model retraining; ops continues on prior version.
- Drift > 5pp → retraining scheduled.
- PII feature detected → block training; investigate.

---

*End of NFR-CRM-003.*
