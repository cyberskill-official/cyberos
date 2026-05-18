---
id: NFR-BRAIN-007
title: "BRAIN PII pre-ingest gate — Presidio + VN-PII recall ≥ 99% before row hits Layer 2"
module: BRAIN
category: privacy
priority: MUST
verification: T
phase: P0
slo: "100% of l2_memory rows have been through PII detection at recall ≥ 99% before commit"
owner: CSO
created: 2026-05-18
related_frs: [FR-BRAIN-111, FR-AI-012, FR-AI-013]
---

## §1 — Statement (BCP-14 normative)

1. Every row entering `l2_memory` **MUST** first pass through the PII detection pipeline: Presidio (English) followed by the VN-PII plugin (Vietnamese, NFR-AI-004 recall floor).
2. Detected PII spans **MUST** be redacted (replaced with `<<PII:TYPE>>` placeholders) before the row is committed to `l2_memory`. The original (unredacted) content **MUST NOT** be persisted anywhere downstream of the pre-ingest stage.
3. The detection pipeline **MUST** be transactional with the ingest: detection failure (e.g., pipeline crashed) rolls back the ingest; the cursor does not advance.
4. The pipeline **MUST** emit a counter `brain_pii_pre_ingest_total{pii_class, redacted}` for every detection outcome; sustained zero detections on a Vietnamese-text tenant is a sev-3 anomaly (pipeline may be silently broken).
5. The combined English+VN recall **MUST** be ≥ 99% on the BRAIN-specific test corpus (separate from the AI Gateway corpus per NFR-AI-004; BRAIN sees longer-form text).

## §2 — Why this constraint

The BRAIN store is the platform's long-term memory; PII leakage here is the worst kind because it's durable. Pre-ingest detection means PII never enters the search/retrieval surface — even an attacker who reads l2_memory directly gets redacted placeholders. The transactional rule prevents the failure mode of "detection crashed mid-row → row committed un-redacted." The counter monitors detection health; the recall floor is the contractual guarantee.

## §3 — Measurement

- Counter `brain_pii_pre_ingest_total{pii_class, redacted}` per detection.
- Counter `brain_pii_pipeline_failure_total` — should be near-zero; sev-2 on sustained > 1/min.
- Eval `services/brain/tests/pii_recall_corpus_v*.jsonl` — run quarterly, asserts overall recall ≥ 99%.

## §4 — Verification

- Recall test `services/brain/tests/pre_ingest_pii_recall_test.rs` (T) — runs the BRAIN-specific corpus; asserts ≥ 99%.
- Integration test (T) — ingests a row containing CMND + email; asserts row in l2_memory has redacted placeholders, not raw PII.
- Transactional test (T) — kill detection mid-row; asserts cursor doesn't advance and l2_memory has no partial row.

## §5 — Failure handling

- Recall < 99% on quarterly refresh → sev-2; pause new VN tenant onboarding; retrain detection.
- Detection pipeline crash → sev-2; pause ingest; investigate; resume after fix.
- Unredacted PII found in l2_memory (post-hoc audit) → sev-1; immediate quarantine of row; root cause + corpus refresh.

---

*End of NFR-BRAIN-007.*
