---
id: NFR-MEMORY-005
title: "memory deterministic export bit-identity — two runs on same store produce byte-identical zip"
module: memory
category: reliability
priority: MUST
verification: T
phase: P1
slo: "Two consecutive `cyberos-memory export` runs on an unchanged store produce identical SHA-256 hashes on the output bundle"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-MEMORY-103]
---

## §1 — Statement (BCP-14 normative)

1. The `cyberos-memory export <tenant_id>` command **MUST** produce a deterministic ZIP bundle: two runs against the same Layer-2 state **MUST** produce byte-identical output (verified by SHA-256 of the zip file).
2. Deterministic ordering **MUST** be enforced: rows sorted by (committed_at ASC, row_id ASC, hash ASC) where ties exist; JSON keys in canonical order; ZIP entries sorted alphabetically.
3. Timestamps in the bundle **MUST NOT** include run-time metadata (no "exported_at" inside the data files; that goes into a separate `manifest.json` with a SHA the verifier can recompute).
4. The bundle **MUST** carry a Merkle root over the contained files, allowing independent third-party verification of bundle integrity.
5. Compression **MUST** use deterministic settings (`zip -X -9` equivalent, no embedded timestamps in ZIP central directory).

## §2 — Why this constraint

Deterministic export is the compliance differentiator for DSAR (Data Subject Access Request) and tenant migration. A regulator who asks "did you give the subject the same data twice?" gets a definitive answer via bundle hash comparison. Without determinism, every export is a unique snapshot and provenance breaks. The Merkle root enables independent verification — a subject can re-export and compute the same root, proving the platform didn't redact differently between exports. The no-runtime-metadata rule prevents accidental non-determinism (the most common source).

## §3 — Measurement

- Test artifact archive — `tests/memory/export_determinism/expected_hashes.txt` lists expected SHA-256s for a battery of test stores.
- Counter `memory_export_hash_mismatch_total` — should always be zero; sev-2 alarm on non-zero.

## §4 — Verification

- Determinism test `services/memory/tests/export_determinism_test.rs` (T) — runs export twice on a fixed seeded store; asserts hashes match.
- Property test (T) — drives 100 random stores; asserts hash stability across two runs each.

## §5 — Failure handling

- Hash mismatch on test → sev-2 PR block; identify non-deterministic source (most often: hashmap iteration order, timestamps, zip metadata).
- Real-world export with mismatch → sev-2; investigate whether store has changed between runs (the legitimate cause).
- Subject reports their two exports differ → sev-1 compliance; immediate CSO investigation.

---

*End of NFR-MEMORY-005.*
