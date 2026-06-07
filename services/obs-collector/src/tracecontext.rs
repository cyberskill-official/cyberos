//! FR-OBS-005 — W3C TraceContext parse/inject helpers.

use serde::{Deserialize, Serialize};

/// Parsed W3C `traceparent`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceContext {
    /// 16-byte trace id as lowercase hex.
    pub trace_id: String,
    /// 8-byte parent span id as lowercase hex.
    pub parent_id: String,
    /// Trace flags byte as lowercase hex.
    pub flags: String,
}

/// Parse a W3C traceparent header.
pub fn parse_traceparent(header: &str) -> Result<TraceContext, String> {
    let parts: Vec<&str> = header.split('-').collect();
    if parts.len() != 4 || parts[0] != "00" {
        return Err("invalid_traceparent_shape".into());
    }
    validate_hex(parts[1], 32, "trace_id")?;
    validate_hex(parts[2], 16, "parent_id")?;
    validate_hex(parts[3], 2, "flags")?;
    Ok(TraceContext {
        trace_id: parts[1].to_ascii_lowercase(),
        parent_id: parts[2].to_ascii_lowercase(),
        flags: parts[3].to_ascii_lowercase(),
    })
}

/// Format a traceparent header.
pub fn format_traceparent(ctx: &TraceContext) -> String {
    format!("00-{}-{}-{}", ctx.trace_id, ctx.parent_id, ctx.flags)
}

fn validate_hex(value: &str, len: usize, field: &str) -> Result<(), String> {
    if value.len() != len || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("invalid_{field}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traceparent_round_trips() {
        let raw = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let parsed = parse_traceparent(raw).unwrap();
        assert_eq!(format_traceparent(&parsed), raw);
    }
}
