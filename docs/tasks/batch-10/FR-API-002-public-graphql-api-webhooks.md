---
title: "API — public GraphQL API + webhook subscriptions for partner integrations"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p2
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

Layer two more developer surfaces on top of FR-API-001's REST foundation: (1) a **public GraphQL endpoint** at `api.cyberos.world/graphql/v1` serving a curated, public-safe subgraph of the internal Apollo Federation v2 supergraph — same auth model (per-tenant API keys + scopes), same rate limits, but giving developers field-selection power that REST can't match; (2) a **webhook subscription system** that lets developers register HTTPS endpoints to receive real-time events (issue.created, deal.stage_changed, invoice.paid, time_entry.submitted, etc.), with HMAC-SHA-256 signing, automatic retry-with-exponential-backoff, dead-letter queue after 24 hours of failures, and a developer-facing webhook event log + replay UI. The GraphQL surface uses GraphQL Yoga (or compatible) gateway with depth-limit + complexity-limit + query allowlisting (no introspection in production). Webhooks are emitted from NATS JetStream events (FR-INFRA-001's substrate) → a webhook-emitter worker that filters per-tenant subscriptions, signs with the tenant's webhook secret, fires HTTPS POSTs. The combined surface unlocks integrations with Zapier, Make, n8n, custom internal tooling, and partner platforms.

## Problem

PRD §7.1 (Public APIs cross-cutting): "REST + GraphQL + webhooks". GraphQL gives partners flexibility for field selection without N+1 calls; webhooks let them react in real-time instead of polling. P4 launch ecosystem stories require both.

Three failure modes if not built carefully:

- **GraphQL query abuse.** Nested queries can be O(n^k) cost. Mitigation: depth-limit (5), complexity-limit (1000 points), persisted-query allowlist for production.
- **Webhook replay attacks.** A leaked webhook secret + replayed event could trick the tenant's downstream system. Mitigation: HMAC-SHA-256 + timestamp + 5-min replay window + webhook secret rotation.
- **Webhook delivery failures cascading.** A single bad subscription endpoint slowing down the queue. Mitigation: per-subscription retry queue + circuit breaker + DLQ + admin-console visibility.

## Customer Quotes

<!-- Required when client_visible: true. Verbatim, attributed where possible. Paraphrasing here costs you the signal. -->

<untrusted_content source="other">
…paste verbatim customer quote here…
</untrusted_content>

<!-- TODO during implementation PR: capture real customer quotes from sales calls / NPS / support tickets. -->

## Proposed Solution

### Public GraphQL API

**Schema scope.**

The public schema is a curated subset of the internal Apollo Federation supergraph. Object types exposed:
- `Project`, `Issue`, `Cycle`, `Engagement` (FR-PROJ).
- `Account`, `Contact`, `Deal`, `Activity` (FR-CRM).
- `Invoice`, `Payment` (FR-INV).
- `TimeEntry` (FR-TIME).
- `KbPage` (FR-KB).
- `User` (limited fields: id, name, email — never compensation).

Object types NEVER exposed: any HR-secure / REW / ESOP / GENIE persona / BRAIN raw / OBS internal / MCP internal / CP internal / DOC raw signing types.

Query types (read-only):
```graphql
type Query {
  project(id: ID!): Project
  projects(filter: ProjectFilter, limit: Int = 50): ProjectConnection!
  issue(id: ID!): Issue
  issues(filter: IssueFilter, limit: Int = 50): IssueConnection!
  account(id: ID!): Account
  accounts(filter: AccountFilter, limit: Int = 50): AccountConnection!
  deal(id: ID!): Deal
  invoice(id: ID!): Invoice
  invoices(filter: InvoiceFilter, limit: Int = 50): InvoiceConnection!
  timeEntries(filter: TimeFilter, limit: Int = 50): TimeEntryConnection!
  kbPage(id: ID!): KbPage
}
```

Mutation types (narrow write surface, mirroring REST writes):
```graphql
type Mutation {
  createIssue(input: CreateIssueInput!): IssuePayload!
  updateIssue(id: ID!, input: UpdateIssueInput!): IssuePayload!
  appendIssueComment(issueId: ID!, body: String!): CommentPayload!
  upsertDeal(input: UpsertDealInput!): DealPayload!
  updateDeal(id: ID!, input: UpdateDealInput!): DealPayload!
}
```

Subscriptions: not exposed in MVP (long-lived WebSocket from public clients is operationally heavy at MVP scale; webhook subscriptions cover the same use case).

**Auth + rate limiting.**

Same `Authorization: Bearer cyberos_<env>_…` as REST. Same scopes. Same rate-limits (1000/min T2, 5000/min T3), but additionally:
- **Depth limit**: 5 nested levels.
- **Complexity limit**: 1000 points (per the standard Apollo cost analysis).
- **Query allowlisting**: in production, only persisted queries (registered by hash via `/admin/security/graphql/persist`) execute. Ad-hoc queries return 403 in production. Sandbox environment allows ad-hoc queries.

**Introspection.**

Disabled in production. Enabled in sandbox. Schema files (`.graphqls`) shipped with SDKs.

**Implementation.**

- GraphQL Yoga or Apollo Server v5 as the gateway runtime.
- Resolvers thin-wrap the same domain services as REST.
- DataLoader for N+1 prevention.
- Apollo Federation v2 internal supergraph stays internal; public GraphQL is a separate gateway with hand-curated schema (defence in depth).

### Webhook Subscriptions

**Schema.**

```sql
CREATE TABLE api.webhook_subscription (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  name TEXT NOT NULL,
  url TEXT NOT NULL,                                                          -- HTTPS only; HTTP rejected
  secret_hash TEXT NOT NULL,                                                  -- HMAC secret hash (key shown once)
  event_types TEXT[] NOT NULL,                                                -- e.g. ["issue.created", "deal.stage_changed"]
  filter JSONB,                                                               -- optional: e.g. {"project_id": "<uuid>"}
  status TEXT NOT NULL DEFAULT 'active',                                      -- "active" | "paused" | "circuit_broken" | "revoked"
  failure_count INT NOT NULL DEFAULT 0,
  last_success_at TIMESTAMPTZ,
  last_failure_at TIMESTAMPTZ,
  circuit_broken_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by_user_id UUID NOT NULL,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE api.webhook_delivery (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  subscription_id UUID NOT NULL REFERENCES api.webhook_subscription(id),
  event_id UUID NOT NULL,
  event_type TEXT NOT NULL,
  payload_blob_id UUID NOT NULL,                                              -- the JSON payload
  delivery_attempt INT NOT NULL DEFAULT 1,
  status TEXT NOT NULL,                                                       -- "pending" | "success" | "failed" | "dead_lettered"
  http_status_code INT,
  response_body_excerpt TEXT,                                                 -- first 1KB
  scheduled_at TIMESTAMPTZ NOT NULL,
  delivered_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);
```

**Event catalogue (Phase 1).**

- `issue.created`, `issue.updated`, `issue.status_changed`, `issue.assigned`.
- `project.created`, `project.archived`.
- `cycle.started`, `cycle.closed`.
- `deal.created`, `deal.updated`, `deal.stage_changed`, `deal.won`, `deal.lost`.
- `account.created`, `account.updated`.
- `invoice.created`, `invoice.sent`, `invoice.paid`, `invoice.overdue`.
- `time_entry.submitted`, `time_entry.approved`, `time_entry.rejected`.
- `kb_page.published`, `kb_page.updated`.

**Payload format.**

```json
{
  "id": "evt_…",                              // unique event id
  "type": "issue.status_changed",
  "occurred_at": "2028-09-15T10:23:45Z",
  "tenant_id": "<tenant-uuid>",
  "object": { ... full object after change ... },
  "previous": { ... fields that changed, before values ... },
  "actor": {
    "kind": "user" | "api_key" | "system",
    "id": "...",
    "display_name": "..."
  }
}
```

**Signing.**

Headers:
```
X-CyberOS-Event: issue.status_changed
X-CyberOS-Delivery: <delivery-uuid>
X-CyberOS-Timestamp: 1745923245
X-CyberOS-Signature: sha256=<hex>
```

Signature: `HMAC-SHA-256(secret, "<timestamp>.<raw_body>")`. Receivers verify timestamp within 5 minutes + signature matches.

**Retry policy.**

- Initial delivery: immediate after event.
- On 2xx: success.
- On 4xx (except 408, 429): no retry; flagged in admin console.
- On 5xx, 408, 429, network error: retry with exponential backoff: 1m, 5m, 30m, 2h, 8h. After 5 failures (≈ 12 hours): subscription enters `circuit_broken` status.
- Circuit break: subscription paused; admin Notify; subsequent events queued for replay.
- Dead letter: deliveries failing after 24 hours total move to DLQ; visible in admin console with a "replay" CTA.

**Idempotency.**

Receivers deduplicate using `X-CyberOS-Delivery` header (unique per delivery attempt) + the event `id` (stable across retries of the same event).

**Admin surface (in FR-TEN-005's Security pane).**

- Subscription list: name, URL, event types, status, success rate (last 24h).
- Create subscription: form + secret rotation + test-fire.
- Delivery log: list of recent deliveries with status, latency, response code; "replay" CTA per delivery.
- Circuit-break recovery: explicit "test-fire and recover" CTA.

**NATS JetStream as event source.**

The webhook-emitter worker subscribes to NATS subjects matching the tenant's event types; for each event, looks up matching subscriptions, schedules deliveries. Decouples webhook delivery from the originating module.

## Alternatives Considered

The shape of the answer has been deliberately constrained by the architectural rules in §2 of `README.md` and the locked decisions cited in *Dependencies*. Notable rejected approaches:

- Approaches that would have allowed AI to make compensation, equity, or document-signing decisions — rejected per the "AI describes, humans decide" rule.
- Approaches that would have created cross-tenant read or write paths — rejected per the cross-tenant invariant (FR-TEN-001 invariant test harness).
- Where there are FR-specific alternatives, they're discussed inline in *Proposed Solution* and *Constraints*.

<!-- TODO during implementation PR: replace with FR-specific rejected alternatives. -->

## Out of Scope

- GraphQL subscriptions (WebSocket from public clients) — deferred.
- gRPC API — not in PRD.
- Webhook event filtering by complex predicates — Phase 1 supports object-id + status filters only.
- Custom event types defined by tenants — not at MVP.
- Webhook payload encryption beyond HTTPS + HMAC — no end-to-end payload encryption at MVP.
- Webhook event replay older than 7 days — not at MVP.

## Dependencies

- FR-API-001 (REST API + auth model + key management).
- FR-AUTH-001/002 (RBAC + audit).
- FR-INFRA-001 (NATS JetStream substrate).
- FR-TEN-001 (residency partitioning — webhooks go through the tenant's shard).
- FR-TEN-005 (admin console webhook + GraphQL persisted-query management).
- FR-BILL-001 (rate-limit + budget).
- FR-OBS-002 (delivery metrics + dead-letter visibility).
- All module FRs whose events are exposed.
- DEC-005, DEC-018 (Apollo Federation v2; NATS).

## Constraints

- **GraphQL persisted-query allowlist mandatory in production.** Ad-hoc queries return 403; sandbox is the test surface.
- **Webhook URL must be HTTPS.** HTTP rejected at registration.
- **HMAC + timestamp + 5-min replay window** required.
- **Circuit-break after 5 failures.** Cannot be bypassed.
- **DLQ retention 7 days.** After that, deliveries are permanently lost.
- **No webhook to localhost / private IP / internal CyberOS IP.** Outbound IP allowlist enforced (no SSRF surface).
- **Webhook secret shown once at creation.** Never retrievable later; rotate flow generates a new one.

## Compliance / Privacy

- **PDPL Decree 13/2023:** webhook payloads can contain personal data; tenants are responsible for protecting payloads at receiver side; this is documented in the API ToS.
- **GDPR Article 32:** HMAC + TLS + timestamp; security controls demonstrated.
- **Cross-border transfer:** webhooks fire from the tenant's residency shard; the receiver URL can be anywhere; tenants are responsible for confirming receiver-jurisdiction compatibility (a Schrems II-relevant note in the FR-CP-004 sub-processor list).
- **Data minimisation:** payload contains only the public-API-exposed fields; no compensation/equity/persona/raw-BRAIN data.

## Risk Assessment (AI-emitting features)

No AI in this FR. `eu_ai_act_risk_class: not_ai`.

## Vietnamese-locale considerations

- GraphQL responses are JSON; locale-irrelevant.
- Error messages localised via `Accept-Language` header.
- Webhook payloads include localised display fields where present (e.g. issue title in its authored language).

## Scope (acceptance criteria — auditable)

- [ ] GraphQL endpoint at `api.cyberos.world/graphql/v1` live; introspection disabled in prod, enabled in sandbox.
- [ ] Persisted-query allowlist enforced in prod; ad-hoc query returns 403.
- [ ] Depth + complexity limits enforced (test cases: depth-6 query → 400, complexity-1500 query → 400).
- [ ] All Phase-1 query types resolvable; mutations work; auth scopes enforced.
- [ ] Webhook subscription create/list/revoke flow in admin console.
- [ ] Webhook secret shown once + hashed at rest.
- [ ] HTTPS-only URL validation at registration.
- [ ] HMAC signing + timestamp on all delivery attempts; receiver verification example provided in docs.
- [ ] Retry policy: 1m, 5m, 30m, 2h, 8h backoff implemented.
- [ ] Circuit break after 5 failures; admin Notify fires.
- [ ] Dead-letter queue with 7-day retention; replay CTA works.
- [ ] SSRF prevention: outbound IP allowlist rejects private/internal IPs.
- [ ] Per-subscription delivery log + admin surface in FR-TEN-005.
- [ ] Pre-launch load test: 1,000 events/min sustained, all delivered within 60s, all NFRs green.
- [ ] Idempotency: same event with same `delivery-id` deduplicated correctly at receiver (test client provided).

**Gherkin (PRD §19.18).**

```gherkin
Feature: GraphQL persisted-query allowlist enforced in production

  Scenario: Ad-hoc query rejected in production
    Given a developer has a valid API key with scope "read:projects"
    When they POST { query: "{ projects { id } }" } to api.cyberos.world/graphql/v1
    Then the response is 403
    And the error message is "Ad-hoc GraphQL queries are not allowed in production. Register your query at /admin/security/graphql/persist."
    When they POST the same query with header X-CyberOS-Persisted-Hash: <registered-hash>
    Then the response is 200 with the data

Feature: Webhook circuit-break after 5 consecutive failures

  Scenario: Webhook subscription fails 5 times in a row
    Given a webhook subscription S with active status
    When 5 consecutive delivery attempts fail (returning 5xx + retried per backoff)
    Then S transitions to "circuit_broken" status
    And the tenant admin receives a Notify
    And subsequent events are queued not delivered (visible in DLQ after 24h)
    When the admin presses "test-fire and recover" with a new test event
    And the test delivers successfully (2xx)
    Then S returns to "active" status
    And the queued events are flushed within 5 minutes

Feature: HMAC signature verification

  Scenario: Receiver detects a tampered payload
    Given a webhook delivery is sent to the receiver
    When an attacker intercepts and modifies the body before reaching the receiver
    Then the receiver computes HMAC of the modified body
    And the computed HMAC does not match the X-CyberOS-Signature header
    And the receiver rejects the delivery as invalid
```

## Success Metrics

- GraphQL adoption: ≥ 3 tenants with persisted queries registered in first 90 days.
- Webhook adoption: ≥ 5 tenants with active subscriptions in first 90 days.
- Webhook delivery success rate: ≥ 99%.
- 99th-percentile webhook delivery latency: ≤ 30 seconds from event occurrence.
- Dead-lettered deliveries: < 0.1% of total.
- GraphQL query rejection due to persisted-query violation: ≤ 1% (as developers learn the model).

## Sales/CS Summary

<!-- Required when client_visible: true. One paragraph written so a non-engineer can pitch the feature. Plain English. No internal jargon, no module codes, no speculation about future scope. -->

<!-- TODO during implementation PR: write the customer-facing pitch. -->

## Open Questions

- **OQ-API-002-01.** Should we offer a "test webhook" surface that fires synthetic events on demand for developer testing? Default: yes; endpoint at `/admin/security/webhooks/<id>/test-fire`.
- **OQ-API-002-02.** Should we support webhook batching (multiple events in one POST) for high-volume subscribers? Default: defer to FR-API-004; one-event-per-POST at MVP.
- **OQ-API-002-03.** Should GraphQL responses include cursor-based pagination at MVP? Default: yes — Connection types (`ProjectConnection`, etc.) with `pageInfo`.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.

## References

- PRD §7.1 Public APIs cross-cutting; PRD §14.5.1 P4 entry-gate.
- SRS Decisions Log: DEC-005, DEC-018.
- FR-API-001, FR-AUTH-001/002, FR-INFRA-001, FR-TEN-001/005, FR-BILL-001, FR-OBS-002.

---

*ai_authorship: co_authored — drafted by Claude Cowork on 2026-05-03.*
