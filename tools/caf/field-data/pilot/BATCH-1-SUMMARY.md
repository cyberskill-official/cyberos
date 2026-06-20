# Pilot batch 1 — calibration summary (2026-06-12)

Three gated runner-mode runs, framework v1.3.0. All three BACKLOGs validate
CLEAN against the framework; portfolio aggregate: **3/3 clean, 0 active
violations, 9 findings total** (`reports/_portfolio-batch1.json`).

| Run | Findings | BACKLOG valid? | Notable |
|---|---|---|---|
| kymondongiap | 1 Critical + 6 High | CLEAN (after 1 R1 fix) | unauth `/public/history` cross-user data exposure; no authn/authz; broken build; blocking AI calls; cache time-correctness bug |
| 3d-preriodic-table | 1 High | CLEAN (first pass) | red test suite — unguarded `localStorage.getItem` |
| mock-exam | 1 High | CLEAN (first pass) | client-trusted exam score at `/api/exam/submit` (RPC unverifiable in-repo) |

All three runs PARKED at the gate (`Approved:` empty) — Phase 3 awaits the
maintainer. Artifacts (audit-profile.yaml + docs/BACKLOG.md) live uncommitted
in each target's worktree.

## What the batch taught the framework (TESTING-PROTOCOL calibration)

**Validator accuracy:** false positives 0/3, misses 0/3 observed,
denylist_gaps 0/3. fabrication_check 10 samples re-run across the batch, 0
mismatches. The validator caught one real artifact defect on run 1
(`R1-UNLINKED-OUTPUT`, a true positive — verify command differed between table
cell and fenced `$`-line) and passed everything else correctly.

**One framework defect found and fixed mid-batch (the point of the pilot):**
runner-mode reports left `protocol_version: null` despite the artifact's
`Protocol:` echo. Fixed same run — framework `493ef18` (build_report falls back
to the loop's echo; copy mode still wins), FAILURE_LOG row 2026-06-12, CI
report-contract now asserts it. All 3 reports carry v1.3.0 provenance.

**R2 discipline held under stress:** mock-exam ran `BENCHMARK_MODE=auto` — the
mode whose v1-era ancestor benchmarked a metaphysics app against Palantir. It
did NOT invent a comparator; "No external benchmark applicable" + INTERNAL
targets. The founding failure mode did not recur even when invited.

## Calibration decision: no CRITIC cycle this batch

Per CRITIC.md Step 2 / Rule of Three: no protocol-wording gap recurred at ≥
High. The lone defect was harness (already shipped). Promoting the queued
candidates (DEPTH semantics, R2 offline clause) would be speculative —
**their evidence gate is the T4 cross-model audit, which was NOT run** (this
batch was single-model). That diff is the open item before either candidate
can be promoted. Cap was ≤1 cycle; the honest outcome is 0.

## Open / next
- Maintainer: write `Approved:` lines (or `none`) in each target's
  docs/BACKLOG.md to unblock Phase 3; score each run's retro (T2) — the
  feedback records hold `retro_score: FILLME`.
- T4 cross-model: re-run one target (e.g. 3d-preriodic-table) with a second
  CLI (`codex exec` / `gemini`), diff finding overlap + severity agreement,
  fill the record's `cross_model` field. Unlocks the queued candidates.
- kymondongiap's Critical (unauth data exposure) is the highest-priority real
  fix surfaced by the batch — recommend approving L1-T1 first.
