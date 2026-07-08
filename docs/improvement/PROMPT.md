# Trigger prompt - agent implementation, human review

Two parts: the agent prompt (paste into a fresh session to start implementation) and the human review protocol (what the operator does when a task reaches `review`). The short form also works once an agent can read this file:

> Implement IMP-004 per docs/improvement/PROMPT.md

or, to let the agent pick:

> Work the improvement backlog per docs/improvement/PROMPT.md

---

## Part 1 - agent prompt

Copy everything inside the block. Replace `{TASK_IDS}` with specific IDs (e.g. `IMP-004, IMP-005`) or leave the placeholder line out to let the agent pick the next eligible tasks.

```
You are running an AUTO_WORK implementation session on the CyberOS improvement backlog.

TASKS: {TASK_IDS}
(If no IDs given: open docs/improvement/BACKLOG.md and take the next eligible tasks in order -
status todo, all depends_on done, wave order then ID order. Work up to 3 tasks this session,
one at a time, never in parallel branches.)

READ FIRST, in this order:
1. docs/improvement/README.md (conventions, lifecycle, safety invariants)
2. docs/improvement/BACKLOG.md (status + eligibility)
3. The task block(s) in the wave file - the block is the binding spec
4. The report section named in refs: docs/strategy/cyberos-deep-audit-and-auto-evolution-plan-2026-07-06.md
5. docs/CONTINUE-HERE.md and any docs the task block links

ENVIRONMENT FACTS:
- Repo: ~/Projects/CyberSkill/cyberos on the operator's Mac.
- If you are in a Cowork sandbox: the mounted repo cannot run cargo builds and cannot take git
  writes from the sandbox. Author files on the mount with file tools; run every build, test,
  gate, and git command on the Mac via Desktop Commander (start_process / interact_with_process).
  If you have direct shell access to the Mac instead, use it the same way.
- Dev stack: scripts/dev (see its README) for anything needing live services.
- Toolchain: rustc pinned by services/rust-toolchain.toml; node version in .nvmrc.

PER-TASK LOOP (repeat for each task):
1. Branch from latest main: auto/imp-NNN-<short-slug>. Never work on main.
2. Flip the task to doing in docs/improvement/BACKLOG.md (commit together with first real change).
3. Implement exactly the task block scope. Scope discipline is strict: anything discovered
   out-of-scope becomes a note in the ledger entry (proposed as a new task), not extra diff.
4. Self-verify continuously - do not pause to ask when the spec already answers it. Only stop
   for a genuine fork the spec cannot resolve; record the question in the ledger, mark the task
   blocked, and move to the next eligible task.
5. Gates before every commit that touches the area:
   - Rust: cargo fmt --all --check; cargo clippy -p <crate> -- -D warnings; cargo test -p <crate>
     for every touched crate.
   - Module work: scripts/caf_gate.sh <module> and scripts/awh_ai_gate.sh <module> where the
     module has a profile.
   - Web: npm run typecheck/lint/test in apps/web (and the Playwright flow when touched).
   - CI/workflow changes: validate syntax locally (act or a scratch push to the branch only).
   - Python (modules/): ruff + pytest for the touched package.
6. Prove the acceptance checklist. Every box needs evidence you actually produced this session
   (test output, command output, screenshot path, measured number). Seeded-failure proofs
   ("gate turns red") run on a scratch branch or are reverted within the same branch.
7. Update in one final commit on the branch: task checkboxes in the wave file, BACKLOG.md
   status doing -> review, and a LEDGER.md entry in the required format (branch, commits,
   gates run with results, evidence, sensitive paths, notes).
8. Commit style: conventional prefix + task id, e.g. "ci(imp-001): add cargo-deny advisories
   gate". Small commits; each one gate-clean.
9. Produce the review packet (format below) as your final output for the task.

HARD STOPS - never do these; hand to the operator instead:
- git push, merging to main, tagging, or anything that triggers deploy.
- Deploying, restarting, or rolling production or staging containers.
- Creating, rotating, or reading secret values; adding secrets to any file.
- Touching dream-loop denylist paths (auth security invariants, audit chain internals, RLS
  policies, PII handling, cost ledger, secrets, deploy tooling) unless the task block names
  them; when it does, isolate those diffs in dedicated commits and flag them in the packet.
- Changing modules/cuo/config/dream.yaml mode, FR statuses outside your tasks, or anything
  under docs/legal/ beyond what a task block specifies.
- Any irreversible action (data deletion, force-push, history rewrite).

REVIEW PACKET (one per task, verbatim headings):
# Review packet IMP-NNN <title>
- Branch + commits:
- What changed and why (5 lines max):
- Acceptance checklist: each box with its evidence pointer
- Gates: commands + results (paste the tail lines)
- Sensitive paths touched: none | list + justification
- Risk and rollback: how to revert (branch delete / revert commit / config flip)
- Operator actions needed: push? deploy? secret? external config? (exact commands)
- Proposed follow-ups: new-task one-liners, if any

SESSION END: even if nothing completed, append a ledger entry stating what was attempted and
why it stopped. Leave the tree clean (no uncommitted changes, no stray files).
```

---

## Part 2 - human review protocol

When a task sits in `review` (the agent's packet is the input):

1. Read the packet, then the diff (`git diff main..auto/imp-NNN-*`). The packet claims; the diff is the truth.
2. Checklist - all must hold before merge:
   - [ ] Diff matches the task block scope; no unrelated files, no scope creep.
   - [ ] Every acceptance box has evidence produced this run (not asserted, shown).
   - [ ] Gates re-run clean on the Mac for at least the touched crates/modules (spot re-run,
         do not trust pasted output alone for security-relevant tasks).
   - [ ] No secrets, tokens, or real personal data anywhere in the diff.
   - [ ] Sensitive paths section: empty, or each entry justified by the task block.
   - [ ] Tests would fail if the feature broke (no defensive asserts; spot-check one).
   - [ ] Docs/env examples updated where behavior changed.
   - [ ] Rollback path stated and plausible.
3. Verdict:
   - Approve: merge the branch to main (operator does the push), flip BACKLOG.md to done,
     append a ledger entry "review -> done" with your name and any conditions.
   - Rework: append a ledger entry listing the defects, flip back to doing, re-trigger the
     agent with "Rework IMP-NNN per ledger entry <date>".
   - Reject: flip to todo or blocked with the reason; the branch is deleted or parked.
4. If the packet lists operator actions (push, deploy, secrets, external config), do them only
   after the merge decision, following the exact commands in the packet - or reject the packet
   if the commands look wrong.
5. Escalations that deserve a DEC entry in the decision ledger: anything touching the dream
   envelope, auth/audit/RLS, spend thresholds, or the first enablement of a new autonomy stage
   (IMP-027, IMP-030, IMP-061).

Cadence suggestion: batch reviews once a day; approve in dependency order so downstream tasks
unblock. Keep sessions to at most 3 tasks so packets stay reviewable.
