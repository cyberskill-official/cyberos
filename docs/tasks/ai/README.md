# AI module — task index

_Generated 2026-05-17 — 23 tasks, 175 engineering-hours total._

## tasks

| Task | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-AI-001](TASK-AI-001-cost-ledger-precheck/spec.md) | MUST | 1 | 8 | AI Gateway cost-ledger pre-call check |
| [TASK-AI-002](TASK-AI-002-cost-ledger-postcall-reconcile/spec.md) | MUST | 1 | 6 | AI Gateway cost-ledger post-call reconcile |
| [TASK-AI-003](TASK-AI-003-memory-audit-bridge/spec.md) | MUST | 1 | 5 | memory audit-row bridge — canonical Writer for AI Gateway |
| [TASK-AI-004](TASK-AI-004-cost-hold-expiry-cleanup/spec.md) | MUST | 1 | 3 | Cost-hold expiry cleanup job — refund unsettled holds + emit audit |
| [TASK-AI-005](TASK-AI-005-tenant-policy-yaml-loader/spec.md) | MUST | 1 | 5 | Tenant-policy YAML loader — per-tenant cap + warn + override + residency |
| [TASK-AI-006](TASK-AI-006-model-alias-resolution/spec.md) | MUST | 2 | 6 | Model-alias resolution (chat.smart → bedrock:claude-3.5-sonnet) with per-tenant override |
| [TASK-AI-007](TASK-AI-007-provider-cost-table-loader/spec.md) | MUST | 2 | 4 | Provider cost-table loader — YAML-backed, hot-reloadable rate table |
| [TASK-AI-008](TASK-AI-008-multi-provider-router/spec.md) | MUST | 2 | 10 | LiteLLM-derived multi-provider router with retry + 30s failover SLA |
| [TASK-AI-009](TASK-AI-009-circuit-breaker/spec.md) | MUST | 2 | 6 | Circuit breaker per (provider, model) with half-open recovery probing |
| [TASK-AI-010](TASK-AI-010-streaming-sse/spec.md) | SHOULD | 2 | 8 | Streaming SSE end-to-end (token-by-token to client) |
| [TASK-AI-011](TASK-AI-011-presidio-pii-redaction/spec.md) | MUST | 3 | 6 | Presidio EN-base PII redaction in-flight (every prompt) |
| [TASK-AI-012](TASK-AI-012-vn-pii-plugin/spec.md) | MUST | 3 | 10 | VN-PII Presidio plugin (CCCD · MST · VN phone · NĐD · VN address · bank account) |
| [TASK-AI-013](TASK-AI-013-vn-pii-recall-gate/spec.md) | MUST | 3 | 8 | VN-PII recall ≥ 99% per-recognizer CI gate on 200-sample fixture |
| [TASK-AI-014](TASK-AI-014-persona-version-stamping/spec.md) | MUST | 3 | 8 | Persona-version system-prompt injection from memory memories/personas/<handle>.md |
| [TASK-AI-015](TASK-AI-015-zdr-enforcement/spec.md) | MUST | 3 | 6 | ZDR (Zero Data Retention) attestation table + enforcement when tenant policy requires |
| [TASK-AI-016](TASK-AI-016-residency-pinning/spec.md) | MUST | 4 | 8 | Tenant residency pinning (sg-1 / eu-1 / us-1 / vn-1) propagating to provider region selection |
| [TASK-AI-017](TASK-AI-017-per-tenant-cache/spec.md) | SHOULD | 4 | 8 | Per-tenant Redis response cache keyed by (tenant × redacted-prompt × model × persona); ≥30% hit-rate |
| [TASK-AI-018](TASK-AI-018-cache-cross-tenant-leak-test/spec.md) | MUST | 4 | 6 | Cross-tenant cache leak property-test (hard zero) — 200K random ops + 7 regression scenarios + adver |
| [TASK-AI-019](TASK-AI-019-bge-m3-embeddings/spec.md) | SHOULD | 4 | 12 | Self-hosted BGE-M3 embeddings (single L4 GPU sidecar) + ONNX-CPU fallback + adaptive batching |
| [TASK-AI-020](TASK-AI-020-bge-rerank/spec.md) | COULD | 4 | 8 | BGE-reranker-v2-m3 cross-encoder for KB reranking (per-region sidecar; CPU fallback) |
| [TASK-AI-021](TASK-AI-021-operator-cli/spec.md) | MUST | 5 | 14 | cyberos-ai operator CLI (usage · models · policy · failover · invoice · breaker · expiry · memory) wi |
| [TASK-AI-022](TASK-AI-022-otel-trace-emission/spec.md) | MUST | 5 | 8 | OpenTelemetry trace + span emission for every call (caller → router → provider → response) with W3C  |
| [TASK-AI-104](TASK-AI-104-vn-provider-integration/spec.md) | SHOULD | 1 | 12 | AI VN provider integration — Viettel Cloud + FPT Cloud as Vn1-residency LLM/embedding providers with |
| [TASK-AI-105](TASK-AI-105-local-model-providers/spec.md) | MUST | 1 | 10 | Local + external model providers (LM Studio + Ollama, no-key local) |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: TASK-AI-006→TASK-AUTH-004
- **OBS**: TASK-AI-022→TASK-OBS-001

**This module is depended on by:**

- **memory**: TASK-MEMORY-101→TASK-AI-019, TASK-MEMORY-111→TASK-AI-012
- **CUO**: TASK-CUO-101→TASK-AI-008
- **EMAIL**: TASK-EMAIL-005→TASK-AI-003
- **KB**: TASK-KB-005→TASK-AI-019, TASK-KB-006→TASK-AI-020
- **OBS**: TASK-OBS-004→TASK-AI-022
- **PROJ**: TASK-PROJ-002→TASK-AI-003
- **SKILL**: TASK-SKILL-101→TASK-AI-003
- **TEN**: TASK-TEN-004→TASK-AI-001, TASK-TEN-103→TASK-AI-016

---

## Historical slice arc (P0 AI Gateway, audited 2026-05-15)

| Slice | tasks | Effort | Theme |
|---|---:|---:|---|
| 1 | TASK-AI-001..005 | 27h | Cost-ledger + memory bridge + policy loader |
| 2 | TASK-AI-006..010 | 34h | Multi-provider router + circuit breaker + streaming |
| 3 | TASK-AI-011..015 | 28h | PII redaction (Presidio + VN plugin) + persona stamping + ZDR |
| 4 | TASK-AI-016..020 | 27h | Residency + cache + cross-leak property test + BGE embeddings/rerank |
| 5 | TASK-AI-021..022 | 12h | Operator CLI + OTel emission |

**P0 AI Gateway closed:** 22 tasks, 128h, all 10/10. **+1 P3 add (TASK-AI-104, 12h)** — Viettel + FPT Cloud integration for `Vn1` residency closure.

**Within-slice build order:**
```
slice 1: TASK-AI-005 → TASK-AI-007 → TASK-AI-003 → TASK-AI-001 → TASK-AI-002 → TASK-AI-004
slice 2: TASK-AI-006 → TASK-AI-008 → TASK-AI-009 → TASK-AI-010
slice 3: TASK-AI-011 → TASK-AI-012 → TASK-AI-013 → TASK-AI-014 → TASK-AI-015
slice 4: TASK-AI-016 → TASK-AI-017 → TASK-AI-018 → TASK-AI-019 → TASK-AI-020
slice 5: TASK-AI-022 → TASK-AI-021
P3:      TASK-AI-104 (depends on TASK-AI-016)
```

**Cross-task consistency anchors:**
- TenantPolicy schema (TASK-AI-005) consumed by 8 other AI tasks
- ResolvedModel (TASK-AI-006) consumed by TASK-AI-008, TASK-AI-015, TASK-AI-016, TASK-AI-104
- memory_writer::emit (TASK-AI-003) consumed by TASK-AI-001, TASK-AI-002, TASK-AI-004, TASK-AI-014
- cost_table::lookup (TASK-AI-007) consumed by TASK-AI-001, TASK-AI-002, TASK-AI-006

---

_See `../IMPLEMENTATION_ORDER.md` for the full topological build sequence._