# TASK-IMP-093 — code review packet

Files under review: new `tools/install/docs-tools/memory-append.mjs` (524 lines, the doc-driven BRAIN chain appender) and `tools/install/tests/test_memory_append.sh` (202 lines, the gating suite), modified `tools/install/build.sh` (+2 lines, guarded vendor copy — spec-declared in `modified_files`). Suite state at review: test_memory_append 4/4, 0 failed (~2 s including the payload build). build.sh is SHARED with batch sibling TASK-IMP-098 per the batch plan (same agent, serial order); at this review only the 093 line has landed.

## §1 clause → proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | `append <store-root> <kind> <payload.json|->`: lock, read HEAD (8-byte LE u64), record with prev_chain = current tip, chain per §6.3, two-phase row + HEAD per §4.1 (tmp + rename + fsync; Darwin caveat documented in-file), unlock | `t01_fresh_store_three_appends` — three appends (file + stdin `-`) advance HEAD 0→3 (LE-u64 read asserted via an independent node one-liner), rows verified chained by an INDEPENDENT frame walk that never calls the tool. Layout is writer.py-identical: frame `>IIQQ`+canonical-JSON (memory-append.mjs header "ON-DISK MAPPING" + `cmdAppend`), chain = SHA-256(canonical(record with chain:"") ‖ raw prev bytes) exactly `writer.py._chain_hash` (raw-byte concat pinned in-code); segment made durable BEFORE HEAD publishes (writer.py order). Two-phase: `atomicWriteBytes` = tmp.<nonce> + fsync + rename + parent-dir fsync; the Darwin F_BARRIERFSYNC gap is documented in the file header AND --help (`darwin` line) per the clause's own wording. Lease acquired before, released after (t02 lock arms). Cross-impl: python recompute of a tool-written store passed (gate-log E4) |
| 1.2 | kinds restricted to the four workflow kinds; any other refused with non-zero exit BEFORE any write | `t03_bad_kind_refused` — `episode_logged` → exit 2 naming the closed set, whole-store sha256 fingerprint byte-identical before/after; refusal precedes bootstrap too (fresh path gains NO files). Validation order is code-pinned in `cmdAppend`: kind → payload → task token, all before `mkdirSync`/lease/any write |
| 1.3 | fresh store bootstraps deterministically: HEAD=0, null-root prev_chain, dirs created; second append chains normally | `t01` bootstrap arm — loud stderr note, canonical dirs asserted (audit/memories/meta/conflicts/exports/index), HEAD=0 published atomically then advanced by exactly 1 per append; row 1's prev_chain is the 64-zero genesis (independent walk asserts it); appends 2-3 chain from the prior tip. Dir set matches install.sh:302-325's canonical scaffold; manifest.json deliberately left to install.sh (context-map) |
| 1.4 | `verify <store-root>`: recompute every link, compare tip against HEAD, exit non-zero naming the FIRST bad ordinal | `t02_verify_and_tamper` — clean pass exit 0; ONE flipped payload byte → exit 4 naming "first bad ordinal 1" (and asserted NOT naming ordinal 2); a chain-field flip with the crc deliberately RECOMPUTED (so only the §6.3 hash can catch it) → exit 4 naming ordinal 2. `walkChain` also enforces seq continuity, prev_chain linkage, truncated frames, and 64-hex chain shape; tip/HEAD mismatch messages are direction-aware (`cmdVerify`). Verify never rewrites (§6.5) — the benign heal lives only in append |
| 1.5 | build.sh vendors the file with the guarded sibling pattern; payload copy gated in the suite against a scratch build | build.sh:178-179 — one comment + one `[ -f ... ] && cp` line inside the docs-tools block, byte-idiom of the task-lint/ship-manifest/backlog-mutate lines above it. `t04_payload_vendored` builds into a scratch dir, asserts presence, `cmp` byte-parity with the source, runs the vendored copy's --help AND a full append+verify lifecycle |
| 1.6 | suite lands at tools/install/tests/test_memory_append.sh (run_all glob discovery) | the file exists at exactly that path; scripts/tests/run_all.sh:43 globs `tools/install/tests/test_*.sh` (gate-log E2 shows the expansion including it). AC 5's ops check is recorded in the gate log per the spec's verify wording |

## §3 edge cases

Interrupted append: t01 seeds `current.binlog.tmp.deadbeef` + `HEAD.tmp` mid-sequence — cleaned with a loud note naming both, chain and HEAD stay consistent; the crash window rows==HEAD+1 heals by re-publishing HEAD (mirrors writer.py `_recover_tail`), any other divergence refuses (exit 4). Stdin payload: t01 append 3 uses `-`. Non-JSON payload: t03 arm (plus a non-object arm). Held lock: t02 arm — exit 3, fast, nothing written, the foreign lease not clobbered; expired lease reaped loudly. Security class: writes only under the store root (all paths computed, task token charset-validated, no network/child_process/eval — import block is node:fs/crypto/path/os).

## Acceptance criteria

AC 1 `t01_fresh_store_three_appends` ok · AC 2 `t02_verify_and_tamper` ok · AC 3 `t03_bad_kind_refused` ok · AC 4 `t04_payload_vendored` ok · AC 5 run_all glob (ops check, gate-log E2) ok. Suite 4/4.

## Diff size

Two new files: `tools/install/docs-tools/memory-append.mjs` (524 lines, self-contained ESM, node stdlib only) and `tools/install/tests/test_memory_append.sh` (202 lines, executable). One modified file: `tools/install/build.sh` +2/−0 (guarded vendor copy). No dependency added anywhere. `dist/` untouched here — rebuild + version-sync before commit are the batch parent's step per payload-sync doctrine.

## Design disclosures

1. Layout fidelity over invention: the row layout is NOT a private format — frames, HEAD, chain math, and the non-file-op record shape are lifted from the canonical implementation (writer.py `_FRAME_HDR`/`_HEAD_FMT`/`_chain_hash`, session.py row shape) so the production walker can read doc-driven rows. The spec's §6.3 prose is ambiguous about prev_chain concat (hex text vs raw bytes); the implementation (bytes.fromhex) is authoritative and this tool matches it, documented in the header.
2. flock caveat: node stdlib exposes no flock(2); the §4.2 lease RECORD is enforced instead (fail-fast on unexpired/unparseable, loud reap on expired). The check-then-write window is documented as a doc-driven single-writer tolerance; the MCP writer remains the arbiter for true concurrency (spec Out of scope).
3. Append implementation: whole-segment tmp+rename rewrite (old bytes + one frame) — a literal §4.1 two-phase write of the row, O(segment) at doc-driven scale; append-only discipline preserved (prior bytes never modified), every append re-verifies the full chain first and refuses to extend a broken one.
4. CRC-32C is implemented for real (Castagnoli table, vector-checked) rather than node's zlib CRC-32 — writer.py's own comment marks the zlib fallback "NOT CRC-32C"; interop with the hw-crc production walker requires the real polynomial.

## Verdict

| Check | State |
|---|---|
| §1 clauses 1.1–1.6 | each proven above by a named test or pinned line |
| Primary metric (3 appends → HEAD+3, verify clean, every suite run) | pass (t01) |
| Guardrail metric (tampered byte → non-zero naming the ordinal) | pass (t02, two tamper classes) |
| §3 edge cases (stale tmp, stdin, held lock, security class) | each covered (t01/t02/t03 + code-pinned) |
| Refuse-before-write discipline (kind/payload/token) | pass (t03, byte-fingerprint compare) |
| Invariants (node stdlib only, payload doctrine, out-of-scope untouched, HITL) | intact |

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
