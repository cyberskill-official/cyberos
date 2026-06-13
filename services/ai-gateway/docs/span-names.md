# AI Gateway Span Names

Canonical FR-AI-022 span names:

| Span | Kind | Required attributes |
|---|---|---|
| `ai_gateway.chat_completion` | server | `ai_gateway.tenant_id`, `ai_gateway.model_alias`, `ai_gateway.agent_persona`, `ai_gateway.request_id`, `ai_gateway.stream`, `ai_gateway.outcome` |
| `ai_gateway.embed` | server | `ai_gateway.tenant_id`, `ai_gateway.model_alias`, `ai_gateway.request_id`, `ai_gateway.region`, `ai_gateway.stream`, `ai_gateway.outcome` |
| `ai_gateway.rerank` | server | `ai_gateway.tenant_id`, `ai_gateway.model_alias`, `ai_gateway.request_id`, `ai_gateway.stream`, `ai_gateway.outcome` |
| `ai_gateway.precheck` | internal | `ai_gateway.tenant_id`, `ai_gateway.model_alias`, `ai_gateway.idempotency_key`, `ai_gateway.estimated_usd`, `ai_gateway.outcome` |
| `ai_gateway.alias_resolve` | internal | `ai_gateway.model_alias`, `ai_gateway.provider`, `ai_gateway.model`, `ai_gateway.region`, `ai_gateway.outcome` |
| `ai_gateway.persona_load` | internal | `ai_gateway.tenant_id`, `ai_gateway.agent_persona`, `ai_gateway.request_id`, `ai_gateway.outcome` |
| `ai_gateway.zdr_check` | internal | `ai_gateway.tenant_id`, `ai_gateway.model_alias`, `ai_gateway.outcome` |
| `ai_gateway.residency_check` | internal | `ai_gateway.tenant_id`, `ai_gateway.model_alias`, `ai_gateway.region`, `ai_gateway.outcome` |
| `ai_gateway.cache_lookup` | internal | `ai_gateway.cache_state`, `ai_gateway.cache_key_hash16`, `ai_gateway.outcome` |
| `ai_gateway.redact` | internal | `ai_gateway.tenant_id`, `ai_gateway.outcome` |
| `ai_gateway.provider_call` | client | `ai_gateway.provider`, `ai_gateway.model`, `ai_gateway.attempt_num`, `ai_gateway.fallback_position`, `ai_gateway.status_code`, `ai_gateway.retried`, `ai_gateway.outcome` |
| `ai_gateway.reconcile` | internal | `ai_gateway.tenant_id`, `ai_gateway.idempotency_key`, `ai_gateway.actual_usd`, `ai_gateway.outcome` |

Provider retry boundaries are events named `retry.attempt`, with attributes
`retry.attempt`, `retry.backoff_ms`, and `retry.prior_status_code`.

Attribute payloads must stay PII-safe: counts, enum labels, tenant-level ids,
UUID-shaped request ids, provider/model identifiers, HTTP status codes, and
hash16 cache keys only. Prompt text, response text, user email, phone, address,
government id, and raw sidecar/provider error bodies are not valid span
attributes.
