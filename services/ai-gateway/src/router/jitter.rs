//! TASK-AI-008 §3 — Jitter helper for exponential backoff.

use rand::Rng;

/// Returns `base_ms + uniform_jitter(±factor * base_ms)`.
///
/// Safe for `factor = 0.0` (returns `base_ms` unchanged).
/// Safe for any `base_ms` (no modulo, no overflow on i32 conversion).
pub fn jitter_ms<R: Rng>(base_ms: u32, factor: f64, rng: &mut R) -> u32 {
    if factor <= 0.0 || base_ms == 0 {
        return base_ms;
    }
    let delta = (base_ms as f64 * factor) as i32;
    if delta == 0 {
        return base_ms;
    }
    let offset = rng.gen_range(-delta..=delta);
    let result = base_ms as i64 + offset as i64;
    result.clamp(0, u32::MAX as i64) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn jitter_200ms_stays_in_band() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..10_000 {
            let result = jitter_ms(200, 0.20, &mut rng);
            assert!(
                (160..=240).contains(&result),
                "jitter out of band: {}",
                result
            );
        }
    }

    #[test]
    fn jitter_800ms_stays_in_band() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..10_000 {
            let result = jitter_ms(800, 0.20, &mut rng);
            assert!(
                (640..=960).contains(&result),
                "jitter out of band: {}",
                result
            );
        }
    }

    #[test]
    fn jitter_zero_factor_returns_base() {
        let mut rng = StdRng::seed_from_u64(42);
        assert_eq!(jitter_ms(200, 0.0, &mut rng), 200);
    }

    #[test]
    fn jitter_zero_base_returns_zero() {
        let mut rng = StdRng::seed_from_u64(42);
        assert_eq!(jitter_ms(0, 0.20, &mut rng), 0);
    }

    #[test]
    fn jitter_negative_factor_returns_base() {
        let mut rng = StdRng::seed_from_u64(42);
        assert_eq!(jitter_ms(200, -0.5, &mut rng), 200);
    }
}
