# AI module — feature request index

_Generated 2026-05-17 — 23 FRs, 175 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-AI-001](FR-AI-001-cost-ledger-precheck.md) | MUST | 1 | 8 | AI Gateway cost-ledger pre-call check |
| [FR-AI-002](FR-AI-002-cost-ledger-postcall-reconcile.md) | MUST | 1 | 6 | AI Gateway cost-ledger post-call reconcile |
| [FR-AI-003](FR-AI-003-memory-audit-bridge.md) | MUST | 1 | 5 | memory audit-row bridge — canonical Writer for AI Gateway |
| [FR-AI-004](FR-AI-004-cost-hold-expiry-cleanup.md) | MUST | 1 | 3 | Cost-hold expiry cleanup job — refund unsettled holds + emit audit |
| [FR-AI-005](FR-AI-005-tenant-policy-yaml-loader.md) | MUST | 1 | 5 | Tenant-policy YAML loader — per-tenant cap + warn + override + residency |
| [FR-AI-006](FR-AI-006-model-alias-resolution.md) | MUST | 2 | 6 | Model-alias resolution (chat.smart → bedrock:claude-3.5-sonnet) with per-tenant override |
| [FR-AI-007](FR-AI-007-provider-cost-table-loader.md) | MUST | 2 | 4 | Provider cost-table loader — YAML-backed, hot-reloadable rate table |
| [FR-AI-008](FR-AI-008-multi-provider-router.md) | MUST | 2 | 10 | LiteLLM-derived multi-provider router with retry + 30s failover SLA |
| [FR-AI-009](FR-AI-009-circuit-breaker.md) | MUST | 2 | 6 | Circuit breaker per (provider, model) with half-open recovery probing |
| [FR-AI-010](FR-AI-010-streaming-sse.md) | SHOULD | 2 | 8 | Streaming SSE end-to-end (token-by-token to client) |
| [FR-AI-011](FR-AI-011-presidio-pii-redaction.md) | MUST | 3 | 6 | Presidio EN-base PII redaction in-flight (every prompt) |
| [FR-AI-012](FR-AI-012-vn-pii-plugin.md) | MUST | 3 | 10 | VN-PII Presidio plugin (CCCD · MST · VN phone · NĐD · VN address · bank account) |
| [FR-AI-013](FR-AI-013-vn-pii-recall-gate.md) | MUST | 3 | 8 | VN-PII recall ≥ 99% per-recognizer CI gate on 200-sample fixture |
| [FR-AI-014](FR-AI-014-persona-version-stamping.md) | MUST | 3 | 8 | Persona-version system-prompt injection from memory memories/personas/<handle>.md |
| [FR-AI-015](FR-AI-015-zdr-enforcement.md) | MUST | 3 | 6 | ZDR (Zero Data Retention) attestation table + enforcement when tenant policy requires |
| [FR-AI-016](FR-AI-016-residency-pinning.md) | MUST | 4 | 8 | Tenant residency pinning (sg-1 / eu-1 / us-1 / vn-1) propagating to provider region selection |
| [FR-AI-017](FR-AI-017-per-tenant-cache.md) | SHOULD | 4 | 8 | Per-tenant Redis response cache keyed by (tenant × redacted-prompt × model × persona); ≥30% hit-rate |
| [FR-AI-018](FR-AI-018-cache-cross-tenant-leak-test.md) | MUST | 4 | 6 | Cross-tenant cache leak property-test (hard zero) — 200K random ops + 7 regression scenarios + adver |
| [FR-AI-019](FR-AI-019-bge-m3-embeddings.md) | SHOULD | 4 | 12 | Self-hosted BGE-M3 embeddings (single L4 GPU sidecar) + ONNX-CPU fallback + adaptive batching |
| [FR-AI-020](FR-AI-020-bge-rerank.md) | COULD | 4 | 8 | BGE-reranker-v2-m3 cross-encoder for KB reranking (per-region sidecar; CPU fallback) |
| [FR-AI-021](FR-AI-021-operator-cli.md) | MUST | 5 | 14 | cyberos-ai operator CLI (usage · models · policy · failover · invoice · breaker · expiry · memory) wi |
| [FR-AI-022](FR-AI-022-otel-trace-emission.md) | MUST | 5 | 8 | OpenTelemetry trace + span emission for every call (caller → router → provider → response) with W3C  |
| [FR-AI-104](FR-AI-104-vn-provider-integration.md) | SHOULD | 1 | 12 | AI VN provider integration — Viettel Cloud + FPT Cloud as Vn1-residency LLM/embedding providers with |
| [FR-AI-105](FR-AI-105-local-model-providers.md) | MUST | 1 | 10 | Local + external model providers (LM Studio + Ollama, no-key local) |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: FR-AI-006→FR-AUTH-004
- **OBS**: FR-AI-022→FR-OBS-001

**This module is depended on by:**

- **memory**: FR-MEMORY-101→FR-AI-019, FR-MEMORY-111→FR-AI-012
- **CUO**: FR-CUO-101→FR-AI-008
- **EMAIL**: FR-EMAIL-005→FR-AI-003
- **KB**: FR-KB-005→FR-AI-019, FR-KB-006→FR-AI-020
- **OBS**: FR-OBS-004→FR-AI-022
- **PROJ**: FR-PROJ-002→FR-AI-003
- **SKILL**: FR-SKILL-101→FR-AI-003
- **TEN**: FR-TEN-004→FR-AI-001, FR-TEN-103→FR-AI-016

---

## Historical slice arc (P0 AI Gateway, audited 2026-05-15)

| Slice | FRs | Effort | Theme |
|---|---:|---:|---|
| 1 | FR-AI-001..005 | 27h | Cost-ledger + memory bridge + policy loader |
| 2 | FR-AI-006..010 | 34h | Multi-provider router + circuit breaker + streaming |
| 3 | FR-AI-011..015 | 28h | PII redaction (Presidio + VN plugin) + persona stamping + ZDR |
| 4 | FR-AI-016..020 | 27h | Residency + cache + cross-leak property test + BGE embeddings/rerank |
| 5 | FR-AI-021..022 | 12h | Operator CLI + OTel emission |

**P0 AI Gateway closed:** 22 FRs, 128h, all 10/10. **+1 P3 add (FR-AI-104, 12h)** — Viettel + FPT Cloud integration for `Vn1` residency closure.

**Within-slice build order:**
```
slice 1: FR-AI-005 → FR-AI-007 → FR-AI-003 → FR-AI-001 → FR-AI-002 → FR-AI-004
slice 2: FR-AI-006 → FR-AI-008 → FR-AI-009 → FR-AI-010
slice 3: FR-AI-011 → FR-AI-012 → FR-AI-013 → FR-AI-014 → FR-AI-015
slice 4: FR-AI-016 → FR-AI-017 → FR-AI-018 → FR-AI-019 → FR-AI-020
slice 5: FR-AI-022 → FR-AI-021
P3:      FR-AI-104 (depends on FR-AI-016)
```

**Cross-FR consistency anchors:**
- TenantPolicy schema (FR-AI-005) consumed by 8 other AI FRs
- ResolvedModel (FR-AI-006) consumed by FR-AI-008, FR-AI-015, FR-AI-016, FR-AI-104
- memory_writer::emit (FR-AI-003) consumed by FR-AI-001, FR-AI-002, FR-AI-004, FR-AI-014
- cost_table::lookup (FR-AI-007) consumed by FR-AI-001, FR-AI-002, FR-AI-006

---

_See `../IMPLEMENTATION_ORDER.md` for the full topological build sequence._