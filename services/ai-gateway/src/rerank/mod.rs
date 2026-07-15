//! TASK-AI-020 — BGE-reranker-v2-m3 cross-encoder for KB reranking.
//!
//! Core types and constants for the rerank subsystem. The actual provider
//! integration (RerankProvider, batch buffer, circuit breaker) depends on
//! TASK-AI-008 (router) and TASK-AI-019 (embedding sidecar) and will be wired
//! when those tasks ship.
//!
//! See TASK-AI-020 for normative behaviour and acceptance criteria.

pub mod types;

pub use types::{
    cost_for_rerank, RerankError, RerankRequest, RerankResponse, MAX_CANDIDATES, MAX_TOTAL_TOKENS,
};
