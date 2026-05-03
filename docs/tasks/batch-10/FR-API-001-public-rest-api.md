---
title: "API — public REST API: per-tenant API keys, OpenAPI 3.1, versioning, rate-limiting, audit"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: backend
eu_ai_act_risk_class: not_ai
target_release: "P4 / 2028-Q3"
client_visible: true
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Stand up the **public REST API** at `api.cyberos.world/v1` — the first surface that lets a tenant's developers integrate CyberOS programmatically (custom internal tools, Zapier-style automations, partner integrations) without going through the Apollo Federation supergraph (which stays internal-only and Module-Federation-driven). Five primitives: (1) **OpenAPI 3.1 spec** as the source of truth at `api.cyberos.world/v1/openapi.json`, with auto-generated Stainless-style SDKs in TypeScript + Python + Go shipped as the first client libraries; (2) **per-tenant API keys** with scoped permissions (read-only, read-write, per-module), generated + revoked from the tenant admin console (FR-TEN-005); (3) **versioning** via URL path (`/v1`) + a deprecation policy (12-month minimum sunset); (4) **rate-limiting** with token-bucket per API key (default 1000 req/min for T2, 5000 req/min for T3, configurable downward by tenant) + 429 with `Retry-After`; (5) **audit + observability** — every API call logged in `audit.events` with the API key + IP + endpoint + status + latency, queryable via the admin console. The API is **read-mostly** at MVP: it covers reading projects/tasks/issues, reading CRM accounts/deals, reading invoices, reading time entries, plus narrow write surfaces for high-value automation (create task, append comment, upsert deal). All compensation, equity, and persona-related endpoints stay off the public API at MVP — the AI/HR boundary remains intact.

## Problem

PRD §7.1 (Public APIs cross-cutting) names this as the "external developer surface" that lets tenants extend CyberOS without forking it. PRD §14.5.1 P4 entry-gate criterion: "First non-tenant-employee developer integration in production using the public API for ≥ 30 days."

Without a public REST API, tenants are locked in to the UI surface; integration with their existing tooling (Slack notifications outside CyberOS-internal, custom dashboards, Zapier-style flows, BI extracts) is impossible. The API is the surface that turns CyberOS from "platform" into "platform-with-ecosystem".

Three failure modes if not built carefully:

- **Apollo Federation drift.** If the REST API duplicates business logic, the two surfaces drift over time. Mitigation: REST is a thin wrapper over the same domain services that GraphQL hits; one source of truth.
- **API-key leakage.** A tenant developer commits an API key to a public GitHub repo. Mitigation: GitHub Push Protection scanning + automatic detection + Notify the tenant admin within 5 minutes + auto-revoke after 24 hours if not rotated.
- **Rate-limit bypass.** A misconfigured automation pegs the API. Mitigation: per-tenant ceilings + per-key budget alerts + auto-throttle that's not bypassable from the client side.

## Customer Quotes

<!-- Required when client_visible: true. Verbatim, attributed where possible. Paraphrasing here costs you the signal. -->

<untrusted_content source="other">
…paste verbatim customer quote here…
</untrusted_content>

<!-- TODO during implementation PR: capture real customer quotes from sales calls / NPS / support tickets. -->

## Proposed Solution

A separate API gateway `api.cyberos.world` running an OpenAPI-first stack. Every endpoint maps to one or more domain services already exposed in Apollo Federation; no duplicate logic.

**OpenAPI 3.1 spec.**

The spec lives at `api.cyberos.world/v1/openapi.json`. Generated from server-side annotations in the domain services. Versioning + change tracking in CI; breaking changes block PR.

Initial endpoint catalogue (read-mostly):

- **Projects** (FR-PROJ-001..010):
  - `GET /v1/projects` — list projects.
  - `GET /v1/projects/{id}` — get project detail.
  - `GET /v1/projects/{id}/issues` — list issues.
  - `GET /v1/issues/{id}` — get issue detail.
  - `POST /v1/issues` — create issue (write).
  - `PATCH /v1/issues/{id}` — update issue status, assignee, priority (write).
  - `POST /v1/issues/{id}/comments` — append a comment (write).
- **CRM** (FR-CRM-001..004):
  - `GET /v1/accounts` — list accounts.
  - `GET /v1/accounts/{id}` — get account detail.
  - `GET /v1/contacts` — list contacts.
  - `GET /v1/deals` — list deals.
  - `GET /v1/deals/{id}` — get deal detail.
  - `POST /v1/deals` — upsert deal (write).
  - `PATCH /v1/deals/{id}` — update deal (write).
  - `GET /v1/activities` — list activities.
- **Invoicing** (FR-INV-001..004):
  - `GET /v1/invoices` — list invoices.
  - `GET /v1/invoices/{id}` — get invoice detail.
  - `GET /v1/invoices/{id}/pdf` — download PDF.
- **Time** (FR-TIME-001..003):
  - `GET /v1/time-entries` — list time entries.
  - `GET /v1/time-entries/me` — list my time entries.
- **Knowledge Base** (FR-KB-001..003):
  - `GET /v1/kb/pages` — list pages.
  - `GET /v1/kb/pages/{id}` — get page.
- **Audit** (FR-AUTH-002):
  - `GET /v1/audit/events` — list events (admin scope only).
- **Health**:
  - `GET /v1/health` — health check (no auth).

NOT exposed at MVP:
- Compensation (REW), equity (ESOP), payroll, employee personal data (HR-secure), persona configuration (GENIE), AI Gateway internals, MCP internals, BRAIN raw access, OBS internals, CP internals, BILL internals, DOC raw signing internals.

**Authentication.**

API keys: `cyberos_<env>_<random32>` (e.g. `cyberos_live_abc123…`).
Header: `Authorization: Bearer cyberos_live_…`.

Per-key scopes:
- `read:projects`, `write:projects`, `read:crm`, `write:crm`, `read:invoices`, `read:time`, `read:kb`, `read:audit`, `read:* (all read)`, `write:* (caution — admin only)`.

Per-tenant key list managed in `/admin/security` (FR-TEN-005). Each key:
- Name (developer-facing label).
- Created-at + last-used-at.
- Scopes selected at creation; immutable after.
- Status: active / revoked.
- Optional IP allowlist.

API keys are stored in `auth.api_key` table:
```sql
CREATE TABLE auth.api_key (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  name TEXT NOT NULL,
  key_hash TEXT NOT NULL,                                                     -- SHA-256 of the key; never store the key itself
  key_prefix TEXT NOT NULL,                                                   -- first 12 chars; for UI surfacing
  scopes TEXT[] NOT NULL,
  ip_allowlist CIDR[],
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by_user_id UUID NOT NULL,
  last_used_at TIMESTAMPTZ,
  revoked_at TIMESTAMPTZ,
  revoked_by_user_id UUID,
  revoke_reason_md TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);
```

**Rate limiting.**

Token-bucket per API key:
- T1 plan: API access not included.
- T2 plan: 1000 req/min per key, 100 keys per tenant max.
- T3 plan: 5000 req/min per key, 500 keys per tenant max.

429 response:
```
HTTP/1.1 429 Too Many Requests
Retry-After: 30
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1745923245

{
  "error": "rate_limit_exceeded",
  "message": "API key has exceeded its rate limit; retry after 30 seconds",
  "request_id": "req_…"
}
```

Per-tenant aggregate ceiling: 10x the per-key limit (so a tenant with many keys can't blow past their plan's allotment).

**Versioning.**

URL path: `/v1`, `/v2`, etc. Major version bump only on breaking changes. Sunset policy:
- 12-month deprecation notice in headers + email + admin-console banner.
- After 12 months, deprecated version returns 410 with the upgrade URL.
- OpenAPI spec includes `deprecated: true` at endpoint level for fields being phased out within a version.

**SDKs.**

Stainless-generated SDKs in TypeScript, Python, Go shipped at:
- `npm install @cyberos/sdk-typescript`
- `pip install cyberos-sdk`
- `go get github.com/cyberos/sdk-go`

Each SDK includes typed clients per endpoint + automatic retry-on-429 with exponential backoff + request-ID logging.

**Observability.**

Every request:
- `audit.events` row with `actor_kind = 'api_key'`, `actor_id = <key.id>`, `endpoint`, `method`, `status`, `latency_ms`, `request_id`, `ip`.
- Prometheus metric `cyberos_api_requests_total{tenant, method, endpoint, status}`.
- Tempo trace.
- Loki log line.

Visible in tenant admin console (`/admin/security` → API tab) and in `/admin/overview` charts.

**Webhook subscriptions.**

Out of scope for this FR (handled in FR-API-002 alongside GraphQL). REST API is request-response only at MVP.

**GitHub Push Protection integration.**

CyberOS submits its API-key prefix `cyberos_live_*` to GitHub's secret-scanning partner program. When a key prefix is committed to a public GitHub repo, GitHub notifies CyberOS within seconds; CyberOS sends a Notify to the tenant admin within 5 minutes; auto-revoke after 24 hours if not rotated by the admin.

**Sandbox environment.**

`api.cyberos.world/v1` is production. A separate sandbox at `api-sandbox.cyberos.world/v1` provides non-production surface for developer testing — same OpenAPI, separate keys (`cyberos_test_*`), real request handling with synthetic test tenant.

## Alternatives Considered

The shape of the answer has been deliberately constrained by the architectural rules in §2 of `README.md` and the locked decisions cited in *Dependencies*. Notable rejected approaches:

- Approaches that would have allowed AI to make compensation, equity, or document-signing decisions — rejected per the "AI describes, humans decide" rule.
- Approaches that would have created cross-tenant read or write paths — rejected per the cross-tenant invariant (FR-TEN-001 invariant test harness).
- Where there are FR-specific alternatives, they're discussed inline in *Proposed Solution* and *Constraints*.

<!-- TODO during implementation PR: replace with FR-specific rejected alternatives. -->

## Out of Scope

- GraphQL API (FR-API-002, next FR in this batch).
- Webhooks (FR-API-002).
- gRPC API (not in PRD; revisit post-launch if partner demand).
- Bulk-export endpoints (deferred — admin console export covers this).
- Compensation, equity, persona, BRAIN, OBS internals (architectural exclusion).
- Multi-tenant API keys (a key always scopes to one tenant).
- OAuth 2.1 third-party app authorisation (deferred — OAuth lives at the AUTH realm for human users; API keys are the surface for machine clients at MVP).
- Real-time streaming endpoints (deferred to FR-API-002 + WebSockets).

## Dependencies

- FR-AUTH-001 (RBAC + RLS — API key scopes mapped to roles).
- FR-AUTH-002 (audit chain — every API call logged).
- FR-INFRA-001 (gateway pattern + Module Federation surfaces remain internal).
- FR-TEN-001 (residency partitioning — API gateway routes to the tenant's shard via Host header or key-prefix lookup).
- FR-TEN-005 (admin console API-key management).
- FR-BILL-001 (per-plan rate limits + 80/100/110 ladder applies to API request budgets).
- FR-OBS-002 (Prometheus + Loki + Tempo).
- All module FRs whose endpoints are exposed (PROJ, CRM, INV, TIME, KB).
- DEC-005 Apollo Federation v2 (REST is a thin wrapper, never duplicate logic).

## Constraints

- **One source of truth.** REST handlers call the same domain services as GraphQL; no business logic in the REST layer.
- **No compensation/equity/persona/BRAIN/OBS endpoints at MVP.** Architectural exclusion.
- **API keys hashed at rest.** Plain text shown once at creation; never retrievable later.
- **Per-tenant ceiling enforced.** Tenant cannot exceed plan's aggregate allotment regardless of key count.
- **Sandbox is real-handler, not mock.** Tests against real code paths.
- **Sunset is enforced via 410 after 12 months.** Cannot extend without explicit Engineering Lead sign-off.
- **GitHub Push Protection participation is mandatory.** Non-negotiable.

## Compliance / Privacy

- **PDPL Decree 13/2023:** API access logs are personal data of the API caller (developer); retention 24 months then archive.
- **GDPR Article 32:** authentication + access control + logging are core security controls.
- **EU AI Act:** N/A.
- **SOC 2 CC6 (Logical Access):** API keys + scopes + audit trail; demonstrated.
- **PCI-DSS:** N/A (no payment data flows through API).
- **API key rotation policy:** recommended 90-day rotation; surfaced in admin console; not enforced.
- **Cross-border transfer:** API requests are routed to the tenant's residency shard; cross-shard calls forbidden.

## Risk Assessment (AI-emitting features)

No AI in this FR. `eu_ai_act_risk_class: not_ai`.

## Vietnamese-locale considerations

- API responses are JSON; locale-irrelevant.
- Error messages localised via `Accept-Language` header (vi-VN, en-US, others fall back to en-US).
- Vietnamese e-invoice numbering preserved exactly in `GET /v1/invoices/{id}` response.
- Documentation site supports vi-VN + en-US; SDK comments en-US only at MVP.

## Scope (acceptance criteria — auditable)

- [ ] OpenAPI 3.1 spec at `api.cyberos.world/v1/openapi.json` — auto-generated, validated.
- [ ] All Phase-1 endpoints (projects, CRM, invoices, time, KB, audit, health) implemented + tested.
- [ ] API key creation + revocation flow in admin console (FR-TEN-005); CI test ensures keys are hashed at rest.
- [ ] Rate-limiting: T2 (1000 req/min) + T3 (5000 req/min) enforced; 429 with `Retry-After` returned.
- [ ] Per-tenant aggregate ceiling enforced (10x per-key).
- [ ] Versioning policy documented; CI breaks PR on breaking change without major-version bump.
- [ ] SDKs published to npm + pip + Go modules; Stainless integration works.
- [ ] Sandbox environment live at `api-sandbox.cyberos.world` with `cyberos_test_*` keys.
- [ ] GitHub Push Protection integration registered; test commit of a sandbox key triggers CyberOS-side detection within 60 seconds.
- [ ] Audit chain captures every API call; queryable in admin console.
- [ ] Cross-shard call attempt fails: a key for vn-shard tenant can't be used to query us-shard data.
- [ ] Pre-launch load test: sustained 1000 req/min across 10 keys for 1 hour, all NFRs green.

**Gherkin (PRD §19.18).**

```gherkin
Feature: API key with read-only scope cannot write

  Scenario: Read-only key attempts a write
    Given an API key K with scopes ["read:projects"]
    When K issues POST /v1/issues with a valid issue payload
    Then the response is 403 Forbidden
    And the audit chain records "api_authz_denied" with K + endpoint + reason

Feature: GitHub Push Protection auto-revoke

  Scenario: A key is committed to public GitHub
    Given a key K is committed to a public GitHub repo
    When GitHub notifies CyberOS via the secret-scanning partner program
    Then CyberOS sends a Notify to the tenant admin within 5 minutes
    And shows the leak in the admin console
    When 24 hours pass without rotation
    Then K is automatically revoked
    And the audit chain records "api_key_auto_revoked" with reason "leaked_in_public_github"
    And subsequent K usage returns 401

Feature: Sunset deprecated version

  Scenario: Client uses /v1 after 12-month deprecation period for an endpoint that was sunsetted
    Given /v1/projects/by-name was deprecated 13 months ago in favor of /v1/projects?name=
    When a client issues GET /v1/projects/by-name?name=Alpha
    Then the response is 410 Gone
    And the response body includes a link to the migration guide
```

## Success Metrics

- First-month API key adoption: ≥ 5 tenants generate ≥ 1 API key.
- API request error rate: < 1% (excluding 429s).
- 99th-percentile latency: ≤ 500ms.
- Auto-revoke from leak detection: 100% of detected leaks within 24 hours.
- SDK monthly download count tracked in npm/pip/Go module registries.

## Sales/CS Summary

<!-- Required when client_visible: true. One paragraph written so a non-engineer can pitch the feature. Plain English. No internal jargon, no module codes, no speculation about future scope. -->

<!-- TODO during implementation PR: write the customer-facing pitch. -->

## Open Questions

- **OQ-API-001-01.** Should we offer OAuth 2.1 third-party app authorisation at MVP (allowing a partner to act on behalf of a tenant user) or defer? Default: defer to FR-API-003.
- **OQ-API-001-02.** Should the audit endpoint be exposed in the public API at all? Default: yes, but require `read:audit` scope which is admin-only by default.
- **OQ-API-001-03.** Should we offer field-level filtering (`?fields=id,name`) at MVP for response-size optimisation? Default: yes if cheap to add.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.

## References

- PRD §7.1 Public APIs cross-cutting; PRD §14.5.1 P4 entry-gate.
- SRS Decisions Log: DEC-005, DEC-013, DEC-016.
- FR-AUTH-001/002, FR-INFRA-001, FR-TEN-001/005, FR-BILL-001, FR-OBS-002, all module FRs cited above.

---

*ai_authorship: co_authored — drafted by Claude Cowork on 2026-05-03.*
