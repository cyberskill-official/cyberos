# `runtime/hooks/` — CyberOS hooks (Aspect 5.1)

Pre- and post-operation callbacks that fire during memory mutations. Lets operators inject organisation-specific policy (gatekeeping, notification, augmentation) without forking the runtime.

## Hook contract

A hook is a Python callable invoked by `memory_writer.py` at well-defined points. Signature:

```python
def hook(event: dict, memory_root: Path) -> dict:
    """Returns a (possibly-modified) event, OR raises HookReject to abort the write."""
```

## Hook points

| Point | When it fires | Use case |
| --- | --- | --- |
| `pre_write` | Just before any memory file is mutated | Veto a write; rewrite the payload; add audit metadata |
| `post_write` | Just after the write commits | Notify downstream systems; emit a metric |
| `pre_audit` | Before an audit row is appended | Validate audit row schema; add correlation IDs |
| `pre_session_start` | At the start of a session | Set per-session policy (e.g. read-only mode) |

## Built-in hooks

| File | Purpose |
| --- | --- |
| [`gateguard.py`](gateguard.py) | Default gatekeeper: blocks writes that violate scope rules, ACLs, or source-tier policy |

## Configuring hooks

Hooks are wired in `.cyberos/memory/store/manifest.json`:

```json
{
  "hooks": {
    "pre_write": ["runtime.hooks.gateguard:check"],
    "post_write": ["myorg.notify:slack_alert"]
  }
}
```

Multiple hooks per point run in declared order; first `HookReject` short-circuits.

## Related

- Aspect 5.1 in the operator manual: [`README.md`](../../README.md)
- Writer that invokes hooks: [`../lib/memory_writer.py`](../lib/memory_writer.py)
