---
template: feature_request@1
id: FR-APP-001
title: "APP CDS web console - operator console over CyberOS service APIs"
author: "@stephen"
department: engineering
status: superseded
superseded_by: the React console (apps/web) - the static console tiles shipped, then were replaced by the SPA
priority: p3
created_at: "2026-06-22T11:00:00+07:00"
ai_authorship: assisted
feature_type: user_facing
eu_ai_act_risk_class: not_ai
target_release: 2026-Q4
client_visible: false
module: app
new_files:
  - apps/console/package.json
  - apps/console/index.html
  - apps/console/src/main.ts
  - apps/console/src/shell/auth_gate.ts
  - apps/console/src/api/gateway.ts
  - apps/console/src/api/obs.ts
  - apps/console/src/screens/compliance.ts
  - apps/console/src/screens/gateway_health.ts
  - apps/console/src/cds/tokens.css
  - apps/console/Caddyfile.snippet
  - apps/console/tests/auth_gate.test.ts
  - apps/console/tests/api_client.test.ts
depends_on: [FR-AUTH-004, FR-AUTH-104, FR-AI-022, FR-AI-105, FR-OBS-002, FR-OBS-008, FR-PORTAL-006, FR-PORTAL-007]
---

# Feature Request

> Turn Your Will Into Real.

## Summary

CyberOS needs its own operator and admin web console: a front-end for the people who run CyberOS, not for tenants. It is built with the CyberSkill Design System (CDS) - the brand's design tokens and components, Umber and Ochre palette, the "Turn Your Will Into Real" identity - so the console looks like CyberSkill rather than a generic dashboard. It is a static single-page app over the service APIs that already ship: the ai-gateway HTTP surface, the obs compliance-view endpoints, the obs-proxy, auth, and memory. It adds no backend. The first screens surface the obs compliance views and the ai-gateway health and usage, behind an auth-gated shell. It deploys as static files behind the Caddy front that already terminates the other CyberOS surfaces. This is the operator console; tenant-facing branding and the tenant portal stay with the `portal` module.

## Problem

CyberOS has a full service layer and no first-party front-end for the operator. To read the obs compliance views (FR-OBS-008) or check the gateway's health and usage (FR-AI-022), the operator queries the HTTP endpoints by hand or reads Grafana through the obs-proxy (FR-OBS-002). There is no single CyberOS-branded screen that signs in once and shows operator state. Grafana is a general tool wearing its own skin; the raw endpoints return JSON. Neither is the CyberSkill operator surface.

The APIs to back a console already exist and are shipped: auth issues and validates the JWT (FR-AUTH-004), the ai-gateway serves health and usage over HTTP (FR-AI-022, extended by FR-AI-105), obs exposes scoped compliance views (FR-OBS-008) and the tenant-aware proxy (FR-OBS-002), and memory holds the audit chain. What is missing is the presentation layer that ties them into one branded, auth-gated console. The brand system to build it with also exists: CDS already defines the tokens, the palette, and the components used across CyberSkill surfaces. The gap is purely the front-end; building it as a static SPA over the shipped APIs avoids any new server code and reuses the deployment path the rest of CyberOS already uses.

## Proposed Solution

A static single-page app under `apps/console/`, built with CDS tokens and components, served as static files behind the existing Caddy front. User-visible behaviour: the operator opens the console, signs in through the existing auth flow, and lands on an auth-gated shell. The first screens are the obs compliance views and the ai-gateway health and usage. Every panel is a read over an endpoint that already exists; the console renders, it does not compute. The look is CDS, so the console is recognisably a CyberSkill surface and not a default admin theme.

### Section 1 - normative requirements (BCP-14)

1. The console MUST consume only service APIs that already ship: the ai-gateway HTTP surface (FR-AI-022 / FR-AI-105), the obs compliance-view endpoints (FR-OBS-008), the obs-proxy (FR-OBS-002), the auth service (FR-AUTH-004), and memory. It MUST NOT introduce a new backend endpoint or server-side component; it is a front-end over shipped APIs.

2. The console MUST use CDS design tokens and components for layout, colour, type, and controls. It MUST NOT introduce ad-hoc styling or a second design language; the Umber and Ochre palette and the CDS component set are the source of truth for the visual layer.

3. The console MUST be gated by an auth shell: an unauthenticated visitor sees only the sign-in path, and every data screen sits behind a valid session. Sign-in MUST reuse the existing auth flow (FR-AUTH-004 JWT, FR-AUTH-104 OIDC SSO for operators who sign in through an IdP). The console MUST NOT define its own credential store.

4. The console MUST surface, as its first screens, at least the obs compliance views (FR-OBS-008) and the ai-gateway health and usage (FR-AI-022). These two are the minimum first-release content; further screens are additive.

5. The console MUST be a static single-page app: HTML, CSS, and client JavaScript that can be served as files, with no server-side rendering and no application server of its own. State lives client-side and in the APIs it calls.

6. The console MUST be deployable behind the existing Caddy front that already terminates the other CyberOS surfaces. It MUST NOT require a new reverse proxy or a separate ingress; a Caddy snippet (static file serving plus the API upstreams it already proxies) is the deployment unit.

7. Every outbound API call MUST carry the session token the auth shell holds, and MUST target the same-origin gateway and obs upstreams that Caddy fronts. The console MUST NOT call third-party services or send operator data anywhere other than the configured CyberOS upstreams.

8. On an expired or rejected session (a 401 from any upstream) the console MUST return the operator to sign-in rather than rendering a partial or stale screen with a dead token.

9. The console MUST fail visibly when an upstream is unreachable or returns a non-2xx: the affected panel shows a clear error state, not an empty panel that reads as "all clear". It MUST NOT fabricate data for a call that did not succeed.

10. The API-client and view-render functions MUST be pure where they can be and unit-tested without a live backend (fixture responses for the gateway-health and compliance-view shapes). A live render against running services is an owner-run check, not a unit-test dependency.

## Alternatives Considered

Use Grafana (through the obs-proxy, FR-OBS-002) as the operator console and skip a bespoke front-end. Rejected as the whole answer. Grafana is right for metric dashboards and the proxy already scopes it per tenant, but it is a general observability tool with its own identity and navigation; it is not a CyberSkill-branded operator surface, and it cannot cleanly host non-metric screens like the compliance views or gateway usage in the CDS look. The console links out to Grafana for deep metric panels and owns the branded shell itself.

Extend the tenant portal (the `portal` module) to also serve the operator console. Rejected because the audiences and the branding are opposite. The portal is tenant-facing and white-labelled per tenant (FR-PORTAL-002); the operator console is CyberSkill-branded and internal. Folding the operator surface into portal would pull operator screens into a tenant-scoped, per-tenant-branded app and blur the module boundary. Keeping `app` separate from `portal` is the point: `app` is CyberOS's own first-party surface, `portal` is the tenant's.

Build the console as a server-rendered app with its own small backend (for session handling and API aggregation). Rejected because it adds a server where none is needed. The auth service already issues the session, the gateway and obs endpoints already return what the screens need, and Caddy already fronts them. A static SPA over those APIs has a smaller surface, no new service to operate, and reuses the existing deployment path; a new backend would duplicate auth and aggregation that already exist.

## Success Metrics

Primary metric - operator console adoption for the first screens.
- Definition: fraction of the operator's compliance-view and gateway-health checks in a week that are done through the console rather than by hitting the raw endpoints or reading Grafana directly.
- Baseline: 0%. No console exists; today these checks go to raw JSON endpoints or straight to Grafana.
- Target: at least 50% of those checks done through the console within six weeks of first release.
- Measurement method: the obs and gateway request logs carry a client-id tag; count requests tagged with the console over total operator requests to the compliance-view and gateway-health endpoints.
- Source: ai-gateway request log (FR-AI-022) and the obs compliance-view access audit rows (FR-OBS-008).

Guardrail metric - new backend endpoints introduced by the console.
- Definition: number of new server-side endpoints or backend components the console requires in order to function.
- Baseline: 0. The console is specified as a pure front-end over shipped APIs.
- Target: zero. Any screen that appears to need a new endpoint is a signal to extend the owning service's FR, not to add a backend inside `app`.
- Measurement method: review of the console's `apps/console/src/api/` clients against the existing service OpenAPI and route lists; any call to a route that does not already exist fails the check.
- Source: the existing gateway and obs route definitions plus code review of the console's API layer.

## Scope

In scope: the static SPA shell built with CDS tokens and components, the auth-gated front (sign-in reusing the existing auth flow, OIDC for IdP operators), the API clients for the gateway and obs upstreams, the first two screen sets (obs compliance views and ai-gateway health and usage), the Caddy static-serving snippet, and unit tests for the API clients and the auth gate against fixtures.

### Out of scope

- Any new backend API or server-side component. The console is a front-end only; new data needs go to the owning service's FR.
- Tenant-facing branding or any white-label theming. That is the `portal` module's job (FR-PORTAL-002); this console is CyberSkill-branded and operator-only.
- A native mobile app. The console is a responsive web SPA; a native shell, if ever wanted, is a separate FR (the desktop trigger is FR-APP-002).
- Re-hosting or replacing Grafana. The console links to Grafana through the obs-proxy for deep metric panels; it does not reimplement dashboards.
- Write or admin-mutation flows beyond what the first screens need. The first release is read-oriented (compliance views, health, usage); operator mutations are added screen by screen in later FRs.

## Dependencies

- FR-AI-022 ai-gateway HTTP serving surface - the health and usage endpoints the console reads for its gateway screen.
- FR-AI-105 local + external model providers - the same serving surface; the gateway-health screen reflects whichever providers (local or cloud) the gateway routes to.
- FR-OBS-008 compliance-view scoping - the scoped compliance-view endpoints the console renders as its first obs screen.
- FR-OBS-002 tenant-aware Grafana proxy - the obs-proxy the console links to for deep metric panels.
- FR-AUTH-004 JWT and JWKS - the session token the auth shell holds and presents on every call.
- FR-AUTH-104 OIDC SSO - the IdP sign-in path for operators who authenticate through an identity provider.
- FR-PORTAL-006 and FR-PORTAL-007 - referenced for contrast and house style, not consumed: those are the tenant-facing client-surface FRs, and the operator console deliberately sits in a separate module from them.
- Cross-cutting: the existing Caddy front that terminates the other CyberOS surfaces; the console adds a static-serving snippet, not a new ingress.

## AI Authorship Disclosure

- Tools used: Claude (Cowork), authoring this FR from Stephen's capability request and the existing gateway, obs, auth, and portal FRs.
- Scope: full draft of this specification, including the normative clauses, the alternatives, the metrics, and the scope boundaries. No console code is written by this FR; the SPA is built in a later session.
- Human review: Stephen reviews and approves before status moves past draft. The "front-end only, no new backend" boundary and the CDS-as-source-of-truth rule are operator-mandated, and the paired audit (FR-APP-001.audit.md) validates the format before merge.
