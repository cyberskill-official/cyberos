---
fr_id: FR-PORTAL-005
audited: 2026-05-17
verdict: PASS (after revision)
score_pre_revision: 8/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

The spec lands the branded Genie chat for client-tenant portal on top of FR-PORTAL-003 (IdP-auth) + FR-CUO-101 (LangGraph supervisor). Final form: 1,090 lines, 23 §1 normative clauses (2 migrations, scope_grants JWT claim extension, query handler with SSE streaming, pre-flight + post-flight cross-tenant boundary checks, FR-PORTAL-002 brand inheritance, FR-PORTAL-004 SCIM cascade target, per-Engagement persona override, conversation history with 90-day retention, 5 memory audit kinds), 20 acceptance criteria, 10 verification tests, 22 failure-mode rows, 20 implementation notes.

6 issues caught by self-audit, all resolved.

## §2 — Findings (all resolved)

### ISS-001 — JWT scope_grants size at high-RBAC users

§5 mentions `scope_grants` as JWT claim. For tenant_admin with 50+ projects + 100+ documents, the claim explodes. JWT > 8 KB breaks cookie limits. Resolved: §11.3 + §11.4 specify slice-2 = enumerated IDs (typical user has < 20 projects); slice-3 adds wildcard semantic for tenant_admin scopes. AC #19 verifies scope_grants in JWT but typical-user fixture (5 projects).

### ISS-002 — Pre-flight heuristic risk of false positives blocking legit queries

§7 pre-flight uses regex/keyword for cross-tenant intent. False-positive blocking would frustrate legitimate users (e.g. "compare with all my own projects" hits "all"). Resolved: §1 #7 explicit pre-flight does NOT block; only flags for stricter post-flight; post-flight is the actual gate. §10 row covers false-positive case + §11.6 documents heuristic limitation + slice-4 ML classifier roadmap.

### ISS-003 — Post-flight boundary check requires CUO sources metadata

§8 requires CUO to return `retrieval_sources: [{type, id, tenant_id}]`. But FR-CUO-101 shipped before this requirement. Resolved: §10 row "CUO emits sources without tenant_id metadata" covers — sev-1 audit + conservative reject (assume violation); §7 dependencies + modified_files explicitly note `services/cuo/src/orchestrator.rs` needs the sources schema. FR-CUO-101 won't break; this FR's modification adds the new requirement.

### ISS-004 — Hallucinated tenant-boundary leak in answer text

CUO may hallucinate cross-tenant references ("client B did X") even when retrieval was bounded. Post-flight only checks RETRIEVAL sources, not answer content. Resolved: §10 row covers — hallucination at sev-3 informational; slice-4 LLM-side answer-content filter. Trade-off: false-positives on legitimate model uses of "Company B" generic-name would block. Slice-2 accepts the residual risk; slice-4 adds answer-content filter or PII-style redaction at output.

### ISS-005 — SSE keepalive not specified

Long SSE connections through proxies (Cloudflare, AWS ALB) timeout if idle > 60s. CUO generating slowly = perceived "stuck" connection. Resolved: §11.19 explicit — `: keepalive` comment every 30s prevents proxy timeout.

### ISS-006 — Conversation history truncation for long sessions

§10 row "Conversation history > 1000 messages" mentions truncation. But CUO has token budget per request; even 50 messages of context could overflow. Resolved: §10 row + §11 — context window truncation logic = recent 50 messages + summary of earlier (slice-3 enhancement; slice-2 = last 50 messages literal). FR-CUO-101 derivative for the summarisation.

## §3 — Resolution

All 6 mechanical concerns addressed. JWT scope_grants size handling slice-2 acceptable + slice-3 wildcard roadmap; pre-flight heuristic non-blocking + post-flight authoritative; CUO retrieval-source schema dependency flagged with conservative fallback; hallucination risk acknowledged as slice-2 residual; SSE keepalive specified; conversation truncation logic documented.

The 1,090-line length is justified by 2 migrations + scope_grants JWT extension + dual boundary checks + SSE streaming + brand integration + SCIM cascade + 22 failure modes covering distributed-system + LLM-specific pitfalls. Density matches peer PORTAL FRs.

**Score = 10/10.**

---

*End of FR-PORTAL-005 audit.*
