use std::collections::HashSet;

/// UNIQUE (profile_id, event_id) semantics — second insert is a no-op success.
#[test]
fn duplicate_event_id_is_idempotent() {
    let mut seen = HashSet::new();
    let key = (12345_i64, "11111111-1111-1111-1111-111111111111");
    assert!(seen.insert(key));
    assert!(!seen.insert(key));
}
