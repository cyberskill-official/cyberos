# TASK-IMP-091 repo context map

## Cone
- `scripts/migrate_improvement_to_task.py` (regen_backlog: the ACTIVE filter at 19-20 and its two use sites, the header writer, the Totals writer)
- new: `scripts/tests/test_regen_backlog.sh`

## Patterns the change must follow
- **ROOT resolves from the script's own path** (`Path(__file__).resolve().parents[1]`), so a test cannot point it at a fixture with a flag - every scenario copies the script into a scratch tree and runs it there. TASK-IMP-086's recorded trial hit exactly this and is why the suite never runs the script inside the repo.
- **Row grammar is the committed convention**: `- [<status>] <STEM> - <title>` plus ` (improvement)` for `class: improvement`. backlog-mutate.mjs's insert is regenerator-identical to it; parity is byte-level in both directions.
- **Loud over silent**: TASK-DOCS-004 §1 #4 already demanded the unparseable skip be loud. It printed to stderr and wrote the file anyway - the fix promotes it to a halt before any write.
- **Suite discovery**: `scripts/tests/run_all.sh` globs `test_*.sh`; a new suite is discovered by naming alone (evidenced in the gate log, AC 4).

## Blast radius
- Files: 1 modified + 1 new. Modules: 1 (scripts).
- Cross-module edges: none. The script is operator-invoked (`--backlog`), not wired into install or gates.
- Coupling to the corpus: t01 compares against the live committed section, so the suite is sensitive to real backlog edits by design - that is the parity guarantee, not a flake.

## Module placement
Correct. Cross-cutting hardening of the index mechanism; `improvement` is its home.
