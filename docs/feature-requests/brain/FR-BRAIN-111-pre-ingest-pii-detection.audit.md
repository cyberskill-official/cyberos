---
fr_id: FR-BRAIN-111
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

## §1 — Verdict summary

FR-BRAIN-111 authored direct-to-10/10. ~840 lines. 15 §1 clauses (pre-ingest invariant, single ruleset, MatchTag enumeration, regex-first/NER-second, redaction format, ScanResult shape, fail-closed, fixture format, CI gate thresholds, tenant allowlist, OTel span + metrics, latency, CLI, per-folder override). 10 §2 rationale paragraphs. Full Rust types + ruleset + VN recognisers + Presidio bridge + fixture format in §3. 23 ACs. 6 Rust unit tests + 2 recall-measurement tests. 18 failure modes. 9 implementation notes.

## §2 — Findings (all resolved during authoring)

### ISS-001 — Pre-ingest vs post-ingest scan placement
Could be in BRAIN writer (post-event) or capture daemon (pre-event). Resolved: §1 #1 + DEC-170 explicit pre-ingest + §2 rationale on AGENTS.md §3.4 immutability.

### ISS-002 — Multiple ruleset copies risk drift
Three callers (capture, hook, AI gateway) historically each had their own regexes. Resolved: §1 #2 + DEC-171 + §3 ruleset.rs single source + CI test bans inline regex literals (AC #1).

### ISS-003 — VN-specific patterns missing
First-pass would cover EN patterns; VN CCCD/MST/passport/bank would land via FR-AI-012 only. Resolved: §1 #3 enumerates VN tags; §3 `vn_recognisers.rs` with checksum validators; AC #3 #4 cover VN-specific FP suppression.

### ISS-004 — Regex panic / NER crash → silent data loss
Without fail-closed, a scan error would emit raw body. Resolved: §1 #7 + `ScanResult::failed()` returns `<SCAN_FAILED>`; `brain.capture_pii_scan_failed` audit row + caller drops body; AC #15.

### ISS-005 — Recall measurement methodology unspecified
Without a fixture corpus + measurement test, "99.5% recall" is unverifiable. Resolved: §1 #8 + #9 + `tests/fixtures/pii-corpus*.jsonl` + §5 `measure()` function returning (recall, fp_rate); AC #12 #13 #14.

### ISS-006 — Tenant-legitimate PII (KYC vendors) gets over-redacted
Without allowlist, KYC vendor's product breaks. Resolved: §1 #10 + per-tenant `pii_allowlist[]` regex list + §3 allowlist filter step + AC #10.

## §3 — Resolution

All 6 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

---

*End of FR-BRAIN-111 audit.*
