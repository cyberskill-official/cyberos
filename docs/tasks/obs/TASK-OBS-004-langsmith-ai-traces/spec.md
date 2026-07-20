---
id: TASK-OBS-004
title: "LangSmith integration for AI traces — self-hosted + per-tenant opt-in + redacted-prompts-only + W3C TraceContext correlation + async non-blocking"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: OBS
priority: p0
status: ready_to_implement
verify: T
phase: P0
milestone: P0 · slice 2
slice: 2
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_tasks: [TASK-AI-011, TASK-AI-014, TASK-AI-022, TASK-OBS-001, TASK-OBS-005]
depends_on: [TASK-AI-022, TASK-OBS-001]
blocks: [TASK-OBS-005]

source_pages:
  - website/docs/modules/obs.html#langsmith
source_decisions:
  - DEC-155 (self-hosted LangSmith; SaaS langchain.com sends data to US — PDPL violation for VN tenants)
  - DEC-156 (per-tenant opt-in; default false; explicit consent for AI-trace-detail export)
  - DEC-157 (REDACTED prompt only — never raw PII; LangSmith storage MUST NOT become a parallel PII repo)
  - DEC-158 (async non-blocking; LangSmith outage doesn't stall gateway)

language: rust 1.81
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/src/langsmith/mod.rs
  - services/ai-gateway/src/langsmith/client.rs
  - services/ai-gateway/src/langsmith/payload.rs
  - services/ai-gateway/tests/langsmith_test.rs
  - services/ai-gateway/tests/langsmith_no_pii_test.rs
  - services/ai-gateway/tests/langsmith_async_test.rs
  - deploy/obs/langsmith-config.yaml
  - deploy/obs/langsmith-docker-compose.yml
modified_files:
  # call export_to_langsmith on response
  - services/ai-gateway/src/router/mod.rs
  # propagate W3C trace_id to LangSmith
  - services/ai-gateway/src/handlers/chat.rs
  # add ai_policy.langsmith_export field
  - services/ai-gateway/src/policy.rs
  # langsmith service
  - deploy/obs/docker-compose.yml
allowed_tools:
  - file_read: services/ai-gateway/**, deploy/obs/**
  - file_write: services/ai-gateway/{src,tests}/**, deploy/obs/**
  - bash: cd services/ai-gateway && cargo test langsmith
disallowed_tools:
  #4)
  - send raw (un-redacted) prompts to LangSmith — must be the TASK-AI-011 redacted version (per §1
  - send to langchain.com or any non-CyberOS LangSmith instance (per DEC-155)
  #6 — async only)
  - block the gateway hot-path on LangSmith export (per §1
  #3 — default false)
  - export for tenants without explicit opt-in (per §1

effort_hours: 6
subtasks:
  - "0.5h: ai_policy.langsmith_export bool field + TASK-AI-005 schema extension"
  - "0.5h: docker-compose.yml langsmith service (self-hosted release)"
  - "1.0h: langsmith/payload.rs — trace JSON shape (LangSmith-compatible)"
  - "1.0h: langsmith/client.rs — async POST with reqwest + bearer auth + 2s timeout"
  - "1.0h: tokio::spawn fire-and-forget pattern (no await in caller hot-path)"
  - "0.5h: W3C trace_id correlation (LangSmith trace_id == OTel trace_id)"
  - "0.5h: REDACTED-prompt assertion at the export boundary (caller MUST pass redacted)"
  - "0.5h: OTel metric ai_langsmith_exports_total + retry queue (3 attempts then drop)"
  - "1.5h: Tests — opt-in/opt-out + no-PII assertion + async non-blocking + LangSmith-down + correlation + payload shape"
risk_if_skipped: "AI-specific debugging (prompt evaluation, tool-call success rate, model regression analysis) impossible. Operational traces (Tempo via TASK-AI-022) show 'AI call took 2s' but not 'because the model decoded a tool-call wrong' or 'because the prompt template returned an empty placeholder'. ISO 42001 AIMS Phase 3 obligation unmet (AI-system observability). Without LangSmith, prompt-quality regressions are invisible until customers report bad outputs. Without per-tenant opt-in, exporting prompt detail violates PDPL Art. 7 for VN tenants who didn't consent."
---

## §1 — Description (BCP-14 normative)

The AI Gateway **MUST** emit LangSmith traces for every LLM call, linked to the operational trace_id from TASK-AI-022. Each export:

1. **MUST** create a LangSmith trace per AI call. The LangSmith `trace_id` field is set to the OTel `trace_id` (W3C TraceContext) — same value, same hex format, enabling cross-tool correlation: opening a Tempo trace AND a LangSmith trace for the same call shows both views of the same operation.
2. **MUST** include in the exported payload:
- Model alias + resolved model + provider.
- REDACTED prompt (post-TASK-AI-011 redaction; placeholders only — no raw PII).
- REDACTED response (placeholders only).
- LLM hints actually sent (temperature, max_tokens, stop sequences).
- Latency (request → response total).
- Cost (USD; from cost_ledger).
- Persona handle (TASK-AI-014).
- Trace_id (matches OTel).
- Tenant_id (which tenant; auditable).
- Tool calls (if any) with tool name + redacted args + tool result outcome.
3. **MUST** respect per-tenant `policy.ai_policy.langsmith_export: bool`; default `false`. Only enable for tenants who explicitly opt in via `cyberos-ai policy set <tenant> --langsmith-export=true --confirm`. The opt-in is logged via TASK-AI-021's CLI audit row + a separate `obs.langsmith_export_enabled` memory row.
4. **MUST** route to self-hosted LangSmith at `https://langsmith.cyberos.world` (per-region deployment for TASK-AI-016 residency). The langchain.com SaaS endpoint is forbidden — it routes data to US infrastructure, violating PDPL/GDPR for VN/EU tenants. Per-region URL config in `deploy/obs/langsmith-config.yaml`.
5. **MUST** assert at the export boundary that the prompt + response passed in are the REDACTED versions. The export function signature accepts `RedactedPrompt(String)` newtype (from TASK-AI-011); raw `String` is a compile error. Defense-in-depth against accidental raw export.
6. **MUST** be asynchronous (fire-and-forget via `tokio::spawn`); the gateway hot-path doesn't await the LangSmith POST. The export adds ≤ 1ms to per-request latency (just the spawn cost).
7. **MUST NOT** block the gateway response on LangSmith availability. LangSmith unreachable → log `tracing::warn!("langsmith_export_failed reason=<r>")` + increment metric; the gateway returns the LLM response normally.
8. **MUST** retry up to 3 attempts with exponential backoff (100ms, 250ms, 500ms) before dropping. After 3 failures, drop with WARN; metric `ai_langsmith_exports_total{outcome=dropped_after_retries}`.
9. **MUST** authenticate to LangSmith via bearer token (per-environment, rotated quarterly per TASK-AUTH-006-style sweeper). Token in env var `LANGSMITH_API_TOKEN`; never logged.
10. **MUST** emit OTel metrics:
- `ai_langsmith_exports_total{outcome, tenant_id}` (counter; outcome ∈ ok | dropped_opt_out | dropped_after_retries | langsmith_unreachable | invalid_payload).
- `ai_langsmith_export_latency_ms` (histogram; the async export's wall-clock).
- `ai_langsmith_queue_depth` (gauge; pending tokio::spawn'd exports).
11. **MUST** include `Idempotency-Key` header per export (the OTel trace_id is a natural unique key) to prevent duplicate ingestion if a retry succeeds after the original eventually delivers.
12. **SHOULD** truncate redacted prompts > 100KB to first 100KB + `"...[truncated by TASK-OBS-004]"` marker. LangSmith UI degrades on huge payloads; truncation preserves the diagnostic value.

---

## §2 — Why this design (rationale for humans)

**Why LangSmith specifically?** Standard observability layer for LLM apps — purpose-built for prompt-quality + tool-call analysis (Tempo handles operational traces but doesn't surface "which prompt produced which response, ranked by quality"). Self-hosted version (open-source release) keeps data in CyberOS residency boundary.

**Why per-tenant opt-in (DEC-156)?** LangSmith stores prompt detail by design. Even REDACTED prompts may contain tenant-business semantics (e.g., a query about a specific product line). Exporting without explicit consent is a privacy boundary violation. Default-off + explicit enablement preserves tenant agency.

**Why REDACTED prompts only (DEC-157)?** Without redaction, LangSmith becomes a parallel PII repository — every CCCD/email/phone the gateway processed shows up in LangSmith UI, accessible to anyone with LangSmith admin access. The compile-time `RedactedPrompt` newtype prevents the wrong type from being passed.

**Why async non-blocking (DEC-158)?** Synchronous LangSmith export would add ~50-200ms to every gateway response (network + LangSmith ingestion). Async fire-and-forget adds ~1ms (spawn cost). The trade-off: rare LangSmith outages don't stall the gateway; the cost is occasional dropped traces (which the metric tracks).

**Why W3C TraceContext correlation (§1 #1)?** The same call has TWO views: Tempo (operational — span tree, latency, errors) AND LangSmith (LLM-specific — prompt, response, tool calls). Without shared trace_id, correlating the two requires manual timestamp matching. With shared trace_id, the operator opens both UIs side-by-side filtering by the same id.

**Why retry with backoff (§1 #8)?** Transient LangSmith errors (network blip, brief overload) recover quickly. 3 retries with exponential backoff covers the common transient cases without indefinite retry queues. Permanent failures (auth failed, malformed payload) drop after the first attempt + appropriate metric label.

**Why Idempotency-Key (§1 #11)?** A retry that succeeds after the original eventually delivers would create a duplicate trace in LangSmith. The trace_id as idempotency key tells LangSmith "if you already have this trace, deduplicate." Standard idempotency pattern.

**Why per-region LangSmith (§1 #4)?** TASK-AI-016 residency pinning applies: a `Sg1` tenant's traces shouldn't ship to an EU-region LangSmith. Per-region deployment (`langsmith.sg-1.cyberos.world`, `langsmith.eu-1.cyberos.world`) keeps data in-region. Single global LangSmith would force all data to one region.

**Why tokio::spawn instead of a queue worker (§1 #6)?** A queue + worker would centralise but adds complexity (worker pool sizing, queue backpressure). At our volume (~100 LLM calls/sec at slice-2 scale), tokio::spawn handles concurrency natively; the runtime's task scheduler is the worker. The metric `ai_langsmith_queue_depth` tracks pending tasks.

**Why payload size limit at 100KB (§1 #12)?** LangSmith UI degrades on huge payloads (slow load times, browser crashes on multi-MB JSON). 100KB covers 99% of real LLM exchanges; the rare oversized cases (full-document RAG context) get truncated with a marker so the operator knows to inspect via Tempo (which doesn't truncate).

---

## §3 — API contract

```rust
// services/ai-gateway/src/langsmith/mod.rs
use std::sync::Arc;
use uuid::Uuid;

pub struct RedactedPrompt(pub String);     // newtype: only redacted strings can be passed
pub struct RedactedResponse(pub String);

#[derive(serde::Serialize)]
pub struct LangSmithMetadata {
    pub model_alias: String,
    pub resolved_model: String,
    pub provider: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub latency_ms: u32,
    pub cost_usd: f64,
    pub persona_handle: String,
    pub tenant_id: Uuid,
    pub trace_id: String,                  // matches OTel hex format
    pub tool_calls: Vec<ToolCallTrace>,
}

#[derive(serde::Serialize)]
pub struct ToolCallTrace {
    pub tool_name: String,
    pub redacted_args: RedactedPrompt,
    pub outcome: String,                   // success | error | timeout
}

pub async fn export(
    trace_id: opentelemetry::trace::TraceId,
    redacted_prompt: RedactedPrompt,
    redacted_response: RedactedResponse,
    metadata: LangSmithMetadata,
    tenant_policy: &TenantPolicy,
) {
    // §1 #3: opt-in check
    if !tenant_policy.ai_policy.langsmith_export {
        metrics::export_dropped(metadata.tenant_id, "opt_out");
        return;
    }

    // §1 #6: fire-and-forget
    let payload = build_payload(trace_id, redacted_prompt, redacted_response, metadata);
    tokio::spawn(async move {
        let result = client::post_with_retry(&payload).await;
        match result {
            Ok(_) => metrics::export_ok(payload.metadata.tenant_id),
            Err(e) => {
                tracing::warn!(error = %e, trace_id = %trace_id, "langsmith_export_failed");
                metrics::export_dropped(payload.metadata.tenant_id, &e.to_string());
            }
        }
    });
}
```

```rust
// services/ai-gateway/src/langsmith/client.rs
const LANGSMITH_URL: &str = env::var("LANGSMITH_URL")
    .unwrap_or_else(|_| "https://langsmith.cyberos.world".into());
const LANGSMITH_TOKEN: &str = env::var("LANGSMITH_API_TOKEN").unwrap_or_default();
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
const RETRY_DELAYS_MS: &[u64] = &[100, 250, 500];

pub async fn post_with_retry(payload: &Payload) -> Result<(), LangSmithError> {
    let client = reqwest::Client::builder().timeout(REQUEST_TIMEOUT).build()?;
    let mut last_error = None;
    for (attempt, delay) in RETRY_DELAYS_MS.iter().enumerate() {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_millis(*delay)).await;
        }
        let res = client.post(format!("{LANGSMITH_URL}/api/v1/traces"))
            .header("Authorization", format!("Bearer {LANGSMITH_TOKEN}"))
            .header("Idempotency-Key", payload.metadata.trace_id.clone())
            .json(payload)
            .send().await;
        match res {
            Ok(r) if r.status().is_success() => return Ok(()),
            Ok(r) if r.status().as_u16() == 401 => return Err(LangSmithError::AuthFailed),
            Ok(r) if r.status().is_client_error() => return Err(LangSmithError::InvalidPayload(r.status().as_u16())),
            Ok(r) => last_error = Some(LangSmithError::ServerError(r.status().as_u16())),
            Err(e) => last_error = Some(LangSmithError::Network(e.to_string())),
        }
    }
    Err(last_error.unwrap_or(LangSmithError::DroppedAfterRetries))
}
```

```rust
// services/ai-gateway/src/langsmith/payload.rs
const MAX_PROMPT_BYTES: usize = 100 * 1024;

pub fn build_payload(
    trace_id: TraceId, redacted_prompt: RedactedPrompt,
    redacted_response: RedactedResponse, metadata: LangSmithMetadata,
) -> Payload {
    let prompt = if redacted_prompt.0.len() > MAX_PROMPT_BYTES {
        format!("{}...[truncated by TASK-OBS-004]", &redacted_prompt.0[..MAX_PROMPT_BYTES])
    } else { redacted_prompt.0 };
    let response = if redacted_response.0.len() > MAX_PROMPT_BYTES {
        format!("{}...[truncated by TASK-OBS-004]", &redacted_response.0[..MAX_PROMPT_BYTES])
    } else { redacted_response.0 };
    Payload { trace_id: format!("{trace_id:032x}"), prompt, response, metadata }
}
```

```yaml
# deploy/obs/langsmith-docker-compose.yml
services:
  langsmith:
    image: langchain/langsmith:0.6.0   # self-hosted release
    ports: ["8090:8080"]
    environment:
      DATABASE_URL: postgres://langsmith:pass@postgres-langsmith/langsmith
      LANGSMITH_RETENTION_DAYS: 30
    volumes: [langsmith-data:/data]
    depends_on: [postgres-langsmith]
  postgres-langsmith:
    image: postgres:16
    environment: { POSTGRES_PASSWORD: pass, POSTGRES_DB: langsmith }
    volumes: [postgres-langsmith-data:/var/lib/postgresql/data]

volumes: { langsmith-data:, postgres-langsmith-data: }
```

---

## §4 — Acceptance criteria

1. **Opt-in tenant: AI call produces LangSmith trace** with matching trace_id (hex format).
2. **Opt-out tenant: no LangSmith export** — metric `ai_langsmith_exports_total{outcome=dropped_opt_out}` increments.
3. **LangSmith unreachable: gateway response unaffected** — gateway returns LLM response normally; warn log emitted.
4. **3 retries with exponential backoff before drop** — synthetic LangSmith returning 500 → 3 attempts at 100ms/250ms/500ms then `outcome=dropped_after_retries`.
5. **Auth failed (401) drops immediately** — no retry; `outcome=auth_failed`; sev-2 alarm.
6. **Exported trace contains REDACTED prompts (no PII)** — assert no CCCD/email/phone-shape strings in payload.
7. **`RedactedPrompt` newtype prevents raw export** — passing `String` directly is a compile error.
8. **Async non-blocking: gateway adds ≤ 1ms** — measured via `obs_proxy_injection_latency_ms`-style benchmark.
9. **Idempotency-Key header set** — payload includes `Idempotency-Key: <trace_id>`.
10. **Per-region routing** — Sg1 tenant exports to `langsmith.sg-1.cyberos.world`; Eu1 to `langsmith.eu-1.cyberos.world`.
11. **Truncation > 100KB** — 200KB prompt → exported version is 100KB + `"...[truncated by TASK-OBS-004]"`.
12. **trace_id correlation** — same call's Tempo trace + LangSmith trace share trace_id.
13. **Tool calls included** — multi-step tool-using call produces `tool_calls` array with each tool's name + redacted args + outcome.
14. **Metric ai_langsmith_queue_depth observable** — under load, queue depth metric increases.
15. **Self-hosted (not langchain.com)** — outgoing URL points at `https://langsmith.cyberos.world`; integration test asserts.
16. **Per-tenant opt-in audit row** — enabling `langsmith_export` via TASK-AI-021 CLI emits `obs.langsmith_export_enabled` memory row.
17. **Cost field accurate** — exported `cost_usd` matches TASK-AI-001 cost_ledger value.

---

## §5 — Verification

```rust
// services/ai-gateway/tests/langsmith_test.rs
#[tokio::test]
async fn opt_in_tenant_produces_langsmith_trace() {
    let langsmith_mock = MockLangSmith::start();
    let policy = test_policy_with_langsmith_export(true);
    let trace_id = test_trace_id();
    langsmith::export(trace_id, redacted("hello"), redacted("hi"), test_metadata(), &policy).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    let received = langsmith_mock.last_received().await;
    assert_eq!(received.trace_id, format!("{trace_id:032x}"));
}

#[tokio::test]
async fn opt_out_tenant_no_export() {
    let langsmith_mock = MockLangSmith::start();
    let policy = test_policy_with_langsmith_export(false);
    langsmith::export(test_trace_id(), redacted("p"), redacted("r"), test_metadata(), &policy).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(langsmith_mock.received_count().await, 0);
    let metric = otel_test_helper::counter_value("ai_langsmith_exports_total",
        &[("outcome", "dropped_opt_out")]);
    assert_eq!(metric, 1);
}

#[tokio::test]
async fn unreachable_langsmith_does_not_block_gateway() {
    let policy = test_policy_with_langsmith_export(true);
    let t0 = std::time::Instant::now();
    langsmith::export(test_trace_id(), redacted("p"), redacted("r"),
                      test_metadata_with_url("http://127.0.0.1:9999/dead"), &policy).await;
    let elapsed = t0.elapsed();
    assert!(elapsed < Duration::from_millis(5), "gateway blocked for {elapsed:?}");
}

#[tokio::test]
async fn retries_3_times_with_backoff() {
    let langsmith_mock = MockLangSmith::start_returning(500);
    let _ = langsmith::client::post_with_retry(&test_payload()).await.expect_err("expected drop");
    assert_eq!(langsmith_mock.received_count().await, 3);
}

#[tokio::test]
async fn auth_failed_drops_immediately_no_retry() {
    let langsmith_mock = MockLangSmith::start_returning(401);
    let err = langsmith::client::post_with_retry(&test_payload()).await.expect_err("expected AuthFailed");
    assert!(matches!(err, LangSmithError::AuthFailed));
    assert_eq!(langsmith_mock.received_count().await, 1);
}

#[test]
fn redacted_prompt_newtype_is_compile_safe() {
    // This file must compile (the type system enforces; if the wrong type is allowed, this test wouldn't even appear).
    let _: RedactedPrompt = RedactedPrompt("safe".into());
    // The line below would NOT compile — uncomment to verify:
    // let _: RedactedPrompt = "raw".to_string();
}

#[test]
fn truncation_at_100kb() {
    let big = RedactedPrompt("x".repeat(200_000));
    let payload = build_payload(test_trace_id(), big, redacted("r"), test_metadata());
    assert!(payload.prompt.len() <= 100 * 1024 + 50);
    assert!(payload.prompt.ends_with("...[truncated by TASK-OBS-004]"));
}

// services/ai-gateway/tests/langsmith_no_pii_test.rs
#[tokio::test]
async fn exported_payload_contains_no_pii_patterns() {
    let policy = test_policy_with_langsmith_export(true);
    let prompt = redacted("hello <VN_CCCD_1> world <VN_PHONE_1>");
    langsmith::export(test_trace_id(), prompt, redacted("ok"), test_metadata(), &policy).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    let received = MockLangSmith::singleton().last_received().await;
    let json = serde_json::to_string(&received).unwrap();

    let cccd_re = regex::Regex::new(r"\b\d{12}\b").unwrap();
    let phone_re = regex::Regex::new(r"\b0\d{9}\b").unwrap();
    let email_re = regex::Regex::new(r"\b[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}\b").unwrap();
    assert!(!cccd_re.is_match(&json));
    assert!(!phone_re.is_match(&json));
    assert!(!email_re.is_match(&json));
    assert!(json.contains("<VN_CCCD_1>"));   // placeholder still present
}
```

```bash
docker compose -f deploy/obs/langsmith-docker-compose.yml up -d
cd services/ai-gateway && cargo test langsmith
```

---

## §6 — Implementation skeleton

See §3.

```rust
// services/ai-gateway/src/router/mod.rs (modified)
async fn handle_request(req: ChatCompleteRequest, policy: &TenantPolicy) -> Result<Response, ApiError> {
    let trace_id = opentelemetry::Context::current().span().span_context().trace_id();
    // ... existing flow: precheck + alias + redact + provider + reconcile ...

    // §1 #6: async fire-and-forget, gateway response not blocked.
    langsmith::export(trace_id,
        RedactedPrompt(redacted_prompt),
        RedactedResponse(redacted_response),
        LangSmithMetadata { /* ... */ },
        policy,
    ).await;   // returns immediately; tokio::spawn happens inside
}
```

---

## §7 — Dependencies

- **TASK-AI-022** — Source of trace_id for correlation.
- **TASK-AI-011** — Source of redacted prompts/responses.
- **TASK-AI-014** — Persona handle in metadata.
- **TASK-AI-001** — Cost in metadata.
- **TASK-AI-005** — Tenant policy schema includes `langsmith_export` field.
- **TASK-AI-021** — CLI enables/disables per tenant + emits audit row.
- Crates: `reqwest@0.12`, `tokio`, `serde`, `serde_json`, `chrono`.
- Self-hosted LangSmith (open-source release).

---

## §8 — Example payloads

### LangSmith trace POST

```http
POST https://langsmith.cyberos.world/api/v1/traces HTTP/1.1
Authorization: Bearer <LANGSMITH_API_TOKEN>
Content-Type: application/json
Idempotency-Key: 0af7651916cd43dd8448eb211c80319c

{
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "prompt": "User asked: <VN_CCCD_1>, please summarise.",
  "response": "Summary: <PERSON_1>'s ID was provided.",
  "metadata": {
    "model_alias": "chat.smart",
    "resolved_model": "anthropic.claude-3-5-sonnet-20241022-v2:0",
    "provider": "bedrock",
    "temperature": 0.4,
    "max_tokens": 1024,
    "latency_ms": 1450,
    "cost_usd": 0.0078,
    "persona_handle": "cuo-cpo@0.4.1",
    "tenant_id": "550e8400-...",
    "tool_calls": []
  }
}
```

### Failed export warn log

```text
WARN trace_id=0af765... reason=langsmith_unreachable
     langsmith_export_failed; metric ai_langsmith_exports_total{outcome=langsmith_unreachable} incremented
```

### `obs.langsmith_export_enabled` audit row

```json
{
  "kind": "obs.langsmith_export_enabled",
  "payload": {
    "tenant_id": "550e8400-...",
    "enabled_by_subject_id": "...",
    "request_id": "cli_..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- LangSmith eval runs (regression-test prompts) — slice 5+.
- Custom datasets per tenant — slice 5+.
- LangSmith UI tenant-isolation (tenant-admin sees only own traces) — slice 4+.
- Sampling at the export layer (not every call exports if tenant is high-volume) — slice 4+ if storage growth becomes an issue.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| LangSmith down | reqwest connect error | 3 retries → drop; warn log | Auto-resume when up |
| LangSmith slow (> 2s timeout) | tokio timeout | Retry next attempt | Self-resolves |
| Tenant opted out | policy check | No export; metric increments | By design |
| Token expired | LangSmith 401 | Drop immediately (no retry); sev-2 alarm | Operator rotates token |
| Token leaked | Unknown caller | Sev-1; rotate token | Standard incident response |
| Raw prompt accidentally exported (regression) | RedactedPrompt newtype enforces at compile time | PR fails to compile | By design |
| Exported PII (post-redaction bug) | `langsmith_no_pii_test` regex check | PR blocked | Investigate TASK-AI-011 |
| Per-region URL misconfigured | Integration test asserts | PR blocked | Operator fixes config |
| Async export queue grows unbounded | metric `ai_langsmith_queue_depth` alarm | Sev-3 | Investigate LangSmith throughput |
| Truncation marker missed | unit test asserts | PR blocked | Fix payload builder |
| Duplicate trace from retry success after original | Idempotency-Key | LangSmith deduplicates | By design |
| Trace_id format mismatch (LangSmith vs OTel) | correlation test asserts hex format | Test fails → PR blocked | Format consistently |
| Cost field stale | snapshot at export time | Cost reflects measurement at request | By design |
| Mock LangSmith vs production drift | Integration tests use real schema | Test fails on schema change | Update mock |
| Tool calls missing in payload | unit test asserts | Fix payload builder | By design |
| Per-tenant opt-in audit row missing | TASK-AI-021 emits | CLI bug | Investigate CLI |
| LangSmith disk fills | LangSmith-side alert | Operator extends storage | Standard ops |
| LangSmith schema changes | Integration test fails | Update payload struct | Pin LangSmith version |
| Bypass: someone calls langchain.com | Outgoing-URL allowlist | Rejected | Operator configures egress firewall |

---

## §11 — Notes

- Self-host LangSmith via the open-source release; commercial features (eval, datasets) at slice 5+ if needed.
- The "REDACTED prompt only" rule is critical — LangSmith trace storage MUST NOT become a parallel PII repository. The `RedactedPrompt` newtype enforces at compile time; the no-PII test enforces at CI time.
- Per-tenant opt-in default false respects PDPL Art. 7. Operators enable per-tenant via TASK-AI-021 CLI; opt-in is auditable.
- W3C trace_id correlation lets operators open Tempo + LangSmith side-by-side for the same call. The two views are complementary — Tempo for operational (latency, errors), LangSmith for LLM-specific (prompt, tool calls).
- Async fire-and-forget keeps LangSmith outages from cascading to gateway latency. The trade-off is occasional dropped traces during prolonged outages — tracked via `ai_langsmith_exports_total{outcome=dropped_after_retries}`.
- 3 retries with exponential backoff (100/250/500ms = 850ms total) covers transient errors without indefinite queuing. Permanent errors (auth, malformed payload) drop on first attempt.
- Idempotency-Key (the trace_id) prevents duplicate-trace ingestion if a retry succeeds after the original eventually delivers.
- 100KB payload truncation prevents LangSmith UI degradation. Rare oversized cases (full-document RAG) get truncated with a marker so the operator knows to inspect via Tempo.
- Per-region LangSmith deployment (sg-1, eu-1) satisfies TASK-AI-016 residency. Tenants pinning to vn-1 currently can't use LangSmith (no VN deployment); the `dropped_residency_no_langsmith` metric label tracks this.
- The langchain.com SaaS endpoint is forbidden via egress firewall + integration-test assertion. A rogue commit pointing at langchain.com would fail the integration test before merge.

---

*End of TASK-OBS-004. Status: draft (10/10 target).*
