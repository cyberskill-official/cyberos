# Plan caps (TASK-TEN-002)

Single source of truth: [`services/ten/src/plans/caps.rs`](../../../services/ten/src/plans/caps.rs).

| tier | seats | api_calls/mo | ai_tokens/mo | storage |
|---|---|---|---|---|
| starter | 3 | 10_000 | 100_000 | 1 GiB |
| team | 25 | 500_000 | 5_000_000 | 100 GiB |
| enterprise | ∞ | ∞ | 50_000_000 | 1 TiB |

Decisions: DEC-770 … DEC-783. Do not hardcode these caps in any other crate.
