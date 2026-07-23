# TASK-CUO-303 — implementation evidence

Implementer: batch/8-audit-hardening swarm worker (Cursor agent), 2026-07-23.
Partition note: this worker's exclusive ownership set was the two docs-tools source
files + `modules/cuo/cuo/**.py` + modules/cuo tests. Spec clauses landing outside that
set are implemented-ready but DEFERRED with exact patches below (shared-tree rule:
only the final sequential pass / owning sibling edits those files).

## What changed and why

| File | Change | Spec clause |
|---|---|---|
| `tools/install/docs-tools/backlog-mutate.mjs` | Verdict gate on exactly `reviewing->ready_to_test` and `testing->done`: refuses exit 8 (naming STATUS-REFERENCE §1.4 + "verdict") unless `--verdict-by` (non-empty) + `--verdict-evidence` (existing, non-empty regular file; dir/unreadable = missing) are supplied. Gate evaluates AFTER all exit-6 refusals. On a gated flip, ONE `status_overridden` row is appended via the sibling `memory-append.mjs` BEFORE the index moves when a BRAIN store resolves (`CYBEROS_STORE` override, else `<root>/.cyberos/memory/store`); append failure on a present store fails the flip with new exit 9; no store = flip succeeds, stderr says the evidence file is the record. `--json` refusal AND success envelopes carry `verdict_by`/`verdict_evidence` (+ `audit_row` on success). Header/help/exit-code docs updated. | 1.1, 1.2, 1.4 + `--json` edge case |
| `tools/install/docs-tools/memory-append.mjs` | `KINDS` extended with `status_overridden` (wire form; doctrine's `memory.status_overridden` is the taxonomy name, per audit ISS-004). Payload validation: required non-empty string fields `{actor, task_id, prior_status, new_status, reason}`, refused exit 2 BEFORE any write (store dir not even created). Unknown kinds keep today's refusal; the four legacy kinds unchanged. Docs/help updated. | 1.3 |
| `.cyberos/docs-tools/backlog-mutate.mjs`, `.cyberos/docs-tools/memory-append.mjs` | MACHINE-LOCAL (gitignored) installed copies refreshed from source to test end-state behavior. | — |

## Verification (all against scratch fixtures under mktemp; live BACKLOG.md and live
## .cyberos/memory/store were never read as a store nor written)

Command: `bash /tmp/cuo303_matrix.sh` (22-case matrix; fixtures mirror
`test_workflow_helpers.sh`'s `emit_guard_fixture`; clock pinned `CYBEROS_NOW=2026-07-23T03:46:00Z`).
Verbatim output:

```
== c01 bare gate flip reviewing->ready_to_test refuses 8, no write ==
  PASS c01
  stderr: backlog-mutate: flip TASK-GUARD-001: 'reviewing -> ready_to_test' is a human-acceptance gate (STATUS-REFERENCE §1.4): a recorded human verdict must accompany it - --verdict-by is missing; --verdict-evidence is missing. Pass --verdict-by <actor> and --verdict-evidence <path to the review/acceptance note the human produced>. Refusing; nothing written.
== c02 flagged gate flip succeeds, no store => stderr says evidence is the record ==
  PASS c02
  stdout: flip TASK-GUARD-001: [reviewing] -> [ready_to_test] at line 7; header retallied at line 5; Totals retallied at line 3; no BRAIN store - the verdict evidence file is the record
  stderr: backlog-mutate: note: no BRAIN store resolvable (no CYBEROS_STORE override, no .cyberos/memory/store under the root) - the verdict evidence file is the record; no status_overridden row appended (TASK-CUO-303)
== c03 route-back testing->ready_to_implement needs NO flags ==
  PASS c03
== c04 superset override implementing->done stays flag-free (exact-pair lock) ==
  PASS c04
== c05 evidence path missing -> 8 ==
  PASS c05
== c06 evidence empty file -> 8 ==
  PASS c06
== c07 evidence is a directory -> 8 ==
  PASS c07
== c08 --verdict-by empty -> 8 ==
  PASS c08
== c09 refusal precedence: pre-image drift beats the gate (6 not 8) ==
  PASS c09
== c10 refusal precedence: truth-precedes-index beats the gate (6 not 8) ==
  PASS c10
== c11 --json refusal envelope carries the verdict fields ==
  PASS c11
== c12 --json success envelope carries verdict fields + audit_row null (no store) ==
  PASS c12
== c13 store present: gated flip lands EXACTLY ONE status_overridden row ==
  PASS c13
  stdout: flip TASK-GUARD-001: [reviewing] -> [ready_to_test] at line 7; header retallied at line 5; Totals retallied at line 3; status_overridden row seq 1 appended (verdict by Stephen Cheng (CTO))
  verify: verify: OK - 1 row(s), HEAD=1, tip chain 70318eb9bab9...
== c14 second gated flip on same store chains seq 2 ==
  PASS c14
== c15 UNWRITABLE present store fails the flip (9), backlog unchanged ==
  PASS c15
== c16 CYBEROS_STORE override wins the resolution ==
  PASS c16
== c17 memory-append: complete status_overridden payload appends + verifies ==
  PASS c17
  stdout: append: seq 1 (status_overridden) chained at 1078d5b8f552... path meta/workflow/TASK-X-001.json HEAD=1
== c18 memory-append: each missing required field refuses 2 BEFORE any write ==
  PASS c18
  sample stderr: memory-append: status_overridden payload field 'reason' must be a non-empty string (STATUS-REFERENCE §1.4 verdict record carries {actor, task_id, prior_status, new_status, reason}) - refused before any write
== c19 memory-append: empty-string field refuses 2 ==
  PASS c19
== c20 memory-append: unknown kind still refused (closed set intact) ==
  PASS c20
== c21 memory-append: legacy four kinds unaffected (workflow_phase_complete) ==
  PASS c21
== c22 non-gate flip is byte-identical & verdict-free across two identical runs ==
  PASS c22
----
matrix: pass=22 fail=0
```

Additional spot checks (verbatim):

- exit-9 refusal wording on an unwritable present store:
  `backlog-mutate: flip TASK-GUARD-001: the status_overridden verdict row could not be appended to the BRAIN store at <scratch>/.cyberos/memory/store - Error: EACCES: permission denied, open '<scratch>/.cyberos/memory/store/.lock'. Audit-before-action (STATUS-REFERENCE §1.4, TASK-CUO-303): the index does not move without its audit row. Refusing; nothing written.` (flip exit 9, BACKLOG sha unchanged)
- flags on a NON-gate transition are ignored, not validated: `flip ready_to_implement implementing --verdict-by x --verdict-evidence <missing>` → exit 0.
- END-STATE via installed copies (`.cyberos/docs-tools/`): bare gate flip exit 8; flagged flip with fresh store appended seq 1 via the installed sibling appender; `memory-append.mjs verify` → `OK - 1 row(s), HEAD=1`.

## AC status

- AC 1 (bare refuse / flagged succeed / route-back flag-free): DONE — c01–c04, c22.
- AC 2 (evidence must exist + non-empty; empty actor): DONE — c05–c08.
- AC 3 (refusal precedence, 6 before 8): DONE — c09 (pre-image), c10 (truth guard).
- AC 4 (kind accepted; per-field refusal exit 2; unknown kind refused): DONE — c17–c21.
- AC 5 (audit-before-action: 1 row store-present, fail on unwritable, storeless note): DONE — c13–c16, c02.
- AC 6 (install.sh drops HITL_REQUIRED): DEFERRED — outside this worker's partition (see open item 1).
- AC 7 (ship-tasks.md flags docs + CHANGELOG breaking entry): DEFERRED — outside partition (open items 2–3).

## Decisions / deviations

1. **Exit 9 introduced** for "append failed on a present store". The spec pins only exit 8 and 1.2 defines 8 as strictly "otherwise legal, verdict missing"; folding append failure into 8 would conflate two failure modes. 9 is documented in header + `--help`.
2. **Store resolution**: the spec says "the same store-resolution the appender already implements", but the appender takes an explicit `<store-root>` argument. Implemented the memory protocol's §0.4 order (explicit `CYBEROS_STORE` override, else the repo-anchored `<root>/.cyberos/memory/store`), anchored to the SAME root whose backlog is flipped.
3. **Row identity**: record-level `actor` stays the appender's own resolution (`--actor`/`CYBEROS_ACTOR`/`doc-driven`) = who wrote the row; the human verdict actor is `extra.actor` per the spec's payload contract = who decided. Passing `--actor <verdict-by>` would have claimed the human operated the tool.
4. **Appender invocation**: `spawnSync(process.execPath, [memory-append.mjs, fixed argv], no shell)` — a direct child-process invocation, not a shell-out; the evidence path travels as payload data only and is never opened/executed/parsed (stat only).
5. **Evidence path** resolved against `--root` when relative (matches the existing `--backlog` convention); the row's `reason` carries the flag value verbatim.

## Open items for the HITL reviewer / final pass

1. **`tools/install/install.sh` (spec 1.5, AC 6)** — owned by the install sibling. Exact patch: delete line 319 (`HITL_REQUIRED="true"`), keep the prose comment at lines 317–318 ("HITL is required: the two human-acceptance gates … never automated"). No consumer exists (re-verified by repo grep 2026-07-23: only the install.sh copies).
2. **`modules/cuo/chief-technology-officer/workflows/ship-tasks.md` (spec 1.6, AC 7)** — this worker's grant covered only HITL_REQUIRED lines there, and the file contains none. Needed: the two HITL steps must document the flag-carrying flip (`--verdict-by <actor> --verdict-evidence <path>`) as how a recorded verdict advances the cell, plus one sentence stating the accepted frontmatter-edit/regen bypass residual (spec edge case).
3. **Root `CHANGELOG.md` (spec 1.6, AC 7)** — suggested top-entry text (also satisfies TASK-CUO-304's AC 5):

   ```
   Breaking
   - `backlog-mutate.mjs flip` now REFUSES the two human-acceptance gate transitions
     (`reviewing -> ready_to_test`, `testing -> done`) with exit code 8 unless a recorded
     human verdict accompanies the flip (`--verdict-by <actor>` + `--verdict-evidence
     <existing non-empty file>`) — breaking for tooling that automates those two bare
     flips (STATUS-REFERENCE §1.4; TASK-CUO-303). On a gated flip with a resolvable BRAIN
     store, one `status_overridden` audit row is appended before the index moves; a
     present store that cannot take the row fails the flip (exit 9).

   Changed
   - `cyberos-cuo drain --halt-on-repeat-rework` default 2 → 3 (`modules/cuo` api.py +
     cli.py), matching the ship-tasks.md §11b route-back ceiling: default drains now
     permit the third cycle before halting (TASK-CUO-304).

   Added
   - `memory-append.mjs` accepts the `status_overridden` kind with validated payload
     `{actor, task_id, prior_status, new_status, reason}` (TASK-CUO-303).
   - `modules/cuo/tests/test_doctrine_constants.py` pins the Python route-back-ceiling
     defaults to the number parsed from ship-tasks.md §11b (TASK-CUO-304).
   ```
4. **`tools/install/tests/test_hitl_lock.sh` (spec new_files)** — not created (outside partition). The manual matrix above covers the t01–t05 behaviors 1:1; t06 (gates.env) and t07 (docs/CHANGELOG) depend on open items 1–3. Case mapping: t01=c01–c04, t02=c05–c08, t03=c09, t04=c17–c21, t05=c13–c16+c02, t06=install.sh AC 6, t07=docs AC 7.
5. **Known-red siblings until flags are added** (spec modified_files anticipate this): `tools/install/tests/test_workflow_helpers.sh` `t15_flip_proceeds_when_truth_agrees` (~line 750-751) does a bare `reviewing -> ready_to_test` success flip → now exit 8; fix = add `--verdict-by`/`--verdict-evidence` with a fixture note file. `tools/install/tests/test_e2e_skeleton.sh` walks the LIFECYCLE (~line 75) through both gate transitions → the two gate steps need the flags. All refusal-path tests (t14/t16/t19 guard tests) still pass unchanged (proven by c09/c10 precedence).
6. **Vendored payload**: the install sibling / final pass should re-run `tools/install/build.sh` so the dist payload picks up both tool changes (no tracked dist copies exist; verified by glob).
