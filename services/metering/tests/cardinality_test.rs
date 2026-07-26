use cyberos_metering::axes::{MeteringAxis, METERING_AXIS_CARDINALITY};
use cyberos_metering::policy::{OveragePolicy, OVERAGE_POLICY_CARDINALITY};

#[test]
fn metering_axis_cardinality() {
    assert_eq!(MeteringAxis::ALL.len(), METERING_AXIS_CARDINALITY);
    assert_eq!(METERING_AXIS_CARDINALITY, 4);
}

#[test]
fn overage_policy_cardinality() {
    assert_eq!(OveragePolicy::ALL.len(), OVERAGE_POLICY_CARDINALITY);
    assert_eq!(OVERAGE_POLICY_CARDINALITY, 3);
}
