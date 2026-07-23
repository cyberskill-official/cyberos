---
task_id: TASK-IMP-137
audited: 2026-07-23
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_revision: 10/10
issues_resolved: 7
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean - `node tools/install/docs-tools/task-lint.mjs docs/tasks/improvement/TASK-IMP-137-install-portability-hardening/spec.md` exits 0 with zero findings, run against the audited revision
---

## §1 — Verdict summary

Seven §1 clauses, seven ACs, seven edge cases including a security-class row (this task is itself a security fix). The audit pressure fell on verify-don't-skip semantics in the checksum fallback, the atomicity claim's honest bounds (what a kill mid-swap can still do), and making the two breaking changes (engines, binding) carry explicit migration paths.

## §2 — Findings (all resolved)

### ISS-001 — the checksum fallback could have been implemented as a skip
"Fall back when sha256sum is absent" is satisfiable by skipping verification — the worst possible reading for the one security step in a curl|bash channel. Resolved: clause 1.3 states "the fallback MUST verify, not skip"; AC 3 asserts the fallback FAILS on a corrupted archive (the negative half that distinguishes verifying from skipping).

### ISS-002 — the atomicity claim was absolute in the first draft
"Atomic vendor" overclaims: `rm -rf old && mv staged` is two operations, not one atomic rename, and a kill between them still leaves a gap. Resolved: clause 1.6 bounds the claim honestly (window bounded by rename/move operations, not copy duration; stray staging cleanup at next install), and the edge case names the pathological kill-inside-the-pair remnant with its recovery story. AC 6 tests both the reader loop and the kill simulation at the stage-complete point.

### ISS-003 — empty-token semantics were undefined
`CYBEROS_MCP_TOKEN=""` could be read as "auth on, all requests fail" (bricking loopback use via a stray export) or "auth off". Resolved: edge case pins empty-as-unset with the rationale (an empty bearer is unusable as a credential) and requires the mcp README to document it.

### ISS-004 — token leakage via logs was unaddressed
A warning line that echoes the configured token (or a 401 body that reflects the attempted one) would put the secret in transcripts. Resolved: the security-class edge case requires that no token value is logged — the warning names the condition, never the secret.

### ISS-005 — binding assertion needed to be observable, not configuration-echo
Asserting "we passed 127.0.0.1 to listen()" tests the argument, not the exposure. Resolved: AC 1 asserts via the OS-reported bound address AND a refused non-loopback connect where the harness supports a secondary address — behavior, not configuration.

### ISS-006 — engines raise lacked a consumer migration statement
Raising the floor from `>=18` to `>=24 <25` is breaking for any consumer on older Node; the first draft noted "breaking" only generically. Resolved: edge case states the intended outcome (clear npm engines error) and requires the CHANGELOG to name the floor and the `.nvmrc` value; AC 7 asserts the breaking language.

### ISS-007 — the vendored `ci/` tree's uninstall story was unstated
Adding a new vendored tree without saying who removes it invites an uninstall leftover (the TASK-IMP-121/126 class). Resolved: Non-Goals states the tree is ownership-marked consistently so existing uninstall logic handles it, with TASK-IMP-126 named as the completeness owner.

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demand | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST bind loopback default; --host only opt-out; warn tokenless-wide | bound address + refused remote connect + wide-bind warning | AC 1: asserts all three | sufficient after revision (ISS-005) |
| 1.2 MUST 401 without exact Bearer; healthz open | tokenless 401, wrong 401, correct 200, healthz 200 | AC 2: asserts all four | sufficient |
| 1.3 MUST verify via fallback; abort when neither tool | fallback success AND corrupted-archive failure + neither-tool abort | AC 3: asserts all three including the verifying-negative | sufficient after revision (ISS-001) |
| 1.4 MUST declare >=24 <25 | exact engines string in scratch payload | AC 4: asserts exact match | sufficient |
| 1.5 MUST vendor ci/ + correct README | installed action.yml + uses: example + no stale claim | AC 5: asserts all three | sufficient |
| 1.6 MUST stage-then-swap; clean strays | zero reader absences over 20 reinstalls + kill leaves old tree + stray cleanup | AC 6: asserts all three | sufficient after revision (ISS-002) |
| 1.7 MUST record five changes, two breaking | five mentions + breaking language | AC 7: asserts both | sufficient |

## §4 — Resolution

Seven findings — two security-honesty, five material — all resolved in the audited revision. **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per `STATUS-REFERENCE.md` §1.1. The two human-acceptance gates in `/ship-tasks` are unchanged — this audit clears the spec-correctness gate only.

---

*End of TASK-IMP-137 audit.*
