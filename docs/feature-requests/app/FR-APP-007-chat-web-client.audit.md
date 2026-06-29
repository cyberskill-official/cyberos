---
fr_id: FR-APP-007
audited: 2026-06-29
verdict: PASS
score: 9.5/10
template: feature_request@1
rubric: feature-request-audit RUBRIC.md (FM / SEC / QA / COND / SAFE)
auditor: "@stephen (assisted)"
---

APP chat web client - a self-contained, CDS-styled chat surface for the first-party cyberos-chat service (FR-CHAT-101). It consumes only the shipped chat REST and websocket surface, authenticates with a CyberOS access token (FR-AUTH-110), and adds no backend; it is a sibling of the FR-APP-001 operator console and deploys as a static file behind the same Caddy front.

Frontmatter (FM-001..111): file opens with `---` on line 1; all keys snake_case, no duplicates; template literal feature_request@1; id FR-APP-007; title under 64 chars (FM-101 ok); author @stephen (FM-102 ok); department engineering; status implementing; priority p3; created_at ISO 8601 with +07:00 offset (FM-106 ok); ai_authorship assisted; feature_type user_facing; eu_ai_act_risk_class not_ai; target_release 2026-Q4 (FM-110 ok); client_visible false (YAML boolean, FM-111 ok); module app; new_files and depends_on are lists. Required sections (SEC-001..008): Summary, Problem, Proposed Solution with a "Section 1 - normative requirements (BCP-14)" block of 10 numbered MUST / MUST NOT clauses, Alternatives Considered (3 distinct - fold into the FR-APP-001 console, framework-with-build-step, thin backend; QA-005 ok), Success Metrics (one primary + one guardrail, each with definition / baseline / target / measurement method / source; QA-004 + QA-007 ok), Scope with an "### Out of scope" subsection of 5 items (QA-006 ok), Dependencies listing FR-CHAT-101, FR-AUTH-110, FR-APP-001, and the Caddy front (QA-008 ok). Heading hierarchy well-formed, one H1, no H2-to-H4 skips (SEC-009 ok); every required H2 has body (SEC-008 ok).

Conditionals: eu_ai_act_risk_class is not_ai, so COND-003 does not fire and there is correctly no AI Risk Assessment section. client_visible is false, so COND-001 and COND-002 do not fire and there are correctly no Customer Quotes or Sales/CS Summary sections. ai_authorship is assisted, so COND-004 fires and the AI Authorship Disclosure section is present with the three required bullets (Tools used / Scope / Human review). No untrusted-content blocks appear, so the SAFE rules are not triggered.

Open items (the -0.5): the first client slice is the flat channel transcript - threads, edit/delete, attachments upload, search UI, and a call UI are deferred to later additive slices, and the server supports all of them already; and the one service change the FR allows (the opt-in `CHAT_DEV_CORS` dev flag) is a development convenience that ships off and adds no route, with production relying on single-origin Caddy serving instead. Both are disclosed in Scope and in clauses 1 and 10 rather than hidden.

Verdict: PASS. Ready as the first usable human surface for cyberos-chat; the richer chat affordances (threads, edit, attachments, search, calls) are the named follow-ups.

*End of FR-APP-007 audit.*
