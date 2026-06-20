# T4 cross-model audit — runbook

**Goal:** have an *independent* model (Gemini, via Antigravity) run the same audit
profile Claude ran, so we can diff finding overlap + severity agreement. That diff
is the open evidence gate for the queued protocol candidates (DEPTH semantics, R2
offline) — they stay un-promoted until a real cross-model run exists.

**Tool:** Google Antigravity IDE (covered by Google One Ultra). The `gemini` CLI
also works (`gemini -p "<prompt>"`) but Antigravity is preferred for quota.

**Designated target:** `~/Projects/Personal/3d-preriodic-table`
(Claude's finding there: **1 High — L1-T1**, unguarded `localStorage` in
`DashboardLayout.tsx` → red test suite. Recorded in
`reports/2026-06-11-personal-periodic.json` + git commit `ffc0f22`.)

## Step 1 — run this prompt in Antigravity (paste into the agent)

```
Read the audit protocol at
/Users/stephencheng/Projects/CyberSkill/code-audit-framework/core/AUDIT.md
and execute Phases 0–2 (DISCOVERY ONLY — do NOT execute any fixes) on the target
repository at /Users/stephencheng/Projects/Personal/3d-preriodic-table.
The target's CONFIG is in its audit-profile.yaml (config: section); PROJECT_PATH
is that directory. Write your findings BACKLOG to docs/BACKLOG-gemini-xmodel.md —
do NOT create or modify docs/BACKLOG.md (Claude's run owns that file). Stop at the
approval gate. Begin at PHASE 0.
```

Why the separate file: it keeps Gemini's findings from clobbering Claude's backlog,
so the two are diffable side by side. (Claude's original is also preserved in git.)

## Step 2 — tell Claude when it's done

Say "the gemini cross-model backlog is ready" (or paste it). Claude will:
1. Diff Gemini's findings vs Claude's: **overlap** (did it find the localStorage
   High?), **severity agreement**, **extras** (found something Claude missed?),
   **over-calls** (flagged a non-issue?).
2. Record the result in `records/2026-06-11-personal-periodic.yaml` → `cross_model:`
   (currently `null`), and note whether it unblocks the queued protocol candidates.

## Optional — second target

For a stronger signal, repeat on `~/Projects/Personal/claude-certified-architect-mock-exam`
(Claude: 1 High L1-T1, BLOCKED — un-versioned scoring RPC). Does Gemini also reach
BLOCKED, or does it over-claim exploitability? Either is useful calibration data.
