---
template: task@1
id: TASK-AI-105
title: "Local + external model providers (LM Studio + Ollama, no-key local)"
author: "@stephen"
department: engineering
status: draft
priority: p1
created_at: "2026-06-22T10:00:00+07:00"
ai_authorship: assisted
feature_type: integration
eu_ai_act_risk_class: limited
target_release: 2026-Q3
client_visible: false
module: ai
new_files:
  - services/ai-gateway/src/router/local_openai.rs
  - services/ai-gateway/src/router/ollama.rs
  - services/ai-gateway/tests/local_provider_roundtrip_test.rs
  - services/ai-gateway/tests/local_provider_unreachable_test.rs
modified_files:
  - services/ai-gateway/src/policy/schema.rs
  - services/ai-gateway/src/router/mod.rs
  - services/ai-gateway/src/router/failover.rs
  - services/ai-gateway/src/streaming/mod.rs
  - services/ai-gateway/src/cost_table/loader.rs
  - services/ai-gateway/src/cost_reconcile.rs
  - services/ai-gateway/src/zdr.rs
  - services/ai-gateway/src/server.rs
depends_on: [TASK-AI-005, TASK-AI-006, TASK-AI-007, TASK-AI-008, TASK-AI-015, TASK-AI-003]
---

# Task

> Turn Your Will Into Real.

## Summary

CyberOS should route real model calls to local, no-key inference servers. Stephen runs models locally (LM Studio today, Ollama earlier), and both must work. This adds two provider adapters to the existing TASK-AI-008 router: one for LM Studio and any OpenAI-compatible local server (POST to /v1/chat/completions, default http://localhost:1234), and one for Ollama-native (POST to /api/chat, default http://localhost:11434). Each is selectable per tenant through the existing TASK-AI-006 alias map and TASK-AI-008 fallback chain, holds no API key, and fails closed when the local server is down. The same change flips the HTTP serving path from the in-repo EchoBackend to the real router, so a tenant whose primary provider is local actually gets a model response rather than an echo.

## Problem

The gateway already has the whole control plane: cost ledger (TASK-AI-001/002), tenant policy (TASK-AI-005), alias resolution (TASK-AI-006), the multi-provider router with retry and failover (TASK-AI-008), PII redaction (TASK-AI-011/012), ZDR (TASK-AI-015), residency (TASK-AI-016), and per-tenant cache (TASK-AI-017). But none of it drives a real model on the local-dev or self-hosted path. The serving endpoint answers through EchoBackend, and the Provider trait's adapters (Anthropic, OpenAI, Bedrock) are stubs that return an error. So today a chat call cannot reach a model without cloud keys, and even with keys the server does not call the router.

Stephen's working setup is a local model server: LM Studio right now, Ollama before that. LM Studio exposes an OpenAI-compatible API; Ollama exposes its own /api/chat shape. CyberOS needs to talk to both, with no key, so the local path proves end to end and self-hosted deployments have a zero-cost inference option.

## Proposed Solution

Two real adapters behind the existing `Provider` trait, plus the serving flip. User-visible behaviour: a tenant sets `primary_provider: { kind: local-openai }` or `{ kind: ollama }` in its policy, maps its aliases to local model ids, and chat calls return real completions from the local server. No key is configured. If the server is down, the call fails with a clear error instead of an echo.

### Section 1 - normative requirements (BCP-14)

1. The gateway MUST ship `Provider::LocalOpenai { model_alias_map }`: POST `{endpoint}/v1/chat/completions` with body `{model, messages, stream:false, temperature?, max_tokens?}`, parsing `choices[0].message.content`, `usage.prompt_tokens`, and `usage.completion_tokens`. This covers LM Studio and any OpenAI-compatible local runtime (llama.cpp server, vLLM, text-generation-webui).

2. The gateway MUST ship `Provider::Ollama { model_alias_map }`: POST `{endpoint}/api/chat` with body `{model, messages, stream:false, options?}`, parsing `message.content`, `prompt_eval_count`, and `eval_count`.

3. Endpoints MUST come from environment, not from tenant YAML: `LMSTUDIO_ENDPOINT` (default `http://localhost:1234`) and `OLLAMA_ENDPOINT` (default `http://localhost:11434`). One local server serves a whole deployment, so the endpoint is deployment-level, not tenant-level.

4. Local providers MUST require no API key, and MUST fail closed: if the local server is unreachable or returns a non-2xx, the adapter MUST return a `RouterError` (never a fabricated or echoed completion). The router's existing retry and failover then apply.

5. The provider taxonomy MUST register `ProviderKind::LocalOpenai` and `ProviderKind::Ollama` in every match site: the config `Provider` enum (kind/region/model_for_alias), the cost-table provider parser, the cost-reconcile parser, the ZDR parser, and the streaming-support table. Streaming-support MUST report false for both until a streaming adapter lands.

6. The HTTP serving path MUST call the real router (`router::call_provider`) for tenants whose resolved provider is a real adapter, instead of EchoBackend. EchoBackend MAY remain available behind an explicit test-only flag, but MUST NOT be the default serving path.

7. Local providers MUST be treated as zero-cost in the cost ledger (no per-token USD), while still recording token usage and emitting the post-call audit row. A missing cost row for a local provider MUST NOT block the call.

8. Both local providers MUST be selectable concurrently: a tenant MAY set one as primary and the other (or a cloud provider) in the fallback chain, and TASK-AI-008 failover MUST work across the mix.

9. The adapters MUST emit memory audit kinds `ai.local_provider_invoked` and `ai.local_provider_unreachable` through the TASK-AI-003 bridge, carrying tenant_id, provider kind, model, and outcome (prompts already hashed at the TASK-AI-006 layer).

10. The request-build and response-parse functions MUST be pure and unit-tested without a live server; the live round trip is covered by an owner-run integration test gated on a reachable local server.

## Alternatives Considered

Reuse the cloud OpenAI adapter pointed at localhost. Rejected: it covers LM Studio (OpenAI-compatible) but not Ollama-native (/api/chat is a different shape), and the cloud adapter assumes an API key and cloud base URL. The user needs both dialects and a no-key path.

Put the endpoint and dialect in the tenant policy YAML (one generic HTTP provider with a `url` + `dialect` field). Rejected: infrastructure endpoints are a deployment concern, not a per-tenant one. Tenant policy stays about caps, residency, and aliases; the local server URL belongs in env. It also widens the policy schema's trust surface.

Keep EchoBackend and add a separate "local test" endpoint. Rejected: it leaves the real router unexercised in production and forks the serving path. Flipping the dispatch once is cleaner and exercises retry, failover, cost, and audit on the real path.

## Success Metrics

Primary metric - local round-trip success rate.
- Definition: fraction of chat calls routed to a local provider that return a real (non-echo) completion while the local server is up.
- Baseline: 0% (no local adapter exists; serving path is EchoBackend).
- Target: at least 99% when the configured local server is reachable.
- Measurement method: integration test plus the `ai.local_provider_invoked` audit count over total local-routed calls.
- Source: memory audit rows (TASK-AI-003) and the gateway request log.

Guardrail metric - key and cross-tenant leakage.
- Definition: number of incidents where a local provider call carries or exposes an API key, or a response crosses tenants.
- Baseline: not applicable (no local path today).
- Target: zero. Local providers hold no key by construction, and TASK-AI-017/018 cache isolation still applies.
- Measurement method: the TASK-AI-018 cross-tenant leak property test plus a static check that local adapters read no key material.
- Source: TASK-AI-018 test output and code review.

## Scope

In scope: the LocalOpenai and Ollama adapters, env-driven endpoints, the no-key fail-closed contract, the provider-taxonomy registrations, the EchoBackend-to-router serving flip, zero-cost ledger handling for local providers, the two audit kinds, and pure unit tests plus one owner-run integration test.

### Out of scope

- Streaming for local providers (SSE token-by-token). Deferred to a follow-up; the streaming-support table returns false for now.
- Local embeddings (Ollama /api/embeddings, LM Studio /v1/embeddings). The embedding path stays on TASK-AI-019 BGE; local embeddings are a separate FR.
- Full KMS wiring of cloud-provider keys (Anthropic, OpenAI, Bedrock). This FR specs the seam and keeps those adapters on the same trait, but the secrets-management implementation is its own FR and stays behind the prohibited-secrets boundary.
- GPU or model lifecycle management of the local server itself (loading, unloading, quantization).

## Dependencies

- TASK-AI-008 multi-provider router - the `Provider` trait, retry, and failover that the adapters plug into.
- TASK-AI-006 model-alias resolution - maps the tenant alias to a local model id and yields the `ResolvedModel` the router consumes.
- TASK-AI-005 tenant-policy loader - the policy schema gains the two local provider variants.
- TASK-AI-007 provider cost-table loader - zero-cost rows for local providers, and the loader must tolerate their absence.
- TASK-AI-015 ZDR enforcement - local providers are inherently zero-retention; the ZDR parser learns the new kinds.
- TASK-AI-003 memory audit bridge - the two new audit kinds.
- Cross-cutting: the server-dispatch flip touches `services/ai-gateway/src/server.rs` (the HTTP handler), which is the one architecturally significant change in this FR.

## AI Risk Assessment

### Data sources

No training or fine-tuning happens here. The adapters forward already-redacted prompts (PII stripped at TASK-AI-011/012 before the router) to a local model the operator controls. For local providers, prompt data never leaves the deployment host, which is the strongest residency and retention posture available.

### Human oversight

Provider selection is set by the tenant operator in policy YAML and reviewed like any policy change. The serving flip is gated by the test suite and the CAF gate. A human (Stephen) approves the policy that points a tenant at a local model. There is no autonomous provider switching beyond the declared fallback chain.

### Failure modes

If the local server is down, the adapter fails closed and the router falls over to the next provider in the chain, or returns a terminal error if none remain; the user sees an error, never an echo or a fabricated answer. If the local server returns malformed JSON, the parse returns a `RouterError::InvalidResponse`. If a local model id is misconfigured, the server's own error surfaces through the adapter. None of these can produce a silent wrong answer.

## AI Authorship Disclosure

- Tools used: Claude (Cowork), authoring this FR from Stephen's capability request and the existing TASK-AI-008 router code.
- Scope: full draft of this specification, including the normative clauses and acceptance criteria. The LocalOpenai and Ollama adapter code is authored in the same session.
- Human review: Stephen reviews and approves before status moves past draft; the paired audit (TASK-AI-105.audit.md) and the CAF gate validate before merge.
