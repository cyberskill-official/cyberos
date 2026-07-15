---
template: task@1
id: TASK-CUO-200
title: "Harness Wave 1 — read-only daily report of self_audit signals per skill"
type: feature
author: "@stephen"
department: engineering
status: done
priority: p1
created_at: 2026-05-19T20:00:00+07:00
ai_authorship: assisted
eu_ai_act_risk_class: minimal
target_release: 2026-Q3
client_visible: false
module: cuo
new_files:
  - modules/cuo/cuo/core/harness.py
  - modules/cuo/cuo/core/harness_signals.py
  - modules/cuo/tests/test_harness_report.py
  - docs/proposals/README.md
blocks: [TASK-CUO-201, TASK-CUO-202, TASK-CUO-203]
depends_on: [TASK-MEMORY-112, TASK-MEMORY-114, TASK-MEMORY-117, TASK-MEMORY-120]
---

## Summary

Build the read-only foundation of the CyberOS continuous-improvement harness: a daemon-or-CLI tool that walks the memory audit chain, computes per-skill and per-workflow anomaly metrics over a configurable window, and emits a daily `harness-report.md` listing which `self_audit.anomaly_signals` (declared in every SKILL.md frontmatter) tripped which thresholds. **No mutation, no proposals, no auto-bumps — just visibility.** This is the foundation Waves 2/3/4 (TASK-CUO-201/202/203) build on.

## Problem

Every SKILL.md already declares `self_audit.anomaly_signals` (e.g. `confidence_low_streak`, `user_correction_streak`, `needs_human_rate_above`, `deterministic_drift`) and `human_fine_tune.signals_to_initiate` (acceptance-rate thresholds, HITL-pause rates, drift signals). The policy is fully specified. **But nothing reads those signals.** Operators have no way to know which skills are struggling, drifting, or repeatedly needing HITL until they manually scan audit logs — which doesn't happen at scale.

Without a harness, the entire self-evolution architecture lives in frontmatter as dormant intent. Wave 1 lights it up by surfacing the signals deterministically, daily, in a markdown report Stephen can read in 5 minutes.

## Proposed Solution

### §1 Normative requirements

1. **MUST** ship `cuo/core/harness.py` with a `compute_report(audit_dir, window: timedelta, skill_root) -> HarnessReport` API that walks the audit chain and returns a structured report.
2. **MUST** ship `cuo/core/harness_signals.py` with one Python function per declared signal type (`confidence_low_streak`, `user_correction_streak`, `rule_reversal_streak`, `needs_human_rate_above`, `deterministic_drift`, plus the `human_fine_tune.signals_to_initiate` set: `acceptance_rate_below`, `hitl_pause_rate_above`, `drift_signal_count_above`). Each signal function takes the windowed audit rows + the skill's threshold dict and returns `(tripped: bool, value: float, evidence_rows: list[dict])`.
3. **MUST** support window arguments via duration strings (`24h`, `7d`, `30d`) parsed once and applied per-skill. *(traces_to: §1 #3 → AC #1)*
4. **MUST** read each skill's `self_audit.anomaly_signals` and `human_fine_tune.signals_to_initiate` blocks from its SKILL.md frontmatter at report time — no hard-coded threshold lists. *(traces_to: §1 #4 → AC #2)*
5. **MUST** emit a markdown report at `docs/harness/harness-report-<YYYY-MM-DD>.md` with sections: (a) skills with tripped signals (sorted by severity), (b) workflows with elevated `ROUTED_BACK` rates, (c) per-task routed-back history with `routed_back_count` over the window, (d) summary stats (total runs, total HITL pauses, total rework events).
6. **MUST** include the matching audit-row IDs as `evidence:` cells in the report so operators can drill in via `cyberos history --row <id>`.
7. **MUST NOT** mutate any skill, RUBRIC, contract, or workflow file. **Read-only.** Wave 2 (TASK-CUO-201) introduces proposal authoring.
8. **MUST** add CLI subcommand `cyberos-cuo harness report --since <window> [--skill <name>] [--workflow <id>] [--out <path>]`. Default window `24h`; default path computed from date.
9. **SHOULD** include a `--watch` mode that re-runs every N minutes and writes the latest report atomically, for use under a cron / systemd timer.
10. **MUST** emit one `harness.report_emitted` memory audit row per run, with payload `{report_path, window, skills_with_signals, workflows_with_signals, evidence_row_count}`.

### §2 Out of scope (deferred to Waves 2/3/4)

- Authoring `refinement_proposal` artefacts (TASK-CUO-201)
- Auto-applying minor bumps (TASK-CUO-202)
- Workflow-level chain evolution (TASK-CUO-203)
- Multi-tenant aggregation across BRAINs (long-term)

## Alternatives Considered

1. **Grafana dashboard** — over-engineered for a single-operator use case. The audit chain isn't yet in Prometheus/OTel (TASK-OBS-001 ships that). Markdown report is the fastest path.
2. **Inline in `cyberos verify`** — would muddle the doctor invariants (which check structural integrity) with the policy-anomaly signals (which check evolution-worthiness). Keep them separate.
3. **Per-skill `self-audit run` subcommand** — useful but doesn't give cross-skill visibility. The aggregate report subsumes per-skill checks (which can still be added later).

## Success Metrics

| metric | baseline | target | deadline |
|---|---|---|---|
| Time to identify a struggling skill (manual today) | ~30 min (grep audit logs) | < 1 min (open report) | 2026-05-31 |
| Operator-perceived signal density of report | n/a (no report) | "actionable" rated ≥ 4/5 in first 4 reports | 2026-06-15 |
| Skills with at least one threshold declared in frontmatter | 100% (already done) | 100% | already met |

## Scope

In scope: `cuo/core/harness.py`, `cuo/core/harness_signals.py`, signal functions for every declared signal type, markdown report formatter, CLI subcommand, audit row emission.

### Out of scope

- Proposal authoring (Wave 2)
- Auto-bumps (Wave 3)
- Workflow chain evolution (Wave 4)
- Per-workflow custom signals beyond the standard set

## Dependencies

- **TASK-MEMORY-112** — episodic memory (provides `episode.logged` rows that some signals aggregate over)
- **TASK-MEMORY-114** — write-time importance (provides `memory.importance_scored` rows for `confidence_low_streak`)
- **TASK-MEMORY-117** — per-store ACL (provides `memory.acl_denied` rows that contribute to drift detection)
- **TASK-MEMORY-120** — `cyberos history` (provides the path-set + row-filter machinery the harness reuses for evidence linking)

## AI Authorship Disclosure

- **Tools used:** Anthropic Claude (the assistant authored the task body via the standard `task-author` chain).
- **Scope:** §1 normative clauses, §4 ACs, §5 test entries, alternatives section — all draft-generated and then revised by the operator.
- **Human review:** Stephen Cheng reviewed the spec end-to-end before audit; revisions are tracked in the audit-fix log per §10 of the sibling .audit.md.

## §4 Acceptance Criteria

1. `cyberos-cuo harness report --since 7d` produces a non-empty markdown file at `docs/harness/harness-report-YYYY-MM-DD.md`. *(traces_to: §1 #5, #8)*
2. The report's "Skills with tripped signals" section lists at least one entry when running against a seeded chain with 11 `memory.fr_routed_back` rows for one task (above `acceptance_rate_below: 0.6` if any forward-runs exist). *(traces_to: §1 #2, #5)*
3. Each tripped signal's row carries the skill name, signal id, observed value, threshold, and at least one evidence row ID. *(traces_to: §1 #6)*
4. The report includes a "Workflows with elevated rework" section sorting workflows by `routed_back_count / total_runs` descending. *(traces_to: §1 #5)*
5. Re-running the same command in `--watch` mode after one new event writes a new report atomically (write-to-temp then rename) without truncation. *(traces_to: §1 #9)*
6. Per run, exactly one `harness.report_emitted` memory aux row is appended; payload validates against §1 #10. *(traces_to: §1 #10)*
7. Running the harness on an audit chain that contains zero qualifying rows produces a report with all sections present but empty (no crash, no missing-key errors). *(traces_to: §1 #5)*

## §5 Verification

- `modules/cuo/tests/test_harness_report.py::test_report_emits_markdown` (covers AC #1)
- `modules/cuo/tests/test_harness_report.py::test_signal_thresholds_trip_correctly` (AC #2, #3)
- `modules/cuo/tests/test_harness_report.py::test_workflow_rework_rate` (AC #4)
- `modules/cuo/tests/test_harness_report.py::test_watch_mode_atomic_write` (AC #5)
- `modules/cuo/tests/test_harness_report.py::test_emits_audit_row` (AC #6)
- `modules/cuo/tests/test_harness_report.py::test_empty_chain_clean_exit` (AC #7)
