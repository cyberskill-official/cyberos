//! FR-OBS-002 — Tenant-aware Grafana query proxy helpers.

use serde::{Deserialize, Serialize};

/// Query request accepted by the tenant-aware Grafana proxy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrafanaQuery {
    /// Tenant from the authenticated subject, never from caller-supplied query text.
    pub tenant_id: String,
    /// Datasource kind.
    pub datasource: Datasource,
    /// Raw LogQL, PromQL, or TraceQL expression.
    pub expression: String,
}

/// Supported Grafana datasource families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Datasource {
    /// Loki LogQL.
    Loki,
    /// Prometheus PromQL.
    Prometheus,
    /// Tempo TraceQL.
    Tempo,
}

/// Rewrite an expression so every backend query is scoped to the authenticated tenant.
pub fn inject_tenant_scope(query: &GrafanaQuery) -> Result<String, String> {
    if query.tenant_id.trim().is_empty() {
        return Err("tenant_id_required".into());
    }
    if query.expression.contains("tenant_id!=") {
        return Err("tenant_bypass_rejected".into());
    }
    let selector = format!(r#"tenant_id="{}""#, escape_label(&query.tenant_id));
    Ok(match query.datasource {
        Datasource::Loki | Datasource::Prometheus => {
            inject_label_selector(&query.expression, &selector)
        }
        Datasource::Tempo => format!("{} && {{{}}}", query.expression.trim(), selector),
    })
}

fn inject_label_selector(expression: &str, selector: &str) -> String {
    let trimmed = expression.trim();
    if let Some(rest) = trimmed.strip_prefix('{') {
        format!("{{{},{}", selector, rest)
    } else {
        format!("{{{}}} {}", selector, trimmed)
    }
}

fn escape_label(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loki_query_gets_tenant_selector() {
        let q = GrafanaQuery {
            tenant_id: "tenant-a".into(),
            datasource: Datasource::Loki,
            expression: r#"{service_name="api"} |= "error""#.into(),
        };
        assert!(inject_tenant_scope(&q)
            .unwrap()
            .contains(r#"tenant_id="tenant-a""#));
    }

    #[test]
    fn bypass_selector_is_rejected() {
        let q = GrafanaQuery {
            tenant_id: "tenant-a".into(),
            datasource: Datasource::Prometheus,
            expression: r#"http_requests_total{tenant_id!="tenant-a"}"#.into(),
        };
        assert_eq!(
            inject_tenant_scope(&q).unwrap_err(),
            "tenant_bypass_rejected"
        );
    }
}
