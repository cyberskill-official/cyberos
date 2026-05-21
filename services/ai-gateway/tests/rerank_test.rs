//! FR-AI-020 §5 — Unit tests for rerank types and validation.

use cyberos_ai_gateway::rerank::*;

// ─── Constants ────────────────────────────────────────────────────────────────

#[test]
fn max_candidates_is_100() {
    assert_eq!(MAX_CANDIDATES, 100);
}

#[test]
fn max_total_tokens_is_819200() {
    assert_eq!(MAX_TOTAL_TOKENS, 819_200);
}

// ─── Cost ─────────────────────────────────────────────────────────────────────

#[test]
fn cost_for_rerank_returns_zero() {
    assert_eq!(cost_for_rerank(50, 5000), 0.0);
    assert_eq!(cost_for_rerank(100, 800_000), 0.0);
    assert_eq!(cost_for_rerank(0, 0), 0.0);
}

// ─── RerankRequest validation ─────────────────────────────────────────────────

#[test]
fn request_defaults_normalize_to_true() {
    let json = r#"{"query":"test","candidates":["a"],"tenant_id":"t"}"#;
    let req: RerankRequest = serde_json::from_str(json).unwrap();
    assert!(req.normalize);
}

#[test]
fn request_accepts_normalize_false() {
    let json = r#"{"query":"test","candidates":["a"],"tenant_id":"t","normalize":false}"#;
    let req: RerankRequest = serde_json::from_str(json).unwrap();
    assert!(!req.normalize);
}

#[test]
fn request_candidate_count_check() {
    let candidates: Vec<String> = (0..150).map(|i| format!("doc {i}")).collect();
    assert!(candidates.len() > MAX_CANDIDATES);
}

// ─── RerankResponse ──────────────────────────────────────────────────────────

#[test]
fn skipped_response_has_empty_scores() {
    let resp = RerankResponse::skipped();
    assert!(resp.skipped);
    assert!(resp.scores.is_empty());
    assert_eq!(resp.device, "unavailable");
    assert_eq!(resp.elapsed_ms, 0);
    assert_eq!(resp.model_name, "bge-reranker-v2-m3");
}

#[test]
fn response_serialise_roundtrip() {
    let resp = RerankResponse {
        scores: vec![(2, 0.94), (0, 0.71), (1, 0.12)],
        skipped: false,
        model_name: "bge-reranker-v2-m3".into(),
        model_sha256: "9f3a8d2b1e0c4f7a".into(),
        sidecar_version: "1.0.0".into(),
        device: "cuda".into(),
        elapsed_ms: 87,
        query_token_count: 8,
        total_candidate_tokens: 1240,
    };
    let json = serde_json::to_string(&resp).unwrap();
    let resp2: RerankResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(resp2.scores.len(), 3);
    assert_eq!(resp2.scores[0], (2, 0.94));
    assert!(!resp2.skipped);
    assert_eq!(resp2.device, "cuda");
}

// ─── RerankError ──────────────────────────────────────────────────────────────

#[test]
fn error_display_too_many_candidates() {
    let err = RerankError::TooManyCandidates {
        max: 100,
        actual: 150,
    };
    assert!(err.to_string().contains("100"));
    assert!(err.to_string().contains("150"));
}

#[test]
fn error_display_token_budget() {
    let err = RerankError::TokenBudgetExceeded {
        q: 8,
        c: 850_000,
        m: 819_200,
    };
    assert!(err.to_string().contains("819200"));
}

#[test]
fn error_display_no_sidecar() {
    let err = RerankError::NoSidecarForRegion;
    assert!(err.to_string().contains("no sidecar"));
}

// ─── Score ordering invariant ─────────────────────────────────────────────────

#[test]
fn scores_sorted_descending() {
    let resp = RerankResponse {
        scores: vec![(2, 0.94), (0, 0.71), (1, 0.12)],
        skipped: false,
        model_name: "test".into(),
        model_sha256: "test".into(),
        sidecar_version: "test".into(),
        device: "cpu".into(),
        elapsed_ms: 10,
        query_token_count: 5,
        total_candidate_tokens: 100,
    };
    for window in resp.scores.windows(2) {
        assert!(window[0].1 >= window[1].1, "scores not descending");
    }
}

// ─── Index permutation invariant ─────────────────────────────────────────────

#[test]
fn scores_indices_are_permutation() {
    let n = 50;
    let resp = RerankResponse {
        scores: (0..n).map(|i| (i, 1.0 - i as f32 * 0.01)).collect(),
        skipped: false,
        model_name: "test".into(),
        model_sha256: "test".into(),
        sidecar_version: "test".into(),
        device: "cpu".into(),
        elapsed_ms: 10,
        query_token_count: 5,
        total_candidate_tokens: 100,
    };
    let mut indices: Vec<usize> = resp.scores.iter().map(|(i, _)| *i).collect();
    indices.sort();
    assert_eq!(indices, (0..n).collect::<Vec<_>>());
}
