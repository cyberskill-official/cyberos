---
artefact: repo-context-map@1
task_id: TASK-IMP-093
created: 2026-07-17
verdict: pass (repo-context-map-audit: patterns pinned to file:line, outside-domain count stated, ADR trigger evaluated)
---
# Repo context map - TASK-IMP-093

## Baseline patterns the new code must follow
- docs-tools convention: node stdlib only, ESM, single self-contained .mjs, whole-file doc comment, exit-code table in --help, loud failures, `--json` stable-stringified envelope - pinned_in: tools/install/docs-tools/backlog-mutate.mjs:1-52 and ship-manifest.mjs:1-52 (the two TASK-IMP-085 peers this tool joins)
- two-phase atomic write idiom: `.tmp.<nonce>` + fsync + rename - pinned_in: ship-manifest.mjs:80-87 (`atomicWrite`); this tool extends it with the §4.1 parent-dir fsync the protocol additionally requires (memory-append.mjs `atomicWriteBytes`)
- injectable clock: `--now` / `CYBEROS_NOW`, wall clock only as fallback - pinned_in: ship-manifest.mjs:155-163 (`nowISO`); here it pins `ts_ns` (memory-append.mjs `nowNs`)
- payload vendoring shape: per-file `[ -f ... ] && cp` guarded lines inside build.sh's docs-tools block - pinned_in: tools/install/build.sh:174-179 (task-lint / ship-manifest / backlog-mutate); the new line lands at build.sh:178-179 in the identical idiom
- install lay-down: the payload `docs-tools/` dir is copied verbatim into `.cyberos/docs-tools/` - pinned_in: tools/install/install.sh (docs-tools block) - vendoring into the payload is the ONLY plumbing needed
- test harness shape: self-contained bash, `set -uo pipefail`, here/repo resolution, mktemp TMP + trap, ok/fail counters, `pass=N fail=N` summary, scenario filter via `want` - pinned_in: tools/install/tests/test_workflow_helpers.sh:54-63
- suite discovery: scripts/tests/run_all.sh:43 globs `tools/install/tests/test_*.sh` - the new suite is picked up with zero wiring (AC 5)

## The protocol this tool implements (source of truth for every byte)
- store layout + scaffold: AGENTS.md §2 tree; install.sh:302-325 creates the live shape (12 canonical top-level dirs, empty `.lock`, 8-byte zero `HEAD`, manifest.json) - the appender bootstraps the same dirs + HEAD and never touches manifest.json (install.sh owns full scaffolding)
- HEAD: 8-byte LE u64 seq counter - AGENTS.md §2; modules/memory/cyberos/core/writer.py:79 (`_HEAD_FMT = struct.Struct("<Q")`), published atomically writer.py:472-492 (`_publish_head`: tmp + fdatasync + rename + parent-dir sync)
- frame format: `[u32 length BE][u32 crc32c BE][u64 seq BE][u64 ts_ns BE][payload]` - AGENTS.md §6.2; writer.py:64-78 (`_FRAME_HDR = struct.Struct(">IIQQ")`, 24 bytes); payload is canonical JSON (msgspec `order='sorted'`, RFC 8785-conforming for the closed schema, writer.py:124-134)
- chain: `chain = SHA-256(canonical(record_minus_chain) || prev_chain)` - AGENTS.md §6.3; the IMPLEMENTATION detail that prev_chain concatenates as RAW 32 bytes (`bytes.fromhex`) and record_minus_chain serializes WITH `chain:""` present (omit_defaults=False) is pinned in writer.py:137-142 (`_chain_hash`) + writer.py:86-121 (AuditRecord); genesis prev_chain is 64 zeros (writer.py:81)
- record shape for non-file-op kinds: `op=<kind>, path=meta/..., actor, content_sha256:"", extra={payload}` - pinned_in: modules/memory/cyberos/core/session.py:118-127 (session.start rows) and memory.schema.json#/definitions/AuditRecord (op is free-form min-length-1; extra untyped object; additionalProperties false)
- lock: `.lock` lease record `{pid, host, monotonic_ns, expiry_ns}` TTL 10 s - AGENTS.md §4.2; modules/memory/cyberos/core/lock.py:44-50 (TTL constants) + :100-120 (acquire semantics: expired leases force-broken). Node stdlib has no flock(2) - the appender enforces the lease RECORD only (header caveat; the check-then-write window is documented tolerance for doc-driven single-writer runs)
- two-phase + Darwin: AGENTS.md §4.1 (tmp + rename + parent-dir sync; "Plain fsync() is insufficient on Darwin" - F_BARRIERFSYNC/F_FULLFSYNC required). Node exposes fsync only; the gap is documented in the file header per the spec's own requirement
- crash recovery: writer.py:496-544 (`_recover_tail`) truncates a torn tail and re-publishes HEAD; §6.5 forbids tail rewrites by anyone else - so this tool REFUSES torn stores in verify (reports first bad ordinal) and heals only the one benign rows==HEAD+1 window on append, mirroring recover_tail's HEAD re-publish
- crc32c: writer.py:145-164 - production is Castagnoli (hw wheel); the zlib fallback is explicitly "NOT CRC-32C ... fine for development". The node tool implements true CRC-32C (table, reflected 0x82F63B78, verified against the 0xE3069283 test vector) so frames interoperate with the production walker (walker.py:131 validates `_crc32c(payload) != crc`)

## Schemas / interfaces in scope
- CLI: `node memory-append.mjs [--json] [--actor <name>] [--now <ISO>] append <store-root> <kind> <payload.json|->` and `... verify <store-root>`
- kinds closed set (§1 #1.2): workflow_phase_complete | workflow_complete | task_routed_back | artefact_write - exactly the four rows ship-tasks.md declares (workflow doc lines 76-79); anything else refused BEFORE any write, exit 2
- exit codes: 0 ok; 2 usage/refused-before-write (bad kind, non-JSON/non-object payload, unreadable input, unsafe task token); 3 lock held / unparseable lease (fail fast); 4 integrity failure naming the first bad ordinal
- record path: `meta/workflow/<task>.json` from payload.task_id|task validated against the MemoryPath segment charset (memory.schema.json#/definitions/MemoryPath), else `meta/workflow/run.json`

## Files outside the immediate domain (tools/install/docs-tools/ + tools/install/tests/)
1. tools/install/build.sh (modified, +2 lines - the guarded vendor copy; spec-declared in `modified_files`, gated by t04)

files_outside_immediate_domain: 1 (<= 3 -> no ADR trigger; spec-declared, two lines in the exact idiom of its three sibling lines).

## Blast radius
file_count: 3 (2 new: memory-append.mjs 524 lines + test_memory_append.sh 202 lines; 1 modified: build.sh +2) | module_count: 2 (tools/install docs-tools+tests, tools/install build) | cross_module_edges: suite -> build.sh (t04 payload gate); tool -> modules/memory protocol at AUTHORING time only (format compiled in; no runtime read of AGENTS.md or writer.py)
module_placement_warning: null (spec declares `service: tools/install/docs-tools`; both new files sit where §1 #1.5/#1.6 fix them)
Behavioral radius: zero on existing production paths - the appender runs only when invoked. It writes ONLY under the given store root (the gitignored `.cyberos/memory/store/` in installed repos); the MCP writer keeps put_if/episodes/checkpoints (spec Out of scope). Consumer repos inherit the tool through the payload on their next install.
