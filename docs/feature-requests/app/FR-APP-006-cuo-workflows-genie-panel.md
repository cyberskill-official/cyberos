---
template: feature_request@1
id: FR-APP-006
title: "APP CUO workflows and GENIE assistant panel - run and monitor workflows and the GENIE assistant over the gateway"
author: "@stephen"
department: engineering
status: draft
priority: p3
created_at: "2026-06-29T11:15:00+07:00"
ai_authorship: assisted
feature_type: user_facing
eu_ai_act_risk_class: not_ai
target_release: 2026-Q4
client_visible: false
module: app
new_files:
  - apps/console/src/screens/workflows.ts
  - apps/console/src/screens/genie.ts
  - apps/console/src/api/mcp.ts
  - apps/console/src/api/genie.ts
  - apps/console/src/components/workflow_run_form.ts
  - apps/console/src/components/confirm_prompt.ts
  - apps/console/tests/mcp_client.test.ts
  - apps/console/tests/genie_client.test.ts
  - apps/console/tests/workflows_render.test.ts
  - apps/console/tests/confirm_prompt.test.ts
depends_on: [FR-APP-001, FR-AUTH-004, FR-MCP-001, FR-MCP-006, FR-MCP-007, FR-MCP-008, FR-CUO-101, FR-AI-022]
---

# Feature Request

> Turn Your Will Into Real.

## Summary

The CyberOS operator console (the `app` module, FR-APP-001) gets one more panel: the CUO workflows and GENIE assistant panel. It is where the operator lists the CUO workflows and skills that the mcp-gateway exposes, triggers a workflow run with a JSON arguments payload, watches that run's status, and talks to the GENIE assistant in a chat scoped to the session. It is a panel inside the same static single-page app under `apps/console/`, reusing the FR-APP-001 shell, auth gate, and CDS design language; it defines none of its own. It adds no backend. The workflow list comes from the mcp-gateway `tools/list` surface (FR-MCP-001), a run is started with `tools/call` and tracked through the tasks primitive (FR-MCP-007), and the GENIE chat routes to the same ai-gateway chat path the console already uses (FR-AI-022). When a workflow step is destructive, the panel surfaces the mcp-gateway confirmation gate (FR-MCP-006 annotation gating, FR-MCP-008 elicitation) as a confirm prompt and honours the verdict; it never bypasses that gate. The first release is read-and-trigger oriented. Authoring or editing a workflow stays in CUO and is named in Out of scope. This is the operator surface; the tenant-facing `portal` is separate.

## Problem

The operator can see CyberOS compliance state and gateway health in the console (FR-APP-001), but the console says nothing about the part the operator actually runs day to day: the CUO workflows and the GENIE assistant. To list the available workflows and skills, trigger one, and watch it move, the operator drives the mcp-gateway by hand (an HTTP client posting `tools/list`, then `tools/call`, then polling the task) or opens the desktop trigger app (FR-APP-002), which is a separate native binary. To talk to GENIE, the operator hits the ai-gateway chat endpoint directly. None of this lives in the one CyberSkill-branded screen the operator already signs in to.

The control plane to back this already ships. The mcp-gateway exposes `tools/list`, `tools/call`, and the tasks primitive for status (FR-MCP-001, FR-MCP-007), with destructive steps gated behind confirmation or elicitation (FR-MCP-006, FR-MCP-008). CUO orchestrates the workflow behind those calls (FR-CUO-101). The ai-gateway serves the chat surface GENIE speaks over (FR-AI-022), and the console already calls that path. Auth already issues and validates the session (FR-AUTH-004). What is missing is the presentation layer: a panel in the existing console that reads the live tool catalogue, lets the operator trigger a run and follow its task, surfaces the confirmation gate where it fires, and renders the GENIE chat. Building it as another screen in the FR-APP-001 SPA adds no server code and reuses the deployment path the rest of the console already uses.

## Proposed Solution

A new screen set inside the FR-APP-001 console under `apps/console/src/screens/`, built with the CDS tokens and components the shell already provides, served as the same static files behind the same Caddy front. User-visible behaviour: the operator opens the console, is already signed in through the FR-APP-001 auth gate, and selects the workflows panel. The panel lists the CUO workflows and skills the mcp-gateway returns from `tools/list`. The operator picks one, fills a JSON arguments payload in a small form, and triggers it; the panel issues a `tools/call`, shows the returned task id, and polls the tasks primitive for status until the run settles. If a step is destructive, the mcp-gateway returns a confirmation or elicitation request; the panel renders it as a confirm prompt, sends the operator's verdict back, and only proceeds if the gate clears. A second tab is the GENIE assistant chat: a conversational surface that posts to the ai-gateway chat path the console already uses, scoped by the session, with each turn rendered as it returns. Every panel is a read or a call over an endpoint that already exists; the console renders and routes, it does not compute and it does not orchestrate.

### Section 1 - normative requirements (BCP-14)

1. This panel MUST be a screen set inside the FR-APP-001 console under `apps/console/`, reusing that app's shell, auth gate, and CDS design language. It MUST NOT define its own shell, its own auth, or a second design language; it is one panel added to the existing single-page app.

2. The panel MUST consume only service APIs that already ship: the mcp-gateway `tools/list`, `tools/call`, tasks, and confirmation/elicitation surfaces (FR-MCP-001, FR-MCP-007, FR-MCP-006, FR-MCP-008) and the ai-gateway chat surface (FR-AI-022). It MUST NOT introduce a new backend endpoint or server-side component. A screen that appears to need a new endpoint is a signal to extend the owning service's FR, not to add a backend in `app`; this is the guardrail metric.

3. The workflow and skill list MUST come from the mcp-gateway `tools/list` surface (FR-MCP-001) at runtime, filtered to what the signed-in subject may invoke. The panel MUST NOT show a hard-coded catalogue that can drift from what the gateway actually exposes.

4. Triggering a workflow MUST issue a `tools/call` to the mcp-gateway (FR-MCP-001) with the operator's JSON arguments payload, and MUST surface the returned run or task id. CUO (FR-CUO-101) orchestrates the run behind that call; the panel MUST NOT re-implement any workflow or skill logic.

5. Run status MUST be read from the mcp-gateway tasks primitive (FR-MCP-007): the panel polls the task and shows its real state through to completion, failure, or cancellation. It MUST NOT infer or fabricate a status that the tasks surface did not report.

6. When a workflow step is destructive, the panel MUST surface the mcp-gateway confirmation/elicitation gate (FR-MCP-006 annotation gating, FR-MCP-008 elicitation) as an explicit confirm prompt, and MUST send the operator's verdict back to the gateway and honour it. The panel MUST NOT bypass, auto-confirm, or suppress that gate; a destructive step proceeds only when the gate clears.

7. The GENIE assistant chat MUST route to the ai-gateway chat surface (FR-AI-022), the same path the console already uses, scoped by the session. The panel MUST NOT call a model provider directly or open a second chat backend; the assistant's behaviour and any model risk are owned by the ai-gateway and CUO FRs, not by this presentation panel.

8. Every outbound call MUST carry the session token the FR-APP-001 auth shell holds, and MUST target the same-origin mcp-gateway and ai-gateway upstreams that Caddy already fronts. The panel MUST NOT call third-party services or send operator data anywhere other than the configured CyberOS upstreams.

9. On an expired or rejected session (a 401 from any upstream) the panel MUST hand back to the FR-APP-001 sign-in path rather than rendering a partial or stale screen with a dead token. On any other non-2xx, or an unreachable upstream, the affected panel MUST fail visibly with a clear error state and MUST NOT fabricate a tool list, a run status, or a chat turn.

10. The API-client and view-render functions (the mcp-gateway client, the GENIE chat client, the confirm-prompt handling, and the list and status renderers) MUST be pure where they can be and unit-tested without a live backend, against fixture responses for the `tools/list`, `tools/call`, tasks, elicitation, and chat shapes. A live run against running services is an owner-run check, not a unit-test dependency.

## Alternatives Considered

Treat the desktop workflow trigger (FR-APP-002) as the only run-and-monitor surface and add nothing to the console. Rejected because the two surfaces are not duplicates: FR-APP-002 is a native Tauri binary the operator installs and updates per machine, and this panel is the web-console equivalent that needs no install and lives in the screen the operator already signs in to. The desktop app suits an always-on launcher on the operator's own machine; the console panel suits any browser behind the Caddy front. Both drive the same mcp-gateway and ai-gateway surfaces, so neither re-implements the other; they are two front-ends over one control plane, and this FR is the web one.

Build a standalone GENIE chat app, separate from the operator console. Rejected because it would duplicate the shell, the auth gate, and the CDS setup that FR-APP-001 already provides, and split the operator's surfaces into two apps to sign in to. The GENIE chat is one tab of the same panel, behind the same session, routing to the same ai-gateway chat path the console already calls; a separate app buys nothing and doubles the maintenance.

Have the panel call CUO or a model provider directly to start runs and serve the assistant, instead of going through the mcp-gateway and ai-gateway. Rejected because it would bypass the surfaces that own the contracts. The mcp-gateway owns `tools/list`, `tools/call`, the tasks primitive, and the destructive-step confirmation gate (FR-MCP-006, FR-MCP-008); the ai-gateway owns the chat surface and its model routing (FR-AI-022). Calling CUO or a provider directly from a front-end would route around the confirmation gate and the gateway's auth and audit, which is exactly what clause 6 and clause 8 forbid. The panel renders and routes; the gateways orchestrate and enforce.

## Success Metrics

Primary metric - operator workflow runs and GENIE turns through the console panel.
- Definition: fraction of the operator's workflow triggers and GENIE chat turns in a week that go through the console panel rather than a hand-driven mcp-gateway call or a direct ai-gateway chat call.
- Baseline: 0%. No panel exists; today these go through a raw HTTP client, the desktop trigger, or a direct chat call.
- Target: at least 50% of those triggers and turns done through the console panel within six weeks of the panel's first release.
- Measurement method: the mcp-gateway and ai-gateway request logs carry a client-id tag; count requests tagged with the console panel over total operator requests to the `tools/call`, tasks, and chat endpoints.
- Source: the mcp-gateway request log (FR-MCP-001 / FR-MCP-007) and the ai-gateway request log (FR-AI-022).

Guardrail metric - new backend endpoints introduced by the panel.
- Definition: number of new server-side endpoints or backend components the panel requires in order to function.
- Baseline: 0. The panel is specified as a pure front-end over the shipped mcp-gateway and ai-gateway surfaces.
- Target: zero. Any screen that appears to need a new endpoint is a signal to extend the owning service's FR (mcp-gateway or ai-gateway), not to add a backend inside `app`.
- Measurement method: review of the panel's `apps/console/src/api/` clients against the existing mcp-gateway and ai-gateway route lists; any call to a route that does not already exist fails the check.
- Source: the existing mcp-gateway and ai-gateway route definitions plus code review of the panel's API layer.

## Scope

In scope: the workflows panel (the `tools/list`-driven catalogue, the JSON-arguments run form, the `tools/call` trigger, and the tasks-primitive status view), the destructive-step confirm prompt that surfaces and honours the mcp-gateway confirmation/elicitation gate, the GENIE assistant chat tab routing to the ai-gateway chat surface, the mcp-gateway and GENIE API clients, and unit tests for the clients, the confirm-prompt handling, and the list and status renderers against fixtures. All of it reuses the FR-APP-001 shell, auth gate, and CDS components.

### Out of scope

- Any new backend API or server-side component. The panel is a front-end only; new data needs go to the owning service's FR (mcp-gateway or ai-gateway).
- Authoring or editing a workflow. The first release lists, triggers, and monitors existing workflows; creating, editing, versioning, or deleting a workflow stays in CUO and is a later FR.
- Re-implementing CUO orchestration or the destructive-step gate. The panel surfaces and honours the gate the mcp-gateway returns; it does not decide what is destructive or run the workflow itself.
- A second auth flow, shell, or design language. Sign-in, the app shell, and the CDS look are owned by FR-APP-001; this panel reuses them and adds none of its own.
- The native desktop trigger. The web panel and the Tauri app (FR-APP-002) are separate front-ends over the same control plane; this FR does not change or absorb the desktop app.

## Dependencies

- FR-APP-001 CDS web console - the shell, auth gate, CDS tokens and components, and Caddy static-serving path this panel is added to; it is one more screen in that same app.
- FR-MCP-001 MCP gateway spec compliance - the `tools/list` catalogue the panel renders and the `tools/call` surface the run trigger drives.
- FR-MCP-007 tasks primitive - the run-status surface the panel polls from trigger through to completion.
- FR-MCP-006 tool-annotation gating - the destructive-step gate the panel surfaces as a confirm prompt and must not bypass.
- FR-MCP-008 elicitation - the server-initiated structured prompt the panel renders when a step needs confirmation or missing input.
- FR-CUO-101 LangGraph supervisor - orchestrates the workflow behind the `tools/call`; the panel does not duplicate this.
- FR-AI-022 ai-gateway HTTP serving surface - the chat path the GENIE assistant routes to, the same one the console already uses.
- FR-AUTH-004 JWT and JWKS - the session token the FR-APP-001 auth shell holds and the panel presents on every call.
- Cross-cutting: the existing Caddy front that terminates the other CyberOS surfaces; this panel ships as part of the FR-APP-001 static bundle and adds no new ingress.

## AI Authorship Disclosure

- Tools used: Claude (Cowork), authoring this FR from the founder's unified-admin-console decision (one operator console with one panel per engine module, no separate GUIs) and the existing module FRs (mcp-gateway, CUO, ai-gateway, auth, and FR-APP-001).
- Scope: full draft of this specification, including the normative clauses, the alternatives, the metrics, and the scope boundaries. No console code is written by this FR; the panel is built in a later session.
- Human review: Stephen reviews and approves before status moves past draft. The "one panel in the FR-APP-001 SPA, no new backend, never bypass the destructive-step gate" boundaries are operator-mandated, and the paired audit (FR-APP-006.audit.md) validates the format before merge.
