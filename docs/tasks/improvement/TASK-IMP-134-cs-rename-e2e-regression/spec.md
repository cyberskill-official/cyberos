---
id: TASK-IMP-134
title: End-to-end regression for the cs/memory/cuo rename
template: task@1
type: improvement
module: improvement
status: done
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-22T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-IMP-130, TASK-IMP-131, TASK-IMP-132]
blocks: []
related_tasks: [TASK-IMP-107]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.9"
owner: Stephen Cheng (CTO)
created: 2026-07-22
memory_chain_hash: null
effort_hours: 3
service: tools/install
new_files:
  - tools/install/tests/test_cs_rename_e2e.sh
modified_files:
  - (none)
source_pages:
  - "docs/plans/PLAN-cli-module-namespacing-2026-07-22/plan.md §6 item 7 ('End-to-end regression test on a clean machine ... confirm cs <top-level cmd> works, cyberos cleanly no longer resolves to the public installer CLI, and cs memory <cmd> reaches the memory module without collision')"
  - "tools/install/tests/test_e2e_skeleton.sh:1-19 (existing e2e precedent: builds from SOURCE into a scratch dir, explicitly NO network/credentials/model — the established offline e2e philosophy in this repo's own test suite)"
  - "TASK-IMP-133 audit ISS-005 (this same batch): no task requires an actual npm release be cut and published, and this task's own 'clean machine' framing in the plan implies real network + a real Homebrew install, which conflicts with test_e2e_skeleton.sh's offline convention — reconciled below by splitting this task into an offline in-repo portion and a manual release-time portion"
  - "TASK-IMP-131 and TASK-IMP-132 Dependencies sections (this same batch): both flagged a sibling-coordination/merge-conflict risk from editing the same cli.mjs dispatch table in parallel — this task's distinct value beyond 130/131/132's own per-task ACs is proving the COMBINED dispatch table works as an integrated whole, not each verb in isolation"
source_decisions:
  - "2026-07-22 Stephen: create-tasks PLAN gate — APPROVE as rendered."
  - "2026-07-22 authoring: the plan's phrase 'clean machine' regression, read literally, implies real network access, a real npm release, and a real Homebrew tap update — none of which fit this repo's own established e2e-testing convention (test_e2e_skeleton.sh is explicitly offline, no network, no credentials). Rather than author an AC this repo's CI cannot actually run, split this task into (a) an offline, in-repo, CI-automatable integration test proving the combined dispatch table (cs + memory + cuo together) is coherent, and (b) a manual, release-time checklist for the genuinely network/Homebrew-dependent portion — matching the same honesty applied in TASK-IMP-133 rather than presenting an unrunnable AC as if it were automated."
  - "2026-07-22 self-audit revision (score_pre_revision 5/10 -> score_post_revision 10/10): AC 1 and AC 6 both originally deferred to 'code review' / 'manual review of this document' for checks that are actually straightforwardly mechanical (a grep on the test file's own source for build-call count and ordering; a grep on this spec file's own text for the required heading) - both converted to grep-based checks. Clause 1.1 and its AC left the single-shared-build requirement as a general principle without citing the concrete existing precedent (test_sync_host_plugins.sh's file-level build-once pattern) an implementer should copy - added the citation directly into the clause. AC 3's ten-verb check risked false positives from incidental substring matches - tightened to word-boundary-safe matching. depends_on originally included TASK-IMP-133, which would block this task's fully-automated, immediately-implementable offline portion behind an externally-gated release event that TASK-IMP-133's own audit (ISS-005) already found has no owner in this batch - removed 133 from depends_on, keeping it only as a soft prerequisite for the non-gating manual checklist. The guardrail metric's 'announced as complete' trigger was subjective - retied to a concrete, checkable event (the release containing TASK-IMP-130's own CHANGELOG entry being tagged)."
---

# TASK-IMP-134: End-to-end regression for the cs/memory/cuo rename

## Summary

Prove the combined result of TASK-IMP-130/131/132 — the renamed `cs` bin with `memory` and `cuo` both added to the same dispatch table — works as an integrated whole via an offline, in-repo test, and separately checklist the genuinely network/Homebrew-dependent "clean machine" portion the plan describes, which cannot run in this repo's CI.

## Problem

TASK-IMP-130, 131, and 132 each have their own acceptance criteria proving their own change in isolation. None of them proves the three changes compose correctly once all land — in particular, TASK-IMP-131 and TASK-IMP-132 both edit the same `cli.mjs` dispatch table object and were flagged in their own specs as having a sibling merge-conflict risk (each depends only on TASK-IMP-130, not on each other, so nothing stops them being authored/implemented in parallel). An e2e test that builds the payload fresh after all four tasks land and exercises `cs install`, `cs memory`, and `cs cuo` together in one process is the only check that would catch a dispatch-table integration break that per-task unit tests would miss.

Separately, the plan's own item 7 describes a "clean machine" regression involving a real Homebrew install and a real npm release — this repo's only existing e2e precedent, `test_e2e_skeleton.sh`, is explicit that it runs "with NO model, NO network, NO credentials." The plan's literal ask does not fit that convention, and TASK-IMP-133's own audit (ISS-005) already found that no task in this batch actually covers cutting the release this regression would need. Presenting a network-dependent AC as if this repo's CI could run it would be the same class of fabricated-citation defect TASK-IMP-133's audit found and fixed.

## Proposed Solution

Two halves, not one:

1. **Offline integration test (this task's own automatable deliverable).** A new test, `tools/install/tests/test_cs_rename_e2e.sh`, following `test_e2e_skeleton.sh`'s established shape (build from source into a scratch dir, no network) that builds a scratch payload after TASK-IMP-130/131/132's changes are present, then in one continuous run: confirms the generated `package.json`'s `bin` is `cs` (not `cyberos`); confirms `cs -h`'s usage text lists `memory` and `cuo` alongside `install`/`uninstall`/`version`/`status`/`create`/`gates`/`mcp`/`help`; confirms `cs memory <cmd>` and `cs cuo <name>` both dispatch correctly using the same stub techniques TASK-IMP-131/132's own tests established, run back-to-back against the SAME built payload (proving the combined `SCRIPTS`-table object is well-formed, not just each entry individually reachable).
2. **Manual release-time checklist (not an automated AC).** A short checklist, recorded in this task's own body rather than as a testable AC, for whoever performs the actual release: install the released `cs` via the updated Homebrew formula (TASK-IMP-133) on a real machine, confirm `cyberos` (bare) resolves only to a locally pip-installed `cyberos-memory` if present (no competing public-CLI claim on that name), and confirm `cs memory <cmd>` reaches the same store. This half is explicitly NOT gated by this task's own `ready_to_implement` → `done` transition, since it cannot happen until a real release exists (the same operational gap TASK-IMP-133's audit named).

## Alternatives Considered

- Author a single AC requiring a real Homebrew install on a clean VM, matching the plan's literal wording. Rejected: this repo's CI has no such capability and no prior task in this codebase (including `test_e2e_skeleton.sh`, the closest precedent) does this — authoring an AC nothing can mechanically satisfy would be the same fabricated-test-authority problem TASK-IMP-133's audit caught and fixed, repeated here.
- Skip the offline portion entirely and treat this task as pure documentation/checklist. Rejected: the dispatch-table integration risk (TASK-IMP-131/132's sibling merge-conflict flag) is real and IS testable offline today; dropping it would leave the one distinctly valuable, achievable check this task can add unbuilt.
- Fold this task's offline portion into TASK-IMP-131 or 132 individually instead of a separate task. Rejected: neither of those tasks' own scope is "prove the combination of both plus 130 works together" — that is inherently a task that can only exist once all three have landed, matching the plan's own dependency ordering (item 7 listed last, after items 1-6).

## Success Metrics

- Primary: `bash tools/install/tests/test_cs_rename_e2e.sh` passes in CI, offline, after TASK-IMP-130/131/132 land. Baseline today: no such test exists, and none of the three per-task test suites runs all three verbs against one shared built payload in sequence.
- Guardrail: the manual release-time checklist (this task's §3 edge cases / body text) is followed at least once before the release containing TASK-IMP-130's own CHANGELOG rename entry (clause 1.6 of that task) is tagged and published - a concrete, checkable trigger event rather than a subjective "announced as complete" - tracked as a release-process step, not as this task's own `done` criterion.

## Scope

In scope: `tools/install/tests/test_cs_rename_e2e.sh` (new), and the manual release-time checklist recorded in this spec's body.

### Out of scope / Non-Goals

- Any code change to `cli.mjs`, `build.sh`, or the memory/cuo dispatch logic itself — this task only tests what TASK-IMP-130/131/132 already implement.
- Actually performing the manual release-time checklist — that is a release-process action, not a deliverable this task's own status transition depends on.
- Cutting the npm release or merging the Homebrew tap PR — those are TASK-IMP-130's and TASK-IMP-133's concerns respectively (and, per TASK-IMP-133's audit ISS-005, not fully owned by any task in this batch).

## Dependencies

Depends on TASK-IMP-130, 131, and 132 — the automated offline portion (clauses 1.1-1.5) only needs their CODE to exist, not TASK-IMP-133's completion. TASK-IMP-133 is deliberately NOT in `depends_on`: its own audit (ISS-005) established that it cannot reach `done` until an npm release is actually cut and published, an externally-gated, indefinite-duration event. Making this task's `depends_on` include TASK-IMP-133 would block this task's own immediately-implementable, fully-automated portion behind that external gate for no reason — only the MANUAL checklist in Edge Cases (clause 1.6, not gated by this task's status transitions) actually needs TASK-IMP-133 to have shipped. Nothing in this batch depends on this task (`blocks: []`).

**Relationship to TASK-IMP-107.** That task built `test_e2e_skeleton.sh`, the offline e2e precedent this task's own test file follows structurally (build-from-source into scratch, no network, no credentials) — including its file-level pattern of building ONE scratch payload before any check function runs, which this task's own clause 1.1 depends on directly (see §1.1's revised wording).

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS `task-author` skill inside Cowork.
- **Scope:** the offline-vs-network split was authored after reading `test_e2e_skeleton.sh`'s actual header comment and cross-referencing TASK-IMP-133's audit finding (ISS-005) authored earlier in this same batch, not asserted independently.
- **Human review:** task decomposition approved at the 2026-07-22 PLAN gate. The offline/manual split is an authoring-time call, flagged for the operator in the batch report.

## 1. Description (normative)

- 1.1 `tools/install/tests/test_cs_rename_e2e.sh` MUST build exactly one scratch payload from source, at file level before any check function is defined or called — matching `test_sync_host_plugins.sh`'s own established pattern ("building scratch payload..." once, consumed by every `tNN_*` function below it) — and every check below MUST consume that SAME built payload; no check function may perform its own independent build.
- 1.2 The test MUST confirm the built payload's `package.json` declares `bin: {"cs": ...}` and does NOT declare a `cyberos` bin key.
- 1.3 The test MUST confirm `cs -h`'s usage output lists all of `install`, `uninstall`, `version`, `status`, `create`, `gates`, `mcp`, `help`, `memory`, and `cuo`.
- 1.4 The test MUST confirm, against the ONE built payload, that both `cs memory <cmd>` (via a stub `python3`) and `cs cuo <name>` (via the redirect-stub check) behave correctly when invoked back-to-back in the same test run.
- 1.5 The test MUST run offline — no network access, no real npm registry query, no real Homebrew invocation — matching `test_e2e_skeleton.sh`'s own established convention.
- 1.6 This task's spec MUST record the manual release-time checklist as body text, explicitly marked as not gating this task's own `ready_to_implement` → `done` transition.

## 2. Acceptance criteria

- [x] AC 1 (traces_to: #1.1) - a grep of `tools/install/tests/test_cs_rename_e2e.sh`'s own source for build invocations (`bash "$BUILD"` or equivalent) returns exactly `1`, and that invocation appears before the first `t0[0-9]_` function definition in the file - test: `grep -c 'bash "\$BUILD"' tools/install/tests/test_cs_rename_e2e.sh` returns `1`, combined with a line-number comparison against the first `t0` function definition
- [x] AC 2 (traces_to: #1.2) - `$TMP/payload/package.json`'s `bin` object has key `cs` and does not have key `cyberos` - test: `tools/install/tests/test_cs_rename_e2e.sh::t02_bin_is_cs_only`
- [x] AC 3 (traces_to: #1.3) - `node $TMP/payload/cli/bin/cli.mjs -h` output contains all ten command names listed in clause 1.3, matched with word-boundary-safe patterns (e.g. `grep -wo`) so that a substring coincidence (e.g. "create" inside a hypothetical unrelated "created") cannot produce a false pass - test: `tools/install/tests/test_cs_rename_e2e.sh::t03_usage_lists_all_ten_verbs`
- [x] AC 4 (traces_to: #1.4) - within the same test run, a stub-`python3` `cs memory doctor` call and a `cs cuo plan` call both produce their expected outputs, run in sequence against the one built payload from AC 1-3 - test: `tools/install/tests/test_cs_rename_e2e.sh::t04_memory_and_cuo_both_work_on_shared_build`
- [x] AC 5 (traces_to: #1.5) - the test file contains no invocation of `curl`, `npm view`, `npm install <remote>`, or `brew` anywhere in its body - test: `grep -Ec 'curl |npm view|npm install [^-]|brew ' tools/install/tests/test_cs_rename_e2e.sh` returns `0`
- [x] AC 6 (traces_to: #1.6) - a grep of THIS spec file for the literal heading text `Manual release-time checklist` returns at least `1` match - test: `grep -c 'Manual release-time checklist' docs/tasks/improvement/TASK-IMP-134-cs-rename-e2e-regression/spec.md` returns `>=1`

## 3. Edge cases

- If TASK-IMP-131 or TASK-IMP-132 ships with a different exact dispatch-table shape than either task's own spec describes (e.g. a routing function instead of a flat object key), this test's assertions about `cli.mjs`'s internal structure (if any are added beyond black-box CLI invocation) would need updating - mitigated by keeping all assertions black-box (invoke the CLI, read stdout/exit code) rather than parsing `cli.mjs`'s source, so implementation-detail changes in HOW the dispatch is built don't break this test as long as the observable behaviour matches.
- **Manual release-time checklist** (not gated by this task's own acceptance criteria — see clause 1.6 and Scope): (1) confirm the npm release referenced by the updated Homebrew Formula (TASK-IMP-133) is live via `npm view @cyberskill/cyberos version`; (2) on a machine with `cyberos-memory` separately pip-installed, `brew install cyberos-cli` then confirm `cyberos` (bare) resolves to the pip-installed memory CLI and `cs` resolves to the Homebrew-installed public CLI, with neither shadowing the other; (3) confirm `cs memory doctor` on that same machine reaches the real BRAIN store and its output matches what `cyberos doctor` (the direct invocation) would show for the same store path.
- A machine with NEITHER `cyberos` nor `cs` previously installed (the actual "clean machine" the plan's wording literally describes): the manual checklist's step 2 is vacuously about "no collision" since there is nothing pre-existing to collide with - the meaningful clean-machine claim is narrower than the plan's wording suggests, and is really about a machine that has BOTH the old internal tool and the new public one, which is the founder's own originally-reported scenario (plan §2).
- Security-class: this task adds a new offline test file only; it introduces no new runtime code path and no new attack surface.
