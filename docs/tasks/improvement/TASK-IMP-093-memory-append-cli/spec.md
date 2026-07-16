---
id: TASK-IMP-093
title: memory-append CLI, doc-driven runs can append chained BRAIN rows
template: task@1
type: improvement
module: improvement
status: testing
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T08:05:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-085]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-17
shipped: null
memory_chain_hash: null
effort_hours: 5
service: tools/install/docs-tools
new_files:
  - tools/install/docs-tools/memory-append.mjs
  - tools/install/tests/test_memory_append.sh
modified_files:
  - tools/install/build.sh
source_pages:
  - "modules/memory AGENTS protocol (vendored to .cyberos/memory/AGENTS.md): §4.1 two-phase write (tmp + rename + dir sync; Darwin barrier caveat), §4.2 lock/HEAD seqlock, §6.3 append-only chain (chain = SHA-256(canonical(record_minus_chain) || prev_chain)), HEAD as 8-byte LE u64"
  - "IMPROVEMENT_HANDOFF.md IMP-05: skills declare memory_emit rows but a doc-driven run has no writer - the sachviet run parked payloads in docs/tasks/_audits/BATCH-2026-07-16-web.md instead of the chain"
  - "tools/install/build.sh guarded docs-tools copies (task-lint, ship-manifest, backlog-mutate) - the vendoring pattern this helper joins"
source_decisions:
  - "2026-07-17 Stephen: batch 4 PLAN approved (§0a, all 7 items)."
---

# TASK-IMP-093: memory-append CLI, doc-driven runs can append chained BRAIN rows

## Summary

Every ship-tasks phase declares memory rows (workflow_phase_complete, workflow_complete, task_routed_back, artefact_write), but without the MCP writer a doc-driven run has nowhere to append them - the real consumer run parked payloads in a tracked _audits file instead of the chain. Ship `memory-append.mjs` in docs-tools: a minimal, protocol-honoring appender for exactly those four kinds, with a verify mode, so governed runs can keep the chain truthful from any environment that has node.

## Problem

The BRAIN's value is the chain; rows that live in a parking file have no prev_chain, no HEAD ordinal, and no tamper evidence. The protocol is written and vendored - what is missing is a writer small enough to run inside a doc-driven session.

## Proposed Solution

`node .cyberos/docs-tools/memory-append.mjs append <store-root> <kind> <payload.json|->` and `... verify <store-root>`. Append: acquire the store lock, read HEAD (8-byte LE u64), build the record (kind, payload, actor, at, prev_chain), compute `chain = SHA-256(canonical(record_minus_chain) || prev_chain)` per §6.3, two-phase-write the row file and the advanced HEAD per §4.1 (tmp + rename + fsync; the Darwin F_BARRIERFSYNC caveat documented in-file - node exposes fsync only), release the lock. A fresh store bootstraps HEAD=0 with a null prev_chain root. Verify: walk the rows, recompute every chain link, compare the tip to HEAD. Kinds outside the four workflow kinds are refused before any write. build.sh gains the guarded vendor copy alongside its three helper siblings.

## Alternatives Considered

- Wire the full MCP memory server into doc-driven runs. Rejected: the server carries put_if, episodes, and checkpoints - far more surface than a run needs; a session without MCP still needs the append path.
- Keep parking rows in _audits. Rejected: it is chain-shaped data with no chain; the 086 incident showed what unverifiable records cost.
- Python instead of node. Rejected: docs-tools is node-stdlib by convention and node is the one runtime install already requires for the status page.

## Success Metrics

- Primary: three appends to a fresh scratch store advance HEAD by exactly 3 and verify recomputes the full chain clean, on every suite run. Baseline: zero doc-driven appends possible (rows parked in _audits). Deadline: final acceptance.
- Guardrail: a tampered row byte makes verify exit non-zero naming the ordinal (chain property, not just a counter).

## Scope

In scope: the two subcommands, the four kinds, fresh-store bootstrap, the suite, the build.sh vendor line.

### Out of scope / Non-Goals

- put_if, episodes, checkpoints, signed tree heads (§7) - the MCP server's domain.
- Migrating the parked sachviet rows (operator choice later; the tool makes it possible).
- Concurrent multi-writer arbitration beyond the protocol's lock file.

## Dependencies

- None upstream. Shares tools/install/build.sh with TASK-IMP-098 - same agent, serial order per the batch plan.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from the vendored memory protocol and IMP-05; implementation under ship-tasks supervision.
- **Human review:** batch-4 PLAN approved 2026-07-17 (§0a); both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 The CLI MUST implement `append <store-root> <kind> <payload.json|->`: lock, read HEAD (8-byte LE u64), build the record with prev_chain = current tip, compute chain per §6.3 (SHA-256 over canonical record-minus-chain concatenated with prev_chain), two-phase-write row and HEAD per §4.1 (tmp + rename + fsync; Darwin caveat documented in-file), unlock.
- 1.2 Kinds MUST be restricted to workflow_phase_complete, workflow_complete, task_routed_back, artefact_write; any other kind is refused with a non-zero exit BEFORE any write.
- 1.3 A fresh store (no HEAD) MUST bootstrap deterministically: HEAD=0, null-root prev_chain, directories created; a second append then chains normally.
- 1.4 The CLI MUST implement `verify <store-root>`: recompute every link, compare the tip against HEAD, exit non-zero naming the first bad ordinal on mismatch.
- 1.5 build.sh MUST vendor the file with the same guarded pattern as its helper siblings, and the payload copy MUST be gated in the suite against a scratch build.
- 1.6 The suite MUST land at tools/install/tests/test_memory_append.sh (run_all glob discovery).

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.3) - fresh store, three appends, HEAD advances by 3, rows chained - test: `tools/install/tests/test_memory_append.sh::t01_fresh_store_three_appends`
- [ ] AC 2 (traces_to: #1.4) - verify passes clean and fails on a tampered byte naming the ordinal - test: `tools/install/tests/test_memory_append.sh::t02_verify_and_tamper`
- [ ] AC 3 (traces_to: #1.2) - malformed kind refused, store byte-untouched - test: `tools/install/tests/test_memory_append.sh::t03_bad_kind_refused`
- [ ] AC 4 (traces_to: #1.5) - scratch payload carries the tool - test: `tools/install/tests/test_memory_append.sh::t04_payload_vendored`
- [ ] AC 5 (traces_to: #1.6) - suite discovered by the run_all glob - verify: runner output lists the suite (ops check recorded in the gate log; glob discovery is the runner's contract).

## 3. Edge cases

- Interrupted append (tmp file present, rename never ran): next append MUST ignore/clean stale tmp files; HEAD and chain stay consistent (asserted inside t01 via a seeded stale tmp).
- Payload on stdin (`-`) vs file: both accepted; non-JSON payload refused before write (t03 arm).
- Concurrent append attempt while locked: second invocation fails fast with a clear message rather than corrupting (t02 arm with a held lock).
- Security-class: writes only under the given store root; no network, no secrets; refuses paths escaping the store root.
