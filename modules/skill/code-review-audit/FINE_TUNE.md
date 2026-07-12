# `code-review-audit` — fine-tune discipline override

Default discipline at `../docs/FINE_TUNE.md`. This file documents the **code-review-audit-specific overrides**.

## Why code-review-audit is different

This rubric carries AI-coding-industry-specific rules that drift faster than other rubrics:

- **QA-SIZE-001** — DORA's 2024 finding that AI-assisted code work inflates batch sizes. The 500-LOC threshold is current best-practice but the data evolves yearly.
- **QA-RUBBER-001** — heuristic to flag rubber-stamp reviews of AI-generated code. The 5-minute / 200-LOC threshold is calibration-sensitive.
- **SEC-AI-001..005** — the mandatory AI-specific review blocks (hallucinated-API check / oversized-diff / dependency provenance / PR-label verification). Industry conventions for these are still settling.

These rules SHALL be re-calibrated annually against the latest DORA Accelerate State of DevOps Report + ThoughtWorks Tech Radar.

## Annual recalibration cadence

Each January, the CTO (or designee) SHALL:

1. Pull the latest DORA Accelerate report.
2. Review the AI-tooling impact analysis chapter.
3. Compare CyberSkill's per-PR data against industry averages.
4. Propose adjustments to QA-SIZE-001 / QA-RUBBER-001 thresholds.
5. Land changes as a minor bump (`code_review_rubric@1.x → @1.x+1`) with full changelog rationale.

## SEC-AI-* additions over time

When a new AI-coding risk is recognised by the industry (e.g. a new failure mode like "AI-generated infrastructure code"), add a new SEC-AI-NNN block as required-when-`ai_assisted: true`. Each addition is a minor bump.

| Change | Bump | Reviewer |
|---|---|---|
| Adjust QA-SIZE-001 threshold based on DORA data | minor | CTO |
| Adjust QA-RUBBER-001 heuristic based on observed false-positive rate | minor | CTO + CPO |
| Add a new SEC-AI-NNN block | minor | CTO + CSecO |
| Add a new auto-fixable rule for AI-code review | minor | CTO |
| Change the OWASP A03 (Software Supply Chain) treatment expectations | minor | CTO + CSecO |

## Forbidden without major version bump

- Removing any SEC-AI-* block (these encode AI-coding hygiene).
- Bypassing the `ai_assisted: true → SEC-AI-* mandatory` linkage.
- Reducing `pr_size_loc > 500` warning to info-only.

## Specific data sources for fine-tune triggers

| Trigger | Source | Cadence |
|---|---|---|
| QA-SIZE-001 threshold update | DORA Accelerate annual report | annual (Jan) |
| QA-RUBBER-001 calibration | CyberSkill internal review-velocity data | quarterly |
| SEC-AI-* additions | ThoughtWorks Tech Radar + DORA + arXiv security research | as-needed |
| OWASP A03 treatment | OWASP Top 10 release cycle (next: 2028) | per-release |

## Cross-references

- `RUBRIC.md` — the rubric body.
- `../docs/FINE_TUNE.md` — master default discipline.
- DORA Accelerate State of DevOps Report — primary calibration source.
- OWASP Top 10:2025 — A03 source.
- `../../../modules/cuo/docs/module.md` §5 — AI integration discipline this rubric encodes.
