---
artefact: edge-case-matrix@1
task_id: TASK-IMP-093
total_rows: 12
created: 2026-07-17
verdict: pass (edge-case-matrix-audit: every category >=1 row, covered-by names real test functions, SECURITY rows point at code+test, DEGRADATION rows carry detection+recovery)
---
# Edge-case matrix - TASK-IMP-093

All test functions live in tools/install/tests/test_memory_append.sh.

| # | category | trigger | expected behavior | covered by |
|---|---|---|---|---|
| 1 | null/empty | append to a store path that does not exist at all | deterministic bootstrap: canonical dirs created, HEAD=0 (8 zero bytes) published atomically, first row chains from the 64-zero null root; loud stderr note | t01_fresh_store_three_appends (bootstrap arm) |
| 2 | null/empty | verify on a bootstrapped-but-empty store (HEAD=0, no rows) | exit 0, "0 row(s), HEAD=0 (empty store)" - an empty chain is consistent, never an error | exercised inside t04 lifecycle (fresh store verifies after 1 append; empty-store branch code-pinned in cmdVerify `rows > 0n` guard) |
| 3 | null/empty | payload names no task (`task_id`/`task` absent) | record path falls back to `meta/workflow/run.json` deterministically - code-pinned (cmdAppend memPath default), t02's p3.json uses `task` and t01's use `task_id`, both resolve | t01/t02 (path visible in --json envelope) |
| 4 | bounds | HEAD file with a byte length other than exactly 8 | Refusal exit 4 naming the actual length - never a guess at the seq | code-pinned (readHead); unreachable through the tool's own writes (leU64 always emits 8) |
| 5 | malformed | payload that is not JSON, and payload that is JSON but not an object | exit 2 BEFORE any write ("refused before any write"); store bytes untouched (proven by whole-store sha256 fingerprint compare) | t03_bad_kind_refused (non-JSON + non-object arms) |
| 6 | malformed | kind outside the closed four-kind set (e.g. `episode_logged`, the MCP writer's domain) | exit 2 BEFORE any write, message names the closed set; existing store byte-identical AND a fresh path gains no files (refusal precedes bootstrap) | t03_bad_kind_refused |
| 7 | malformed | one flipped payload byte in a mid-chain row; separately a flipped chain-field hex char with the crc RECOMPUTED so only the §6.3 hash can catch it | verify exits 4 naming the FIRST bad ordinal (crc32c arm names ordinal 1; chain-hash arm names ordinal 2); later ordinals never named first | t02_verify_and_tamper |
| 8 | concurrency/order | append while another writer holds an unexpired §4.2 lease in `.lock` | exit 3 fast, "failing fast, nothing written"; binlog+HEAD byte-identical; the held lease is NOT clobbered | t02_verify_and_tamper (held-lock arm) |
| 9 | concurrency/order | append over an EXPIRED lease (writer killed; `expiry_ns` in the past) | stale lease reaped loudly (stderr note, matching python StoreLock force-break), append proceeds, chain stays consistent | t02_verify_and_tamper (expired-lease arm) |
| 10 | SECURITY | hostile payload tries to steer the record path (`task_id` carrying `../` or `/` separators) or smuggle non-finite numbers | task token validated against the MemoryPath segment charset - violation is exit 2 before any write; canonicalJSON refuses NaN/Infinity; the tool writes ONLY under the given store root, no network, no child_process, no eval (import block: node:fs/crypto/path/os) | code-pinned (SAFE_TOKEN check + canonicalJSON guard); t03 proves the refuse-before-write discipline the path check shares |
| 11 | SECURITY | tampered ledger presented as truth (the 086 incident class: unverifiable records) | verify recomputes EVERY link from raw bytes (crc32c + seq continuity + prev_chain linkage + §6.3 hash with raw-byte prev concat) and compares tip to HEAD - a single flipped bit anywhere in any row is named by ordinal | t02_verify_and_tamper + t01's independent (non-tool) chain walk |
| 12 | DEGRADATION | interrupted append: stale `current.binlog.tmp.<nonce>`/`HEAD.tmp` litter present; or crash between segment rename and HEAD publish (rows exactly one ahead of HEAD) | detection: readers/walker open exact final paths so tmp is never state; next locked append cleans litter with a loud note (t01 seeds both patterns mid-sequence). recovery: the one benign rows==HEAD+1 window re-publishes HEAD (mirrors writer.py _recover_tail); ANY other divergence refuses with exit 4 telling the operator to run verify - §6.5 forbids this tool rewriting a tail | t01_fresh_store_three_appends (seeded stale tmp); rows-ahead heal code-pinned (cmdAppend lastSeq==head+1 branch) |

Documented-by-design: compacted `.binlog.zst` segments are refused by name (exit 4, "use the canonical cyberos walker") - consolidation is the MCP server's domain and node stdlib carries no zstd on the supported floor. Cross-implementation canonical-JSON identity is asserted for the ASCII workflow payloads this tool writes (independent python sorted-keys recompute passed during implementation); exotic payloads (non-ASCII keys, shortest-repr floats) are documented header caveats, and verify is immune by construction (raw-byte recompute, never reserialization). Darwin power-loss durability is a documented §4.1 gap (node exposes fsync only, no F_BARRIERFSYNC) - crash CONSISTENCY via rename atomicity is what the suite proves.
