//! TASK-AI-022 §1 #6 — Approved attribute keys. PII-safe by construction.
//!
//! New keys MUST be added here with a `// PII-safe because: ...` comment.
//! String-literal attribute keys at call sites are forbidden.

pub const TENANT_ID: &str = "ai_gateway.tenant_id"; // PII-safe: org-level id, not personal
pub const MODEL_ALIAS: &str = "ai_gateway.model_alias"; // PII-safe: e.g., "chat.smart"
pub const AGENT_PERSONA: &str = "ai_gateway.agent_persona"; // PII-safe: e.g., "cuo-cpo@0.4.1"
pub const IDEMPOTENCY_KEY: &str = "ai_gateway.idempotency_key"; // PII-safe: caller-generated UUID-shape
pub const STREAM: &str = "ai_gateway.stream"; // PII-safe: bool
pub const OUTCOME: &str = "ai_gateway.outcome"; // PII-safe: enum (allow|refuse|error)
pub const PROVIDER: &str = "ai_gateway.provider"; // PII-safe: enum (bedrock|anthropic|openai|...)
pub const MODEL: &str = "ai_gateway.model"; // PII-safe: model id like "claude-3-5-sonnet"
pub const ATTEMPT_NUM: &str = "ai_gateway.attempt_num"; // PII-safe: integer
pub const FALLBACK_POSITION: &str = "ai_gateway.fallback_position"; // PII-safe: integer
pub const STATUS_CODE: &str = "ai_gateway.status_code"; // PII-safe: HTTP integer
pub const RETRIED: &str = "ai_gateway.retried"; // PII-safe: bool
pub const PROMPT_TOKENS: &str = "ai_gateway.prompt_tokens"; // PII-safe: count, not content
pub const COMPLETION_TOKENS: &str = "ai_gateway.completion_tokens"; // PII-safe: count, not content
pub const ESTIMATED_USD: &str = "ai_gateway.estimated_usd"; // PII-safe: number
pub const ACTUAL_USD: &str = "ai_gateway.actual_usd"; // PII-safe: number
pub const CACHE_STATE: &str = "ai_gateway.cache_state"; // PII-safe: enum (hit|miss|skipped|error)
pub const CACHE_KEY_HASH16: &str = "ai_gateway.cache_key_hash16"; // PII-safe: hash, not content
pub const REQUEST_ID: &str = "ai_gateway.request_id"; // PII-safe: UUID-shape, not personal
pub const REGION: &str = "ai_gateway.region"; // PII-safe: AWS region string

// Span event attribute keys (used in events, not spans)
pub const RETRY_ATTEMPT: &str = "retry.attempt";
pub const RETRY_BACKOFF_MS: &str = "retry.backoff_ms";
pub const RETRY_PRIOR_STATUS: &str = "retry.prior_status_code";

// FORBIDDEN at compile time (PII; requires task amendment + DPO sign-off):
// pub const USER_EMAIL:    &str = ... — would leak personal email
// pub const PROMPT_TEXT:   &str = ... — would leak prompt content
// pub const RESPONSE_TEXT: &str = ... — would leak response content
// pub const PHONE:         &str = ... — would leak phone number
// pub const CCCD:          &str = ... — would leak Vietnamese government ID
