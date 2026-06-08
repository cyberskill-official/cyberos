//! FR-AI-008 §3 — Router types (ProviderResponse, AttemptRecord, etc.).

use std::pin::Pin;

use futures::Stream;

use crate::policy::ProviderKind;
use crate::streaming::ProviderStreamEvent;

/// Normalized response from any LLM provider.
#[derive(Debug, Clone, PartialEq)]
pub struct ProviderResponse {
    /// Provider-supplied request id.
    pub id: String,
    /// Token usage.
    pub usage: ProviderUsage,
    /// Response choices.
    pub choices: Vec<Choice>,
    /// Why the model stopped generating.
    pub finish_reason: FinishReason,
    /// Per-attempt latency in ms (total wall time for this call).
    pub latency_ms: u32,
    /// Cache state (prompt cache hit/miss).
    pub cache_state: CacheState,
    /// Full attempt history for audit trail.
    pub attempts: Vec<AttemptRecord>,
    /// EU AI Act Art. 50 attribution metadata for persona-authored outputs.
    pub made_by_genie: Option<MadeByGenie>,
}

/// User-facing attribution metadata rendered by clients as "Made by Genie".
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MadeByGenie {
    pub id: String,
    pub version: String,
}

/// Token usage from the provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProviderUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    /// 0 if no prompt-cache feature.
    pub cached_input_tokens: u32,
}

/// A single response choice.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Choice {
    pub index: u8,
    /// May be empty on tool-call-only responses.
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: FinishReason,
}

/// Tool call in a response.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// Why the model stopped generating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FinishReason {
    /// Natural end.
    Stop,
    /// Hit max_tokens.
    Length,
    /// Model invoked tools.
    ToolCalls,
    /// Provider safety filter triggered.
    ContentFilter,
    /// Catch-all.
    Other,
}

/// Cache state for prompt caching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheState {
    /// No caching used.
    None,
    /// Prompt-cache served some tokens.
    Hit { saved_tokens: u32 },
    /// Requested cache, didn't hit.
    Miss,
}

/// Metadata for a single attempt within a call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttemptRecord {
    pub provider: ProviderKind,
    pub model: String,
    /// 1..=MAX_RETRIES_PER_PROVIDER per provider.
    pub attempt_num: u8,
    /// Matches ResolvedModel.fallback_position.
    pub fallback_position: u8,
    pub status: AttemptStatus,
    pub elapsed_ms: u32,
    /// None for non-HTTP errors (e.g. timeout).
    pub http_status: Option<u16>,
}

/// Outcome of a single attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptStatus {
    Succeeded,
    RetriedAfter5xx,
    RetriedAfter429,
    RetriedAfterTimeout,
    RetriedAfterConnReset,
    /// Provider exhausted retries; switching to next.
    FailedOver,
    /// Bad request; no retry, no failover.
    Terminal400,
    /// Model not found.
    Terminal404,
    /// 401/403.
    TerminalAuth,
    /// tokio::time::timeout fired.
    TimeoutBeforeFirstToken,
    /// Caller deadline elapsed during attempt.
    DeadlineExceededMidCall,
}

/// Chat completion request (minimal for router dispatch).
#[derive(Debug, Clone)]
pub struct ChatCompleteRequest {
    /// The alias that was resolved (e.g. "chat.smart").
    pub alias: String,
    /// The prompt messages.
    pub messages: Vec<Message>,
    /// Max tokens to generate.
    pub max_tokens: Option<u32>,
    /// Temperature.
    pub temperature: Option<f32>,
    /// Optional CyberOS persona handle, e.g. `cuo-cpo@0.4.1`.
    pub agent_persona: Option<String>,
    /// W3C traceparent header value.
    pub traceparent: Option<String>,
    /// W3C tracestate header value.
    pub tracestate: Option<String>,
}

/// A single message in a chat completion request.
#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Embed request (stub for slice 2).
#[derive(Debug, Clone)]
pub struct EmbedRequest {
    pub input: Vec<String>,
    pub model: String,
}

/// Embed response (stub for slice 2).
#[derive(Debug, Clone)]
pub struct EmbedResponse {
    pub embeddings: Vec<Vec<f32>>,
    pub usage: ProviderUsage,
}

/// Normalized streaming response from any LLM provider.
///
/// The router owns retry/failover until a provider accepts the streaming HTTP
/// request. Once this stream is returned, later provider failures are yielded
/// as stream items and are never retried behind the client's partial response.
pub struct ProviderStreamResponse {
    events: Pin<Box<dyn Stream<Item = Result<ProviderStreamEvent, RouterError>> + Send + 'static>>,
    attempts: Vec<AttemptRecord>,
}

impl ProviderStreamResponse {
    /// Construct a streaming response from normalized provider events.
    pub fn new<S>(events: S) -> Self
    where
        S: Stream<Item = Result<ProviderStreamEvent, RouterError>> + Send + 'static,
    {
        Self {
            events: Box::pin(events),
            attempts: Vec::new(),
        }
    }

    /// Attach router attempt metadata collected before the stream was accepted.
    pub fn with_attempts(mut self, attempts: Vec<AttemptRecord>) -> Self {
        self.attempts = attempts;
        self
    }

    /// Consume the response and return its event stream.
    pub fn into_events(
        self,
    ) -> Pin<Box<dyn Stream<Item = Result<ProviderStreamEvent, RouterError>> + Send + 'static>>
    {
        self.events
    }

    /// Router attempts made before the stream was accepted.
    pub fn attempts(&self) -> &[AttemptRecord] {
        &self.attempts
    }
}

impl std::fmt::Debug for ProviderStreamResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderStreamResponse")
            .field("attempts", &self.attempts)
            .finish_non_exhaustive()
    }
}

/// Errors from the router.
#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("deadline exceeded")]
    DeadlineExceeded,

    #[error("all providers failed (last: {last_error})")]
    AllProvidersFailed {
        last_error: Box<RouterError>,
        attempts: Vec<AttemptRecord>,
    },

    #[error("auth error: provider={provider:?} status={status}")]
    AuthError { provider: ProviderKind, status: u16 },

    #[error("terminal provider error: provider={provider:?} status={status} message={message}")]
    TerminalProviderError {
        provider: ProviderKind,
        status: u16,
        message: String,
        /// Populated from `Retry-After` header on 429 responses.
        retry_after_secs: Option<u64>,
    },

    #[error("serialization error: {reason}")]
    SerializationError { reason: String },

    #[error("invalid response: {reason}")]
    InvalidResponse { reason: String },

    #[error("redaction failed: {reason}")]
    RedactionFailed { reason: String },

    #[error("unknown persona {agent_persona}; available: {available:?}")]
    UnknownPersona {
        agent_persona: String,
        available: Vec<String>,
    },

    #[error("persona tampered: {handle}")]
    PersonaTampered { handle: String },

    #[error("persona audit failed: {reason}")]
    PersonaAuditFailed { reason: String },

    #[error("streaming not implemented in slice 2")]
    StreamingNotImplemented,
}
