# Trigger prompts - memory improvement program

Two prompts. Paste prompt A into a fresh agent session (Claude Code on the Mac, or any executor with repo access) to start or resume implementation. Paste prompt B to run a review session when tasks reach `in_review`. Both assume the repo root as working directory.

---

## Prompt A - implementation session (agent)

```text
AUTO_WORK: memory improvement program.

Context to load first, in this order:
1. docs/improvement/memory/README.md            (working agreement, gates, lifecycle)
2. docs/improvement/memory/backlog.yaml         (source of truth for task status)
3. The card file for the task you pick (docs/improvement/memory/tasks-phase-*.md)
4. docs/strategy/memory-enterprise-grade-and-auto-evolution-plan-2026-07-06.md
   (only the sections your task's refs point at - do not re-audit the module)
5. modules/cuo/EXECUTION-DISCIPLINE.md          (halt rules - binding)

Branch: auto/memory-enterprise (create from latest main if absent; never work on main).

Loop, until no eligible task remains or a §2 halt condition applies:
1. Pick the next task: lowest phase with open tasks, then priority
   (critical > high > medium > low), then lowest id, skipping any task whose
   deps are not done. Flip blocked tasks to ready_to_implement when their deps
   are done. Set your task implementing in backlog.yaml in the same commit as
   your first change.
2. Implement exactly what the card says. The card wins over your own ideas;
   if the card conflicts with the code you find, prefer the card's intent,
   record the deviation in the ledger, and keep the diff minimal. Do not
   refactor beyond the card's scope. New behavior needs tests in the same task.
3. Self-verify continuously (no pausing): cargo fmt --check, cargo clippy -p
   cyberos-memory --all-targets -- -D warnings, cargo test -p cyberos-memory
   (services/dev: docker compose up -d first), python -m pytest when
   modules/memory was touched, plus the task's own named tests and, once
   MEM-009 exists, the golden runner. Fix red gates yourself; a gate you
   broke is never a reason to stop (EXECUTION-DISCIPLINE §3).
4. When every gate is green and every acceptance bullet on the card passes,
   set the task in_review in backlog.yaml, commit as "MEM-0NN: <title>"
   (one task per commit; include the backlog status change and the ledger
   update in that commit), and continue to the next task.
5. If blocked past the circuit-breaker budget (5 consecutive gate failures on
   the same task), set it back to ready_to_implement with a blocked_note in
   backlog.yaml, ledger the blocker, and move to the next eligible task.

Ledger: append docs/auto-work/<today>-memory-<n>.md per session: tasks touched,
gate outputs (paste the tail, not the world), decisions taken, deviations,
anything routed back. ADR-class decisions get a real file under docs/adrs/.

Hard rules (EXECUTION-DISCIPLINE §2 - the only halts):
- NEVER git push, deploy, or merge. Stop and name the action instead.
- NEVER run destructive operations on shared/staging/prod data, enter or
  rotate secrets, or file legal documents (MEM-041 prepares; the operator files).
- NEVER weaken a security invariant to get a gate green (RLS fail-closed,
  deny-by-default access, consent default-deny, DEC-2701 no-raw-bodies,
  DEC-2721 Layer-1-wins, DEC-2723 gateway-only model calls). If a task seems
  to require it, that is a fork: ledger it, route the task back, continue.
- Operator-decision forks (genuinely direction-setting, costly to reverse):
  ledger the question with your recommended default, route the task back,
  continue with the next one. Do not wait.
- Review checklists on the cards are for the human; you do not self-approve.
  in_review is your terminal state for a task.

If the environment cannot build Rust (sandbox), author the changes anyway,
route gates through the Mac-gate loop (Desktop Commander on the operator's
machine per docs), and record the gate transcript in the ledger. No recorded
green gate = the task stays implementing.

Sanity check before your first commit each session: git status is clean apart
from your work, you are on auto/memory-enterprise, and backlog.yaml parses
(python -c "import yaml,sys; yaml.safe_load(open('docs/improvement/memory/backlog.yaml'))").

Report at milestones ("MEM-0NN in_review, moving to MEM-0MM"), one line each.
Reporting is not pausing. Begin now.
```

---

## Prompt B - review session (human, agent-assisted)

Run this when `backlog.yaml` shows tasks `in_review`. The agent prepares the packet; you decide. Only you set `done`.

```text
REVIEW: memory improvement program.

Load docs/improvement/memory/backlog.yaml and list every task in_review on
branch auto/memory-enterprise, oldest first. Then for each task, one at a time:

1. Show me: the card (from tasks-phase-*.md), the commit diff (git show, full,
   no elision of security-relevant hunks), the gate transcript from the ledger,
   and the golden-runner delta if it ran.
2. Walk the card's acceptance bullets one by one: point at the exact code or
   test satisfying each. Any bullet you cannot point at = flag it, do not
   argue it.
3. Walk the card's "Review (human)" checklist and prepare what I need for
   each item: the probe command ready to paste (I will run security probes
   myself), the file to read, or the question to decide. For security-class
   tasks (MEM-001..004, 015, 025, 032, 040, 042, 053) also run an adversarial
   pass: try to name one input or path that defeats the change, and show me
   why it does not (or flag it).
4. Wait for my verdict per task:
   - "approve MEM-0NN"  -> set done in backlog.yaml, commit
     "review: MEM-0NN approved".
   - "reject MEM-0NN: <reason>" -> set ready_to_implement with review_note,
     commit, and ledger it. Do not fix it in this session unless I say so.
   - "defer" -> leave in_review, move on.
5. After all verdicts: summarize phase progress, list newly unblocked tasks,
   and tell me whether a phase gate closed. If a phase closed, remind me of
   the operator actions now due (push the branch, and any §2.2 items the
   ledger named: filings, secrets, deploys, anchor keys).

Rules: you never set done on your own judgment; you never push; you present
evidence, not persuasion. If the diff and the ledger disagree, the diff wins
and you flag the ledger. Begin with the list.
```

---

## Operator quick reference

- Kick off implementation: paste prompt A. Safe to re-paste any time; it resumes from `backlog.yaml`.
- Review: paste prompt B when the ops rhythm suits (suggested: end of each working day while P0 is open, then per phase).
- Phase close = every task in the phase `done`. Then you: `git push` the branch, open the PR (or merge per repo habit), and run the named §2.2 operator actions from the ledger.
- Emergency stop: just stop the session; state lives in `backlog.yaml` + ledger, nothing is lost.
- Changing scope: edit the card + `backlog.yaml` (they are docs like any other); the next prompt-A session picks up the change. Keep the report as the why; keep cards as the how.
