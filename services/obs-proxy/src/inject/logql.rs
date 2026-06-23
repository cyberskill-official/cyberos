//! LogQL label injection (FR-OBS-002 §1 #5, §3). Hand-rolled because no mature Rust LogQL crate
//! exists; this covers the subset Grafana actually emits - a `{...}` stream selector optionally
//! wrapped in a metric function and followed by pipe stages.
//!
//! The parser is quote-aware: it finds the selector's closing `}` by tracking quoted strings, so a
//! `}` inside a label value (e.g. `{service="a}b"}`) never ends the selector early. That is exactly
//! the bypass class string concatenation is vulnerable to (DEC-146 / §2).

use crate::error::{Backend, ProxyError};

/// Inject `key="value"` into the first stream selector of `query`, preserving everything else
/// (metric wrapper, pipe stages) verbatim.
///
/// Errors with `ParseFailed` if the selector already contains `key` (a bypass attempt - the proxy
/// also catches this up front via [`has_label`]) or if no `{...}` selector is present.
pub fn add_label(query: &str, key: &str, value: &str) -> Result<String, ProxyError> {
    let (open, close) = find_selector(query)?;
    let inner = &query[open + 1..close];
    if matcher_names(inner).iter().any(|n| n == key) {
        return Err(ProxyError::ParseFailed {
            backend: Backend::Loki,
            reason: format!("query already contains {key} label"),
        });
    }
    let injected = if inner.trim().is_empty() {
        format!("{key}=\"{value}\"")
    } else {
        format!("{inner},{key}=\"{value}\"")
    };
    Ok(format!(
        "{}{{{}}}{}",
        &query[..open],
        injected,
        &query[close + 1..]
    ))
}

/// True if the first stream selector already contains a matcher named `key`.
pub fn has_label(query: &str, key: &str) -> Result<bool, ProxyError> {
    let (open, close) = find_selector(query)?;
    Ok(matcher_names(&query[open + 1..close])
        .iter()
        .any(|n| n == key))
}

/// Byte offsets of the first `{` and its matching `}`, honoring quoted strings.
fn find_selector(query: &str) -> Result<(usize, usize), ProxyError> {
    let bytes = query.as_bytes();
    let open = query.find('{').ok_or_else(|| ProxyError::ParseFailed {
        backend: Backend::Loki,
        reason: "no stream selector '{...}' found".into(),
    })?;
    let mut i = open + 1;
    let mut quote: Option<u8> = None;
    let mut escaped = false;
    while i < bytes.len() {
        let c = bytes[i];
        match quote {
            Some(q) => {
                if escaped {
                    escaped = false;
                } else if c == b'\\' && q == b'"' {
                    escaped = true;
                } else if c == q {
                    quote = None;
                }
            }
            None => match c {
                b'"' | b'`' => quote = Some(c),
                b'}' => return Ok((open, i)),
                _ => {}
            },
        }
        i += 1;
    }
    Err(ProxyError::ParseFailed {
        backend: Backend::Loki,
        reason: "unterminated stream selector".into(),
    })
}

/// Matcher names inside a selector body (the text between the braces), honoring quotes when
/// splitting on commas and stopping each name at its operator (`=`, `!=`, `=~`, `!~`).
fn matcher_names(inner: &str) -> Vec<String> {
    split_top_level_commas(inner)
        .into_iter()
        .filter_map(|part| {
            let name: String = part
                .trim()
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        })
        .collect()
}

/// Split on commas that are not inside a quoted string.
fn split_top_level_commas(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for c in s.chars() {
        match quote {
            Some(q) => {
                cur.push(c);
                if escaped {
                    escaped = false;
                } else if c == '\\' && q == '"' {
                    escaped = true;
                } else if c == q {
                    quote = None;
                }
            }
            None => match c {
                '"' | '`' => {
                    quote = Some(c);
                    cur.push(c);
                }
                ',' => out.push(std::mem::take(&mut cur)),
                _ => cur.push(c),
            },
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injects_simple_selector() {
        assert_eq!(
            add_label("{service=\"x\"}", "tenant_id", "T").unwrap(),
            "{service=\"x\",tenant_id=\"T\"}"
        );
    }

    #[test]
    fn preserves_pipe_stages() {
        let r = add_label(
            "{service=\"x\"} | json | line_format \"...\"",
            "tenant_id",
            "T",
        )
        .unwrap();
        assert!(r.starts_with("{service=\"x\",tenant_id=\"T\"}"));
        assert!(r.contains("| json"));
        assert!(r.contains("| line_format"));
    }

    #[test]
    fn injects_into_empty_selector() {
        assert_eq!(
            add_label("{}", "tenant_id", "T").unwrap(),
            "{tenant_id=\"T\"}"
        );
    }

    #[test]
    fn injects_into_metric_wrapped_selector() {
        assert_eq!(
            add_label("rate({service=\"x\"}[5m])", "tenant_id", "T").unwrap(),
            "rate({service=\"x\",tenant_id=\"T\"}[5m])"
        );
    }

    #[test]
    fn brace_inside_value_does_not_break_out() {
        // The string-concat bypass: a '}' inside a quoted value must not end the selector.
        assert_eq!(
            add_label("{service=\"a}b\"}", "tenant_id", "T").unwrap(),
            "{service=\"a}b\",tenant_id=\"T\"}"
        );
    }

    #[test]
    fn rejects_user_supplied_key() {
        let e = add_label("{tenant_id=\"other\"}", "tenant_id", "T").expect_err("must reject");
        assert!(matches!(e, ProxyError::ParseFailed { .. }));
    }

    #[test]
    fn has_label_detects_presence() {
        assert!(has_label("{service=\"x\",tenant_id=\"other\"}", "tenant_id").unwrap());
        assert!(!has_label("{service=\"x\"}", "tenant_id").unwrap());
    }

    #[test]
    fn missing_selector_errors() {
        assert!(add_label("foobar", "tenant_id", "T").is_err());
    }
}
