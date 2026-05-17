---
contract_id: test-strategy
contract_version: v1
template_literal: test-strategy@1
description: Canonical test-strategy@1 — Test Strategy outline per SDP Template §4.6. Authored by test-strategy-author; validated by test-strategy-audit via test_strategy_rubric@1.0.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cto
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Risk-priority + tool choices are judgement." }
emitted_source_freshness_tier: 14
---

# `test-strategy@1` — canonical Test Strategy contract

> Frontmatter: `test-strategy-audit/RUBRIC.md` §2. Body: §3 (`SEC-001..010`) — scope / risk-based priorities / test levels (unit / integration / system / UAT) / test types (functional / performance / security / accessibility / regression) / environments + data / tooling / entry+exit criteria / defect management / metrics. Conditional: §4 (`COND-001..006`) — UI / public API / high risk / perf NFR / personal data / AI-driven.

## Citations

- SDP Template §4.6.
- OWASP Top 10:2025 — security-testing coverage.
- WCAG 2.2 — accessibility-testing coverage.
- ISO/IEC 25010:2023 — NFR coverage.
- Consumers: `test-strategy-author`, `test-strategy-audit`.
