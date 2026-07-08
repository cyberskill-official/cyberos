---
name: cyberos-improve-review
description: "Use when reviewing or signing off a CyberOS improvement program's tasks that have reached review status — the human pass (optionally agent-assisted) that moves tasks from review to done. Triggers: \"review the memory backlog\", \"review packet for chat improvement\", \"sign off the improvement tasks\", \"prep a review of docs/improvement/<program>\". This is the official CyberOS review protocol; it is generic and reads each program's program.yaml. To implement/advance tasks use cyberos-improve-implement instead."
---

# CyberOS improvement-program review protocol

The one official way tasks move from `review` to `done`. Only a human sets `done`; an agent may only prepare a
review packet and present evidence. Generic across programs: paths, gates, id prefix, and guardrails come from
`<program>/program.yaml` (see `cyberos-improve-implement` and `.claude/skills/cyberos/README.md`).

Two modes: **solo** (the operator walks the checklist against the branch) and **assisted** (an agent prepares
a packet first so the human pass takes minutes). Either way the human verdict is the only thing that closes a
task.

## Step 0 — resolve the program

Determine the program directory and read `<program>/program.yaml` for `{backlog.file}`, `{ledger}`, `{gates}`,
`{branch}`, `{report}`, `{guardrails}`, and `{statuses}`. List every task currently in the `review` status,
oldest first.

## The human checklist (per task in review)

1. Open the task's ledger entry and its spec/card side by side. Confirm every acceptance bullet has evidence
   that would fail if the feature were reverted (a test, not a claim). Spot-run one.
2. Read the diff for the commit(s) named in the ledger. Look for: scope creep beyond the spec; a protected
   invariant from `{guardrails.protected}` touched without a test (RLS / tenant-scoping, auth, audit chain,
   consent, PII, cost ledger); destructive or mis-ordered migrations; hardcoded secrets or URLs; missing
   EN+VI strings where the program requires them.
2b. For **security-class** tasks, run an adversarial pass: name one input or path that would defeat the
    change and show why it does not (or flag it). Do a revert-in-worktree spot check on one acceptance claim
    — does the named test actually fail without the change?
3. For **protocol / cross-cutting** tasks: new surface (events, fields, frames, error envelopes) matches the
   report's naming section; additions are versioned; no old client breaks silently.
4. **Feel pass** for user-facing tasks: use the feature for five minutes on staging. Trust your daily-tool
   instinct over the ledger.
5. **Verdict** in `{backlog.file}`: `review -> done`, or back to `ready`/`in_progress` with a one-line rework
   note appended to the ledger (never edit the original ledger entry). Commit the status change.
6. When a phase/wave closes (every task in it `done`), run the operator actions the ledger named: `git push`
   the branch, open/merge the PR, and any operator-only items (filings, secrets, deploys, go-live sign-offs).

Human-only items no agent may close: anything in `{guardrails.human_only_ids}` (legal/counsel, pen-test
vendor + triage, decision-gate verdicts), every push/deploy/release, and any phase exit-bar sign-off.

## Assisted mode — the review-prep packet (agent, read-only)

Paste "prepare a review packet for `<program>`" (or invoke this skill on a program). The agent is **read-only
except for the one packet file it writes** — it must NOT modify code, backlog statuses, or the ledger.

For every task in `review`:
1. Diff-read its commit(s). Map each changed file to the task spec; flag anything outside spec scope.
2. Re-run its `{gates}` and the acceptance tests named in the ledger (Mac-gate loop for native builds; the
   program's client checks where relevant). Record pass/fail with counts — do not fix anything.
3. Adversarial pass: try to falsify one acceptance claim per task (revert-in-worktree spot check: does the
   named test fail without the change?).
4. Check the cross-cutting rules from `{guardrails}` (protected invariants, EN+VI, migration expand/contract,
   error-envelope usage, no new client timers, no secrets).
5. Score risk low/medium/high with one sentence.

Write the packet to `<program>/notes/review-packet-<date>.md` (or the program's notes location): one section
per task (what it does, evidence status, flags, risk, suggested verdict), then a global section (gate health,
suite runtimes, any drift between report and code), ending with the three tasks most deserving the human feel
pass and why. Do not mark anything `done` — the human does that.

## Output of a review session

After all verdicts: summarize phase progress, list newly unblocked tasks (dependents whose deps are now
`done`), and state whether a phase gate closed. If it did, remind the operator of the pending operator-only
actions (push, filings, secrets, deploys) the ledgers named. Present evidence, not persuasion; if the diff and
the ledger disagree, the diff wins and you flag the ledger.
