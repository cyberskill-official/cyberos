---
task_id: TASK-AI-010
audited: 2026-05-16
auditor: manual (engineering-spec template)
verdict: PASS (after revision)
score_pre_revision: 7.0/10        # the first-pass compressed version (284 lines)
score_post_expansion: 9.0/10      # after expanding to TASK-AI-001 depth (~880 lines)
score_post_revision: 10/10         # after 4 mechanical fixes
issues_open: 0
issues_resolved: 6
issues_critical: 0
template: engineering-spec@1
revised_at: 2026-05-16
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 ISSes)
---

## §1 — Verdict summary

TASK-AI-010 was expanded from 284 lines (compressed first-pass) to ~880 lines matching TASK-AI-001 depth. The expansion added 3 §1 clauses (#9 heartbeats every 15s, #14 max stream duration 300s, #15 missing-usage handling), 8 additional §2 paragraphs (mpsc-vs-broadcast, Stream-vs-callback, fall-back-rationale, RAII-guard-rationale, heartbeat-15s, max-300s, SSE-event-name-matching, retry-after-first-token-forbidden), full schema types in §3 (`StreamEvent` with `index` field, `ErrorCode` enum with 7 variants + `as_metric_label`, `ReconcileReason` enum with 5 variants + `as_metric_label`, `ReconcileGuard` with RAII Drop impl), 4 additional §4 ACs (#10 reconcile exactly-once on panic, #11 heartbeat every 15s, #12 max stream duration 300s, #13 missing usage event, #14 token index monotonic), full Rust integration test bodies in §5 (10 named tokio tests + Criterion benchmark stub), expanded ~430-line §6 skeleton with metrics module + `run_provider_stream` + `ReconcileGuard` impl + `heartbeat::run` task, code/concept/operational deps in §7, 5 example payloads in §8, 16 failure modes in §10, 9 implementation notes in §11.

Four residual issues prevent 10/10.

## §2 — Findings

### ISS-001 — AC #9 (unsupported fallback) and AC #11 metric assertions missing from §5
- **severity:** error
- **rule_id:** test-coverage
- **location:** §4 ACs #9, #11; §5 (verification)
- **status:** open

#### Description
Two ACs lack matching test bodies:

- AC #9: *"Unsupported model graceful — `chat.long` resolved to a non-streaming-only provider. MUST silently fall back to non-streaming response (full JSON, `Content-Type: application/json`, NOT `text/event-stream`); MUST emit `ai_streaming_unsupported_fallback_total{model}`."* No test exists in §5.
- AC #11: *"Heartbeat emitted every 15s — Stream a long generation (>30s). Assert `event: heartbeat` events at ~15s and ~30s. `ai_streaming_heartbeats_emitted_total{provider}` increments to 2."* The test `heartbeat_every_15s` exists but only counts events from the stream — it doesn't assert the metric counter incremented to 2.

Same shape as the TASK-AI-007/008/009 ISS-001 patterns: ACs reference behaviors that lack a matching `#[tokio::test]` body or a metric-counter assertion.

#### Suggested fix
Add the missing tests to §5:

```rust
#[tokio::test]
async fn unsupported_model_falls_back_silently() {
    let pool = mocks::test_pool().await;
    let policy = mocks::policy_with_batch_only_provider();   // alias resolves to a non-streaming provider
    let req = mocks::stream_chat_req_with_alias("chat.long");

    let result = handle_streaming_chat(req, pool.clone(), policy).await;
    match result {
        Err(StreamingHandlerError::UnsupportedFallback { model }) => {
            assert!(!model.is_empty());
            let metric = mocks::metric("ai_streaming_unsupported_fallback_total", &[("model", &model)]).await;
            assert_eq!(metric, 1.0);
        }
        Ok(_) => panic!("MUST NOT return SSE for unsupported model"),
        Err(other) => panic!("expected UnsupportedFallback; got {other:?}"),
    }
}
```

And tighten the existing heartbeat test:

```rust
#[tokio::test]
async fn heartbeat_every_15s() {
    // ... existing setup ...
    let heartbeat_count = events.iter().filter(|e| matches!(e, StreamEvent::Heartbeat)).count();
    assert!(heartbeat_count >= 2, "expected ≥2 heartbeats over 36s, got {heartbeat_count}");
    let metric = mocks::metric("ai_streaming_heartbeats_emitted_total",
        &[("provider", "bedrock"), ("model", "test-model")]).await;
    assert!(metric >= 2.0, "metric MUST match event count: got {metric}");
}
```

### ISS-002 — 200ms abort SLA from §1 #6 not enforced; `ABORT_TIMEOUT` constant unused
- **severity:** error
- **rule_id:** correctness
- **location:** §1 #6, §6 skeleton (`run_provider_stream` loop + `const ABORT_TIMEOUT`)
- **status:** open

#### Description
§1 #6 mandates: *"MUST detect client disconnect via `axum`'s connection-watcher and abort the upstream provider call within **200ms** of the disconnect."*

The §6 skeleton defines `const ABORT_TIMEOUT: Duration = Duration::from_millis(200);` but **never references it anywhere**. The actual disconnect detection path is implicit:
1. Client disconnects → receiver dropped.
2. Next `tx.send(...).await` returns `Err`.
3. Loop returns `StreamResult::Cancelled`.

The "next send" might be 100ms away (typical inter-token gap) OR 12s away (between heartbeats during a paused stream). No 200ms ceiling is enforced. AC #2 asserts ≤700ms (which gives headroom for the loop step) but doesn't enforce the §1 #6 200ms invariant.

#### Suggested fix
Add explicit cooperative-abort using `tokio::select!`:

```rust
use tokio::sync::watch;

// At handler entry, build a disconnect-watcher.
let (disconnect_tx, mut disconnect_rx) = watch::channel(false);
// (axum signal wires `disconnect_tx.send(true)` on connection close.)

// In run_provider_stream loop:
loop {
    let next = tokio::select! {
        biased;
        _ = disconnect_rx.changed() => {
            if *disconnect_rx.borrow() {
                metrics::DISCONNECTS.with_label_values(&[provider_label, model_label, phase()]).inc();
                return StreamResult::Cancelled {
                    partial_usage: last_usage,
                    reason: ReconcileReason::ClientDisconnect,
                };
            }
            continue;
        }
        result = tokio::time::timeout(iter_timeout.min(ABORT_TIMEOUT), stream.next()) => result,
    };
    // ... existing match on next ...
}
```

The `iter_timeout.min(ABORT_TIMEOUT)` ensures the loop wakes at least every 200ms to check `disconnect_rx`. Combined with the `tokio::select!` racing the disconnect channel, the 200ms SLA is enforced even when the provider stream is silent.

Add a test:

```rust
#[tokio::test]
async fn disconnect_aborts_within_200ms_even_during_silent_stream() {
    // Provider that goes silent for 30s after first token; client disconnects at t=2s.
    // Assert reconcile fires within 2.2s of disconnect.
}
```

### ISS-003 — Client disconnect during Usage/Done sends silently swallowed; reports success
- **severity:** error
- **rule_id:** correctness
- **location:** §6 skeleton (`run_provider_stream`, `let _ = tx.send(StreamEvent::Usage/Done)`)
- **status:** open

#### Description
The Token-send branch checks `tx.send(...).await.is_err()` and returns `Cancelled` on disconnect. But the Usage and Done branches use `let _ = tx.send(...).await;` which silently swallows the Err:

```rust
ProviderStreamEvent::Usage(usage) => {
    last_usage = Some(usage);
    let _ = tx.send(StreamEvent::Usage { ... }).await;   // BUG: ignores disconnect
}
ProviderStreamEvent::Done(finish_reason) => {
    let _ = tx.send(StreamEvent::Done { finish_reason }).await;
    got_done = true;   // sets even if client disconnected
}
```

Sequence that triggers the bug:
1. Provider sends N tokens (all received by client).
2. Client disconnects.
3. Provider sends Usage event — `tx.send` returns Err, ignored. `last_usage = Some(...)` set.
4. Provider sends Done event — `tx.send` returns Err, ignored. `got_done = true`.
5. Loop sees `Ok(None)` (provider stream ended), `got_done == true`, returns `StreamResult::Completed { usage }`.

Result: `cost_ledger::reconcile` is called with `Success { usage }` even though the client never received the Done event. The client is billed for tokens they didn't receive (the column-floor billing per TASK-AI-002 §1 #12 SHOULD apply but doesn't because we report Completed, not Cancelled).

#### Suggested fix
Promote the disconnect check to all `tx.send` paths:

```rust
ProviderStreamEvent::Usage(usage) => {
    last_usage = Some(usage);
    if tx.send(StreamEvent::Usage { ... }).await.is_err() {
        metrics::DISCONNECTS.with_label_values(&[provider_label, model_label, "after_first_token"]).inc();
        return StreamResult::Cancelled {
            partial_usage: last_usage,
            reason: ReconcileReason::ClientDisconnect,
        };
    }
}
ProviderStreamEvent::Done(finish_reason) => {
    if tx.send(StreamEvent::Done { finish_reason }).await.is_err() {
        metrics::DISCONNECTS.with_label_values(&[provider_label, model_label, "after_first_token"]).inc();
        return StreamResult::Cancelled {
            partial_usage: last_usage,
            reason: ReconcileReason::ClientDisconnect,
        };
    }
    got_done = true;
}
```

Add §4 AC: *"**Disconnect during terminal events** — Provider sends N tokens then Usage; client disconnects between Token N and Usage. Reconcile MUST report `Cancelled { reason: ClientDisconnect }`, NOT `Success`. The partial_usage may be None or Some — both are acceptable; what matters is the outcome class."*

### ISS-004 — `ReconcileGuard::Drop` relies on `tokio::runtime::Handle::try_current()` which fails during runtime shutdown
- **severity:** warning
- **rule_id:** robustness
- **location:** §6 skeleton (`impl Drop for ReconcileGuard`)
- **status:** open

#### Description
The Drop impl spawns a recovery reconcile via `tokio::runtime::Handle::try_current().ok().and_then(|h| h.spawn(...))`. If the Drop fires during runtime shutdown (e.g., the process is exiting via `Ctrl+C`, or `tokio::runtime::Runtime::shutdown_timeout` is in progress), `try_current()` returns `Err` and the spawn never happens. Result: a panicked spawned task during shutdown leaves the hold in `state="held"` permanently — until the cleanup job (TASK-AI-001 §1 #14) sweeps it.

The spec's §11 note 4 says *"Production code never imports it; it lives in `tests/mocks/reconcile_spy.rs` behind `#[cfg(test)]`"* — but this is about the test spy, not the runtime-shutdown case.

#### Suggested fix
Two-part:

1. **Block_on as last resort** in Drop:

```rust
impl Drop for ReconcileGuard {
    fn drop(&mut self) {
        if self.fired.load(Ordering::SeqCst) { return; }

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            // In-runtime path: spawn recovery.
            let hold = self.hold_id;
            let pool = self.pool.clone();
            handle.spawn(async move {
                metrics::RECONCILES.with_label_values(&["panic_recovery"]).inc();
                let _ = cost_ledger::reconcile(hold, /* InternalError */, &pool).await;
            });
        } else {
            // Out-of-runtime path (shutdown): can't reconcile from here.
            // Log loudly so operators know to expect a held entry; the cleanup
            // job (TASK-AI-001 §1 #14) will sweep it in ≤60s.
            tracing::error!(
                hold_id = ?self.hold_id,
                severity = "sev-2",
                "reconcile_guard_drop_outside_runtime; cleanup job will sweep"
            );
        }
    }
}
```

2. **Add an `ai_streaming_drop_outside_runtime_total` counter** that operators can alarm on. A non-zero rate means the cleanup job is doing work it shouldn't have to.

Add §10 row: *"Drop fires outside tokio runtime (process shutdown, runtime timeout) → reconcile not called from Drop; cleanup job sweeps within 60s; sev-2 log emitted; OBS counter increments."*

### ISS-005 — task-audit skill §3.7 rule 22 (traceparent on outbound SSE) — header propagation into the SSE response stream not asserted
- **severity:** warning
- **rule_id:** authoring-md-§3.7 (rule 22)
- **location:** §1 (no clause about traceparent propagation in SSE response), §3 (StreamEvent schema), §6 (SSE response handler)
- **status:** open

#### Description
The SSE response is a long-lived HTTP connection from gateway → client. Per task-audit skill §3.7 rule 22, the inbound `traceparent` header MUST propagate to (a) the outbound provider HTTPS call (TASK-AI-008 #17 owns this) AND (b) the SSE response headers so downstream consumers (browser, CUO router, OBS) can correlate. Currently §1 says nothing about the SSE response's `traceparent` header. A client subscribing to the stream sees no trace context — they can't link their `tracestate` to the server-side spans. The `X-Request-Id` header is set but not the W3C `traceparent`.

#### Suggested fix
Add §1 #16: "**MUST** set the W3C `traceparent` header in the SSE response (status 200 + Content-Type: text/event-stream) so clients can correlate streaming events with server spans. The value MUST be derived from the inbound request's span context (child span of the inbound trace_id). TASK-AI-022's `tracing-opentelemetry` layer provides the `traceparent::for_span(&current_span())` helper." Add AC #17 verifying via a `reqwest`-based test that the SSE response carries the same `trace_id` (32-hex) as the inbound request.

### ISS-006 — task-audit skill §3.10 rule 29 (failure-mode per architectural decision) — heartbeat-during-disconnect path not in §10
- **severity:** warning
- **rule_id:** authoring-md-§3.10 (rule 29)
- **location:** §10 failure-modes inventory
- **status:** open

#### Description
§10 covers many failure paths but is missing one: client connection abruptly closes (TCP RST) DURING a heartbeat send. The heartbeat is the only event the gateway initiates while the provider stream is silent; if the client RSTs at the same instant, the `tx.send(heartbeat).await` returns `Err` but the recovery logic for heartbeat-during-disconnect is not explicitly documented as one of the §10 rows. Could lead to a missed `ai_streaming_disconnects_total{provider, model, when="heartbeat"}` label value never being incremented because the code only checks the heartbeat send Err and falls through without metric.

#### Suggested fix
Add §10 row: *"Client disconnects mid-heartbeat (TCP RST during the 15s heartbeat send) → `tx.send(heartbeat).await` returns Err; recovery MUST emit `ai_streaming_disconnects_total{provider, model, when="heartbeat"}` and return `StreamResult::Cancelled { reason: ClientDisconnect, partial_usage: None }`. The reconcile_guard fires with InternalError column-floor billing."* Update `metrics::DISCONNECTS` labels to include `when ∈ before_first_token | after_first_token | heartbeat`. Add corresponding AC #18.

## §3 — Strengths preserved through expansion

- §1 grew from 12 to 15 numbered MUST/SHOULD clauses; each MUST has at least one §4 AC and one §6 enforcement point.
- §3 introduces `ErrorCode::as_metric_label` and `ReconcileReason::as_metric_label` following the TASK-AI-007 ISS-003 pattern; OBS labels are rename-safe across the streaming subsystem.
- §3's `ReconcileGuard` RAII pattern correctly encodes the exactly-once invariant (§1 #7); the AtomicBool-swapped-on-fire ensures Drop doesn't double-fire.
- §5 includes dedicated tests for the panic-recovery path (`reconcile_exactly_once_on_panic`) and concurrent isolation (`concurrent_50_streams_isolated`) — both of which catch real-world streaming bugs that simple happy-path tests miss.
- §6 skeleton's metrics module includes 8 distinct counters/histograms; the `RECONCILES` counter is documented in §11 as the leak canary (operator dashboard panel).
- §10 inventory covers 16 distinct paths including the `provider_supports_streaming` table-mismatch row (which would catch a class of refactor bugs).
- §11 explicitly calls out the no-resume contract (no `Last-Event-Id` semantics) and explains why; this prevents future PRs from adding partial-resume features that would compromise the cost-of-everything invariant.

## §4 — Resolution

All 6 mechanical revisions applied:
- ISS-001 RESOLVED (2026-05-16): §5 `unsupported_model_falls_back_silently` test added (AC #9) and `heartbeat_every_15s` tightened to assert metric/event count parity (AC #11).
- ISS-002 RESOLVED (2026-05-16): §6 `run_provider_stream` loop uses `tokio::select!` racing disconnect-watcher vs provider stream; `ABORT_TIMEOUT` hard upper bound per select-iteration; AC #15 added.
- ISS-003 RESOLVED (2026-05-16): §6 Usage and Done send paths check `.is_err()` and return `Cancelled { reason: ClientDisconnect }`; AC #16 added.
- ISS-004 RESOLVED (2026-05-16): §6 `ReconcileGuard::Drop` branches on `Handle::try_current()` — spawns recovery if runtime alive, else sev-2 log + `ai_streaming_drop_outside_runtime_total`; §10 row added.
- ISS-005 RESOLVED (2026-05-16, task-audit skill compliance pass): §1 #16 added asserting W3C `traceparent` header in SSE response; AC #17 added (reqwest-based trace_id correlation test).
- ISS-006 RESOLVED (2026-05-16, task-audit skill compliance pass): §10 row added for client-disconnect-during-heartbeat; `metrics::DISCONNECTS` labels expanded to include `when ∈ before_first_token | after_first_token | heartbeat`; AC #18 added.

**Score = 10/10.** Ship as-is. Ready to transition `draft → accepted`.

---

*End of TASK-AI-010 audit (final). Status: PASS at 10/10.*
