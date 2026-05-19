//! FR-MCP-001 §1 #25 + §1 #26 — Axum router mounting `POST /mcp` + `GET /mcp/healthz`.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use serde::Serialize;
use serde_json::{json, Value};
use tracing::warn;

use crate::federation::registry::ToolRegistry;
use crate::protocol::errors::{codes, err};
use crate::protocol::initialize::{build_response_value, InitializeParams};
use crate::protocol::jsonrpc::{Inbound, Request, Response};
use crate::protocol::tools_call::{dispatch as call_dispatch, ToolsCallParams};
use crate::protocol::tools_list::{build_response as build_tools_list, ToolsListParams};
use crate::MCP_PROTOCOL_VERSION;

/// Shared state passed through every handler.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Federated tool registry.
    pub registry: Arc<ToolRegistry>,
}

/// FR-MCP-001 §1 #25 healthz payload.
#[derive(Debug, Serialize)]
pub struct HealthZ {
    /// Always `"ok"` while we're up.
    pub status: &'static str,
    /// MCP protocol version we speak.
    pub protocol_version: &'static str,
    /// Distinct modules registered.
    pub registered_modules: usize,
    /// Total tools registered.
    pub registered_tools: usize,
}

/// Build the slice-1 Axum router.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/mcp", post(handle_mcp))
        .route("/mcp/healthz", get(handle_healthz))
        .with_state(state)
}

async fn handle_healthz(State(state): State<AppState>) -> (StatusCode, Json<HealthZ>) {
    (
        StatusCode::OK,
        Json(HealthZ {
            status: "ok",
            protocol_version: MCP_PROTOCOL_VERSION,
            registered_modules: state.registry.modules().len(),
            registered_tools: state.registry.len(),
        }),
    )
}

async fn handle_mcp(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> (StatusCode, Json<Value>) {
    let inbound = match Inbound::parse(&body) {
        Ok(i) => i,
        Err(e) => {
            warn!(error = %e, "parse failure");
            return (
                StatusCode::OK,
                Json(
                    serde_json::to_value(Response::error(Value::Null, err(codes::PARSE_ERROR, &e)))
                        .expect("serialise"),
                ),
            );
        }
    };

    match inbound {
        Inbound::Single(req) => {
            if req.is_notification() {
                // Per JSON-RPC 2.0, no response is emitted for notifications.
                return (StatusCode::OK, Json(json!(null)));
            }
            let resp = dispatch_one(&state, req).await;
            (
                StatusCode::OK,
                Json(serde_json::to_value(resp).expect("serialise")),
            )
        }
        Inbound::Batch(reqs) => {
            let mut out: Vec<Response> = Vec::with_capacity(reqs.len());
            for r in reqs {
                if r.is_notification() {
                    continue;
                }
                out.push(dispatch_one(&state, r).await);
            }
            (
                StatusCode::OK,
                Json(serde_json::to_value(out).expect("serialise")),
            )
        }
    }
}

async fn dispatch_one(state: &AppState, req: Request) -> Response {
    let id = req.id.clone().unwrap_or(Value::Null);
    match req.method.as_str() {
        "initialize" => {
            let params: InitializeParams = match req.params {
                Some(p) => match serde_json::from_value(p) {
                    Ok(v) => v,
                    Err(e) => {
                        return Response::error(
                            id,
                            err(codes::INVALID_PARAMS, &format!("initialize: {e}")),
                        );
                    }
                },
                None => InitializeParams {
                    protocol_version: String::new(),
                    client_info: None,
                    capabilities: None,
                },
            };
            match build_response_value(&params) {
                Ok(v) => Response::success(id, v),
                Err(e) => Response::error(id, e),
            }
        }
        "tools/list" => {
            let params: ToolsListParams = match req.params {
                Some(p) => match serde_json::from_value(p) {
                    Ok(v) => v,
                    Err(e) => {
                        return Response::error(
                            id,
                            err(codes::INVALID_PARAMS, &format!("tools/list: {e}")),
                        );
                    }
                },
                None => ToolsListParams::default(),
            };
            let r = build_tools_list(&state.registry, &params);
            Response::success(id, serde_json::to_value(r).expect("serialise"))
        }
        "tools/call" => {
            // Slice-1: caller scopes come from JWT verification (FR-MCP-004) once wired;
            // for now we accept a permissive default and rely on FR-MCP-002+004 to
            // tighten. The `_caller_scopes` is the integration point.
            let params: ToolsCallParams = match req.params {
                Some(p) => match serde_json::from_value(p) {
                    Ok(v) => v,
                    Err(e) => {
                        return Response::error(
                            id,
                            err(codes::INVALID_PARAMS, &format!("tools/call: {e}")),
                        );
                    }
                },
                None => {
                    return Response::error(
                        id,
                        err(codes::INVALID_PARAMS, "tools/call: missing params"),
                    );
                }
            };
            let caller_scopes = vec!["mcp:tools".to_string()];
            match call_dispatch(&state.registry, &params, &caller_scopes).await {
                Ok(r) => Response::success(id, serde_json::to_value(r).expect("serialise")),
                Err(e) => Response::error(id, e),
            }
        }
        "notifications/initialized" => {
            // Should have been short-circuited by `is_notification()`; if we got here it
            // means the client sent an id, which the spec allows but is unusual. Just
            // return success.
            Response::success(id, Value::Null)
        }
        other => Response::error(
            id,
            err(
                codes::METHOD_NOT_FOUND,
                &format!("method not found: {other}"),
            ),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with_tools(n: usize) -> AppState {
        let r = ToolRegistry::new();
        for i in 0..n {
            r.register(
                format!("cyberos.test.tool_{i}"),
                "test".into(),
                json!({"type":"object"}),
                crate::annotations::ToolAnnotations::read_only_idempotent("t"),
                "test".into(),
                "http://localhost/test".into(),
                vec!["mcp:tools".into()],
            );
        }
        AppState {
            registry: Arc::new(r),
        }
    }

    #[tokio::test]
    async fn dispatch_unknown_method_is_method_not_found() {
        let state = state_with_tools(0);
        let req = Request {
            jsonrpc: "2.0".into(),
            id: Some(json!(1)),
            method: "no/such/thing".into(),
            params: None,
        };
        let r = dispatch_one(&state, req).await;
        assert!(r.error.is_some());
        assert_eq!(r.error.unwrap().code, -32601);
    }

    #[tokio::test]
    async fn dispatch_initialize_with_correct_version_succeeds() {
        let state = state_with_tools(0);
        let req = Request {
            jsonrpc: "2.0".into(),
            id: Some(json!(1)),
            method: "initialize".into(),
            params: Some(json!({"protocolVersion": MCP_PROTOCOL_VERSION})),
        };
        let r = dispatch_one(&state, req).await;
        assert!(r.error.is_none(), "got {:?}", r.error);
        let result = r.result.unwrap();
        assert_eq!(result["protocolVersion"], MCP_PROTOCOL_VERSION);
    }

    #[tokio::test]
    async fn dispatch_tools_list_returns_descriptors() {
        let state = state_with_tools(3);
        let req = Request {
            jsonrpc: "2.0".into(),
            id: Some(json!(1)),
            method: "tools/list".into(),
            params: None,
        };
        let r = dispatch_one(&state, req).await;
        assert!(r.error.is_none());
        let tools = &r.result.unwrap()["tools"];
        assert_eq!(tools.as_array().unwrap().len(), 3);
    }
}
