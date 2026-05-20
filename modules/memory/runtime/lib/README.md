# `runtime/lib/` — Shared library scripts

Scripts that other runtime modules depend on. Not directly invocable as CLI subcommands — but loaded by `runtime/tools/cyberos` and friends.

## Files

| File | Purpose |
| --- | --- |
| [`memory_writer.py`](memory_writer.py) | THE canonical memory-mutation API. Every `cyberos add`, `cyberos edit`, `cyberos prune` ultimately calls this. Enforces audit-ledger append, source-tier policy, scope-rule check, hooks. |
| [`apply-bundle-Q.sh`](apply-bundle-Q.sh) | Atomic rollout helper for §0.5 + §0.6 protocol upgrades. Invoked by `cyberos rollout apply`. Stages → verifies → swaps. |
| [`cleanup-host.sh`](cleanup-host.sh) | Host-filesystem cleanup of sandbox-can't-unlink leftovers. Run when sandbox-based agents leave behind `.legacy.bak`, empty husk dirs, etc. |

## Why "lib"?

Pre-Batch 26 these scripts lived under the misleadingly-named `outputs/` folder. They are SOURCE CODE that the runtime *calls* — not outputs of the runtime. UNIX convention puts shared library code in `lib/`. Generated state goes elsewhere (now in the memory cache under `.cyberos-memory/cache/`).

## Calling conventions

`memory_writer.py` is the only file here that's frequently imported. Import path:
```python
import sys; sys.path.insert(0, "runtime/lib")
import memory_writer
```

The shell scripts are invoked as commands by `cyberos rollout` and `cyberos cleanup`.

## Related

- memory protocol that `memory_writer.py` enforces: [`../../memory/docs/AGENTS.md`](../../../../AGENTS.md)
- Rollout flow that invokes `apply-bundle-Q.sh`: [`../../memory/docs/README.md` Part 26.0.5](../../memory/docs/README.md)
