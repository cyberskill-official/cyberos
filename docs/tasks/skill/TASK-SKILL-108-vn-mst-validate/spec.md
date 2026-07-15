---
id: TASK-SKILL-108
title: "vietnam-mst-validate@1 skill — Vietnamese Tax ID (MST) validation against General Department of Taxation (GDT) public registry"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: SKILL
priority: p0
status: done
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-SKILL-103, TASK-SKILL-104, TASK-SKILL-105, TASK-SKILL-109, TASK-SKILL-110, TASK-MEMORY-111, TASK-AI-016]
depends_on: [TASK-SKILL-104]
blocks: [TASK-SKILL-109, TASK-SKILL-110]

source_pages:
  - website/docs/skills/vietnam-mst-validate.html
  - website/docs/legal/vn-pdpl-compliance.html#mst
source_decisions:
  - DEC-210 (MST validation MUST be authoritative — local checksum + remote GDT lookup; both required)
  - DEC-211 (every validation emits a memory audit row for KYC/AML trail)
  - DEC-212 (cache GDT responses 24h; stale-cache acceptable in offline mode)
  - DEC-213 (PDPL 2025: MST is PII; redact in logs; never persist raw outside the audit row)

language: rust 1.81
service: cyberos/skills/vietnam-mst-validate/
new_files:
  - skills/vietnam-mst-validate/SKILL.md
  - skills/vietnam-mst-validate/main.rs
  - skills/vietnam-mst-validate/src/lib.rs
  - skills/vietnam-mst-validate/src/checksum.rs
  - skills/vietnam-mst-validate/src/gdt_client.rs
  - skills/vietnam-mst-validate/src/cache.rs
  - skills/vietnam-mst-validate/tests/checksum_test.rs
  - skills/vietnam-mst-validate/services/skill-broker/tests/integration.rs
modified_files:
  - cyberos/Cargo.toml                                  # workspace member
allowed_tools:
  - file_read: skills/vietnam-mst-validate/**
  - file_write: skills/vietnam-mst-validate/**
  - bash: cd skills/vietnam-mst-validate && cargo test
disallowed_tools:
  - call GDT public API with > 100 req/min (per official rate limit; will be rate-limited otherwise)
  - cache MST validation results > 24h (per DEC-212; data freshness matters for KYC)
  - log raw MST in plain text (per DEC-213; redact to last-4 + ****)

effort_hours: 7
subtasks:
  - "0.5h: SKILL.md — id=vietnam-mst-validate, version=1.0.0, allowed_tools=[MemoryEmit, HttpFetch], allowed_memory_scopes=memories/compliance/mst/**"
  - "1.0h: checksum.rs — 10-digit GDT checksum (weight vector [31,29,23,19,17,13,7,5,3,1]); 13-digit branch (prefix-3 + dash + 10-digit)"
  - "1.5h: gdt_client.rs — POST to https://gdtapi.gdt.gov.vn/Service.asmx; parse SOAP XML response; map to canonical struct"
  - "0.5h: cache.rs — sled-backed local cache; 24h TTL; key = MST, value = ValidationOutcome"
  - "0.5h: lib.rs — public API: `validate_mst(mst: &str) -> Result<ValidationOutcome, MstError>`"
  - "0.5h: main.rs — skill subprocess entrypoint; broker JSON-RPC parser; calls lib.rs"
  - "1.5h: checksum_test.rs — 50+ vector test (10-digit + 13-digit; valid + invalid; province prefixes)"
  - "1.0h: integration_test.rs — mock GDT server; verify XML parsing, cache hits, retry logic"
risk_if_skipped: "Without authoritative MST validation, customer-onboarded MSTs may be invalid (typo, expired, suspended). Hóa đơn (VAT invoice) emission against an invalid MST is a regulatory violation (Decree 123 Art. 9.4). Without the local checksum, every validation requires a network call → onboarding latency spikes 200-500ms. Without 24h cache, repeat validations during the same business day flood GDT with redundant requests → rate-limited. Without memory audit row, KYC compliance review has no trail (PDPL Art. 23 requires retention of identity-verification records 5 years)."
---

## §1 — Description (BCP-14 normative)

The `vietnam-mst-validate@1` skill **MUST** validate Vietnamese Tax IDs (MST) using a two-stage process: local checksum + remote GDT verification. The contract:

1. **MUST** accept input as a 10-digit or 13-digit MST string. 10-digit = company head office; 13-digit = company branch (`AAAAAAAAAA-BBB` format where AAAAAAAAAA is the parent 10-digit and BBB is the branch suffix).
2. **MUST** run local checksum FIRST per the official GDT algorithm:
    - 10-digit: `Sum = Σ digit[i] × weight[i]` where weights `= [31, 29, 23, 19, 17, 13, 7, 5, 3, 1]`. Check digit (last) = `10 - (Sum % 11) mod 10`, mod 10.
    - 13-digit: validate the parent 10-digit first (same algorithm); branch suffix (BBB) is informational, no checksum.
3. **MUST** reject with `MstError::ChecksumFailed` if local checksum doesn't match. Network call NOT attempted on checksum failure (saves the round-trip).
4. **MUST** call the GDT public API at `https://gdtapi.gdt.gov.vn/Service.asmx` with SOAP envelope (canonical method `ttin_NNT_DN`). Parse the XML response:
    - `valid: true` if response contains `<TrangThai>00</TrangThai>` (active status).
    - `name: <DiaChiNNT>` (registered tax name).
    - `address: <DiaChi>` (registered address).
    - `business_type: <NganhNghe>` (registered business sector).
    - `valid_from: <NgayCap>` (registration date).
    - `status: <TenTrangThai>` (human-readable status).
5. **MUST** treat the following GDT statuses as "valid for transactions":
    - `00` — đang hoạt động (active).
    - `04` — chờ đóng cửa (closing; warn but accept).
   All others → reject with `MstError::Inactive { status, status_text }`.
6. **MUST** cache validation outcomes in sled-backed local cache with 24h TTL. Cache key: MST string. Cache hit within TTL → no GDT call; cache miss OR expired → fresh call.
7. **MUST** support `force_refresh: true` flag to bypass cache (operator-controlled; e.g. when re-validating after a known status change).
8. **MUST** retry GDT calls on transient failure with exp backoff: 3 retries at 500ms, 2s, 8s. Total max wait ~10s. Permanent failure → `MstError::GdtUnreachable`.
9. **MUST** rate-limit GDT calls to 100 req/min globally (per GDT's published cap). Implemented via governor token bucket; over-limit returns `MstError::RateLimited` immediately (don't burn budget on already-rate-limited calls).
10. **MUST** emit memory audit row `compliance.mst_validated` on EVERY validation (cache hit or fresh) with payload `{mst_redacted, valid, status, business_name, validated_at_ns, cache_hit, gdt_round_trip_ms, trace_id}`. `mst_redacted` is `XXXXXX{last_4}` (last 4 digits visible per PDPL).
11. **MUST** log MSTs as redacted (`{first_2}*****{last_2}`) — never plain text.
12. **MUST** offline-mode fallback: if `CYBEROS_OFFLINE=true` OR network unreachable, return last cached outcome (regardless of TTL) with `stale: true` flag. Caller decides whether to accept stale data.
13. **MUST** emit OTel span `skill.vn_mst.validate` with attributes `mst_redacted`, `valid`, `cache_hit`, `gdt_round_trip_ms`, `duration_ms`.
14. **MUST** emit OTel metrics:
    - `skill_vn_mst_validations_total{outcome}` (counter; outcome ∈ valid | checksum_failed | inactive | gdt_unreachable | rate_limited | offline_stale).
    - `skill_vn_mst_gdt_round_trip_seconds` (histogram; only when cache_hit=false).
    - `skill_vn_mst_cache_hit_ratio` (gauge).

---

## §2 — Why this design (rationale for humans)

**Why two-stage validation (§1 #2 + #4)?** Local checksum catches 99% of typos at zero cost; GDT call catches the remaining 1% (validly-checksummed but registry-inactive). Skipping local would burn GDT quota on obvious typos; skipping GDT would miss the truly-invalid MSTs.

**Why cache 24h (§1 #6, DEC-212)?** Onboarding flows revalidate the same MST many times in a day (form submit + edit + retry). 24h cache catches same-day churn without staleness risk: MST status changes are rare, and task-INV (Invoice module) re-validates before each hóa đơn anyway.

**Why audit on cache hit too (§1 #10)?** KYC compliance needs the trail. The auditor's question is "did you check this MST before emitting hóa đơn?" — answer = "yes, validation row at <time> with status=<x>". Cache hit doesn't mean "we didn't check"; it means "we checked recently."

**Why `04` accepted (§1 #5)?** Status `04` is "winding down" — the company is still legally trading but in dissolution. Hóa đơn is legal during this period; rejecting it would block valid business. We warn the caller but accept.

**Why redact MST in logs (§1 #11, DEC-213)?** PDPL 2025 Art. 6 lists tax IDs as restricted personal data. Logs persist beyond the validation event; redacting (XXXXX1234) lets operators trace flows without exposing the full ID. The audit row (encrypted, retention-controlled) carries the redacted form too.

**Why governor rate limit (§1 #9)?** GDT publishes a 100-req/min cap. Exceeding triggers IP-level block (15 minutes). Better to fail fast (RateLimited) and let the caller queue or retry than to consume the quota and lock the platform.

---

## §3 — API contract

### SKILL.md

```markdown
---
id: vietnam-mst-validate
version: 1.0.0
description: Validate Vietnamese Tax ID (MST) via GDT public API + local checksum.
allowed_memory_scopes:
  - memories/compliance/mst/**
allowed_tools:
  - MemoryEmit
  - HttpFetch
sync_class: private    # MST data is PDPL-restricted; never sync
tenant_scope: any
effort_minutes: 1
tags: [vn, mst, compliance, kyc, gdt]
signature:
  algo: ed25519
  public_key_hex: "<release-populated>"
  signature_hex:  "<release-populated>"
x-allowed-domains:
  - gdtapi.gdt.gov.vn
---

# vietnam-mst-validate@1

Vietnamese Tax ID (MST) validation skill.

```rust
use cyberos_vn_mst_validate::{validate_mst, ValidateOptions};

let outcome = validate_mst("0312345678", ValidateOptions::default()).await?;
if outcome.valid {
    println!("MST belongs to: {} ({})", outcome.business_name, outcome.business_type);
}
```
```

### Rust API

```rust
// skills/vietnam-mst-validate/src/lib.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationOutcome {
    pub mst:           String,         // original (plain) — caller holds; redacted in logs
    pub valid:         bool,
    pub status:        String,         // "00", "04", etc.
    pub status_text:   String,         // "đang hoạt động"
    pub business_name: Option<String>,
    pub address:       Option<String>,
    pub business_type: Option<String>,
    pub valid_from:    Option<String>, // ISO date
    pub validated_at:  i64,            // unix ms
    pub cache_hit:     bool,
    pub stale:         bool,           // true when offline-fallback used
    pub trace_id:      String,
}

#[derive(Debug, thiserror::Error)]
pub enum MstError {
    #[error("MST must be 10 or 13 digits (got {0} chars)")]    InvalidLength(usize),
    #[error("MST checksum failed")]                            ChecksumFailed,
    #[error("MST inactive (status {status}: {status_text})")]  Inactive { status: String, status_text: String },
    #[error("GDT API unreachable after 3 retries")]            GdtUnreachable,
    #[error("rate limited; retry after Δ")]                    RateLimited,
    #[error("GDT response parse failed: {0}")]                 ParseError(String),
}

#[derive(Default)]
pub struct ValidateOptions {
    pub force_refresh: bool,
}

pub async fn validate_mst(mst: &str, opts: ValidateOptions) -> Result<ValidationOutcome, MstError> {
    let normalised = normalise_mst(mst)?;
    checksum::verify(&normalised)?;

    // Cache lookup (unless force_refresh)
    if !opts.force_refresh {
        if let Some(cached) = cache::get(&normalised).await {
            return Ok(ValidationOutcome { cache_hit: true, ..cached });
        }
    }
    // Offline mode → stale cache OR error
    if std::env::var("CYBEROS_OFFLINE").as_deref() == Ok("true") {
        return cache::get_stale(&normalised).await
            .map(|c| ValidationOutcome { stale: true, ..c })
            .ok_or(MstError::GdtUnreachable);
    }

    // Rate limit
    if !gdt_client::rate_limiter().check() { return Err(MstError::RateLimited); }

    // GDT call with retry
    let raw = gdt_client::fetch_with_retry(&normalised).await?;
    let outcome = gdt_client::parse_response(&raw, &normalised)?;

    // Cache the result
    cache::put(&normalised, &outcome).await;

    // Emit memory audit row
    emit_audit_row(&outcome).await;

    Ok(outcome)
}

fn normalise_mst(mst: &str) -> Result<String, MstError> {
    let cleaned: String = mst.chars().filter(|c| c.is_ascii_digit() || *c == '-').collect();
    let digits_only: String = cleaned.chars().filter(|c| c.is_ascii_digit()).collect();
    match digits_only.len() {
        10 => Ok(digits_only),
        13 => Ok(format!("{}-{}", &digits_only[..10], &digits_only[10..])),
        n  => Err(MstError::InvalidLength(n)),
    }
}
```

### Checksum

```rust
// skills/vietnam-mst-validate/src/checksum.rs
use crate::MstError;

const WEIGHTS: [u32; 10] = [31, 29, 23, 19, 17, 13, 7, 5, 3, 1];

pub fn verify(normalised: &str) -> Result<(), MstError> {
    // Strip branch suffix if present
    let parent = if normalised.contains('-') {
        normalised.split('-').next().unwrap()
    } else { normalised };
    if parent.len() != 10 { return Err(MstError::InvalidLength(parent.len())); }

    let digits: Vec<u32> = parent.chars().map(|c| c.to_digit(10).unwrap()).collect();
    let sum: u32 = digits.iter().zip(WEIGHTS.iter()).take(9).map(|(d, w)| d * w).sum();
    let expected = (10 - (sum % 11)) % 10;
    if digits[9] == expected { Ok(()) } else { Err(MstError::ChecksumFailed) }
}
```

### GDT client (SOAP)

```rust
// skills/vietnam-mst-validate/src/gdt_client.rs
use std::time::Duration;
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use std::sync::OnceLock;

const GDT_URL:  &str = "https://gdtapi.gdt.gov.vn/Service.asmx";
const RATE_LIMIT: NonZeroU32 = NonZeroU32::new(100).unwrap();

pub fn rate_limiter() -> &'static RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock> {
    static R: OnceLock<RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>> = OnceLock::new();
    R.get_or_init(|| RateLimiter::direct(Quota::per_minute(RATE_LIMIT)))
}

pub async fn fetch_with_retry(mst: &str) -> Result<String, crate::MstError> {
    let body = format!(r#"<?xml version="1.0" encoding="utf-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope">
  <soap:Body>
    <ttin_NNT_DN xmlns="http://tempuri.org/">
      <strMST>{mst}</strMST>
    </ttin_NNT_DN>
  </soap:Body>
</soap:Envelope>"#);

    let delays = [Duration::from_millis(500), Duration::from_secs(2), Duration::from_secs(8)];
    for (i, delay) in delays.iter().enumerate() {
        let start = std::time::Instant::now();
        let resp = reqwest::Client::new()
            .post(GDT_URL)
            .header("Content-Type", "application/soap+xml; charset=utf-8")
            .body(body.clone())
            .timeout(Duration::from_secs(5))
            .send()
            .await;
        match resp {
            Ok(r) if r.status().is_success() => {
                let txt = r.text().await.map_err(|e| crate::MstError::ParseError(e.to_string()))?;
                tracing::debug!(mst_redacted = redact(mst), round_trip_ms = start.elapsed().as_millis() as u64, "gdt fetch ok");
                return Ok(txt);
            }
            Ok(r) => {
                tracing::warn!(status = r.status().as_u16(), "gdt non-success; retrying");
                if i + 1 == delays.len() { return Err(crate::MstError::GdtUnreachable); }
            }
            Err(e) => {
                tracing::warn!(?e, "gdt error; retrying");
                if i + 1 == delays.len() { return Err(crate::MstError::GdtUnreachable); }
            }
        }
        tokio::time::sleep(*delay).await;
    }
    Err(crate::MstError::GdtUnreachable)
}

pub fn parse_response(xml: &str, mst: &str) -> Result<crate::ValidationOutcome, crate::MstError> {
    // SOAP XML; parse via quick-xml + serde
    let doc: GdtResponse = quick_xml::de::from_str(xml).map_err(|e| crate::MstError::ParseError(e.to_string()))?;
    let valid = matches!(doc.status.as_str(), "00" | "04");
    if !valid {
        return Err(crate::MstError::Inactive { status: doc.status.clone(), status_text: doc.status_text.clone() });
    }
    Ok(crate::ValidationOutcome {
        mst:           mst.into(),
        valid:         true,
        status_text:   doc.status_text,
        business_name: Some(doc.tax_name),
        address:       Some(doc.address),
        business_type: Some(doc.business_sector),
        valid_from:    Some(doc.registration_date),
        validated_at:  chrono::Utc::now().timestamp_millis(),
        cache_hit:     false,
        stale:         false,
        trace_id:      current_trace_id(),
    })
}

pub fn redact(mst: &str) -> String {
    let n = mst.len();
    if n < 4 { return "****".into(); }
    format!("{}{}", &mst[..2], "*".repeat(n - 4)) + &mst[n-2..]
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GdtResponse {
    #[serde(rename = "TrangThai")] status:           String,
    #[serde(rename = "TenTrangThai")] status_text:   String,
    #[serde(rename = "DiaChiNNT")] tax_name:         String,
    #[serde(rename = "DiaChi")] address:             String,
    #[serde(rename = "NganhNghe")] business_sector:  String,
    #[serde(rename = "NgayCap")] registration_date:  String,
}
```

---

## §4 — Acceptance criteria

1. **10-digit MST: valid checksum** — `0312345678` (real test vector) → checksum passes; GDT call proceeds.
2. **10-digit MST: invalid checksum** — `1234567890` → `Err(ChecksumFailed)`; no GDT call made.
3. **13-digit MST normalised** — `0312345678001` (no dash) → normalised to `0312345678-001`; parent checksum verified.
4. **Invalid length rejected** — `123456789` (9 digits) → `Err(InvalidLength(9))`.
5. **GDT response parsed: valid (status 00)** — mock GDT returns SOAP with `<TrangThai>00</TrangThai>` → `valid: true`; business name populated.
6. **GDT response parsed: closing (status 04)** — mock returns `04` → `valid: true` (per §1 #5); status_text set.
7. **GDT response parsed: inactive (status 03)** — mock returns `03` → `Err(Inactive { status: "03", ... })`.
8. **GDT timeout retried 3×** — mock returns 504 three times → SDK retries; 4th attempt unmocked succeeds; outcome ok.
9. **GDT unreachable after 3 retries** — mock always 504 → `Err(GdtUnreachable)` after ~10s.
10. **Rate limit triggers RateLimited error** — exhaust governor (101 calls in 60s) → next call immediate `Err(RateLimited)`.
11. **Cache hit within 24h** — call twice within TTL → second call cache_hit: true; only 1 GDT request made.
12. **Cache miss after TTL** — call once; advance time 25h; call again → cache miss; fresh GDT call.
13. **Force refresh bypasses cache** — call once; `ValidateOptions { force_refresh: true }` → cache_hit: false; fresh GDT call.
14. **Offline mode returns stale cache** — `CYBEROS_OFFLINE=true` → returns last cached outcome with `stale: true`.
15. **Offline + no cache → GdtUnreachable** — `CYBEROS_OFFLINE=true` + cache empty → `Err(GdtUnreachable)`.
16. **memory audit row emitted** — every successful validation → `compliance.mst_validated` row in memory; `mst_redacted` field redacts to `XX******1234` format.
17. **memory audit row on cache hit** — cache hits also emit audit row (with `cache_hit: true`) for KYC trail.
18. **Logs redact MST** — `tracing::info!(mst, ...)` in source → grep log file → no plain MST appears; redacted form (`03********78`) appears instead.
19. **OTel span emitted** — `skill.vn_mst.validate` span per call with `mst_redacted`, `valid`, `cache_hit`, `gdt_round_trip_ms` attrs.
20. **Metrics increment** — `skill_vn_mst_validations_total{outcome="valid"}` non-zero after happy calls; `outcome="checksum_failed"` non-zero after #2.
21. **SOAP XML parser robust to whitespace** — varied whitespace in real GDT responses → parser succeeds.
22. **broker enforces allowed_tools** — skill calls only MemoryEmit + HttpFetch; trying Bash within skill → broker denial (verified via fixture).
23. **domain enforcement** — HttpFetch to non-`gdtapi.gdt.gov.vn` → broker denial.

---

## §5 — Verification

```rust
// skills/vietnam-mst-validate/tests/checksum_test.rs
#[test]
fn valid_10_digit_passes() {
    assert!(checksum::verify("0312345678").is_ok());
}
#[test]
fn invalid_checksum_fails() {
    assert!(matches!(checksum::verify("1234567890"), Err(MstError::ChecksumFailed)));
}
#[test]
fn valid_13_digit_normalises() {
    let n = normalise_mst("0312345678-001").unwrap();
    assert!(checksum::verify(&n).is_ok());
}
#[test]
fn invalid_length_rejected() {
    assert!(matches!(normalise_mst("123"), Err(MstError::InvalidLength(_))));
}
// ... 46 more test vectors (positive + negative, real-world MSTs from public registry)
```

```rust
// skills/vietnam-mst-validate/services/skill-broker/tests/integration.rs
#[tokio::test]
async fn happy_path_against_mock_gdt() {
    let mock = MockGdt::start_with("0312345678", "00", "CYBERSKILL JSC").await;
    let outcome = validate_mst("0312345678", ValidateOptions::default()).await.unwrap();
    assert!(outcome.valid);
    assert_eq!(outcome.status, "00");
    assert_eq!(outcome.business_name.as_deref(), Some("CYBERSKILL JSC"));
}

#[tokio::test]
async fn cache_hit_skips_gdt() {
    let mock = MockGdt::start_with("0312345678", "00", "X").await;
    let _ = validate_mst("0312345678", ValidateOptions::default()).await.unwrap();
    let second = validate_mst("0312345678", ValidateOptions::default()).await.unwrap();
    assert!(second.cache_hit);
    assert_eq!(mock.call_count(), 1);
}

#[tokio::test]
async fn offline_returns_stale() {
    let mock = MockGdt::start_with("0312345678", "00", "X").await;
    let _ = validate_mst("0312345678", ValidateOptions::default()).await.unwrap();
    std::env::set_var("CYBEROS_OFFLINE", "true");
    let stale = validate_mst("0312345678", ValidateOptions::default()).await.unwrap();
    assert!(stale.stale);
    assert_eq!(stale.business_name.as_deref(), Some("X"));
}

#[tokio::test]
async fn rate_limit_triggers_error() {
    // Burn the governor budget
    for _ in 0..100 { let _ = rate_limiter().check(); }
    assert!(matches!(rate_limiter().check(), Err(_)));
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton.)

---

## §7 — Dependencies

- **TASK-SKILL-103** — frontmatter schema.
- **TASK-SKILL-104** — capability broker enforces `allowed_tools: [MemoryEmit, HttpFetch]` + domain allowlist.
- **TASK-SKILL-105** — memory-capture SDK (used internally for audit emit).
- **TASK-SKILL-109, 110** — VAT invoice + bank transfer skills depend on validated MST.
- **TASK-MEMORY-111** — PII detection; MST is a recognised PII tag (`VnMst`).
- **TASK-AI-016** — residency pinning; GDT API call MUST originate from VN region (sg-1 fallback acceptable for slice-3).

---

## §8 — Example payloads

### `compliance.mst_validated` audit row

```json
{
  "kind": "compliance.mst_validated",
  "payload": {
    "mst_redacted":     "03******78",
    "valid":            true,
    "status":           "00",
    "status_text":      "đang hoạt động",
    "business_name":    "CYBERSKILL JSC",
    "validated_at_ns":  1747407137483000000,
    "cache_hit":        false,
    "stale":            false,
    "gdt_round_trip_ms": 247,
    "trace_id":         "0af7651916cd43dd8448eb211c80319c"
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- MST-to-business-info enrichment (full registered shareholders, legal representative) — slice 4+; GDT public API doesn't expose; need different data source.
- Cross-reference with NBT (Ngân hàng Trung ương) for blacklist check — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Checksum failure | local check | `ChecksumFailed`; no GDT call | Caller fixes typo |
| GDT unreachable | network err / 5xx | retry 3×; final `GdtUnreachable` | Operator retries; uses offline cache |
| GDT returns inactive status | `00`/`04` check fails | `Inactive { status, text }` | Caller surfaces to user; may stop transaction |
| Rate limit exceeded | governor.check Err | `RateLimited` | Caller backs off |
| Cache corruption | sled error | Treat as miss; fresh GDT call | Cache eventually recovers |
| Offline mode + no cache | env check + cache miss | `GdtUnreachable` | Operator brings network OR accepts no validation |
| MST not in GDT registry | response has `<TrangThai>05</TrangThai>` ("not found") | `Inactive { status: "05", ... }` | Caller informs user |
| SOAP parse error | quick_xml Err | `ParseError(detail)`; metric increments | Operator investigates GDT format change |
| GDT timeout (5s default) | reqwest timeout | retried as transient | None |
| Stale cache served (offline) | flag in outcome | Caller decides | None |
| Concurrent same-MST validations | lib is async-safe | Either races to populate cache; second call sees cache hit | By design |
| Multi-byte UTF-8 in business name | parse handles | Stored correctly | None |
| GDT IP-blocked us | 403/blocked | Treated as transient; retry; final `GdtUnreachable` | Operator contacts GDT |
| Cache file size grows | sled compaction; capped at 100 MB | Old entries evicted | None |
| `force_refresh` racing with another caller | last writer wins | Slight inefficiency | None |
| MST normalisation strips letters | `normalise_mst` filters non-digit/dash | Invalid input flagged as InvalidLength | Caller passes correct format |
| Audit row write fails | MemoryEmit Err | Validation still succeeds; audit lost; sev-2 alarm | Operator restores memory |
| OTel span buffered then dropped | exporter unavailable | Logged WARN | Operator restores collector |
| Tracing inadvertently logs raw MST | `tracing::info!(mst, ...)` instead of redacted | PII leak | Author uses `redact(mst)` helper |
| Concurrent rate_limiter().check() race | governor handles | Either accept or reject; deterministic per call | None |

---

## §11 — Implementation notes

- `quick-xml` (with serde feature) is the canonical SOAP parser; `xmlrs` was rejected for being heavier with no benefit on small responses.
- The sled cache file lives at `~/.cyberos/cache/vn-mst.sled` (user-scope) or `/var/lib/cyberos/cache/vn-mst.sled` (system-scope); TASK-MEMORY-110 sweeper doesn't touch it.
- `governor` rate limiter is process-local. Multiple cyberos instances on the same host could exceed global GDT rate; slice-3+ may add Redis-backed shared limit.
- The `redact()` function is used everywhere except the audit row and the returned `ValidationOutcome.mst`; both intentional (audit needs the full thing; caller holds the original).
- SOAP namespace `xmlns="http://tempuri.org/"` is the actual GDT convention (legacy ASP.NET service); not a typo.
- The 5s per-request timeout is empirical: 95th percentile GDT response is ~600ms; 5s is generous. Hard exp backoff handles longer tails.
- `valid_from` is parsed as ISO date but returned as String — different callers want different date types. Caller converts.
- The 24h cache TTL covers both `valid` and `inactive` outcomes; caller checks `outcome.valid` before transacting.
- Rate-limit recovery: governor restores tokens at 100/min linearly; after 1 minute of zero usage, full burst available.

---

*End of TASK-SKILL-108.*
