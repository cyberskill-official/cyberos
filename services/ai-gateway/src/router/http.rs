//! Shared HTTP helpers for FR-AI-008 provider implementations.

use std::time::Instant;

use reqwest::header::RETRY_AFTER;

use super::{ChatCompleteRequest, RouterError};
use crate::policy::ProviderKind;

pub(crate) fn apply_trace_headers(
    mut builder: reqwest::RequestBuilder,
    req: &ChatCompleteRequest,
) -> reqwest::RequestBuilder {
    if let Some(traceparent) = &req.traceparent {
        builder = builder.header("traceparent", traceparent);
    }
    if let Some(tracestate) = &req.tracestate {
        builder = builder.header("tracestate", tracestate);
    }
    if let Some(baggage) = &req.baggage {
        builder = builder.header("baggage", baggage);
    }
    builder
}

pub(crate) fn remaining_deadline(deadline: Instant) -> Result<std::time::Duration, RouterError> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|duration| !duration.is_zero())
        .ok_or(RouterError::DeadlineExceeded)
}

pub(crate) async fn send_with_deadline(
    builder: reqwest::RequestBuilder,
    deadline: Instant,
    provider: ProviderKind,
) -> Result<reqwest::Response, RouterError> {
    let remaining = remaining_deadline(deadline)?;
    match tokio::time::timeout(remaining, builder.send()).await {
        Err(_) => Err(RouterError::DeadlineExceeded),
        Ok(Err(err)) => {
            if err.is_timeout() {
                Err(RouterError::DeadlineExceeded)
            } else {
                Err(RouterError::TerminalProviderError {
                    provider,
                    status: 503,
                    message: format!("provider network error: {err}"),
                    retry_after_secs: None,
                })
            }
        }
        Ok(Ok(response)) => Ok(response),
    }
}

pub(crate) async fn error_from_response(
    provider: ProviderKind,
    response: reqwest::Response,
) -> RouterError {
    let status = response.status().as_u16();
    let retry_after_secs = response
        .headers()
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());
    let message = response.text().await.unwrap_or_default();

    match status {
        401 | 403 => RouterError::AuthError { provider, status },
        _ => RouterError::TerminalProviderError {
            provider,
            status,
            message,
            retry_after_secs,
        },
    }
}
