# TASK-IMP-140 — implementation evidence (batch/8, 2026-07-23)

Implemented mid-wave under the batch/8 shared-tree partition (5 sibling workers editing
other parts of this tree; no commits, no full-suite runs, no payload/dist rebuilds, no
memory-store writes from this worker — a final sequential pass owns those). HITL: both
human-acceptance gates remain ahead; final acceptance additionally requires the deferred
§1.6 BRAIN recording (below).

## What changed and why

| File | Change | Why (spec clause) |
|---|---|---|
| `docs/verification/benchmark-gates.md` | NEW — all sixteen gate definitions in the normalized 7-field shape + the 16-row status table (live/report-only + owning task per gate) | §1.1; the definitions existed only in the audit conversation |
| `scripts/tests/test_benchmark_gates.sh` | NEW — the six unowned checkers (`t_g03` enum, `t_g04` counts, `t_g05` payload walker, `t_g06` vendored-gate smoke, `t_g13` stuck-WIP report, `t_g16` reinstall idempotency), root/target-parameterized, plus meta-tests t01–t07; auto-registers via `run_all.sh`'s glob | §1.2–§1.4; sibling-owned gates (G1/G2/G7–G12/G14/G15) deliberately NOT re-implemented (one gate, one checker, one owner) |
| `docs/reference/risk-register.md` | EXTENDED — new section `## R-EXT extensions — 2026-07-23 deep audit` with R-EXT-01..07, each carrying description/cause/impact/detection/prevention/recovery/automation-tier + G-references | §1.5; first R-EXT rows carried on the markdown page itself (pre-audit rows live only in the site page's client data) |
| `brain-record.sh` + `brain-recording-checklist.md` (this folder) | NEW — ready-to-run §1.6 recording: doctor/state READY guard (refuses below READY, exit 2, no write), three `put`s through the canonical writer (`--kind decisions`), chain `verify`, doctor after, §13 block printed | §1.6 + the batch/8 carve-out: recording is HARD-DEFERRED behind TASK-MEMORY-303's operator-gated store repair |
| `.github/workflows/caf-evals-gate.yml` (TASK-IMP-136 file) | the `benchmark-gates` job runs this suite in CI | spec's CI-arrival sentence (rides IMP-136's workflow + IMP-128's run_all job via the glob) |
| `CHANGELOG.md` | NOT edited (outside this worker's ownership) | §1.7 — paste-ready text below |

## Checker design notes (for the reviewer)

- **G3** parses the STATUS-REFERENCE §1 tables as canonical and set-compares FM-104,
  both `STATUSES` arrays, and the BACKLOG template's lifecycle+off-ramp vocabulary.
  Parenthesized qualifiers are stripped before token extraction (drops `type: bug` /
  `duplicate_of` qualifiers without a hand-kept exception list). Checked at the tracked
  SOURCE paths — `.cyberos/` is an untracked install and absent in CI checkouts.
- **G4** recomputes: modules = `modules/*/` (26), workflows = `modules/*/*/workflows/*.md`
  (224), tasks = `docs/tasks/*/TASK-*/spec.md`, domains = task dirs containing ≥1 spec
  (29; `_archive/`/`_audits/` excluded by the rule).
- **G5** builds a scratch payload (`build.sh <tmp>` — never `dist/`), extracts
  `.cyberos/<path>` references, honors the inline `benchmark-gates:exempt` marker, and
  carries a commented structural-exclusion list (AGENT-ENTRY.md is install-generated;
  store/gates.env/config.yaml/sessions are runtime; docs/tasks is consumer-owned).
- **G6** smoke: seeded `CAF_CMD` names the vendored script; `caf_gate.sh .` with no
  profile must exit the SEMANTIC fail-closed 1 (never 127/2/"not vendored"); with a
  trivial root `audit-profile.yaml` (`RUN_COMMANDS: true`) must exit CLEAN 0.
- **G13** age = newest of `created_at` and the last commit whose diff CHANGED the
  `status:` value (`git log --follow -G'^status:' -p` + value compare) — a corpus-wide
  prose polish or the 07-14 tree rename must not reset a stuck task's clock; a shallow
  checkout falls back to `created_at`. Report-only by tier: it lists, it never fails the
  suite on findings, it never writes (t04 proves byte-identical corpus).
- **G16** double-install diff over `.cyberos/` with a one-entry commented exclusion list
  (`gates.env.bak.<epoch>` churn), pre-set `config.yaml` byte-survival (the C1 wipe
  class), and a reader poll for vendored-tree absence during install #2 (the
  TASK-IMP-137 window; the poll under-detects pre-137 and hard-catches regressions
  post-137).

## Deviations (recorded)

1. **§1.6 BRAIN recording NOT executed** (the batch/8 carve-out, and the spec's own
   `depends_on: [TASK-MEMORY-303]` edge): the live store is `FROZEN_RECOVERABLE`;
   recording below READY violates §12/§1. Delivered instead: `brain-record.sh` (guarded,
   fail-closed) + `brain-recording-checklist.md`. AC 6's fixture-store demonstration
   ships with the §1.6 execution post-303; until then `t06_brain_record_fixture` asserts
   the deliverable exists and prints the deferral loudly. Final acceptance includes the
   executed recording (checklist's Acceptance linkage section).
2. **CHANGELOG.md not edited** (ownership): text below; t07 fails loudly until pasted.
3. **The plan's Phase-3 checker list named "loop-constant pin"**; the audited spec
   supersedes the plan (its `source_decisions` records the one-owner rule): G11's checker
   is TASK-CUO-304's `modules/cuo/tests/test_doctrine_constants.py`. The doc's G11
   section carries the post-hardening expected state (ceiling **3** everywhere:
   ship-tasks §11b = `api.py` `halt_on_repeat_rework` = CLI); this suite adds no second
   G11 authority. G13 (stuck-WIP), in the spec's six, is implemented instead.

## Verification (verbatim, focused only)

```
$ bash -n scripts/tests/test_benchmark_gates.sh && bash -n docs/tasks/.../brain-record.sh
bash -n OK (both)
$ CYBEROS_BENCHMARK_SKIP_HEAVY=1 bash scripts/tests/test_benchmark_gates.sh
  ok   t01_doc_complete_and_consistent
  ok   t02_checkers_fail_on_violations          # all six negatives bite
  FAIL t03_green_at_head_reportonly_declared: G4 headline counts failed at HEAD ...
    g04: README.md task claim stale — tree measures 572 tasks (fix: update the headline line)
    g04: docs/README.md claim stale — tree measures 572 task specs across 29 domains (fix: the tasks/ row in the directory map)
  defer g05/g06/g16 — CYBEROS_BENCHMARK_SKIP_HEAVY=1 (mid-wave dry-run; the final pass runs this suite without the flag)
  g13: scanned 12 in-flight spec(s), 0 stale (threshold 30d) — report-only, no status was changed
  ok   t04_g13_reports_never_mutates
  ok   t05_risk_rows_complete
  ok   t06_brain_record_fixture (deliverable present; recording deferred per depends_on)
  FAIL t07_changelog_four_deliverables (×4: 'benchmark-gates.md' / 'test_benchmark_gates' / 'R-EXT' / 'BRAIN')
benchmark-gates: 5 passed, 5 failed
```

G13 corpus note (hand-verified): the 12 in-flight specs' last true status transitions
are 2026-06-28 (`draft -> implementing` in commit 3220b93a "feat: dc slice"; e.g.
TASK-MCP-005 — created 2026-05-18 draft, flipped 06-28). At 25 days they sit under the
30-day threshold and cross it 2026-07-28; a naive last-touch signal would instead have
reset every clock at the 07-14 tree rename, which is why the checker diffs the status
VALUE across `--follow` history. The plan's T9 sibling owns the triage either way.

## Dry-run / final-pass classification

| Check | Classification | Status mid-wave |
|---|---|---|
| t01 doc completeness + spec consistency | dry-runnable now | PASSING |
| t02 six negative fixtures | dry-runnable now | PASSING |
| t_g03 enum cross-check (live run) | dry-runnable now | PASSING (enum unified at HEAD by phase-1) |
| t_g04 headline counts (live run) | dry-runnable now | FAILING as designed: README.md says 562 tasks, tree measures 572; docs/README.md says 562/29, measures 572/29 — final pass updates the two lines (message names them; README not in this worker's ownership) |
| t_g05 payload walker (live run) | needs final-pass (builds scratch payload; sibling owns payload sources mid-wave) | deferred via CYBEROS_BENCHMARK_SKIP_HEAVY=1 |
| t_g06 vendored-gate smoke (live run) | needs final-pass (scratch install) | deferred (same) |
| t_g13 stuck-WIP report (live run) | dry-runnable now | PASSING (report-only; 12 scanned, 0 stale — they cross the threshold 2026-07-28) |
| t_g16 reinstall idempotency (live run) | needs final-pass (double scratch install; TASK-IMP-137's atomic swap mid-wave) | deferred (same) |
| t04 g13 never-mutates | dry-runnable now | PASSING |
| t05 risk rows | dry-runnable now | PASSING |
| t06 brain deliverable | dry-runnable now (full fixture demo deferred to post-303) | PASSING with loud deferral line |
| t07 changelog | needs final-pass (CHANGELOG paste) | failing, names the fix |
| brain-record.sh execution | post-303 + final acceptance only | NOT run (guard verified by inspection: refuses unless `state` prints READY and `doctor` exits 0) |

Final-pass command for the heavy half: `bash scripts/tests/test_benchmark_gates.sh`
(no env flag) after the sibling payload/install work has landed.

## CHANGELOG entry — paste into the existing `## [Unreleased]` block (final pass)

```markdown
Added
- `docs/verification/benchmark-gates.md` — the sixteen benchmark gates (G1–G16) from the
  2026-07-23 deep audit, published as re-checkable pass/fail criteria with severities,
  tiers, checked files, and one owning checker per gate; the status table is the
  report-only/enforcing coordination surface. (TASK-IMP-140)
- `scripts/tests/test_benchmark_gates` suite — automated checkers for the six unowned
  gates: G3 status-enum cross-check, G4 README headline-count truth, G5 payload reference
  walker, G6 vendored-gate executability smoke, G13 stuck-WIP detector (report-only,
  never mutates), G16 reinstall idempotency + config survival. Registered via the
  run_all glob and the caf-evals-gate CI workflow. (TASK-IMP-140)
- `docs/reference/risk-register.md` gains R-EXT-01..07 — the audit's risk classes
  (self-approval, vacuous green gates, config wipe, prompt injection, payload
  divergence, partial-install window, frozen BRAIN), each with detection, preventing
  gate, and recovery. (TASK-IMP-140)
- BRAIN recording of the audit verdict + gates + wave decisions: prepared as a guarded
  script (`brain-record.sh`, refuses below READY) and executed after TASK-MEMORY-303's
  store repair per the spec's depends_on edge. (TASK-IMP-140)
```

## Open items for the HITL reviewer / final pass

1. Update the two headline-count lines (README.md: `562 tasks` → measured; docs/README.md:
   `562 task specs across 29 domains` → measured) — t_g04's failure message prints the
   live numbers; recount at final-pass time since siblings may add tasks.
2. Paste the CHANGELOG entry above (t07 flips green in the same commit).
3. Run the heavy checkers without the skip flag once sibling payload/install work landed.
4. Post-TASK-MEMORY-303: run `brain-record.sh` (checklist in this folder); final
   acceptance includes doctor READY before/after + chain verify + the §13 block.
5. If any sibling batch/8 task slips past this landing, flip its gates' rows in the doc's
   status table to `pending (owning task in flight)` — the table is the coordination
   surface (doc §Status table note).
