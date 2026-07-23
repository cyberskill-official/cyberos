---
task_id: TASK-CUO-302
audited: 2026-07-23
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_revision: 10/10
issues_resolved: 6
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean - `node tools/install/docs-tools/task-lint.mjs docs/tasks/cuo/TASK-CUO-302-fail-closed-gates/spec.md` exits 0 with zero findings, run against the audited revision
---

## §1 — Verdict summary

Six §1 clauses, six ACs, six edge cases including a security-class row. Every clause traces 1:1 to an AC via `traces_to`; every factual claim in Problem/source_pages was re-verified against the working tree during authoring (fail-open exit path read in source, all-empty local `gates.env` confirmed, header/regen-notice contradiction confirmed at `install.sh:299` vs `:326`). The two consequential findings were an unpinned exit code and undefined env-var value semantics — both would have shipped ambiguity into the exact automation surface (G1 checker) this task exists to serve.

## §2 — Findings (all resolved)

### ISS-001 — RED-on-empty exit code was unpinned (spec said only "non-zero")
`run-gates.sh` already uses exit 1 (gate failure) and exit 2 (missing gates.env / malformed config.yaml). A bare "non-zero" for the empty floor would collide with both, and TASK-IMP-140's G1 checker needs to distinguish "gates ran and failed" from "nothing configured" mechanically. Resolved: clause 1.1 pins exit 3 and names why it is distinct; AC 1 asserts the exact code.

### ISS-002 — escape-hatch value semantics were undefined (TRACE-006-adjacent)
"`CYBEROS_ALLOW_EMPTY_GATES=1` is set" left `=true`, `=yes`, `=0` undefined — an operator exporting `=true` would get RED and reasonably call it a bug, or worse, a sloppy implementation would accept any non-empty value and `=0` would acknowledge-empty. Resolved: clause 1.1 requires the literal `1` and requires every other value to behave as unset; AC 1 asserts `=true` and `=0` still exit 3.

### ISS-003 — non-execution of fallback probes needed an observable test method
Clause 1.4 demands the fallback MUST NOT execute probe targets at install time, but the first AC draft asserted only that the command string was seeded — a test that cannot see execution cannot verify a MUST NOT. Resolved: AC 4 asserts "install runs neither probe target", verifiable via sentinel fixtures (probe scripts that write a marker file when executed; the test asserts the marker is absent).

### ISS-004 — acknowledged-empty was distinguishable only by exit code in the first draft
If the ack path printed the normal `GATES: GREEN` line, log readers (and the ship-tasks transcript) could not tell an acknowledged-empty run from a real green run — the exact conflation C1 is about. Resolved: clause 1.3 requires the distinct `GATES: EMPTY-ACKNOWLEDGED` line AND the absence of `GATES: GREEN`; AC 3 asserts both halves.

### ISS-005 — fallback probe order was stated but not contractual
With both `scripts/tests/run_all.sh` and a `Makefile` present, the seeded command depended on implementation order. Resolved: clause 1.4 fixes the ordered, closed probe list; the edge-case section adds the both-present fixture; AC 4 asserts run_all wins.

### ISS-006 — stale installed trees (old header text) vs new behavior needed an explicit posture
An upgraded payload enforces RED-on-empty while the consumer's `gates.env` still carries the old "edit freely" header until the next install — a reader could conclude enforcement waits on the header. Resolved: edge case states behavior ships with the vendored `run-gates.sh` regardless of header vintage; stale-header-only trees are acceptable, stale behavior is not.

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demand | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST exit 3 on empty floor; other env values behave as unset | exact exit code on empty; unchanged semantics when configured; literal-1 gate | AC 1: exit==3 empty, ==0/==1 configured, `=true`/`=0` still 3 | sufficient |
| 1.2 MUST name both fixes + hatch | three actionable substrings present in RED output | AC 2: asserts all three substrings | sufficient |
| 1.3 MUST print distinct ack line, not GREEN | positive line present AND green line absent | AC 3: asserts both halves | sufficient |
| 1.4 MUST seed fallback with provenance; MUST NOT execute | seeded value + SRC_TEST provenance + non-execution + precedence | AC 4: asserts seed, provenance, sentinel non-execution, run_all-beats-Makefile | sufficient after revision (ISS-003, ISS-005) |
| 1.5 MUST NOT say "edit freely"; MUST state machine-owned | negative substring + positive replacement in generated file | AC 5: asserts both halves against a scratch install | sufficient |
| 1.6 MUST gain CHANGELOG entry | positive content present in top entry | AC 6: asserts three required substrings | sufficient |

## §4 — Resolution

Six findings, all material, all resolved in the audited revision. **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per `STATUS-REFERENCE.md` §1.1. The two human-acceptance gates in `/ship-tasks` (review acceptance, final acceptance) are unchanged and remain recorded human verdicts — this audit clears the spec-correctness gate only.

---

*End of TASK-CUO-302 audit.*
