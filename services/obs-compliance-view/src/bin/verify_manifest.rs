//! Standalone offline manifest verifier (FR-OBS-009 §1 #7, DEC-183). Auditors run this with the
//! published public key and the exported files, with no CyberOS access:
//!
//!   verify_manifest --manifest <manifest.json> --rows <rows.json> --pubkey <hex32>
//!
//! Exits 0 on PASS, 1 on FAIL, 2 on a usage error.

use cyberos_obs_compliance_view::manifest::Manifest;
use cyberos_obs_compliance_view::manifest_signing::{verify, Verdict};

fn arg(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn parse_hex32(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let b = s.as_bytes();
    let mut out = [0u8; 32];
    for (i, slot) in out.iter_mut().enumerate() {
        let hi = hex_val(b[i * 2])?;
        let lo = hex_val(b[i * 2 + 1])?;
        *slot = (hi << 4) | lo;
    }
    Some(out)
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (Some(mpath), Some(rpath), Some(pubhex)) = (
        arg(&args, "--manifest"),
        arg(&args, "--rows"),
        arg(&args, "--pubkey"),
    ) else {
        eprintln!("usage: verify_manifest --manifest <manifest.json> --rows <rows.json> --pubkey <hex32>");
        std::process::exit(2);
    };

    let manifest: Manifest = match std::fs::read_to_string(&mpath) {
        Ok(s) => match serde_json::from_str(&s) {
            Ok(m) => m,
            Err(e) => fail(&format!("cannot parse manifest: {e}")),
        },
        Err(e) => fail(&format!("cannot read manifest {mpath}: {e}")),
    };
    let rows = match std::fs::read(&rpath) {
        Ok(r) => r,
        Err(e) => fail(&format!("cannot read rows {rpath}: {e}")),
    };
    let Some(pubkey) = parse_hex32(&pubhex) else {
        fail("--pubkey must be 64 hex chars");
    };

    match verify(&pubkey, &manifest, &rows) {
        Verdict::Pass => println!("PASS: manifest {} verified", manifest.export_id),
        Verdict::Fail(reason) => fail(reason),
    }
}

fn fail(reason: &str) -> ! {
    println!("FAIL: {reason}");
    std::process::exit(1);
}
