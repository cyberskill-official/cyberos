---
id: NFR-AI-010
title: "AI persona-version drift detection — LangSmith cosine deviation > 0.30 triggers alert"
module: AI
category: observability
priority: SHOULD
verification: T
phase: P1
slo: "Drift alert fires within 24h of cosine deviation > 0.30 on a persona's eval set"
owner: CTO
created: 2026-05-18
related_frs: [FR-AI-014, FR-OBS-004]
---

## §1 — Statement (BCP-14 normative)

1. Every active CUO persona **MUST** carry a versioned eval set (≥ 50 representative prompts per persona) committed under `modules/cuo/personas/<role>/evals/v<N>.jsonl`.
2. The AI Gateway **MUST** sample 1% of live traffic per persona into LangSmith with the `persona_version` tag, enabling cosine-similarity comparison against the baseline eval set's responses.
3. If the rolling 7-day cosine similarity between current production responses and the baseline eval responses drops by **> 0.30** absolute, the gateway **MUST** trigger a sev-3 drift alert via OBS within 24 hours.
4. The drift alert payload **MUST** carry `{persona_role, persona_version, baseline_cosine, current_cosine, sample_size_n}` and route to the CUO supervisor's runbook for persona-drift triage.
5. The eval baseline **MUST** be refreshed every persona-version bump (e.g., `cto/v3 → cto/v4`); the new baseline replaces the prior version's reference for future drift calculations.

## §2 — Why this constraint

LLM upstream models silently shift over time (Anthropic publishes minor model updates; OpenAI's `gpt-4o` changes weekly). A persona that gave consistent advice yesterday may give materially different advice today — without alerting, this drift is invisible until a customer complains. The 0.30 cosine threshold is calibrated against historical noise (cross-day variance on a stable persona is ~0.10-0.15); 0.30 catches real shifts without firing on routine noise. The 24h detection window is the tradeoff between fast alerting and false-positive cost from small sample sizes.

## §3 — Measurement

- LangSmith dataset `cyberos-persona-drift-<role>` updated continuously from the 1% sample.
- Daily job `services/ai-gateway/src/drift/persona_drift_check.rs` computes cosine vs baseline; writes gauge `ai_gateway_persona_cosine_drift{persona_role, persona_version}`.
- Sev-3 alarm at drift > 0.30; sev-2 at drift > 0.50.

## §4 — Verification

- Synthetic drift test `services/ai-gateway/tests/persona_drift_test.rs` (T) — injects a known-divergent response into the LangSmith sample; asserts the drift job flags it within one daily run.
- Quarterly review (A) — CTO reviews drift trends per persona; persona-versions with chronic > 0.20 drift get human re-evaluation.

## §5 — Failure handling

- Drift > 0.30 → sev-3; ticket auto-opened to persona-owner (per CUO role mapping); investigate whether upstream model changed.
- Drift > 0.50 → sev-2; consider pinning to a previous upstream model version until persona is re-tuned.
- Eval set baseline missing for an active persona → block deploy of that persona version until baseline exists.

---

*End of NFR-AI-010.*
