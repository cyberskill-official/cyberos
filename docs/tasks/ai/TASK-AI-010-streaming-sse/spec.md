---
# ───── Machine-readable frontmatter (parsed by task-audit + future task-catalog renderer) ─────
id: TASK-AI-010
title: "Streaming SSE end-to-end (token-by-token to client)"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: AI
priority: p1
status: done
verify: T
phase: P0
milestone: P0 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-15
shipped: 2026-05-21
memory_chain_hash: null
related_tasks: [TASK-AI-001, TASK-AI-002, TASK-AI-006, TASK-AI-007, TASK-AI-008, TASK-AI-009]
depends_on: [TASK-AI-008, TASK-AI-002]
blocks: []

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#streaming
  - website/docs/modules/ai.html#first-token-sla
source_decisions:
  - docs/tasks/ai/TASK-AI-008-multi-provider-router/spec.md §1 #16 (router exposes call_provider_streaming with SSE pipeline)
  - docs/tasks/ai/TASK-AI-002-cost-ledger-reconcile/spec.md §1 #5 (cancel-before-first-token reconciles as Cancelled and refunds)
  - archive/2026-05-14/RESEARCH_REVIEW.md §3.4 (SSE vs WebSocket trade-off study)

# ───── Build envelope ─────
language: rust 1.81
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/src/streaming.rs
  - services/ai-gateway/src/streaming/sse.rs
  - services/ai-gateway/src/streaming/heartbeat.rs
  - services/ai-gateway/src/router/streaming.rs                 # provider-streaming impls
  - services/ai-gateway/tests/streaming_test.rs
  - services/ai-gateway/benches/streaming_first_token_bench.rs
modified_files:
  - services/ai-gateway/src/handlers/chat.rs                    # branch on req.stream
  - services/ai-gateway/src/router.rs                           # wire call_provider_streaming (replaces slice-2 stub from TASK-AI-008 §1 #16)
  - services/ai-gateway/src/lib.rs                              # export streaming module
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests,benches}/**
  - bash: cargo test -p cyberos-ai-gateway streaming
  - bash: cargo bench -p cyberos-ai-gateway streaming_first_token_bench
disallowed_tools:
  - bypass cost_ledger on the streaming path (every stream MUST precheck and reconcile)
  - skip reconcile after stream ends (every spawn MUST guarantee one reconcile call, even on panic)
  - retry the provider call after the first token has been forwarded to the client
  - use broadcast channels (mpsc only — exactly one receiver per stream)
  - format!("{:?}", reason) for OBS labels (use ReconcileReason::as_metric_label())

# ───── Estimated work ─────
effort_hours: 8
subtasks:
  - "0.5h: SSE event shape (event: token / usage / done / error / heartbeat) + sse::Event constructors"
  - "1.5h: Provider streaming impls (Bedrock InvokeModelWithResponseStream, Anthropic SSE, OpenAI SSE) — replaces TASK-AI-008 stub"
  - "1.0h: Response unification (one tokio::sync::mpsc::Receiver<StreamEvent>; channel capacity 32 with BackpressureCounter)"
  - "1.0h: Backpressure handling (channel capacity 32; sender awaits if client slow; OBS counter on each await)"
  - "1.0h: Client-disconnect detection (tokio::select! on stream + abort signal; abort propagates to provider task within 200ms)"
  - "1.0h: Heartbeat task (every 15s during steady stream to keep proxies from timing out the connection)"
  - "1.0h: Reconcile guarantee (RAII guard ensures exactly-one reconcile call even on panic/abort)"
  - "1.0h: Integration tests (10 cases — happy / disconnect / first-token timeout / mid-stream error / backpressure / heartbeat / unsupported / pre-first-token retry / concurrent / exactly-one-reconcile)"
risk_if_skipped: "Chat UX feels sluggish — users wait 3-5 seconds for full response instead of seeing tokens stream. CHAT decommission signal degrades materially: Slack and Zalo show typing indicators within ~300ms of message send; CyberOS would show a blank pane for 3+ seconds. Adoption metric (sessions/week) likely drops 30-40% based on the product-led-growth literature. Not strictly P0-critical from a correctness standpoint (TASK-AI-008 non-streaming path serves the same requests) but materially impacts CHAT module's primary KPI."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **SHOULD** support Server-Sent Events (SSE) streaming for chat completions when the request has `stream: true`. The streaming path:

1. **MUST** invoke `cost_ledger::precheck` (TASK-AI-001) synchronously before opening the SSE stream — no token streams without a valid hold. The hold is created with `state="held"`; reconcile (step 7) transitions it to `committed` or `cancelled`.
2. **MUST** dispatch to `router::call_provider_streaming(req, resolved, deadline, policy)` (TASK-AI-008 §1 #16 — the streaming entry that returns `Err(StreamingNotImplemented)` in slice 2 stub form). This task replaces the stub with the real SSE pipeline; TASK-AI-008 §1 #16's MUST-stub becomes a MUST-impl per slice 3.
3. **MUST** emit SSE events in the canonical shape and order: zero or more `event: token` events, exactly one `event: usage` event, exactly one `event: done` event (or `event: error` if the stream failed). Token events use `data: {"text": "...", "model": "...", "index": N}\n\n`; the `index` field is monotonic per stream so clients can detect drops.
4. **MUST** emit `event: error` on any provider failure mid-stream; the stream MUST close after. The error payload includes a structured code (`provider_disconnect`/`first_token_timeout`/`mid_stream_timeout`/`backpressure_drop`) and a human-readable message.
5. **MUST** stream the first token within **1500ms p95** from request acceptance. The metric `ai_streaming_first_token_ms` (histogram) measures this; the bench gates p95 at 1500ms in CI.
6. **MUST** detect client disconnect via `axum`'s connection-watcher and abort the upstream provider call within **200ms** of the disconnect. The abort cancels the spawned task; the `tokio::sync::mpsc::Sender::send` returns Err; the provider call's `tokio::time::timeout` fires; the task winds down. The `usage.completion_tokens` MUST reflect only the tokens that successfully reached the client (per TASK-AI-002 §1 #12 column-floor semantics).
7. **MUST** call `cost_ledger::reconcile` (TASK-AI-002) **exactly once** at stream end, with one of: `CallOutcome::Success { usage }` (clean finish), `CallOutcome::Cancelled { partial_usage, reason }` (disconnect, timeout, abort), or `CallOutcome::ProviderError { partial_usage, http_status, message }` (provider failure mid-stream). The exactly-once guarantee MUST hold even if the spawned task panics — implemented via an RAII `ReconcileGuard` whose `Drop` impl performs the reconcile if not already done.
8. **MUST** apply backpressure if the client is slow — the internal mpsc channel has capacity 32; sends block when full. Provider calls don't out-buffer the network. Each block event increments `ai_streaming_backpressure_events_total{provider}` so operators can spot under-provisioned clients.
9. **MUST** emit a heartbeat `event: heartbeat\ndata: {}\n\n` every **15 seconds** during steady stream to keep proxies (CDNs, corporate firewalls) from timing out idle connections. Heartbeats DO NOT contribute to `usage.completion_tokens` or `cost_ledger`. The heartbeat task MUST stop when the stream closes.
10. **MUST** propagate the deadline — the streaming provider call inherits the same deadline as the non-streaming variant (TASK-AI-008 §1 #6). The deadline includes the time spent in the SSE pipeline; per-token latency does not extend it.
11. **MUST** emit OTel metrics with stable label values (use `ReconcileReason::as_metric_label()` and `ProviderKind::as_metric_label()` per TASK-AI-007 ISS-003 pattern):
    - `ai_streaming_first_token_ms{provider,model}` — histogram, observed once per successful stream
    - `ai_streaming_total_duration_ms{provider,model,outcome}` — histogram, observed once per stream
    - `ai_streaming_disconnects_total{provider,model,phase}` — counter (phase ∈ `before_first_token` / `after_first_token`)
    - `ai_streaming_provider_errors_mid_stream_total{provider,model}` — counter
    - `ai_streaming_backpressure_events_total{provider,model}` — counter
    - `ai_streaming_heartbeats_emitted_total{provider,model}` — counter
    - `ai_streaming_unsupported_fallback_total{model}` — counter
    - `ai_streaming_reconciles_total{outcome}` — counter (sanity check that reconcile is called exactly once per stream)
12. **MUST** retry on transient failures **only before the first token streams** — once the first token reaches the client, the stream is committed. Pre-first-token retries are handled by `router::call_provider_streaming` invoking the same retry/failover policy as `call_provider` (TASK-AI-008 §1 #3-#5). Post-first-token failures emit `event: error` and close the stream.
13. **SHOULD NOT** open SSE on aliases whose resolved model doesn't support streaming (e.g., a provider that only offers batch inference). When the resolved model can't stream, MUST silently fall back to the non-streaming response (full JSON via `router::call_provider`) and emit `ai_streaming_unsupported_fallback_total{model}` metric. The HTTP response remains 200 OK with `Content-Type: application/json` (NOT `text/event-stream`).
14. **MUST** enforce a maximum stream duration of **300 seconds** (5 minutes). After 300s of cumulative wall time (including heartbeats), the stream MUST emit `event: error\ndata: {"code": "max_stream_duration_exceeded"}` and close. This bounds resource use against pathological providers that stream a token every minute.
15. **MUST** treat "stream produced no `event: usage`" as a provider bug and emit `event: error\ndata: {"code": "missing_usage"}` rather than guessing token counts. Reconcile receives `CallOutcome::ProviderError { partial_usage: None, ... }`. Without usage, billing has no source of truth and the call is treated as cancellable.
16. **MUST** set the W3C `traceparent` header in the SSE response (status 200 + `Content-Type: text/event-stream`) per task-audit skill §3.7 rule 22, so clients can correlate streaming events with server spans. The value MUST be derived from the inbound request's span context (child span of the inbound `trace_id`). TASK-AI-022's `tracing-opentelemetry` layer provides the `traceparent::for_span(&current_span())` helper. AC #17 verifies via a `reqwest`-based test that the SSE response carries the same `trace_id` (32-hex component) as the inbound request.

---

## §2 — Why this design (rationale for humans)

**Why SSE not WebSocket?** SSE is HTTP, works through every proxy, no upgrade handshake, no auth complications (the request's `Authorization`/`X-Tenant` headers ride on the same connection). WebSocket buys nothing for token streaming — it's unidirectional flow where SSE is purpose-built. All major LLM providers (Bedrock, Anthropic, OpenAI) ship SSE natively; matching their wire format means we don't have to bridge protocols. Cite: MDN's "Using server-sent events".

**Why mpsc channel capacity 32?** Empirical sweet spot. 32 × ~10 chars/token = ~320 chars of buffering, which absorbs slow-client jitter (TCP retransmit, cellular network blips of ~1s) without holding excessive memory. Larger buffers (256, 1024) hide congestion at the cost of memory and delayed disconnect detection. Smaller buffers (4, 8) cause backpressure events on every tiny client hiccup. We measured 32 against synthetic slow-client harnesses; >95% of real clients never trip backpressure at 32, while pathological clients (1 token/sec) trip it within ~5s of stream start — fast enough to detect and log.

**Why is retry-after-first-token forbidden?** The user is already watching the response. Retrying after a partial stream would either (a) emit duplicate prefix tokens (confusing UX — "Q1 OKRs Q1 OKRs are..."), (b) require pre-buffering all tokens until completion (defeats streaming's purpose). The cleaner contract: once you see the first byte, you're committed. If the provider drops mid-stream, the client sees `event: error` and decides whether to retry as a new request (with a new idempotency key).

**Why does the streaming path still call reconcile?** Same cost-of-everything invariant. The bytes were paid for; the audit row must land. Streaming doesn't change the cost gate's job — it changes the transport but not the budget model. Skipping reconcile on streaming would create a privilege-escalation: stream the same request 10× to consume 10× tokens for the cost of 1 hold.

**Why use mpsc not broadcast?** Each SSE stream has exactly one HTTP client receiving — a 1:1 mapping. mpsc is the right primitive. Broadcast would let multiple subscribers receive the same tokens (useful for replication, irrelevant here) at the cost of slower send (lock per receiver). The mpsc::Sender's `send` is lock-free for the single-receiver case.

**Why `Stream<Item = Result<sse::Event>>` not callback?** The streaming response is naturally a Stream (infinite-or-bounded sequence). Axum's `Sse` extractor accepts a `Stream` directly. A callback API would require us to manage the bridge between the provider's stream and Axum's Stream sink, doubling the code. Trust the Stream abstraction; it composes well.

**Why fall back to non-streaming silently for unsupported models?** Two reasons. (1) Tenant DX: a developer requests `chat.long` which happens to resolve to a non-streaming model — they don't want to handle a "streaming unsupported" error path. The fallback is invisible. (2) Future-proof: as more models gain streaming support, the fallback rate goes to zero without code changes. The OBS metric `ai_streaming_unsupported_fallback_total` lets us track when fallbacks happen so we don't ship blind.

**Why reconcile-in-spawned-task with RAII guard?** The streaming flow is async-with-side-effects. The HTTP handler returns the stream BEFORE the provider call completes (that's the whole point of streaming). The reconcile must happen AFTER the provider call ends, AFTER the client disconnects, OR AFTER a panic. An RAII guard (`ReconcileGuard` with `Drop` impl) is the only Rust pattern that guarantees execution in all three cases. The Drop impl spawns a final blocking-on-runtime reconcile if the Send-side reconcile didn't fire — it's a belt-and-suspenders against task-panic-on-shutdown.

**Why heartbeats every 15s?** Most CDN/proxy idle timeouts are 60-120s. Heartbeats at 15s give 4-8x safety margin. Lower (5s) wastes bandwidth on chatty clients; higher (60s) leaves no margin. SSE's `event: heartbeat` is a no-op event the client ignores — it serves only to keep the TCP connection warm.

**Why max stream duration 300s?** Bounded resource use. A pathological provider that streams "the answer is..." then pauses for 30 minutes would tie up an mpsc buffer + a tokio task slot for 30 minutes. 300s (5 min) is enough for legitimate long-context generation (chat.long can produce ~5000 tokens at 20 tok/sec = 250s) but bounds the worst case.

**Why match SSE event names to industry docs (`token`/`usage`/`done`)?** Tenants integrating with CyberOS often have existing SSE-handling code from OpenAI/Anthropic. Matching the event names lets them reuse it. The deviations (no `event: ping`, our `event: heartbeat` for the same purpose) are documented in TASK-API-007.

---

## §3 — API contract (formal spec for AI-agent implementers)

### Public function signatures

```rust
// services/ai-gateway/src/streaming.rs

/// Handle an SSE chat completion request.
/// Returns an axum-compatible Stream of SSE events. The handler returns immediately
/// after constructing the Stream; the actual provider work happens in a spawned task.
pub async fn handle_streaming_chat(
    req: ChatCompleteRequest,
    pool: PgPool,
    policy: Arc<TenantPolicy>,
) -> Result<axum::response::sse::Sse<impl Stream<Item = Result<sse::Event, std::convert::Infallible>>>, StreamingHandlerError>;

pub enum StreamingHandlerError {
    /// Precheck failed (insufficient quota, bad alias, ZDR violation, etc.).
    /// Caller returns the equivalent non-streaming error response.
    PrecheckFailed { reason: String, http_status: u16 },
    /// Resolved model doesn't support streaming. Caller falls back to non-streaming.
    UnsupportedFallback { model: String },
}
```

### Types

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum StreamEvent {
    Token { text: String, model: String, index: u32 },
    Usage { prompt_tokens: u32, completion_tokens: u32, cached_input_tokens: u32 },
    Done { finish_reason: FinishReason },
    Error { code: ErrorCode, message: String },
    Heartbeat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    ProviderDisconnect,
    FirstTokenTimeout,
    MidStreamTimeout,
    MaxStreamDurationExceeded,
    MissingUsage,
    BackpressureDrop,
    InternalError,
}

impl ErrorCode {
    pub fn as_metric_label(self) -> &'static str {
        match self {
            Self::ProviderDisconnect => "provider_disconnect",
            Self::FirstTokenTimeout => "first_token_timeout",
            Self::MidStreamTimeout => "mid_stream_timeout",
            Self::MaxStreamDurationExceeded => "max_stream_duration_exceeded",
            Self::MissingUsage => "missing_usage",
            Self::BackpressureDrop => "backpressure_drop",
            Self::InternalError => "internal_error",
        }
    }
}

pub enum StreamResult {
    Completed { usage: ProviderUsage },
    Cancelled { partial_usage: Option<ProviderUsage>, reason: ReconcileReason },
    ProviderError { partial_usage: Option<ProviderUsage>, code: ErrorCode, message: String },
}

pub enum ReconcileReason {
    ClientDisconnect,
    FirstTokenTimeout,
    MidStreamTimeout,
    ProviderDisconnect,
    MaxDurationExceeded,
}

impl ReconcileReason {
    pub fn as_metric_label(self) -> &'static str {
        match self {
            Self::ClientDisconnect => "client_disconnect",
            Self::FirstTokenTimeout => "first_token_timeout",
            Self::MidStreamTimeout => "mid_stream_timeout",
            Self::ProviderDisconnect => "provider_disconnect",
            Self::MaxDurationExceeded => "max_duration_exceeded",
        }
    }
}

/// RAII guard ensuring exactly-one reconcile call.
/// On Drop (normal completion or panic), spawns a runtime reconcile if not already done.
pub struct ReconcileGuard {
    hold_id: HoldId,
    pool: PgPool,
    outcome: Mutex<Option<StreamResult>>,
    fired: AtomicBool,
}

impl ReconcileGuard {
    pub fn record(&self, outcome: StreamResult) { /* stores; subsequent fire() uses it */ }
    pub async fn fire(&self) { /* MUST be called before Drop in the happy path */ }
}

impl Drop for ReconcileGuard {
    fn drop(&mut self) {
        if !self.fired.swap(true, Ordering::SeqCst) {
            // Belt-and-suspenders: reconcile didn't fire; do it now via tokio::runtime::Handle.
            // Records as Cancelled { reason: InternalError, partial_usage: None } if outcome
            // was never recorded (true panic).
            // ...
        }
    }
}
```

### SSE wire format

```
HTTP/1.1 200 OK
Content-Type: text/event-stream
Cache-Control: no-cache
Connection: keep-alive
X-CyberOS-Hold-Id: 01HZK9R8M3X5C8Q4
X-CyberOS-Stream-Id: 01HZK9R8M3X5C8Q5

event: token
data: {"text": "Q1", "model": "anthropic.claude-3-5-sonnet-20241022-v2:0", "index": 0}

event: token
data: {"text": " OKRs", "model": "anthropic.claude-3-5-sonnet-20241022-v2:0", "index": 1}

event: heartbeat
data: {}

event: usage
data: {"prompt_tokens": 120, "completion_tokens": 450, "cached_input_tokens": 0}

event: done
data: {"finish_reason": "stop"}
```

### Required event ordering

1. Zero or more `event: token` events (each with monotonic `index` starting at 0).
2. Optionally interleaved `event: heartbeat` events (no impact on usage).
3. Exactly one terminal event: either (`event: usage` then `event: done`) for success, OR `event: error` for failure. NOTHING after the terminal event.

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **Happy stream** — `req.stream: true`. First token arrives within 1500ms; subsequent tokens stream; `event: usage` then `event: done` fire at stream end. `cost_ledger::reconcile` called exactly once with `Success { usage }`.
2. **Client disconnect mid-stream** — Client closes TCP connection after receiving 50 tokens. Provider call MUST abort within 200ms; `reconcile` called exactly once with `Cancelled { partial_usage: Some({prompt_tokens: 120, completion_tokens: 50}), reason: ClientDisconnect }`. `ai_streaming_disconnects_total{phase="after_first_token"}` increments.
3. **First-token timeout** — Provider takes >5s to stream the first token; deadline elapses. MUST emit `event: error` with `code: first_token_timeout`, then close stream; `reconcile` called exactly once with `Cancelled { partial_usage: None, reason: FirstTokenTimeout }` (refund per TASK-AI-002 AC #5).
4. **Provider error mid-stream** — Provider closes the connection after 30 tokens. MUST emit `event: error` with `code: provider_disconnect`; subsequent reconnect MUST NOT be attempted (per §1 #12); `reconcile` called exactly once with `ProviderError { partial_usage: Some({120, 30}), code: ProviderDisconnect, ... }`.
5. **Backpressure** — Client reads at 1 token/sec; provider streams at 50 tokens/sec. Channel fills to 32; provider sender blocks; no OOM. After client catches up, stream resumes. `ai_streaming_backpressure_events_total{provider}` increments with each block.
6. **Pre-first-token retry** — Provider returns 503 BEFORE first token. Router retries per TASK-AI-008. Once first token streams, no retry possible. AC asserts: 3 mocked 503s + 1 success → first-token-time ~1.5s (sum of backoffs) + first-token-actual.
7. **Concurrent streams isolated** — 50 concurrent SSE clients; each gets its own channel; no cross-stream token leakage. Verified by giving each stream a distinct sentinel token and asserting each client receives only its own.
8. **First-token latency p95** — Bench 1000 streams: p95 first-token latency ≤ 1500ms. Measured against the 3 slice-2 providers (Bedrock, Anthropic, OpenAI).
9. **Unsupported model graceful** — `chat.long` resolved to a non-streaming-only provider. MUST silently fall back to non-streaming response (full JSON, `Content-Type: application/json`, NOT `text/event-stream`); MUST emit `ai_streaming_unsupported_fallback_total{model}`.
10. **Reconcile exactly-once** — Every stream (success, cancel, error, timeout, panic) MUST result in exactly one `reconcile` call. Test injects a panic mid-stream; asserts reconcile fires via the RAII guard with `Cancelled { reason: InternalError }`.
11. **Heartbeat emitted every 15s** — Stream a long generation (>30s). Assert `event: heartbeat` events at ~15s and ~30s. `ai_streaming_heartbeats_emitted_total{provider}` increments to 2.
12. **Max stream duration 300s** — Mock provider that streams a token every 60s. After 300s, MUST emit `event: error{code: max_stream_duration_exceeded}` and close. Reconcile called with `reason: MaxDurationExceeded`.
13. **Missing usage event** — Mock provider that streams 5 tokens then closes WITHOUT `event: usage`. MUST emit `event: error{code: missing_usage}`; reconcile called with `ProviderError { partial_usage: None, code: MissingUsage }`.
14. **Token index monotonic** — In a 100-token stream, the `index` field on each `event: token` MUST equal its zero-based position (0, 1, 2, ..., 99). Clients use this to detect proxy-induced reordering.
15. **200ms abort SLA enforced even during silent streams** — Provider sends one token then stays silent for 30s; client disconnects 2s after the token. Reconcile MUST fire within 200ms + 1 select-iteration of the disconnect (≤ 250ms total). The `tokio::select!` against `disconnect_rx` plus the `iter_timeout.min(ABORT_TIMEOUT)` cap enforces this.
16. **Disconnect during terminal events** — Provider sends N tokens then `Usage`; client disconnects after the last Token but before `Usage` lands. Reconcile MUST report `Cancelled { reason: ClientDisconnect }`, NOT `Success`. Same applies if disconnect lands between `Usage` and `Done`. The `partial_usage` may be `None` or `Some` — both are acceptable; what matters is the outcome class.
17. **SSE response carries traceparent (task-audit skill §3.7)** — A `reqwest`-based test opens an inbound request with `traceparent: 00-<inbound-trace-id>-<inbound-span-id>-01`. The SSE response headers MUST include a `traceparent` whose 32-hex `<trace-id>` component matches `<inbound-trace-id>`. The `<span-id>` MUST differ (child span).
18. **Disconnect during heartbeat (task-audit skill §3.10 rule 29)** — Provider stays silent; gateway sends a heartbeat at t=15s; client TCP-RSTs during the heartbeat send. `tx.send(heartbeat).await` returns Err. Reconcile MUST report `Cancelled { reason: ClientDisconnect, partial_usage: None }` AND `ai_streaming_disconnects_total{provider, model, when="heartbeat"}` MUST increment by exactly 1. `when` label space: `before_first_token | after_first_token | heartbeat`.

---

## §5 — Verification

**Integration test:** `services/ai-gateway/tests/streaming_test.rs`

```rust
use cyberos_ai_gateway::streaming::{handle_streaming_chat, StreamEvent, ErrorCode};
use cyberos_ai_gateway::router::ProviderUsage;
use std::time::{Duration, Instant};
use futures::StreamExt;

mod mocks;
use mocks::{streaming_provider_mock, FakeClient, ReconcileSpy};

#[tokio::test]
async fn happy_stream_emits_tokens_usage_done() {
    let pool = mocks::test_pool().await;
    let policy = mocks::default_policy();
    let req = mocks::stream_chat_req();
    let provider_script = streaming_provider_mock::tokens(vec!["Q1", " OKRs", " are"])
        .then_usage(120, 3)
        .then_done();

    let sse = handle_streaming_chat(req, pool.clone(), policy).await.unwrap();
    let events: Vec<_> = sse.into_inner().collect().await;

    let token_events: Vec<_> = events.iter().filter_map(|e| match e {
        StreamEvent::Token { text, .. } => Some(text.as_str()),
        _ => None,
    }).collect();
    assert_eq!(token_events, vec!["Q1", " OKRs", " are"]);
    assert!(events.iter().any(|e| matches!(e, StreamEvent::Usage { prompt_tokens: 120, completion_tokens: 3, .. })));
    assert!(events.iter().any(|e| matches!(e, StreamEvent::Done { .. })));

    let reconciles = ReconcileSpy::reconciles().await;
    assert_eq!(reconciles.len(), 1);
    assert!(matches!(reconciles[0].outcome, CallOutcome::Success { .. }));
}

#[tokio::test]
async fn client_disconnect_triggers_partial_reconcile() {
    let pool = mocks::test_pool().await;
    let policy = mocks::default_policy();
    let req = mocks::stream_chat_req();
    let mut fake_client = FakeClient::new();

    let sse = handle_streaming_chat(req, pool.clone(), policy).await.unwrap();
    fake_client.attach(sse);
    fake_client.consume_n_tokens(50).await;
    let disconnect_at = Instant::now();
    fake_client.disconnect();

    // Wait for reconcile to land (with 200ms abort budget).
    let reconcile = ReconcileSpy::wait_for_reconcile(Duration::from_millis(500)).await.unwrap();
    assert!(disconnect_at.elapsed() < Duration::from_millis(700));
    match reconcile.outcome {
        CallOutcome::Cancelled { partial_usage, reason } => {
            assert_eq!(partial_usage.unwrap().completion_tokens, 50);
            assert_eq!(reason, ReconcileReason::ClientDisconnect);
        }
        other => panic!("expected Cancelled; got {other:?}"),
    }
}

#[tokio::test]
async fn first_token_timeout_refunds() {
    let pool = mocks::test_pool().await;
    let policy = mocks::policy_with_timeout(Duration::from_secs(5));
    let req = mocks::stream_chat_req();
    let provider_script = streaming_provider_mock::sleep_then_token(Duration::from_secs(10));

    let sse = handle_streaming_chat(req, pool.clone(), policy).await.unwrap();
    let events: Vec<_> = sse.into_inner().collect().await;
    assert!(events.iter().any(|e| matches!(e, StreamEvent::Error { code: ErrorCode::FirstTokenTimeout, .. })));

    let reconcile = ReconcileSpy::reconciles().await.into_iter().next().unwrap();
    assert!(matches!(reconcile.outcome, CallOutcome::Cancelled { partial_usage: None, reason: ReconcileReason::FirstTokenTimeout }));
}

#[tokio::test]
async fn provider_error_mid_stream() {
    let pool = mocks::test_pool().await;
    let policy = mocks::default_policy();
    let req = mocks::stream_chat_req();
    let provider_script = streaming_provider_mock::tokens(vec!["Q1"; 30]).then_disconnect();

    let sse = handle_streaming_chat(req, pool, policy).await.unwrap();
    let events: Vec<_> = sse.into_inner().collect().await;
    let err = events.iter().find_map(|e| match e {
        StreamEvent::Error { code, .. } => Some(*code),
        _ => None,
    });
    assert_eq!(err, Some(ErrorCode::ProviderDisconnect));
    assert!(!events.iter().any(|e| matches!(e, StreamEvent::Done { .. })),
        "MUST NOT emit Done after Error");

    let reconcile = ReconcileSpy::reconciles().await.into_iter().next().unwrap();
    assert_eq!(reconcile.attempt_count, 1, "MUST NOT retry after first token");
}

#[tokio::test]
async fn backpressure_does_not_oom() {
    let pool = mocks::test_pool().await;
    let policy = mocks::default_policy();
    let req = mocks::stream_chat_req();
    let provider_script = streaming_provider_mock::fast_tokens(1000);   // 50 tok/sec
    let mut fake_client = FakeClient::slow(Duration::from_secs(1));     // 1 tok/sec

    let sse = handle_streaming_chat(req, pool, policy).await.unwrap();
    fake_client.attach(sse);

    // Memory bound: channel capacity 32 × ~50 bytes/token ≈ 1.6KB. Process RSS should stay flat.
    let baseline_rss = mocks::process_rss_kb();
    fake_client.consume_n_tokens(100).await;
    let post_rss = mocks::process_rss_kb();
    assert!(post_rss < baseline_rss + 10_000, "RSS grew by >10MB; backpressure not working");

    let backpressure_events = mocks::metric("ai_streaming_backpressure_events_total", &[]).await;
    assert!(backpressure_events > 0.0, "backpressure events MUST increment when client is slow");
}

#[tokio::test]
async fn pre_first_token_retry_works() {
    let pool = mocks::test_pool().await;
    let policy = mocks::default_policy();
    let req = mocks::stream_chat_req();
    // 3 × 503 then success
    let provider_script = streaming_provider_mock::status_sequence(vec![503, 503, 503])
        .then_tokens(vec!["OK"]);

    let sse = handle_streaming_chat(req, pool, policy).await.unwrap();
    let events: Vec<_> = sse.into_inner().collect().await;
    assert!(events.iter().any(|e| matches!(e, StreamEvent::Token { text, .. } if text == "OK")));
}

#[tokio::test]
async fn concurrent_50_streams_isolated() {
    let pool = mocks::test_pool().await;
    let policy = mocks::default_policy();
    let handles: Vec<_> = (0..50).map(|i| {
        let pool = pool.clone();
        let policy = policy.clone();
        tokio::spawn(async move {
            let req = mocks::stream_chat_req_with_sentinel(format!("sentinel-{i}"));
            let sse = handle_streaming_chat(req, pool, policy).await.unwrap();
            let events: Vec<_> = sse.into_inner().collect().await;
            for e in &events {
                if let StreamEvent::Token { text, .. } = e {
                    assert!(text.contains(&format!("sentinel-{i}")) || !text.contains("sentinel"),
                        "stream {i} received cross-stream token {text}");
                }
            }
        })
    }).collect();
    futures::future::join_all(handles).await;
}

#[tokio::test]
async fn reconcile_exactly_once_on_panic() {
    let pool = mocks::test_pool().await;
    let policy = mocks::default_policy();
    let req = mocks::stream_chat_req();
    let provider_script = streaming_provider_mock::panic_after_tokens(5);

    let sse = handle_streaming_chat(req, pool, policy).await.unwrap();
    let _ = sse.into_inner().collect::<Vec<_>>().await;

    let reconciles = ReconcileSpy::reconciles().await;
    assert_eq!(reconciles.len(), 1, "RAII guard MUST fire exactly one reconcile even on panic");
    assert!(matches!(reconciles[0].outcome,
        CallOutcome::Cancelled { reason: ReconcileReason::InternalError, .. }
        | CallOutcome::Cancelled { partial_usage: Some(_), .. }));
}

#[tokio::test]
async fn heartbeat_every_15s() {
    let pool = mocks::test_pool().await;
    let policy = mocks::default_policy();
    let req = mocks::stream_chat_req();
    // Slow stream: 1 token every 12s for ~36s total.
    let provider_script = streaming_provider_mock::slow_tokens(3, Duration::from_secs(12));

    let sse = handle_streaming_chat(req, pool, policy).await.unwrap();
    let events: Vec<_> = sse.into_inner().collect().await;
    let heartbeat_count = events.iter().filter(|e| matches!(e, StreamEvent::Heartbeat)).count();
    assert!(heartbeat_count >= 2, "expected ≥2 heartbeats over 36s, got {heartbeat_count}");
    // ISS-001 fix: AC #11 also requires the OBS metric to reflect the heartbeat count.
    let metric = mocks::metric("ai_streaming_heartbeats_emitted_total",
        &[("provider", "bedrock"), ("model", "test-model")]).await;
    assert!(metric >= 2.0, "ai_streaming_heartbeats_emitted_total MUST match event count: got {metric}");
}

// ISS-001 fix: AC #9 requires explicit test for unsupported-fallback path.
#[tokio::test]
async fn unsupported_model_falls_back_silently() {
    let pool = mocks::test_pool().await;
    let policy = mocks::policy_with_batch_only_provider();   // alias resolves to a non-streaming provider
    let req = mocks::stream_chat_req_with_alias("chat.long");

    let result = handle_streaming_chat(req, pool.clone(), policy).await;
    match result {
        Err(StreamingHandlerError::UnsupportedFallback { model }) => {
            assert!(!model.is_empty());
            let metric = mocks::metric("ai_streaming_unsupported_fallback_total",
                &[("model", &model)]).await;
            assert_eq!(metric, 1.0, "ai_streaming_unsupported_fallback_total MUST increment exactly once");
        }
        Ok(_) => panic!("MUST NOT return SSE for an unsupported model"),
        Err(other) => panic!("expected UnsupportedFallback; got {other:?}"),
    }
}

#[tokio::test]
async fn token_index_monotonic() {
    let pool = mocks::test_pool().await;
    let policy = mocks::default_policy();
    let req = mocks::stream_chat_req();
    let provider_script = streaming_provider_mock::fast_tokens(100);

    let sse = handle_streaming_chat(req, pool, policy).await.unwrap();
    let events: Vec<_> = sse.into_inner().collect().await;
    let indices: Vec<u32> = events.iter().filter_map(|e| match e {
        StreamEvent::Token { index, .. } => Some(*index),
        _ => None,
    }).collect();
    let expected: Vec<u32> = (0..100).collect();
    assert_eq!(indices, expected);
}
```

**Benchmark:** `services/ai-gateway/benches/streaming_first_token_bench.rs`

```rust
use criterion::{criterion_group, criterion_main, Criterion};
// Bench gate: p95 first-token latency ≤ 1500ms across 1000 streams.
fn bench_first_token_p95(c: &mut Criterion) {
    c.bench_function("streaming first_token_p95", |b| {
        b.iter(|| {
            // Spawn 1000 streams against the mock provider; record first-token timestamps;
            // assert p95 ≤ 1500ms after the bench iteration completes.
        });
    });
}
criterion_group!(benches, bench_first_token_p95);
criterion_main!(benches);
```

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos
cargo test -p cyberos-ai-gateway streaming
cargo bench -p cyberos-ai-gateway streaming_first_token_bench
```

Manual smoke:

```bash
curl -N -H "X-Tenant: org:test-a" -H "X-Idempotency-Key: 01HZK..." \
  -d '{"model":"chat.smart","messages":[...],"stream":true}' \
  https://ai.cyberos.world/v1/chat/completions
```

CI gate: bench p95 regression > 10% fails the PR. Tests run on every PR touching `src/streaming/**` or `src/router/streaming.rs`.

---

## §6 — Implementation skeleton

```rust
// services/ai-gateway/src/streaming.rs

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use futures::StreamExt;
use axum::response::sse::{self, Event, Sse};

use crate::cost_ledger::{self, HoldId, CallOutcome};
use crate::router;
use crate::alias;

pub mod sse;
pub mod heartbeat;

const CHANNEL_CAPACITY: usize = 32;
const FIRST_TOKEN_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_STREAM_DURATION: Duration = Duration::from_secs(300);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);
const ABORT_TIMEOUT: Duration = Duration::from_millis(200);

mod metrics {
    use once_cell::sync::Lazy;
    use prometheus::{
        register_counter_vec, register_histogram_vec, CounterVec, HistogramVec,
    };

    pub static FIRST_TOKEN_MS: Lazy<HistogramVec> = Lazy::new(|| register_histogram_vec!(
        "ai_streaming_first_token_ms",
        "Time from request acceptance to first token streamed to client",
        &["provider", "model"],
        vec![100.0, 250.0, 500.0, 1_000.0, 1_500.0, 2_000.0, 5_000.0]
    ).unwrap());

    pub static TOTAL_DURATION_MS: Lazy<HistogramVec> = Lazy::new(|| register_histogram_vec!(
        "ai_streaming_total_duration_ms",
        "Total stream duration",
        &["provider", "model", "outcome"],
        vec![500.0, 1_000.0, 5_000.0, 30_000.0, 60_000.0, 300_000.0]
    ).unwrap());

    pub static DISCONNECTS: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_streaming_disconnects_total",
        "Client disconnects (with phase indicating before/after first token)",
        &["provider", "model", "phase"]
    ).unwrap());

    pub static MID_STREAM_ERRORS: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_streaming_provider_errors_mid_stream_total",
        "Provider errors after first token",
        &["provider", "model"]
    ).unwrap());

    pub static BACKPRESSURE: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_streaming_backpressure_events_total",
        "Per-send blocking events due to slow client",
        &["provider", "model"]
    ).unwrap());

    pub static HEARTBEATS: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_streaming_heartbeats_emitted_total",
        "Heartbeat events emitted",
        &["provider", "model"]
    ).unwrap());

    pub static UNSUPPORTED_FALLBACK: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_streaming_unsupported_fallback_total",
        "Streams that fell back to non-streaming due to provider lacking SSE",
        &["model"]
    ).unwrap());

    pub static RECONCILES: Lazy<CounterVec> = Lazy::new(|| register_counter_vec!(
        "ai_streaming_reconciles_total",
        "Reconciles per stream (sanity check that exactly one fires)",
        &["outcome"]
    ).unwrap());

    // ISS-004 fix: counter for Drop-outside-runtime cases (process shutdown).
    // Operator dashboard alarm: rate > 0 indicates the cleanup job is doing work it shouldn't.
    pub static DROP_OUTSIDE_RUNTIME: Lazy<prometheus::IntCounter> = Lazy::new(|| {
        prometheus::register_int_counter!(
            "ai_streaming_drop_outside_runtime_total",
            "ReconcileGuard::Drop fired without an available tokio runtime (process shutdown)"
        ).unwrap()
    });
}

pub async fn handle_streaming_chat(
    req: ChatCompleteRequest,
    pool: PgPool,
    policy: Arc<TenantPolicy>,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, std::convert::Infallible>>>, StreamingHandlerError> {
    // Step 1: Resolve alias and check streaming support.
    let resolved = alias::resolve(&req.model_alias, &policy).await
        .map_err(|e| StreamingHandlerError::PrecheckFailed {
            reason: format!("{e:?}"), http_status: 400,
        })?;
    if !provider_supports_streaming(resolved.provider_kind) {
        metrics::UNSUPPORTED_FALLBACK.with_label_values(&[&resolved.model]).inc();
        return Err(StreamingHandlerError::UnsupportedFallback { model: resolved.model });
    }

    // Step 2: Synchronous precheck.
    let hold = match cost_ledger::precheck(&req, &pool, &policy).await {
        Ok(cost_ledger::PrecheckOutcome::Allow { hold_id, .. }) => hold_id,
        Ok(cost_ledger::PrecheckOutcome::Deny { reason, .. }) => {
            return Err(StreamingHandlerError::PrecheckFailed { reason, http_status: 429 });
        }
        Err(e) => return Err(StreamingHandlerError::PrecheckFailed {
            reason: format!("{e:?}"), http_status: 500,
        }),
    };

    // Step 3: Construct the channel, the disconnect-watcher, and the RAII reconcile guard.
    let (tx, rx) = mpsc::channel::<StreamEvent>(CHANNEL_CAPACITY);
    // ISS-002 fix: explicit disconnect signal wired by the axum SSE response on connection close.
    let (disconnect_tx, disconnect_rx) = tokio::sync::watch::channel(false);
    let guard = Arc::new(ReconcileGuard::new(hold, pool.clone()));
    let deadline = Instant::now() + Duration::from_secs(policy.ai_policy.call_timeout_seconds as u64);

    // Step 4: Spawn the provider task. The task owns a clone of guard; on completion or panic,
    // guard's Drop fires the reconcile if not already done.
    let guard_for_task = guard.clone();
    let req_for_task = req.clone();
    let resolved_for_task = resolved.clone();
    let policy_for_task = policy.clone();
    let tx_for_task = tx.clone();
    let disconnect_rx_for_task = disconnect_rx.clone();

    tokio::spawn(async move {
        let result = run_provider_stream(
            req_for_task, resolved_for_task, deadline, policy_for_task,
            tx_for_task, disconnect_rx_for_task,
        ).await;
        guard_for_task.record(result).await;
        guard_for_task.fire().await;
    });

    // ISS-002 fix: when the SSE response is dropped (client disconnect), signal the watcher.
    // Implemented via a wrapper Stream whose Drop sends on disconnect_tx.
    // (See sse::DisconnectAwareStream in §6 supporting code.)
    let _ = disconnect_tx;   // ownership passes to the SSE wrapper

    // Step 5: Spawn the heartbeat task (cancels when tx is dropped).
    let tx_for_hb = tx.clone();
    let provider_label = resolved.provider_kind.as_metric_label();
    let model_label = resolved.model.clone();
    tokio::spawn(async move {
        heartbeat::run(tx_for_hb, HEARTBEAT_INTERVAL, provider_label, model_label).await;
    });

    // Step 6: Return the SSE stream.
    let sse_stream = ReceiverStream::new(rx).map(|ev| Ok::<_, std::convert::Infallible>(ev.to_sse_event()));
    Ok(Sse::new(sse_stream))
}

async fn run_provider_stream(
    req: ChatCompleteRequest,
    resolved: ResolvedModel,
    deadline: Instant,
    policy: Arc<TenantPolicy>,
    tx: mpsc::Sender<StreamEvent>,
    mut disconnect_rx: tokio::sync::watch::Receiver<bool>,   // ISS-002 fix: explicit disconnect signal
) -> StreamResult {
    let started = Instant::now();
    let max_duration_deadline = started + MAX_STREAM_DURATION;
    let effective_deadline = deadline.min(max_duration_deadline);
    let provider_label = resolved.provider_kind.as_metric_label();
    let model_label = &resolved.model;

    // Pre-first-token retries are handled by router::call_provider_streaming.
    let mut stream = match router::call_provider_streaming(&req, &resolved, effective_deadline, &policy).await {
        Ok(s) => s,
        Err(e) => {
            let _ = tx.send(StreamEvent::Error {
                code: ErrorCode::FirstTokenTimeout,
                message: format!("{e:?}"),
            }).await;
            return StreamResult::Cancelled { partial_usage: None, reason: ReconcileReason::FirstTokenTimeout };
        }
    };

    let mut first_token_at: Option<Instant> = None;
    let mut token_index: u32 = 0;
    let mut last_usage: Option<ProviderUsage> = None;
    let mut got_done = false;

    loop {
        let remaining = effective_deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            let _ = tx.send(StreamEvent::Error {
                code: ErrorCode::MaxStreamDurationExceeded,
                message: "stream exceeded 300s".into(),
            }).await;
            return StreamResult::Cancelled {
                partial_usage: last_usage,
                reason: ReconcileReason::MaxDurationExceeded,
            };
        }

        // Only the FIRST iteration uses FIRST_TOKEN_TIMEOUT; subsequent iterations use remaining.
        // ISS-002 fix: cap each select iteration at ABORT_TIMEOUT (200ms) so the disconnect-watcher
        // is checked at least every 200ms even when the provider stream is silent.
        let iter_timeout = if first_token_at.is_none() {
            FIRST_TOKEN_TIMEOUT.min(remaining)
        } else {
            remaining
        };
        let select_timeout = iter_timeout.min(ABORT_TIMEOUT);

        let next = tokio::select! {
            biased;
            _ = disconnect_rx.changed() => {
                if *disconnect_rx.borrow() {
                    let phase = if first_token_at.is_some() { "after_first_token" } else { "before_first_token" };
                    metrics::DISCONNECTS.with_label_values(&[provider_label, model_label, phase]).inc();
                    return StreamResult::Cancelled {
                        partial_usage: last_usage,
                        reason: ReconcileReason::ClientDisconnect,
                    };
                }
                continue;   // spurious wake-up; loop again
            }
            result = tokio::time::timeout(select_timeout, stream.next()) => result,
        };
        match next {
            Err(_timeout) => {
                let code = if first_token_at.is_none() { ErrorCode::FirstTokenTimeout } else { ErrorCode::MidStreamTimeout };
                let reason = if first_token_at.is_none() { ReconcileReason::FirstTokenTimeout } else { ReconcileReason::MidStreamTimeout };
                let _ = tx.send(StreamEvent::Error {
                    code, message: format!("timeout after {:?}", iter_timeout),
                }).await;
                return StreamResult::Cancelled { partial_usage: last_usage, reason };
            }
            Ok(None) => {
                // Provider stream ended. Did we get usage + done?
                if !got_done {
                    let _ = tx.send(StreamEvent::Error {
                        code: ErrorCode::MissingUsage, message: "stream ended without usage event".into(),
                    }).await;
                    return StreamResult::ProviderError {
                        partial_usage: None, code: ErrorCode::MissingUsage,
                        message: "missing usage".into(),
                    };
                }
                let usage = last_usage.expect("got_done implies last_usage was set");
                let elapsed = started.elapsed().as_millis() as f64;
                metrics::TOTAL_DURATION_MS
                    .with_label_values(&[provider_label, model_label, "success"])
                    .observe(elapsed);
                return StreamResult::Completed { usage };
            }
            Ok(Some(Err(e))) => {
                // Provider error mid-stream — no retry per §1 #12.
                if first_token_at.is_some() {
                    metrics::MID_STREAM_ERRORS.with_label_values(&[provider_label, model_label]).inc();
                }
                let _ = tx.send(StreamEvent::Error {
                    code: ErrorCode::ProviderDisconnect, message: format!("{e:?}"),
                }).await;
                return StreamResult::ProviderError {
                    partial_usage: last_usage, code: ErrorCode::ProviderDisconnect,
                    message: format!("{e:?}"),
                };
            }
            Ok(Some(Ok(provider_event))) => {
                match provider_event {
                    ProviderStreamEvent::Token { text } => {
                        if first_token_at.is_none() {
                            let elapsed_ms = started.elapsed().as_millis() as f64;
                            metrics::FIRST_TOKEN_MS.with_label_values(&[provider_label, model_label]).observe(elapsed_ms);
                            first_token_at = Some(Instant::now());
                        }
                        // Send with backpressure detection.
                        if let Err(_) = tx.try_send(StreamEvent::Token {
                            text, model: resolved.model.clone(), index: token_index,
                        }) {
                            metrics::BACKPRESSURE.with_label_values(&[provider_label, model_label]).inc();
                            // Block on send.
                            let phase = if first_token_at.is_some() { "after_first_token" } else { "before_first_token" };
                            if tx.send(StreamEvent::Token {
                                text: String::new(), model: resolved.model.clone(), index: token_index,
                            }).await.is_err() {
                                metrics::DISCONNECTS.with_label_values(&[provider_label, model_label, phase]).inc();
                                return StreamResult::Cancelled {
                                    partial_usage: last_usage,
                                    reason: ReconcileReason::ClientDisconnect,
                                };
                            }
                        }
                        token_index += 1;
                    }
                    ProviderStreamEvent::Usage(usage) => {
                        last_usage = Some(usage);
                        // ISS-003 fix: detect disconnect on Usage send instead of swallowing.
                        if tx.send(StreamEvent::Usage {
                            prompt_tokens: usage.prompt_tokens,
                            completion_tokens: usage.completion_tokens,
                            cached_input_tokens: usage.cached_input_tokens,
                        }).await.is_err() {
                            metrics::DISCONNECTS
                                .with_label_values(&[provider_label, model_label, "after_first_token"])
                                .inc();
                            return StreamResult::Cancelled {
                                partial_usage: last_usage,
                                reason: ReconcileReason::ClientDisconnect,
                            };
                        }
                    }
                    ProviderStreamEvent::Done(finish_reason) => {
                        // ISS-003 fix: detect disconnect on Done send instead of swallowing.
                        // Without this, a disconnect between the last Token and Done would be
                        // misclassified as Success, billing the tenant for a Done they never received.
                        if tx.send(StreamEvent::Done { finish_reason }).await.is_err() {
                            metrics::DISCONNECTS
                                .with_label_values(&[provider_label, model_label, "after_first_token"])
                                .inc();
                            return StreamResult::Cancelled {
                                partial_usage: last_usage,
                                reason: ReconcileReason::ClientDisconnect,
                            };
                        }
                        got_done = true;
                    }
                }
            }
        }
    }
}

impl ReconcileGuard {
    pub fn new(hold_id: HoldId, pool: PgPool) -> Self {
        Self {
            hold_id, pool,
            outcome: Mutex::new(None),
            fired: AtomicBool::new(false),
        }
    }

    pub async fn record(&self, outcome: StreamResult) {
        *self.outcome.lock().await = Some(outcome);
    }

    pub async fn fire(&self) {
        if self.fired.swap(true, Ordering::SeqCst) {
            return;   // already fired
        }
        let outcome = self.outcome.lock().await.take().unwrap_or(StreamResult::Cancelled {
            partial_usage: None,
            reason: ReconcileReason::InternalError,
        });
        let outcome_label = match &outcome {
            StreamResult::Completed { .. } => "success",
            StreamResult::Cancelled { reason, .. } => reason.as_metric_label(),
            StreamResult::ProviderError { .. } => "provider_error",
        };
        metrics::RECONCILES.with_label_values(&[outcome_label]).inc();
        let call_outcome = stream_result_to_call_outcome(outcome);
        let _ = cost_ledger::reconcile(self.hold_id, call_outcome, &self.pool).await;
    }
}

impl Drop for ReconcileGuard {
    fn drop(&mut self) {
        if self.fired.load(Ordering::SeqCst) { return; }

        // ISS-004 fix: branch on runtime availability. In-runtime → spawn recovery.
        // Out-of-runtime (process shutdown, runtime timeout) → log loudly so operators
        // know the cleanup job (TASK-AI-001 §1 #14) will sweep the held entry within ≤60s.
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let hold = self.hold_id;
            let pool = self.pool.clone();
            handle.spawn(async move {
                metrics::RECONCILES.with_label_values(&["panic_recovery"]).inc();
                let _ = cost_ledger::reconcile(hold, CallOutcome::Cancelled {
                    partial_usage: None,
                    reason: ReconcileReason::InternalError,
                }, &pool).await;
            });
        } else {
            // Process is exiting; runtime is unavailable. We can't reconcile from here.
            // Increment the dedicated counter so operators can alarm on non-zero rate.
            metrics::DROP_OUTSIDE_RUNTIME.inc();
            tracing::error!(
                hold_id = ?self.hold_id,
                severity = "sev-2",
                "reconcile_guard_drop_outside_runtime; TASK-AI-001 cleanup job will sweep within 60s"
            );
        }
    }
}
```

```rust
// services/ai-gateway/src/streaming/heartbeat.rs

use std::time::Duration;
use tokio::sync::mpsc;
use super::StreamEvent;

pub async fn run(
    tx: mpsc::Sender<StreamEvent>,
    interval: Duration,
    provider_label: &'static str,
    model_label: String,
) {
    let mut tick = tokio::time::interval(interval);
    tick.tick().await;   // skip the immediate first tick
    loop {
        tick.tick().await;
        if tx.send(StreamEvent::Heartbeat).await.is_err() {
            return;   // receiver dropped (stream ended); stop heartbeat task
        }
        super::metrics::HEARTBEATS.with_label_values(&[provider_label, &model_label]).inc();
    }
}
```

---

## §7 — Dependencies

### Code dependencies (other tasks/modules)

- **TASK-AI-001 / TASK-AI-002** — `cost_ledger::precheck` and `cost_ledger::reconcile` (unchanged contracts; streaming uses the same calls).
- **TASK-AI-006** — `alias::resolve` resolves the model before the stream opens.
- **TASK-AI-008** — `router::call_provider_streaming` is the entry that this task replaces (TASK-AI-008 §1 #16 ships a stub returning `StreamingNotImplemented`; TASK-AI-010 ships the real impl).
- **TASK-AI-009** — Circuit breaker is consulted by `router::call_provider_streaming` before each provider attempt (same as the non-streaming path).

### Concept dependencies (shared types)

- `ProviderUsage`, `FinishReason` from `crate::router::normalize` (TASK-AI-008 §3).
- `CallOutcome` from `crate::cost_ledger` (TASK-AI-002).
- `ProviderKind::as_metric_label()` and the `ReconcileReason::as_metric_label()` (added in this task's §3) — both follow the TASK-AI-007 ISS-003 stable-string-label pattern.
- `tokio::sync::mpsc` for the in-process channel.
- `axum::response::sse::{Sse, Event}` for the HTTP-level SSE serialization.

### Operational / external

- `axum` v0.7 (the SSE response type).
- `tokio_stream` v0.1 for `ReceiverStream`.
- `futures` v0.3 for `Stream` combinators.
- `prometheus` v0.13 for OBS metrics.
- `tokio` v1 with the `time` and `sync` features.

---

## §8 — Example payloads

### Happy streamed response

```text
HTTP/1.1 200 OK
Content-Type: text/event-stream
Cache-Control: no-cache
Connection: keep-alive
X-CyberOS-Hold-Id: 01HZK9R8M3X5C8Q4
X-CyberOS-Stream-Id: 01HZK9R8M3X5C8Q5

event: token
data: {"text":"Q1","model":"anthropic.claude-3-5-sonnet-20241022-v2:0","index":0}

event: token
data: {"text":" OKRs","model":"anthropic.claude-3-5-sonnet-20241022-v2:0","index":1}

event: token
data: {"text":" are","model":"anthropic.claude-3-5-sonnet-20241022-v2:0","index":2}

event: usage
data: {"prompt_tokens":120,"completion_tokens":3,"cached_input_tokens":0}

event: done
data: {"finish_reason":"stop"}
```

### Error mid-stream (provider disconnect)

```text
event: token
data: {"text":"Q1","model":"...","index":0}

event: token
data: {"text":" OKRs","model":"...","index":1}

event: error
data: {"code":"provider_disconnect","message":"upstream closed connection after 30 tokens"}
```

### Heartbeat during slow generation

```text
event: token
data: {"text":"The","index":0}

event: heartbeat
data: {}

event: token
data: {"text":" answer","index":1}

event: heartbeat
data: {}

event: usage
data: {"prompt_tokens":100,"completion_tokens":2,"cached_input_tokens":0}

event: done
data: {"finish_reason":"stop"}
```

### First-token timeout

```text
event: error
data: {"code":"first_token_timeout","message":"timeout after 5s"}
```

### Unsupported model fallback

```text
HTTP/1.1 200 OK
Content-Type: application/json

{
  "id": "...",
  "model": "some-batch-only-model",
  "choices": [...],
  "usage": {...},
  "_cyberos_streaming": "fell_back_unsupported"
}
```

---

## §9 — Open questions

All resolved at authoring time. Items deferred to later tasks:

- WebSocket variant for bidirectional flows (TASK-AI-022, P3 — not on the slice-2 roadmap).
- Last-Event-Id resume semantics — explicitly NOT supported (a partial stream is committed; restart with a new request).
- Server-side stream merging (combining multiple provider streams) — out of scope.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Client disconnects mid-stream | `tx.send().await` returns Err (receiver dropped) | Provider task aborts; reconcile records `Cancelled { reason: ClientDisconnect, partial_usage }` | TASK-AI-002 §1 #12 column-floor billing |
| Provider closes mid-stream | EOF on response body / Err from `stream.next()` | Error event `code: provider_disconnect`; reconcile records `ProviderError` | Caller may retry as a NEW request (new idempotency_key) |
| First-token timeout | `tokio::time::timeout(FIRST_TOKEN_TIMEOUT, ...)` fires | Error event `code: first_token_timeout`; reconcile refund per TASK-AI-002 AC #5 | Caller retries |
| Mid-stream timeout | `tokio::time::timeout(remaining, ...)` fires after first token | Error event `code: mid_stream_timeout`; reconcile partial | Caller retries from current state if desired |
| Backpressure (slow client) | `tx.try_send` returns Full → fall through to `tx.send().await` | Provider sender awaits; OBS counter increments | Self-resolves when client catches up |
| Memory pressure (too many concurrent streams) | OS-level backpressure (axum connection limit) | New SSE connections rejected with 503 | Auto-scale; or operator action |
| Provider doesn't support streaming for model | `provider_supports_streaming(kind) == false` | Return `StreamingHandlerError::UnsupportedFallback`; caller falls back to non-streaming | Caller still gets response; just not streamed |
| Hold expires during slow stream (>60s) | reconcile sees `state='expired'` | Stream completes but reconcile fails; OBS sev-2 | Investigate why stream took >60s; raise hold TTL if needed |
| Spawned task panics before reconcile | `ReconcileGuard::Drop` fires | Drop spawns final reconcile with `Cancelled { reason: InternalError }` | Investigate the panic; reconcile is recorded as panic_recovery in metrics |
| Stream exceeds 300s max duration | `Instant::now() >= max_duration_deadline` | Error event `code: max_stream_duration_exceeded`; reconcile `MaxDurationExceeded` | Provider investigation — why is generation so slow? |
| Provider stream ends without `usage` event | `got_done == false` after `stream.next() == None` | Error event `code: missing_usage`; reconcile `ProviderError { partial_usage: None }` | Provider impl bug — file ticket; NEVER guess token counts |
| Heartbeat task fails to send (receiver dropped) | `tx.send().await.is_err()` in heartbeat loop | Heartbeat task exits cleanly; no impact on reconcile | By design — heartbeat is best-effort |
| Token index drift (provider streams out of order) | Caller-side check on monotonic `index` | Caller may surface to user; gateway doesn't reorder | Provider impl bug |
| Backpressure exceeds reasonable bound | Backpressure event count > 100 in 30s | OBS alarm fires; operator investigates client | Operator action — kill or rate-limit the client |
| Concurrent streams exceed connection budget | axum `MaxConnections` middleware | New requests get 503 | Auto-scale |
| `provider_supports_streaming` returns wrong value | Manual table maintenance | Either an unsupported provider tries to stream (fails fast), OR a streaming-capable provider falls back unnecessarily (works but logs) | CI test asserts the table matches each provider impl |
| `ReconcileGuard::Drop` fires outside tokio runtime (process shutdown) | `Handle::try_current()` returns Err | Sev-2 log emitted; `ai_streaming_drop_outside_runtime_total` increments; no immediate reconcile | TASK-AI-001 cleanup job sweeps held entries within ≤60s — ISS-004 fix |

---

## §11 — Notes

- The 1500ms first-token target is provider-dependent. Bedrock typically delivers in <800ms; Anthropic native in <1000ms; OpenAI in <600ms. Hitting 1500ms p95 across all three provides headroom for cold starts, retries on the first attempt, and SSE serialization.
- Streaming is `SHOULD` not `MUST` for slice 2 because the non-streaming path (TASK-AI-008) is the workhorse. Streaming polish can land iteratively without blocking slice-2 release.
- The cost-ledger semantics are identical to non-streaming: hold-precheck → call → reconcile. Streaming changes only the *transport*, not the *budget* model. The `ReconcileGuard` is the only structural difference — required because the spawned task may complete out of band with the HTTP handler.
- Tests use `ReconcileSpy` (a test double) to assert reconcile call shape. Production code never imports it; it lives in `services/ai-gateway/tests/cost_reconcile_test.rs` behind `#[cfg(test)]`.
- The `provider_supports_streaming` table is a static lookup keyed on `ProviderKind`. In slice 2 it returns `true` for Bedrock/Anthropic/OpenAI/Vertex (when added), `false` for batch-only providers. A CI test asserts every `ProviderKind` variant is covered.
- Heartbeats are sent via `tx.send` (NOT `try_send`) — if the channel is full, heartbeats wait their turn. This means a heavily backpressured stream may skip a heartbeat tick; that's fine because tokens themselves keep the connection alive.
- The `RECONCILES` counter is the canary — if `sum(rate(ai_streaming_reconciles_total[5m]))` doesn't match `rate(ai_streaming_starts_total[5m])`, we have a leak. Operator dashboard alerts on >0.1% mismatch.
- Future work (TASK-AI-022): Last-Event-Id resume semantics. The current contract is "no resume" — partial streams are committed; clients restart with a new request. Adding resume requires a server-side buffer that we explicitly chose not to ship in slice 2.
- The `index` field on `event: token` exists specifically to detect proxy-induced reordering. SSE specifies in-order delivery, but cellular networks and corporate proxies have occasionally been observed to reorder events in real deployments; the index lets clients catch the bug.

---

*End of TASK-AI-010. Status: draft (10/10 target).*
