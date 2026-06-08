//! FR-AI-019 — Adaptive BGE sidecar batch buffer.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::{oneshot, Mutex};

use super::{EmbedRequest, EmbedResponse, EmbedTask, ProviderUsage, RouterError};
use crate::policy::ProviderKind;

/// Maximum time a request waits for fan-in before dispatch.
pub const BATCH_FAN_IN_TIMEOUT_MS: u64 = 50;
/// Fairness grace when the queue reaches the max batch size quickly.
pub const BATCH_FULL_GRACE_MS: u64 = 15;
/// Maximum texts sent to the sidecar in one HTTP call.
pub const BATCH_MAX_SIZE: usize = 32;

/// Adaptive batch buffer shared by all BGE provider clones.
#[derive(Clone)]
pub struct BatchBuffer {
    inner: Arc<BatchBufferInner>,
}

impl std::fmt::Debug for BatchBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BatchBuffer").finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct BatchBufferInner {
    client: reqwest::Client,
    queues: dashmap::DashMap<String, Arc<QueueState>>,
}

struct QueueState {
    pending: Mutex<VecDeque<PendingRequest>>,
    running: AtomicBool,
}

impl std::fmt::Debug for QueueState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueueState")
            .field("running", &self.running.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

struct PendingRequest {
    req: EmbedRequest,
    model: String,
    deadline: Instant,
    response_tx: oneshot::Sender<Result<EmbedResponse, RouterError>>,
}

#[derive(Debug, Serialize)]
struct SidecarEmbedRequest {
    texts: Vec<String>,
    tenant_id: String,
    tenant_ids: Vec<String>,
    task: EmbedTask,
}

#[derive(Debug, Deserialize)]
struct SidecarEmbedResponse {
    embeddings: Vec<Vec<f32>>,
    model_name: String,
    model_sha256: String,
    sidecar_version: String,
    device: String,
    elapsed_ms: u32,
}

impl BatchBuffer {
    /// Construct a batch buffer using the supplied HTTP client.
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            inner: Arc::new(BatchBufferInner {
                client,
                queues: dashmap::DashMap::new(),
            }),
        }
    }

    /// Submit an embedding request to the sidecar for adaptive batching.
    pub async fn submit(
        &self,
        url: &str,
        req: EmbedRequest,
        model: &str,
        deadline: Instant,
    ) -> Result<EmbedResponse, RouterError> {
        if req.texts.is_empty() {
            return Ok(EmbedResponse {
                embeddings: Vec::new(),
                usage: ProviderUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    cached_input_tokens: 0,
                },
                model_name: "bge-m3".to_string(),
                model_sha256: String::new(),
                sidecar_version: String::new(),
                device: String::new(),
                elapsed_ms: 0,
            });
        }
        if req.texts.len() > BATCH_MAX_SIZE {
            return Err(RouterError::InvalidResponse {
                reason: format!(
                    "BGE request has {} texts; max batch size is {BATCH_MAX_SIZE}",
                    req.texts.len()
                ),
            });
        }

        let state = self
            .inner
            .queues
            .entry(url.to_string())
            .or_insert_with(|| {
                Arc::new(QueueState {
                    pending: Mutex::new(VecDeque::new()),
                    running: AtomicBool::new(false),
                })
            })
            .clone();
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = state.pending.lock().await;
            pending.push_back(PendingRequest {
                req,
                model: model.to_string(),
                deadline,
                response_tx: tx,
            });
        }

        if state
            .running
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            let inner = self.inner.clone();
            let state_clone = state.clone();
            let url = url.to_string();
            tokio::spawn(async move {
                worker_loop(inner, state_clone, url).await;
            });
        }

        rx.await.map_err(|_| RouterError::InvalidResponse {
            reason: "BGE batch worker dropped response channel".to_string(),
        })?
    }
}

async fn worker_loop(inner: Arc<BatchBufferInner>, state: Arc<QueueState>, url: String) {
    loop {
        let wait = {
            let pending = state.pending.lock().await;
            if pending.len() >= BATCH_MAX_SIZE {
                Duration::from_millis(BATCH_FULL_GRACE_MS)
            } else {
                Duration::from_millis(BATCH_FAN_IN_TIMEOUT_MS)
            }
        };
        tokio::time::sleep(wait).await;

        let batch = {
            let mut pending = state.pending.lock().await;
            drain_fair_batch(&mut pending)
        };

        if batch.is_empty() {
            state.running.store(false, Ordering::Release);
            let should_continue = {
                let pending = state.pending.lock().await;
                !pending.is_empty()
                    && state
                        .running
                        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
            };
            if should_continue {
                continue;
            }
            break;
        }

        dispatch_batch(&inner.client, &url, batch).await;
    }
}

fn drain_fair_batch(queue: &mut VecDeque<PendingRequest>) -> Vec<PendingRequest> {
    let Some(first) = queue.front() else {
        return Vec::new();
    };
    let model = first.model.clone();
    let task = first.req.task;

    let mut compatible = Vec::new();
    let mut deferred = VecDeque::new();
    while let Some(req) = queue.pop_front() {
        if req.model == model && req.req.task == task {
            compatible.push(req);
        } else {
            deferred.push_back(req);
        }
    }

    let mut tenant_order: Vec<String> = Vec::new();
    let mut by_tenant: Vec<(String, VecDeque<PendingRequest>)> = Vec::new();
    for req in compatible {
        let tenant = req.req.tenant_id.clone();
        if let Some((_, entries)) = by_tenant.iter_mut().find(|(id, _)| id == &tenant) {
            entries.push_back(req);
        } else {
            tenant_order.push(tenant.clone());
            let mut entries = VecDeque::new();
            entries.push_back(req);
            by_tenant.push((tenant, entries));
        }
    }

    let mut batch = Vec::with_capacity(BATCH_MAX_SIZE);
    while batch_text_len(&batch) < BATCH_MAX_SIZE {
        let mut progressed = false;
        for tenant in &tenant_order {
            if batch_text_len(&batch) >= BATCH_MAX_SIZE {
                break;
            }
            let Some((_, entries)) = by_tenant.iter_mut().find(|(id, _)| id == tenant) else {
                continue;
            };
            let Some(next) = entries.pop_front() else {
                continue;
            };
            if batch_text_len(&batch) + next.req.texts.len() <= BATCH_MAX_SIZE {
                batch.push(next);
                progressed = true;
            } else {
                entries.push_front(next);
                break;
            }
        }
        if !progressed {
            break;
        }
    }

    for (_, mut entries) in by_tenant {
        while let Some(req) = entries.pop_front() {
            deferred.push_back(req);
        }
    }
    *queue = deferred;
    batch
}

fn batch_text_len(batch: &[PendingRequest]) -> usize {
    batch.iter().map(|req| req.req.texts.len()).sum()
}

async fn dispatch_batch(client: &reqwest::Client, url: &str, batch: Vec<PendingRequest>) {
    let mut texts = Vec::new();
    let mut tenant_ids = Vec::new();
    let mut spans = Vec::new();
    let task = batch[0].req.task;
    let deadline = batch
        .iter()
        .map(|req| req.deadline)
        .min()
        .unwrap_or_else(Instant::now);

    for req in &batch {
        spans.push(req.req.texts.len());
        for text in &req.req.texts {
            texts.push(text.clone());
            tenant_ids.push(req.req.tenant_id.clone());
        }
    }
    let tenant_id = if tenant_ids.windows(2).all(|w| w[0] == w[1]) {
        tenant_ids.first().cloned().unwrap_or_default()
    } else {
        "mixed".to_string()
    };

    let body = SidecarEmbedRequest {
        texts,
        tenant_id,
        tenant_ids,
        task,
    };

    let response = call_sidecar(client, url, body, deadline).await;
    match response {
        Ok(resp) => fan_out_success(batch, spans, resp),
        Err(err) => {
            for req in batch {
                let _ = req.response_tx.send(Err(clone_router_error(&err)));
            }
        }
    }
}

async fn call_sidecar(
    client: &reqwest::Client,
    url: &str,
    body: SidecarEmbedRequest,
    deadline: Instant,
) -> Result<SidecarEmbedResponse, RouterError> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Err(RouterError::DeadlineExceeded);
    }

    let endpoint = format!("{}/embed", url.trim_end_matches('/'));
    let request = async {
        let response = client
            .post(&endpoint)
            .json(&body)
            .send()
            .await
            .map_err(|source| RouterError::TerminalProviderError {
                provider: ProviderKind::Bge,
                status: 503,
                message: source.to_string(),
                retry_after_secs: None,
            })?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(RouterError::TerminalProviderError {
                provider: ProviderKind::Bge,
                status: status.as_u16(),
                message: body,
                retry_after_secs: None,
            });
        }
        response
            .json::<SidecarEmbedResponse>()
            .await
            .map_err(|source| RouterError::InvalidResponse {
                reason: format!("invalid BGE sidecar response: {source}"),
            })
    };

    tokio::time::timeout(remaining, request)
        .await
        .map_err(|_| RouterError::DeadlineExceeded)?
}

fn fan_out_success(batch: Vec<PendingRequest>, spans: Vec<usize>, response: SidecarEmbedResponse) {
    if response.embeddings.len() != spans.iter().sum::<usize>() {
        let err = RouterError::InvalidResponse {
            reason: format!(
                "BGE sidecar returned {} embeddings for {} texts",
                response.embeddings.len(),
                spans.iter().sum::<usize>()
            ),
        };
        for req in batch {
            let _ = req.response_tx.send(Err(clone_router_error(&err)));
        }
        return;
    }

    let mut offset = 0usize;
    for (req, span) in batch.into_iter().zip(spans) {
        let embeddings = response.embeddings[offset..offset + span].to_vec();
        offset += span;
        let prompt_tokens = req
            .req
            .texts
            .iter()
            .map(|text| rough_token_count(text))
            .sum::<u32>();
        let _ = req.response_tx.send(Ok(EmbedResponse {
            embeddings,
            usage: ProviderUsage {
                prompt_tokens,
                completion_tokens: 0,
                cached_input_tokens: 0,
            },
            model_name: response.model_name.clone(),
            model_sha256: response.model_sha256.clone(),
            sidecar_version: response.sidecar_version.clone(),
            device: response.device.clone(),
            elapsed_ms: response.elapsed_ms,
        }));
    }
}

fn rough_token_count(text: &str) -> u32 {
    text.split_whitespace().count().max(1) as u32
}

fn clone_router_error(err: &RouterError) -> RouterError {
    match err {
        RouterError::DeadlineExceeded => RouterError::DeadlineExceeded,
        RouterError::TerminalProviderError {
            provider,
            status,
            message,
            retry_after_secs,
        } => RouterError::TerminalProviderError {
            provider: *provider,
            status: *status,
            message: message.clone(),
            retry_after_secs: *retry_after_secs,
        },
        RouterError::InvalidResponse { reason } => RouterError::InvalidResponse {
            reason: reason.clone(),
        },
        RouterError::NoSidecarForRegion { region } => RouterError::NoSidecarForRegion {
            region: region.clone(),
        },
        other => RouterError::InvalidResponse {
            reason: other.to_string(),
        },
    }
}
