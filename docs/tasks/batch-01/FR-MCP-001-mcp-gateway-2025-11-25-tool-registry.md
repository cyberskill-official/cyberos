---
title: "MCP Gateway implementing 2025-11-25 spec — tool registry, OAuth 2.1 + PKCE, PRM discovery, destructive-tool gating"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p0
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: limited
target_release: "P0 / 2026-Q3"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Stand up the Model Context Protocol (MCP) Gateway that makes every CyberOS module agent-operable. The gateway implements the **2025-11-25 production-stable MCP specification** and is a federation router across per-module MCP servers — never a single monolithic MCP. It owns OAuth 2.1 with PKCE for client auth, audience-bound tokens with no upstream-token passthrough, the per-tenant authorisation server pattern (`https://{tenant-slug}.cyberos.world/.well-known/oauth-authorization-server`), the Protected Resource Metadata (PRM) discovery flow new in the 2025-11-25 spec (`/.well-known/oauth-protected-resource`), the unified tool registry with `cyberos.{module}.{verb}_{noun}` naming and conflict rejection at registration time, the destructive-tool annotation enforced as a human-in-the-loop confirmation step, and the ambient-mode safety contract that the CUO never auto-acts on irreversible operations regardless of agent intent. This is the substrate for Bet 1 (agent parity is the moat) and Bet 2 (CUO is the brand).

## Problem

The PRD makes a specific commercial bet: any task a human can perform in CyberOS, an AI agent can perform via MCP under the same RBAC, audit, and tenancy. That bet is worth nothing if the MCP surface is a different door from the human GraphQL door — every per-module security check would have to be re-implemented, and a 10-engineer team cannot maintain two parallel security postures. The MCP Gateway is therefore a *federation* of per-module servers that share Postgres connections, RBAC predicates, and audit writes with the GraphQL subgraphs they sit beside.

The 2025-11-25 spec line introduces three changes from the 2025-06-18 line we originally targeted (PRD §8.4):

- **Audience-bound tokens, no upstream passthrough.** The "token shadow handoff" via `X-Forwarded-Authorization` flagged by the security community in 2025 is rejected at gateway level. An Acme-tenant token cannot be replayed at the Beta-tenant gateway even if leaked.
- **Protected Resource Metadata (PRM) discovery.** Agents discover the authorisation server programmatically at `/.well-known/oauth-protected-resource`. We expose this and chain to the tenant's authorisation server.
- **Tool annotations with safety semantics.** `destructive: true`, `idempotent: true|false`, `requires_confirmation: true`, `read_only: true|false`. The Gateway enforces these at the call boundary.

Sprint S0-2 lands this gateway alongside AUTH and the AI Gateway; sprint S0-4 puts the first real consumer (CUO observing CHAT events) on top of it (PRD §17.4). The persona-scope contract risk-gate from S0-4 — "a synthetic prompt-injection in a CHAT message must not cause CUO to escape its scope" — is enforced at this gateway as well as at the CaMeL dual-LLM (FR-CHAT-001).

## Proposed Solution

The gateway is a single Kubernetes Deployment `cyberos-mcp-gateway` with three replicas, fronted by an HTTPS Ingress at `https://mcp.cyberos.world` for the canonical CyberSkill tenant and `https://{tenant-slug}.cyberos.world/mcp` for additional tenants from P3+. Per-module MCP servers run as sidecars to their subgraph deployments and register with the gateway at startup.

**Federation, not monolith.** Each module owns its MCP server: `cyberos-auth-mcp`, `cyberos-brain-mcp`, `cyberos-genie-mcp`, `cyberos-chat-mcp`, etc. Each server speaks MCP 2025-11-25 over stdio when run locally (for unit tests and developer experience) and over HTTP+SSE when run in production behind the gateway. The gateway is a thin router that:

1. Accepts incoming MCP HTTP+SSE connections from agent clients (Claude.ai, Cursor, Claude Desktop, the embedded CUO client).
2. Authenticates the connection via the OAuth 2.1 access token in `Authorization: Bearer ...`.
3. Resolves the tenant from the token's `aud` claim and the request's subdomain (rejecting any mismatch).
4. Proxies tool-list and tool-call requests to the appropriate per-module server based on the tool name's `cyberos.{module}.*` prefix.
5. Enforces tool annotations (destructive, idempotent, requires_confirmation, read_only) at the proxy boundary.
6. Writes an audit row for every tool list and tool invocation in scope `mcp.{tenant}`.

The gateway's own state is minimal: the registered tool catalogue (refreshed on each module's startup heartbeat), in-flight session bindings, and rate-limit counters.

**Tool naming convention.** `cyberos.{module}.{verb}_{noun}` — for example `cyberos.proj.create_task`, `cyberos.brain.search`, `cyberos.rew.payslip_explain`. The `payslip_explain` tool is read-only and produces a narrative; there is no `cyberos.rew.payslip_compute` tool — compute paths are deliberately non-MCP because they affect compensation and require human-only execution. Tool-name collisions are rejected at registration time with a 409.

**OAuth 2.1 with PKCE.** Every MCP client authenticates via Authorization Code with PKCE through the AUTH module's authorisation server. PKCE code challenge S256 is mandatory. The token's `aud` is exactly `https://mcp.cyberos.world` (canonical tenant) or `https://{tenant-slug}.cyberos.world/mcp` (other tenants); the gateway rejects any token whose `aud` does not match. There is no resource-owner-password-grant pattern, no API-key pattern, no service-account pattern. Agent clients are first-class OAuth clients registered per Member they act on behalf of (FR-AUTH-001 §"Agent authentication").

**No upstream token passthrough.** The gateway never sends the inbound `Authorization` header to the per-module MCP servers. Instead, it issues a short-lived (60-second) signed internal JWT containing the resolved subject, role, and tenant, and sends that to the module server. The internal JWT has `aud: cyberos-internal` and is signed by a key separate from the AUTH module's user-facing signing keys — a leaked internal JWT cannot be used to call public APIs. This kills the "token shadow handoff" attack class.

**PRM discovery.** The gateway exposes:

- `https://mcp.cyberos.world/.well-known/oauth-protected-resource` returning `{ "resource": "https://mcp.cyberos.world", "authorization_servers": ["https://auth.cyberos.world"], "bearer_methods_supported": ["header"] }`.
- `https://auth.cyberos.world/.well-known/oauth-authorization-server` returning the standard OAuth 2.1 metadata (issuer, authorization_endpoint, token_endpoint, jwks_uri, code_challenge_methods_supported, etc.).

An agent client following the 2025-11-25 spec can discover the authorisation server from the resource URL alone; no client-side configuration is needed beyond the resource URL.

**Tool registry.** Each per-module MCP server registers its tools at startup by `POST /mcp/registry/v1/register` with a body of `{ tools: [...] }`. The registry validates: (a) names match `^cyberos\.[a-z]+\.[a-z][a-z_]+_[a-z][a-z_]+$`, (b) annotations are present and well-typed, (c) names do not collide with other modules. The registry is in-memory per gateway replica with periodic NATS-broadcast reconciliation (the canonical truth lives in `cyberos_meta.mcp_tool` and survives gateway restarts).

**Tool annotations enforced at the proxy.**

- `read_only: true` — the tool reads but does not write. The gateway does not require confirmation; RBAC is the only gate.
- `destructive: true` — the tool deletes or revokes. The gateway requires `requires_confirmation: true`. Calls without `client_confirmed: true` in the request are rejected with `code: "DESTRUCTIVE_NEEDS_CONFIRMATION"`.
- `idempotent: true` — the tool can be safely retried with the same `Idempotency-Key`; the gateway dedupes by key for 24 hours.
- `irreversible: true` — the tool affects compensation, equity, signed contracts, payments, or any state that cannot be undone by a subsequent tool call. **Irreversible tools are not registered.** Compensation, equity, payments, and signature paths are non-MCP by architectural rule (PRD §6.4 defer-to-human triggers, §2.5 anti-positioning). The gateway logs an attempt to register an irreversible tool as a critical security event.

**Persona scope contract.** Every CUO persona is a Skills directory with a `scope_contract` block declaring which tool prefixes the persona can call. The gateway enforces this: a persona authored as `cuo-coo-v0.4` declaring `tools: ["cyberos.proj.*", "cyberos.chat.*", "cyberos.brain.*", "cyberos.time.*"]` cannot call `cyberos.rew.*` even if the underlying RBAC would otherwise allow it. The persona scope contract is the second floor under the Member's RBAC; both must allow.

**Per-tenant rate limiting.** Per tenant per minute and per hour ceilings, configurable per plan. Default P0 internal-only ceilings: 1,200 tool calls/min, 60,000 calls/hour. Excess returns 429 with `Retry-After`.

**Auditable.** Every tool list and tool call writes an audit row in scope `mcp.{tenant}` with: `tool_name`, `arguments_redacted`, `result_size`, `latency_ms`, `actor_subject`, `actor_agent_client`, `persona_version`, `confirmation_token` (if destructive). Arguments are passed through the Presidio redactor before being logged so audit rows do not become a PII back-door.

**Streaming.** The 2025-11-25 spec mandates HTTP+SSE for streaming tool outputs. The gateway terminates SSE on the inbound side and re-establishes SSE to the per-module server, forwarding events with a small `cyberos-correlation-id` header. Long-running tool calls (BRAIN reranking on a large corpus, KB Q&A with multi-step retrieval) stream tokens or progress events to the client.

**Health and observability.** `GET /health` returns `{ status: "healthy", registered_modules: [...], registered_tools: 47 }`. Gateway emits OpenTelemetry traces with span attributes `mcp.tool_name`, `mcp.tool_module`, `mcp.persona_version`, `mcp.actor_kind`. Prometheus metrics: `cyberos_mcp_calls_total{tool, module, status}`, `cyberos_mcp_call_duration_seconds_bucket{tool}`, `cyberos_mcp_destructive_confirmations_total`.

**S0-2 minimum tool surface.** AUTH (`whoami`, `list_members`, `invite_member`, `revoke_session`) ships in S0-2. The four-tool minimum exercises read-only, destructive, and confirmation paths. Subsequent sprints add BRAIN, GENIE, CHAT, OBS tools. The gateway does *not* auto-generate tools from GraphQL schemas; every tool is hand-authored in the module's MCP server with care taken to its annotation correctness.

## Alternatives Considered

- **Single monolithic MCP server fronted by the gateway.** Rejected: violates the "module owns end-to-end" principle (PRD §7.2). A change in the BRAIN team would need to coordinate with the AUTH team's MCP code base.
- **Pass-through authorization (forward `Authorization: Bearer ...` to per-module servers).** Rejected: this is the X-Forwarded-Authorization shadow-handoff attack class. The gateway re-mints a short-lived internal JWT.
- **API keys for agent clients.** Rejected: API keys cannot be tied to a specific Member's MFA state, do not rotate, and are exfiltrated trivially. OAuth 2.1 is the floor.
- **Tool-name collisions resolved by precedence (e.g., later registration wins).** Rejected: surprising and dangerous. Conflicts must be a hard error so module owners are forced to coordinate.
- **Auto-generate MCP tools from GraphQL mutations.** Rejected: agent-facing tool design is a different concern from human-API design — a tool name like `cyberos.brain.search` reads like a natural sentence; the equivalent GraphQL field is `brainSearch(query: ..., filters: ...)`. Hand-authored tools also force the destructive/irreversible annotation discipline that auto-generation would skip.

## Success Metrics

- **Primary metric.** S0-2 demo passes the four-tool minimum: an MCP client (Claude.ai or Cursor) connects via PRM discovery, completes the OAuth 2.1 + PKCE flow, calls `cyberos.auth.whoami` and receives the founder's identity, calls `cyberos.auth.invite_member` without `client_confirmed: true` and is rejected with `DESTRUCTIVE_NEEDS_CONFIRMATION`, retries with confirmation and succeeds, all four operations land in `audit.entry` with the correct scope.
- **Guardrail metric.** Persona scope-contract violations = 0 over the lifetime of P0. A persona calling a tool prefix outside its declared scope is sev-1 and the persona version is rolled back.
- **Performance NFR.** Gateway proxy adds ≤ 12 ms p99 to tool-call latency under 200 calls/sec synthetic load (NFR-PERF-MCP-001).

## Scope

**In-scope (S0-2 base; S0-4 adds CUO persona-scope enforcement).**
- `cyberos-mcp-gateway` Deployment, 3 replicas, behind HTTPS Ingress.
- 2025-11-25 spec compliance: HTTP+SSE transport, PRM discovery, OAuth 2.1 + PKCE.
- Tool registry (in-memory per replica + Postgres canonical + NATS reconciliation).
- Tool annotations enforced at proxy: `read_only`, `destructive`, `idempotent`, `requires_confirmation`. Irreversible tools refused at registration.
- Audience-bound tokens; no upstream-token passthrough; internal JWT for module servers.
- Per-tenant authorisation server discovery via `/.well-known/oauth-authorization-server`.
- Persona-scope contract enforcement (S0-4 once GENIE persona ships).
- Audit integration in scope `mcp.{tenant}`.
- Per-tenant rate limits.
- Streaming via SSE pass-through.
- The four AUTH tools (S0-2 minimum).
- OpenTelemetry traces + Prometheus metrics + dashboards in OBS.

**Out-of-scope (deferred).**
- Tool catalogue UI for end-Members (P1) — Members can list tools via Genie panel "what can you do" but a full catalogue browser is P1.
- Per-tool fine-grained per-Member permissions beyond RBAC role (P3).
- Public-facing MCP for external integrations (P4 PORTAL surface).
- Tool versioning + deprecation flow beyond a hard `since_version` annotation (P1).

## Dependencies

- FR-INFRA-001 (federation gateway, Postgres, NATS).
- FR-AUTH-001 (OAuth 2.1 + PKCE flow, per-tenant authorisation server).
- FR-AUTH-002 (audit log; `mcp.{tenant}` scope).
- FR-AI-001 (the AI Gateway is the consumer most callers route through; persona-version stamping is read by the MCP Gateway when binding personas).
- The per-module MCP servers begin landing in S0-2 (AUTH only) with subsequent sprints adding BRAIN (S0-3), GENIE/CUO + CHAT (S0-4), PROJ + OBS (S0-5).
- 2025-11-25 spec implementation libraries: an MCP SDK in TypeScript and Rust (we maintain a small fork of the Anthropic-published MCP SDK to add the audit-and-redaction middleware). SDK choice locked in DEC-031.
- Compliance: EU AI Act Article 14 human oversight (the destructive-tool human-confirmation rule is the architectural enforcement).
- Locked decisions referenced: DEC-007 (MCP 2025-11-25), DEC-008 (federation router not monolith), DEC-009..DEC-012 (OAuth 2.1, PKCE, per-tenant auth server, audience binding), DEC-031 (MCP SDK).

## AI Risk Assessment

The gateway is the agent surface; while it does not itself run inference, it exposes AI-driven workflows to natural persons through tool calls. EU AI Act risk class: `limited`. The three required subsections follow.

### Data Sources

The gateway holds no training data. The tool registry is metadata-only. Tool calls forward arguments to per-module MCP servers; arguments may include personal data (a Member's name, a CRM contact) but the redactor in the audit-log path masks them. The 60-second internal JWT contains only opaque IDs (subject UUID, tenant UUID, role enum) — no personal data.

### Human Oversight

Human-in-the-loop confirmation is the architectural enforcement: every destructive tool requires `client_confirmed: true`. The agent UI must collect confirmation from the human before sending. If an agent client passes `client_confirmed: true` without actually showing the confirmation prompt to the human, the audit log captures the agent client identity and the operator pattern — repeated bypass triggers a manual review of the agent client's authorisation. Irreversible tool classes (compensation, equity, payments, signed contracts) are not registered at all, so no agent can call them regardless of confirmation state. A founder-only kill switch can disable the entire MCP gateway in 30 seconds; the kill switch is itself an audit-logged human action.

### Failure Modes

- **Tool registration collision.** Hard 409 at registration; the conflicting module is paged. The platform refuses to start the second module.
- **Token replay across tenants.** Audience-bound tokens reject replays; an attempted replay writes an audit row with `action: "mcp.token_audience_mismatch"` and an alert is fired.
- **Persona scope-contract violation.** The gateway rejects the call with `code: "PERSONA_SCOPE_VIOLATION"`; the persona version is automatically marked `quarantined: true` if the violation rate exceeds a threshold; the on-call is paged.
- **Prompt-injection through a tool argument.** Caught by the consumer module's input validation and by CaMeL on EMAIL/CHAT (FR-CHAT-001 / FR-EMAIL-001 in batch-03). The gateway is the third floor.
- **Module MCP server outage.** The gateway proxies failures back to the agent with `code: "MCP_MODULE_UNAVAILABLE"`; the agent UI surfaces "tool unavailable, please retry"; circuit breaker opens after 5% error rate.
- **Internal JWT key compromise.** The internal signing key is rotated independently of the AUTH module's user signing keys; a key-rotation script clears the old key in 15 minutes and the gateway re-mints all in-flight internal tokens.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted the spec-compliance section, the tool-annotation enforcement section, and the failure-modes block.
- **Human review:** `@stephen-cheng` reviewed; technical accuracy of the 2025-11-25 spec details to be verified against the published spec at PR-review by the Engineering Lead.
