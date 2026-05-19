//! FR-AUTH-005 — in-memory deny-list behaviour tests.
//!
//! Covers G-011 (deny-list primitive) + G-012 (deny-list-survives-unrevoke
//! security default) + the §1 #11 propagation-SLO assertion that "within
//! 30 seconds" is trivially satisfied because the deny-list lives in the
//! same process as `verify_jwt` (zero propagation distance).
//!
//! Full unit-level coverage of the `DenyList` type is in
//! `services/auth/src/deny_list.rs` `#[cfg(test)] mod tests`. These are
//! end-to-end assertions about the public contract.

use cyberos_auth::deny_list::DenyList;
use std::time::Duration;

#[test]
fn deny_list_propagation_is_synchronous_in_process() {
    // §1 #11 SLO: "propagate within 30 seconds". In the single-process
    // topology this is trivially zero seconds — deny() returns BEFORE the
    // next is_denied() call observes the state, because they share Arc-RwLock.
    let d = DenyList::new();
    let start = std::time::Instant::now();
    d.deny_for("jti-prop", Duration::from_secs(60));
    let observed = d.is_denied("jti-prop");
    let elapsed = start.elapsed();
    assert!(observed);
    assert!(elapsed < Duration::from_millis(10), "in-process propagation should be sub-ms; got {elapsed:?}");
}

#[test]
fn deny_list_is_clone_arc_shared() {
    // Cloning the AppState clones the deny_list field. The clone MUST share
    // state with the original (otherwise the middleware sees a stale view).
    let original = DenyList::new();
    let cloned = original.clone();
    original.deny_for("jti-shared", Duration::from_secs(60));
    assert!(cloned.is_denied("jti-shared"));
}

#[test]
fn deny_list_expiry_self_evicts() {
    // Insert a jti with past expiry → is_denied returns false; len() reports
    // 0 (post-GC). This is the natural cleanup path; no manual sweeper is
    // strictly required for correctness.
    let d = DenyList::new();
    d.deny("jti-expired", std::time::Instant::now() - Duration::from_secs(1));
    assert!(!d.is_denied("jti-expired"));
    assert_eq!(d.len(), 0);
}

#[test]
fn unrevoke_security_default_no_remove_api() {
    // ECM-010 / G-012: the public surface of DenyList exposes no `remove()`,
    // `clear()`, or `un_deny()` — the only removal path is natural expiry.
    // This compile-time invariant is what enforces "unrevoke does NOT clear
    // the deny-list" structurally, not by handler discipline alone.
    //
    // This test is intentionally a structural-only assertion: if a future PR
    // adds `pub fn remove()`, this test still compiles, but the §10.7
    // ECM-010 audit row demands the security review approve such an addition.
    let d = DenyList::new();
    d.deny_for("jti-a", Duration::from_secs(60));
    assert!(d.is_denied("jti-a"));
    // d.remove("jti-a")  ←  compile error if uncommented; that's the point.
    assert!(d.is_denied("jti-a"));
}
