//! TASK-AI-016 §5 — Property tests for residency matching.
//!
//! AC #8: no cross-residency leak — proptest 1000 trials over (any_residency, any_region).

use proptest::prelude::*;

use cyberos_ai_gateway::policy::Residency;
use cyberos_ai_gateway::residency::{self, Region};

fn any_residency() -> impl Strategy<Value = Residency> {
    prop_oneof![
        Just(Residency::Sg1),
        Just(Residency::Eu1),
        Just(Residency::Us1),
        Just(Residency::Vn1),
    ]
}

fn any_region() -> impl Strategy<Value = Region> {
    let known_regions = [
        "ap-southeast-1",
        "ap-southeast-2",
        "ap-northeast-1",
        "eu-central-1",
        "eu-west-1",
        "eu-west-2",
        "eu-north-1",
        "us-east-1",
        "us-east-2",
        "us-west-1",
        "us-west-2",
        "ca-central-1",
        "sa-east-1",
    ];
    prop::sample::select(known_regions.to_vec())
        .prop_map(|r| Region::from_provider_string(r).unwrap())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn no_cross_residency_leak(r in any_residency(), region in any_region()) {
        // AC #8: matches(R, region) → true ⇒ region is in the expected set for R
        if residency::matches(r, &region) {
            let expected = match r {
                Residency::Sg1 => vec!["ap-southeast-1"],
                Residency::Eu1 => vec!["eu-central-1", "eu-west-1"],
                Residency::Us1 => vec!["us-east-1", "us-east-2", "us-west-2"],
                Residency::Vn1 => vec![],
            };
            prop_assert!(
                expected.contains(&region.as_str()),
                "cross-residency leak: {:?} matched {:?} but region not in expected set {:?}",
                r, region, expected
            );
        }
    }

    #[test]
    fn deterministic(r in any_residency(), region in any_region()) {
        // §1 #9: same pair → same result, run twice.
        let r1 = residency::matches(r, &region);
        let r2 = residency::matches(r, &region);
        prop_assert_eq!(r1, r2);
    }
}
