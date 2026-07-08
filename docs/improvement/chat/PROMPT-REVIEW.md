# Review trigger prompt (human pass, optionally agent-assisted)

Two ways to use this file. Solo: Stephen walks the checklist himself against the branch. Assisted:
paste the prompt below into a fresh agent session first; it prepares a review packet so the human pass
takes minutes, not hours. Either way, the human verdict is the only thing that moves a task from
review to done.

## The human checklist (per task in review)

1. Open the LEDGER.md entry and the task spec side by side. Every acceptance check has evidence that
   would fail if the feature were reverted (a test, not a claim). Spot-run one.
2. Read the diff for the commit(s) named in the ledger. Look for: scope creep beyond the spec, RLS or
   tenant-scoping touched without a test, migrations that are destructive or ordered wrong, secrets or
   URLs hardcoded, English-only strings.
3. Protocol tasks (phase 1, T-053): naming matches report section 3; frame/field additions are
   versioned; nothing breaks an old client silently (T-030 policy).
4. Feel pass for user-facing tasks: use the feature for five minutes on staging. Trust your daily-tool
   instinct over the ledger.
5. Verdict in BACKLOG.md: review -> done, or back to in_progress with a one-line rework note appended
   to the ledger entry (never edit the original entry).

Human-only items no agent may close: T-041 (counsel), T-058 (pen-test vendor + findings triage),
T-066 verdicts (D1-D6), anything blocked:input, every push/deploy/release, and the phase exit-bar
sign-offs in docs/deploy/chat-go-live-checklist.md.

---

Copy below into an agent session for an assisted review packet:

REVIEW_PREP: CHAT backlog review packet.

You are the review-prep agent. Do not modify code, BACKLOG.md statuses, or the ledger. Read-only
except for the one packet file you write.

Inputs: docs/improvement/chat/ (BACKLOG.md, LEDGER.md, tasks/), the report
docs/strategy/chat-enterprise-grade-plan-2026-07-06.md, and branch auto/chat-enterprise.

For every task currently in review status:
1. Diff-read its commits. Map each changed file to the task spec; flag anything outside spec scope.
2. Re-run its gate and the acceptance tests named in the ledger (Mac-gate loop for cargo; pnpm for
   client). Record pass/fail with counts - do not fix anything.
3. Adversarial pass: try to falsify the acceptance evidence (revert-in-worktree spot check on one
   claim per task: does the named test actually fail without the change?).
4. Check the cross-cutting rules: RLS on any new table, EN+VI strings, migration expand/contract,
   error envelope usage (post T-053), no new interval timers in the client, no secrets.
5. Score risk low/medium/high with one sentence.

Write docs/improvement/chat/notes/review-packet-<date>.md: one section per task with verdict-ready
summary (what it does, evidence status, flags, risk, suggested verdict), then a global section
(gate health, suites runtime, anything smelling of drift between report and code). End with the
three tasks most deserving of the human feel pass and why.

Do not mark anything done. The human does that in BACKLOG.md.
