---
template: feature_request@1
id: FR-APP-004
title: "APP MCP registry and tools panel - federated modules, tools, health, and OAuth clients over the mcp-gateway"
author: "@stephen"
department: engineering
status: superseded
superseded_by: the React console (apps/web) - the static console tiles shipped, then were replaced by the SPA
priority: p3
created_at: "2026-06-29T11:05:00+07:00"
ai_authorship: assisted
feature_type: user_facing
eu_ai_act_risk_class: not_ai
target_release: 2026-Q4
client_visible: false
module: app
new_files:
  - apps/console/src/api/mcp.ts
  - apps/console/src/screens/mcp_registry.ts
  - apps/console/src/screens/mcp_tools.ts
  - apps/console/src/screens/mcp_health.ts
  - apps/console/src/screens/mcp_oauth.ts
  - apps/console/src/render/mcp_health_badge.ts
  - apps/console/tests/mcp_api_client.test.ts
  - apps/console/tests/mcp_health_badge.test.ts
  - apps/console/tests/mcp_tools_render.test.ts
depends_on: [FR-APP-001, FR-AUTH-004, FR-MCP-001, FR-MCP-002, FR-MCP-003, FR-MCP-004, FR-MCP-005, FR-MCP-006]
---

# Feature Request

> Turn Your Will Into Real.

## Summary

The operator console needs one panel that shows what the mcp-gateway is federating and whether it is healthy. This FR adds an MCP registry and tools panel to the same static single-page app as FR-APP-001: the registered modules and their tool catalogs (the gateway's `tools/list`), per-module server health (the `healthy` / `degraded` / `unhealthy` / `deregistered` states from the FR-MCP-002 server-status list), the OAuth clients and the Protected Resource Metadata (FR-MCP-004 / FR-MCP-005), and the federation registry as a whole. It is a panel inside the FR-APP-001 shell, behind the same auth gate, built with the same CDS tokens and components; it defines no shell, no auth, and no design language of its own. It reads only mcp-gateway endpoints that already ship and adds no backend. The first release is read-oriented: an operator can see the federation and its health but cannot trigger a tool from here. Triggering a tool is a named follow-up in Out of scope, and if it is ever added, a destructive tool MUST route through the mcp-gateway confirmation gate (FR-MCP-006), never around it.

## Problem

CyberOS now runs an mcp-gateway that federates the per-module MCP servers (FR-MCP-001), tracks each module's registration and heartbeat health (FR-MCP-002), enforces SEP-986 tool naming (FR-MCP-003), issues OAuth 2.1 PKCE tokens (FR-MCP-004), and publishes Protected Resource Metadata (FR-MCP-005). To see which modules are federated, which tools they expose, and whether a given module's server is healthy or has gone unhealthy, the operator queries the gateway's HTTP surface by hand: `tools/list` for the catalog, `GET /v1/mcp/servers` for the per-server health array, `GET /mcp/healthz` for the aggregate counts, and the `/.well-known/oauth-protected-resource` documents for the OAuth picture. That is several raw JSON reads against several endpoints, with no single CyberSkill-branded screen that ties them together.

The mcp-gateway is the external-agent door: when it federates a stale or unhealthy module, external agents (Claude, Cursor, Codex, Cline) get `skill_unavailable` or `module_unreachable`, and the operator is the person who needs to notice. FR-APP-001 already built the console shell, the auth gate, and the first two panels (obs compliance and ai-gateway health) as a static SPA over shipped APIs. The MCP picture is the obvious next panel, and every datum it needs is already served: the gateway exposes the federated catalog, the per-server health states, and the OAuth metadata over its existing HTTP surface. What is missing is the presentation layer. Building it as another panel in the same SPA reuses the shell, the auth gate, the CDS look, and the Caddy deployment path, and adds no server code.

## Proposed Solution

A new panel set inside the FR-APP-001 console, under `apps/console/src/`, built with CDS tokens and components, served by the same static deployment. User-visible behaviour: the operator signs in through the existing auth flow, opens the MCP panel, and sees four views over the mcp-gateway. The registry view lists the federated modules with each module's server health badge. The tools view lists a selected module's tool catalog from `tools/list`, showing each tool's name (SEP-986 form), description, and its spec annotations (`readOnlyHint` / `destructiveHint` / `idempotentHint` / `openWorldHint`). The health view shows the per-server health array (`healthy` / `degraded` / `unhealthy` / `deregistered`) plus the aggregate counts from `/mcp/healthz`. The OAuth view shows the registered OAuth clients and the Protected Resource Metadata. Every view is a read over a gateway endpoint that already exists; the panel renders, it does not compute and it does not mutate. The look is CDS, so the panel is recognisably a CyberSkill operator surface, consistent with the rest of the console.

### Section 1 - normative requirements (BCP-14)

1. The panel MUST be a screen set inside the FR-APP-001 single-page app under `apps/console/src/`, reusing the FR-APP-001 shell, navigation, and auth gate. It MUST NOT define its own application shell, its own sign-in, or its own design language; it is an addition to the existing console, not a second app.

2. The panel MUST consume only mcp-gateway endpoints that already ship: the federated catalog via `tools/list` (FR-MCP-001), the per-server status list `GET /v1/mcp/servers` and the aggregate `GET /mcp/healthz` (FR-MCP-002 / FR-MCP-001), and the OAuth Protected Resource Metadata at `/.well-known/oauth-protected-resource` and its per-module documents (FR-MCP-005). It MUST NOT introduce a new backend endpoint or server-side component; a view that appears to need a new endpoint is a signal to extend the owning mcp-gateway FR, not to add a backend inside `app`.

3. The panel MUST use CDS design tokens and components for layout, colour, type, and controls, the same Umber and Ochre palette and component set FR-APP-001 establishes. It MUST NOT introduce ad-hoc styling or a second design system.

4. The panel MUST sit behind the FR-APP-001 auth gate: it is reachable only within a valid operator session, and it MUST reuse the existing auth flow (FR-AUTH-004 JWT, FR-AUTH-104 OIDC for IdP operators) rather than defining its own credential path.

5. The panel MUST render the per-module server health using the closed set of states the mcp-gateway defines (FR-MCP-002): `healthy`, `degraded`, `unhealthy`, `deregistered`. It MUST NOT invent additional states, collapse the four into a binary, or infer a health value the gateway did not return.

6. The panel MUST render each tool's spec annotations as the gateway reports them in `tools/list` (FR-MCP-001): `readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint`, and the SEP-986 tool name (FR-MCP-003). A tool the gateway marks `destructiveHint: true` MUST be shown as destructive in the panel, not flattened to a generic entry.

7. The first release MUST be read-only over the mcp-gateway. The panel MUST NOT trigger a tool (`tools/call`), register or deregister a module, mint or revoke an OAuth client, or mutate gateway state in any way. Tool-triggering and other mutations are named follow-ups in Out of scope.

8. If a later release ever surfaces tool-triggering from this panel, a tool the gateway annotates `destructiveHint: true` (or `openWorldHint: true`) MUST route through the mcp-gateway destructive-tool confirmation gate (the FR-MCP-006 confirm or Elicitation flow). The panel MUST NOT bypass that gate, carry a bypass scope, or call a destructive tool without the gateway's confirmation step. This holds even though tool-triggering is out of scope for the first release; it is the standing contract for any future mutation screen.

9. Every outbound call from the panel MUST carry the operator session token the auth shell holds, and MUST target the same-origin mcp-gateway that Caddy fronts (the PRM `/.well-known/oauth-protected-resource` read, which is unauthenticated by RFC 9728, may omit the token, but it MUST still go to the same-origin gateway). The panel MUST NOT call third-party services or send operator data anywhere other than the configured mcp-gateway upstream.

10. On an expired or rejected session (a 401 from the gateway) the panel MUST return the operator to sign-in through the FR-APP-001 gate rather than rendering a partial or stale view with a dead token.

11. The panel MUST fail visibly when the gateway is unreachable or returns a non-2xx: the affected view shows a clear error state, not an empty list that reads as "nothing federated" or "all healthy". It MUST NOT fabricate a module, a tool, a health state, or an OAuth client for a call that did not succeed.

12. The API-client and view-render functions MUST be pure where they can be and unit-tested without a live gateway, against fixture responses for the `tools/list`, server-status, `/mcp/healthz`, and PRM shapes. A live render against a running gateway is an owner-run check, not a unit-test dependency.

## Alternatives Considered

Fold the MCP view into the FR-APP-001 ai-gateway-health panel instead of giving it its own panel. Rejected because the two surfaces answer different questions about different services. The ai-gateway-health panel reads the ai-gateway (FR-AI-022) for model serving health and usage; the MCP panel reads the mcp-gateway for federation, tool catalogs, per-module server health, and OAuth. Sharing one panel would force two unrelated service APIs and two unrelated mental models into one screen and blur which service an error belongs to. A separate panel in the same shell keeps each panel mapped to one service while reusing the console.

Build a standalone MCP admin app outside the console with its own shell and auth. Rejected because it duplicates exactly what FR-APP-001 already provides. The console already has the CDS shell, the auth gate, and the Caddy deployment path; a second app would re-implement sign-in and re-host the design system for one panel's worth of content, and split the operator surface across two front-ends. The founder's decision is one unified operator console with one panel per engine module, and this FR honours that by extending the existing SPA.

Let the panel trigger tools directly and add a small backend in `app` to broker the `tools/call` and the confirmation handshake. Rejected on two counts. First, tool-triggering is out of scope for the first release, which is read-only. Second, even when triggering is added, the mcp-gateway already owns `tools/call`, the SEP-986 routing, and the FR-MCP-006 destructive-tool confirmation gate; brokering any of that through a new `app` backend would duplicate gateway logic and create a second path around the confirmation gate, which clause 8 forbids. A future trigger screen calls the gateway directly through its confirmation flow, with no new backend.

## Success Metrics

Primary metric - operator MCP-state checks done through the panel.
- Definition: fraction of the operator's MCP federation and health checks in a week that are done through the console panel rather than by hitting the gateway's `tools/list`, `GET /v1/mcp/servers`, `/mcp/healthz`, or PRM endpoints by hand.
- Baseline: 0%. No panel exists; today these checks go to the raw gateway endpoints.
- Target: at least 50% of those checks done through the panel within six weeks of the panel's first release.
- Measurement method: the mcp-gateway request log carries a client-id tag; count requests tagged with the console panel over total operator requests to the federation, server-status, healthz, and PRM endpoints.
- Source: the mcp-gateway request log and the OTel `mcp_gateway_request_total` counter (FR-MCP-001).

Guardrail metric - new backend endpoints introduced by the panel.
- Definition: number of new server-side endpoints or backend components the panel requires in order to function.
- Baseline: 0. The panel is specified as a pure front-end over shipped mcp-gateway APIs.
- Target: zero. Any view that appears to need a new endpoint is a signal to extend the owning mcp-gateway FR, not to add a backend inside `app`.
- Measurement method: review of `apps/console/src/api/mcp.ts` against the existing mcp-gateway route list (`tools/list`, `/v1/mcp/servers`, `/mcp/healthz`, `/.well-known/oauth-protected-resource`); any call to a route that does not already exist fails the check.
- Source: the existing mcp-gateway route definitions plus code review of the panel's API layer.

## Scope

In scope: the MCP panel's four views (registry, tools, per-module health, OAuth clients and PRM) built with CDS tokens and components inside the FR-APP-001 shell and auth gate; the API client for the mcp-gateway read endpoints; the health-badge render that maps the four FR-MCP-002 states to CDS styling; and unit tests for the API client and the render functions against fixtures.

### Out of scope

- Any new backend API or server-side component. The panel is a front-end only over shipped mcp-gateway endpoints; new data needs go to the owning mcp-gateway FR.
- Triggering a tool from the panel (`tools/call`). The first release is read-only; a trigger screen is a named follow-up, and per clause 8 any destructive tool it surfaces routes through the FR-MCP-006 confirmation gate.
- Registering, deregistering, or editing a module, and minting, editing, or revoking an OAuth client. Those are gateway mutations owned by the mcp-gateway FRs; a panel mutation screen, if ever wanted, is a later FR.
- Editing the per-tenant gating policy (FR-MCP-006) or the gating-decision log. The panel reads federation and health state, not the gating policy admin surface.
- The shell, the auth gate, and the CDS token definitions themselves. Those belong to FR-APP-001; this FR consumes them and does not redefine them.

## Dependencies

- FR-APP-001 APP CDS web console - the shell, navigation, auth gate, CDS tokens, and Caddy deployment path this panel extends; the panel is a screen set inside that SPA, not a new app.
- FR-MCP-001 mcp-gateway 2025-11-25 spec compliance - the `tools/list` federated catalog the tools view renders, the tool annotations it shows, and the `GET /mcp/healthz` aggregate counts the health view reads.
- FR-MCP-002 per-module server registration and heartbeat lifecycle - the `GET /v1/mcp/servers` status list and the closed `healthy` / `degraded` / `unhealthy` / `deregistered` health states the registry and health views render.
- FR-MCP-003 SEP-986 naming validator - the `cyberos.{module}.{verb}_{noun}` tool-name form the tools view displays as the gateway reports it.
- FR-MCP-004 OAuth 2.1 PKCE - the OAuth clients the OAuth view surfaces and the audience-bound token model behind the gateway's auth.
- FR-MCP-005 Protected Resource Metadata - the `/.well-known/oauth-protected-resource` and per-module documents the OAuth view reads.
- FR-MCP-006 tool-annotation gating - referenced for the standing destructive-tool contract in clause 8: any future tool-trigger screen routes destructive tools through this confirmation gate.
- FR-AUTH-004 JWT and JWKS - the operator session token the auth shell holds and the panel presents on every authenticated gateway call.
- Cross-cutting: the existing Caddy front that terminates the other CyberOS surfaces and the mcp-gateway; the panel ships as part of the FR-APP-001 static bundle behind the same front, not a new ingress.

## AI Authorship Disclosure

- Tools used: Claude (Cowork), authoring this FR from the founder's unified-admin-console decision (one console, one panel per engine module) and the existing mcp-gateway FRs (FR-MCP-001 through FR-MCP-006) plus FR-APP-001.
- Scope: full draft of this specification, including the normative clauses, the alternatives, the metrics, and the scope boundaries. No console code is written by this FR; the panel is built in a later session.
- Human review: Stephen reviews and approves before status moves past draft. The "panel in the same SPA, no new backend" boundary and the destructive-tool-routes-through-FR-MCP-006 rule are operator-mandated, and the paired audit (FR-APP-004.audit.md) validates the format before merge.
