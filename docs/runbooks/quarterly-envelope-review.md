# Quarterly envelope review ritual

**Task:** TASK-IMP-062  
**Cadence:** once per calendar quarter  
**Owner:** CTO (or delegated session operator)  
**Depends on:** historical TASK-IMP-027 (closed won't-do for auto-mode); this
ritual remains **manual** for CyberOS 1.x.

## Purpose

Replay the quarter's governed work against the docs/skills / ship-tasks
envelope: what agents were allowed to do, what incidents taught us, and whether
allow/deny edges and judge anchors still match reality. No automation is
required beyond this checklist and a calendar reminder.

## Calendar reminder stub

| Field | Value |
|---|---|
| Title | CyberOS quarterly envelope review |
| Recurrence | Quarterly (first business Monday of Jan / Apr / Jul / Oct) |
| Duration | 60–90 minutes |
| Invite | CTO + on-call docs/skills owner |
| Location | Working session (repo checkout + BACKLOG open) |
| Prep | Export last quarter's batch notes under `docs/batches/` |

Add the same row to the team calendar manually; this file is the source of the
invite text.

## Checklist

Copy into the session notes; check items as you go.

### 1. Intake

- [ ] List batches closed this quarter (`docs/batches/batch-*`).
- [ ] List incidents / gate failures that changed agent behaviour.
- [ ] Skim `docs/adrs/README.md` for any ADR accepted this quarter.

### 2. Envelope edges

- [ ] Confirm payload vs platform scope still matches [ADR-003](../adrs/ADR-003-payload-vs-platform-scope.md).
- [ ] Review HITL gate discipline ([ADR-005](../adrs/ADR-005-hitl-acceptance-gates.md)) — any silent bypasses?
- [ ] Review skill / workflow allow surfaces that agents actually used vs docs.

### 3. Allowlist / denylist

- [ ] Re-derive allow/deny from incidents (not from aspiration).
- [ ] Update runbooks or ADRs if an edge changed; do not only edit chat memory.

### 4. Judge / eval anchors (if evals exist)

- [ ] Spot-check LLM-judge anchors against fresh human grades where applicable.
- [ ] Note gaps as new IMP drafts — do not silently widen auto-mode.

### 5. Close-out

- [ ] Write a short quarter note under `docs/batches/batch-QYY-envelope-review.md`.
- [ ] File follow-up tasks in `docs/tasks/BACKLOG.md` (draft → author later).
- [ ] Schedule the next quarter's reminder.

## Pointers

- Backlog theme: docs/governance improvements (IMP-055…).
- Deep-audit Stage 5 note: quarterly envelope review in
  `docs/strategy/cyberos-deep-audit-and-auto-evolution-plan-2026-07-06.md`.
