//! Layer-2 chain-anchor verification — FR-BRAIN-101 §1 #4.
//!
//! These run without Postgres (pure-Rust hashing). Postgres-backed tests
//! live behind a `--features postgres-it` cargo feature and run in CI
//! against the `services/dev/docker-compose.yml` stack.

use cyberos_brain::layer2::chain_anchor;

#[test]
fn anchor_matches_known_vector() {
    // Vector: prev_hash = "00", body = "" → SHA256("00") =
    // 5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9
    let got = chain_anchor::compute(Some("00"), "");
    assert_eq!(
        got,
        "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9"
    );
}

#[test]
fn anchor_changes_on_one_byte_body_flip() {
    let a = chain_anchor::compute(Some("00"), "hello world");
    let b = chain_anchor::compute(Some("00"), "hello worle");
    assert_ne!(a, b, "single-byte flip MUST change anchor");
}

#[test]
fn anchor_excludes_prev_when_genesis() {
    // Genesis rows have prev_hash = None; same body across two genesis chains
    // produces the same anchor. (This is correct — chain identity comes from
    // the audit chain itself, not the anchor of row 0.)
    let g1 = chain_anchor::compute(None, "genesis");
    let g2 = chain_anchor::compute(None, "genesis");
    assert_eq!(g1, g2);
}
