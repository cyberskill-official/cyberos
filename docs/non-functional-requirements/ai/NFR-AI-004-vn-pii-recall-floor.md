---
id: NFR-AI-004
title: "VN-PII detector recall floor — recall ≥ 99% on quarterly refresh corpus"
module: AI
category: privacy
priority: MUST
verification: T
phase: P0
slo: "Recall ≥ 99% on the VN-PII test corpus; refreshed quarterly"
owner: CSO
created: 2026-05-18
related_frs: [FR-AI-012, FR-AI-013, FR-AI-011]
---

## §1 — Statement (BCP-14 normative)

1. The Vietnamese PII detector plugin **MUST** achieve **recall ≥ 99%** on the curated VN-PII test corpus (`services/ai-gateway/tests/fixtures/vn_pii_corpus_v*.jsonl`) covering: CMND/CCCD national IDs, MST tax IDs, Vietnamese bank account numbers, Vietnamese phone numbers (mobile + landline), full names with diacritics, and Hanoi/HCMC street addresses.
2. The corpus **MUST** be refreshed at least quarterly (every calendar quarter) with at least 50 new ground-truth examples drawn from anonymised production traffic samples (NEVER raw production text — only synthetically reconstructed shapes).
3. Precision **MUST** be ≥ 95% (false-positive rate ≤ 5%) — VN-PII over-detection forces customer-visible redaction of innocent strings.
4. Each corpus refresh **MUST** be committed with a paired audit `RUBRIC.md`-style entry showing per-category recall AND precision; degradation from the previous quarter's numbers must be justified in the commit message.
5. The recall measurement **MUST** be re-run on every `services/ai-gateway/src/pii/vn/**` change in CI; PR fails if recall drops below 99% on the current corpus.

## §2 — Why this constraint

Vietnamese PII is the platform's compliance differentiator. Presidio's English defaults miss CMND (9 or 12 digits with no separator), MST (10 digits with optional dash), and diacritic-bearing personal names. A 99% recall floor is the threshold below which PDPL Article 7 "data minimisation" arguments fail — regulators will treat a 95%-recall detector as non-functional. The 95% precision floor protects against the equally bad failure mode of over-redaction (customer complaints, support cost).

## §3 — Measurement

Test harness `services/ai-gateway/tests/vn_pii_recall_test.rs` loads the JSONL corpus, runs each example through the detector, and computes recall per category and overall. Output schema:

```json
{
  "corpus_version": "v3",
  "overall_recall": 0.9942,
  "overall_precision": 0.971,
  "per_category": {
    "cccd": {"recall": 0.998, "precision": 0.99, "n": 50},
    "mst":  {"recall": 0.994, "precision": 0.98, "n": 50},
    "...": {"...": "..."}
  }
}
```

Asserted in CI; report archived to `docs/audits/vn-pii-recall/YYYY-Q*.json`.

## §4 — Verification

- CI gate (T) — `make test-vn-pii-recall` runs the harness; PR blocked on overall_recall < 0.99 or any per-category recall < 0.95.
- Quarterly review (A) — CSO reviews the diff between Q-1 and Q corpus; signs off in `docs/feature-requests/ai/FR-AI-012-vn-pii-plugin.audit.md`.

## §5 — Failure handling

- Recall drops below 99% on next quarter's corpus refresh → sev-2 ticket; freeze new VN tenant onboarding until corpus engineer re-trains regex/NER pipeline.
- Precision drops below 95% → sev-3; investigate over-redaction complaints in support queue.
- Two consecutive quarterly drops → escalate to CSO + CEO; consider whether the platform's "Vietnamese PII-first" marketing claim must be temporarily withdrawn.

---

*End of NFR-AI-004.*
