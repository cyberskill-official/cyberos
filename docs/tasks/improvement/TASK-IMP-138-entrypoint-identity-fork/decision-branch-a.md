# TASK-IMP-138 — operator decision: Branch A (thin spine)

**Recorded:** 2026-07-23  
**Actor:** Stephen Cheng (operator chat)  
**Verdict:** **Branch A — thin spine everywhere**

> IMP-138 = Branch A (thin spine) — record the decision; do NOT fully implement IMP-138 in this turn

## Meaning

Platform repo root `AGENTS.md` becomes the same thin workflow spine consumers already get from `install.sh`. Normative memory protocol lives at `modules/memory/cyberos/data/AGENTS.md` (installed copy `.cyberos/memory/AGENTS.md`). Delete `is_platform_repo()` exception; `CLAUDE.md` becomes a pointer. Sweep links that assume "root AGENTS.md is the protocol".

## Status

- Task remains `ready_to_implement` until Batch D ships the implementation on its own branch.
- This note + the matching `source_decisions` entry on `spec.md` close the fork block; they do **not** start implementation.

## Non-goals for this record

- No file moves of `AGENTS.md` / `CLAUDE.md`
- No `install.sh` edits
- No status flip past `ready_to_implement`
- No push/merge
