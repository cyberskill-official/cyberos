---
id: TASK-IMP-140
title: Benchmark gates G1-G16 - checkers, risk register, BRAIN recording
template: task@1
type: improvement
module: improvement
status: implementing
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-23T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-MEMORY-303]
blocks: []
related_tasks: [TASK-CUO-302, TASK-CUO-303, TASK-CUO-304, TASK-IMP-136, TASK-SKILL-202, TASK-IMP-137, TASK-IMP-138, TASK-IMP-139, TASK-IMP-128]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.1.0"
owner: Stephen Cheng (CTO)
created: 2026-07-23
memory_chain_hash: null
effort_hours: 16
service: docs/verification + scripts/tests
new_files:
  - docs/verification/benchmark-gates.md
  - scripts/tests/test_benchmark_gates.sh
modified_files:
  - docs/reference/risk-register.md
  - CHANGELOG.md
source_pages:
  - "2026-07-23 CyberOS deep-audit conversation (operator-approved; the G1-G16 definitions and the risk entries originate there and are embedded in full below so this spec is self-contained)"
  - "docs/reference/risk-register.md:11 (numbering contract: 'RSK-01 through RSK-15 are the canonical top risks; R-EXT-* additions are inferred from project context and marked with their rationale' - the audit entries extend as R-EXT-* rows)"
  - "measured 2026-07-23: docs/verification/ exists (caf-absorption-design.md etc.) and has no benchmark-gates.md; scripts/tests/run_all.sh discovers suites by glob so a new test_benchmark_gates.sh is auto-registered"
  - "AGENTS.md §13 (end-of-response BRAIN reporting) + AGENT-ENTRY.md #4 (record decisions, audits, and plans into the BRAIN) - the audit is currently recorded nowhere durable because the live store is FROZEN_RECOVERABLE until TASK-MEMORY-303's layout repair (measured: stray adrs/ + impl-plans/ fail layout-root-canonical)"
  - "checker ownership measured against this authoring wave: G1<-TASK-CUO-302 test, G2<-TASK-CUO-303 test, G7/G8<-TASK-SKILL-202 checkers, G9/G10<-TASK-MEMORY-303, G11<-TASK-CUO-304 pin test, G12<-TASK-IMP-139 suite, G14<-TASK-IMP-136 test_ci_truth.sh (+TASK-IMP-128 for run_all-in-CI), G15<-TASK-IMP-138 suite, G16<-TASK-IMP-137 t06 + test_e2e_skeleton.sh - leaving G3, G4, G5, G6, G13 and the G16 full-idempotency half as this task's own checkers"
source_decisions:
  - "2026-07-23 operator: CyberOS Hardening Plan approved; Phase 3 'Benchmark gates + drift protection' authored as ONE improvement task per the coordinator's instruction (plan file cyberos_hardening_plan_49404998)."
  - "2026-07-23 authoring: gates whose checkers ship inside sibling hardening tasks are NOT re-implemented here - benchmark-gates.md maps each gate to its owning checker, and this task's suite implements only the unowned gates (G3, G4, G5, G6, G13, G16-full). One gate, one checker, one owner; the doc is the index."
  - "2026-07-23 authoring: the BRAIN recording step is gated on TASK-MEMORY-303 via depends_on (reciprocal blocks entry on 303) because §12 forbids writes on a store that fails invariants - recording the audit into a frozen store would itself violate the protocol the audit measured."
---

# TASK-IMP-140: Benchmark gates G1-G16 - checkers, risk register, BRAIN recording

## Summary

The 2026-07-23 deep audit defined sixteen benchmark gates - drift-prevention criteria that make its findings re-checkable forever instead of a one-time snapshot. This task lands them as product: `docs/verification/benchmark-gates.md` carrying all sixteen definitions (embedded in full below, so this spec is self-contained); automated checkers for the six gates no sibling hardening task owns (G3 enum cross-check, G4 headline counts, G5 payload reference walker, G6 vendored-gate smoke, G13 stuck-WIP detector, G16 reinstall idempotency), wired into CI through the existing suite-discovery glob; the audit's seven risk entries appended to `docs/reference/risk-register.md` as R-EXT-* rows; and the audit + its decisions recorded into the BRAIN per §13 - a step that depends on TASK-MEMORY-303 unfreezing the live store first.

## Problem

The audit's issue register is point-in-time: every finding it verified (fail-open gates, enum forks, payload gaps, schema drift...) regrows the moment attention moves, because nothing mechanical re-measures. The audit therefore defined G1-G16 with pass/fail criteria and automation tiers - but they exist only in a conversation transcript. Meanwhile `docs/reference/risk-register.md` predates the audit (its newest rows are project-context R-EXT entries, none covering self-approval, vacuous gates, or the config-wipe class the audit demonstrated), and the audit itself is recorded in no durable store: the BRAIN is FROZEN_RECOVERABLE (layout failure), so the repo's own doctrine - record decisions and audits into the BRAIN - is currently unsatisfiable.

## Proposed Solution

Author `docs/verification/benchmark-gates.md` from the embedded definitions below: one section per gate (purpose, pass/fail, severity, test method, automation tier, checked files, owning checker), plus a status table the checkers can be diffed against. Implement `scripts/tests/test_benchmark_gates.sh` with one function per unowned gate - t_g03 (parse the status enum from STATUS-REFERENCE.md §1 and compare byte-for-byte against RUBRIC.md FM-104's list, `task-lint.mjs` STATUSES, `render-status-hub.mjs` STATUSES, and the vendored BACKLOG template's off-ramp vocabulary), t_g04 (recompute module/workflow/task counts and compare against README.md + docs/README.md headline claims), t_g05 (walk every path reference in vendored docs/skills against the built payload's file set - the skill-log.mjs class), t_g06 (scratch-install smoke: the vendored caf/awh gate entry points exit with semantic codes, never 127/not-found), t_g13 (report-only detector: in-flight statuses older than N=30 days emitted as a triage list; detection automated, decision human), t_g16 (install -> reinstall on a scratch repo diffs `.cyberos/` clean modulo timestamped backups AND preserves a pre-set config.yaml override - the C1 wipe class). The suite auto-registers via `run_all.sh`'s glob and therefore rides TASK-IMP-128's CI job and the TASK-IMP-136 workflow when those land. Append the seven audit risk entries to the risk register as R-EXT rows with the register's full field set. Finally - after TASK-MEMORY-303 lands - record the audit verdict, the sixteen gates, and the hardening-wave decisions into the BRAIN as memory files + chained audit rows per §13, closing the loop the frozen store left open.

## Alternatives Considered

- **Sixteen checkers in this task (re-implement the owned ones too).** Rejected: G1/G2/G7-G12/G14/G15 checkers ship inside the sibling tasks whose behavior they verify; duplicating them here creates two authorities per gate and guarantees drift between them. The doc maps ownership; this task fills only the gaps.
- **benchmark-gates.md generated from the transcript verbatim.** Rejected: the transcript is untrusted-by-protocol prose (§11) and phrased for a conversation; the doc needs testable pass/fail wording and checked-file lists an implementer can act on. The definitions below are the transcript's content normalized to that shape - same gates, same severities, same tiers.
- **A new CI workflow for the gate suite.** Rejected: `run_all.sh`'s glob is the repo's documented registration mechanism ("the glob IS the registration"); a dedicated workflow would be a second wiring to forget. CI arrival rides IMP-128/IMP-136, which own CI wiring.
- **Record the audit to the BRAIN now, store frozen or not.** Rejected: §12 forbids writes below READY; §1's pre-write checklist halts at state verification. Recording into a failing store would make the audit's own record a protocol violation - the dependency on TASK-MEMORY-303 is the honest sequencing.
- **Fold the risk entries into a new document.** Rejected: `docs/reference/risk-register.md` exists, has the R-EXT extension convention documented in its own header, and is reviewed in the founder weekly sync - extending it puts the audit risks where the eyes already are.

## Success Metrics

- Primary: by the next CyberOS release - benchmark-gates.md exists with all sixteen gates in the normalized shape; `test_benchmark_gates.sh` green in `run_all.sh` with its six checkers each demonstrably failing on a constructed violation; the risk register carries the seven new R-EXT rows; and (post TASK-MEMORY-303) the BRAIN holds the audit record with chained rows. Baseline today: zero of the four exist.
- Guardrail: `run_all.sh` stays green corpus-wide (the six new checkers must pass against the CURRENT repo the day they land - any gate that measures a not-yet-fixed defect lands as report-only until its owning task ships, and the doc's status table says which mode each gate is in).

## Scope

In scope: benchmark-gates.md, the six-checker suite, the risk-register extension, the BRAIN recording step, CHANGELOG.

### Out of scope / Non-Goals

- The checkers owned by sibling tasks (mapped in the doc; see source_decisions) and any change to those tasks' scopes.
- The status-hub UI for G13 (the detector here is a report-only test output; the hub sentinel is the v3.x roadmap item).
- Fixing any defect a checker finds - checkers measure; the owning tasks fix. A checker that fails on an unshipped fix runs report-only per the guardrail metric.
- Editing the fifteen existing RSK rows or re-scoring them.

## Dependencies

`depends_on: [TASK-MEMORY-303]` - the BRAIN recording step (clause 1.6) requires the store un-frozen by 303's layout repair (303 carries the reciprocal `blocks` entry). The dependency gates only that clause: the doc, checkers, and register rows are implementable immediately, and the task may ship its non-BRAIN clauses while 303 is in flight per the slice plan, but final acceptance includes the recording. Soft references (no cycles): TASK-IMP-128 + TASK-IMP-136 carry the gate suite into CI; TASK-CUO-302/303/304, TASK-SKILL-202, TASK-IMP-137/138/139 own the mapped checkers.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS `task-author` skill in Cursor, as the task-authoring wave of the 2026-07-23 hardening plan.
- **Scope:** the G1-G16 definitions were extracted from the operator-approved audit conversation and normalized (not re-derived); every checked-file path and ownership mapping below was verified against the working tree at authoring time; the risk-register numbering convention was read from the register's own header.
- **Human review:** the hardening plan (Phase 3 scope: benchmark-gates.md, checkers, risk register, BRAIN recording) was operator-approved 2026-07-23.

## Gate definitions (G1-G16) - embedded normative reference

The content contract for `docs/verification/benchmark-gates.md`. Severity: how bad a regression is. Tier: `ci` (fully automated), `ci+human` (automated floor, periodic human judgment), `detect+human` (automated detection, human decision). Owner: the checker that enforces it.

### G1 - Gate-floor non-vacuous (severity: critical, tier: ci)
- **Purpose:** the machine-gate floor must never report GREEN having run nothing - vacuous green poisons both HITL gates downstream.
- **Pass/fail:** on a scratch install with an all-empty gate env, `run-gates.sh` exits non-zero (RED, distinct code) unless the operator sets the explicit empty-ack variable, which yields a distinct `EMPTY-ACKNOWLEDGED` line, never `GREEN`. Fail: exit 0 + GREEN on empty.
- **Test method:** scratch install; empty `gates.env`/config; assert exit code + output line.
- **Checked files:** `tools/install/gates/run-gates.sh`, `tools/install/install.sh` (autodetect + header).
- **Owner:** `tools/install/tests/test_fail_closed_gates.sh` (TASK-CUO-302).

### G2 - HITL mechanical lock (severity: critical, tier: ci; verdicts stay human)
- **Purpose:** the two human-acceptance transitions must be refusable by the machine, not just forbidden by prose.
- **Pass/fail:** `backlog-mutate flip` refuses `reviewing->ready_to_test` and `testing->done` without a recorded verdict artifact (distinct exit code, no write); with `--verdict-by` + `--verdict-evidence` the flip succeeds and (store present) lands one `status_overridden` row. Fail: bare flip succeeds.
- **Test method:** e2e flip attempts without/with verdict on a scratch backlog.
- **Checked files:** `tools/install/docs-tools/backlog-mutate.mjs`, `memory-append.mjs`.
- **Owner:** `tools/install/tests/test_hitl_lock.sh` (TASK-CUO-303).

### G3 - Status-enum single source (severity: high, tier: ci)
- **Purpose:** one canonical 12-value status enum; every consumer (rubric, lint, hub, templates) agrees byte-for-byte - the 10-vs-12 fork class.
- **Pass/fail:** the enum parsed from `STATUS-REFERENCE.md` §1 equals the sets in `RUBRIC.md` FM-104, `task-lint.mjs` STATUSES, `render-status-hub.mjs` STATUSES, and the vendored BACKLOG template's off-ramp vocabulary. Fail: any mismatch, extra, or missing value anywhere.
- **Test method:** parse + set-compare across the five surfaces; mismatch message names surface and value.
- **Checked files:** `.cyberos/cuo/STATUS-REFERENCE.md` (source: `modules/skill/contracts/task/STATUS-REFERENCE.md`), `modules/skill/task-audit/RUBRIC.md`, `tools/install/docs-tools/task-lint.mjs`, `tools/docs-site/render-status-hub.mjs`, `tools/install/templates/BACKLOG.md`.
- **Owner:** `scripts/tests/test_benchmark_gates.sh::t_g03` (this task).

### G4 - Headline-count truth (severity: medium, tier: ci)
- **Purpose:** README headline counts (modules / workflows / tasks) stay measured, not remembered.
- **Pass/fail:** counts recomputed from the tree equal the numbers claimed in `README.md` and `docs/README.md`. Fail: any drift.
- **Test method:** recount (module dirs, workflow files, task spec folders) and substring-compare.
- **Checked files:** `README.md`, `docs/README.md`.
- **Owner:** `scripts/tests/test_benchmark_gates.sh::t_g04` (this task).

### G5 - Payload completeness (severity: high, tier: ci)
- **Purpose:** every path a vendored doc/skill references exists in the built payload - the unvendored-`skill-log.mjs` class, where a workflow promises a tool the install never delivers.
- **Pass/fail:** a reference walker over the built payload's markdown/scripts resolves every intra-payload path; zero missing. Fail: any referenced path absent.
- **Test method:** build scratch payload; extract path references; stat each against the payload tree (extends the `check-chain-coverage.sh` approach).
- **Checked files:** `dist/cyberos/**` (built), `tools/install/build.sh` (vendor list).
- **Owner:** `scripts/tests/test_benchmark_gates.sh::t_g05` (this task).

### G6 - Vendored-gate executability (severity: high, tier: ci)
- **Purpose:** the optional CAF/awh gate entry points must run in a consumer-shaped install - semantic exit codes, never "not found"/127 or a structurally-impossible path (the vendored-CAF ROOT-resolution class).
- **Pass/fail:** in a scratch install, invoking the vendored caf gate (and awh path when enabled) yields a semantic exit (pass/fail/skip-with-reason); never 127, never a missing-root error. Fail: any structural failure.
- **Test method:** scratch install; invoke entry points with `CAF_ENABLED=true` fixtures; classify exits.
- **Checked files:** `.cyberos/cuo/gates/caf/caf_gate.sh` (vendored), the seeded `CAF_CMD`/`AWH_CMD` in `gates.env`.
- **Owner:** `scripts/tests/test_benchmark_gates.sh::t_g06` (this task).

### G7 - Skill quality floor (severity: high, tier: ci+human)
- **Purpose:** vendored skills meet a minimum body/section floor and every author/audit pair carries its file classes - no more ~20-line stubs shipped as product.
- **Pass/fail:** `check-skill-floor.sh` green over the payload AND `check-pair-parity.sh` green with SCOPE = every vendored pair. Fail: any undersized skill or missing class file. Human half: periodic prompt-quality review (not mechanical).
- **Test method:** the two checkers against a scratch payload.
- **Checked files:** `dist/cyberos/cuo/skills/**`, `tools/install/check-pair-parity.sh`, `tools/install/check-skill-floor.sh`.
- **Owner:** `tools/install/tests/test_skill_floor.sh` (TASK-SKILL-202).

### G8 - Injection-discipline coverage (severity: high, tier: ci+human)
- **Purpose:** every repo-reading vendored skill declares `untrusted_inputs` + wrapping rules - prompt-injection posture is a floor, not a virtue of the best skills.
- **Pass/fail:** each repo/artefact-reading vendored skill carries the `untrusted_inputs` frontmatter block AND a non-empty per-skill `references/UNTRUSTED_CONTENT.md`. Fail: either half missing on any such skill. Human half: quality of the wrapping rules.
- **Test method:** presence + shape scan over the payload skill set.
- **Checked files:** `dist/cyberos/cuo/skills/*/SKILL.md` + `references/`.
- **Owner:** `tools/install/tests/test_skill_floor.sh::t03` (TASK-SKILL-202).

### G9 - BRAIN health in gates (severity: high, tier: ci)
- **Purpose:** where memory is installed, store health is part of the machine floor - a frozen BRAIN silently dropping the audit trail is a gate matter, not a curiosity.
- **Pass/fail:** `run-gates.sh` runs `cyberos doctor` when the store + CLI exist (RED on doctor FAIL, provenance SKIP when absent), and the live store passes layout invariants. Fail: doctor absent from gates where memory exists, or store non-canonical.
- **Test method:** three-state scratch-repo matrix (healthy store / violating store / no store).
- **Checked files:** `tools/install/gates/run-gates.sh`, `modules/memory/cyberos/core/invariants.py`, `.cyberos/memory/store/` layout.
- **Owner:** `tools/install/tests/test_doctor_gate.sh` (TASK-MEMORY-303).

### G10 - Schema copy consistency (severity: high, tier: ci)
- **Purpose:** every `memory.schema.json` copy byte-identical and the drift test pointed at real paths - the StoreAcl fork class, where consumers validate against a schema missing normative definitions.
- **Pass/fail:** root, package-data, and vendored copies hash-identical; generator `--check` green; the drift test executes (cannot skip on a missing path). Fail: any divergence or a skipping guard.
- **Test method:** three-way hash + generator check + pytest collection assertion.
- **Checked files:** `modules/memory/memory.schema.json`, `modules/memory/cyberos/data/memory.schema.json`, payload `memory/memory.schema.json`, `modules/memory/tests/test_schema_drift.py`.
- **Owner:** `modules/memory/tests/test_schema_single_source.py` (TASK-MEMORY-303).

### G11 - Loop-bound single-sourcing (severity: high, tier: ci)
- **Purpose:** the loop constants doctrine names (route-back ceiling 3; debugging breaker 5) match every machine encoding - the 3-vs-2 fork class.
- **Pass/fail:** the ceiling parsed from ship-tasks.md §11b equals the `api.py` default, the CLI default, and the help text; (breaker: pinned the day it gains a machine constant). Fail: any surface disagrees.
- **Test method:** doctrine-parsing conformance test; loud failure on parse miss.
- **Checked files:** `modules/cuo/chief-technology-officer/workflows/ship-tasks.md`, `modules/cuo/cuo/api.py`, `modules/cuo/cuo/cli.py`.
- **Owner:** `modules/cuo/tests/test_doctrine_constants.py` (TASK-CUO-304).

### G12 - UNREVIEWED hygiene (severity: high, tier: ci)
- **Purpose:** no non-draft spec carries `# UNREVIEWED` - compliance fields on shipped work are confirmed or the task is not shipped (FM-112, enforced corpus-wide instead of per-lint-invocation).
- **Pass/fail:** corpus scan finds zero non-draft spec.md files containing the marker. Fail: any hit.
- **Test method:** status-aware grep over `docs/tasks/*/TASK-*/spec.md`.
- **Checked files:** the task corpus.
- **Owner:** `scripts/tests/test_corpus_hygiene.sh::t02` (TASK-IMP-139).

### G13 - Stuck-WIP detection (severity: medium, tier: detect+human)
- **Purpose:** in-flight statuses older than a threshold are surfaced for operator triage - eleven tasks sat in `implementing` for ten weeks with nothing noticing.
- **Pass/fail:** the detector lists every task in an in-flight status (`implementing`, `ready_to_review`, `reviewing`, `ready_to_test`, `testing`) whose last recorded transition (or `created_at` fallback) exceeds N=30 days; the list is report output, never an automatic status change. Fail (of the gate itself): detector absent or silent on a constructed stale fixture.
- **Test method:** corpus scan + fixture with a backdated in-flight task.
- **Checked files:** the task corpus; threshold configurable, default 30 days.
- **Owner:** `scripts/tests/test_benchmark_gates.sh::t_g13` (this task; report-only).

### G14 - CI parity & stub honesty (severity: high, tier: ci)
- **Purpose:** the offline suite runs in root CI, the CAF fixtures validate in root CI, and no always-green stub workflow exists - green means checked, everywhere a check is named.
- **Pass/fail:** a root workflow invokes `run_all.sh`; a root workflow invokes the CAF eval validator + `caf_precommit_check.sh`; zero workflows carry the stub placeholder marker; no stub-named check is branch-protection-required. Fail: any half missing.
- **Test method:** workflow-content asserts + placeholder scan (+ operator-side protection query documented in the owning task).
- **Checked files:** `.github/workflows/**`, `.pre-commit-config.yaml` (or its absence), `.githooks/pre-commit`.
- **Owner:** `scripts/tests/test_ci_truth.sh` (TASK-IMP-136; run_all-in-CI half via TASK-IMP-128).

### G15 - Entry-point consistency (severity: medium, tier: ci+human)
- **Purpose:** pointer files across all tools reference one workflow spine and the platform-vs-consumer identity of root AGENTS.md is explicit - an agent's first file load must reach task/HITL law.
- **Pass/fail:** root `AGENTS.md` reaches task law within its first 30 lines; every pointer file names `.cyberos/AGENT-ENTRY.md`; exactly one normative protocol source exists with all copies self-declaring. Fail: any invariant broken. Human half: the Branch A/B identity decision itself (TASK-IMP-138's fork).
- **Test method:** line-window + marker greps.
- **Checked files:** `AGENTS.md`, `CLAUDE.md`, `.cursorrules`, `.cursor/rules/cyberos.mdc`, `GEMINI.md`, `.github/copilot-instructions.md`, `.windsurfrules`, `tools/install/install.sh`.
- **Owner:** `scripts/tests/test_entrypoint_identity.sh` (TASK-IMP-138).

### G16 - Idempotent reinstall (severity: high, tier: ci)
- **Purpose:** install -> reinstall produces an equivalent `.cyberos/` (modulo timestamped backups) and never silently degrades operator config - the observed C1 wipe (a working TEST_CMD surviving only in a `.bak`) can never recur unnoticed.
- **Pass/fail:** on a scratch repo: two consecutive installs diff clean modulo the documented backup files; a pre-set `.cyberos/config.yaml` override survives byte-identical; no reader-visible vendored-tree absence during the loop. Fail: any silent config degradation or divergent tree.
- **Test method:** double-install diff + config-survival assert + reader poll (builds on `test_e2e_skeleton.sh` and TASK-IMP-137's atomic vendor).
- **Checked files:** `tools/install/install.sh`, `.cyberos/` (scratch), `tools/install/tests/test_e2e_skeleton.sh`.
- **Owner:** `scripts/tests/test_benchmark_gates.sh::t_g16` (this task; the reader-gap half also covered by TASK-IMP-137's t06).

## Risk-register extension (content contract for the seven R-EXT rows)

Each row carries the register's full field set (description, cause, impact, detection, prevention, recovery, automation tier), sourced from the audit: (1) **Self-approval / skipped HITL** - no mechanical lock; unreviewed work marked done; detect: audit chain lacks verdict rows; prevent: G2; recover: reconcile + `done -> ready_to_review` flip. (2) **Vacuous green gates** - fail-open floor + autodetect unknown; false confidence at both HITL gates; prevent: G1; recover: re-run with restored config. (3) **Config wipe on reinstall** - gates.env regeneration; silent loss of the only test command (observed); prevent: G16 + durable config.yaml; recover: `.bak` restore. (4) **Prompt injection via repo files** - repo-reading skills without untrusted discipline; steered workflows in consumer repos; prevent: G8; detect: injection-marker scan. (5) **Payload/doc divergence** - docs referencing unvendored files; consumer workflows halt on missing tools; prevent: G5. (6) **Partial install window** - rm/cp vendor step; broken `.cyberos/` mid-install; prevent: staged swap (TASK-IMP-137) + G16; recover: re-run install. (7) **BRAIN frozen-by-layout** - store pollution; protocol-compliant agents must refuse writes, audit trail silently absent; prevent: G9; recover: operator-gated move (TASK-MEMORY-303).

## 1. Description (normative)

- 1.1 `docs/verification/benchmark-gates.md` MUST carry all sixteen gate definitions in the normalized shape above (purpose, pass/fail, severity, tier, test method, checked files, owner), plus a status table stating per gate whether its checker is live or report-only and which task owns it. The doc MUST match the embedded definitions in this spec (same gates, same severities, same tiers) - it is their published home, not a fork.
- 1.2 `scripts/tests/test_benchmark_gates.sh` MUST implement the six unowned checkers (t_g03 enum cross-check, t_g04 headline counts, t_g05 payload reference walker, t_g06 vendored-gate smoke, t_g13 stuck-WIP report, t_g16 reinstall idempotency) with each checker demonstrably failing on a constructed violation, and MUST register through `run_all.sh`'s existing glob (no hand-list edit).
- 1.3 A checker whose gate measures a defect fixed by a sibling task that has not yet shipped MUST run in report-only mode (loud output, non-failing) with the doc's status table saying so, flipping to enforcing in the same change that lands the fix or in a one-line follow-up when the sibling ships first. The suite MUST be green on the repo the day it lands.
- 1.4 `t_g13` MUST NOT change any task's status - it emits the stale-WIP list for operator triage only (detection automated, decision human, per the gate's tier).
- 1.5 `docs/reference/risk-register.md` MUST gain the seven audit rows as `R-EXT-*` entries following the register's documented numbering and field conventions, each naming its preventing gate(s) and recovery path per the content contract above.
- 1.6 After TASK-MEMORY-303's repair lands (the `depends_on` edge), the audit verdict, the sixteen gate definitions (by reference to the doc), and the hardening-wave decisions MUST be recorded into the BRAIN as memory files through the canonical writer with chained audit rows, and the session's §13 end-of-response block MUST report the write. This clause MUST NOT execute against a store that `cyberos doctor` reports below READY.
- 1.7 `CHANGELOG.md` MUST record the four deliverables (doc, suite, register rows, BRAIN record).

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - benchmark-gates.md exists with sixteen `### G` sections each carrying the seven fields and the status table; a field-completeness scan finds zero gaps, and the doc's severities/tiers match this spec's - test: `scripts/tests/test_benchmark_gates.sh::t01_doc_complete_and_consistent`
- [ ] AC 2 (traces_to: #1.2) - each of the six checkers passes on the repo and fails on its constructed violation fixture (six negative fixtures, one per checker) - test: `scripts/tests/test_benchmark_gates.sh::t02_checkers_fail_on_violations`
- [ ] AC 3 (traces_to: #1.3) - the suite exits green at HEAD; any report-only gate prints its report block and the doc's status table names it report-only - test: `scripts/tests/test_benchmark_gates.sh::t03_green_at_head_reportonly_declared`
- [ ] AC 4 (traces_to: #1.4) - running t_g13 against a fixture corpus with a backdated implementing task lists it and leaves every spec file byte-identical - test: `scripts/tests/test_benchmark_gates.sh::t04_g13_reports_never_mutates`
- [ ] AC 5 (traces_to: #1.5) - the register carries exactly seven new R-EXT rows, each with all seven fields non-empty and at least one G-reference - test: `scripts/tests/test_benchmark_gates.sh::t05_risk_rows_complete`
- [ ] AC 6 (traces_to: #1.6) - post-303: the BRAIN holds the audit record (memory files present, chain verify green, doctor READY before and after), demonstrated on the live store at final acceptance and on a fixture store in CI - test: `scripts/tests/test_benchmark_gates.sh::t06_brain_record_fixture`
- [ ] AC 7 (traces_to: #1.7) - CHANGELOG's top entry names all four deliverables - test: `scripts/tests/test_benchmark_gates.sh::t07_changelog_four_deliverables`

## 3. Edge cases

- **TASK-MEMORY-303 slips:** clauses 1.1-1.5 + 1.7 are independent and ship; the task holds short of final acceptance until 1.6 executes on the un-frozen store - the depends_on edge plus 1.6's READY-precondition make the sequencing mechanical, not mannered.
- **A sibling task lands AFTER this suite (checker exists before the fix):** 1.3's report-only mode is the designed state; the flip-to-enforcing is part of the sibling's landing checklist via the doc's status table - the table is the coordination surface, so neither task needs the other's timeline.
- **G4 counts change legitimately (new module, new tasks):** the checker recomputes both sides; only the README claim can be stale. The failure message prints the measured numbers so the fix is a one-line doc edit - measured truth stays cheap to restore.
- **G5 false positives on illustrative paths:** the walker honors an explicit inline exemption marker (documented in the doc) for paths that are examples, not promises; every exemption is visible in the diff and greppable - the same allowlist-with-reasons pattern `chain-allowlist.txt` uses.
- **G13 threshold tuning:** N=30 days is the default judgment, configurable via env in the checker; the gate's pass/fail is about the DETECTOR existing and speaking, not about any particular task being stale.
- **G16 vs machine-local files:** the double-install diff excludes the documented machine-local set (timestamped `.bak`s, `docs/tasks/.workflow/` run-state) - the exclusion list lives in the checker with a comment per entry, so "modulo timestamps" cannot silently grow into "modulo everything".
- **Security-class:** checkers are read-only against the repo + scratch dirs (G6/G16 execute only the repo's own vendored scripts in throwaway installs); the BRAIN write goes through the canonical writer under its lock + ACL discipline; no new secrets, no network.
