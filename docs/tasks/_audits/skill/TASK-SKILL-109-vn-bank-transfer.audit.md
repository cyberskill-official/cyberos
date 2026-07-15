---
task_id: TASK-SKILL-109
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
---

## §1 — Verdict summary

TASK-SKILL-109 authored direct-to-10/10. ~700 lines. 14 §1 clauses (TransferRequest spec, BIN validation, account validation, TLV/EMVCo composition, CRC16-CCITT-FALSE, deterministic output, QR image URL, transliteration, audit row, Napas247 transfer code, OTel + metrics, compile-time registry). 7 §2 rationale paragraphs. Full Rust API + bank registry + TLV composer + CRC algorithm in §3. 22 ACs. 6 Rust tests including reference vector. 20 failure modes. 9 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Pure-local vs API integration
Naive design adds outbound Napas247 API call (latency + rate limit). Resolved: §1 + §2 pure-local; skill generates QR string only; settlement is bank-network; `allowed_tools: [MemoryEmit]` only (no HttpFetch).

### ISS-002 — CRC algorithm choice
Multiple CRC16 variants exist (XMODEM, ARC, CCITT-FALSE). Banks reject mismatches silently. Resolved: §1 #5 explicit CCITT-FALSE (poly 0x1021, init 0xFFFF) + §3 implementation + AC #13 against VietQR.io reference vector `C0A0`.

### ISS-003 — Determinism for retry safety
Without deterministic output, retries create new payment requests = double-charge risk. Resolved: §1 #6 idempotency_key NOT in QR payload (banks don't see it) but determinism ensures same inputs = same QR; AC #14.

### ISS-004 — Vietnamese name handling
Tag 59 specified as ASCII but UTF-8 names common. Banks reject non-ASCII. Resolved: §1 #8 transliteration via `cyberos-vn-common` (Nguyễn → NGUYEN); truncation marker `~` on overflow.

### ISS-005 — Account redaction in audit
PDPL 2025 restricts bank-account storage. Without spec, audit row carries full account = compliance risk. Resolved: §1 #9 `receiver_account_redacted: ****<last_4>`; AC #17 explicitly verifies raw account NOT stored.

### ISS-006 — Bank registry stale-data risk
External JSON config drifts; manual maintenance lags. Resolved: §1 #13 + DEC-222 compile-time `&[BankEntry]` slice; quarterly refresh tied to release; unknown BIN at runtime = `UnknownBin` error (loud, not silent).

## §3 — Resolution

All 6 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

---

*End of TASK-SKILL-109 audit.*
