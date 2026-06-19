---
id: FR-SKILL-109
title: "vietnam-bank-transfer@1 skill — VietQR + Napas247 transfer-code generator with bank-prefix validation, memory audit, and per-transfer idempotency"
module: SKILL
priority: MUST
status: done
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_frs: [FR-SKILL-103, FR-SKILL-104, FR-SKILL-105, FR-SKILL-108, FR-SKILL-110, FR-MEMORY-111]
depends_on: [FR-SKILL-104, FR-SKILL-108]
blocks: [FR-SKILL-110]

source_pages:
  - website/docs/skills/vietnam-bank-transfer.html
  - website/docs/legal/vn-payments.html
source_decisions:
  - DEC-220 (VietQR is the canonical inter-bank QR format; Napas247 backbone)
  - DEC-221 (every QR generation MUST emit memory audit row for reconciliation)
  - DEC-222 (bank-prefix table maintained in skill; quarterly refresh via NAPAS public registry)
  - DEC-223 (idempotency key required for repeat-safe QR generation per transaction)

language: rust 1.81
service: cyberos/skills/vietnam-bank-transfer/
new_files:
  - skills/vietnam-bank-transfer/SKILL.md
  - skills/vietnam-bank-transfer/main.rs
  - skills/vietnam-bank-transfer/src/lib.rs
  - skills/vietnam-bank-transfer/src/vietqr.rs
  - skills/vietnam-bank-transfer/src/banks.rs
  - skills/vietnam-bank-transfer/src/crc16.rs
  - skills/vietnam-bank-transfer/tests/vietqr_test.rs
  - skills/vietnam-bank-transfer/tests/banks_test.rs
allowed_tools:
  - file_read: skills/vietnam-bank-transfer/**
  - file_write: skills/vietnam-bank-transfer/**
  - bash: cd skills/vietnam-bank-transfer && cargo test
disallowed_tools:
  - call any external payment API (per §1 — skill is offline; only generates the QR string, doesn't execute transfer)
  - skip CRC16-CCITT-FALSE on QR payload (per VietQR spec; banks reject malformed QR)
  - cache QR strings (each transaction is unique; caching causes wrong-amount payments)

effort_hours: 7
sub_tasks:
  - "0.5h: SKILL.md frontmatter (allowed_tools=[MemoryEmit]; no HttpFetch — skill is pure-local)"
  - "0.5h: Cargo.toml + main.rs (broker subprocess entry)"
  - "1.0h: banks.rs — NAPAS bank-bin registry (40+ banks: 970403=Sacombank, 970415=Vietinbank, 970418=BIDV, 970422=MBBank, 970432=VPBank, ...)"
  - "1.5h: vietqr.rs — TLV (Tag-Length-Value) encoding per EMVCo Merchant QR spec; payload composition (00–63 fields)"
  - "0.5h: crc16.rs — CRC16-CCITT-FALSE (poly 0x1021, init 0xFFFF) for the trailing 4-hex checksum"
  - "0.5h: lib.rs — public API: `generate_vietqr(req: TransferRequest) -> Result<VietQrString, BankError>`"
  - "0.5h: Idempotency key support (caller-provided UUID; same key + same body = same QR string deterministically)"
  - "1.0h: vietqr_test.rs — 20+ fixture transfers (cross-validated against Sacombank/Techcombank/VietQR.io test vectors)"
  - "0.5h: banks_test.rs — bank-prefix lookup + invalid-bin rejection"
  - "0.5h: memory audit row emission (vn.qr_generated kind with amount, receiver_bank, idempotency_key)"
risk_if_skipped: "Without authoritative VietQR generation, every payment-collecting feature (FR-INV invoice, FR-CRM deal-won, FR-CHAT escrow) reinvents the wheel. Without CRC16 checksum, generated QR codes are silently rejected by banks (banks compute CRC themselves; mismatch = no transfer). Without bank-bin validation, transfers go to non-existent banks (typed BIN like 970999 = no such bank; user sees 'transfer pending' forever). Without idempotency, retry-on-error duplicates payment requests; double-charged customers churn. Without memory audit, reconciliation (matching incoming bank statement entry to outgoing QR) is impossible at scale."
---

## §1 — Description (BCP-14 normative)

The `vietnam-bank-transfer@1` skill **MUST** generate VietQR strings conforming to NAPAS's EMVCo-based Merchant Presented Mode (MPM) specification. The skill is pure-local — it generates the QR payload string; banks/customers scan it; settlement happens out-of-band via Napas247. The contract:

1. **MUST** accept a `TransferRequest` with: `receiver_account` (6–18 digits), `receiver_bank_bin` (6-digit NAPAS BIN), `receiver_name` (≤ 25 ASCII chars; non-ASCII transliterated), `amount` (i64 VND, ≥ 0 — 0 = "amount unspecified, customer enters"), `memo` (≤ 25 ASCII chars), `idempotency_key` (UUID).
2. **MUST** validate `receiver_bank_bin` against the embedded NAPAS bank registry (40+ Vietnamese banks). Unknown BIN → `BankError::UnknownBin(<bin>)`.
3. **MUST** validate `receiver_account` length (6–18 digits per NAPAS standard); other characters rejected.
4. **MUST** compose the EMVCo MPM payload per VietQR spec:
    - Tag `00` — Payload Format Indicator: `01`.
    - Tag `01` — Point of Initiation: `12` (dynamic) if `amount > 0`, else `11` (static).
    - Tag `38` — Merchant Account Information (nested TLV):
      - `00` GUID: `A000000727`.
      - `01` Beneficiary Organization: nested TLV with `00`=BIN, `01`=account.
      - `02` Service Code: `QRIBFTTA` (account-based transfer).
    - Tag `52` — Merchant Category Code: `0000` (general).
    - Tag `53` — Currency: `704` (VND, ISO 4217).
    - Tag `54` — Transaction Amount (if specified, no decimals — VND has none).
    - Tag `58` — Country Code: `VN`.
    - Tag `59` — Merchant Name: receiver_name (uppercased, ASCII).
    - Tag `60` — Merchant City: `HOCHIMINHCITY` (default; configurable).
    - Tag `62` — Additional Data (nested): `08` = memo (if provided).
    - Tag `63` — CRC16-CCITT-FALSE checksum of the entire prior payload (4 uppercase hex chars).
5. **MUST** compute CRC16-CCITT-FALSE (polynomial `0x1021`, init `0xFFFF`, no reflection, no final XOR). The CRC is computed over all bytes from the start of the payload through the `6304` (tag 63 + length 04) literal. The 4-hex-char checksum is appended.
6. **MUST** be deterministic — same `TransferRequest` (including same idempotency_key) produces byte-identical VietQR string. Idempotency_key is ignored in the QR payload itself (banks don't see it); it's only used in the memory audit row + dedup cache.
7. **MUST** support short-form output as well: `generate_vietqr_image_url(req)` returns a URL pointing to a free QR rendering service (e.g. `https://img.vietqr.io/image/<bin>-<account>-<style>.png?amount=<n>&addInfo=<memo>&accountName=<name>`) for callers who want a PNG directly instead of rendering the QR themselves.
8. **MUST** transliterate non-ASCII receiver_name + memo per Vietnamese conventions:
    - `Nguyễn Văn A` → `NGUYEN VAN A`.
    - `Trịnh Thái Anh` → `TRINH THAI ANH`.
    - Truncate to 25 chars after transliteration; if truncation occurs, append `~` indicator.
9. **MUST** emit memory audit row `vn.qr_generated` per generation with payload `{idempotency_key, receiver_bank_bin, receiver_bank_name, receiver_account_redacted, amount_vnd, memo_hash, qr_string_hash, generated_at_ns, trace_id}`. `receiver_account_redacted` is `****<last_4>`; full account NOT stored (PDPL).
10. **MUST** support `generate_napas247_transfer_code(req)` as an alternative that produces a 16-char human-typed transfer code (for callers without a camera; mostly legacy). Code format: `<bin:3><account_last_8:8><amount_compressed:5>` with check digit; algorithm per NAPAS Doc 24.
11. **MUST** emit OTel span `skill.vn_bank_transfer.generate` with attributes `bank_bin`, `bank_name`, `has_amount`, `qr_length_bytes`, `duration_ms`.
12. **MUST** emit OTel metrics:
    - `skill_vn_bank_transfer_generations_total{bank, has_amount}` (counter; bank is short name).
    - `skill_vn_bank_transfer_generation_duration_seconds` (histogram; expected p99 ≤ 5ms — pure-local).
13. **MUST** maintain the bank registry in `src/banks.rs` as a compile-time `&[BankEntry]` slice; quarterly refresh via NAPAS public registry. Adding a bank = code change + PR + new release.
14. **SHOULD** provide a CLI `cyberos-bank-transfer generate --bin 970422 --account 12345678 --name "X Y" --amount 1500000` for ad-hoc operator use.

---

## §2 — Why this design (rationale for humans)

**Why pure-local (§1 #1)?** VietQR is a presentation format, not a payment API. The customer scans the QR with their bank app; their app calls Napas247; settlement happens in the bank network. Cyberos's role is generating the correct QR string. No outbound network call means: zero latency, zero external dependency, 100% offline-capable, no rate limit.

**Why TLV encoding (§1 #4)?** EMVCo's MPM spec uses Tag-Length-Value framing. Banks parse it deterministically. Free-form string generation drifts; TLV is the universal standard across VN/SG/TH/ID/PH QR payments.

**Why CRC16-CCITT-FALSE (§1 #5)?** EMVCo specifies this exact variant. Banks compute CRC themselves on receipt and reject mismatches silently — the user just sees "scan failed." Getting the CRC algorithm wrong is the #1 cause of "VietQR doesn't work" debug threads. We commit the algorithm to a tested, documented function.

**Why deterministic output (§1 #6)?** Idempotency: caller retries a generation with the same inputs → gets the same QR string → bank-side dedup recognises it → no double-charge. Without determinism, retry creates a "new" payment request that banks may treat as a separate transaction.

**Why transliteration (§1 #8)?** EMVCo allows UTF-8 in tag 59 but VN banks historically reject non-ASCII (legacy POS systems). Forcing ASCII at our boundary avoids this class of failure. `~` suffix on truncation tells the operator "your name was too long and got cut."

**Why account redacted in audit (§1 #9)?** PDPL 2025 lists bank accounts as restricted personal data. The full account belongs to the receiver, not the operator. Storing only last-4 + bank_bin gives enough info for reconciliation without exposing the full account.

**Why Napas247 transfer code option (§1 #10)?** Some users prefer typing a code into their banking app (no camera, accessibility). Napas247 codes are the official VN inter-bank transfer format. Generating both gives flexibility.

**Why compile-time bank registry (§1 #13)?** External-config registries drift: ops updates manually, code keeps stale list. Compile-time = atomic update with code release. Quarterly cadence matches NAPAS's BIN-registry update frequency.

---

## §3 — API contract

### Public API

```rust
// skills/vietnam-bank-transfer/src/lib.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransferRequest {
    pub receiver_bank_bin: String,    // "970422"
    pub receiver_account:  String,    // "1234567890"
    pub receiver_name:     String,    // "NGUYEN VAN A" or "Nguyễn Văn A" (will be transliterated)
    pub amount:            i64,       // VND; 0 = unspecified
    pub memo:              String,    // ≤ 25 ASCII chars
    pub idempotency_key:   Uuid,
    #[serde(default)]
    pub merchant_city:     Option<String>,   // default "HOCHIMINHCITY"
}

#[derive(Clone, Debug, Serialize)]
pub struct GenerateOutcome {
    pub qr_string:        String,           // The actual VietQR EMVCo MPM payload
    pub qr_image_url:     String,           // CDN URL for PNG rendering
    pub bank_name:        String,           // "VPBank"
    pub receiver_account_redacted: String,  // "****7890"
    pub qr_length_bytes:  usize,
    pub trace_id:         String,
}

#[derive(Debug, thiserror::Error)]
pub enum BankError {
    #[error("unknown bank BIN: {0}")]                              UnknownBin(String),
    #[error("receiver_account length must be 6..=18 digits (got {0})")] InvalidAccountLength(usize),
    #[error("receiver_account contains non-digit characters")]     InvalidAccountChars,
    #[error("amount must be ≥ 0 (got {0})")]                       NegativeAmount(i64),
    #[error("memo exceeds 25 chars after transliteration ({0})")]  MemoTooLong(usize),
    #[error("receiver_name empty after transliteration")]          NameEmpty,
}

pub fn generate_vietqr(req: TransferRequest) -> Result<GenerateOutcome, BankError> {
    let bank = banks::lookup(&req.receiver_bank_bin)
        .ok_or_else(|| BankError::UnknownBin(req.receiver_bank_bin.clone()))?;
    validate_account(&req.receiver_account)?;
    if req.amount < 0 { return Err(BankError::NegativeAmount(req.amount)); }

    let name_translit  = transliterate(&req.receiver_name);
    if name_translit.is_empty() { return Err(BankError::NameEmpty); }
    let memo_translit  = transliterate(&req.memo);
    if memo_translit.chars().count() > 25 { return Err(BankError::MemoTooLong(memo_translit.chars().count())); }
    let name_truncated = truncate25(&name_translit);

    let payload = vietqr::compose(
        &req.receiver_bank_bin,
        &req.receiver_account,
        req.amount,
        &name_truncated,
        &memo_translit,
        req.merchant_city.as_deref().unwrap_or("HOCHIMINHCITY"),
    );
    let crc = crc16::ccitt_false(format!("{payload}6304").as_bytes());
    let qr_string = format!("{payload}6304{:04X}", crc);
    let qr_image_url = format!(
        "https://img.vietqr.io/image/{}-{}-print.png?amount={}&addInfo={}&accountName={}",
        bank.short, req.receiver_account, req.amount,
        urlencoding::encode(&memo_translit),
        urlencoding::encode(&name_truncated),
    );
    let redacted = redact_account(&req.receiver_account);
    let outcome = GenerateOutcome {
        qr_length_bytes: qr_string.len(),
        qr_string,
        qr_image_url,
        bank_name: bank.short.into(),
        receiver_account_redacted: redacted,
        trace_id: current_trace_id(),
    };
    emit_audit_row(&req, &outcome);
    Ok(outcome)
}
```

### Bank registry

```rust
// skills/vietnam-bank-transfer/src/banks.rs
pub struct BankEntry {
    pub bin:       &'static str,
    pub short:     &'static str,
    pub full_name: &'static str,
}

pub const REGISTRY: &[BankEntry] = &[
    BankEntry { bin: "970403", short: "Sacombank",    full_name: "Sài Gòn Thương Tín" },
    BankEntry { bin: "970415", short: "Vietinbank",   full_name: "VietinBank" },
    BankEntry { bin: "970418", short: "BIDV",         full_name: "Đầu Tư & Phát Triển VN" },
    BankEntry { bin: "970422", short: "MBBank",       full_name: "Quân Đội" },
    BankEntry { bin: "970432", short: "VPBank",       full_name: "VPBank" },
    BankEntry { bin: "970436", short: "Vietcombank",  full_name: "Ngoại Thương VN" },
    BankEntry { bin: "970441", short: "Techcombank",  full_name: "Kỹ Thương VN" },
    BankEntry { bin: "970443", short: "SHB",          full_name: "Sài Gòn - Hà Nội" },
    BankEntry { bin: "970452", short: "TPBank",       full_name: "Tiên Phong" },
    BankEntry { bin: "970454", short: "Eximbank",     full_name: "Xuất Nhập Khẩu" },
    BankEntry { bin: "970465", short: "ACB",          full_name: "Á Châu" },
    // ... 30+ more entries
];

pub fn lookup(bin: &str) -> Option<&'static BankEntry> {
    REGISTRY.iter().find(|e| e.bin == bin)
}
```

### TLV composer

```rust
// skills/vietnam-bank-transfer/src/vietqr.rs
pub fn compose(
    bank_bin: &str,
    account:  &str,
    amount:   i64,
    name:     &str,
    memo:     &str,
    city:     &str,
) -> String {
    // tag NN + length LL + value V
    fn tlv(tag: &str, value: &str) -> String {
        format!("{tag}{:02}{value}", value.len())
    }
    let merchant_account_info = format!(
        "{}{}",
        tlv("00", "A000000727"),
        format!("01{:02}{}",
            (tlv("00", bank_bin).len() + tlv("01", account).len()),
            format!("{}{}", tlv("00", bank_bin), tlv("01", account))),
    );
    let mut payload = String::new();
    payload.push_str(&tlv("00", "01"));                              // Payload Format Indicator
    payload.push_str(&tlv("01", if amount > 0 { "12" } else { "11" })); // Point of Initiation
    payload.push_str(&format!("38{:02}{}{}",
        merchant_account_info.len() + tlv("02", "QRIBFTTA").len(),
        merchant_account_info,
        tlv("02", "QRIBFTTA"),
    ));
    payload.push_str(&tlv("52", "0000"));                            // Merchant Category Code
    payload.push_str(&tlv("53", "704"));                              // Currency = VND
    if amount > 0 { payload.push_str(&tlv("54", &amount.to_string())); }
    payload.push_str(&tlv("58", "VN"));                               // Country
    payload.push_str(&tlv("59", name));                                // Merchant Name
    payload.push_str(&tlv("60", city));                                // Merchant City
    if !memo.is_empty() {
        let memo_block = tlv("08", memo);
        payload.push_str(&format!("62{:02}{}", memo_block.len(), memo_block));
    }
    payload
}
```

### CRC

```rust
// skills/vietnam-bank-transfer/src/crc16.rs
pub fn ccitt_false(bytes: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in bytes {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}
```

---

## §4 — Acceptance criteria

1. **VPBank transfer (970422) with amount → valid QR** — request with bin=970422, account=12345678, amount=150000 → outcome.qr_string starts with `0002011...`; CRC verified.
2. **Sacombank transfer (970403) → valid QR** — verify bank_name in outcome = "Sacombank".
3. **Unknown BIN rejected** — bin="970999" → `Err(UnknownBin("970999"))`.
4. **Account too short rejected** — account="12345" (5 chars) → `Err(InvalidAccountLength(5))`.
5. **Account too long rejected** — account = 19 digits → `Err(InvalidAccountLength(19))`.
6. **Non-digit account rejected** — account="12345abc" → `Err(InvalidAccountChars)`.
7. **Negative amount rejected** — amount=-100 → `Err(NegativeAmount(-100))`.
8. **Amount=0 produces static QR** — Tag 01 = "11" in output (customer fills amount).
9. **Amount > 0 produces dynamic QR** — Tag 01 = "12".
10. **Vietnamese name transliterated** — `"Nguyễn Văn A"` → output contains `NGUYEN VAN A`.
11. **Long name truncated with marker** — 30-char ASCII name → first 24 chars + `~`.
12. **Memo too long rejected** — 30-char ASCII memo → `Err(MemoTooLong(30))`.
13. **CRC matches reference vector** — known-good fixture from VietQR.io: `00020101021238540010A000000727012400069704220110123456780208QRIBFTTA53037045802VN62080804test6304XXXX` → CRC matches.
14. **Deterministic output** — same request twice → byte-identical qr_string.
15. **Idempotency key in memory row but not QR** — qr_string MUST NOT contain idempotency_key; audit row payload MUST contain it.
16. **memory audit row emitted** — `vn.qr_generated` row appears in memory with payload schema per §1 #9.
17. **receiver_account redacted in audit** — payload `receiver_account_redacted` = "****7890" for account `1234567890`.
18. **qr_image_url constructed** — output contains `https://img.vietqr.io/image/VPBank-1234567890-...`.
19. **OTel span emitted** — span `skill.vn_bank_transfer.generate` with attrs.
20. **Pure-local: no HttpFetch** — broker enforcement: skill cannot call HttpFetch (frontmatter denies); verified via fixture invocation.
21. **Latency < 5ms** — 1000-call benchmark: p99 < 5ms.
22. **Napas247 transfer code generator works** — generate_napas247_transfer_code(req) → 16-char alphanumeric code with valid check digit.

---

## §5 — Verification

```rust
// skills/vietnam-bank-transfer/tests/vietqr_test.rs

#[test]
fn vpbank_dynamic_qr_generates() {
    let req = TransferRequest {
        receiver_bank_bin: "970422".into(),
        receiver_account:  "12345678".into(),
        receiver_name:     "NGUYEN VAN A".into(),
        amount:            150_000,
        memo:              "test".into(),
        idempotency_key:   Uuid::new_v4(),
        merchant_city:     None,
    };
    let out = generate_vietqr(req).unwrap();
    assert!(out.qr_string.starts_with("000201"));
    assert!(out.qr_string.contains("970422"));
    assert!(out.qr_string.contains("5406150000"));   // amount tag
    assert_eq!(out.bank_name, "MBBank");
    // CRC verification: extract last 4 chars; recompute
    let payload = &out.qr_string[..out.qr_string.len()-4];
    let claimed = u16::from_str_radix(&out.qr_string[out.qr_string.len()-4..], 16).unwrap();
    let expected = crc16::ccitt_false(format!("{payload}").as_bytes());
    assert_eq!(claimed, expected);
}

#[test]
fn vietnamese_name_transliterated() {
    let req = test_request_with_name("Nguyễn Văn A");
    let out = generate_vietqr(req).unwrap();
    assert!(out.qr_string.contains("NGUYEN VAN A"));
}

#[test]
fn unknown_bin_rejected() {
    let req = test_request_with_bin("970999");
    assert!(matches!(generate_vietqr(req), Err(BankError::UnknownBin(_))));
}

#[test]
fn deterministic_output() {
    let req = test_request();
    let a = generate_vietqr(req.clone()).unwrap();
    let b = generate_vietqr(req).unwrap();
    assert_eq!(a.qr_string, b.qr_string);
}

#[test]
fn crc_against_reference_vector() {
    // From VietQR.io spec, Appendix B
    let payload = "00020101021238540010A000000727012400069704220110123456780208QRIBFTTA53037045802VN62080804test6304";
    let crc = crc16::ccitt_false(payload.as_bytes());
    assert_eq!(format!("{:04X}", crc), "C0A0");   // expected per spec
}

#[test]
fn audit_row_emitted() {
    let req = test_request();
    let _ = generate_vietqr(req.clone()).unwrap();
    let row = memory_test_helper::latest("vn.qr_generated");
    assert_eq!(row["payload"]["receiver_bank_bin"], "970422");
    assert_eq!(row["payload"]["receiver_account_redacted"], "****5678");
    assert!(!row["payload"].as_object().unwrap().contains_key("receiver_account"));  // raw account NOT stored
    assert!(row["payload"].as_object().unwrap().contains_key("idempotency_key"));
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton.)

---

## §7 — Dependencies

- **FR-SKILL-103** — frontmatter schema.
- **FR-SKILL-104** — broker enforces `allowed_tools: [MemoryEmit]`; verifies skill cannot call HttpFetch.
- **FR-SKILL-105** — memory-capture SDK used internally for audit emit.
- **FR-SKILL-108** — vietnam-mst-validate is a sibling VN-pack skill; both depend on shared `transliterate()` + `redact()` helpers (in `cyberos-vn-common` crate).
- **FR-SKILL-110** — vietnam-vat-invoice depends on this skill for embedding QR in hóa đơn.
- **FR-MEMORY-111** — PII detection ruleset includes `VnBankAccount` tag.

---

## §8 — Example payloads

### Generated VietQR string

```text
00020101021238540010A000000727012400069704220110123456780208QRIBFTTA53037045802VN62080804test6304C0A0
```

### `vn.qr_generated` audit row

```json
{
  "kind": "vn.qr_generated",
  "payload": {
    "idempotency_key":           "0e3b1a2c-4f5d-6789-abcd-ef0123456789",
    "receiver_bank_bin":         "970422",
    "receiver_bank_name":        "MBBank",
    "receiver_account_redacted": "****5678",
    "amount_vnd":                150000,
    "memo_hash":                 "9b0e8c5...",
    "qr_string_hash":            "ab12cd...",
    "generated_at_ns":           1747407137483000000,
    "trace_id":                  "0af7651916cd43dd8448eb211c80319c"
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- VietQR v2 spec adoption (when NAPAS announces) — slice 4+.
- Multi-currency support (USD on Vietcombank) — slice 4+; rare.
- ACL on `vn.qr_generated` rows (only finance team sees full memo) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Unknown BIN | registry lookup miss | `UnknownBin`; no audit emit | Operator verifies BIN against NAPAS registry |
| Account length out of bounds | length check | `InvalidAccountLength` | Caller fixes |
| Account has letters | digit check | `InvalidAccountChars` | Caller fixes |
| Negative amount | arithmetic check | `NegativeAmount` | Caller fixes |
| Memo too long after translit | char count | `MemoTooLong` | Caller shortens |
| Name empty after translit | check | `NameEmpty` | Caller fixes |
| CRC algorithm bug | unit test catches | CI blocked | Author fixes |
| TLV length-prefix bug | unit test against fixtures | CI blocked | Author fixes |
| Bank registry stale (new bank added by NAPAS) | unknown BIN at runtime | `UnknownBin` | Operator opens PR to add bank entry |
| Caller passes wrong country | n/a — VN-pack is VN-only | n/a | n/a |
| 13-bit VND amount overflows i64 | impossible (i64 covers all VND amounts) | n/a | n/a |
| Amount has decimals | i64 type rejects | type error | Caller passes integer VND |
| QR image URL service down | qr_image_url still returned; CDN unavailable | Caller renders locally via qr_string | Operator switches CDN |
| Audit row write fails | MemoryEmit Err | QR still returned to caller; audit lost; sev-2 alarm | Operator restores memory |
| Two callers same idempotency_key + different bodies | deterministic but diff content → diff qr_string | Both succeed; both emit audit rows | By design (no cross-caller dedup) |
| Unicode normalisation in receiver_name | NFC normalised before translit | Consistent output | None |
| Account starts with 0 (legacy banks) | digit check passes | QR includes leading 0 | Correct |
| Memo contains emoji | filtered to ASCII; emoji dropped | qr_string has emoji-free memo | By design |
| BIN has wrong length (5 or 7) | length check | `UnknownBin` (not in registry) | Operator verifies |
| Bank name in audit row drifts from registry | registry is single source | Always matches | None |

---

## §11 — Implementation notes

- The bank registry is hand-curated from NAPAS's public BIN list (last refresh: 2026-04). Quarterly refresh is owned by FR-SKILL-109's release captain.
- The transliteration map is shared with FR-SKILL-108 + FR-SKILL-110 via `cyberos-vn-common` crate (created in slice-3 release).
- `urlencoding::encode` (instead of percent-encoding) is used for qr_image_url path components per VietQR.io's API conventions.
- The CRC test vector in §5 (`C0A0`) is from VietQR.io spec Appendix B; if the spec updates, this vector updates too.
- Pure-local: no `HttpFetch` in `allowed_tools`. The qr_image_url is a STRING returned to caller; the skill itself doesn't fetch.
- Determinism is critical: callers retry on failure with the same idempotency_key; same key + same body MUST produce same qr_string so the bank-side dedup works.
- `i64` amount supports up to ~9.2 quintillion VND — comfortably more than any real-world transaction.
- The Napas247 transfer code generator (§1 #10) is implemented but unused by most callers; deferred docs.
- The skill does NOT call any external API at runtime; all data is compile-time (registry) or caller-provided (request). This is the gold-standard offline-capable skill design.

---

*End of FR-SKILL-109.*
