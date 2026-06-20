# CRITIC.md — the self-improvement cycle (meta-prompt)

This file is how AUDIT.md improves itself. It is a complete, self-contained
procedure that an AI agent (Claude Code, Cursor, Gemini CLI, Codex — or a
human) runs against this repository. Each cycle applies **at most one** change
to the protocol, gated by the eval harness. There is no limit on how many
cycles may run over the project's lifetime; each individual cycle, however,
must end at a defined stop point.

Paste what follows into your agent from this repo's root, or just say:
**"Run one improvement cycle per core/improve/CRITIC.md."**

---

## THE CYCLE PROMPT

You are a prompt engineer maintaining AUDIT.md, an agent protocol for
evidence-based codebase audits. You value a stable, short protocol over a
clever, long one. Your job this cycle: find where AUDIT.md's wording lets an
agent satisfy the letter while violating the intent, fix the single
highest-leverage instance, and prove you didn't break anything.

### Step 1 — Gather evidence (read, in order)
1. `AUDIT.md` — the current protocol.
2. `CHANGELOG.md` — what changed recently and why (do not repeat a reverted edit).
3. `core/improve/FAILURE_LOG.md` — open failures and how often each recurred.
4. `core/improve/retros/` — the 3 most recent retrospectives, if any.
5. Eval status: run `python3 core/evals/validate.py --all` and read the summary.

### Step 2 — Critique (severity-weighted, no quota)
Identify up to 3 weaknesses where the protocol's wording permits
letter-over-intent behavior, ranked by severity (Critical / High / Medium /
Low). Judge ONLY against evidence: a failure-log row, a retro item, an eval
gap, or a concrete exploit you can describe ("an agent could write X and
technically comply"). Speculative style preferences are not findings.
**Finding nothing >= High is a valid outcome** — record "No significant
findings this cycle — rationale: ..." and go to Step 6.

### Step 3 — Propose ONE minimal change
For the single highest-severity finding only:
- Draft the smallest wording change that closes it (a line, not a section).
- Classify it: PATCH (clarity) / MINOR (new rule or vector) / MAJOR (restructure).
- Check the instruction budget: if the change ADDS net rules, state which
  existing text you trimmed to pay for it. AUDIT.md must stay under 200 lines.
- New rules require evidence the failure recurred (Rule of Three; see
  FAILURE_LOG.md) — a single observation gets logged, not codified.

### Step 4 — Apply, version, gate
1. Apply the edit to `AUDIT.md`. Bump the version in its title line.
2. If the eval harness can test the new/changed behavior, add or update a
   fixture in `core/evals/fixtures/` and register it in `core/evals/rules.json` in the
   SAME cycle. A rule the harness cannot see will silently rot.
3. Run `python3 core/evals/validate.py --all`. ALL fixtures must pass — any
   previously-green fixture that breaks means revert or fix before release.
4. Copy the released file to `core/improve/versions/AUDIT-v<x.y.z>.md` (immutable).
5. Append a CHANGELOG.md entry: the change, the trigger (cite the failure-log
   row / retro item / eval gap), and the eval result.
6. Update the failure-log row's "Promoted to version?" column if applicable.

### Step 5 — Retrospective
Fill `core/improve/retros/<date>-cycle-<n>.md` from `core/improve/RETROSPECTIVE.md`
(score the protocol-editing run itself: was the change minimal, evidenced,
eval-gated, logged?).

### Step 6 — Stop decision (per-cycle AND per-campaign)
A "campaign" is a series of consecutive cycles in one sitting. STOP the
campaign when ANY of:
  (a) 2 consecutive cycles produced zero findings >= High (diminishing returns);
  (b) every open failure-log row is promoted or explicitly deferred;
  (c) a human asked for a fixed number of cycles and it is reached.
Record which condition fired in the last cycle's retro. Future campaigns may
always run later — the loop has no lifetime cap, only per-campaign stop rules.

### Hard rules for the critic itself
- ONE protocol change per cycle. Batching edits destroys attribution.
- Never edit files in `core/improve/versions/` (immutable history).
- Never weaken an eval fixture to make a change pass. If a fixture is wrong,
  fixing it IS the cycle's one change, with its own changelog entry.
- Never delete a failure-log row; mark it promoted/deferred instead.
- Test the changed protocol on the NEXT real run before trusting it: if the
  next retro scores below the previous baseline, revert (new PATCH version,
  changelog entry "revert — failed live validation").

---

## Escalation path (when manual cycles stop being enough)

Stay manual while runs are bespoke and judged by human review. Graduate to
automated optimization ONLY when all three hold: (1) one standardized,
repeating project type; (2) a single measurable success metric; (3) ~50+
labeled examples. Then evaluate, in order: promptfoo (regression-style eval
runner; `--repeat 3` for non-determinism), DSPy, GEPA. Until then, this
file is the optimizer.
