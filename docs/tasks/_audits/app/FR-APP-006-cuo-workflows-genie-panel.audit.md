---
task_id: TASK-APP-006
audited: 2026-06-29
verdict: PASS
score: 9.5/10
template: task@1
rubric: task-audit RUBRIC.md (FM / SEC / QA / COND / SAFE)
auditor: "@stephen (assisted)"
---

APP CUO workflows and GENIE assistant panel - one more panel in the TASK-APP-001 operator console (the same static single-page app under `apps/console/`, reusing its shell, auth gate, and CDS design language, adding no backend). The panel lists the CUO workflows and skills the mcp-gateway exposes via `tools/list` (TASK-MCP-001), triggers a run with `tools/call` and a JSON arguments payload, monitors status through the tasks primitive (TASK-MCP-007), surfaces the destructive-step confirmation/elicitation gate (TASK-MCP-006 / TASK-MCP-008) as a confirm prompt it must not bypass, and renders the GENIE assistant chat over the ai-gateway chat surface (TASK-AI-022), all carrying the TASK-AUTH-004 session token behind the existing Caddy front.

Frontmatter (FM-001..111, FM-004): file opens with `---` on line 1; all keys snake_case, no duplicates; template literal task@1; id TASK-APP-006; title within length (FM-101 ok); author @stephen (FM-102 ok); department engineering; status draft; priority p3; created_at ISO 8601 with +07:00 offset (FM-106 ok); ai_authorship assisted; feature_type user_facing; eu_ai_act_risk_class not_ai; target_release 2026-Q4 (FM-110 ok); client_visible false (YAML boolean, FM-111 ok); module app; new_files and depends_on are lists, with new_files all under apps/console/src/... and apps/console/tests/... and depends_on naming the real consumed FRs (TASK-APP-001, TASK-AUTH-004, TASK-MCP-001, TASK-MCP-006, TASK-MCP-007, TASK-MCP-008, TASK-CUO-101, TASK-AI-022). Required sections (SEC-001..008): Summary, Problem, Proposed Solution with a "Section 1 - normative requirements (BCP-14)" block of 10 numbered MUST / MUST NOT clauses, Alternatives Considered (3 distinct - desktop trigger as the only surface, a standalone GENIE app, and calling CUO or a provider directly past the gateways; QA-005 ok), Success Metrics (one primary + one guardrail, each with definition / baseline / target / measurement method / source; QA-004 + QA-007 ok), Scope with an "### Out of scope" subsection of 5 items (QA-006 ok), Dependencies listing the mcp-gateway, CUO, ai-gateway, auth, and TASK-APP-001 FRs plus the Caddy front (QA-008 ok). Heading hierarchy is well-formed, one H1, no H2-to-H4 skips (SEC-009 ok); every required H2 has body (SEC-008 ok).

Conditionals: eu_ai_act_risk_class is not_ai, so COND-003 does not fire and there is correctly no AI Risk Assessment section - defensible because the panel only renders the GENIE chat and routes it to the ai-gateway, and clause 7 places the assistant's behaviour and any model risk on the ai-gateway and CUO FRs rather than on this presentation panel. client_visible is false, so COND-001 and COND-002 do not fire and there are correctly no Customer Quotes or Sales/CS Summary sections. ai_authorship is assisted, so COND-004 fires and the AI Authorship Disclosure section is present with the three required bullets (Tools used / Scope / Human review). No untrusted-content blocks appear, so the SAFE rules are not triggered.

Open items (the -0.5): the first release is read-and-trigger oriented (list, trigger, monitor, chat) and workflow authoring or editing is deferred to CUO, so the panel runs and watches workflows before it can author them; and the "no new backend" and "never bypass the destructive-step gate" rules depend on every screen mapping to an already-shipped mcp-gateway or ai-gateway endpoint and on the confirmation gate being honoured, which the guardrail metric, the API-layer review, and clauses 2 and 6 are set up to enforce. Both are disclosed in Scope and in the normative clauses rather than hidden.

Verdict: PASS. Ready for implementation as a panel in the TASK-APP-001 console: the `tools/list` catalogue, the `tools/call` trigger with JSON arguments, the tasks-primitive status view, the destructive-step confirm prompt, and the GENIE chat tab, all over the shipped gateway surfaces; workflow authoring is the named follow-up in CUO.

*End of TASK-APP-006 audit.*
