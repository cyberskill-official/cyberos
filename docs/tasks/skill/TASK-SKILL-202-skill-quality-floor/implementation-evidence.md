# TASK-SKILL-202 — implementation evidence

Batch worker evidence, 2026-07-23, branch `batch/8-audit-hardening`. Shared-tree batch rules applied: this worker's write set was `modules/skill/**`, `tools/install/check-pair-parity.sh`, `scripts/tests/test_skill_stub_lint.sh`, and this file. Everything the spec demands OUTSIDE that set is enumerated under "Final-pass items" with exact edits. Nothing was committed; the final sequential pass owns commits and full verification.

## 1. What changed and why

### 1a. Injection-discipline backport — 20 skills (spec §1 #1.3, AC 3)

Each of the 20 enumerated gap skills received BOTH halves:

- `untrusted_inputs` frontmatter block (task-author pattern, byte-identical keys: `wrap_in_marker: "untrusted_content"`, `injection_scan: required`, `on_marker_hit: surface_to_human`), inserted with the house `# ── Untrusted-content discipline ──` rule directly before the frontmatter's closing `---`. All 20 frontmatters re-parse under PyYAML with `name` == dir name.
- `references/UNTRUSTED_CONTENT.md`, per-skill: a bespoke `§0 This skill's untrusted input surface` naming that skill's actual inputs and where the wrap happens (e.g. repo-context-map-author = arbitrary consumer-repo files, the canonical injection vector; coverage-gate-audit = the untruncated `raw_terminal` blocks), followed by the canonical §1–§7 module discipline (identical across skills by design — AC 3's edge case allows identical general sections, forbids full-file identity). Pairwise `cmp` over all 20 finds no byte-copies; each doc names its own skill.

The 20: plan-author, plan-audit, task-reconcile, workflow-improver, architectural-spike-{author,audit}, repo-context-map-{author,audit}, edge-case-matrix-{author,audit}, mock-contract-test-{author,audit}, observability-injection-{author,audit}, backlog-state-update-{author,audit}, coverage-gate-{author,audit}, debugging-cycle-{author,audit}.

### 1b. Pair-parity SCOPE expansion — 11 → 25 (spec §1 #1.4, AC 4)

`tools/install/check-pair-parity.sh` SCOPE now enumerates every author/audit pair in the vendored payload, in build.sh VENDORED_SKILLS lifecycle order; its comment records the design inversion (all pairs held; a vendored-but-unscoped pair is a defect, asserted mechanically by the new lint's t07).

**Measured fact that shrank this work item:** the spec's ISS-005 anticipated 14 newly-scoped pairs missing their class files. Measured at HEAD against `modules/skill/` (source), 13 of the 14 already carry every AUTHOR_CLASSES/AUDIT_CLASSES file — only the `plan` pair was missing all 12. The spec measured the built payload at `dist/cyberos/cuo/skills` (stale relative to source). Per the batch instruction, my own measurement prevails; the expanded checker passing 25/25 against source confirms it.

### 1c. plan-pair deepening — 12 new files (spec §1 #1.4)

Authored at parity with the deepened pairs (implementation-plan pair as model), not placeholders:

- `plan-author/`: `PIPELINE.md` (front-door pipeline: operator upstream, brownfield mid-run `repo-context-map-author scope: repo` sub-invocation, plan-audit + create-tasks downstream, NATS events, decision-gate/ambiguous-mode halts), `INVARIANTS.md` (INV-001..010 keyed to SKILL.md §2–§9 + plan_rubric rules), `envelopes/input.json` + `envelopes/output.json` (idea/repo_root in; gate_outcome/plan_path/memory_rows out; nullable artefact fields for ABORT/PENDING/MODE_HALT), `references/FAILURE_MODES.md` (canonical BOOT/drift/self-audit + PLAN-AUTHOR-001..004 skill-specific codes), `acceptance/README.md` (6-flow fixture catalog incl. ambiguous-halt and gate-abort).
- `plan-audit/`: `RUBRIC.md` (a BINDING to `../rubrics/plan_rubric.md` — rule-family map + constants, deliberately not restating the tables so `plan_rubric@1.0` stays the single authority, the same single-authority discipline as ISS-007), `AUDIT_LOOP.md` (canonical 8-step binding, PLAN-GATE-001 immediate-needs_human termination override, never-auto-fix rule for the three load-bearing PLAN rules), `REPORT_FORMAT.md` (issue-block/summary/re-entrancy/byte-stability, plan-specific values), `envelopes/{input,output}.json`, `acceptance/README.md` (8-flow catalog covering PLAN-OPT-001/PLAN-DEC-001/PLAN-OUT-001/PLAN-GATE-001/PLAN-SAFE-003/PLAN-SAFE-004/PLAN-SET-002 negatives).

### 1d. Skill-quality-floor lint — `scripts/tests/test_skill_stub_lint.sh` (spec §1 #1.5/#1.6, work item 4)

New suite, auto-registered by `run_all.sh`'s glob, bash-3.2-safe, offline, no payload build needed (parses the vendored set from build.sh's VENDORED_SKILLS heredoc — a property, never a hardcoded count or name list). Floor = SIZE (≥ 60 lines across SKILL.md + the TASK-SKILL-118 class files) AND STRUCTURE (≥ 2 `## ` sections in the SKILL.md body); `FLOOR_EXEMPT` list starts empty, additions are reviewed changes (spec §3 edge case). Tests: t01 good-fixture pass, t02 stub-fixture fail (both halves), t03 padded-fixture fail (structure half alone — padding cannot satisfy the floor), t04 live floor over the vendored set, t05 live both-halves injection-discipline presence, t06 the 20 backported docs are per-skill (pairwise non-identity + self-naming + §0 present), t07 parity-SCOPE completeness (SCOPE == vendored pair set, both directions).

## 2. Deviations from the spec letter (for the review gate)

1. **Floor metric.** Spec 1.5 says "fewer than 60 non-frontmatter lines" of the SKILL.md and claims the threshold is "far below any real skill". Measured 2026-07-23: ten real vendored skills carry 12–39 SKILL.md body lines (their contract mass lives in RUBRIC/AUDIT_LOOP/PIPELINE etc.), so the literal metric fails real skills. The lint therefore counts the skill's whole contract surface (SKILL.md + class files: stubs = 20–22 lines, smallest real = 163) and requires ≥ 2 body sections (stubs = 0, smallest real = 2). This makes the spec's own calibration claim true against the measured corpus; the spec's "required sections" wording ("MUST/MUST-NOT block") similarly fails architectural-spike-audit and was generalised to section-count structure.
2. **Lint location.** Spec names `tools/install/check-skill-floor.sh` + `tools/install/tests/test_skill_floor.sh` wired into build.sh. The batch ownership grant for this worker was `scripts/tests/test_skill_stub_lint.sh` (one new file); the floor logic lives there as `floor_check()`, written so the final pass can lift it verbatim into `tools/install/check-skill-floor.sh` and wire `build.sh` (item F4 below) if the build-time gate is still wanted on top of the suite-time gate.
3. **Backport count.** Plan said 21; spec measured 24 missing both halves (4 NFR stubs + 20). Source-tree measurement here confirms exactly 24/20. The 20 were backported; the 4 stubs are dispositioned by delisting, not backporting.
4. **Pair count.** Plan said 24; spec measured 25. Confirmed 25 (56 vendored dirs = 25 pairs + 6 singles: 4 NFR stubs, task-reconcile, workflow-improver).

## 3. Verification (verbatim output)

`bash tools/install/check-pair-parity.sh modules/skill` (direct run):

```
parity OK: 115 author dirs scanned, scope 25 pairs
exit=0
```

`bash tools/install/tests/test_pair_parity.sh` (existing TASK-SKILL-118 suite):

```
  ok   t01
  ok   t02
  ok   t03
  WARN t04: in-flight (uncommitted) changes on: repo-context-map-author repo-context-map-audit edge-case-matrix-author edge-case-matrix-audit mock-contract-test-author mock-contract-test-audit observability-injection-author observability-injection-audit backlog-state-update-author backlog-state-update-audit coverage-gate-author coverage-gate-audit debugging-cycle-author debugging-cycle-audit - removal check deferred to the committed state
  ok   t04
  ok   t05
  ok   t06
----
pass=6 fail=0
```

(t04's WARN is the suite's designed handling of uncommitted edits; the backport is purely additive — frontmatter insertion only, zero lines removed — so the committed-state check will pass at final-pass commit time.)

`bash scripts/tests/test_skill_stub_lint.sh` (direct run, PRE-delist — the two FAILs are the H7 finding, by design):

```
  ok   t01
  ok   t02
  ok   t03
         FLOOR nfr-certification-author: contract surface is 22 lines (< 60) - a name reservation, not a skill
         FLOOR nfr-certification-author: SKILL.md body has 0 '## ' sections (< 2) - no contract structure
         FLOOR nfr-evaluator: contract surface is 20 lines (< 60) - a name reservation, not a skill
         FLOOR nfr-evaluator: SKILL.md body has 0 '## ' sections (< 2) - no contract structure
         FLOOR nfr-test-runner: contract surface is 21 lines (< 60) - a name reservation, not a skill
         FLOOR nfr-test-runner: SKILL.md body has 0 '## ' sections (< 2) - no contract structure
         FLOOR nfr-regression-handler: contract surface is 22 lines (< 60) - a name reservation, not a skill
         FLOOR nfr-regression-handler: SKILL.md body has 0 '## ' sections (< 2) - no contract structure
         ^ nfr-* are the TASK-SKILL-202 H7 stubs: the delist (drop 4 names from build.sh VENDORED_SKILLS + 4 chain-allowlist.txt lines) is a FINAL-PASS item - this failure clears when it lands.
  FAIL t04: vendored skills under the floor: nfr-certification-author nfr-evaluator nfr-test-runner nfr-regression-handler
         nfr-* stubs are delisted by TASK-SKILL-202 (final-pass build.sh item), not backported - this failure clears when the delist lands.
  FAIL t05: missing untrusted_inputs frontmatter and/or references/UNTRUSTED_CONTENT.md: nfr-certification-author nfr-evaluator nfr-test-runner nfr-regression-handler
  ok   t06
  ok   t07
----
pass=5 fail=2
```

t04/t05 fail on EXACTLY the four vendored stubs and nothing else — the detector working as specified against the true pre-delist tree state. Applying final-pass item F1 flips the suite to 7/7.

Supporting checks: all 20 edited frontmatters parse (PyYAML) with the three required keys; `bash scripts/check_doc_anchors.sh` exits 0 (526 references resolved; WARNs are pre-existing historical-spec refs); doc §1–§7 headings byte-match the task-author exemplar with §0 as the only structural addition.

## 4. Final-pass items (edits OUTSIDE this worker's ownership — apply before running the full suite)

- **F1 `tools/install/build.sh`** (spec 1.1; a sibling also has in-flight edits to this file — apply by content, not line number): delete these four lines from the `VENDORED_SKILLS` heredoc:
  `nfr-certification-author                    # SDP 4  NFR (allowlisted unpaired)`, `nfr-evaluator                               # SDP 4  NFR`, `nfr-test-runner                             # SDP 4  NFR`, `nfr-regression-handler                      # SDP 4  NFR`. Source dirs stay in `modules/skill/` as unvendored scaffolds (spec's recorded decision).
- **F2 `tools/install/chain-allowlist.txt`** (spec 1.1, coupled with F1): delete the four `nfr-*` UNPAIRED exemption lines (lines 6–9; entries no payload dir matches trigger the file's own rot warning). `ship-tasks.md` and `check-chain-coverage.sh` contain no nfr references, so chain coverage stays green (AC 1).
- **F3 `modules/cuo/chief-technology-officer/workflows/certify-nfrs.md`** (spec 1.2): add the not-yet-shipped notice at the skill-routing step, e.g.: "NOTICE (TASK-SKILL-202): the four NFR skills (nfr-certification-author, nfr-evaluator, nfr-test-runner, nfr-regression-handler) are NOT yet shipped — they exist only as unvendored scaffolds in modules/skill/. This workflow requires their full implementation before it can run; do not improvise their outputs."
- **F4 (optional, spec 1.5's build-time gate)** `tools/install/check-skill-floor.sh` + wiring in build.sh next to the check-pair-parity call: lift `floor_check()` from `scripts/tests/test_skill_stub_lint.sh` verbatim, loop over `<skills-dir>/*/`. The suite-time gate already holds the floor repo-side; this adds it payload-side.
- **F5 `CHANGELOG.md`** (spec 1.7): top entry must name the delisting (superseding TASK-CUO-209's vendoring decision), the injection-discipline backport count (20), and the parity SCOPE expansion (11 → 25).
- **F6 payload rebuild + payload-side checks** (deferred here — a sibling owns dist/): after F1/F2, `bash tools/install/build.sh` then `bash tools/install/check-pair-parity.sh dist/cyberos/cuo/skills`; the stale pre-batch dist/ payload will fail parity until rebuilt (it lacks the plan-pair class files and all 20 backported docs).
- **F7 `docs/status/data/task/*.js`**: regenerated status data will pick up this file; not hand-edited (docs/status/** is outside every worker's set).

## 5. Open items for the HITL reviewer

- The two deviations in §2 (floor metric generalisation; lint location) are the substantive review questions. Both preserve the spec's intent and calibration claim against the measured corpus; both are recorded here rather than silently absorbed.
- `plan-author/INVARIANTS.md` documents the module-standard anomaly-signal set as intent: plan-author's frontmatter (untouched beyond the discipline block, per the spec's no-body-rewrite guardrail) does not yet declare a `self_audit` block. Adding one is a plan-pair follow-up, not this task.
- AC 6 (suite green under run_all.sh) holds structurally (glob-registered) but the suite reports 5/7 until F1/F2 land — deliberate, per §3 above.
- Not done, per batch rules: no commit, no BACKLOG/status edit, no BRAIN store write (operator instruction overrides the AGENT-ENTRY BRAIN-append rule for this batch), no payload rebuild, no full run_all.sh.
