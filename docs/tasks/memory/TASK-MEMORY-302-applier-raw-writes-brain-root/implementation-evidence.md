# TASK-MEMORY-302 — implementation evidence

**Date:** 2026-07-23  
**Branch:** `cursor/post-120-followups-c0a7`

## Change

`modules/cuo/cuo/core/applier.py` no longer raw-writes under `.cyberos/memory/store/{adrs,impl-plans,audits,code-reviews,obs-injections}/`.

New helper `_put_brain_artefact` routes through `cyberos.core.ops.put` under:

| artefact class | kind (AGENTS.md §2) |
|----------------|---------------------|
| adrs | decisions |
| impl-plans | projects |
| audits | refinements |
| code-reviews | refinements |
| obs-injections | facts |

Paths are `memories/<kind>/<hex>/<hex>/<filename>`. If the writer/store is unavailable, appliers **refuse** the store-root fallback (audit sibling-of-task remains for the no-store case only).

## Regression

`modules/memory/tests/test_store_layout.py` — canonical top-level set + shard shape.

## Residual

Live stores that already have stray dirs need an operator `move` (same class as MEMORY-303 repair). This task stops **new** contamination.
