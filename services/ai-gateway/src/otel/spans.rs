//! Span construction and propagation helpers for FR-AI-022.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use once_cell::sync::Lazy;
use opentelemetry::trace::{SpanKind, Status, TraceContextExt, Tracer};
use opentelemetry::{global, Context, KeyValue};
use tracing::warn;

use super::attributes;
use super::propagation;
use crate::router::{ChatCompleteRequest, EmbedRequest};

pub const CHAT_COMPLETION_SPAN: &str = "ai_gateway.chat_completion";
pub const EMBED_SPAN: &str = "ai_gateway.embed";
pub const RERANK_SPAN: &str = "ai_gateway.rerank";
pub const PRECHECK_SPAN: &str = "ai_gateway.precheck";
pub const ALIAS_RESOLVE_SPAN: &str = "ai_gateway.alias_resolve";
pub const PERSONA_LOAD_SPAN: &str = "ai_gateway.persona_load";
pub const ZDR_CHECK_SPAN: &str = "ai_gateway.zdr_check";
pub const RESIDENCY_CHECK_SPAN: &str = "ai_gateway.residency_check";
pub const CACHE_LOOKUP_SPAN: &str = "ai_gateway.cache_lookup";
pub const REDACT_SPAN: &str = "ai_gateway.redact";
pub const PROVIDER_CALL_SPAN: &str = "ai_gateway.provider_call";
pub const RECONCILE_SPAN: &str = "ai_gateway.reconcile";

const RETRY_EVENT: &str = "retry.attempt";

static FINISHED_SPANS: Lazy<Mutex<Vec<FinishedSpanRecord>>> = Lazy::new(|| Mutex::new(Vec::new()));
static NEXT_SPAN_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishedSpanRecord {
    pub name: String,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub status: RecordedSpanStatus,
    pub attributes: Vec<(String, String)>,
    pub events: Vec<SpanEventRecord>,
    pub duration_us: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanEventRecord {
    pub name: String,
    pub attributes: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordedSpanStatus {
    Ok,
    Error,
}

pub struct OtelSpan {
    name: &'static str,
    ctx: Context,
    trace_id: String,
    span_id: String,
    parent_span_id: Option<String>,
    started: Instant,
    attributes: Vec<(String, String)>,
    events: Vec<SpanEventRecord>,
    status: Option<RecordedSpanStatus>,
    ended: bool,
}

impl std::fmt::Debug for OtelSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OtelSpan")
            .field("name", &self.name)
            .field("trace_id", &self.trace_id)
            .field("span_id", &self.span_id)
            .field("parent_span_id", &self.parent_span_id)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl OtelSpan {
    pub fn child(&self, name: &'static str, kind: SpanKind) -> Self {
        start_span(name, &self.ctx, kind, Some(self.span_id.clone()))
    }

    pub fn set_str(&mut self, key: &'static str, value: impl Into<String>) {
        let value = value.into();
        self.ctx
            .span()
            .set_attribute(KeyValue::new(key, value.clone()));
        self.attributes.push((key.to_string(), value));
    }

    pub fn set_bool(&mut self, key: &'static str, value: bool) {
        self.ctx.span().set_attribute(KeyValue::new(key, value));
        self.attributes.push((key.to_string(), value.to_string()));
    }

    pub fn set_i64(&mut self, key: &'static str, value: i64) {
        self.ctx.span().set_attribute(KeyValue::new(key, value));
        self.attributes.push((key.to_string(), value.to_string()));
    }

    pub fn add_retry_event(
        &mut self,
        attempt_num: u8,
        backoff_ms: u64,
        prior_status_code: Option<u16>,
    ) {
        let prior_status = prior_status_code
            .map(|status| status.to_string())
            .unwrap_or_else(|| "none".to_string());
        let attrs = vec![
            KeyValue::new(attributes::RETRY_ATTEMPT, i64::from(attempt_num)),
            KeyValue::new(attributes::RETRY_BACKOFF_MS, backoff_ms as i64),
            KeyValue::new(attributes::RETRY_PRIOR_STATUS, prior_status.clone()),
        ];
        self.ctx.span().add_event(RETRY_EVENT, attrs);
        self.events.push(SpanEventRecord {
            name: RETRY_EVENT.to_string(),
            attributes: vec![
                (
                    attributes::RETRY_ATTEMPT.to_string(),
                    attempt_num.to_string(),
                ),
                (
                    attributes::RETRY_BACKOFF_MS.to_string(),
                    backoff_ms.to_string(),
                ),
                (attributes::RETRY_PRIOR_STATUS.to_string(), prior_status),
            ],
        });
    }

    pub fn traceparent(&self) -> String {
        format!("00-{}-{}-01", self.trace_id, self.span_id)
    }

    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    pub fn span_id(&self) -> &str {
        &self.span_id
    }

    pub fn end_ok(&mut self) {
        self.status = Some(RecordedSpanStatus::Ok);
        self.ctx.span().set_status(Status::Ok);
        self.end();
    }

    pub fn end_error(&mut self, description: impl Into<String>) {
        self.status = Some(RecordedSpanStatus::Error);
        self.ctx
            .span()
            .set_status(Status::error(description.into()));
        self.end();
    }

    fn end(&mut self) {
        if self.ended {
            return;
        }
        self.ended = true;
        self.ctx.span().end();
        let mut spans = FINISHED_SPANS
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        spans.push(FinishedSpanRecord {
            name: self.name.to_string(),
            trace_id: self.trace_id.clone(),
            span_id: self.span_id.clone(),
            parent_span_id: self.parent_span_id.clone(),
            status: self.status.unwrap_or(RecordedSpanStatus::Error),
            attributes: self.attributes.clone(),
            events: self.events.clone(),
            duration_us: self.started.elapsed().as_micros(),
        });
    }
}

pub fn clear_finished_spans() {
    FINISHED_SPANS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clear();
}

pub fn finished_spans() -> Vec<FinishedSpanRecord> {
    FINISHED_SPANS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

pub fn start_chat_root(
    req: &ChatCompleteRequest,
    tenant_id: &str,
    request_id: &str,
    stream: bool,
) -> OtelSpan {
    let mut root = start_root_from_headers(
        CHAT_COMPLETION_SPAN,
        req.traceparent.as_deref(),
        req.tracestate.as_deref(),
    );
    root.set_str(attributes::TENANT_ID, tenant_id);
    root.set_str(attributes::MODEL_ALIAS, &req.alias);
    root.set_str(attributes::REQUEST_ID, request_id);
    root.set_bool(attributes::STREAM, stream);
    if let Some(persona) = &req.agent_persona {
        root.set_str(attributes::AGENT_PERSONA, persona);
    }
    root
}

pub fn start_embed_root(req: &EmbedRequest, model_alias: &str, request_id: &str) -> OtelSpan {
    let mut root = start_span(EMBED_SPAN, &Context::new(), SpanKind::Server, None);
    root.set_str(attributes::TENANT_ID, &req.tenant_id);
    root.set_str(attributes::MODEL_ALIAS, model_alias);
    root.set_str(attributes::REQUEST_ID, request_id);
    root.set_str(attributes::REGION, &req.region);
    root.set_bool(attributes::STREAM, false);
    root
}

pub fn start_rerank_root(tenant_id: &str, model_alias: &str, request_id: &str) -> OtelSpan {
    let mut root = start_span(RERANK_SPAN, &Context::new(), SpanKind::Server, None);
    root.set_str(attributes::TENANT_ID, tenant_id);
    root.set_str(attributes::MODEL_ALIAS, model_alias);
    root.set_str(attributes::REQUEST_ID, request_id);
    root.set_bool(attributes::STREAM, false);
    root
}

pub fn start_precheck_span(
    tenant_id: &str,
    agent_persona: &str,
    model_alias: &str,
    idempotency_key: &str,
) -> OtelSpan {
    let mut span = start_span(PRECHECK_SPAN, &Context::new(), SpanKind::Internal, None);
    span.set_str(attributes::TENANT_ID, tenant_id);
    span.set_str(attributes::AGENT_PERSONA, agent_persona);
    span.set_str(attributes::MODEL_ALIAS, model_alias);
    span.set_str(attributes::IDEMPOTENCY_KEY, idempotency_key);
    span
}

pub fn start_reconcile_span(hold_id: &str) -> OtelSpan {
    let mut span = start_span(RECONCILE_SPAN, &Context::new(), SpanKind::Internal, None);
    span.set_str(attributes::REQUEST_ID, hold_id);
    span
}

pub fn start_detached_provider_span(req: &ChatCompleteRequest) -> OtelSpan {
    let mut span = start_root_from_headers(
        PROVIDER_CALL_SPAN,
        req.traceparent.as_deref(),
        req.tracestate.as_deref(),
    );
    span.set_str(attributes::MODEL_ALIAS, &req.alias);
    if let Some(persona) = &req.agent_persona {
        span.set_str(attributes::AGENT_PERSONA, persona);
    }
    span
}

pub fn baggage_header(tenant_id: &str, agent_persona: Option<&str>, request_id: &str) -> String {
    let mut entries = vec![
        format!("tenant_id={}", baggage_escape(tenant_id)),
        format!("request_id={}", baggage_escape(request_id)),
    ];
    if let Some(agent_persona) = agent_persona.filter(|value| !value.is_empty()) {
        entries.push(format!("agent_persona={}", baggage_escape(agent_persona)));
    }
    entries.join(",")
}

pub fn apply_outgoing_trace(
    req: &ChatCompleteRequest,
    provider_span: &OtelSpan,
    baggage: Option<&str>,
) -> ChatCompleteRequest {
    let mut traced = req.clone();
    traced.traceparent = Some(provider_span.traceparent());
    if let Some(baggage) = baggage {
        traced.baggage = Some(baggage.to_string());
    }
    traced
}

fn start_root_from_headers(
    name: &'static str,
    traceparent: Option<&str>,
    tracestate: Option<&str>,
) -> OtelSpan {
    let parent_ids =
        traceparent.and_then(|value| cyberos_obs_sdk::tracecontext::parse_traceparent(value).ok());
    if traceparent.is_some() && parent_ids.is_none() {
        warn!(
            traceparent_hash16 = %cyberos_obs_sdk::tracecontext::hash16(traceparent.unwrap_or_default().as_bytes()),
            "malformed_traceparent_ignored"
        );
    }

    let mut headers = http::HeaderMap::new();
    if let Some(value) = traceparent.and_then(header_value) {
        headers.insert("traceparent", value);
    }
    if let Some(value) = tracestate.and_then(header_value) {
        headers.insert("tracestate", value);
    }
    let parent_ctx = propagation::extract_context_from_headers(&headers);
    let mut span = start_span(
        name,
        &parent_ctx,
        SpanKind::Server,
        parent_ids.as_ref().map(|ids| ids.parent_span_id.clone()),
    );
    if let Some(ids) = parent_ids {
        span.trace_id = ids.trace_id;
    }
    span
}

fn start_span(
    name: &'static str,
    parent_ctx: &Context,
    kind: SpanKind,
    parent_span_id: Option<String>,
) -> OtelSpan {
    let tracer = global::tracer("cyberos-ai-gateway");
    let span = tracer
        .span_builder(name)
        .with_kind(kind)
        .start_with_context(&tracer, parent_ctx);
    let ctx = parent_ctx.with_span(span);
    let actual = ctx.span().span_context().clone();

    let actual_span_id = actual.span_id().to_string();
    let use_actual =
        actual.is_valid() && parent_span_id.as_deref() != Some(actual_span_id.as_str());
    let trace_id = if use_actual {
        actual.trace_id().to_string()
    } else {
        parent_trace_id(parent_ctx).unwrap_or_else(new_trace_id)
    };
    let span_id = if use_actual {
        actual_span_id
    } else {
        new_span_id()
    };

    OtelSpan {
        name,
        ctx,
        trace_id,
        span_id,
        parent_span_id,
        started: Instant::now(),
        attributes: Vec::new(),
        events: Vec::new(),
        status: None,
        ended: false,
    }
}

fn parent_trace_id(parent_ctx: &Context) -> Option<String> {
    let parent = parent_ctx.span().span_context().clone();
    parent.is_valid().then(|| parent.trace_id().to_string())
}

fn header_value(value: &str) -> Option<http::HeaderValue> {
    http::HeaderValue::from_str(value).ok()
}

fn new_trace_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

fn new_span_id() -> String {
    let next = NEXT_SPAN_ID.fetch_add(1, Ordering::Relaxed).max(1);
    format!("{next:016x}")
}

fn baggage_escape(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}
