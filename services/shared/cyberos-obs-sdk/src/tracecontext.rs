//! W3C TraceContext helpers for CyberOS service boundaries.

use http::{HeaderMap, HeaderValue};
use rand::RngCore;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::red;

/// Parsed and locally-bound W3C TraceContext.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceContext {
    /// 16-byte trace id as lowercase hex.
    pub trace_id: String,
    /// Local 8-byte span id as lowercase hex.
    pub span_id: String,
    /// Incoming parent span id, if this context came from a header.
    pub parent_span_id: Option<String>,
    /// Trace flags byte as lowercase hex.
    pub trace_flags: String,
}

/// Header extraction outcome label for `obs_tracecontext_extracted_total`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtractOutcome {
    /// A valid incoming `traceparent` was parsed.
    Extracted,
    /// The header was absent and a new trace was generated.
    MissingGeneratedNew,
    /// The header was malformed and a new trace was generated.
    Malformed,
}

impl ExtractOutcome {
    /// Stable metric label value.
    pub fn as_label(self) -> &'static str {
        match self {
            Self::Extracted => "extracted",
            Self::MissingGeneratedNew => "missing_generated_new",
            Self::Malformed => "malformed",
        }
    }
}

/// Result of extracting or generating a request context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtractedTraceContext {
    /// Context to use for this request.
    pub context: TraceContext,
    /// How the context was obtained.
    pub outcome: ExtractOutcome,
    /// Hash of the malformed header, when applicable.
    pub malformed_hash16: Option<String>,
}

/// Parsed incoming `traceparent` header before local span binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedTraceparent {
    /// 16-byte trace id as lowercase hex.
    pub trace_id: String,
    /// Incoming parent span id as lowercase hex.
    pub parent_span_id: String,
    /// Trace flags byte as lowercase hex.
    pub trace_flags: String,
}

/// TraceContext parse errors.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum TraceContextError {
    /// Header did not match W3C TraceContext shape or constraints.
    #[error("malformed traceparent (hash16: {hash16})")]
    Malformed {
        /// SHA-256 hash prefix of the offending header.
        hash16: String,
    },
}

impl TraceContext {
    /// Generate a fresh W3C-compatible context for a service boundary.
    pub fn generate() -> Self {
        Self {
            trace_id: random_hex_nonzero(16),
            span_id: random_hex_nonzero(8),
            parent_span_id: None,
            trace_flags: "01".to_string(),
        }
    }

    /// Build a local context from a parsed incoming parent.
    pub fn from_parent(parent: ParsedTraceparent) -> Self {
        Self {
            trace_id: parent.trace_id,
            span_id: random_hex_nonzero(8),
            parent_span_id: Some(parent.parent_span_id),
            trace_flags: parent.trace_flags,
        }
    }

    /// Format this context as an outgoing `traceparent` header value.
    pub fn traceparent(&self) -> String {
        format!("00-{}-{}-{}", self.trace_id, self.span_id, self.trace_flags)
    }
}

/// Parse a W3C `traceparent` header strictly.
pub fn parse_traceparent(value: &str) -> Result<ParsedTraceparent, TraceContextError> {
    let mut parts = value.split('-');
    let version = parts.next();
    let trace_id = parts.next();
    let parent_span_id = parts.next();
    let trace_flags = parts.next();
    let ok = parts.next().is_none()
        && version == Some("00")
        && trace_id.is_some_and(is_valid_trace_id)
        && parent_span_id.is_some_and(is_valid_span_id)
        && trace_flags.is_some_and(is_valid_flags);

    if !ok {
        return Err(TraceContextError::Malformed {
            hash16: hash16(value.as_bytes()),
        });
    }

    Ok(ParsedTraceparent {
        trace_id: trace_id.expect("checked above").to_ascii_lowercase(),
        parent_span_id: parent_span_id.expect("checked above").to_ascii_lowercase(),
        trace_flags: trace_flags.expect("checked above").to_ascii_lowercase(),
    })
}

/// Extract a request context from headers or generate a new one.
pub fn extract_or_generate(headers: &HeaderMap) -> ExtractedTraceContext {
    match headers
        .get("traceparent")
        .and_then(|value| value.to_str().ok())
    {
        Some(value) => match parse_traceparent(value) {
            Ok(parent) => {
                red::record_tracecontext_extraction(ExtractOutcome::Extracted.as_label());
                ExtractedTraceContext {
                    context: TraceContext::from_parent(parent),
                    outcome: ExtractOutcome::Extracted,
                    malformed_hash16: None,
                }
            }
            Err(TraceContextError::Malformed { hash16 }) => {
                red::record_tracecontext_extraction(ExtractOutcome::Malformed.as_label());
                ExtractedTraceContext {
                    context: TraceContext::generate(),
                    outcome: ExtractOutcome::Malformed,
                    malformed_hash16: Some(hash16),
                }
            }
        },
        None => {
            red::record_tracecontext_extraction(ExtractOutcome::MissingGeneratedNew.as_label());
            ExtractedTraceContext {
                context: TraceContext::generate(),
                outcome: ExtractOutcome::MissingGeneratedNew,
                malformed_hash16: None,
            }
        }
    }
}

/// Inject this context into outbound headers using the local span as parent.
pub fn inject_traceparent(headers: &mut HeaderMap, context: &TraceContext) {
    if let Ok(value) = HeaderValue::from_str(&context.traceparent()) {
        headers.insert("traceparent", value);
    }
}

/// Validate a lowercase or uppercase W3C trace id.
pub fn is_valid_trace_id(value: &str) -> bool {
    value.len() == 32 && !all_zero(value) && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

/// Validate a lowercase or uppercase W3C span id.
pub fn is_valid_span_id(value: &str) -> bool {
    value.len() == 16 && !all_zero(value) && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

/// Hash an attacker-controlled value without logging the raw bytes.
pub fn hash16(value: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value);
    lower_hex(&hasher.finalize()[..8])
}

fn is_valid_flags(value: &str) -> bool {
    value.len() == 2 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn all_zero(value: &str) -> bool {
    value.bytes().all(|byte| byte == b'0')
}

fn random_hex_nonzero(bytes: usize) -> String {
    let mut buf = vec![0_u8; bytes];
    loop {
        rand::thread_rng().fill_bytes(&mut buf);
        if buf.iter().any(|byte| *byte != 0) {
            return lower_hex(&buf);
        }
    }
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
