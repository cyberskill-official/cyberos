//! W3C TraceContext parsing, validation, and propagation (FR-OBS-005 §1 #1, #4, #11).
//!
//! This is the pure correlation primitive every CyberOS service shares: parse an incoming `traceparent`
//! header strictly per the W3C spec, format one for an outgoing request, and - when a header is malformed
//! - reduce the bad value to a forensic hash so it is never logged raw (it may be attacker-controlled).
//!
//! Strict validation is deliberate (§1 #11). Honouring an attacker-supplied trace_id would let two
//! unrelated requests appear linked in Tempo, so a malformed header is rejected and the caller generates
//! a fresh id rather than trusting the input. The id generation, the `tracing-subscriber` enrichment
//! layer (§1 #2), the histogram exemplar (§1 #3), and the axum `with_trace_context` wrapper are the
//! integration layer built on top of these primitives, against the live OTel context.

use axum::http::HeaderMap;
use sha2::{Digest, Sha256};

/// A parsed W3C `traceparent`: `version "-" trace-id "-" parent-id "-" trace-flags`. Held as the
/// validated lowercase-hex strings the header carries, which is also exactly what logs and exemplars
/// embed - the conversion to `opentelemetry::trace::TraceId` happens at the integration boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceContext {
    /// 32 lowercase hex chars, never all-zero.
    pub trace_id: String,
    /// 16 lowercase hex chars, never all-zero.
    pub span_id: String,
    /// The trace-flags byte; bit 0 is "sampled".
    pub flags: u8,
}

impl TraceContext {
    /// Whether the sampled flag (bit 0) is set.
    pub fn sampled(&self) -> bool {
        self.flags & 0x01 != 0
    }
}

/// Why a `traceparent` could not be extracted.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ExtractError {
    #[error("traceparent header missing")]
    Missing,
    /// The header was present but invalid. Carries the forensic hash of the bad value, never the raw
    /// bytes (§1 #11).
    #[error("traceparent header malformed (hash16: {0})")]
    Malformed(String),
}

/// Parse a W3C version-00 `traceparent` strictly. Returns `None` on any deviation: a non-`00` version, a
/// wrong-length or non-lowercase-hex field, or an all-zero trace-id or parent-id (both invalid per spec).
pub fn parse_w3c_traceparent(s: &str) -> Option<TraceContext> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 4 {
        return None;
    }
    let (version, trace_id, span_id, flags) = (parts[0], parts[1], parts[2], parts[3]);

    // §1 #11 - only version 00 is accepted; future versions are a deviation this slice does not parse.
    if version != "00" {
        return None;
    }
    if trace_id.len() != 32 || !is_lower_hex(trace_id) || is_all_zero(trace_id) {
        return None;
    }
    if span_id.len() != 16 || !is_lower_hex(span_id) || is_all_zero(span_id) {
        return None;
    }
    if flags.len() != 2 || !is_lower_hex(flags) {
        return None;
    }
    let flags = u8::from_str_radix(flags, 16).ok()?;

    Some(TraceContext {
        trace_id: trace_id.to_string(),
        span_id: span_id.to_string(),
        flags,
    })
}

/// Format a `TraceContext` as a version-00 `traceparent` header value.
pub fn format_traceparent(tc: &TraceContext) -> String {
    format!("00-{}-{}-{:02x}", tc.trace_id, tc.span_id, tc.flags)
}

/// Extract and validate the `traceparent` from request headers. `Missing` when absent; `Malformed(hash)`
/// when present but invalid or non-ASCII.
pub fn extract_traceparent(headers: &HeaderMap) -> Result<TraceContext, ExtractError> {
    let raw = headers.get("traceparent").ok_or(ExtractError::Missing)?;
    let s = raw
        .to_str()
        .map_err(|_| ExtractError::Malformed(hash16(raw.as_bytes())))?;
    parse_w3c_traceparent(s).ok_or_else(|| ExtractError::Malformed(hash16(s.as_bytes())))
}

/// Inject a `traceparent` into outgoing request headers (§1 #4). A value that cannot be parsed as a
/// header (it always can, given the validated fields) is silently skipped rather than panicking.
pub fn inject_traceparent(headers: &mut HeaderMap, tc: &TraceContext) {
    if let Ok(value) = format_traceparent(tc).parse() {
        headers.insert("traceparent", value);
    }
}

/// The forensic hash of a value: the first 16 hex chars of SHA-256. Used to log a malformed traceparent
/// without echoing attacker-controlled bytes (§1 #11).
pub fn hash16(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().take(8).map(|b| format!("{b:02x}")).collect()
}

fn is_lower_hex(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn is_all_zero(s: &str) -> bool {
    s.bytes().all(|b| b == b'0')
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

    #[test]
    fn parses_a_valid_traceparent() {
        let tc = parse_w3c_traceparent(VALID).expect("valid");
        assert_eq!(tc.trace_id, "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(tc.span_id, "00f067aa0ba902b7");
        assert_eq!(tc.flags, 1);
        assert!(tc.sampled());
    }

    #[test]
    fn format_then_parse_roundtrips() {
        let tc = parse_w3c_traceparent(VALID).unwrap();
        assert_eq!(format_traceparent(&tc), VALID);
        assert_eq!(parse_w3c_traceparent(&format_traceparent(&tc)), Some(tc));
    }

    #[test]
    fn rejects_non_zero_version() {
        assert!(
            parse_w3c_traceparent("01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01")
                .is_none()
        );
        assert!(
            parse_w3c_traceparent("ff-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01")
                .is_none()
        );
    }

    #[test]
    fn rejects_all_zero_trace_or_span_id() {
        assert!(
            parse_w3c_traceparent("00-00000000000000000000000000000000-00f067aa0ba902b7-01")
                .is_none()
        );
        assert!(
            parse_w3c_traceparent("00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000000-01")
                .is_none()
        );
    }

    #[test]
    fn rejects_wrong_lengths_and_uppercase() {
        assert!(parse_w3c_traceparent("00-4bf9-00f067aa0ba902b7-01").is_none()); // short trace id
        assert!(
            parse_w3c_traceparent("00-4BF92F3577B34DA6A3CE929D0E0E4736-00f067aa0ba902b7-01")
                .is_none()
        ); // upper
        assert!(
            parse_w3c_traceparent("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-1")
                .is_none()
        ); // 1-char flags
        assert!(parse_w3c_traceparent("not-a-traceparent").is_none());
        assert!(parse_w3c_traceparent("").is_none());
    }

    #[test]
    fn extract_missing_then_malformed_then_valid() {
        let mut headers = HeaderMap::new();
        assert_eq!(extract_traceparent(&headers), Err(ExtractError::Missing));

        headers.insert("traceparent", "garbage".parse().unwrap());
        match extract_traceparent(&headers) {
            Err(ExtractError::Malformed(h)) => assert_eq!(h.len(), 16),
            other => panic!("expected Malformed, got {other:?}"),
        }

        headers.insert("traceparent", VALID.parse().unwrap());
        assert_eq!(
            extract_traceparent(&headers).unwrap().trace_id,
            "4bf92f3577b34da6a3ce929d0e0e4736"
        );
    }

    #[test]
    fn inject_sets_the_header() {
        let tc = parse_w3c_traceparent(VALID).unwrap();
        let mut headers = HeaderMap::new();
        inject_traceparent(&mut headers, &tc);
        assert_eq!(headers.get("traceparent").unwrap().to_str().unwrap(), VALID);
    }

    #[test]
    fn hash16_is_deterministic_16_hex() {
        let a = hash16(b"bad-value");
        assert_eq!(a.len(), 16);
        assert_eq!(a, hash16(b"bad-value"));
        assert_ne!(a, hash16(b"other"));
    }

    #[test]
    fn unsampled_flag_is_detected() {
        let tc = parse_w3c_traceparent("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-00")
            .unwrap();
        assert!(!tc.sampled());
    }
}
