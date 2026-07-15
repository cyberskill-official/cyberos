---
id: TASK-SKILL-118
title: "Bring the six thin vendored pairs to full contract parity (RUBRIC, PIPELINE, INVARIANTS, envelopes, references, acceptance)"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: improvement
created_at: 2026-07-12T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: SKILL
priority: p0
status: done
verify: T
phase: Wave B - finish the children
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: 2026-07-12
memory_chain_hash: null
related_tasks: [TASK-SKILL-116, TASK-SKILL-117, TASK-CUO-207, TASK-CUO-209]
depends_on: []
blocks: []
source_pages:
  - modules/skill/task-author/SKILL.md
  - modules/skill/task-audit/RUBRIC.md
  - modules/skill/repo-context-map-author/SKILL.md
  - modules/skill/coverage-gate-audit/SKILL.md
source_decisions:
  - "2026-07-12 investigation: of the 10 vendored pairs, 4 are full contracts (task, implementation-plan, architecture-decision-record, code-review) and 6 are thin - SKILL.md + acceptance/TRIGGER_TESTS.md only: repo-context-map, edge-case-matrix, mock-contract-test, observability-injection, backlog-state-update, coverage-gate."
  - "The thin pairs already state numeric gates in prose (rows >= 8, branch coverage >= 80%, coverage 90%, status-cell-only, etc.); parity means those gates become versioned rubric rules, not new invented policy."
language: markdown + JSON (skill contracts)
service: modules/skill/
new_files:
  - modules/skill/repo-context-map-audit/RUBRIC.md
  - modules/skill/edge-case-matrix-audit/RUBRIC.md
  - modules/skill/mock-contract-test-audit/RUBRIC.md
  - modules/skill/observability-injection-audit/RUBRIC.md
  - modules/skill/backlog-state-update-audit/RUBRIC.md
  - modules/skill/coverage-gate-audit/RUBRIC.md
  - modules/skill/repo-context-map-author/PIPELINE.md
  - modules/skill/edge-case-matrix-author/PIPELINE.md
  - modules/skill/mock-contract-test-author/PIPELINE.md
  - modules/skill/observability-injection-author/PIPELINE.md
  - modules/skill/backlog-state-update-author/PIPELINE.md
  - modules/skill/coverage-gate-author/PIPELINE.md
  - tools/cyberos-install/check-pair-parity.sh
  - tools/cyberos-install/tests/test_pair_parity.sh
modified_files:
  - modules/skill/repo-context-map-author/SKILL.md
  - modules/skill/edge-case-matrix-author/SKILL.md
  - modules/skill/mock-contract-test-author/SKILL.md
  - modules/skill/observability-injection-author/SKILL.md
  - modules/skill/backlog-state-update-author/SKILL.md
  - modules/skill/coverage-gate-author/SKILL.md
---

# TASK-SKILL-118: Thin-pair contract parity

## §1 - Description

Six pairs at the heart of /ship-tasks run on a SKILL.md and trigger tests alone: no rubric an auditor can score against, no pipeline, no envelopes, no invariants. The audits therefore re-derive their bar from prose every run. This task raises all six to the file-level contract the four full pairs already have.

Normative clauses:

1. Each of the six author skills (`repo-context-map`, `edge-case-matrix`, `mock-contract-test`, `observability-injection`, `backlog-state-update`, `coverage-gate`) MUST gain: `PIPELINE.md` (phased steps incl. HALT points), `INVARIANTS.md`, `envelopes/input.json` + `envelopes/output.json`, `references/FAILURE_MODES.md`, and `acceptance/README.md` - matching the file classes of `task-author`.
2. Each of the six audit skills MUST gain: `RUBRIC.md` (versioned `<name>_rubric@1.0`, rule families with stable rule IDs, /10 scoring, 10/10 pass bar), `AUDIT_LOOP.md`, `REPORT_FORMAT.md`, `envelopes/input.json` + `envelopes/output.json`, and `acceptance/README.md` - matching the file classes of `task-audit`.
3. The rubrics MUST encode exactly the gates already normative in each pair's SKILL.md prose, with rule IDs: edge-case-matrix (>= 1 row per category; SECURITY rows point at real test paths; DEGRADATION rows carry detection + recovery; total_rows >= 8 for MUST tasks), observability-injection (>= 1 log point per state transition, >= 1 span per external IO, >= 1 counter per error branch, branch_coverage >= 80%, redaction policy when PII in scope), coverage-gate (tests_failed == 0; files_below_90pct empty; ecm_rows_uncovered empty; raw terminal present + non-truncated; §1-clause test closure), backlog-state-update (status in the 10-value enum; line_number resolves; old_line byte-match; evidence rows resolve; mutation_kind == status-cell-only), mock-contract-test (>= 1 request/response pair; error_modes cover every SECURITY/DEGRADATION matrix row; swap_target is a real symbol; sunset criterion observable; contract tests pass against the mock), repo-context-map (three baseline patterns present; pinned_in references resolve; schemas present when task declares migrations; module-placement warning null or escalated).
4. Numeric gates MUST be expressed as named constants at the top of each RUBRIC.md, with the coverage threshold noted as overridable by `.cyberos/config.yaml` `coverage_threshold` once TASK-CUO-207 lands (default 90 preserved).
5. Artefact schemas MUST stay at @1 - additive documentation only; no change to any emitted artefact shape, so already-shipped tasks' artefacts remain valid.
6. A script `tools/cyberos-install/check-pair-parity.sh <skills-dir>` MUST verify, for every author/audit pair present: authors carry the #1 file classes, audits carry the #2 file classes; missing files exit 10 as `PARITY <skill>: missing <file>`. `build.sh` MUST run it over the vendored set after the chain-coverage check (TASK-SKILL-116 ordering).
7. Each rewritten SKILL.md MUST keep its existing trigger description contract (TASK-SKILL-111/112/113 conventions) - descriptions, trigger tests, and frontmatter shape unchanged except for pointers to the new files.

## §2 - Why this design

Parity is defined by pointing at the four existing full pairs rather than inventing a new standard - the repo already voted. Encoding prose gates as rubric rule IDs is what makes audit verdicts reproducible across sessions and agents (the auditor cites SPK-style IDs instead of paraphrasing prose). The parity checker turns the definition of done into a machine gate, and covers TASK-SKILL-117's new pair and TASK-CUO-209's expansion at no extra cost.

## §3 - Contract

Rubric header convention (all six):

```markdown
# <name>_rubric@1.0
constants: TOTAL_ROWS_MIN=8 (MUST tasks) | BRANCH_COVERAGE_MIN=80 | COVERAGE_THRESHOLD=90 (config-overridable, TASK-CUO-207)
families: <ABBR>-STRUCT | <ABBR>-GATE | <ABBR>-TRACE   (per-pair families as needed)
verdict: pass requires 10/10; any family failure -> fail; ambiguity -> needs_human
```

`check-pair-parity.sh <skills-dir>`: exit 0 | exit 10 with `PARITY` lines | exit 2 unreadable dir.

## §4 - Acceptance criteria

1. **All 12 skill dirs carry their file classes** (§1 #1, #2) - `check-pair-parity.sh modules/skill` reports zero PARITY lines for the six pairs (and the four full pairs still pass).
2. **Every prose gate has a rule ID** (§1 #3) - for each of the six audits, every numeric/enum gate quoted from its SKILL.md appears in RUBRIC.md with a stable rule ID; spot-list in each RUBRIC.md's changelog block maps prose -> rule.
3. **Constants are declared once** (§1 #4) - the numeric gates appear as named constants; coverage rubric names the TASK-CUO-207 override hook.
4. **No artefact shape change** (§1 #5) - each pair's SKILL.md artefact section is diff-identical for the artefact fields before/after (documentation-only diff).
5. **Parity checker catches a miss** (§1 #6) - deleting one RUBRIC.md in a scratch copy exits 10 naming the pair and file; `build.sh` propagates the failure.
6. **Trigger contracts untouched** (§1 #7) - all six author trigger-test suites pass unmodified (descriptions and trigger tests byte-identical except added file pointers).

## §5 - Verification

```bash
# tools/cyberos-install/tests/test_pair_parity.sh
t01_all_pairs_parity_clean()     # AC 1
t02_prose_gate_rule_ids()        # AC 2  (grep each rubric for its constants + rule-ID table)
t03_constants_block()            # AC 3
t04_artefact_sections_stable()   # AC 4  (git diff scoped to artefact-spec heading ranges is empty)
t05_checker_catches_missing()    # AC 5
t06_trigger_tests_unchanged()    # AC 6  (sha256 of the six TRIGGER_TESTS.md before/after)
```

## §6 - Implementation skeleton

Full file matrix = 72 files across the 12 skill dirs (6 per author, 6 per audit, per §1 #1-#2); frontmatter `new_files` lists the rubric/pipeline spine plus tooling, and the parity checker is the completeness authority (AC 1 gates the whole matrix, so nothing hides behind the abbreviated list). Per pair: derive PIPELINE.md phases from the SKILL.md's own step prose; INVARIANTS.md lifts MUST/MUST NOT lines; envelopes copy the shape of the nearest full pair (backlog-state-update mirrors task-audit's, coverage-gate mirrors implementation-plan-audit's); RUBRIC.md per §3 header + one family table per gate group. Order of work: backlog-state-update and coverage-gate first (they gate every ship run), then edge-case-matrix, observability-injection, mock-contract-test, repo-context-map.

## §7 - Dependencies

None upstream. TASK-CUO-207 later flips COVERAGE_THRESHOLD to config-driven (the rubric already names the hook). TASK-CUO-209 vendors the enriched pairs as-is. TASK-CUO-205 will bump backlog-state-update to @2 (insert-row) - land THIS task first so the @2 change edits a full contract, not a thin one.

## §8 - Example payloads

```
$ bash tools/cyberos-install/check-pair-parity.sh modules/skill
PARITY coverage-gate-audit: missing RUBRIC.md
PARITY coverage-gate-audit: missing REPORT_FORMAT.md
$ echo $?
10
```

## §9 - Open questions

None blocking. Shared references (HITL_PROTOCOL.md, ANTI_FABRICATION.md, UNTRUSTED_CONTENT.md) stay per-skill copies for now, matching the existing full pairs; deduplication into a shared dir is a separate refactor candidate, deliberately not smuggled in here.

## §10 - Failure modes inventory

1. Rubric invents a stricter bar than the SKILL.md prose - AC 2's prose->rule mapping table makes each rule cite its prose source; unsourced rules are review findings.
2. Envelope drift from actual artefacts - envelopes are copied from the nearest full pair then field-checked against each SKILL.md artefact section (AC 4 protects the section).
3. Parity checker too rigid for legitimately different skills (backlog-state-update has no references/ need) - the checker checks FILE CLASSES per side as listed in §1; the list IS the policy, adjustable only by editing this task's clauses.
4. Vendored copies go stale - build.sh copies from modules/skill on every build; TASK-IMP-068's gate rebuilds on every touch of modules/skill/**.
5. Six-pair scope creep into content rewrites - clause #7 pins descriptions and trigger tests byte-stable; the task adds files, it does not re-author skills.

## §11 - Implementation notes

Keep rule-ID prefixes distinct per pair (RCM-, ECM-, MCT-, OBS-, BSU-, COV-) so audit reports stay greppable across a ship run's artefact trail. The parity checker's file-class lists live at the top of the script as two arrays - one place to evolve the convention.

**Post-ship amendment (2026-07-12, TASK-IMP-071 leg):** AC 4's t04 guard false-fired on three
legitimate mid-flight citation mutations (the point-in-time-guard class). Amended to an at-rest
guard: dirty-worktree files warn and defer to the committed state; CI always checks clean trees, so
the additive-only guarantee holds where it matters.

*End of TASK-SKILL-118.*
