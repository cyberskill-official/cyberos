---
id: NFR-OBS-005
title: "LangSmith AI-trace retention — 90-day window, queryable by persona_version"
module: OBS
category: observability
priority: SHOULD
verification: I
phase: P1
slo: "AI traces queryable for 90 days; filter by persona_version returns results in < 5s"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-OBS-004, TASK-AI-014]
---

## §1 — Statement (BCP-14 normative)

1. Every AI Gateway call (1% sample per NFR-AI-010 + 100% of error/slow calls per NFR-OBS-003) **MUST** be exported to LangSmith with tags `{tenant_id, persona_version, route, model_alias, status}`.
2. LangSmith retention **MUST** be configured to keep traces for a minimum of 90 days; quarterly capacity reviews adjust retention if storage cost trends require.
3. A LangSmith query filtered by `persona_version=<value>` **MUST** return results in **< 5s p95** for the 90-day window.
4. The retention policy **MUST** be enforced at the LangSmith project level — not via ad-hoc cron deletes. If retention is shortened, the change goes through a CSO sign-off (compliance evidence path may depend on traces).
5. PII-redacted versions of prompts/responses **MUST** be the only versions in LangSmith (redaction per NFR-AI-004 applied **before** export).

## §2 — Why this constraint

90 days is the minimum to investigate quarterly persona drift, root-cause regressions reported by customers ("the AI gave bad advice last month"), and produce evidence for AI Act conformity assessments. The 5s query SLO is necessary for the LangSmith UI to be useful for ad-hoc drift investigation; longer queries kill the investigator's flow. PII redaction before export is critical — LangSmith is a third-party SaaS and storing raw PII there would breach the ZDR claim of NFR-AI-005's spiritual companion in the prompt-only context.

## §3 — Measurement

- LangSmith project-config audit (`docs/compliance/langsmith-config.md`) lists current retention; reviewed quarterly.
- LangSmith UI query benchmark — operator runs a `persona_version=cto/v3` filter monthly; logs result time. p95 from 12 monthly runs should be < 5s.
- Counter `ai_gateway_langsmith_export_failed_total` — should be near-zero.

## §4 — Verification

- Inspection (I) — quarterly CSO audit verifies LangSmith retention policy is 90d+, redaction is enabled, and no raw PII has leaked.
- Smoke test (T) — `tests/obs/langsmith_export_test.rs` exports a synthetic redacted trace and confirms it's queryable within 1 minute.

## §5 — Failure handling

- Export failures > 1% → sev-3; investigate LangSmith API health.
- Retention drops below 90d due to LangSmith billing constraints → CSO + CFO budget review; decide whether to absorb cost or self-host an alternative.
- PII detected in a LangSmith trace → sev-1; emergency redaction policy review; that tenant's traces purged from LangSmith.

---

*End of NFR-OBS-005.*
