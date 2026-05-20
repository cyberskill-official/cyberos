---
contract_id: code-review
contract_version: v1
template_literal: code-review@1
description: Canonical code-review@1 — structured PR review write-up per IEEE 1028. Authored by code-review-author; validated by code-review-audit via code_review_rubric@1.0. Implements modules/cuo/README.md#software-development-process §2(g) + Template §4.5.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-cto
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }

determinism: { reproducible: false, fixity_notes: "Review judgement is per-PR. Structural section set + AI-specific blocks are reproducible." }

emitted_source_freshness_tier: 12
---

# `code-review@1` — canonical code-review write-up contract

> Frontmatter: `code-review-audit/RUBRIC.md` §2.
> Required body sections: §3 (`SEC-001..012`) covers Template §4.5 (correctness, readability, tests, secrets, injection, input validation, error handling, logging, perf, backwards-compat, SAST/SCA, SBOM).
> AI-specific blocks: §4 (`SEC-AI-001..005`) — mandatory when `ai_assisted: true`.
> Conditional sections: §5 (`COND-001..005`) — DB migration, auth/crypto, API surface, personal data, conditional approval.

## Citations

- IEEE 1028-2008 — review/audit standard.
- SDP §2(g) + Template §4.5.
- SDP §5 — AI-integration discipline (drives the SEC-AI-* blocks per the DORA 2024 findings on AI-assisted code review).
- Consumers: `code-review-author`, `code-review-audit`.
