---
id: TASK-IMP-146
title: "memory-append --dry-run — rehearse an append, write nothing"
template: task@1
type: improvement
module: improvement
status: done
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-25T10:40:00+00:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-141, TASK-CUO-303, TASK-CUO-305]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.5.1"
owner: Stephen Cheng (CTO)
created: 2026-07-25
effort_hours: 4
service: tools/install/docs-tools
new_files: []
modified_files:
  - tools/install/docs-tools/memory-append.mjs
  - tools/install/tests/test_memory_append.sh
  - CHANGELOG.md
source_pages:
  - "tools/install/docs-tools/memory-append.mjs cmdAppend(): mkdirSync(store) + acquireLease() + bootstrap + cleanStaleTmp() all run before the payload is proven appendable, so today the only way to test a payload is to write it"
  - "docs/batches/batch-8b-ship-notes.md / batch-8c: gated flips probed against the live BRAIN store during batch/8 shipping"
  - "modules/memory doctrine §6.5 - the chain is append-only; recovery belongs to the canonical writer, so a probe row is permanent"
source_decisions:
  - "2026-07-25 session operator: post-1.2.0 plan Wave 2 residual - the plan's optional '--dry-run; avoid probe rows on the live store' item was not carried by TASK-CUO-305."
---

# TASK-IMP-146: memory-append --dry-run

## Summary

`memory-append.mjs append` has no rehearsal mode: the only way to learn whether a
`status_overridden` payload will be accepted is to append it, and the chain is append-only,
so a probe row is permanent. Add `--dry-run`, which runs every refusal the real path runs
and reports the seq, chain hash and memory path the append would produce — touching no
byte of the store.

## Problem

The gated-flip path (`backlog-mutate flip … --verdict-by --verdict-evidence`) appends one
`status_overridden` row per HITL verdict. Its payload has five required non-empty string
fields, a safe-token constraint on `task_id`, and preconditions on the store itself (lease
free, chain verifies, HEAD agrees with the rows). All of those are checked inside
`cmdAppend`, and the only way to exercise them is to perform the append.

Batch/8 shipping did exactly that and left probe rows on the live BRAIN store, which the
post-1.2.0 plan called out as Wave 2 work ("optional: `memory-append --dry-run`; avoid
probe rows on the live store"). `TASK-CUO-305` folded the other four friction items into
`ship-tasks.md` and did not carry this one. Doctrine §6.5 forbids tail rewrites, so the
operator's only alternative today is to copy the whole store and probe the copy — which
also drops the one precondition that matters most, the live store's own state.

## Proposed Solution

Add a `--dry-run` flag to `append`. In dry-run mode `cmdAppend` performs, in order: the
same closed-kind / JSON-object / `status_overridden`-field / safe-token refusals it always
performs before any write; a lease *inspection* that refuses with the same exit 3 when the
lease is held (without minting one); the full chain walk and HEAD/tip reconciliation
(reporting, never re-publishing, a one-behind HEAD); and then the seq, chain hash,
prev_chain and memory path it would write. It creates no store root, writes no `.lock`,
bootstraps no scaffold, sweeps no tmp litter, appends no frame, publishes no HEAD and
rebuilds no `peaks.bin`. Against a store that does not exist, it reports the bootstrap it
would perform instead of performing it.

The lease check is factored out of `acquireLease` into an `inspectLease` helper so the
dry-run and the real path share one refusal rule rather than growing a second copy of it.

## Alternatives Considered

- **`--dry-run` as a separate `plan` command.** Rejected: the flag composes with the same
  positional grammar and `--json` envelope the real command already has; a second command
  would duplicate the argument handling and the refusal ladder.
- **Copy the store to a temp dir and append there.** Rejected: that is today's workaround.
  It costs an O(store) copy, and it cannot check the live store's lease or HEAD — the two
  preconditions most likely to be the reason a real append refuses.
- **Validate the payload only (no chain walk).** Rejected: payload validation is the cheap
  half. The refusals that actually surprise an operator mid-flip are lock-held and
  HEAD/tip disagreement, and both are store-state checks.
- **Let the dry run acquire and release the lease for fidelity.** Rejected: a lease write
  is a store mutation, and a rehearsal that mutates the thing it is rehearsing against
  fails its one promise. Inspecting the lease gives the same refusal without the write.

## Success Metrics

- Primary: a dry run against the live store returns the projected seq/chain and leaves
  every byte under the store root — `HEAD`, `.lock`, `audit/`, `audit/mmr/peaks.bin` —
  bit-identical, proven by a recursive hash before and after.
- Guardrail: the real append path's exit codes, stdout and `--json` envelope are unchanged
  for every existing test in `tools/install/tests/test_memory_append.sh`.

## Scope

In scope: the flag, the `inspectLease` refactor, help/header documentation, suite
coverage, CHANGELOG.

### Out of scope / Non-Goals

- A dry-run mode for `verify` — verify already only reports (§6.5).
- Any change to exit codes, output shape or behaviour of the real append path.
- Teaching `backlog-mutate.mjs` to call the dry run before a flip (a possible follow-up;
  this task ships the capability, not a new gate).

## Dependencies

None. `TASK-IMP-141` (MMR sync) is `done` and its `syncMmrPeaks` call is one of the writes
the dry run must skip.

## AI Authorship Disclosure

- **Tools used:** Cursor agent (Composer) auditing the post-1.2.0 plan's Wave 2 residuals
  against live `main`.
- **Scope:** the plan's own framing was treated as a claim to verify, not as an input.
  **re-derived and CONFIRMED:** `cmdAppend` at HEAD performs seven distinct store writes —
  store-root `mkdir`, `.lock` lease, bootstrap scaffold, stale-tmp sweep, segment,
  `HEAD` publish and `peaks.bin` rebuild — each read directly out of the function.
  **re-derived and CORRECTED:** the plan calls this "optional"; the lease and HEAD/tip
  checks live inside the same function as the writes, so the copy-the-store workaround
  cannot exercise them — the item is a capability gap, not a convenience.
  **measured and ADDED:** `acquireLease` today mixes inspection and minting in one
  function, which is why this task factors out `inspectLease` rather than duplicating the
  staleness rule. No claim is made about paths outside `memory-append.mjs`.
- **Human review:** session operator instruction (2026-07-25) to close genuine plan gaps;
  HITL verdicts recorded with evidence at both gates.

## 1. Description

- 1.1 `append` MUST accept `--dry-run`. With it set, the command MUST NOT create, modify,
  delete or rename any path under the store root — specifically no store-root `mkdir`, no
  `.lock` write, no bootstrap scaffold, no stale-tmp sweep, no segment write, no `HEAD`
  publish and no `audit/mmr/peaks.bin` rebuild.
- 1.2 Every refusal the real path raises before writing MUST still be raised, with the same
  exit code: unknown kind, non-JSON or non-object payload, a `status_overridden` payload
  missing a required non-empty string field, an unsafe task token and a bad `--now` (2);
  a held or unparseable lease (3); a chain that does not verify or a HEAD that disagrees
  with the rows (4).
- 1.3 The lease rule MUST be single-sourced: `acquireLease` and the dry-run check MUST
  share one `inspectLease` implementation, so a change to staleness or TTL semantics cannot
  apply to one path and not the other.
- 1.4 On success the dry run MUST report the seq, chain hash, prev_chain, kind, memory path
  and actor the real append would produce for the same inputs, clock and store state.
- 1.5 With `--json`, the envelope MUST carry `dry_run: true` alongside the projected
  fields; without it, the prose line MUST say plainly that nothing was written.
- 1.6 A dry run against a store root that does not exist MUST exit 0, state that the real
  append would bootstrap the store, and leave the path absent.
- 1.7 A HEAD that is one behind the intact rows MUST be reported as the re-publish the real
  append would perform, not performed.
- 1.8 `--help`, the header usage block and `CHANGELOG.md` Unreleased MUST document the flag.

## Acceptance Criteria

- [ ] AC 1 (traces_to: #1.1, #1.4, #1.5) — a dry run on a populated store projects the seq and chain a subsequent real append actually writes, emits `dry_run: true` under `--json`, and leaves a recursive hash of the store root identical before and after - test: `tools/install/tests/test_memory_append.sh::t06_dry_run_writes_nothing`
- [ ] AC 2 (traces_to: #1.2, #1.3) — every refusal class exits with its documented code under `--dry-run` and leaves the store hash unchanged, and both paths route through the single `inspectLease` helper - test: `tools/install/tests/test_memory_append.sh::t07_dry_run_refuses_identically`
- [ ] AC 3 (traces_to: #1.6, #1.7) — a dry run against a nonexistent store root exits 0, reports the bootstrap and creates no path; a one-behind HEAD is reported rather than re-published - test: `tools/install/tests/test_memory_append.sh::t08_dry_run_bootstrap_and_head_report`
- [ ] AC 4 (traces_to: #1.8) — `--help` names `--dry-run` and CHANGELOG Unreleased records it - test: `tools/install/tests/test_memory_append.sh::t09_dry_run_documented`

## Test plan

1. `bash tools/install/tests/test_memory_append.sh`
2. `bash tools/install/tests/test_hitl_lock.sh`
3. `bash scripts/tests/run_all.sh`
4. `bash .cyberos/cuo/gates/run-gates.sh`

## 3. Edge cases

- **A concurrent writer appends between the dry run and the real append.** The projected
  seq/chain then differ from what lands — inherent to a rehearsal, and the dry run's own
  refusal ladder is what protects the real call. The output says "would write", not "will".
- **The clock moves between the rehearsal and the real call.** `ts_ns` is inside the hashed
  record, so an unpinned wall clock gives a different chain hash for the same payload. The
  projection is exact under the tool's documented determinism contract (same store, args,
  actor and `--now`/`CYBEROS_NOW`); the seq and path are exact either way.
- **Store root exists but is empty (no HEAD, no rows).** Same as the nonexistent case:
  report the bootstrap, create nothing — the real path's bootstrap branch is the one being
  described.
- **A store with audit rows but no HEAD.** Refuses with exit 4 exactly as the real path
  does; the dry run must not "helpfully" treat it as a bootstrap.
- **A compacted (`.binlog.zst`) store.** `listSegments` refuses with exit 4 during the walk,
  which the dry run inherits unchanged.
- **Security-class:** the flag strictly reduces what the tool may do; it grants no new
  read, adds no network or secret surface, and cannot be used to write a row that the
  normal path would refuse.
