---
fr_id: FR-APP-002
audited: 2026-06-22
verdict: PASS
score: 9.5/10
template: feature_request@1
rubric: feature-request-audit RUBRIC.md (FM / SEC / QA / COND / SAFE)
auditor: "@stephen (assisted)"
---

APP desktop workflow trigger - a Tauri desktop app (Rust backend, OS-webview front-end, one small multi-OS binary, macOS first) that lets the operator trigger CyberOS workflows and skills directly. It drives the existing ai-gateway HTTP surface (FR-AI-022 / FR-AI-105) and the mcp-gateway MCP surface (FR-MCP-001), signs in through the existing auth service (FR-AUTH-004 / FR-AUTH-005), stores the session token in the OS keychain, and re-implements no workflow logic - CUO (FR-CUO-101) orchestrates behind the endpoints.

Frontmatter (FM-001..111, FM-004): file opens with `---` on line 1; all keys snake_case, no duplicates; template literal feature_request@1; id FR-APP-002; title 65 chars (FM-101 ok); author @stephen (FM-102 ok); department engineering; status draft; priority p2; created_at ISO 8601 with +07:00 offset (FM-106 ok); ai_authorship assisted; feature_type user_facing; eu_ai_act_risk_class not_ai; target_release 2026-Q4 (FM-110 ok); client_visible false (YAML boolean, FM-111 ok); module app; new_files and depends_on are lists. Required sections (SEC-001..008): Summary, Problem, Proposed Solution with a "Section 1 - normative requirements (BCP-14)" block of 10 numbered MUST / MUST NOT clauses, Alternatives Considered (3 distinct - Electron, a terminal TUI, and reusing the portal write path; QA-005 ok), Success Metrics (one primary + one guardrail, each with definition / baseline / target / measurement method / source; QA-004 + QA-007 ok), Scope with an "### Out of scope" subsection of 5 items (QA-006 ok), Dependencies listing the gateway, MCP, CUO, and auth FRs plus the keychain seam (QA-008 ok). Heading hierarchy is well-formed, one H1, no H2-to-H4 skips (SEC-009 ok); every required H2 has body (SEC-008 ok).

Conditionals: eu_ai_act_risk_class is not_ai, so COND-003 does not fire and there is correctly no AI Risk Assessment section. client_visible is false, so COND-001 and COND-002 do not fire and there are correctly no Customer Quotes or Sales/CS Summary sections. ai_authorship is assisted, so COND-004 fires and the AI Authorship Disclosure section is present with the three required bullets (Tools used / Scope / Human review). No untrusted-content blocks appear, so the SAFE rules are not triggered.

Open items (the -0.5): Windows and Linux are named follow-on targets but only macOS is built and shipped in v1, so the multi-OS promise is specified, not delivered; and the keychain backends for those platforms are sketched behind the storage trait but exercised only on macOS first. Both are disclosed in Scope and in the normative clauses (clause 6 and clause 4) rather than hidden.

Verdict: PASS. Ready for implementation of the macOS build; the Windows and Linux targets and their keychain backends are the named follow-ups.

*End of FR-APP-002 audit.*
