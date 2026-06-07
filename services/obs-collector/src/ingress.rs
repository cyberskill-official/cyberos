//! FR-OBS-001 ingress gate.
//!
//! The upstream OpenTelemetry `bearertokenauth` extension authenticates a bearer
//! token, but it does not know which `service.name` that token is allowed to
//! represent. This module supplies the missing CyberOS policy: public OTLP HTTP
//! and gRPC ingress validates `Authorization: Bearer <token>` against the
//! service-name attributes in the request before forwarding to otelcol.

use std::collections::{BTreeSet, HashMap};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::Context;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response as AxumResponse};
use axum::routing::{get, post};
use axum::Router;
use opentelemetry_proto::tonic as otlp;
use prost::Message;
use reqwest::Client;
use serde_json::Value;
use thiserror::Error;
use tokio::net::TcpListener;
use tonic::metadata::MetadataValue;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::auth::TokenFile;

type TraceRequest = otlp::collector::trace::v1::ExportTraceServiceRequest;
type TraceResponse = otlp::collector::trace::v1::ExportTraceServiceResponse;
type LogsRequest = otlp::collector::logs::v1::ExportLogsServiceRequest;
type LogsResponse = otlp::collector::logs::v1::ExportLogsServiceResponse;
type MetricsRequest = otlp::collector::metrics::v1::ExportMetricsServiceRequest;
type MetricsResponse = otlp::collector::metrics::v1::ExportMetricsServiceResponse;

use otlp::collector::logs::v1::logs_service_client::LogsServiceClient;
use otlp::collector::logs::v1::logs_service_server::{LogsService, LogsServiceServer};
use otlp::collector::metrics::v1::metrics_service_client::MetricsServiceClient;
use otlp::collector::metrics::v1::metrics_service_server::{MetricsService, MetricsServiceServer};
use otlp::collector::trace::v1::trace_service_client::TraceServiceClient;
use otlp::collector::trace::v1::trace_service_server::{TraceService, TraceServiceServer};
use otlp::common::v1::any_value::Value as AnyValueKind;
use otlp::common::v1::KeyValue;
use otlp::resource::v1::Resource;

/// Runtime configuration for the CyberOS OTLP ingress gate.
#[derive(Debug, Clone)]
pub struct IngressConfig {
    /// Public HTTP listen address for OTLP/HTTP and `/ready`.
    pub http_listen: SocketAddr,
    /// Public gRPC listen address for OTLP/gRPC.
    pub grpc_listen: SocketAddr,
    /// Service token map, in CyberOS `<service> <token>` format.
    pub token_file: PathBuf,
    /// Internal token used from this gate to the upstream collector.
    pub collector_token_file: PathBuf,
    /// Upstream collector OTLP/HTTP base URL.
    pub upstream_http: String,
    /// Upstream collector OTLP/gRPC endpoint.
    pub upstream_grpc: String,
}

/// Errors returned while enforcing service-token binding.
#[derive(Debug, Error)]
pub enum IngressAuthError {
    /// The request has no bearer token.
    #[error("missing bearer token")]
    MissingBearer,
    /// The `Authorization` header was not valid UTF-8 or did not use Bearer.
    #[error("malformed bearer token")]
    MalformedBearer,
    /// The supplied token is not in the token map.
    #[error("invalid bearer token")]
    InvalidBearer,
    /// The request did not contain `service.name` on every resource record.
    #[error("missing service.name resource attribute")]
    MissingServiceName,
    /// The token owner and payload service names disagree.
    #[error("token for {authorized_service} cannot emit telemetry for {claimed_service}")]
    ServiceMismatch {
        /// Service that owns the token.
        authorized_service: String,
        /// Service claimed by the OTLP payload.
        claimed_service: String,
    },
    /// The request body was not valid OTLP JSON/protobuf for the target signal.
    #[error("malformed OTLP payload: {0}")]
    MalformedPayload(String),
    /// Token file could not be loaded.
    #[error("token file: {0}")]
    TokenFile(String),
}

/// Start the HTTP and gRPC ingress gate.
pub async fn serve(config: IngressConfig) -> anyhow::Result<()> {
    let http_state = HttpState {
        token_file: config.token_file.clone(),
        collector_token_file: config.collector_token_file.clone(),
        upstream_http: config.upstream_http.trim_end_matches('/').to_string(),
        client: Client::new(),
        metrics: Arc::new(IngressMetrics::default()),
    };
    let metrics = Arc::clone(&http_state.metrics);
    let app = Router::new()
        .route("/ready", get(ready))
        .route("/metrics", get(metrics_http))
        .route("/v1/traces", post(proxy_traces_http))
        .route("/v1/logs", post(proxy_logs_http))
        .route("/v1/metrics", post(proxy_metrics_http))
        .with_state(http_state);

    let grpc_proxy = GrpcProxy {
        token_file: config.token_file.clone(),
        collector_token_file: config.collector_token_file.clone(),
        upstream_grpc: config.upstream_grpc,
        metrics,
    };

    let http_listener = TcpListener::bind(config.http_listen)
        .await
        .with_context(|| format!("bind HTTP ingress {}", config.http_listen))?;
    let http = async move {
        axum::serve(http_listener, app)
            .await
            .context("HTTP ingress stopped")
    };

    let grpc_addr = config.grpc_listen;
    let grpc = async move {
        Server::builder()
            .add_service(TraceServiceServer::new(grpc_proxy.clone()))
            .add_service(LogsServiceServer::new(grpc_proxy.clone()))
            .add_service(MetricsServiceServer::new(grpc_proxy))
            .serve(grpc_addr)
            .await
            .context("gRPC ingress stopped")
    };

    tokio::try_join!(http, grpc)?;
    Ok(())
}

#[derive(Debug, Clone)]
struct HttpState {
    token_file: PathBuf,
    collector_token_file: PathBuf,
    upstream_http: String,
    client: Client,
    metrics: Arc<IngressMetrics>,
}

#[derive(Debug, Default)]
struct IngressMetrics {
    counters: Mutex<HashMap<(String, String), u64>>,
}

impl IngressMetrics {
    fn inc_signal(&self, signal: &str, service: &str) {
        let metric = match signal {
            "traces" => "obs_collector_received_spans_total",
            "logs" => "obs_collector_received_logs_total",
            "metrics" => "obs_collector_received_metrics_total",
            _ => return,
        };
        let mut counters = self.counters.lock().expect("metrics lock");
        *counters
            .entry((metric.to_string(), service.to_string()))
            .or_insert(0) += 1;
    }

    fn render(&self) -> String {
        let counters = self.counters.lock().expect("metrics lock");
        let mut lines = String::new();
        for ((metric, service), value) in counters.iter() {
            lines.push_str(metric);
            lines.push_str("{service=\"");
            lines.push_str(service);
            lines.push_str("\"} ");
            lines.push_str(&value.to_string());
            lines.push('\n');
        }
        lines
    }
}

async fn ready() -> &'static str {
    "ready\n"
}

async fn metrics_http(State(state): State<HttpState>) -> String {
    state.metrics.render()
}

async fn proxy_traces_http(
    State(state): State<HttpState>,
    headers: HeaderMap,
    body: Bytes,
) -> AxumResponse {
    proxy_http_signal(state, headers, body, "traces", "/v1/traces").await
}

async fn proxy_logs_http(
    State(state): State<HttpState>,
    headers: HeaderMap,
    body: Bytes,
) -> AxumResponse {
    proxy_http_signal(state, headers, body, "logs", "/v1/logs").await
}

async fn proxy_metrics_http(
    State(state): State<HttpState>,
    headers: HeaderMap,
    body: Bytes,
) -> AxumResponse {
    proxy_http_signal(state, headers, body, "metrics", "/v1/metrics").await
}

async fn proxy_http_signal(
    state: HttpState,
    headers: HeaderMap,
    body: Bytes,
    signal: &'static str,
    path: &'static str,
) -> AxumResponse {
    let content_type = headers
        .get(CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("application/json");

    let service_names = match service_names_from_http_body(signal, content_type, &body) {
        Ok(names) => names,
        Err(e) => return auth_error_response(e),
    };

    let bearer = match bearer_from_headers(&headers) {
        Ok(token) => token,
        Err(e) => return auth_error_response(e),
    };
    let tokens = match load_tokens(&state.token_file) {
        Ok(tokens) => tokens,
        Err(e) => return auth_error_response(e),
    };
    let owner = match authorize_service_names(&tokens, &bearer, &service_names) {
        Ok(owner) => owner,
        Err(e) => return auth_error_response(e),
    };
    state.metrics.inc_signal(signal, &owner);

    let collector_token = match read_collector_token(&state.collector_token_file) {
        Ok(token) => token,
        Err(e) => return auth_error_response(e),
    };

    let upstream = format!("{}{}", state.upstream_http, path);
    let response = state
        .client
        .post(upstream)
        .header(CONTENT_TYPE, content_type)
        .header(AUTHORIZATION, format!("Bearer {collector_token}"))
        .body(body)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status =
                StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            match resp.bytes().await {
                Ok(bytes) => (status, bytes).into_response(),
                Err(err) => (
                    StatusCode::BAD_GATEWAY,
                    format!("upstream collector body error: {err}"),
                )
                    .into_response(),
            }
        }
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            format!("upstream collector unavailable: {err}"),
        )
            .into_response(),
    }
}

fn auth_error_response(error: IngressAuthError) -> AxumResponse {
    let status = match error {
        IngressAuthError::MissingBearer
        | IngressAuthError::MalformedBearer
        | IngressAuthError::InvalidBearer => StatusCode::UNAUTHORIZED,
        IngressAuthError::ServiceMismatch { .. } => StatusCode::FORBIDDEN,
        IngressAuthError::MissingServiceName | IngressAuthError::MalformedPayload(_) => {
            StatusCode::BAD_REQUEST
        }
        IngressAuthError::TokenFile(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, error.to_string()).into_response()
}

#[derive(Debug, Clone)]
struct GrpcProxy {
    token_file: PathBuf,
    collector_token_file: PathBuf,
    upstream_grpc: String,
    metrics: Arc<IngressMetrics>,
}

#[tonic::async_trait]
impl TraceService for GrpcProxy {
    async fn export(
        &self,
        request: Request<TraceRequest>,
    ) -> Result<Response<TraceResponse>, Status> {
        let token = bearer_from_metadata(request.metadata())?;
        let message = request.into_inner();
        let names = service_names_from_trace_request(&message).map_err(auth_status)?;
        let owner = self.authorize_grpc(&token, &names)?;
        self.metrics.inc_signal("traces", &owner);
        let mut client = TraceServiceClient::connect(self.upstream_grpc.clone())
            .await
            .map_err(upstream_status)?;
        let request = self.upstream_request(message)?;
        client.export(request).await.map_err(upstream_status)
    }
}

#[tonic::async_trait]
impl LogsService for GrpcProxy {
    async fn export(
        &self,
        request: Request<LogsRequest>,
    ) -> Result<Response<LogsResponse>, Status> {
        let token = bearer_from_metadata(request.metadata())?;
        let message = request.into_inner();
        let names = service_names_from_logs_request(&message).map_err(auth_status)?;
        let owner = self.authorize_grpc(&token, &names)?;
        self.metrics.inc_signal("logs", &owner);
        let mut client = LogsServiceClient::connect(self.upstream_grpc.clone())
            .await
            .map_err(upstream_status)?;
        let request = self.upstream_request(message)?;
        client.export(request).await.map_err(upstream_status)
    }
}

#[tonic::async_trait]
impl MetricsService for GrpcProxy {
    async fn export(
        &self,
        request: Request<MetricsRequest>,
    ) -> Result<Response<MetricsResponse>, Status> {
        let token = bearer_from_metadata(request.metadata())?;
        let message = request.into_inner();
        let names = service_names_from_metrics_request(&message).map_err(auth_status)?;
        let owner = self.authorize_grpc(&token, &names)?;
        self.metrics.inc_signal("metrics", &owner);
        let mut client = MetricsServiceClient::connect(self.upstream_grpc.clone())
            .await
            .map_err(upstream_status)?;
        let request = self.upstream_request(message)?;
        client.export(request).await.map_err(upstream_status)
    }
}

impl GrpcProxy {
    fn authorize_grpc(&self, token: &str, service_names: &[String]) -> Result<String, Status> {
        let tokens = load_tokens(&self.token_file).map_err(auth_status)?;
        authorize_service_names(&tokens, token, service_names).map_err(auth_status)
    }

    fn upstream_request<T>(&self, message: T) -> Result<Request<T>, Status> {
        let collector_token =
            read_collector_token(&self.collector_token_file).map_err(auth_status)?;
        let auth_header = format!("Bearer {collector_token}");
        let mut request = Request::new(message);
        let metadata = MetadataValue::try_from(auth_header.as_str())
            .map_err(|_| Status::internal("invalid internal collector bearer token"))?;
        request.metadata_mut().insert("authorization", metadata);
        Ok(request)
    }
}

fn auth_status(error: IngressAuthError) -> Status {
    match error {
        IngressAuthError::MissingBearer
        | IngressAuthError::MalformedBearer
        | IngressAuthError::InvalidBearer => Status::unauthenticated(error.to_string()),
        IngressAuthError::ServiceMismatch { .. } => Status::permission_denied(error.to_string()),
        IngressAuthError::MissingServiceName | IngressAuthError::MalformedPayload(_) => {
            Status::invalid_argument(error.to_string())
        }
        IngressAuthError::TokenFile(_) => Status::internal(error.to_string()),
    }
}

fn upstream_status(error: impl std::fmt::Display) -> Status {
    Status::unavailable(format!("upstream collector unavailable: {error}"))
}

/// Enforce that a bearer token is known and only emits for its owning service.
pub fn authorize_service_names(
    tokens: &TokenFile,
    token: &str,
    service_names: &[String],
) -> Result<String, IngressAuthError> {
    if service_names.is_empty() {
        return Err(IngressAuthError::MissingServiceName);
    }
    let owner = tokens
        .service_for_token(token)
        .ok_or(IngressAuthError::InvalidBearer)?;
    for claimed in service_names {
        if claimed != owner {
            return Err(IngressAuthError::ServiceMismatch {
                authorized_service: owner.to_string(),
                claimed_service: claimed.clone(),
            });
        }
    }
    Ok(owner.to_string())
}

fn load_tokens(path: &Path) -> Result<TokenFile, IngressAuthError> {
    TokenFile::load(path).map_err(|e| IngressAuthError::TokenFile(e.to_string()))
}

fn read_collector_token(path: &Path) -> Result<String, IngressAuthError> {
    let token = std::fs::read_to_string(path)
        .map_err(|e| IngressAuthError::TokenFile(e.to_string()))?
        .trim()
        .to_string();
    if token.is_empty() || token.split_whitespace().count() != 1 {
        return Err(IngressAuthError::TokenFile(
            "internal collector token file must contain exactly one token".into(),
        ));
    }
    Ok(token)
}

fn bearer_from_headers(headers: &HeaderMap) -> Result<String, IngressAuthError> {
    let raw = headers
        .get(AUTHORIZATION)
        .ok_or(IngressAuthError::MissingBearer)?
        .to_str()
        .map_err(|_| IngressAuthError::MalformedBearer)?;
    raw.strip_prefix("Bearer ")
        .filter(|token| !token.trim().is_empty())
        .map(str::to_string)
        .ok_or(IngressAuthError::MalformedBearer)
}

fn bearer_from_metadata(metadata: &tonic::metadata::MetadataMap) -> Result<String, Status> {
    let raw = metadata
        .get("authorization")
        .ok_or_else(|| Status::unauthenticated("missing bearer token"))?
        .to_str()
        .map_err(|_| Status::unauthenticated("malformed bearer token"))?;
    raw.strip_prefix("Bearer ")
        .filter(|token| !token.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| Status::unauthenticated("malformed bearer token"))
}

fn service_names_from_http_body(
    signal: &str,
    content_type: &str,
    body: &Bytes,
) -> Result<Vec<String>, IngressAuthError> {
    if content_type.contains("protobuf") {
        return service_names_from_protobuf(signal, body);
    }

    let value: Value = serde_json::from_slice(body)
        .map_err(|e| IngressAuthError::MalformedPayload(e.to_string()))?;
    match signal {
        "traces" => service_names_from_json_resources(&value, "resourceSpans"),
        "logs" => service_names_from_json_resources(&value, "resourceLogs"),
        "metrics" => service_names_from_json_resources(&value, "resourceMetrics"),
        _ => Err(IngressAuthError::MalformedPayload(format!(
            "unknown signal {signal}"
        ))),
    }
}

fn service_names_from_protobuf(
    signal: &str,
    body: &Bytes,
) -> Result<Vec<String>, IngressAuthError> {
    match signal {
        "traces" => {
            let req = TraceRequest::decode(body.clone())
                .map_err(|e| IngressAuthError::MalformedPayload(e.to_string()))?;
            service_names_from_trace_request(&req)
        }
        "logs" => {
            let req = LogsRequest::decode(body.clone())
                .map_err(|e| IngressAuthError::MalformedPayload(e.to_string()))?;
            service_names_from_logs_request(&req)
        }
        "metrics" => {
            let req = MetricsRequest::decode(body.clone())
                .map_err(|e| IngressAuthError::MalformedPayload(e.to_string()))?;
            service_names_from_metrics_request(&req)
        }
        _ => Err(IngressAuthError::MalformedPayload(format!(
            "unknown signal {signal}"
        ))),
    }
}

fn service_names_from_json_resources(
    value: &Value,
    root_key: &str,
) -> Result<Vec<String>, IngressAuthError> {
    let resources = value
        .get(root_key)
        .and_then(Value::as_array)
        .ok_or_else(|| IngressAuthError::MalformedPayload(format!("missing {root_key}")))?;
    if resources.is_empty() {
        return Err(IngressAuthError::MissingServiceName);
    }

    let mut names = Vec::with_capacity(resources.len());
    for resource in resources {
        let attrs = resource
            .get("resource")
            .and_then(|r| r.get("attributes"))
            .and_then(Value::as_array)
            .ok_or(IngressAuthError::MissingServiceName)?;
        let name = attr_string_from_json(attrs, "service.name")
            .ok_or(IngressAuthError::MissingServiceName)?;
        names.push(name);
    }
    Ok(names)
}

fn attr_string_from_json(attrs: &[Value], key: &str) -> Option<String> {
    attrs.iter().find_map(|attr| {
        let attr_key = attr.get("key")?.as_str()?;
        if attr_key != key {
            return None;
        }
        attr.get("value")
            .and_then(|value| value.get("stringValue"))
            .and_then(Value::as_str)
            .map(str::to_string)
    })
}

fn service_names_from_trace_request(
    request: &TraceRequest,
) -> Result<Vec<String>, IngressAuthError> {
    service_names_from_resources(
        request
            .resource_spans
            .iter()
            .map(|resource| resource.resource.as_ref()),
    )
}

fn service_names_from_logs_request(request: &LogsRequest) -> Result<Vec<String>, IngressAuthError> {
    service_names_from_resources(
        request
            .resource_logs
            .iter()
            .map(|resource| resource.resource.as_ref()),
    )
}

fn service_names_from_metrics_request(
    request: &MetricsRequest,
) -> Result<Vec<String>, IngressAuthError> {
    service_names_from_resources(
        request
            .resource_metrics
            .iter()
            .map(|resource| resource.resource.as_ref()),
    )
}

fn service_names_from_resources<'a>(
    resources: impl Iterator<Item = Option<&'a Resource>>,
) -> Result<Vec<String>, IngressAuthError> {
    let mut count = 0usize;
    let mut names = BTreeSet::new();
    for resource in resources {
        count += 1;
        let resource = resource.ok_or(IngressAuthError::MissingServiceName)?;
        let service_name = attr_string_from_kv(&resource.attributes, "service.name")
            .ok_or(IngressAuthError::MissingServiceName)?;
        names.insert(service_name);
    }

    if count == 0 || names.is_empty() {
        return Err(IngressAuthError::MissingServiceName);
    }
    Ok(names.into_iter().collect())
}

fn attr_string_from_kv(attrs: &[KeyValue], key: &str) -> Option<String> {
    attrs.iter().find_map(|attr| {
        if attr.key != key {
            return None;
        }
        match attr.value.as_ref()?.value.as_ref()? {
            AnyValueKind::StringValue(value) => Some(value.clone()),
            _ => None,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use otlp::common::v1::AnyValue;
    use otlp::metrics::v1::ResourceMetrics;
    use otlp::resource::v1::Resource;
    use otlp::trace::v1::ResourceSpans;

    fn token_file() -> TokenFile {
        TokenFile::parse("ai-gateway token-ai\nauth-service token-auth\n").unwrap()
    }

    fn resource(service: &str) -> Resource {
        Resource {
            attributes: vec![KeyValue {
                key: "service.name".into(),
                value: Some(AnyValue {
                    value: Some(AnyValueKind::StringValue(service.into())),
                }),
            }],
            dropped_attributes_count: 0,
        }
    }

    #[test]
    fn authorizes_matching_service_token() {
        let tokens = token_file();
        let names = vec!["ai-gateway".to_string()];
        assert_eq!(
            authorize_service_names(&tokens, "token-ai", &names).unwrap(),
            "ai-gateway"
        );
    }

    #[test]
    fn rejects_cross_service_token() {
        let tokens = token_file();
        let names = vec!["auth-service".to_string()];
        let err = authorize_service_names(&tokens, "token-ai", &names).unwrap_err();
        assert!(matches!(err, IngressAuthError::ServiceMismatch { .. }));
    }

    #[test]
    fn extracts_trace_service_names_from_json() {
        let body = serde_json::json!({
            "resourceSpans": [{
                "resource": {
                    "attributes": [
                        {"key": "service.name", "value": {"stringValue": "ai-gateway"}},
                        {"key": "tenant_id", "value": {"stringValue": "00000000-0000-0000-0000-000000000001"}}
                    ]
                },
                "scopeSpans": []
            }]
        });
        let names = service_names_from_json_resources(&body, "resourceSpans").unwrap();
        assert_eq!(names, vec!["ai-gateway"]);
    }

    #[test]
    fn extracts_metric_service_names_from_protobuf() {
        let request = MetricsRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(resource("auth-service")),
                scope_metrics: vec![],
                schema_url: String::new(),
            }],
        };
        let names = service_names_from_metrics_request(&request).unwrap();
        assert_eq!(names, vec!["auth-service"]);
    }

    #[test]
    fn rejects_missing_service_name_in_protobuf() {
        let request = TraceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![],
                    dropped_attributes_count: 0,
                }),
                scope_spans: vec![],
                schema_url: String::new(),
            }],
        };
        assert!(matches!(
            service_names_from_trace_request(&request).unwrap_err(),
            IngressAuthError::MissingServiceName
        ));
    }
}
