---
id: TASK-MEMORY-302
title: "applier.py raw-writes artefacts to the BRAIN root, bypassing the canonical writer and AGENTS.md §2"
template: task@1
type: bug
module: memory
author: "@stephencheng"
department: engineering
status: draft
priority: p1
severity: sev2
created_at: 2026-07-15T12:00:00+07:00
ai_authorship: co_authored
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
first_bad_commit: null
regression_test: modules/memory/tests/test_store_layout.py::test_no_noncanonical_top_level_dirs
incident: null
---

# applier.py raw-writes artefacts to the BRAIN root

## Reproduction

```
1. python3 -m cyberos doctor
2. observe: [ERROR] layout-root-canonical — unexpected top-level entries:
   ['impl-plans/', 'code-reviews/', 'audits/', 'obs-injections/', 'adrs/']
3. overall: FAIL (total 13, pass 12, error 1)
```

**Environment**: any repo where `ship-tasks` has run at least once. **Frequency**: always, once the ADR / code-review / impl-plan / obs-injection appliers have fired.

## Expected vs observed

| | |
|---|---|
| **Expected** | AGENTS.md §2 fixes the canonical top-level set: `manifest.json`, `HEAD`, `.lock`, `audit/`, `memories/<kind>/<hex>/<hex>/`, `meta/ company/ module/ member/ client/ project/ persona/`, `conflicts/`, `exports/`, `index/`. Artefacts belong under `memories/<kind>/`. §14.1: all chain-touching operations route through the canonical writer. |
| **Observed** | Five non-canonical dirs at the store root, holding 9 files written by `Path.write_text()` — never through `put()`, so no audit row and no recorded `content_sha256`. The chain does not know these files exist. |

## Blast radius

- **Who is affected**: every repo running `ship-tasks`. `cyberos doctor` reports FAIL, which is the signal operators are told to trust.
- **Since when**: predates the task->task rename. The rename did not create these dirs — `--emit-brain-ops` derives every path from `store.rglob("*.md")`, so it can only rename within a layout that already existed. Proven: the same five dirs appear in the first BRAIN inspection of 2026-07-14, before any op ran.
- **Workaround**: none needed today — see below.
- **Data integrity**: **no corruption, and this is the important part.** The chain is intact: 252,940 records, `ledger-link-invariant` / `ledger-hash-invariant` / `ledger-crc-tail` / `ledger-mmr-cross-check` all PASS. These 9 files are *invisible* to the chain, not *inconsistent* with it. Nothing to backfill; the fix is to relocate them and route future writes through the writer.

## Root cause

`modules/cuo/cuo/core/applier.py:747`:

```python
adrs_dir = repo_root / ".cyberos/memory/store" / "adrs"
adrs_dir.mkdir(parents=True, exist_ok=True)
...
adr_path = adrs_dir / filename
adr_path.write_text(body, encoding="utf-8")
```

Four sibling appliers do the same (`:487` audits, `:909` impl-plans, plus code-reviews and obs-injections). Each computes a store-relative path itself, `mkdir`s it, and writes raw. Two rules break at once:

1. **§2 layout** — the path is invented, not `memories/<kind>/<hex>/<hex>/`.
2. **§14.1 writer discipline** — a raw `write_text` emits no audit row, so the file has no `content_sha256` on the chain and no provenance.

It failed silently because the walker's *chain* invariants only inspect the chain, and a file the chain never heard of cannot make it inconsistent. Only `layout-root-canonical` — which inspects the filesystem — could see it. It is the one invariant that looks outside.

## Fix

Route the five appliers through `cyberos.core.ops.put()` with an explicit `kind`, and let the writer own path computation. Sketch:

```python
from cyberos.core.ops import put
put(f"memories/decisions/{shard(adr_id)}/{filename}", body, meta={"kind": "decisions", ...})
```

Then one migration pass to move the 9 existing files under `memories/<kind>/` as `move()` ops — same protocol-legal pattern as the task->task BRAIN rename, for the same reason.

## Regression test

```
modules/memory/tests/test_store_layout.py::test_no_noncanonical_top_level_dirs
```

Asserts the store's top-level set is a subset of AGENTS.md §2. Red today (five extra dirs), green after the fix.

**REGRESSION-002 caveat**: `first_bad_commit` is null and the store is gitignored, so the red-at-`HEAD~1` proof cannot be run by worktree checkout. The test is red at `HEAD` *right now* against the live store, which is the same evidence in a different shape — record that terminal as the red run.

## Edge cases

| category | trigger | covered by |
|---|---|---|
| boundary | a store with zero artefacts written yet | `test_clean_store_passes` |
| malformed | an applier writing a *canonical* dir name with a bad shard depth | `test_shard_depth_enforced` |
| security | path traversal via a crafted `adr_id` (`../../etc`) — raw `mkdir` has no §3.3 check, `put()` does | `test_put_rejects_traversal` |

## Prevention

The class: **a write path that bypasses its own protocol's writer.** §14.1 exists precisely to stop this, and nothing enforced it. The walker cannot catch what the chain never saw — only the filesystem-facing invariant could, and it took a rename to make anyone run `doctor` and read the output.

Worth considering: a test that greps for `write_text` / `open(..., "w")` under any path containing `memory/store`, outside `cyberos/core/`. A convention nothing checks is a convention nothing follows.
