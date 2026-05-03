---
title: "AI Gateway — centralised LLM routing with Bedrock primary, ZDR fallback, and PII redaction"
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

Stand up the centralised AI Gateway through which every LLM call in CyberOS flows. The gateway owns model routing (primary AWS Bedrock with Anthropic Claude Sonnet 4.6 and Haiku 4.5; failover Anthropic API on Zero-Data-Retention; second failover OpenAI on ZDR), prompt caching, per-tenant cost accounting, PII redaction (Microsoft Presidio + Vietnamese-aware custom rules), persona-version stamping (the active CUO persona's system prompt is prepended at gateway level, not at client level), residency enforcement (the gateway routes to the Bedrock endpoint in the same region as the tenant's primary data store), latency-budget enforcement, per-tenant rate limiting, and circuit breakers per provider. LiteLLM is the routing core extended with CyberOS middleware. Self-hosted small models — `BAAI/bge-m3` for embeddings, `BAAI/bge-reranker-v2-m3` for reranking, optional `Llama-3.1-8B` for the redaction classifier — run on a single shared GPU node behind the gateway. Cost target: ≤ $150/month at internal scale (PRD §4.3 guardrail) and ≤ $4 per active user per month at 50-tenant scale.

## Problem

Every other CyberOS module that does anything AI — BRAIN's hybrid retrieval, GENIE/CUO's persona, CHAT's smart replies, EMAIL's CaMeL extraction, KB's "ask this page", REW's payslip narrator (read-only), CRM's next-action drafter — needs three things from "the AI": stable provider routing under outage conditions, defensible compliance (Vietnamese PDPL data residency, EU AI Act transparency for the limited-risk class, ZDR for any provider that holds tenant data), and tractable cost. None of those concerns belong in the consumer module's code. The AI Gateway is the seam between "this module needs an LLM" and "the platform answers under contract".

The PRD's locked decisions concentrate here: AWS Bedrock as primary (DEC-024), Anthropic ZDR as failover (DEC-025), OpenAI ZDR as second failover (DEC-026), LiteLLM as routing core (DEC-027), bge-m3 for embeddings (DEC-028), Presidio + custom Vietnamese rules for redaction (DEC-029), persona-version stamping at the gateway (DEC-030). Every consumer module assumes these properties without re-implementing them.

The gateway is the point at which we satisfy EU AI Act Article 50 (transparency obligation for AI-generated content visible to a natural person): every response that the gateway returns carries a `persona_version`, a `model`, and an `ai_disclosure_id` that downstream UIs render as a citation chip. The gateway also satisfies the residency invariant: a Vietnamese tenant's prompt never traverses an AWS US region.

## Proposed Solution

The shape of the answer is a single service `cyberos-ai-gateway` running as a Kubernetes Deployment with three replicas in each residency region, fronted by a Kubernetes Service and exposed only inside the cluster (no public internet exposure). The gateway exposes one HTTP endpoint to internal callers and one MCP-aware endpoint for tool-driven calls.

**Routing core.** LiteLLM (open-source, MIT-licensed, current stable line as of 2026-05) is the routing primitive. The gateway is a thin server that wraps LiteLLM with CyberOS-specific middleware. The default route per request:

1. **Primary**: AWS Bedrock in the tenant's residency region (Singapore for vn/sg tenants; Frankfurt for eu tenants; Ohio for us tenants), with `anthropic.claude-sonnet-4-6-20251022` for the "thinking" tier and `anthropic.claude-haiku-4-5-20251001` for the "fast" tier.
2. **First failover**: Anthropic API on the `zdr-1` data-retention setting, region-locked (the API has region-aware endpoints as of 2026-Q1).
3. **Second failover**: OpenAI API on `zdr-1`, region-locked.

The selection logic is a small state machine: prefer Primary; if the Primary returns 5xx or exceeds the latency budget, mark the provider as `unhealthy` for 60 seconds and try First failover; if First failover also fails, try Second failover; if all three fail, return a 503 to the caller and write an audit row with `action: "ai.gateway.exhausted"`. Health is per-provider per-region; a Bedrock-Frankfurt outage does not affect Bedrock-Singapore.

**Prompt caching.** The gateway maintains a per-tenant Redis-backed cache keyed on `sha256(system_prompt || persona_version || canonical_user_input || model || temperature)`. Hits return in-memory in ≤ 5 ms p99 with no model call. The cache TTL is 1 hour for Notify-class CUO outputs, 6 hours for read-only KB Q&A, and zero for any prompt that includes BRAIN-derived facts (we do not want stale cached facts shadowing a newly added memory). Cache hit rate target: > 30% on synthetic load by S0-2 demo (PRD §17.2).

**Per-tenant cost accounting.** Every gateway call writes a row to `cyberos_meta.ai_call`:
```
id, tenant_id, request_id, persona_version, model, route_taken, latency_ms,
prompt_tokens, completion_tokens, cached_prompt_tokens,
estimated_cost_usd_micros, region, occurred_at, audit_entry_id
```
A nightly job aggregates into `cyberos_meta.ai_call_daily_rollup` per tenant per model. The OBS Compliance Cockpit reads from the rollup. A tenant exceeding its monthly cost ceiling triggers a `genie.notify` to the tenant administrator and (after 110% of ceiling) a hard cap; tools in `genie.notify` mode degrade to Haiku-only routing automatically.

**PII redaction.** Inbound prompts pass through a redaction filter before being sent to any external provider:
- Microsoft Presidio's analyser with the standard recognisers (EMAIL_ADDRESS, PHONE_NUMBER, US_SSN, IBAN, CREDIT_CARD, etc.).
- Custom Vietnamese-aware recognisers built in-house: `VN_CCCD` (12-digit citizen-ID, the post-2021 Decree-104 format), `VN_TAX_CODE` (10 or 13 digits with a checksum rule), `VN_BANK_ACCOUNT` (provider-pattern-aware), `VN_PHONE` (full coverage of Viettel, Mobifone, Vinaphone, Vietnamobile, Reddi prefixes), `VN_NAME` (best-effort against a curated list of common surnames).
- Inline replacement: detected entities are replaced with `<E:KIND:NN>` placeholders; the gateway maintains a per-request reverse map and re-inserts the original values into the response *only if the response contains the placeholder verbatim*. Otherwise the placeholder is left in place — the model sees the redacted form, the human sees the redacted form, no exfiltration path. This is the structural reason the gateway never lets BRAIN compensation/equity content reach an external provider unredacted (the ingestion-side denylist catches it earlier; redaction is the second line of defence).

For prompts that explicitly traffic in redactable values (e.g. an HR Member asking about a specific Member's onboarding), the calling module sets a per-request `redaction_mode: "permissive"` flag with an audit-log entry justifying the access, and the redaction engine narrows to high-precision recognisers only.

**Persona-version stamping.** The Genie/CUO persona is authored as Anthropic Skills directories under `~/.cyberos/skills/cuo/<role>/SKILL.md` (FR-GENIE-001 specifies the directory layout in detail). Each persona version is published to a small `cyberos_meta.persona_version` table with: `id`, `name` (e.g. `cuo-v0.4.2`), `signed_by_founder`, `signed_by_engineering_lead`, `published_at`, `system_prompt`. The gateway *prepends* the active persona's `system_prompt` to every request whose `persona: "cuo"` flag is set; the client cannot override the system prompt. Audit entries record the exact `persona_version` used, so the question "what persona produced this answer in March?" is answerable two years later by joining `audit.entry` with `persona_version`.

**Residency enforcement.** On every call the gateway resolves the tenant's primary residency region (read from `cyberos_meta.tenant.region`) and rejects any provider URL that does not match the residency. A Vietnamese tenant's prompt is never sent to `bedrock-runtime.us-east-2.amazonaws.com`; the gateway will instead 503 if the Singapore region is fully unhealthy. The audit row records the routed region.

**Latency budgets.** PRD §8.5.1 names the budgets for the canonical operations:
- Notify-class smart-reply (Haiku): p50 ≤ 600 ms, p95 ≤ 1.4 s.
- Question-class CUO answer (Sonnet, with BRAIN retrieval): p50 ≤ 2.5 s, p95 ≤ 6 s.
- Review-class long-form (Sonnet, with BRAIN retrieval + tool calls): p50 ≤ 8 s, p95 ≤ 20 s.
- Embedding generation (bge-m3 self-hosted): p50 ≤ 80 ms, p95 ≤ 200 ms.
- Reranking (bge-reranker-v2-m3 self-hosted): p50 ≤ 60 ms, p95 ≤ 150 ms per 20-document batch.

Each request carries a `latency_budget_ms` header; the gateway tracks elapsed time, fails fast if over budget, and prefers fallback only when fallback can complete inside the remaining budget.

**Per-tenant rate limiting.** Per tenant per minute and per hour ceilings, configurable per plan; default P0 internal-only ceilings: 600 calls/min, 30,000 calls/hour. Excess requests are queued for up to 2 seconds, then return 429 with `Retry-After`. The CUO ambient mode respects 429 by silently dropping the nudge rather than retrying.

**Circuit breakers.** Per provider per region: a 5xx rate over 5% in any 60-second window opens the breaker; the breaker stays open for 30 seconds, then half-opens with a single probe, then closes if the probe succeeds. The gateway emits `cyberos.{tenant}.ai.gateway.circuit_breaker.opened|closed` on NATS so OBS dashboards reflect breaker state.

**Self-hosted models.** A single GPU worker node (1× NVIDIA L4 or equivalent in Hetzner) runs three small models behind the gateway: `BAAI/bge-m3` for multilingual embeddings (Vietnamese + English native), `BAAI/bge-reranker-v2-m3` for reranking, and optional `Llama-3.1-8B-Instruct` quantised to AWQ-INT4 for the redaction-classifier role (extending Presidio with a learned recogniser for Vietnamese-specific entities Presidio does not natively cover). The models are served via vLLM (v0.7+); the gateway speaks vLLM's OpenAI-compatible API as one of LiteLLM's providers. Cost is the GPU node ($120-180/month at Hetzner spot pricing).

**Inbound API to consumers.**
```
POST /v1/chat/complete
Headers: Authorization: Bearer <member-or-agent-token>
         X-Tenant-Id: <tenant-uuid>           (validated against token's claims)
         X-Persona: cuo|none
         X-Latency-Budget-Ms: 6000
         X-Idempotency-Key: <uuid>            (optional; cached for 24h)
Body:    { messages: [...], model_tier: "fast"|"thinking"|"long-form",
           cache: "auto"|"never", tools?: [...] }
```

```
POST /v1/embed
Body:    { texts: [...], model: "bge-m3" }
```

```
POST /v1/rerank
Body:    { query: "...", documents: [...], model: "bge-reranker-v2-m3" }
```

The endpoints are private; only in-cluster services can reach them. External callers (Claude.ai, Cursor) talk to the MCP gateway, not the AI Gateway directly.

**Audit integration.** Every call writes an audit row in scope `ai.{tenant}` with payload including `persona_version`, `model`, `route_taken`, latency, token counts, cache hit/miss, redacted-entity count by kind. The audit row is the canonical evidence for EU AI Act Article 12 (logging) compliance.

## Alternatives Considered

- **Direct provider calls from each module.** Rejected: tenant residency, persona stamping, redaction, and audit would be re-implemented per module with predictable inconsistency. The PRD's compliance posture cannot survive that.
- **Cloudflare AI Gateway / Helicone / Portkey hosted.** Rejected at P0: data leaves the residency boundary; persona-version stamping would not be enforced. We may reconsider Cloudflare AI Gateway for the eu/us shards at P3 if the residency proof improves.
- **Fully self-hosted models (skip Bedrock entirely).** Rejected: Sonnet-class quality is required for the CUO persona's CEO/COO/CTO skill outputs; self-hosting Sonnet-equivalent models is out of budget.
- **OpenRouter / Together AI as routing layer.** Rejected: ZDR posture is harder to verify, and per-tenant cost accounting at our granularity is not exposed.

## Success Metrics

- **Primary metric.** S0-2 demo passes: the gateway answers a hardcoded "Hello, I am the Founder; brief me on the day" prompt with provider routing visible in the audit log; cache hit rate on a 1,000-call synthetic load > 30%; latency budgets enforced per the table above; the redaction filter masks a synthetic VN_CCCD in a synthetic prompt and the placeholder is preserved in the response.
- **Guardrail metric.** Monthly LLM spend ≤ $150 at internal scale (PRD §4.3 guardrail). Any month that exceeds is reviewed and the gateway-level cost-control switches (Haiku-only fallback, cache TTL extension) are enabled.
- **Compliance metric.** EU AI Act Article 50 transparency: every gateway response that flows to a UI surface carries a `persona_version` and `ai_disclosure_id`; UI surfaces render the disclosure chip. Lighthouse-style audits run weekly.

## Scope

**In-scope (S0-2).**
- `cyberos-ai-gateway` service with three replicas; in-cluster service exposure only.
- LiteLLM-based routing core; Bedrock Singapore primary; Anthropic ZDR + OpenAI ZDR fallbacks.
- Redis-backed prompt cache with the keying rules above.
- Per-tenant cost accounting with `ai_call` table + nightly rollup.
- Presidio-based redaction with VN_CCCD, VN_TAX_CODE, VN_BANK_ACCOUNT, VN_PHONE, VN_NAME custom recognisers.
- Persona-version stamping table and prepend logic.
- Residency enforcement.
- Latency-budget header enforcement with per-budget metrics.
- Per-tenant rate limits + circuit breakers per provider.
- bge-m3 + bge-reranker-v2-m3 served via vLLM on the GPU node.
- Audit integration in scope `ai.{tenant}`.
- Three internal endpoints: `/v1/chat/complete`, `/v1/embed`, `/v1/rerank`.

**Out-of-scope (deferred).**
- Llama-3.1-8B redaction classifier (P1; Presidio + custom rules ship in P0).
- Cloudflare AI Gateway in front (P3 reconsideration).
- Public API (P4 — covered by the PORTAL/Public APIs FR cluster).
- Multi-region active-active (P3; P0 is single-region per residency).

## Dependencies

- FR-INFRA-001 (Postgres, Redis, NATS, K8s, GPU node provisioned).
- FR-AUTH-001 (Member/agent tokens for the Authorization header).
- FR-AUTH-002 (audit log; the gateway is one of the highest-volume writers).
- AWS Bedrock account in `ap-southeast-1` (Singapore) and Anthropic + OpenAI ZDR contracts signed.
- Hetzner GPU offering: 1× NVIDIA L4 reserved.
- Compliance: PDPL Decree 13 (consent for processing personal data via external LLM provider — covered by tenant ToS); EU AI Act Article 50 (transparency); EU AI Act Articles 9–15 (high-risk AI systems requirements applied to the limited-risk surface as a forward-compatible posture).
- Locked decisions referenced: DEC-024..DEC-030.

## AI Risk Assessment

This feature emits AI-generated content that downstream modules will surface to natural persons. EU AI Act risk class: `limited` (Article 50 transparency obligation). The three required subsections follow.

### Data Sources

The AI Gateway does not train or fine-tune any model in P0. The models the gateway routes to are: (1) Bedrock-hosted Anthropic Claude Sonnet 4.6 and Haiku 4.5 — Anthropic's published training data with Anthropic's standard ZDR posture for Bedrock; (2) Anthropic API direct — same model line; (3) OpenAI API — GPT-4o and GPT-4o-mini under OpenAI's published policy with ZDR enabled per our enterprise contract. Self-hosted models: BAAI/bge-m3 and BAAI/bge-reranker-v2-m3 are open-weights from BAAI under the MIT licence; weights are mirrored to a private artefact store and signed; their training data provenance is BAAI's published release notes.

Per-tenant data: prompts and responses are not used to train any model. ZDR contracts at Anthropic and OpenAI prohibit retention beyond 30 days for abuse monitoring; Bedrock has no retention by default. The gateway logs prompts and responses *internally* to `cyberos_meta.ai_call` (with redacted PII) for audit, cost accounting, and quality regression. The internal log is per-tenant and never leaves the tenant's residency region.

Personal data implications: prompts may include personal data (Member names, CRM contact emails). The Vietnamese-aware redactor masks high-risk identifiers before they leave the residency boundary. The PDPL DPIA template (FR-CP-001 in batch-02) covers this flow with the standard "necessary for performance of the service" lawful basis.

### Human Oversight

This is the gateway, not the consumer surface; human oversight is enforced *at the consumer*. The gateway provides the substrate for it: every response carries `persona_version`, `model`, `ai_disclosure_id`. Consumer modules (CUO, EMAIL summarisation, KB Q&A) render the disclosure chip and route the response through the Notify / Question / Review interaction model (PRD §6.5). Specifically, no gateway-routed response auto-acts on irreversible operations; the destructive-tool human-confirmation rule is enforced at the MCP gateway (FR-MCP-001) on top of this gateway.

The gateway operator (the Engineering Lead) has a kill switch: a single config flag disables a model line within 60 seconds across the cluster. The kill switch is auditable. If a regression in a model line is observed (e.g. hallucinated citations rising above the regression threshold), the kill switch is the immediate response and the persona version is rolled back via `cyberos_meta.persona_version`.

### Failure Modes

- **Primary provider outage.** Failover to ZDR Anthropic, then ZDR OpenAI. If all three fail, the gateway returns 503 and consumer modules show "I cannot answer right now — please retry" rather than guessing.
- **Hallucinated citations or policy violations.** Detected by the persona-regression test suite that runs in CI on every persona version change; a regression rolls back the persona and pages the on-call.
- **PII leak through redaction gap.** Mitigated by the layered defence: BRAIN ingestion-side denylist (FR-BRAIN-002), gateway redaction here, EU AI Act DPIA process auditing the rule set quarterly. A confirmed leak (PII appears in an external provider's logs) is sev-0, triggers a 72-hour PDPL breach notification, and adds a recogniser to the redaction set.
- **Cost runaway.** Hard cap at 110% of the monthly ceiling forces Haiku-only mode; consumer modules degrade gracefully (no smart replies in CHAT, summary instead of full Q&A in CUO).
- **Latency budget breach.** Caller fails fast and the consumer surface shows the truncation explicitly (UI pattern: "Genie is taking longer than expected; you can keep working — I'll notify you when ready.").

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6 (drafted the proposed solution and the failure-mode list).
- **Scope:** Drafted the routing and redaction sections; reviewed and rewrote the latency budget section to align with PRD §8.5.1.
- **Human review:** `@stephen-cheng` reviewed the entire artefact; technical accuracy of the LiteLLM/vLLM integration to be re-verified by the Engineering Lead at PR-review time.
