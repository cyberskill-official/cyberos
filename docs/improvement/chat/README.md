# CHAT improvement backlog - how this folder works

Source of truth for executing docs/strategy/chat-enterprise-grade-plan-2026-07-06.md (C1-C147).
The report says WHY and WHAT; this folder says IN WHAT ORDER and WHEN IS IT DONE. Do not re-audit the
module; if a task spec conflicts with the code you find, trust the code, note the delta in the ledger,
and adjust the task, not the report.

## Files

- BACKLOG.md - master table of all tasks (T-001..T-066). Status lives HERE and only here.
- tasks/phase-0.md .. tasks/phase-4.md - the detailed spec for every task (goal, C-refs, files,
  implementation notes, acceptance checks, review notes). Specs are static; do not edit status into them.
- LEDGER.md - append-only evidence log, one entry per task that reaches review.
- program.yaml - the adapter the `cyberos-improve-implement` / `cyberos-improve-review` skills read
  (branch, gate commands, id prefix, ledger, guardrails). Replaces the old PROMPT-IMPLEMENT.md +
  PROMPT-REVIEW.md; the shared loop now lives in `.claude/skills/cyberos/`.

## How to run

Driven by the official CyberOS skills. In Claude, invoke `cyberos-improve-implement` for this program
("work the chat improvement backlog") to advance tasks to `review`, and `cyberos-improve-review` for the
human sign-off pass. Any non-Claude agent can be handed
`.claude/skills/cyberos/cyberos-improve-implement/SKILL.md` plus this directory.

## Status vocabulary (BACKLOG.md Status column)

- ready - dependencies met, an agent may pick it up
- blocked:<reason> - waiting on a dependency task, a D-decision, or an input from Stephen
- in_progress - an agent is on it right now
- review - built + gates green + ledger entry written; waiting for Stephen
- done - Stephen accepted in review
- parked - deliberately shelved (keep the reason inline)

## Ordering rules

1. Phases run in order (0 -> 4). Inside a phase, take the first ready task top-to-bottom unless the
   Depends column says otherwise.
2. Nothing from phase 2+ ships to the whole tenant before every phase-1 task is done - features stacked
   on an unconverged sync layer produce "chat lost my message" reports (report section 6, sequencing rule).
3. A task is one gated increment: implement, gate, ledger, commit. If a task turns out bigger than a day
   or two of work, split it in BACKLOG.md (T-0NNa/T-0NNb) rather than holding a long-lived dirty tree.

## The gate (definition of green)

Backend: cargo fmt --check, cargo clippy -D warnings, cargo test, migrations apply clean on a throwaway
DB. Client: tsc --noEmit and vite build for apps/web (and the chat-core package once it exists), plus the
relevant smoke in services/chat/tests/. Gates run on Stephen's Mac via the Mac-gate loop (the sandbox
cannot build); author on the mount, gate remotely, then commit. Branch: auto/chat-enterprise. Pushing and
deploying are Stephen-confirm actions, never automatic.

## Inputs only Stephen can provide (tracked as blocked:input in BACKLOG.md)

- FCM service account + APNs key (T-023), Apple Developer + Play Console accounts and signing keys
  (T-029, T-033) - the exact list is in RELEASE.md
- Decisions D1-D6 from the report section 7 (T-036 needs D3, T-060 needs D1, T-062 needs D4)
- Counsel review for the PDPL items (T-041) and a pen-test vendor (T-058)

## Traceability

Every task lists its C-refs. Every C1-C147 appears in at least one task (verified 2026-07-06; coverage
map at the bottom of BACKLOG.md). D1-D6 are tracked in BACKLOG.md as decision rows, not tasks.
