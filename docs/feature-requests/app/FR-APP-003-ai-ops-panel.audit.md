---
fr_id: FR-APP-003
audited: 2026-06-29
verdict: PASS
score: 9.5/10
template: feature_request@1
rubric: feature-request-audit RUBRIC.md (FM / SEC / QA / COND / SAFE)
auditor: "@stephen (assisted)"
---

APP AI ops panel - the AI cost-and-health screen for the CyberOS operator console. It extends FR-APP-001's basic gateway-health screen with four read views: per-tenant cost-ledger spend against the monthly cap and warn threshold, the resolved model-alias map with provider and circuit-breaker health, response-cache hit and skip stats, and a read-only view of the tenant policy (alias map, caps, residency, ZDR). It is one more screen inside the same static single-page app under `apps/console/`, reusing FR-APP-001's shell, CDS tokens and components, auth gate, and API-client pattern, consuming only ai-gateway endpoints that already ship (FR-AI-022 / FR-AI-001 / FR-AI-002 / FR-AI-006 / FR-AI-009 / FR-AI-017 / FR-AI-005 / FR-AI-105) and adding no backend.

Frontmatter (FM-001..111, FM-004): file opens with `---` on line 1; all keys snake_case, no duplicates; template literal feature_request@1; id FR-APP-003; title 77 chars (FM-101 ok); author @stephen (FM-102 ok); department engineering; status draft; priority p3; created_at ISO 8601 with +07:00 offset (FM-106 ok); ai_authorship assisted; feature_type user_facing; eu_ai_act_risk_class not_ai; target_release 2026-Q4 (FM-110 ok); client_visible false (YAML boolean, FM-111 ok); module app; new_files and depends_on are lists, and every depends_on id resolves to a real FR (FR-APP-001 the parent console, FR-AUTH-004 the JWT, and the ai-gateway FRs FR-AI-022 / FR-AI-001 / FR-AI-002 / FR-AI-006 / FR-AI-009 / FR-AI-017 / FR-AI-005 / FR-AI-105). Required sections (SEC-001..008): Summary, Problem, Proposed Solution with a "Section 1 - normative requirements (BCP-14)" block of 10 numbered MUST / MUST NOT clauses, Alternatives Considered (3 distinct - reading raw JSON as today, a separate standalone AI app, and a thin aggregator backend in app; QA-005 ok), Success Metrics (one primary + one guardrail, each with definition / baseline / target / measurement method / source; QA-004 + QA-007 ok), Scope with an "### Out of scope" subsection of 5 items (QA-006 ok), Dependencies listing the parent console FR, the auth FR, and each ai-gateway FR the panel reads (QA-008 ok). Heading hierarchy is well-formed, one H1, no H2-to-H4 skips (SEC-009 ok); every required H2 has body (SEC-008 ok).

Conditionals: eu_ai_act_risk_class is not_ai, so COND-003 does not fire and there is correctly no AI Risk Assessment section - right for a console panel that reads gateway state and runs no model itself. client_visible is false, so COND-001 and COND-002 do not fire and there are correctly no Customer Quotes or Sales/CS Summary sections. ai_authorship is assisted, so COND-004 fires and the AI Authorship Disclosure section is present with the three required bullets (Tools used / Scope / Human review). No untrusted-content blocks appear, so the SAFE rules are not triggered.

Open items (the -0.5): the first release renders the tenant policy and the cost, model-health, and cache views read-only, so the panel is an operator viewing surface before it is a control surface - policy editing and any gateway mutation (cap changes, breaker resets, cache flush) are deferred; and the "no new backend" rule depends on every view mapping to an already-shipped ai-gateway endpoint, which the guardrail metric and the API-layer review are set up to enforce, with the residency and ZDR fields of the policy view leaning on FR-AI-005 / FR-AI-015 / FR-AI-016 returning them through the gateway's policy read. Both are disclosed in Scope and in the normative clauses (clause 2, clause 7, and the read-only wording across clauses 4-7) rather than hidden.

Verdict: PASS. Ready for implementation as a screen set inside the FR-APP-001 console shell; the policy-edit and gateway-control screens are the named follow-ups.

*End of FR-APP-003 audit.*
