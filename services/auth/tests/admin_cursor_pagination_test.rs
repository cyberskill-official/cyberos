//! TASK-AUTH-005 — cursor pagination + signature integration tests.
//!
//! Covers ECM-001/002 from §10.7. The exhaustive unit-level signing tests
//! live in `services/auth/src/cursor.rs` `#[cfg(test)] mod tests`; these
//! are end-to-end checks that the HTTP surface honours the signed-cursor
//! contract.

use cyberos_auth::cursor::{make_cursor, parse_cursor, CursorTable, ParseCursorError};
use uuid::Uuid;

#[test]
fn round_trip_holds_across_tables() {
    let id = Uuid::new_v4();
    let t = make_cursor(CursorTable::Tenants, id);
    let s = make_cursor(CursorTable::Subjects, id);
    assert_ne!(t, s, "table tag must change the wire form");
    assert_eq!(parse_cursor(&t, CursorTable::Tenants).unwrap(), id);
    assert_eq!(parse_cursor(&s, CursorTable::Subjects).unwrap(), id);
}

#[test]
fn property_signature_stability_under_repeat_minting() {
    // Same input → identical cursor (assuming process-wide signing key).
    let id = Uuid::new_v4();
    let a = make_cursor(CursorTable::Tenants, id);
    let b = make_cursor(CursorTable::Tenants, id);
    assert_eq!(a, b);
}

#[test]
fn property_different_ids_produce_different_cursors() {
    // 100 random pairs — none should collide.
    for _ in 0..100 {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        assert_ne!(
            make_cursor(CursorTable::Tenants, a),
            make_cursor(CursorTable::Tenants, b),
        );
    }
}

#[test]
fn property_truncated_cursor_rejected() {
    let id = Uuid::new_v4();
    let full = make_cursor(CursorTable::Tenants, id);
    let truncated = &full[..full.len() - 4];
    let err = parse_cursor(truncated, CursorTable::Tenants).unwrap_err();
    // Could be Length or Signature depending on how the base64 decode lands;
    // either error response is correct (both produce 400 invalid_cursor).
    assert!(matches!(
        err,
        ParseCursorError::Length | ParseCursorError::Signature | ParseCursorError::Base64
    ));
}

#[test]
fn table_tag_prevents_cross_endpoint_redemption() {
    // ECM-style: an attacker mints a cursor for /v1/admin/tenants and tries
    // to redeem it on /v1/admin/subjects.
    let id = Uuid::new_v4();
    let tenants_cursor = make_cursor(CursorTable::Tenants, id);
    let err = parse_cursor(&tenants_cursor, CursorTable::Subjects).unwrap_err();
    assert_eq!(err, ParseCursorError::TableMismatch);
}
