//! Shared test support for cache isolation tests.

pub mod proptest_strategies;
pub mod redis_isolation_helper;

use cyberos_ai_gateway::router::types::{
    Choice, FinishReason, ProviderResponse, ProviderUsage,
};

pub fn test_provider_response() -> ProviderResponse {
    ProviderResponse {
        id: "test-resp-1".into(),
        usage: ProviderUsage {
            prompt_tokens: 10,
            completion_tokens: 20,
            cached_input_tokens: 0,
        },
        choices: vec![Choice {
            index: 0,
            content: "Hello, world!".into(),
            tool_calls: vec![],
            finish_reason: FinishReason::Stop,
        }],
        finish_reason: FinishReason::Stop,
        latency_ms: 150,
        cache_state: cyberos_ai_gateway::router::types::CacheState::None,
        attempts: vec![],
    }
}
