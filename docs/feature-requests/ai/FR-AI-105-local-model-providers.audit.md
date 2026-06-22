---
fr_id: FR-AI-105
audited: 2026-06-22
verdict: PASS
score: 9.5/10
template: feature_request@1
rubric: feature-request-audit RUBRIC.md (FM / SEC / QA / COND / SAFE)
auditor: "@stephen (assisted)"
---

Local + external model providers - two no-key local adapters (LM Studio via OpenAI-compatible /v1/chat/completions on :1234, Ollama-native /api/chat on :11434), endpoints from env, fail-closed, selectable per tenant through the FR-AI-006 alias map and FR-AI-008 fallback chain, plus the EchoBackend-to-real-router serving flip.

Frontmatter (FM-001..111, FM-004): all required keys present and well-typed; title 66 chars (FM-101 ok); template literal feature_request@1; priority p1; eu_ai_act_risk_class limited; ai_authorship assisted; client_visible false. Required sections (SEC-001..008): Summary, Problem, Proposed Solution (with a Section 1 normative BCP-14 block of 10 MUST clauses), Alternatives Considered (3 distinct, QA-005 ok), Success Metrics (one primary + one guardrail, each with definition / baseline / target / measurement_method / source, QA-004 + QA-007 ok), Scope with Out of scope (4 items, QA-006 ok), Dependencies (6 upstream FRs + the server-dispatch seam, QA-008 ok). Conditional (COND-003): AI Risk Assessment present with Data Sources, Human Oversight, Failure Modes in order. Conditional (COND-004): AI Authorship Disclosure present with Tools used / Scope / Human review. No untrusted-content blocks, so SAFE rules not triggered. Heading hierarchy well-formed (SEC-009).

Open items (the -0.5): the cloud-provider key management is named as a seam but deferred to a separate FR, so the "external models" half is specified, not built; and the serving flip is the one architecturally significant change and should be reviewed by Stephen before implementation lands. Both are called out in Scope and Dependencies, so they are disclosed rather than hidden.

Verdict: PASS. Ready for implementation of the local half; the cloud-key FR and the serving-flip review are the two follow-ups.

*End of FR-AI-105 audit.*
