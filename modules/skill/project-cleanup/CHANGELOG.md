# project-cleanup — CHANGELOG

## v1.0.1 — 2026-05-17 (find_fragments.py — skip fenced code blocks)

### Changed

- `find_fragments.py::find_broken_links` now strips fenced code blocks (```…```, ~~~…~~~) and inline code spans (`…`) before scanning for `[text](path.md)` patterns. Eliminates false positives where source-code examples or YAML snippets inside docs reference example filenames (e.g. `./part-1.md` inside a Rust string literal).
- Also skips link paths containing template placeholders `{` or `<` (e.g. `./part-{}.md`, `<ts>.md`).

### Verification

- Re-ran on cyberos repo (post-link-fixes pass): `broken_links: 0` (was 4 — all four were false positives from TASK-CHAT-009 code blocks).

## v1.0.0 — 2026-05-17 (initial release)

Genesis release. Generic 4-phase cleanup skill with cyberos-flavor detection.

Phases:
1. Inventory — read-only scan for fragments + leftovers + orphans
2. Absorb + merge — consolidate small markdown into suitable parent docs (operator approves per-fragment)
3. Delete leftovers — confirm + remove orphan/stale/backup files (HITL-gated)
4. Verify state — flavor-aware checks: task DAG coherence + audit-score grep (cyberos) OR broken-link + orphan-ref check (generic)

Helper scripts:
- `find_fragments.py` — phase 1 scanner
- `propose_absorbs.py` — phase 2 merge-plan generator
- `coherence_check.py` — phase 4 cyberos task DAG checker
- `gen_module_readmes.py` — phase 4 cyberos per-module README regen
- `gen_impl_artifacts.py` — phase 4 cyberos IMPLEMENTATION_ORDER + SPRINT_PLAN regen
- `generic_verify.py` — phase 4 generic broken-link + orphan check

Pinned at v1.0.0 — bump on any script behavior change or new phase.
