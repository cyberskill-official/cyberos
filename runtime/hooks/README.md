# `runtime/hooks/` — CyberOS hooks (Aspect 5.1)

Pre- and post-operation callbacks that fire during BRAIN mutations. Lets operators inject organisation-specific policy (gatekeeping, notification, augmentation) without forking the runtime.

## Hook contract

A hook is a Python callable invoked by `brain_writer.py` at well-defined points. Signature:

```python
def hook(event: dict, brain_root: Path) -> dict:
    """Returns a (possibly-modified) event, OR raises HookReject to abort the write."""
```

## Hook points

| Point | When it fires | Use case |
| --- | --- | --- |
| `pre_write` | Just before any BRAIN file is mutated | Veto a write; rewrite the payload; add audit metadata |
| `post_write` | Just after the write commits | Notify downstream systems; emit a metric |
| `pre_audit` | Before an audit row is appended | Validate audit row schema; add correlation IDs |
| `pre_session_start` | At the start of a session | Set per-session policy (e.g. read-only mode) |

## Built-in hooks

| File | Purpose |
| --- | --- |
| [`gateguard.py`](gateguard.py) | Default gatekeeper: blocks writes that violate scope rules, ACLs, or source-tier policy |

## Configuring hooks

Hooks are wired in `.cyberos-memory/manifest.json`:

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

- Aspect 5.1 in the operator manual: [`../../docs/memory/README.md`](../../docs/memory/README.md) Part 26.5.1
- Writer that invokes hooks: [`../lib/brain_writer.py`](../lib/brain_writer.py)
