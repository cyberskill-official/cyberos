---
template: task@1
id: TASK-APP-003
title: "APP AI ops panel - cost, budget, policy, and model health over the ai-gateway"
author: "@stephen"
department: engineering
status: superseded
superseded_by: the React console (apps/web) - the static console tiles shipped, then were replaced by the SPA
priority: p3
created_at: "2026-06-29T11:00:00+07:00"
ai_authorship: assisted
feature_type: user_facing
eu_ai_act_risk_class: not_ai
target_release: 2026-Q4
client_visible: false
module: app
new_files:
  - apps/console/src/screens/ai_ops.ts
  - apps/console/src/screens/ai_cost.ts
  - apps/console/src/screens/ai_policy.ts
  - apps/console/src/api/ai_cost.ts
  - apps/console/src/api/ai_models.ts
  - apps/console/src/api/ai_policy.ts
  - apps/console/src/render/budget_bar.ts
  - apps/console/tests/ai_cost_client.test.ts
  - apps/console/tests/ai_models_client.test.ts
  - apps/console/tests/ai_policy_render.test.ts
depends_on: [TASK-APP-001, TASK-AUTH-004, TASK-AI-022, TASK-AI-001, TASK-AI-002, TASK-AI-006, TASK-AI-009, TASK-AI-017, TASK-AI-005, TASK-AI-105]
---

# Task

> Turn Your Will Into Real.

## Summary

The CyberOS operator console (TASK-APP-001) ships a basic ai-gateway health screen; this FR adds the AI ops panel that goes deeper. It surfaces the operator state the gateway already exposes over HTTP: per-tenant cost-ledger spend against the monthly cap and the warn threshold, the resolved model-alias map with provider and circuit-breaker health, response-cache hit and skip stats, and a read view of the tenant policy (alias map, caps, residency, ZDR). It is one more panel inside the same static single-page app under `apps/console/`, built with the same CDS tokens and components, sitting behind the same auth-gated shell. It adds no backend: every figure is a read over an ai-gateway endpoint that already ships. The first release is read-only; editing the policy from the console is a named follow-up. This is the operator's AI cost-and-health view, distinct from anything tenant-facing in the `portal` module.

## Problem

TASK-APP-001 gives the operator a sign-in and a gateway-health screen, but that screen answers only "is the gateway up". It does not answer the questions the operator actually runs the gateway by: how close is each tenant to its monthly cap, which tenants have crossed the warn line, which alias resolves to which provider model today, which circuit breakers are open, how often the response cache is paying off, and what the tenant policy says about caps, residency, and zero-data-retention. Today those answers live in the ai-gateway HTTP responses and in the policy YAML, read by hand: the operator hits the cost-ledger and model endpoints with a token and reads raw JSON, or opens the policy file. There is no CyberSkill-branded screen that signs in once and lays this out.

The endpoints to back the panel already exist and already ship. The gateway serves its HTTP surface with trace and span emission (TASK-AI-022); the cost ledger runs a pre-call check and a post-call reconcile that hold and settle per-tenant spend (TASK-AI-001, TASK-AI-002); alias resolution maps a logical alias to a provider model with per-tenant override (TASK-AI-006); the circuit breaker tracks health per provider and model (TASK-AI-009); the per-tenant response cache reports hit rate (TASK-AI-017); the tenant-policy loader holds the cap, warn, override, and residency the read view renders (TASK-AI-005); and the provider set reflects whichever local or cloud providers the gateway routes to (TASK-AI-105). The gap is the presentation layer, not the data. Building it as one more screen in the existing static SPA keeps the no-new-backend rule of TASK-APP-001 and reuses its shell, its auth gate, and its deployment path.

## Proposed Solution

A new AI ops panel inside the existing `apps/console/` SPA, reachable from the console shell once the operator is signed in, built with the same CDS tokens and components and reusing the same API-client pattern TASK-APP-001 established. User-visible behaviour: the operator opens the panel and sees four read views. A cost view lists each tenant's current spend against its monthly cap with the warn threshold marked, so a tenant near or over the line is obvious at a glance. A model-health view shows the resolved alias map (alias to provider model), the provider each alias routes to, and the circuit-breaker state per provider and model. A cache view shows the response-cache hit and skip counts. A policy view renders the tenant policy as read-only: the alias map, the caps, the residency setting, and the ZDR flag. Every figure is a read over an ai-gateway endpoint that already ships; the panel renders, it does not compute spend or decide health. The look is CDS, so the panel is recognisably the same CyberSkill operator surface as the rest of the console.

### Section 1 - normative requirements (BCP-14)

1. The panel MUST be a screen inside the same static single-page app under `apps/console/` defined by TASK-APP-001. It MUST reuse that app's shell, navigation, CDS tokens and components, and API-client pattern. It MUST NOT define its own shell, its own design language, or a second SPA.

2. The panel MUST consume only ai-gateway endpoints that already ship: the gateway HTTP surface (TASK-AI-022), the cost-ledger state (TASK-AI-001 / TASK-AI-002), the resolved alias map (TASK-AI-006), the circuit-breaker health (TASK-AI-009), the response-cache stats (TASK-AI-017), and the tenant policy the loader holds (TASK-AI-005). It MUST NOT introduce a new backend endpoint or server-side component. A view that appears to need an endpoint the gateway does not already serve is a signal to extend the ai-gateway's owning FR, not to add a backend inside `app`.

3. The panel MUST be reachable only behind the existing auth-gated shell (TASK-APP-001 reusing TASK-AUTH-004 JWT): an unauthenticated visitor never reaches the panel. The panel MUST NOT define its own credential store or its own sign-in.

4. The cost view MUST show, per tenant, the current cost-ledger spend against the monthly cap, with the warn threshold marked, so a tenant near or past its cap or warn line is visible without arithmetic by the reader. The figures MUST come from the cost-ledger endpoints (TASK-AI-001 pre-call holds and TASK-AI-002 post-call reconcile); the panel reads settled and held spend, it does not recompute it.

5. The model-health view MUST show the resolved alias map (logical alias to provider model, TASK-AI-006), the provider each alias routes to including whichever local or cloud providers the gateway uses (TASK-AI-105), and the circuit-breaker state per provider and model (TASK-AI-009: closed, open, or half-open).

6. The cache view MUST show the per-tenant response-cache hit and skip stats the gateway reports (TASK-AI-017). It MUST present them as the gateway returns them and MUST NOT derive a hit rate the gateway did not report.

7. The policy view MUST render the tenant policy as read-only in the first release: the model-alias map, the caps, the warn threshold, the residency setting, and the ZDR flag the tenant-policy loader holds (TASK-AI-005, residency and ZDR per TASK-AI-015 / TASK-AI-016). The panel MUST NOT offer a control that edits or writes the policy in this release; policy editing is named in Out of scope.

8. Every outbound call the panel makes MUST carry the session token the auth shell holds and MUST target the same-origin gateway upstream that Caddy already fronts. The panel MUST NOT call third-party services or send operator or tenant data anywhere other than the configured CyberOS gateway upstream.

9. On an expired or rejected session (a 401 from the gateway) the panel MUST hand control back to the shell's sign-in path rather than rendering a partial or stale view with a dead token. On any other non-2xx or an unreachable gateway, the affected view MUST show a clear error state, not an empty view that reads as "no spend" or "all breakers closed". The panel MUST NOT fabricate cost, health, cache, or policy data for a call that did not succeed.

10. The panel's API-client and view-render functions MUST be pure where they can be and unit-tested without a live gateway, against fixture responses for the cost-ledger, alias-map, circuit-breaker, cache-stats, and policy shapes. A live render against a running gateway is an owner-run check, not a unit-test dependency.

## Alternatives Considered

Read the cost, model-health, cache, and policy figures the way they are read today: the operator hits the ai-gateway endpoints with a token and reads the JSON, or opens the policy YAML. Rejected because that is the exact gap this FR closes. Raw JSON answers a one-off question but is not a branded screen that lays out cap-versus-spend, the alias map, breaker state, and the policy together for an at-a-glance read, and it leaves the warn-line arithmetic to the operator's eye. The panel renders the same shipped responses into a CDS view so the state is read, not reconstructed each time.

Build the AI ops panel as its own separate small app rather than a screen inside the TASK-APP-001 console. Rejected because it fragments the operator surface the console exists to unify. A second app means a second shell, a second auth integration, and a second deployment unit, all for state that belongs next to the gateway-health screen TASK-APP-001 already ships. The founder's decision is one unified operator console with one panel per engine module; a standalone AI app would break that and duplicate the shell and the auth gate for no gain.

Add a thin backend in `app` that aggregates the cost-ledger, alias, breaker, cache, and policy reads into one console-shaped response. Rejected because it adds a server where none is needed and breaks the TASK-APP-001 no-new-backend rule. The gateway already serves each of these over HTTP; the panel can call them directly and render client-side. A console-side aggregator would duplicate reads the gateway already answers, add a service to operate, and create a second place where tenant cost and policy data lives in transit. If the shape of a gateway response is awkward for a screen, the fix is to extend the ai-gateway's FR, not to grow a backend inside `app`.

## Success Metrics

Primary metric - AI ops checks done through the panel.
- Definition: fraction of the operator's per-tenant cost, model-health, cache, and policy checks in a week that are done through the AI ops panel rather than by hitting the raw ai-gateway endpoints or reading the policy YAML.
- Baseline: 0%. No panel exists; today these checks go to raw JSON or the policy file.
- Target: at least 50% of those checks done through the panel within six weeks of first release.
- Measurement method: the gateway request log carries a client-id tag (TASK-AI-022); count requests tagged with the console against the panel's read endpoints over total operator requests to those same cost-ledger, model, cache, and policy endpoints.
- Source: the ai-gateway request and trace log (TASK-AI-022).

Guardrail metric - new backend endpoints introduced by the panel.
- Definition: number of new server-side endpoints or backend components the panel requires in order to function.
- Baseline: 0. The panel is specified as a pure front-end over shipped ai-gateway endpoints.
- Target: zero. Any view that appears to need an endpoint the gateway does not already serve is a signal to extend the ai-gateway's FR, not to add a backend inside `app`.
- Measurement method: review of the panel's `apps/console/src/api/` clients against the existing ai-gateway route list; any call to a route that does not already exist fails the check.
- Source: the existing ai-gateway route definitions (TASK-AI-022 and the cost, model, cache, and policy FRs it serves) plus code review of the panel's API layer.

## Scope

In scope: the AI ops panel as a screen set inside the existing `apps/console/` SPA, built with CDS tokens and components and reachable behind the TASK-APP-001 auth-gated shell; the cost view (per-tenant spend against cap with the warn line marked); the model-health view (resolved alias map, provider per alias, circuit-breaker state); the cache view (hit and skip stats); the read-only policy view (alias map, caps, residency, ZDR); the API clients for the gateway's cost-ledger, model, cache, and policy reads; and unit tests for those clients and the render functions against fixtures.

### Out of scope

- Editing the tenant policy from the console. The first release renders the policy read-only; a policy-edit screen that writes back through the owning service is a named follow-up FR, not part of this one.
- Any new backend API or aggregator in `app`. The panel is a front-end only over shipped ai-gateway endpoints; a new data need goes to the ai-gateway's FR.
- Setting or moving a tenant's cap or warn threshold, or any cost-ledger mutation. The cost view is read-only; changing a budget is an owning-service action, surfaced later if at all.
- Resetting or forcing a circuit breaker, flushing the response cache, or any other gateway control action. The model-health and cache views are read-only in this release.
- Deep metric dashboards and time-series charts. Trend panels stay in Grafana behind the obs-proxy that TASK-APP-001 links to; this panel shows current operator state, not historical series.

## Dependencies

- TASK-APP-001 APP CDS web console - the SPA shell, navigation, CDS token set, auth gate, and API-client pattern this panel is a screen inside. This FR adds a panel; it does not re-create any of that.
- TASK-AUTH-004 JWT and JWKS - the session token the shell holds and the panel presents on every gateway call.
- TASK-AI-022 ai-gateway HTTP serving surface - the gateway's HTTP surface (trace and span emission) the panel reads, and the request log that backs the primary metric.
- TASK-AI-001 cost-ledger pre-call check - the per-tenant held spend the cost view reads.
- TASK-AI-002 cost-ledger post-call reconcile - the settled per-tenant spend the cost view reads against the cap and warn line.
- TASK-AI-006 model-alias resolution - the resolved alias map (alias to provider model, with per-tenant override) the model-health view renders.
- TASK-AI-009 circuit breaker - the per-provider, per-model breaker state (closed, open, half-open) the model-health view shows.
- TASK-AI-017 per-tenant response cache - the hit and skip stats the cache view shows.
- TASK-AI-005 tenant-policy YAML loader - the cap, warn, override, and residency the read-only policy view renders (with ZDR per TASK-AI-015 and residency pinning per TASK-AI-016).
- TASK-AI-105 local + external model providers - the provider set the model-health view reflects, whichever local or cloud providers the gateway routes to.
- Cross-cutting: the existing Caddy front and the TASK-APP-001 static-serving snippet; this panel ships as more static files under the same path, not a new ingress.

## AI Authorship Disclosure

- Tools used: Claude (Cowork), authoring this FR from the founder's unified-admin-console decision (one console, one panel per engine module) and the existing ai-gateway, auth, and TASK-APP-001 console FRs.
- Scope: full draft of this specification, including the normative clauses, the alternatives, the metrics, and the scope boundaries. No console code is written by this FR; the panel is built in a later session.
- Human review: Stephen reviews and approves before status moves past draft. The "one panel in the TASK-APP-001 SPA, no new backend, read-only first release" boundary is operator-mandated, and the paired audit (TASK-APP-003.audit.md) validates the format before merge.
