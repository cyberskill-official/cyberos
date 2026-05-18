---
skill_id: <artefact>-author
baseline_version: 1.0.0
baseline_measured_at: <ISO 8601 with timezone — when this baseline was measured>
attested_by: <cuo-<role> | human:<id>>
next_review_due: <ISO 8601 — default +12 months from baseline_measured_at>
---

# BASELINE for <artefact>-author

> **Purpose:** design-time performance baseline at v0.x → v1.0 promotion. Per FR-SKILL-114. The artefact justifies the promotion + anchors future drift detection. Required when `skill_version >= 1.0.0`; advisory before then.

## Workflow under test

Describe the workflow this skill automates. State the human-baseline (operator does it by hand) and the skill-augmented form. Include a one-sentence "what the skill does" framing.

## Without-skill baseline

**Measurement window:** YYYY-MM-DD → YYYY-MM-DD (N weeks). **Sample size:** n=N real sessions across <persona(s)>.

| Measurement | Without skill (mean ± stddev) | Methodology |
|---|---|---|
| Tool-call count | <N> ± <σ> | OBS `tool_call_count` per session; filter by `task_taxonomy: <slug>` |
| Token count | <N>,000 ± <σ>,000 | OBS `tokens_total` per session |
| Failure rate | <N>% | Sessions ending in "abort" or "reformulate" / total |

## With-skill measurements

Same window, same sample (n=N operators chose to re-run with the skill).

| Measurement | With skill (mean ± stddev) | Ratio vs baseline | Pass threshold |
|---|---|---|---|
| Tool-call count | <N> ± <σ> | <ratio> | ≤0.7 |
| Token count | <N>,000 ± <σ>,000 | <ratio> | ≤0.7 |
| Failure rate | <N>% | <ratio> | ≤0.5 |
| Iteration count (audit-loop only) | <N> ± <σ> | — | info-only |

**Verdict:** all three thresholds passed [/ failed; see operator-override in Authoring notes].

## Token-budget transparency

Per-invocation token budget for `<artefact>-author`:

- **Prompt tokens (mean):** <N>,000
- **Prompt tokens (95th percentile):** <N>,000
- **Completion tokens (mean):** <N>,000
- **Completion tokens (95th percentile):** <N>,000
- **Total ceiling guidance:** budget <N>,000 tokens for a worst-case invocation.

## Trust calibration

`confidence_band.default: <N>` chosen because:

- <Rationale — why this default for this skill's domain>.

`defer_below: <N>` chosen because:

- <Rationale — why HITL fires at this floor>.

Empirical acceptance rate during the measurement window: **<N>%** (auto-pause threshold per DEC-055 is 40%).

## Authoring notes

- Sample-size caveats.
- Persona-distribution caveats (single-persona vs cross-persona).
- Operator-override reasoning (if any threshold failed).
- Attestation chain: measurement gathered by <agent / human>; reviewed by <persona>; signed off by <human or persona-id>.
