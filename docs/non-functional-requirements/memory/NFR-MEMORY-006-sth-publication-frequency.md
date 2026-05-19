---
id: NFR-MEMORY-006
title: "memory STH publication frequency — consolidate every 24h or 100MB whichever first"
module: memory
category: compliance
priority: MUST
verification: T
phase: P1
slo: "Signed Tree Head (STH) published every 24h OR 100MB of new rows, whichever occurs first"
owner: CSO
created: 2026-05-18
related_frs: [FR-MEMORY-102, FR-MEMORY-101]
---

## §1 — Statement (BCP-14 normative)

1. The memory consolidation process **MUST** publish a Signed Tree Head (STH) — a Merkle root over the consolidated window's rows, signed by the platform key — every 24 hours of wall-clock time OR every 100MB of new rows ingested since the last STH, whichever threshold is hit first.
2. Each STH **MUST** include: `{tree_size_n, merkle_root_hex, timestamp, signature, prev_sth_hash, schema_version}`.
3. STHs **MUST** form a chain: each STH carries the prev STH's hash; tampering with any STH breaks the chain.
4. STHs **MUST** be persisted in a tamper-evident store: appended to `l1_audit` table with a UNIQUE constraint on `sth_id`, and mirrored to an external log (e.g., Sigstore-style transparency log, P2 deferral).
5. STH publication **MUST NOT** be skipped on consolidation failure — the failing consolidation still emits a "tombstone STH" carrying the error reason for forensic continuity.

## §2 — Why this constraint

The STH is the cryptographic checkpoint that lets external parties verify "the platform's audit chain didn't get rewritten." Without periodic STHs, the audit chain is only as trustworthy as the platform's own database — an adversary with DB access could rewrite history. The 24h/100MB combined cadence balances STH overhead (one signature per STH) against the forensic resolution (max 24h or 100MB of unwitnessed history). The chain-of-STHs makes any retroactive tampering detectable: an attacker would have to forge every STH from the tamper-point forward.

## §3 — Measurement

- Counter `memory_sth_published_total{result}` per STH attempt.
- Gauge `memory_sth_age_seconds` — time since last STH; alarm at > 86400 + 3600 (1h grace beyond 24h target).
- Gauge `memory_sth_bytes_since_last` — alarm at > 100 * 1024 * 1024 + grace.
- memory doctor invariant: STH chain unbroken end-to-end.

## §4 — Verification

- Integration test `services/memory/tests/sth_publication_test.rs` (T) — drives 100MB of writes; asserts STH published before threshold; drives time-fast-forward 24h; asserts time-based STH published.
- Doctor full-chain verify (T) — `cyberos-memory doctor --verify-sth-chain` walks every STH and verifies signatures + prev-hashes.

## §5 — Failure handling

- STH age > 25h → sev-2; consolidation may have stalled; investigate.
- STH chain broken → sev-1; forensic investigation; possible tampering.
- Signing key unavailable → sev-1; STH falls behind; emergency rotate signing key.

---

*End of NFR-MEMORY-006.*
