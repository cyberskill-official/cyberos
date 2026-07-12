---
id: FR-OBS-009
title: "Chain-of-custody manifest with Ed25519 signature on every compliance export — PDF cover + JSON sidecar + audit row + verifier CLI"
module: OBS
priority: MUST
status: implementing
verify: T
phase: P0
milestone: P0 · slice 3
slice: 3
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_frs: [FR-OBS-008, FR-AUTH-006]
depends_on: [FR-OBS-008]
blocks: []

source_pages:
  - website/docs/modules/obs.html#chain-of-custody
source_decisions:
  - DEC-180 (Ed25519 over chain-head-at-export + row-hash; auditor independently verifies)
  - DEC-181 (manifest accompanies BOTH PDF cover + JSON sidecar; both must be tamper-evident)
  - DEC-182 (interrupted export = state: incomplete; never trust silent partials)
  - DEC-183 (verifier CLI ships separately so auditors can verify offline without CyberOS infra access)

language: rust 1.81
service: cyberos/services/obs-compliance-view/
new_files:
  - services/obs-compliance-view/src/manifest.rs
  - services/obs-compliance-view/src/manifest_signing.rs
  - services/obs-compliance-view/src/manifest_pdf.rs
  - services/obs-compliance-view/src/bin/verify_manifest.rs       # standalone verifier
  - services/obs-compliance-view/tests/manifest_test.rs
  - services/obs-compliance-view/tests/manifest_verify_test.rs
  - services/obs-compliance-view/tests/manifest_interrupted_test.rs
  - services/obs-compliance-view/docs/manifest-format.md
modified_files:
  - services/obs-compliance-view/src/views/{eu_ai_act,pdpl,soc2,iso27001}.rs   # call manifest::sign() on export
  - services/obs-compliance-view/src/export/{pdf,json}.rs                       # attach manifest to output
allowed_tools:
  - file_read: services/obs-compliance-view/**
  - file_write: services/obs-compliance-view/**
  - bash: cd services/obs-compliance-view && cargo test manifest && cargo build --bin verify_manifest
disallowed_tools:
  - export compliance data without manifest (per §1 #1)
  - skip memory audit row on export (per §1 #5)
  - silently truncate exports (per §1 #6 — must mark state: incomplete)
  - sign with anything other than Ed25519 (per DEC-180)

effort_hours: 8
sub_tasks:
  - "0.5h: manifest.rs — ChainOfCustodyManifest struct + ExportState enum"
  - "1.0h: manifest_signing.rs — Ed25519 sign over canonical(manifest_minus_signature)"
  - "0.5h: SHA-256 of canonical row dump (deterministic JSON)"
  - "0.5h: memory MMR head query (uses memory existing API)"
  - "1.0h: verify_manifest.rs standalone binary (auditor runs offline)"
  - "1.0h: manifest_pdf.rs — PDF cover page rendering with manifest fields + QR code"
  - "0.5h: JSON sidecar format (rows.json + manifest.json paired)"
  - "0.5h: ExportState::Incomplete on panic-during-streaming"
  - "0.5h: canonical::export_compliance memory audit row builder"
  - "1.0h: Integration into views (every view's export calls sign())"
  - "1.5h: Tests — manifest creation + signature verify + offline verifier + incomplete state + PDF render"
  - "0.5h: docs/manifest-format.md (auditor-facing reference)"
risk_if_skipped: "Auditor receives a JSON dump. No proof it's authentic. Could be tampered with mid-flight (operator deletes incriminating rows before sending). PDPL Art. 5 + SOC 2 CC7 chain-of-custody requirements unmet. Without offline verifier, auditor must trust online verification (we sign + verify ourselves) — defeats independent-verification principle."
---

## §1 — Description (BCP-14 normative)

Every compliance export from FR-OBS-008 **MUST** be accompanied by a chain-of-custody manifest. Each manifest:

1. **MUST** include the following fields:
    - `export_id` (ULID-26).
    - `tenant_id` (UUID).
    - `regulation` (string: `"EU AI Act" | "PDPL" | "SOC 2" | "ISO 27001"`).
    - `time_range` (ISO8601 tuple).
    - `row_count` (u64).
    - `chain_head_at_export` (32-byte memory MMR root, hex-encoded).
    - `exporter` (auditor JWT subject_id + email).
    - `exported_at` (ISO8601 UTC).
    - `sha256_of_rows` (32 bytes hex; deterministic over canonical JSON of rows).
    - `ed25519_signature` (64 bytes base64, over canonical-bytes-of-everything-above).
    - `public_key_id` (string: identifier for the signing key version, e.g., `"cyberos-infra-2026-Q2"`).
    - `state` (enum: `Complete | Incomplete`).
2. **MUST** sign with the CyberOS infrastructure Ed25519 key. The key is rotated quarterly via FR-AUTH-006-style sweeper; key versions enumerate in `public_key_id`.
3. **MUST** include the public key ID in the manifest. Auditors fetch the corresponding public key from `https://keys.cyberos.world/<public_key_id>.pub` (a static file served via CDN; no auth required for public keys).
4. **MUST** be both human-readable (PDF cover page with manifest fields + QR code linking to verifier) AND machine-verifiable (JSON sidecar with raw bytes for cryptographic verification).
5. **MUST** record the export to memory as `obs.export_compliance` audit row before returning to the caller. The row carries `export_id`, `tenant_id`, `regulation`, `row_count`, `exporter_subject_id`, `chain_head_at_export`, `request_id`. Self-anchoring: the export itself is in the chain.
6. **MUST** prevent partial exports — if export is interrupted (panic, network drop mid-stream), the manifest carries `state: Incomplete` AND the memory row records the incompleteness. Incomplete manifests fail offline verification (ExportState mismatch).
7. **MUST** ship a standalone verifier binary `verify_manifest` that auditors run offline. The verifier:
    - Takes a manifest JSON path + rows JSON path.
    - Fetches the public key from `keys.cyberos.world` (or accepts via `--pubkey` flag for fully-offline use).
    - Verifies the Ed25519 signature.
    - Recomputes SHA-256 of rows; compares to `sha256_of_rows`.
    - Outputs PASS/FAIL with reason.
8. **MUST** use deterministic canonical-JSON serialisation for both `sha256_of_rows` AND the signed bytes. The same rows + same manifest input always produce the same hash + signature. RFC 8785 JCS is the standard.
9. **MUST** complete signing in ≤ 100ms per export (Ed25519 is microseconds; the 100ms budget covers JSON serialisation + key load).
10. **MUST** attach the manifest to BOTH PDF and JSON exports:
    - PDF: cover page renders manifest fields + QR code linking to `https://verify.cyberos.world/?export_id=<id>` (online verifier).
    - JSON: paired files `<export_id>_rows.json` + `<export_id>_manifest.json` in a single zip.
11. **SHOULD** emit OTel metrics:
    - `obs_export_compliance_total{regulation, state}` (counter).
    - `obs_export_signing_latency_ms` (histogram).
    - `obs_export_verification_total{outcome}` (counter; verify_manifest binary's metrics if telemetry-enabled).

---

## §2 — Why this design (rationale for humans)

**Why chain-of-custody manifest at all (DEC-180)?** Auditors need TRACEABILITY. Without proof, an export is just JSON — could be tampered with at any point between server and auditor. The manifest answers: "where did this come from? at what memory chain state? signed by whom?"

**Why Ed25519 specifically?** Modern, fast, small signatures (64 bytes), broad library support (Python, Go, Rust, JS verifiers). RSA-2048 would also work but produces larger signatures + slower verification. ECDSA P-256 has equivalent security but more implementation pitfalls historically. Ed25519 is the right modern default.

**Why offline verifier (DEC-183)?** Auditor's value depends on independent verification — if verification requires CyberOS infra access, the auditor depends on the entity being audited. The standalone verifier + public key on a CDN means the auditor verifies WITHOUT touching CyberOS systems.

**Why PDF cover + JSON sidecar (DEC-181)?** Auditors prefer PDF for reading (familiar format, signature-able by their own tools). JSON for machine verification (parseable, deterministic). Both formats convey the same trust; the manifest fields are identical.

**Why state: Incomplete on interrupted export (DEC-182)?** Silent partial exports are catastrophic — auditor reviews 50% of rows thinking they're 100%, misses the bad ones. Marking incomplete + failing offline verification ensures the auditor knows.

**Why memory audit row of the export (§1 #5)?** Self-anchoring. The export is itself in the chain; a regulator asking "did you export anything during the period under review?" gets a positive answer (rows showing each export).

**Why public key on CDN (§1 #3)?** Auditors verifying offline need the public key. Hosting on a public CDN (no auth) means the verifier works from anywhere. The CDN doesn't need to be trustworthy — the public key can be cross-checked against an out-of-band channel (operator emails the key fingerprint to auditor at engagement start).

**Why deterministic canonical JSON (§1 #8)?** Same rows must produce same signature. Without canonicalisation (sort keys, normalize whitespace), two valid JSON serialisations of the same data produce different signatures — verification fails despite identical content. RFC 8785 JCS is the standard.

**Why QR code on PDF (§1 #10)?** Auditors scanning the PDF can verify online with one click instead of typing the export_id. Bridge between paper-based audit and digital verification.

**Why 100ms signing budget (§1 #9)?** Ed25519 itself is sub-millisecond; the 100ms covers serialisation + key load. Above 100ms, exports become noticeably slow — auditors notice.

---

## §3 — API contract

```rust
// services/obs-compliance-view/src/manifest.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ChainOfCustodyManifest {
    pub export_id: String,                      // ULID-26
    pub tenant_id: Uuid,
    pub regulation: String,
    pub time_range: (DateTime<Utc>, DateTime<Utc>),
    pub row_count: u64,
    pub chain_head_at_export: String,           // hex 64 chars
    pub exporter: ExporterInfo,
    pub exported_at: DateTime<Utc>,
    pub sha256_of_rows: String,                 // hex 64 chars
    pub ed25519_signature: String,              // base64 (64 bytes → 88 base64)
    pub public_key_id: String,                  // e.g., "cyberos-infra-2026-Q2"
    pub state: ExportState,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ExporterInfo {
    pub subject_id: Uuid,
    pub email_hash16: String,                   // 16-hex of SHA-256(email)
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ExportState { Complete, Incomplete }

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("private signing key unavailable: {0}")]
    SigningKeyUnavailable(String),
    #[error("memory MMR head query failed: {0}")]
    MemoryQueryFailed(String),
    #[error("canonicalisation failed: {0}")]
    Canonicalisation(String),
    #[error("export interrupted")]
    Interrupted,
}

pub async fn sign(rows: &[AuditRow], regulation: &str, exporter: &Claims, time_range: (DateTime<Utc>, DateTime<Utc>))
    -> Result<ChainOfCustodyManifest, ManifestError>
{
    let row_hash = sha256_canonical(rows)?;
    let chain_head = memory::current_mmr_head().await?;
    let key = manifest_signing::load_active_key().await?;

    let mut manifest = ChainOfCustodyManifest {
        export_id: ulid::Ulid::new().to_string(),
        tenant_id: exporter.tenant_id,
        regulation: regulation.into(),
        time_range,
        row_count: rows.len() as u64,
        chain_head_at_export: hex::encode(chain_head),
        exporter: ExporterInfo {
            subject_id: exporter.subject_id,
            email_hash16: hex::encode(&sha256(exporter.email.as_bytes())[..8]),
        },
        exported_at: Utc::now(),
        sha256_of_rows: hex::encode(row_hash),
        ed25519_signature: String::new(),       // placeholder
        public_key_id: key.id.clone(),
        state: ExportState::Complete,
    };

    let canonical_bytes = canonicalise_for_signing(&manifest)?;
    let sig = key.signing_key.sign(&canonical_bytes);
    manifest.ed25519_signature = base64::encode(sig.to_bytes());

    memory::emit(canonical::export_compliance(&manifest)).await?;

    Ok(manifest)
}

fn canonicalise_for_signing(m: &ChainOfCustodyManifest) -> Result<Vec<u8>, ManifestError> {
    // Build a JSON object identical to manifest BUT with ed25519_signature = ""
    // Then serialize via RFC 8785 JCS.
    let mut clone = m.clone();
    clone.ed25519_signature = String::new();
    serde_jcs::to_vec(&clone).map_err(|e| ManifestError::Canonicalisation(e.to_string()))
}

pub fn sha256_canonical(rows: &[AuditRow]) -> Result<[u8; 32], ManifestError> {
    let bytes = serde_jcs::to_vec(rows).map_err(|e| ManifestError::Canonicalisation(e.to_string()))?;
    Ok(sha256(&bytes))
}
```

```rust
// services/obs-compliance-view/src/bin/verify_manifest.rs
//! Offline verifier — auditor runs locally without CyberOS infra access.

use clap::Parser;
use ed25519_dalek::{VerifyingKey, Signature, Verifier};

#[derive(Parser)]
#[command(name = "verify_manifest", version)]
struct Cli {
    #[arg(long)] manifest: PathBuf,
    #[arg(long)] rows: PathBuf,
    #[arg(long, help = "Public key file (PEM); if absent, fetches from keys.cyberos.world")]
    pubkey: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let manifest: ChainOfCustodyManifest = serde_json::from_str(&std::fs::read_to_string(&cli.manifest)?)?;
    let rows_bytes = std::fs::read(&cli.rows)?;

    // 1. Verify SHA-256 of rows
    let actual_row_hash = sha256(&rows_bytes);
    let expected = hex::decode(&manifest.sha256_of_rows)?;
    if actual_row_hash != expected[..] {
        eprintln!("❌ FAIL: sha256_of_rows mismatch");
        eprintln!("   expected: {}", manifest.sha256_of_rows);
        eprintln!("   actual:   {}", hex::encode(actual_row_hash));
        std::process::exit(1);
    }

    // 2. Load public key
    let pubkey: VerifyingKey = match cli.pubkey {
        Some(p) => load_pem(&p)?,
        None => fetch_from_cdn(&manifest.public_key_id)?,
    };

    // 3. Recompute canonical bytes (manifest with empty signature)
    let canonical_bytes = manifest_canonicalise_for_verify(&manifest)?;
    let sig_bytes = base64::decode(&manifest.ed25519_signature)?;
    let sig = Signature::from_bytes(&sig_bytes.try_into().unwrap());

    // 4. Verify signature
    pubkey.verify(&canonical_bytes, &sig)
        .map_err(|e| anyhow::anyhow!("signature verify failed: {e}"))?;

    // 5. Check state
    if manifest.state != ExportState::Complete {
        eprintln!("❌ FAIL: export state is {:?}; manifest cannot be trusted", manifest.state);
        std::process::exit(1);
    }

    println!("✅ PASS: manifest valid");
    println!("   export_id: {}", manifest.export_id);
    println!("   regulation: {}", manifest.regulation);
    println!("   exporter: {}", manifest.exporter.subject_id);
    println!("   exported_at: {}", manifest.exported_at);
    println!("   row_count: {}", manifest.row_count);
    println!("   chain_head: {}", manifest.chain_head_at_export);
    Ok(())
}
```

```rust
// services/obs-compliance-view/src/manifest_pdf.rs
pub fn render_cover_page(manifest: &ChainOfCustodyManifest) -> Vec<u8> {
    let html = format!(r#"
<html><head><title>Compliance Export — {}</title></head><body>
<h1>Chain-of-Custody Manifest</h1>
<table>
  <tr><th>Export ID</th><td>{}</td></tr>
  <tr><th>Regulation</th><td>{}</td></tr>
  <tr><th>Tenant</th><td>{}</td></tr>
  <tr><th>Time Range</th><td>{} to {}</td></tr>
  <tr><th>Row Count</th><td>{}</td></tr>
  <tr><th>Exporter</th><td>{}</td></tr>
  <tr><th>Exported At</th><td>{}</td></tr>
  <tr><th>memory Chain Head</th><td><code>{}</code></td></tr>
  <tr><th>SHA-256 of Rows</th><td><code>{}</code></td></tr>
  <tr><th>Public Key ID</th><td><code>{}</code></td></tr>
  <tr><th>State</th><td>{:?}</td></tr>
</table>
<h2>Verify Online</h2>
<img src="data:image/png;base64,{}" />
<p>Or run offline: <code>verify_manifest --manifest manifest.json --rows rows.json</code></p>
</body></html>"#,
        manifest.regulation, manifest.export_id, manifest.regulation,
        manifest.tenant_id, manifest.time_range.0, manifest.time_range.1,
        manifest.row_count, manifest.exporter.subject_id, manifest.exported_at,
        manifest.chain_head_at_export, manifest.sha256_of_rows,
        manifest.public_key_id, manifest.state,
        generate_qr_code_base64(&format!("https://verify.cyberos.world/?export_id={}", manifest.export_id)),
    );
    wkhtmltopdf::convert(&html).unwrap()
}
```

---

## §4 — Acceptance criteria

1. **Every export attaches a manifest** — both PDF and JSON exports include the manifest.
2. **Signature verifies with the public key** — `verify_manifest` returns PASS.
3. **SHA-256 of rows matches `sha256_of_rows`** — recompute equals stored value.
4. **Chain head at export matches memory's current head** at export time.
5. **Manifest export records `obs.export_compliance` memory row** with all fields.
6. **Interrupted export → state: Incomplete** — panic during streaming → manifest carries Incomplete; verifier flags it as untrusted.
7. **PDF cover page renders manifest readably** — fields visible; QR code present.
8. **JSON sidecar parseable + signature-verifiable by external tooling**.
9. **Offline verifier works without infra access** — `verify_manifest --manifest m.json --rows r.json --pubkey k.pem` returns PASS.
10. **Online verifier (CDN-fetched key) works** — `verify_manifest --manifest m.json --rows r.json` (no --pubkey) → fetches from keys.cyberos.world; returns PASS.
11. **Tampered rows fail verification** — modify one byte in rows.json → SHA-256 mismatch → FAIL.
12. **Tampered manifest fails verification** — modify one field in manifest.json → signature mismatch → FAIL.
13. **Wrong public key fails verification** — use different key → signature verify fails.
14. **Signing latency < 100ms** — measured per export.
15. **Public key on CDN matches signed manifests** — operator's keys.cyberos.world entry equals the in-process signing key.
16. **State enum serialises/deserialises correctly** — JSON round-trip preserves Complete/Incomplete.
17. **Quarterly key rotation supported** — new `public_key_id` after rotation; both keys served from CDN during overlap.

---

## §5 — Verification

```rust
#[tokio::test]
async fn manifest_signature_verifies() {
    let rows = test_rows_for_eu_ai_act();
    let claims = test_auditor_claims();
    let manifest = manifest::sign(&rows, "EU AI Act", &claims, test_time_range()).await.unwrap();

    let canonical = manifest::canonicalise_for_signing(&manifest).unwrap();
    let pubkey = test_helper::active_public_key();
    let sig = base64::decode(&manifest.ed25519_signature).unwrap();
    pubkey.verify(&canonical, &Signature::from_bytes(&sig.try_into().unwrap())).unwrap();
}

#[tokio::test]
async fn sha256_of_rows_correct() {
    let rows = test_rows_for_eu_ai_act();
    let claims = test_auditor_claims();
    let manifest = manifest::sign(&rows, "EU AI Act", &claims, test_time_range()).await.unwrap();

    let computed = manifest::sha256_canonical(&rows).unwrap();
    assert_eq!(hex::encode(computed), manifest.sha256_of_rows);
}

#[tokio::test]
async fn memory_export_compliance_row_emitted() {
    let rows = test_rows_for_eu_ai_act();
    let claims = test_auditor_claims();
    let manifest = manifest::sign(&rows, "EU AI Act", &claims, test_time_range()).await.unwrap();

    let row = memory_test_helper::find_latest("obs.export_compliance").unwrap();
    assert_eq!(row.payload["export_id"], manifest.export_id);
    assert_eq!(row.payload["row_count"], manifest.row_count);
}

#[tokio::test]
async fn interrupted_export_marks_incomplete() {
    let rows = test_rows_for_eu_ai_act();
    let claims = test_auditor_claims();
    test_helper::inject_panic_during_streaming();
    let manifest = manifest::sign(&rows, "EU AI Act", &claims, test_time_range()).await.unwrap();
    assert_eq!(manifest.state, ExportState::Incomplete);
}

#[tokio::test]
async fn tampered_rows_fail_offline_verify() {
    let rows = test_rows_for_eu_ai_act();
    let claims = test_auditor_claims();
    let manifest = manifest::sign(&rows, "EU AI Act", &claims, test_time_range()).await.unwrap();

    let mut tampered = rows.clone();
    tampered[0].payload["leaked"] = serde_json::Value::String("malicious".into());

    let recomputed_hash = manifest::sha256_canonical(&tampered).unwrap();
    assert_ne!(hex::encode(recomputed_hash), manifest.sha256_of_rows);
    // Offline verifier would fail at this check.
}

#[tokio::test]
async fn tampered_manifest_field_fails_signature() {
    let rows = test_rows_for_eu_ai_act();
    let claims = test_auditor_claims();
    let mut manifest = manifest::sign(&rows, "EU AI Act", &claims, test_time_range()).await.unwrap();
    manifest.row_count = 999999;   // tamper

    let canonical = manifest::canonicalise_for_signing(&manifest).unwrap();
    let pubkey = test_helper::active_public_key();
    let sig = base64::decode(&manifest.ed25519_signature).unwrap();
    let result = pubkey.verify(&canonical, &Signature::from_bytes(&sig.try_into().unwrap()));
    assert!(result.is_err());
}

#[tokio::test]
async fn signing_latency_under_100ms() {
    let mut samples = vec![];
    for _ in 0..100 {
        let t0 = std::time::Instant::now();
        let _ = manifest::sign(&test_rows_for_eu_ai_act(), "EU AI Act", &test_auditor_claims(), test_time_range()).await.unwrap();
        samples.push(t0.elapsed().as_millis() as u64);
    }
    samples.sort();
    let p95 = samples[(samples.len() as f64 * 0.95) as usize];
    assert!(p95 < 100, "p95 {p95}ms exceeds 100ms");
}

// Verifier binary integration test
#[test]
fn verifier_binary_passes_on_valid_export() {
    let m_path = "/tmp/test_manifest.json";
    let r_path = "/tmp/test_rows.json";
    test_helper::write_test_export(m_path, r_path);

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_verify_manifest"))
        .args(&["--manifest", m_path, "--rows", r_path, "--pubkey", "/opt/cyberos/test_pubkey.pem"])
        .output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("✅ PASS"));
}
```

---

## §6 — Implementation skeleton

See §3.

---

## §7 — Dependencies

- **FR-OBS-008** — compliance views; this FR signs their exports.
- **FR-AUTH-006** — bootstrap CLI generates initial signing key; quarterly rotation cron.
- memory MMR head access (existing API).
- Crates: `ed25519-dalek@2`, `serde-jcs@0.1`, `wkhtmltopdf` (PDF), `qrcode@0.14`, `clap@4`, `base64`, `hex`, `ulid`.
- CDN for public keys: `https://keys.cyberos.world/<public_key_id>.pub` (static file hosting).

---

## §8 — Example payloads

### Manifest JSON

```json
{
  "export_id": "01HZK9R7A2B4C8D6...",
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
  "regulation": "EU AI Act",
  "time_range": ["2026-01-01T00:00:00Z", "2026-05-31T23:59:59Z"],
  "row_count": 12450,
  "chain_head_at_export": "a3f9c8d7e6b5a4f3e2d1c0b9a8f7e6d5c4b3a2f1e0d9c8b7a6f5e4d3c2b1a0f9",
  "exporter": {
    "subject_id": "auditor-...",
    "email_hash16": "4b8c0d2f1a7e9c3b"
  },
  "exported_at": "2026-06-15T14:30:00Z",
  "sha256_of_rows": "...",
  "ed25519_signature": "base64-encoded-sig...",
  "public_key_id": "cyberos-infra-2026-Q2",
  "state": "complete"
}
```

### Verifier output (success)

```text
$ verify_manifest --manifest manifest.json --rows rows.json
✅ PASS: manifest valid
   export_id: 01HZK9R7A2B4C8D6...
   regulation: EU AI Act
   exporter: auditor-...
   exported_at: 2026-06-15T14:30:00Z
   row_count: 12450
   chain_head: a3f9c8d7e6b5a4f3...
```

### Verifier output (failure)

```text
$ verify_manifest --manifest tampered.json --rows rows.json
❌ FAIL: sha256_of_rows mismatch
   expected: a3f9c8d7e6b5a4f3...
   actual:   9d6e3a2b1c0f8e7d...
```

### `obs.export_compliance` audit row

```json
{
  "kind": "obs.export_compliance",
  "payload": {
    "export_id": "01HZK9R7A2B4C8D6...",
    "tenant_id": "550e8400-...",
    "regulation": "EU AI Act",
    "row_count": 12450,
    "exporter_subject_id": "auditor-...",
    "chain_head_at_export": "a3f9c8d7e6b5a4f3...",
    "request_id": "compliance_export_..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Multi-signature manifests (operator + auditor counter-sign) — slice 5+.
- Time-stamped signatures (RFC 3161 TSA) — slice 5+.
- Hardware security module (HSM)-backed signing key — slice 5+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Private key unavailable | KMS error | Export fails 500 + sev-1 | Operator restores key |
| memory MMR query fails | memory error | Export fails | Self-resolves when memory up |
| Interrupted export | catch panic during streaming | Manifest carries `state: Incomplete` | Auditor re-runs |
| Public key rotation in flight | key version mismatch | Manifest uses key in effect AT export time | Key rotation respects "no break-in-flight" |
| Public key CDN unreachable | offline verifier --pubkey flag | Auditor uses local copy | Fallback by design |
| Tampered rows post-export | offline verifier SHA-256 check | FAIL | Auditor flags |
| Tampered manifest field post-export | offline verifier signature check | FAIL | Auditor flags |
| Wrong public key supplied | signature verify fails | FAIL | Auditor uses correct kid |
| Signing latency > 100ms | OTel histogram | sev-3 alarm | Investigate key load OR JCS perf |
| memory audit row emit fails | memory_writer error | Sev-1 (export happened but unaudited) | Operator investigates memory |
| Canonicalisation produces different bytes (RFC 8785 bug) | unit test asserts | PR blocked | Update serde-jcs version |
| Verifier binary panics on bad input | catch + clean error | exit 1 with reason | By design |
| QR code generation fails | catch | Render PDF without QR; sev-3 | Operator investigates |
| State enum not deserialising | strict serde | Verifier fails parse | Schema match |
| Public key file format mismatch | PEM parser | Verifier fails | Auditor uses correct format |
| Multiple key rotations during long-running export | key version pinned at export start | Manifest uses original key | By design |
| Auditor verifier on different OS/arch | static binary distribution | Works | Cross-compile |

---

## §11 — Notes

- The Ed25519 signing key is stored in the same KMS as memory's STH key. Rotation cadence aligns (quarterly).
- The `obs.export_compliance` memory row makes the export self-anchoring — perfect circularity for the auditor (the export is in the chain that the export covers).
- Public keys on CDN at `https://keys.cyberos.world/<kid>.pub` — no auth, no infra access needed for verification.
- Offline verifier binary is the primary trust mechanism. Auditors verify WITHOUT touching CyberOS infra; the verification is genuinely independent.
- RFC 8785 JCS for deterministic canonical JSON. Same data → same bytes → same hash → same signature. Without canonicalisation, two valid serialisations of the same data have different signatures.
- State: Incomplete is the loud-failure path. Silent partial exports would let auditors trust 50% of data thinking it's 100%.
- QR code on PDF cover page bridges paper-audit and digital-verification workflows.
- Quarterly key rotation: new `public_key_id` AND CDN serves both old + new during overlap. Manifests signed with old key continue to verify until the old key's CDN entry expires (typically 90 days post-rotation).
- Chain-head-at-export anchors the export to a specific memory state — auditor can verify "what was the chain at that moment?" for additional non-repudiation.

---

*End of FR-OBS-009. Status: draft (10/10 target).*
