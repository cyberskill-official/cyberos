---
id: NFR-SKILL-010
title: "SKILL output contract conformance — produced output MUST match declared schema"
module: SKILL
category: maintainability
priority: MUST
verification: T
phase: P0
slo: "100% of skill outputs validate against their declared output contract"
owner: CTO
created: 2026-05-18
related_frs: [FR-SKILL-103, FR-SKILL-101]
---

## §1 — Statement (BCP-14 normative)

1. Every skill output **MUST** match the JSON Schema declared in `outputs:` of its `SKILL.md` frontmatter, validated by the runtime before the output is handed back to the caller.
2. Schema-mismatched output **MUST** be treated as a skill failure: the caller receives `E_OUTPUT_CONTRACT_VIOLATION`, the audit row records `outcome=contract_violation`, and the output is NOT returned to the caller (avoid type-confusion downstream).
3. The schema check **MUST** run on every output, every invocation — there is no opt-out.
4. The output schema **MUST** be the same one CUO chain-walks rely on for hand-off; chain-walkers can trust the declared shape.
5. Schema-mismatch rate **MUST** stay below 0.01% across all skill invocations in a 7-day window — anything higher indicates author drift between SKILL.md and runtime code.

## §2 — Why this constraint

Output contracts are the platform's typed-pipeline glue. CUO chains hand outputs to downstream skills; downstream code assumes the declared shape. A schema violation propagated downstream causes deep, far-from-source errors. The strict at-runtime gate fails the violation close to the source. The 0.01% budget is the tolerance for known LLM-output formatting noise; sustained higher rates signal real drift.

## §3 — Measurement

- Counter `skill_output_schema_check_total{skill, result=pass|fail}`.
- Gauge `skill_output_contract_violation_ratio{skill}` over 7-day rolling.
- Histogram `skill_output_schema_check_latency_seconds` — the schema check itself must be fast (< 5ms).

## §4 — Verification

- Per-skill CI test (T) — invoke skill with fixture inputs; assert output passes schema.
- Property test (T) — drive 100 random inputs; assert ≥ 99.99% of outputs pass.
- Production runtime check is the canonical gate; CI is a leading indicator.

## §5 — Failure handling

- Per-skill violation rate > 0.01% for 24h → sev-3; skill author notified.
- Per-skill violation rate > 1% sustained → sev-2; skill is auto-quarantined pending fix.
- Schema check latency > 5ms p99 → sev-3; check is too expensive; profile and optimise.

---

*End of NFR-SKILL-010.*
