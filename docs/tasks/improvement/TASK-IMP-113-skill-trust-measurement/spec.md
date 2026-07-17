---
id: TASK-IMP-113
title: Skill trust measurement - pass rates as a report, not a gate
template: task@1
type: improvement
module: improvement
status: ready_to_implement
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T14:00:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-IMP-084]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.0.0"
owner: Stephen Cheng (CTO)
created: 2026-07-17
memory_chain_hash: null
effort_hours: 5
service: modules/skill
new_files:
  - tools/install/docs-tools/skill-log.mjs
  - tools/install/tests/test_skill_log.sh
modified_files:
  - modules/skill/task-audit/SKILL.md
  - modules/cuo/chief-technology-officer/workflows/ship-tasks.md
  - tools/install/install.sh
  - modules/cuo/tests/test_ship_manifest.py
source_pages:
  - "IMPROVEMENT_HANDOFF.md §11 IMP-30"
  - "How to Build An Agentic OS using Fable 5 (Avid, 2026-07-06) BUILD 4: autonomy per skill, 20 runs / 95 % - the mechanism, adopted as measurement only because our two gates forbid unattended shipping"
source_decisions:
  - "2026-07-17 Stephen: PLAN gate - scope C (all 13 actionable handoff findings), template override to task@1 (recorded HITL answer)."
---

# TASK-IMP-113: Skill trust measurement - pass rates as a report, not a gate

## Summary

All 53 skills have identical standing forever. `task-lint` (deterministic, 8-scenario suite) and `code-review-author` (a model's opinion) are trusted equally, and no pass-rate data exists for any of them because nothing records per-skill outcomes. Log pass/fail per invocation and render the table. Tiers are a REPORT for the operator, never a licence for the machine.

## Problem

"Which of our 53 skills actually works?" is answered today by vibes. Nothing distinguishes a skill that is provably right from one that is plausibly right, and nothing notices when a skill degrades. The article's line is exact: "turn up autonomy as trust grows" is not a mechanism; a table with rules is.

The honest limit is also exact. Their ledger governs unattended shipping, which our two gates forbid by design. So the tier cannot be a gate here - what transfers is the measurement, which IMP-20 needs (to know which skill is degrading) and IMP-23 needs (to price the loop).

## Proposed Solution

`skill-log.mjs` appends `<skill> <pass|fail> <task-id> <iso8601>` at each audit verdict; `--render` prints skill, runs, passes, rate, and a tier label. Tier thresholds mirror the article (auto >= 20 runs and >= 95 %; watch < 10 runs or < 90 %) because they are as good as any and someone tested them - but the label is INFORMATION. No workflow reads it, no gate consults it, and a skill at 60 % is a finding for the operator, not a signal to the machine.

## Alternatives Considered

- Tiers as a gate (the article's design). Rejected: it exists to allow unattended shipping, which deletes our premise. Adopting the gate would be adopting their product.
- Infer pass rates from git history. Rejected: an audit verdict is not visible in a diff, and reconstructing it is a guess.
- Log every skill invocation including reads. Rejected: the signal is the verdict, and logging everything buries it.

## Success Metrics

- Primary: after a batch, `--render` shows real runs and rates per skill - suite-asserted against a fixture ledger. Baseline: zero data on any skill.
- Guardrail: no workflow reads a tier; the ledger is append-only and never gates anything.

## Scope

In scope: `skill-log.mjs`, the append at audit verdicts, `--render`, suite arms.

### Out of scope / Non-Goals

- Any gate, threshold, or routing decision keyed on a tier.
- Unattended shipping at any tier - the two HITL gates are unconditional.
- Model or prompt quality evaluation.

## Dependencies

None logically. Pairs with TASK-IMP-110 (consumer) and TASK-IMP-114 (sibling metric).

**Serialisation note:** touches `install.sh` (shared with TASK-IMP-103, 104) and `ship-tasks.md` (shared with 108, 109, 114, 115). Per §11a the parent serialises shared-tree writes - these MUST NOT run as concurrent swarm members.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from IMPROVEMENT_HANDOFF.md §11 IMP-30, adapting BUILD 4's mechanism to a doctrine that forbids its purpose.
- **Human review:** scope approved at the 2026-07-17 PLAN gate; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 `skill-log.mjs` MUST append one row per audit verdict: skill name, `pass|fail`, task id, ISO-8601 timestamp.
- 1.2 The ledger MUST be append-only; the helper MUST NOT rewrite or delete rows.
- 1.3 `--render` MUST print skill, runs, passes, rate, and tier label.
- 1.4 Tier labels MUST be informational. No workflow, gate, or queue may read a tier to decide anything, and the helper's own docs MUST say so.
- 1.5 A skill with zero runs MUST render as `no data` rather than 0 % - an unmeasured skill is not a failing one.
- 1.6 The ledger MUST live at `docs/tasks/.workflow/skill-trust.tsv` and MUST be gitignored. TASK-IMP-090's seed covers `*.ship.json` and `*.manifest.json` only, so this task MUST extend the seed with `skill-trust.tsv` using 090's append-once discipline - the ledger is NOT covered today and asserting otherwise would be a false claim.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1, #1.3) - appended verdicts render with correct runs, passes, and rate - test: `tools/install/tests/test_skill_log.sh::t01_append_and_render`
- [ ] AC 2 (traces_to: #1.2) - the helper never rewrites an existing row - test: `tools/install/tests/test_skill_log.sh::t02_append_only`
- [ ] AC 3 (traces_to: #1.5) - a zero-run skill renders `no data` - test: `tools/install/tests/test_skill_log.sh::t03_zero_runs_no_data`
- [ ] AC 4 (traces_to: #1.6) - the seed gains the ledger pattern append-once (no duplicate on re-install) and `git check-ignore docs/tasks/.workflow/skill-trust.tsv` exits 0 - test: `tools/install/tests/test_skill_log.sh::t04_ledger_gitignored`
- [ ] AC 5 (traces_to: #1.4) - no workflow or gate reads a tier - verify: recorded grep in the gate log showing zero reads of the tier label outside the renderer (a negative structural claim; same rationale as TASK-IMP-090 AC 1).

## 3. Edge cases

- A skill renamed between runs: the ledger shows two skills. Correct - the old name's history belongs to the old name, and merging them would fabricate continuity.
- A run cut mid-flight (the API spend limit this run hit twice): no verdict, so no row. The ledger records verdicts, not attempts - and a skill must not be marked failed because the harness died.
- Concurrent appends from a swarm batch: append-only rows of one line each; the OS handles it for small writes, and the helper MUST NOT read-modify-write the file.
- A ledger with 10k rows: render aggregates in one pass, never loads a structure per row.
- Security-class: writes a TSV of names and verdicts; reads it back to count. Skill names come from the workflow, not from user input, and nothing is executed.
