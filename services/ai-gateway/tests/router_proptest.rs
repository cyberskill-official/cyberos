//! TASK-AI-008 §5 — Property tests for the router (jitter bounds).

use cyberos_ai_gateway::router::jitter::jitter_ms;
use proptest::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

proptest! {
    /// AC #11: jitter at 200ms ± 20% stays within [160, 240].
    #[test]
    fn jitter_attempt2_stays_in_band(seed in 0u64..u64::MAX) {
        let mut rng = StdRng::seed_from_u64(seed);
        for _ in 0..1000 {
            let result = jitter_ms(200, 0.20, &mut rng);
            prop_assert!((160..=240).contains(&result), "jitter out of band: {}", result);
        }
    }

    /// AC #11 (extension): jitter at 800ms ± 20% stays within [640, 960].
    #[test]
    fn jitter_attempt3_stays_in_band(seed in 0u64..u64::MAX) {
        let mut rng = StdRng::seed_from_u64(seed);
        for _ in 0..1000 {
            let result = jitter_ms(800, 0.20, &mut rng);
            prop_assert!((640..=960).contains(&result), "jitter out of band: {}", result);
        }
    }

    /// Edge case: jitter with factor=0.0 must not panic (no modulo by zero).
    #[test]
    fn jitter_zero_factor_no_panic(base_ms in 1u32..10_000, seed in 0u64..u64::MAX) {
        let mut rng = StdRng::seed_from_u64(seed);
        let result = jitter_ms(base_ms, 0.0, &mut rng);
        prop_assert_eq!(result, base_ms);
    }
}
