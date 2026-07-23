# Benchmark gates G1–G16 — drift-prevention criteria (2026-07-23 deep audit)

Published home of the sixteen benchmark gates defined by the operator-approved 2026-07-23
deep audit and normalized in [TASK-IMP-140](../tasks/improvement/TASK-IMP-140-benchmark-gates-drift-protection/spec.md)
(this document is their published home, not a fork — same gates, same severities, same
tiers). A gate is a re-checkable pass/fail criterion that keeps an audit finding fixed
after attention moves on; the audit's issue register was point-in-time, these are not.

**One gate, one checker, one owner.** Nine of the sixteen checkers ship inside the sibling
hardening tasks whose behavior they verify; re-implementing them here would create two
authorities per gate and guarantee drift between them. This document maps ownership; the
`scripts/tests/test_benchmark_gates.sh` suite implements only the six gates no sibling
owns (G3, G4, G5, G6, G13, G16).

Field key —
**Severity**: how bad a regression is.
**Tier**: `ci` (fully automated) · `ci+human` (automated floor, periodic human judgment) ·
`detect+human` (automated detection, human decision).
**Owner**: the one checker that enforces the gate.

Run the locally-owned checkers: `bash scripts/tests/test_benchmark_gates.sh` (auto-registered
in `scripts/tests/run_all.sh` by its `test_*.sh` glob; also wired into CI by
`.github/workflows/caf-evals-gate.yml`'s `benchmark-gates` job, and rides the
run_all-in-CI job when TASK-IMP-128 lands).

---

## Status table

Per gate: whether its checker is live (enforcing) or report-only, and which task owns it.
This table is the coordination surface between sibling tasks (TASK-IMP-140 §1.3): a
checker whose gate measures a defect fixed by a not-yet-shipped sibling runs report-only
and flips to enforcing when the fix lands — the flip edits this table in the same change.

| Gate | Severity | Tier | Owning task | Checker | Mode |
|---|---|---|---|---|---|
| G1 | critical | ci | TASK-CUO-302 | `tools/install/tests/test_fail_closed_gates.sh` | live |
| G2 | critical | ci | TASK-CUO-303 | `tools/install/tests/test_hitl_lock.sh` | live |
| G3 | high | ci | TASK-IMP-140 | `scripts/tests/test_benchmark_gates.sh::t_g03` | live |
| G4 | medium | ci | TASK-IMP-140 | `scripts/tests/test_benchmark_gates.sh::t_g04` | live |
| G5 | high | ci | TASK-IMP-140 | `scripts/tests/test_benchmark_gates.sh::t_g05` | live |
| G6 | high | ci | TASK-IMP-140 | `scripts/tests/test_benchmark_gates.sh::t_g06` | live |
| G7 | high | ci+human | TASK-SKILL-202 | `tools/install/tests/test_skill_floor.sh` | live |
| G8 | high | ci+human | TASK-SKILL-202 | `tools/install/tests/test_skill_floor.sh::t03` | live |
| G9 | high | ci | TASK-MEMORY-303 | `tools/install/tests/test_doctor_gate.sh` | live |
| G10 | high | ci | TASK-MEMORY-303 | `modules/memory/tests/test_schema_single_source.py` | live |
| G11 | high | ci | TASK-CUO-304 | `modules/cuo/tests/test_doctrine_constants.py` | live |
| G12 | high | ci | TASK-IMP-139 | `scripts/tests/test_corpus_hygiene.sh::t02` | live |
| G13 | medium | detect+human | TASK-IMP-140 | `scripts/tests/test_benchmark_gates.sh::t_g13` | report-only (permanent, by tier) |
| G14 | high | ci | TASK-IMP-136 | `scripts/tests/test_ci_truth.sh` | live (run_all-in-CI half rides TASK-IMP-128, draft) |
| G15 | medium | ci+human | TASK-IMP-138 | `scripts/tests/test_entrypoint_identity.sh` | live |
| G16 | high | ci | TASK-IMP-140 | `scripts/tests/test_benchmark_gates.sh::t_g16` | live |

Sibling-owned rows (G1/G2/G7–G12/G14-in-part/G15) list the mode their owning batch/8 task
lands with; if a sibling slips, its row reads `pending (owning task in flight)` until it
ships — the checker, and therefore the enforcement, exists only inside that task.
TASK-IMP-140's own BRAIN-recording clause (spec §1.6) is NOT a gate mode: it is deferred
behind TASK-MEMORY-303's store repair and tracked in the task folder's
`brain-recording-checklist.md`.

---

## The sixteen gates

### G1 — Gate-floor non-vacuous

- **Purpose:** the machine-gate floor must never report GREEN having run nothing — vacuous
  green poisons both HITL gates downstream.
- **Pass/fail:** on a scratch install with an all-empty gate env, `run-gates.sh` exits
  non-zero (RED, distinct code) unless the operator sets the explicit empty-ack variable,
  which yields a distinct `EMPTY-ACKNOWLEDGED` line, never `GREEN`. Fail: exit 0 + GREEN
  on empty.
- **Severity:** critical
- **Tier:** ci
- **Test method:** scratch install; empty `gates.env`/config; assert exit code + output line.
- **Checked files:** `tools/install/gates/run-gates.sh`, `tools/install/install.sh`
  (autodetect + header).
- **Owner:** `tools/install/tests/test_fail_closed_gates.sh` (TASK-CUO-302).

### G2 — HITL mechanical lock

- **Purpose:** the two human-acceptance transitions must be refusable by the machine, not
  just forbidden by prose.
- **Pass/fail:** `backlog-mutate flip` refuses `reviewing->ready_to_test` and
  `testing->done` without a recorded verdict artifact (distinct exit code, no write); with
  `--verdict-by` + `--verdict-evidence` the flip succeeds and (store present) lands one
  `status_overridden` row. Fail: bare flip succeeds.
- **Severity:** critical
- **Tier:** ci (verdicts stay human)
- **Test method:** e2e flip attempts without/with verdict on a scratch backlog.
- **Checked files:** `tools/install/docs-tools/backlog-mutate.mjs`, `memory-append.mjs`.
- **Owner:** `tools/install/tests/test_hitl_lock.sh` (TASK-CUO-303).

### G3 — Status-enum single source

- **Purpose:** one canonical 12-value status enum; every consumer (rubric, lint, hub,
  templates) agrees byte-for-byte — the 10-vs-12 fork class.
- **Pass/fail:** the enum parsed from `STATUS-REFERENCE.md` §1 equals the sets in
  `RUBRIC.md` FM-104, `task-lint.mjs` STATUSES, `render-status-hub.mjs` STATUSES, and the
  vendored BACKLOG template's lifecycle + off-ramp vocabulary. Fail: any mismatch, extra,
  or missing value anywhere.
- **Severity:** high
- **Tier:** ci
- **Test method:** parse + set-compare across the five surfaces; mismatch message names
  surface and value.
- **Checked files:** `.cyberos/cuo/STATUS-REFERENCE.md` (source:
  `modules/skill/contracts/task/STATUS-REFERENCE.md` — the checker parses the tracked
  source; `.cyberos/` is an untracked install), `modules/skill/task-audit/RUBRIC.md`,
  `tools/install/docs-tools/task-lint.mjs`, `tools/docs-site/render-status-hub.mjs`,
  `tools/install/templates/BACKLOG.md`.
- **Owner:** `scripts/tests/test_benchmark_gates.sh::t_g03` (TASK-IMP-140).

### G4 — Headline-count truth

- **Purpose:** README headline counts (modules / workflows / tasks) stay measured, not
  remembered.
- **Pass/fail:** counts recomputed from the tree equal the numbers claimed in `README.md`
  and `docs/README.md`. Fail: any drift.
- **Severity:** medium
- **Tier:** ci
- **Test method:** recount (module dirs = `modules/*/`; workflow files =
  `modules/*/*/workflows/*.md`; task spec folders = `docs/tasks/*/TASK-*/spec.md`; task
  domains = `docs/tasks/*/` dirs containing at least one spec) and substring-compare. The
  failure message prints the measured numbers so the fix is a one-line doc edit.
- **Checked files:** `README.md`, `docs/README.md`.
- **Owner:** `scripts/tests/test_benchmark_gates.sh::t_g04` (TASK-IMP-140).

### G5 — Payload completeness

- **Purpose:** every path a vendored doc/skill references exists in the built payload —
  the unvendored-`skill-log.mjs` class, where a workflow promises a tool the install never
  delivers.
- **Pass/fail:** a reference walker over the built payload's markdown/scripts resolves
  every intra-payload path; zero missing. Fail: any referenced path absent.
- **Severity:** high
- **Tier:** ci
- **Test method:** build scratch payload (`tools/install/build.sh <scratch>` — never
  `dist/`); extract `.cyberos/<path>` references; stat each against the payload tree
  (extends the `check-chain-coverage.sh` approach). Illustrative paths that are examples,
  not promises, carry the inline exemption marker `benchmark-gates:exempt` on the same
  line — every exemption is visible in diffs and greppable (the allowlist-with-reasons
  pattern `chain-allowlist.txt` uses); runtime-generated paths (store, gates.env,
  config.yaml, session state) are excluded by a commented structural list in the checker.
- **Checked files:** `dist/cyberos/**` (built), `tools/install/build.sh` (vendor list).
- **Owner:** `scripts/tests/test_benchmark_gates.sh::t_g05` (TASK-IMP-140).

### G6 — Vendored-gate executability

- **Purpose:** the optional CAF/awh gate entry points must run in a consumer-shaped
  install — semantic exit codes, never "not found"/127 or a structurally-impossible path
  (the vendored-CAF ROOT-resolution class).
- **Pass/fail:** in a scratch install, invoking the vendored caf gate (and awh path when
  enabled) yields a semantic exit (pass/fail/skip-with-reason); never 127, never a
  missing-root error. Fail: any structural failure.
- **Severity:** high
- **Tier:** ci
- **Test method:** scratch install; invoke entry points with `CAF_ENABLED=true` fixtures
  (a minimal root `audit-profile.yaml` exercises the CLEAN path; its absence must produce
  the semantic FAIL-CLOSED exit, not a structural error).
- **Checked files:** `.cyberos/cuo/gates/caf/caf_gate.sh` (vendored), the seeded
  `CAF_CMD`/`AWH_CMD` in `gates.env`.
- **Owner:** `scripts/tests/test_benchmark_gates.sh::t_g06` (TASK-IMP-140).

### G7 — Skill quality floor

- **Purpose:** vendored skills meet a minimum body/section floor and every author/audit
  pair carries its file classes — no more ~20-line stubs shipped as product.
- **Pass/fail:** `check-skill-floor.sh` green over the payload AND `check-pair-parity.sh`
  green with SCOPE = every vendored pair. Fail: any undersized skill or missing class
  file. Human half: periodic prompt-quality review (not mechanical).
- **Severity:** high
- **Tier:** ci+human
- **Test method:** the two checkers against a scratch payload.
- **Checked files:** `dist/cyberos/cuo/skills/**`, `tools/install/check-pair-parity.sh`,
  `tools/install/check-skill-floor.sh`.
- **Owner:** `tools/install/tests/test_skill_floor.sh` (TASK-SKILL-202).

### G8 — Injection-discipline coverage

- **Purpose:** every repo-reading vendored skill declares `untrusted_inputs` + wrapping
  rules — prompt-injection posture is a floor, not a virtue of the best skills.
- **Pass/fail:** each repo/artefact-reading vendored skill carries the `untrusted_inputs`
  frontmatter block AND a non-empty per-skill `references/UNTRUSTED_CONTENT.md`. Fail:
  either half missing on any such skill. Human half: quality of the wrapping rules.
- **Severity:** high
- **Tier:** ci+human
- **Test method:** presence + shape scan over the payload skill set.
- **Checked files:** `dist/cyberos/cuo/skills/*/SKILL.md` + `references/`.
- **Owner:** `tools/install/tests/test_skill_floor.sh::t03` (TASK-SKILL-202).

### G9 — BRAIN health in gates

- **Purpose:** where memory is installed, store health is part of the machine floor — a
  frozen BRAIN silently dropping the audit trail is a gate matter, not a curiosity.
- **Pass/fail:** `run-gates.sh` runs `cyberos doctor` when the store + CLI exist (RED on
  doctor FAIL, provenance SKIP when absent), and the live store passes layout invariants.
  Fail: doctor absent from gates where memory exists, or store non-canonical.
- **Severity:** high
- **Tier:** ci
- **Test method:** three-state scratch-repo matrix (healthy store / violating store / no
  store).
- **Checked files:** `tools/install/gates/run-gates.sh`,
  `modules/memory/cyberos/core/invariants.py`, `.cyberos/memory/store/` layout.
- **Owner:** `tools/install/tests/test_doctor_gate.sh` (TASK-MEMORY-303).

### G10 — Schema copy consistency

- **Purpose:** every `memory.schema.json` copy byte-identical and the drift test pointed
  at real paths — the StoreAcl fork class, where consumers validate against a schema
  missing normative definitions.
- **Pass/fail:** root, package-data, and vendored copies hash-identical; generator
  `--check` green; the drift test executes (cannot skip on a missing path). Fail: any
  divergence or a skipping guard.
- **Severity:** high
- **Tier:** ci
- **Test method:** three-way hash + generator check + pytest collection assertion.
- **Checked files:** `modules/memory/memory.schema.json`,
  `modules/memory/cyberos/data/memory.schema.json`, payload `memory/memory.schema.json`,
  `modules/memory/tests/test_schema_drift.py`.
- **Owner:** `modules/memory/tests/test_schema_single_source.py` (TASK-MEMORY-303).

### G11 — Loop-bound single-sourcing

- **Purpose:** the loop constants doctrine names (route-back ceiling 3; debugging breaker
  5) match every machine encoding — the 3-vs-2 fork class.
- **Pass/fail:** the ceiling parsed from ship-tasks.md §11b (`routed_back_count >= 3`)
  equals the `api.py` default (`halt_on_repeat_rework`), the CLI default, and the help
  text; (breaker: pinned the day it gains a machine constant). Fail: any surface
  disagrees.
- **Severity:** high
- **Tier:** ci
- **Test method:** doctrine-parsing conformance test; loud failure on parse miss.
- **Checked files:** `modules/cuo/chief-technology-officer/workflows/ship-tasks.md`,
  `modules/cuo/cuo/api.py`, `modules/cuo/cuo/cli.py`.
- **Owner:** `modules/cuo/tests/test_doctrine_constants.py` (TASK-CUO-304).

### G12 — UNREVIEWED hygiene

- **Purpose:** no non-draft spec carries `# UNREVIEWED` — compliance fields on shipped
  work are confirmed or the task is not shipped (FM-112, enforced corpus-wide instead of
  per-lint-invocation).
- **Pass/fail:** corpus scan finds zero non-draft spec.md files containing the marker.
  Fail: any hit.
- **Severity:** high
- **Tier:** ci
- **Test method:** status-aware grep over `docs/tasks/*/TASK-*/spec.md`.
- **Checked files:** the task corpus.
- **Owner:** `scripts/tests/test_corpus_hygiene.sh::t02` (TASK-IMP-139).

### G13 — Stuck-WIP detection

- **Purpose:** in-flight statuses older than a threshold are surfaced for operator
  triage — eleven tasks sat in `implementing` for ten weeks with nothing noticing.
- **Pass/fail:** the detector lists every task in an in-flight status (`implementing`,
  `ready_to_review`, `reviewing`, `ready_to_test`, `testing`) whose last recorded
  transition (or `created_at` fallback) exceeds N=30 days; the list is report output,
  never an automatic status change. Fail (of the gate itself): detector absent or silent
  on a constructed stale fixture.
- **Severity:** medium
- **Tier:** detect+human
- **Test method:** corpus scan + fixture with a backdated in-flight task. Age = the newest
  of (`created_at`, last git commit touching the spec when real history is available); the
  threshold is configurable via `CYBEROS_G13_THRESHOLD_DAYS` (default 30). The gate's
  pass/fail is about the DETECTOR existing and speaking, not about any particular task
  being stale — detection automated, decision human.
- **Checked files:** the task corpus; threshold configurable, default 30 days.
- **Owner:** `scripts/tests/test_benchmark_gates.sh::t_g13` (TASK-IMP-140; report-only).

### G14 — CI parity & stub honesty

- **Purpose:** the offline suite runs in root CI, the CAF fixtures validate in root CI,
  and no always-green stub workflow exists — green means checked, everywhere a check is
  named.
- **Pass/fail:** a root workflow invokes `run_all.sh`; a root workflow invokes the CAF
  eval validator + `caf_precommit_check.sh`; zero workflows carry the stub placeholder
  marker; no stub-named check is branch-protection-required. Fail: any half missing.
- **Severity:** high
- **Tier:** ci
- **Test method:** workflow-content asserts + placeholder scan (+ operator-side protection
  query documented in the owning task's `stub-disposition.md`).
- **Checked files:** `.github/workflows/**`, `.pre-commit-config.yaml` (or its absence),
  `.githooks/pre-commit`.
- **Owner:** `scripts/tests/test_ci_truth.sh` (TASK-IMP-136; run_all-in-CI half
  via TASK-IMP-128).

### G15 — Entry-point consistency

- **Purpose:** pointer files across all tools reference one workflow spine and the
  platform-vs-consumer identity of root AGENTS.md is explicit — an agent's first file load
  must reach task/HITL law.
- **Pass/fail:** root `AGENTS.md` reaches task law within its first 30 lines; every
  pointer file names `.cyberos/AGENT-ENTRY.md`; exactly one normative protocol source
  exists with all copies self-declaring. Fail: any invariant broken. Human half: the
  Branch A/B identity decision itself (TASK-IMP-138's fork).
- **Severity:** medium
- **Tier:** ci+human
- **Test method:** line-window + marker greps.
- **Checked files:** `AGENTS.md`, `CLAUDE.md`, `.cursorrules`, `.cursor/rules/cyberos.mdc`,
  `GEMINI.md`, `.github/copilot-instructions.md`, `.windsurfrules`,
  `tools/install/install.sh`.
- **Owner:** `scripts/tests/test_entrypoint_identity.sh` (TASK-IMP-138).

### G16 — Idempotent reinstall

- **Purpose:** install -> reinstall produces an equivalent `.cyberos/` (modulo timestamped
  backups) and never silently degrades operator config — the observed C1 wipe (a working
  TEST_CMD surviving only in a `.bak`) can never recur unnoticed.
- **Pass/fail:** on a scratch repo: two consecutive installs diff clean modulo the
  documented backup files; a pre-set `.cyberos/config.yaml` override survives
  byte-identical; no reader-visible vendored-tree absence during the loop. Fail: any
  silent config degradation or divergent tree.
- **Severity:** high
- **Tier:** ci
- **Test method:** double-install diff + config-survival assert + reader poll (builds on
  `test_e2e_skeleton.sh` and TASK-IMP-137's atomic vendor). The diff-exclusion list lives
  in the checker with a comment per entry (timestamped `gates.env.bak.*` churn and nothing
  silently more), so "modulo timestamps" cannot grow into "modulo everything".
- **Checked files:** `tools/install/install.sh`, `.cyberos/` (scratch),
  `tools/install/tests/test_e2e_skeleton.sh`.
- **Owner:** `scripts/tests/test_benchmark_gates.sh::t_g16` (TASK-IMP-140; the reader-gap
  half also covered by TASK-IMP-137's t06).
