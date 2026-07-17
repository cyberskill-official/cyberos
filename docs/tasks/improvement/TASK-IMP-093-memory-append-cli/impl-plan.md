---
artefact: implementation-plan@1
task_id: TASK-IMP-093
created: 2026-07-17
estimate_pts: 3
verdict: pass (implementation-plan-audit: every matrix row addressed by a slice, context-map patterns respected, estimate sane vs spec effort_hours 5)
---
# Implementation plan - TASK-IMP-093

Slices (each maps to §1 clauses and edge-case-matrix rows):
1. Protocol primitives in tools/install/docs-tools/memory-append.mjs - true CRC-32C
   (table, reflected 0x82F63B78, vector-checked 0xE3069283), canonical JSON matching
   msgspec order='sorted' for the closed schema (sorted keys, compact, BigInt ts_ns,
   NaN/Infinity refused), §4.1 two-phase `atomicWriteBytes` (tmp.<nonce> + fsync +
   rename + parent-dir fsync; Darwin F_BARRIERFSYNC caveat in the header), injectable
   clock `nowNs` (--now / CYBEROS_NOW), LE-u64 HEAD read/write (§1.1; rows 4, 10).
2. §4.2 lease lock without flock - acquireLease reads `.lock`, fails fast (exit 3) on
   an unexpired or unparseable lease, reaps expired leases loudly, writes the
   {pid, host, monotonic_ns, expiry_ns, version} record, releases by truncation;
   check-then-write window documented (§3 edge; rows 8, 9).
3. walkChain - the shared integrity walk (append and verify both use it): every frame
   across audit/*.binlog (sealed segments name-sorted first, current.binlog last,
   .binlog.zst refused by name) recomputing crc32c, seq continuity, prev_chain linkage,
   and the §6.3 chain hash by blanking the top-level chain span IN THE RAW BYTES
   (immune to float/bigint reserialization); first failure throws Refusal(4) naming
   the ordinal (§1.4; rows 7, 11).
4. cmdAppend - refusals BEFORE any write (closed kind set, JSON-object payload, safe
   task token), fresh-store bootstrap (canonical dirs + HEAD=0), stale-tmp cleanup,
   full-chain pre-verification, record assembly per the session.py non-file-op shape
   (op=kind, path=meta/workflow/<task>.json, extra=payload, content_sha256=""),
   frame append via whole-segment two-phase rewrite, HEAD publish AFTER the segment
   is durable (writer.py order), benign rows==HEAD+1 heal mirroring _recover_tail
   (§1.1, §1.2, §1.3; rows 1, 3, 5, 6, 12).
5. cmdVerify + CLI shell - tip-vs-HEAD compare with direction-aware messages, --json
   stable envelopes, --help with the exit-code table, then the gating suite
   tools/install/tests/test_memory_append.sh (t01-t04 per the spec's AC names, harness
   mirrored from test_workflow_helpers.sh) + the 2-line guarded vendor copy in
   build.sh's docs-tools block (§1.4, §1.5, §1.6; rows 2, 7, and t04).

Pattern conformance (context-map): node stdlib only (node:fs, node:crypto, node:path,
node:os), single ESM file, whole-file doc comment carrying the on-disk mapping the
spec demands, loud failures, deterministic under a pinned clock. Out of scope honored:
no put_if/episodes/checkpoints/STH (§7 stays with the MCP server), no migration of the
parked sachviet rows, no multi-writer arbitration beyond the protocol's lease.

Estimate: 3 pts (~5 h) - matches spec effort_hours: 5. Actual landed surface: 2 new
files (memory-append.mjs 524 lines, test_memory_append.sh 202 lines), 1 modified
(build.sh +2, zero deletions), suite 4/4 in ~2 s including the payload build.
