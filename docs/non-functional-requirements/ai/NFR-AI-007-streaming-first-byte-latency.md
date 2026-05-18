---
id: NFR-AI-007
title: "AI Gateway streaming first-byte latency — p95 < 2s for non-cached completions"
module: AI
category: performance
priority: MUST
verification: T
phase: P0
slo: "p95 time-to-first-byte (TTFB) < 2s for streaming /v1/chat/completions, cache-miss path"
owner: CTO
created: 2026-05-18
related_frs: [FR-AI-010, FR-AI-008]
---

## §1 — Statement (BCP-14 normative)

1. For streaming chat-completion requests on the cache-miss path, the AI Gateway **MUST** emit the first SSE event to the caller at **p95 < 2s** and **p99 < 4s** measured from inbound HTTP request received to first SSE data line written to the response stream.
2. The TTFB budget **MUST** include all gateway-internal work: JWT verify, RBAC check, cost-ledger admit (NFR-AI-002), PII redaction (NFR-AI-004), provider selection, and upstream connection setup. It explicitly excludes upstream model latency beyond first-token return.
3. Cache-hit responses (NFR-AI-006) **MUST** emit first byte at **p95 < 100ms** — these don't traverse the upstream provider.
4. The gateway **MUST NOT** buffer streaming responses internally beyond what is required for SSE event framing (one event ≤ 4KB). Buffering full completions would inflate TTFB.
5. On TTFB breach, the request **MUST** complete normally — the SLO is a measurement target, not a deadline that aborts the request.

## §2 — Why this constraint

User-perceived "the AI is thinking" latency is dominated by TTFB. Beyond ~2s, users start second-guessing and clicking refresh. The 2s ceiling preserves the platform's UX assertion that LLM responses feel "live, not batched." Cache-hit at 100ms is the floor we want to drive caller experience toward (cached prompts feel instant). Importantly, this SLO measures **gateway overhead** plus **upstream-first-token** — slow upstreams that themselves take > 2s to first token are a provider problem, surfaced via this metric and addressable via failover (NFR-AI-001).

## §3 — Measurement

- Histogram `ai_gateway_streaming_ttfb_seconds{route, cache_hit, model_alias}` emitted by `services/ai-gateway/src/streaming/sse.rs`. Buckets: 0.05, 0.1, 0.25, 0.5, 1, 2, 4, 8.
- Cache-miss subset p95 alarm at > 2.0s; cache-hit subset p95 alarm at > 0.1s.
- Reported in the OBS `ai-gateway-slo` dashboard; the host shell surfaces the metric as the "AI response feel" tile.

## §4 — Verification

- Integration test `services/ai-gateway/tests/streaming_ttfb_test.rs` (T) drives 200 chat-completion requests against the mock provider; asserts p95 < 2s cache-miss, p95 < 0.1s cache-hit. Runs in CI.
- Load test `deploy/loadtest/ai-gateway-streaming.k6.js` (T) — pre-release — 1000 concurrent streaming clients for 5 minutes; asserts p95 < 2s holds at concurrency.

## §5 — Failure handling

- p95 > 2s cache-miss for 10 minutes → sev-3 alert; on-call inspects which provider/route is regressing.
- p99 > 4s cache-miss → sev-2; consider whether failover (NFR-AI-001) is needed.
- p95 cache-hit > 100ms → sev-3; Redis health-check; cache may be degraded.

---

*End of NFR-AI-007.*
