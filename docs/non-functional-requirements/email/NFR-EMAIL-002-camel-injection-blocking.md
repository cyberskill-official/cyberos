---
id: NFR-EMAIL-002
title: "EMAIL CaMeL prompt-injection blocking — recall ≥ 95% on the held-out adversarial set"
module: EMAIL
category: security
priority: MUST
verification: T
phase: P0
slo: "Recall ≥ 95% on the platform's adversarial prompt-injection email corpus"
owner: CISO
created: 2026-05-18
related_tasks: [TASK-EMAIL-005]
---

## §1 — Statement (BCP-14 normative)

1. The CaMeL dual-LLM defence (`TASK-EMAIL-005`) **MUST** detect prompt-injection content in inbound mail at **recall ≥ 95%** on the platform's held-out adversarial corpus (200+ samples curated by CISO).
2. False-positive rate on the legitimate-mail corpus **MUST** stay ≤ 0.5%.
3. The held-out corpus **MUST** be refreshed quarterly with new adversarial patterns; benchmark metrics tracked over time.
4. CaMeL block decisions **MUST** be auditable: every blocked message carries `{block_reason, classifier_score, classifier_version, sampled_for_review=bool}`.
5. False-positive review cycle: 5% of CaMeL blocks **MUST** be sampled for human review weekly; persistent FP patterns inform classifier retuning.

## §2 — Why this constraint

Prompt injection via email is a serious attack vector: a benign-looking email could instruct downstream LLM workflows to exfiltrate data, fabricate replies, etc. The 95% recall is the security floor; below this, attackers reliably get through. The 0.5% FP rate is the usability floor; above this, users lose trust and disable the feature. The quarterly corpus refresh keeps the benchmark current against evolving attacks.

## §3 — Measurement

- Per-quarter benchmark: recall + FP rate against held-out + legitimate corpus.
- Counter `email_camel_block_total{block_reason}`.
- Counter `email_camel_false_positive_confirmed_total` (from weekly review).

## §4 — Verification

- Quarterly benchmark (T) — assert recall ≥ 95% + FP ≤ 0.5%.
- Adversarial test (T) — CI runs a 50-sample subset on every PR.
- Manual review queue + SLA for handling reported false positives.

## §5 — Failure handling

- Quarterly recall < 95% → sev-2; retune classifier; CISO + product engaged.
- FP rate > 0.5% → sev-3; user-facing trust damage; investigate.
- Pattern of evading attacks → sev-2 + security postmortem.

---

*End of NFR-EMAIL-002.*
