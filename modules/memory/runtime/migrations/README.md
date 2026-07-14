# `runtime/migrations/` — memory schema migration scripts

Numbered migrations that mutate the memory's schema or contents when the AGENTS protocol bumps a MAJOR version. Each migration is idempotent — re-running on an already-migrated memory is a no-op.

## File naming

```
NNN-<kebab-case-description>.py
```

`NNN` is zero-padded sequential. Example: `001-example-add-tag.py`.

## Migration contract

A migration file MUST define:

```python
SCHEMA_VERSION_BEFORE = "<version>"   # required: memory schema before this migration runs
SCHEMA_VERSION_AFTER  = "<version>"   # required: memory schema after success

def migrate(memory_root: Path, dry_run: bool = False) -> dict:
    """Apply this migration. Returns {'status': 'ok'|'skipped'|'error', 'changes': int}."""
    ...
```

## Running

**Single migration:**
```shell
cyberos migrate run 001-example-add-tag
```

**All pending migrations to current target:**
```shell
cyberos migrate up
```

**Dry-run (preview without writing):**
```shell
cyberos migrate up --dry-run
```

## When a migration is needed

- AGENTS protocol bumps MAJOR (e.g. v1 → v2) — schema-level break.
- A contract bumps MAJOR (e.g. `subtask@1` → `task@2`).
- A `cyberos doctor` rule changes its source-of-truth pattern.

When a migration is NOT needed: any change that's backward-compatible — adding optional fields, new memory types, new validators that grandfather existing records.

## Related

- Migration tooling: [`../tools/cyberos_migrate.py`](../tools/cyberos_migrate.py)
- Protocol upgrade flow (§0.5 + §0.6): [`AGENTS.md`](../../AGENTS.md) §0.5
