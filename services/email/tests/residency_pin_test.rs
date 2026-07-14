//! TASK-EMAIL-001 §4 #9 + §4 #10 — residency-pinning invariants.
//!
//! These are pure-Rust tests that don't need a live Postgres — the
//! residency::binding_for_residency function is the policy lookup; the
//! DB layer is a small wrapper on top.

use cyberos_email::residency::{assert_residency_match, binding_for_residency};
use uuid::Uuid;

#[test]
fn vn_tenant_resolves_to_ap_southeast_1_bucket() {
    let b = binding_for_residency("vn-1").unwrap();
    assert_eq!(b.region, "ap-southeast-1");
    assert_eq!(b.bucket, "cyberos-email-vn-1-bodies");
    assert_eq!(b.kms_key_id, "alias/cyberos-email-vn-1-bodies");
}

#[test]
fn sg_tenant_distinct_from_vn() {
    let sg = binding_for_residency("sg-1").unwrap();
    let vn = binding_for_residency("vn-1").unwrap();
    assert_eq!(sg.region, vn.region); // both in ap-southeast-1
    assert_ne!(sg.bucket, vn.bucket); // distinct buckets
    assert_ne!(sg.kms_key_id, vn.kms_key_id); // distinct KMS aliases
}

#[test]
fn eu_tenant_lands_in_eu_west_1() {
    let b = binding_for_residency("eu-1").unwrap();
    assert_eq!(b.region, "eu-west-1");
    assert_eq!(b.bucket, "cyberos-email-eu-1-bodies");
}

#[test]
fn us_tenant_lands_in_us_east_1() {
    let b = binding_for_residency("us-1").unwrap();
    assert_eq!(b.region, "us-east-1");
    assert_eq!(b.bucket, "cyberos-email-us-1-bodies");
}

#[test]
fn unknown_residency_rejected() {
    for bad in ["zz-1", "vn", "vn-2", "", "VN-1"] {
        let r = binding_for_residency(bad);
        assert!(r.is_err(), "expected unknown for '{bad}'");
    }
}

#[test]
fn cross_residency_write_is_fail_closed() {
    // TASK-EMAIL-001 §1 #12 — VN tenant body must NOT land in eu-1 bucket.
    let tid = Uuid::new_v4();
    let vn = binding_for_residency("vn-1").unwrap();
    let err = assert_residency_match(tid, &vn, "cyberos-email-eu-1-bodies").unwrap_err();
    assert_eq!(err.code(), "residency_mismatch");
}

#[test]
fn matching_residency_passes() {
    let tid = Uuid::new_v4();
    let sg = binding_for_residency("sg-1").unwrap();
    assert!(assert_residency_match(tid, &sg, "cyberos-email-sg-1-bodies").is_ok());
}
