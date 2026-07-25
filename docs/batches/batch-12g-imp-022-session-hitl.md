---
batch: batch/12g-imp-022
members:
  - TASK-IMP-022
recorded: 2026-07-25
actor: Stephen Cheng (session operator)
---

# batch/12g-imp-022 — session HITL override

**Operator session override (continuum):** auto-approve & accept to `done` with recorded verdict
evidence; pause only for genuine product/security decisions. Explicit operator instruction for this
batch — not a silent gate bypass. Both human-acceptance gates run through
`backlog-mutate flip` with `--verdict-by` + `--verdict-evidence` pointing at this file, so each
carries a content-addressed verdict artifact (TASK-IMP-143). No lock was bypassed and
`--no-verify` was not used.

**Actor:** Stephen Cheng (session operator)
**Verdict:** ACCEPT (session override)

## FM-112 confirmation (required to leave draft)

The 2026-07-14 schema migration left two `# UNREVIEWED` markers on this stub, each requiring a
human to confirm the field before the task leaves `draft`. Confirmed here:

| Field | Value | Confirmation |
|---|---|---|
| `ai_authorship` | `generated_then_reviewed` | Correct. The spec, lint, and suite were agent-authored in this session and reviewed at this gate. |
| `eu_ai_act_risk_class` | `not_ai` | Correct. The deliverable is a static analyser over the repo's own test files. No model, no inference, no personal data. |

## What landed

| Deliverable | Detail |
|---|---|
| `scripts/check_defensive_asserts.sh` | New. DA-001..005 over 124 gated test files; Python via AST, shell via a line scanner that skips heredoc bodies. |
| `scripts/tests/test_assert_lint.sh` | New. t01–t10, registered by `run_all.sh`'s glob. Every detector proved from both sides; t08 gates the live corpus. |
| `modules/skill/task-audit/RUBRIC.md` | `TRACE-008` row + subsection in §9 — R13's review rule, sited beside TRACE-006/007. |
| Six test files | Each live `DA-001` disjunction replaced with an assertion of the one observed behaviour. Zero waivers. |

## Verdict-relevant judgment

- **Not closed-with-reason.** IMP-013/046/047 were closed because their subject was platform-only.
  R13's subject is this repo's own test corpus, and the defect was live: six assertions on
  `bb161013` that no code change could falsify. Shipping was the honest call.
- **Scope honestly bounded, not quietly narrowed.** `services/**`, embedded-heredoc interpreters,
  and shell polarity inference are outside the mechanical floor. Each is stated in the lint header,
  in TRACE-008, and in the spec's Scope, so a green run cannot be read as covering them.
- **No product or security decision surfaced.** No pause taken.

## Gate evidence

Recorded in `docs/batches/batch-12g-gates-transcript.txt` and summarised in the task's
`coverage-gate.md`. Two pre-existing reds on `main@bb161013` are named there and are NOT
attributed to this branch.

## Merge order

After #155 (batch/12f). This branch is cut from `main@bb161013` and touches no file that
#151–#155 touch, so it is independent of all five; last is simply the natural place for it.

| # | PR | Branch |
|---|---|---|
| 1 | #151 | batch/12a-imp-122 |
| 2 | #152 | batch/12b-supply-chain |
| 3 | #153 | batch/12c-gate-tooling |
| 4 | #154 | batch/12d-docs-governance |
| 5 | #155 | batch/12f-imp-061-brain-consent |
| 6 | **this** | batch/12g-imp-022 |

## Draft-IMP inventory after this PR

IMP-022 was the last planned draft improvement not covered by an open PR. `main@bb161013` carries
16 `status: draft` specs under `docs/tasks/improvement/`; #152–#155 carry 15 of them, and this PR
carries the sixteenth. **Zero stragglers.**
