---
fr_id: FR-APP-001
audited: 2026-06-22
verdict: PASS
score: 9.5/10
template: feature_request@1
rubric: feature-request-audit RUBRIC.md (FM / SEC / QA / COND / SAFE)
auditor: "@stephen (assisted)"
---

APP CDS web console - a CyberOS operator and admin web console (CyberOS's own front-end, not a tenant portal) built with the CyberSkill Design System (CDS) tokens and components. It is a static single-page app over service APIs that already ship - the ai-gateway HTTP surface (FR-AI-022 / FR-AI-105), the obs compliance-view endpoints (FR-OBS-008), the obs-proxy (FR-OBS-002), auth (FR-AUTH-004 / FR-AUTH-104), and memory - adding no backend, and deployable behind the existing Caddy front.

Frontmatter (FM-001..111, FM-004): file opens with `---` on line 1; all keys snake_case, no duplicates; template literal feature_request@1; id FR-APP-001; title 64 chars (FM-101 ok); author @stephen (FM-102 ok); department engineering; status draft; priority p3; created_at ISO 8601 with +07:00 offset (FM-106 ok); ai_authorship assisted; feature_type user_facing; eu_ai_act_risk_class not_ai; target_release 2026-Q4 (FM-110 ok); client_visible false (YAML boolean, FM-111 ok); module app; new_files and depends_on are lists. Required sections (SEC-001..008): Summary, Problem, Proposed Solution with a "Section 1 - normative requirements (BCP-14)" block of 10 numbered MUST / MUST NOT clauses, Alternatives Considered (3 distinct - Grafana-only, extending the portal, and a server-rendered app with its own backend; QA-005 ok), Success Metrics (one primary + one guardrail, each with definition / baseline / target / measurement method / source; QA-004 + QA-007 ok), Scope with an "### Out of scope" subsection of 5 items (QA-006 ok), Dependencies listing the gateway, obs, and auth FRs plus the Caddy front (QA-008 ok). Heading hierarchy is well-formed, one H1, no H2-to-H4 skips (SEC-009 ok); every required H2 has body (SEC-008 ok).

Conditionals: eu_ai_act_risk_class is not_ai, so COND-003 does not fire and there is correctly no AI Risk Assessment section. client_visible is false, so COND-001 and COND-002 do not fire and there are correctly no Customer Quotes or Sales/CS Summary sections. ai_authorship is assisted, so COND-004 fires and the AI Authorship Disclosure section is present with the three required bullets (Tools used / Scope / Human review). No untrusted-content blocks appear, so the SAFE rules are not triggered.

Open items (the -0.5): the first release is read-oriented (compliance views, gateway health and usage) and operator mutation screens are deferred, so the console is a viewing surface before it is a full admin surface; and the "no new backend" rule depends on every screen mapping to an already-shipped endpoint, which the guardrail metric and the API-layer review are set up to enforce. Both are disclosed in Scope and in the normative clauses (clause 1 and clause 4) rather than hidden.

Verdict: PASS. Ready for implementation of the first two screen sets behind the auth-gated CDS shell; the operator-mutation screens are the named follow-ups.

*End of FR-APP-001 audit.*
