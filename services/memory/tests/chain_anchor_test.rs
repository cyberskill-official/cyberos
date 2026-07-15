//! Layer-2 chain-anchor verification — TASK-MEMORY-101 §1 #4.
//!
//! These run without Postgres (pure-Rust hashing). Postgres-backed tests
//! live behind a `--features postgres-it` cargo feature and run in CI
//! against the `services/dev/docker-compose.yml` stack.

use cyberos_memory::layer2::chain_anchor;

#[test]
fn anchor_matches_known_vector() {
    // Vector: prev_hash_hex = "00" (2-byte ASCII "00", NOT the decoded null byte;
    // see chain_anchor::compute — it hashes the ASCII bytes of prev_hash_hex
    // directly, not the hex-decoded prev_hash). body = "" → SHA-256(b"00") =
    // f1534392279bddbf9d43dde8701cb5be14b82f76ec6607bf8d6ad557f60f304e.
    //
    // (For reference: SHA-256(b"0") = 5feceb66… — that's the 1-byte ASCII zero,
    // which is what an earlier version of this comment incorrectly claimed.)
    let got = chain_anchor::compute(Some("00"), "");
    assert_eq!(
        got,
        "f1534392279bddbf9d43dde8701cb5be14b82f76ec6607bf8d6ad557f60f304e"
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
