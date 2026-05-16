# BRAIN Layer 2 — source-of-truth contract

**Owner:** Stephen Cheng (CEO) · **Status:** v1.0.0 normative, 2026-05-15
**Companion files:** `cyberos/AGENTS.md` (Layer 1 spec) · `BRAIN_AUTOSYNC_DESIGN.md` (sync mechanism)
**Per research review §2.1:** "Before P1, write a one-pager that says explicitly: Layer 1 is the source of truth; Layer 2 is a derived index; on conflict Layer 1 wins; the Merkle chain is anchored at Layer 1 only; Layer 2 rebuild from Layer 1 is a tested CI job."

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when, and only when, they appear in all capitals.

---

## §1 — The contract in one sentence

**Layer 1 is the source of truth. Layer 2 is a derived index. On any conflict, Layer 1 wins, and the Layer 2 row MUST be regenerated from Layer 1.**

This is not a wishful design preference; it is a structural invariant the rest of CyberOS depends on. Every audit obligation (EU AI Act Art. 12, ISO 42001 AIMS, SOC 2 Type II) ultimately points at the chain that lives in Layer 1. If Layer 2 drifted and we treated it as authoritative for even one decision, the chain stops being audit-grade.

---

## §2 — What each layer actually is

| Layer | Storage | Mutator | Role | Trust |
|---|---|---|---|---|
| **Layer 1** | `<memory-root>/` on local filesystem — `audit/*.binlog`, `HEAD`, `memories/<kind>/...`, `meta/`, `index/` | `python3 -m cyberos.writer` (the canonical Writer) | **Source of truth.** Append-only, content-addressed, Merkle-chained (MMR), Ed25519-signed tree heads per consolidation. Deterministic export. | **Authoritative.** Every Tier 2 source per AGENTS.md §8.1 reads from here. |
| **Layer 2** | `pgvector` HNSW index + Apache AGE graph rows in Postgres | `cyberos.brain.layer2.ingest` (read-only consumer of Layer 1 binlog) | **Derived index.** Vector similarity (BGE-M3 embeddings), graph traversal, full-text search via PGroonga. | **Advisory.** Read-only for any application; never written to by anything except the ingest path. |
| **Layer 3** | S3/R2/MinIO archive of compacted binlogs | `cyberos.brain.consolidate --archive` | **Long-tail cold storage** (older than 12 months). Object-Lock Compliance bucket; 10-year retention. | **Cold-authoritative** for archived rows; the Layer 1 chain head holds the anchor. |

---

## §3 — Why Layer 1 is the only chain anchor

The Merkle Mountain Range (MMR) tree-head + Ed25519 signature lives **only** at Layer 1. Layer 2's pgvector + AGE rows have row-level timestamps and tenant labels, but they are not chain leaves. If a Layer 2 row contradicts a Layer 1 row, the auditor's verdict is: Layer 1 wins, Layer 2 is corrupt, regenerate.

Anchoring the chain in Layer 2 would require:
1. pgvector to support append-only semantics — it does not (rows can be UPDATE'd and DELETE'd).
2. A second cryptographic primitive on the database side — adds operational and audit complexity.
3. A reconciliation protocol when local and database disagree — non-trivial and error-prone.

None of these are worth solving when Layer 1 already provides the property and Layer 2's job is to be fast, not authoritative.

---

## §4 — Read-your-writes guarantee (the consistency model)

**Bounded staleness, with explicit reconciliation.**

When a memory is written:
1. The canonical Writer (Layer 1) appends to the binlog, fsyncs the row, advances HEAD. The row is durably committed within ~10ms (per AGENTS.md §4.1).
2. The Layer 2 ingest consumer reads the new binlog row and applies an UPSERT to the pgvector + AGE backing store. **Target staleness: ≤ 1 second under nominal load.**
3. The reader (Genie, OBS dashboard, KB search) MAY query Layer 2 first for performance, but MUST fall back to Layer 1 if the row is not present AND the reader needs read-your-writes (e.g., the AI Gateway's `ai.invocation` audit row that guards a sensitive operation).

**Behaviour under degraded Layer 2:**
- Layer 2 unavailable → Layer 1 reads continue uninterrupted; Genie's search latency degrades from ~150ms p95 to ~800ms p95 (a binlog scan vs a vector lookup).
- Layer 2 stale by &gt; 60 seconds → OBS sev-2 alert; Layer 2 ingest restarted from the last-known-good HEAD.
- Layer 2 chain-divergence detected (a Layer 2 row's `chain_anchor` doesn't match Layer 1 at the same seq) → Layer 2 partition truncated and rebuilt from Layer 1.

---

## §5 — Conflict resolution (one rule)

**Layer 1 wins. Always. No exceptions.**

If the Layer 2 ingest consumer encounters a row that conflicts with what's currently in Layer 2 (e.g., a memory was retracted in Layer 1 via the `correction_to` mechanism but the prior version is still indexed in Layer 2):
1. The conflicting Layer 2 row is DELETE'd.
2. The new Layer 1 row's content is ingested.
3. A `layer2_correction` OBS event is emitted with both row IDs and the chain seq.

There is no merge logic. There is no "prefer recent". There is no human dispute resolution at the row level. The Layer 1 chain is by-construction the answer.

---

## §6 — Layer 2 rebuild from Layer 1 (the CI gate)

**Layer 2 must be rebuildable from Layer 1 in bounded time, deterministically, by a CI job that runs on every commit to the BRAIN crate.**

The rebuild job:
1. Spins up an empty pgvector + AGE Postgres.
2. Streams every row of `<memory-root>/audit/*.binlog` (and the archived `.binlog.zst` segments in Layer 3) through the Layer 2 ingest pipeline.
3. Verifies that the resulting Layer 2 row count == the chain seq count.
4. Verifies that 100 random Layer 1 memory IDs return correct embeddings (cosine ≥ 0.99 against the stored vector — accounts for floating-point drift).
5. Asserts wall-clock rebuild time ≤ 30 minutes for a 100k-row tenant binlog on a 4-core dev VM.

CI gate status: **REQUIRED.** Adding any Layer 2 ingest code without the rebuild test landing in the same PR is a hard CI failure.

This test is not optional polish. It is the operational evidence that the contract in §1 is enforceable. Without the rebuild test, "Layer 2 is a derived index" becomes wishful thinking — there's no way to prove that Layer 2 actually re-derives from Layer 1 if it ever drifts.

---

## §7 — What can break this contract (and how we catch it)

| Break | Detection | Recovery |
|---|---|---|
| Layer 2 ingest stalls; rows in Layer 1 never reach Layer 2 | OBS `layer2_ingest_lag_seconds` metric > 60s for 5 minutes | Restart ingest from last known seq; alert; investigate cause |
| Direct UPDATE to Layer 2 outside the ingest path | RLS-style check: ingest writes use a dedicated DB user; any non-ingest UPDATE triggers a Postgres alert | Truncate affected partition; rebuild from Layer 1 |
| Layer 1 binlog corruption | `cyberos doctor` invariant 7 (chain integrity check) | `FROZEN_RECOVERABLE` state; human invoke `cyberos doctor --repair`; Layer 2 rebuild after Layer 1 is repaired |
| Layer 2 row claims to be at a chain seq that doesn't exist in Layer 1 | Periodic reconciliation job (hourly) compares Layer 2 row count vs Layer 1 HEAD | Truncate divergent partition; rebuild |
| Two Layer 2 instances diverge from each other on the same Layer 1 | Per-instance hash-of-rows aggregate compared across replicas | Promote one as canonical (highest seq); rebuild the other from Layer 1 |

---

## §8 — What this means for non-BRAIN modules

Every CyberOS module that depends on memory **MUST**:

1. Treat Layer 1 as the authoritative source for any decision with persistent effect (a payroll calc, a contract clause, a compliance attestation).
2. Use Layer 2 for any read-only display, search, or ambient query (Genie's "what did we decide last week?" panel).
3. Surface to the user any case where Layer 2 lag &gt; 5 seconds matters operationally (e.g., a freshly-written audit row is being read back for a chain-of-custody export).
4. Never write directly to Layer 2. Period.

Modules that need ACID guarantees stronger than Layer 2's "≤ 1 second staleness" use Layer 1 directly via the canonical Writer's read APIs.

---

## §9 — Cost ceilings (informational, sourced from NFR catalog)

At 50-tenant scale with 1M chunks each:
- **Layer 1** storage: ~5 GB/tenant of binlogs + compaction archives = ~250 GB / 50 tenants. Cost ≈ negligible.
- **Layer 2** memory: 1024-dim BGE-M3 embeddings × 1M chunks × 50 tenants = ~200 GB HNSW resident memory. Cost ≈ $1500/month on dedicated VMs (NOT managed RDS — see research review §2.1).
- **Layer 3** cold storage: ~50 GB/tenant of compacted segments × 50 tenants = 2.5 TB. Cost ≈ $25/month at S3 IA pricing.

The NFR catalog target "$2.2k/month at 50 tenants" is achievable IF Layer 2 runs on self-hosted dedicated VMs, NOT managed RDS. Plan for this transition by P3.

---

## §10 — Anchoring decisions

The following decisions are pinned by this document. Changing any of them is a protocol change per AGENTS.md §0.2 — requires explicit user approval citing the section being changed.

- **DEC-070** (this doc, 2026-05-15): Layer 1 is the only source of truth; Layer 2 is a derived index.
- **DEC-071** (this doc, 2026-05-15): The Merkle chain is anchored at Layer 1 only.
- **DEC-072** (this doc, 2026-05-15): A Layer 2 rebuild-from-Layer-1 CI job is a REQUIRED gate on every BRAIN PR.
- **DEC-073** (this doc, 2026-05-15): Layer 2 stale-by-default; readers needing read-your-writes MUST fall back to Layer 1.
- **DEC-074** (this doc, 2026-05-15): No application code may write directly to Layer 2.

---

*End of BRAIN Layer 2 source-of-truth one-pager.*
*Companion: `BRAIN_AUTOSYNC_DESIGN.md` for the multi-device sync mechanism that consumes this contract.*
