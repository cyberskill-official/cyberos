//! TraceQL filter injection (TASK-OBS-002 §1 #5, §4 #5, §8). Hand-rolled subset covering the
//! `{ <conditions> }` spanset filter Grafana emits. The tenant filter is AND-ed in with `&&`:
//!
//!   `{ service.name = "x" }`  ->  `{ service.name = "x" && resource.tenant_id = "T" }`
//!
//! The brace finder is quote-aware, so a `}` inside a quoted value cannot end the filter early
//! (the DEC-146 bypass class).

use crate::error::{Backend, ProxyError};

/// Inject `key = "value"` (AND-ed) into the first `{...}` spanset filter and reserialise.
pub fn add_label(query: &str, key: &str, value: &str) -> Result<String, ProxyError> {
    let (open, close) = find_selector(query)?;
    let inner = &query[open + 1..close];
    if condition_keys(inner).iter().any(|k| k == key) {
        // Already present - inject-only (the proxy rejects user-supplied tenant_id up front).
        return Ok(query.to_string());
    }
    let trimmed = inner.trim();
    let new_inner = if trimmed.is_empty() {
        format!(" {key} = \"{value}\" ")
    } else {
        format!(" {trimmed} && {key} = \"{value}\" ")
    };
    Ok(format!(
        "{}{{{}}}{}",
        &query[..open],
        new_inner,
        &query[close + 1..]
    ))
}

/// True if the first spanset filter already constrains `key`.
pub fn has_label(query: &str, key: &str) -> Result<bool, ProxyError> {
    let (open, close) = find_selector(query)?;
    Ok(condition_keys(&query[open + 1..close])
        .iter()
        .any(|k| k == key))
}

/// Byte offsets of the first `{` and its matching `}`, honoring quoted strings.
fn find_selector(query: &str) -> Result<(usize, usize), ProxyError> {
    let bytes = query.as_bytes();
    let open = query.find('{').ok_or_else(|| ProxyError::ParseFailed {
        backend: Backend::Tempo,
        reason: "no '{...}' spanset filter found".into(),
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
        backend: Backend::Tempo,
        reason: "unterminated '{...}' spanset filter".into(),
    })
}

/// The attribute name of each top-level condition (split on `&&` / `||`, honoring quotes).
fn condition_keys(inner: &str) -> Vec<String> {
    split_conditions(inner)
        .into_iter()
        .filter_map(|cond| {
            let name: String = cond
                .trim()
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
                .collect();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        })
        .collect()
}

/// Split on top-level `&&` / `||` operators, honoring quoted strings.
fn split_conditions(inner: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = inner.chars().collect();
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if let Some(q) = quote {
            cur.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' && q == '"' {
                escaped = true;
            } else if c == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match c {
            '"' | '`' => {
                quote = Some(c);
                cur.push(c);
                i += 1;
            }
            '&' | '|' if i + 1 < chars.len() && chars[i + 1] == c => {
                out.push(std::mem::take(&mut cur));
                i += 2;
            }
            _ => {
                cur.push(c);
                i += 1;
            }
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
    fn injects_spec_example_exactly() {
        // TASK-OBS-002 §8.
        assert_eq!(
            add_label(
                "{ service.name = \"ai-gateway\" }",
                "resource.tenant_id",
                "org:cyberskill"
            )
            .unwrap(),
            "{ service.name = \"ai-gateway\" && resource.tenant_id = \"org:cyberskill\" }"
        );
    }

    #[test]
    fn injects_into_compact_selector() {
        let r = add_label("{service.name=\"x\"}", "resource.tenant_id", "T").unwrap();
        assert!(r.contains("resource.tenant_id = \"T\""));
        assert!(r.contains("service.name"));
    }

    #[test]
    fn injects_into_empty_selector() {
        assert_eq!(
            add_label("{}", "resource.tenant_id", "T").unwrap(),
            "{ resource.tenant_id = \"T\" }"
        );
    }

    #[test]
    fn brace_inside_value_does_not_break_out() {
        assert_eq!(
            add_label("{ service.name = \"a}b\" }", "resource.tenant_id", "T").unwrap(),
            "{ service.name = \"a}b\" && resource.tenant_id = \"T\" }"
        );
    }

    #[test]
    fn has_label_detects_presence() {
        assert!(has_label(
            "{ service.name = \"x\" && resource.tenant_id = \"other\" }",
            "resource.tenant_id"
        )
        .unwrap());
        assert!(!has_label("{ service.name = \"x\" }", "resource.tenant_id").unwrap());
    }

    #[test]
    fn missing_selector_errors() {
        assert!(add_label("not a span query", "resource.tenant_id", "T").is_err());
    }
}
