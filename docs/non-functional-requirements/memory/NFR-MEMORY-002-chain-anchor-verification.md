---
id: NFR-MEMORY-002
title: "memory chain-anchor verification — every l2_memory read re-checks anchor; mismatch → quarantine"
module: memory
category: security
priority: MUST
verification: T
phase: P0
slo: "100% of l2_memory reads re-verify the chain anchor; any mismatch quarantines the row and sev-1 alerts"
owner: CSO
created: 2026-05-18
related_tasks: [TASK-MEMORY-101, TASK-MEMORY-102]
---

## §1 — Statement (BCP-14 normative)

1. Every read from `l2_memory` **MUST** re-verify the row's chain anchor (hash of `(prev_chain_hash, payload_canonical_json)`) against the on-row `chain_hash` column.
2. A verification failure (computed hash ≠ stored hash) **MUST** quarantine the row: row is moved to `l2_memory_quarantine`, NOT served to callers, and a sev-1 alert fires.
3. The chain-anchor verification **MUST** add < 200µs p99 per row read; cryptographic hashing (BLAKE3) is the only allowed primitive.
4. A doctor full-chain scan (`cyberos-memory doctor`) **MUST** verify every anchor in O(N) time and complete < 30s on a 100k-row store (NFR-MEMORY-004).
5. Quarantined rows **MUST NOT** be deleted automatically — they remain for forensic review by CSO. Manual `cyberos-memory release-quarantine <row_id>` restores after CSO sign-off.

## §2 — Why this constraint

The chain anchor is the platform's tamper-evidence primitive. Without read-time verification, an attacker who writes a malicious row to Postgres bypasses the memory audit chain (the chain only protects writes via the ingest pipeline; direct DB write would corrupt the chain silently). Read-time verify catches direct-DB-write tampering. The < 200µs p99 budget keeps verify invisible inside the much larger query roundtrip. Quarantine instead of delete preserves the evidence for forensic analysis — deleting would lose the attack signature.

## §3 — Measurement

- Counter `memory_l2_anchor_verify_total{result}` where result ∈ {`ok`, `mismatch`}.
- Gauge `memory_l2_quarantine_rows_total` — should be near-zero.
- Histogram `memory_l2_anchor_verify_seconds` — p99 < 0.0002.
- Sev-1 alarm on `result=mismatch > 0`.

## §4 — Verification

- Property test `services/memory/tests/chain_anchor_test.rs` (T) — 1000 random rows; asserts every legit row verifies; injects 100 tampered rows; asserts all quarantine.
- Doctor smoke (T) — `cyberos-memory doctor` full scan on 100k-row test store; asserts no false-positive quarantines.

## §5 — Failure handling

- `mismatch > 0` → sev-1; CSO immediate review; investigate whether direct DB write occurred (audit pg_stat_statements + pgaudit).
- Quarantine rows piling up (> 100) → sev-1; tampering may be systematic; emergency CSO + CTO call.
- Verify latency > 1ms p99 → sev-3; BLAKE3 implementation may have regressed; benchmark.

---

*End of NFR-MEMORY-002.*
