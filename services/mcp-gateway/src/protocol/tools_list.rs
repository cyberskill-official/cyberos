//! FR-MCP-001 §1 #6 — `tools/list` handler with cursor pagination.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::annotations::ToolAnnotations;
use crate::federation::registry::ToolRegistry;

const PAGE_SIZE: usize = 100;

/// Client-supplied params for `tools/list`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolsListParams {
    /// Opaque cursor from a previous response's `nextCursor`. Empty/None = start at zero.
    #[serde(default)]
    pub cursor: Option<String>,
}

/// One tool as exposed to the MCP client (FR-MCP-001 §1 #6).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescriptor {
    /// SEP-986 name (`cyberos.<module>.<verb>_<noun>`).
    pub name: String,
    /// Plain-English description.
    pub description: String,
    /// JSONSchema for `arguments`.
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
    /// Spec-defined annotations (`destructiveHint`/`readOnlyHint`/…).
    pub annotations: ToolAnnotations,
}

/// `tools/list` response payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListResult {
    /// One page of tools (≤ `PAGE_SIZE`).
    pub tools: Vec<ToolDescriptor>,
    /// Opaque cursor for the next page; `None` on the last page.
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Build one page of `tools/list` from the registry.
pub fn build_response(registry: &ToolRegistry, params: &ToolsListParams) -> ToolsListResult {
    let offset = params
        .cursor
        .as_deref()
        .and_then(decode_cursor)
        .unwrap_or(0);

    let all = registry.snapshot_sorted();
    let total = all.len();
    let end = (offset + PAGE_SIZE).min(total);
    let tools = all[offset..end].to_vec();
    let next_cursor = if end < total {
        Some(encode_cursor(end))
    } else {
        None
    };

    ToolsListResult { tools, next_cursor }
}

fn encode_cursor(offset: usize) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD_NO_PAD.encode(format!("offset={offset}"))
}

fn decode_cursor(c: &str) -> Option<usize> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD_NO_PAD
        .decode(c.as_bytes())
        .ok()?;
    let s = std::str::from_utf8(&bytes).ok()?;
    let n_str = s.strip_prefix("offset=")?;
    n_str.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotations::ToolAnnotations;

    fn add_tools(registry: &ToolRegistry, count: usize) {
        for i in 0..count {
            registry.register(
                format!("cyberos.test.tool_{i:03}"),
                "test tool".into(),
                serde_json::json!({"type":"object"}),
                ToolAnnotations::read_only_idempotent("test"),
                "test-module".into(),
                "http://localhost/test".into(),
                vec!["mcp:tools".into()],
            );
        }
    }

    #[test]
    fn empty_registry_returns_empty_page() {
        let r = ToolRegistry::new();
        let resp = build_response(&r, &Default::default());
        assert!(resp.tools.is_empty());
        assert!(resp.next_cursor.is_none());
    }

    #[test]
    fn small_registry_no_pagination() {
        let r = ToolRegistry::new();
        add_tools(&r, 5);
        let resp = build_response(&r, &Default::default());
        assert_eq!(resp.tools.len(), 5);
        assert!(resp.next_cursor.is_none());
    }

    #[test]
    fn large_registry_paginates() {
        let r = ToolRegistry::new();
        add_tools(&r, 250);
        let p1 = build_response(&r, &Default::default());
        assert_eq!(p1.tools.len(), 100);
        assert!(p1.next_cursor.is_some());

        let p2 = build_response(
            &r,
            &ToolsListParams {
                cursor: p1.next_cursor.clone(),
            },
        );
        assert_eq!(p2.tools.len(), 100);
        assert!(p2.next_cursor.is_some());

        let p3 = build_response(
            &r,
            &ToolsListParams {
                cursor: p2.next_cursor.clone(),
            },
        );
        assert_eq!(p3.tools.len(), 50);
        assert!(p3.next_cursor.is_none());
    }
}
