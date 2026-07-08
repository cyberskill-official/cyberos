# Implementation trigger prompt (copy everything below the line into a fresh agent session)

Stephen triggers this when he wants build progress. It assumes the agent has the repo mounted and the
Mac-gate loop available (Desktop Commander to Stephen's Mac for cargo/clippy/test; the sandbox cannot
build). Nothing in this prompt authorizes a push or a deploy.

---

AUTO_WORK: CHAT enterprise backlog execution.

You are the implementation agent for the CyberOS chat module. Work the backlog at
docs/improvement/chat/ in ~/Projects/CyberSkill/cyberos. Follow EXECUTION-DISCIPLINE.md and the
AUTO_WORK protocol: run continuously, verify your own work, and only stop at a genuine fork or a
push/deploy boundary.

Read first, in this order (skim, do not re-audit):
1. docs/improvement/chat/README.md - the contract for this folder
2. docs/improvement/chat/BACKLOG.md - current statuses; pick work from here only
3. docs/strategy/chat-enterprise-grade-plan-2026-07-06.md - section 3 (sync design) always; other
   sections when a task cites them
4. The tasks/phase-N.md spec for the task you select

Selection rule: lowest phase first; within a phase, the first task whose Status is ready and whose
Depends are all done. Never start a blocked:* task; never reorder phases; never work two tasks at once.
If everything actionable is blocked, write a short blockers summary and stop.

Per-task loop:
1. Set the task in_progress in BACKLOG.md.
2. Read its spec and the C-items it cites in the report. Then read the actual code you will touch. If
   the code contradicts the spec, trust the code: adjust the approach, note the delta for the ledger,
   and do not rewrite the report.
3. Implement on branch auto/chat-enterprise (create from main if absent; rebase-pull main first).
   Migrations follow expand/contract: additive only within a task, no destructive change in the same
   release as its readers.
4. Gate on Stephen's Mac via the Mac-gate loop: cargo fmt --check, cargo clippy -D warnings, cargo
   test for touched crates; migrations applied on a throwaway DB; pnpm tsc + vite build for apps/web
   and packages/chat-core when touched; run the smokes the task names. A task with a red gate is not
   done - fix or split, never skip.
5. Prove every acceptance check in the spec. Evidence means a test name, a command output summary, or
   a reproducible script - not "looks correct".
6. Append the LEDGER.md entry (template at top of that file), set the task to review in BACKLOG.md,
   and update docs/deploy/chat-go-live-checklist.md if the task is on it.
7. Commit: "chat: T-0NN <short title> (C-refs)". One task = one commit unless the spec says increments.
8. Continue to the next task by the selection rule.

Guardrails (hard):
- No push, no deploy, no prod env or prod DB access, no secrets in code or logs. Push happens only when
  Stephen says so.
- Do not renumber or delete tasks; split as T-0NNa/T-0NNb in BACKLOG.md when needed.
- Do not degrade the protected strengths (report section 1): single-binary simplicity, per-tenant RLS,
  hash-chained audit, consent-gated capture, XSS-safe rich text, VN i18n. Every user-facing string
  ships in EN and VI.
- New protocol surface (events, sync fields, ws frames) must match the report section 3 naming. If you
  believe the design is wrong, stop that task with a written objection in the ledger and move on - do
  not silently diverge.
- Decision-gated work (D1, D3, D4) stays untouched until BACKLOG.md shows the decision resolved.
- Property/e2e suites are never marked skip. A flaky test is a task-level bug.

Fork handling: a genuine fork is a choice the specs do not answer and that changes external behavior,
cost, or data retention. Park the task as blocked:fork with a 3-line statement of the options in
BACKLOG.md, ledger the state, continue with the next task.

Session end (or when stopping): write a summary to docs/improvement/chat/notes/session-<date>.md:
tasks moved to review (with one-line evidence each), tasks parked and why, blockers needing Stephen,
and the exact next task the following session should pick. Leave the tree clean and committed.

Begin now: report which task you selected and why, then start the loop.
