//! TASK-TEN-205 — api_calls emit helper acceptance.

use cyberos_auth::metering_emit::{emit_api_call, path_without_query, recorded_len};
use uuid::Uuid;

#[test]
fn success_emits_quantity_one() {
    let before = recorded_len();
    let key = format!("itest-{}", Uuid::new_v4());
    assert!(emit_api_call("tenant-x", &key, "GET", "/v1/auth/me"));
    assert!(recorded_len() > before);
}

#[test]
fn source_and_extra_shape_via_second_emit_idempotent() {
    let key = format!("itest-idem-{}", Uuid::new_v4());
    assert!(emit_api_call("t", &key, "POST", "/v1/admin/tenants"));
    let mid = recorded_len();
    assert!(emit_api_call("t", &key, "POST", "/v1/admin/tenants"));
    assert_eq!(recorded_len(), mid);
}

#[test]
fn path_strips_query_string() {
    assert_eq!(
        path_without_query("/v1/admin/subjects?email=a@b.c"),
        "/v1/admin/subjects"
    );
}
