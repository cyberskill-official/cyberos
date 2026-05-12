#!/usr/bin/env python3
"""
cyberos_hooks.py — install / remove / inspect Claude Code hook integrations.

Aspect 5.1 (operator-surface) of the Layer-1 improvement catalog.

The actual hook scripts live at:
  - runtime/hooks/gateguard.py            (PreToolUse, 3-stage DENY/FORCE/ALLOW)
  - runtime/hooks/refinement_candidates.py (Stop-hook, §0.4 candidate detection)

This tool wires them into ~/.claude/settings.json. It is idempotent —
re-running `cyberos hooks on` keeps the existing settings intact.

Settings file shape Claude Code expects:

    {
      "hooks": {
        "PreToolUse": [{ "command": "...gateguard.py" }],
        "Stop":       [{ "command": "...refinement_candidates.py" }]
      }
    }

Usage:
    cyberos hooks status        # show what's currently wired
    cyberos hooks on            # install both hooks (idempotent)
    cyberos hooks off           # remove both hooks (leaves other entries alone)
    cyberos hooks on --hook gateguard         # one hook only
    cyberos hooks off --hook refinement       # one hook only

NOTE: Sandboxed environments may not have write access to ~/.claude/.
In that case the tool prints the JSON snippet the operator should paste
manually. Detect via a try/except around the write.
"""
from __future__ import annotations
import argparse
import json
import os
import sys
from pathlib import Path


HOOK_DEFS = {
    "gateguard": {
        "settings_event": "PreToolUse",
        "script_rel": "runtime/hooks/gateguard.py",
        "purpose": "3-stage DENY/FORCE/ALLOW gate on tool use (Aspect 5.1)",
    },
    "refinement_candidates": {
        "settings_event": "Stop",
        "script_rel": "runtime/hooks/refinement_candidates.py",
        "purpose": "auto-detect §0.4 refinement candidates at session end (Aspect 3.1)",
    },
}
# Friendly aliases
ALIASES = {"refinement": "refinement_candidates", "refinements": "refinement_candidates"}


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def settings_path() -> Path:
    return Path(os.environ.get("CYBEROS_CLAUDE_SETTINGS",
                               str(Path.home() / ".claude" / "settings.json")))


def load_settings() -> dict:
    p = settings_path()
    if not p.exists():
        return {}
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return {}


def save_settings(data: dict) -> tuple[bool, str | None]:
    p = settings_path()
    try:
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
        return True, None
    except (PermissionError, OSError) as e:
        return False, str(e)


def hook_command(brain_root: Path, hook_key: str) -> str:
    spec = HOOK_DEFS[hook_key]
    script_abs = brain_root / spec["script_rel"]
    return f"python3 {script_abs}"


def has_hook(settings: dict, event: str, command_substr: str) -> bool:
    entries = ((settings.get("hooks") or {}).get(event)) or []
    for entry in entries:
        if not isinstance(entry, dict):
            continue
        if command_substr in entry.get("command", ""):
            return True
    return False


def install_hook(settings: dict, event: str, command: str) -> bool:
    """Returns True if the hook was newly added."""
    hooks = settings.setdefault("hooks", {})
    entries = hooks.setdefault(event, [])
    for entry in entries:
        if isinstance(entry, dict) and entry.get("command") == command:
            return False
    entries.append({"command": command})
    return True


def remove_hook(settings: dict, event: str, command_substr: str) -> int:
    hooks = settings.get("hooks") or {}
    entries = hooks.get(event)
    if not entries:
        return 0
    kept = [e for e in entries if not (isinstance(e, dict) and command_substr in e.get("command", ""))]
    removed = len(entries) - len(kept)
    if not kept:
        del hooks[event]
    else:
        hooks[event] = kept
    return removed


def resolve_hook_key(name: str) -> str:
    name = name.strip().lower()
    if name in ALIASES:
        return ALIASES[name]
    if name not in HOOK_DEFS:
        raise SystemExit(f"unknown hook: {name!r}; valid: {list(HOOK_DEFS)}")
    return name


def cmd_status(args):
    brain_root = find_brain()
    settings = load_settings()
    p = settings_path()
    print(f"  settings file: {p}")
    print(f"  exists:        {p.exists()}")
    print()
    for key, spec in HOOK_DEFS.items():
        substr = spec["script_rel"]
        present = has_hook(settings, spec["settings_event"], substr)
        flag = "✓ installed" if present else "✗ not installed"
        print(f"  {key:24s} {spec['settings_event']:14s} {flag}")
        print(f"      script:  {brain_root / spec['script_rel']}")
        print(f"      purpose: {spec['purpose']}")
    return 0


def cmd_on(args):
    brain_root = find_brain()
    settings = load_settings()
    targets = [resolve_hook_key(args.hook)] if args.hook else list(HOOK_DEFS)
    changed = []
    for key in targets:
        spec = HOOK_DEFS[key]
        command = hook_command(brain_root, key)
        if install_hook(settings, spec["settings_event"], command):
            changed.append(key)
    ok, err = save_settings(settings)
    if not ok:
        print(f"  ⚠ could not write settings file ({err})")
        print(f"  Paste this into {settings_path()} manually:")
        print(json.dumps(settings, indent=2))
        return 1
    if changed:
        print(f"  ✓ installed: {', '.join(changed)}")
    else:
        print(f"  ✓ already installed (no changes)")
    return 0


def cmd_off(args):
    settings = load_settings()
    if not settings:
        print(f"  no settings file at {settings_path()} — nothing to remove")
        return 0
    targets = [resolve_hook_key(args.hook)] if args.hook else list(HOOK_DEFS)
    removed_total = 0
    for key in targets:
        spec = HOOK_DEFS[key]
        n = remove_hook(settings, spec["settings_event"], spec["script_rel"])
        removed_total += n
        if n:
            print(f"  removed {n} {key} hook entry(s)")
    if removed_total == 0:
        print(f"  ✓ nothing to remove")
        return 0
    ok, err = save_settings(settings)
    if not ok:
        print(f"  ⚠ could not write settings file ({err})")
        return 1
    print(f"  ✓ saved settings — {removed_total} entries removed")
    return 0


def main():
    p = argparse.ArgumentParser(description="install / remove / inspect Claude Code hook integrations")
    sub = p.add_subparsers(dest="cmd", required=True)
    ps = sub.add_parser("status", help="show currently-wired hooks")
    ps.set_defaults(func=cmd_status)
    pon = sub.add_parser("on", help="install hooks (default: all)")
    pon.add_argument("--hook", help="name of single hook to install")
    pon.set_defaults(func=cmd_on)
    poff = sub.add_parser("off", help="remove hooks (default: all)")
    poff.add_argument("--hook", help="name of single hook to remove")
    poff.set_defaults(func=cmd_off)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
