#!/usr/bin/env python3
"""PostToolUse hook: auto-format a file right after the agent edits it.

Non-blocking by design (always exits 0): formatting is a convenience, not a gate.
Keeps diffs clean so the human review in AUTO_WORK Phase 4 sees intent, not noise.
Default: Python files -> ``ruff format``. Extendable via $AWH_FORMAT_PY.
"""

from __future__ import annotations

import json
import os
import shlex
import subprocess
import sys
from pathlib import Path

_EDIT_TOOLS = ("Edit", "Write", "MultiEdit")


def _formatter_for(path: str) -> list[str] | None:
    if path.endswith(".py"):
        custom = os.environ.get("AWH_FORMAT_PY")
        if custom:
            return [*shlex.split(custom), path]
        return ["ruff", "format", path]
    return None


def main(
    stdin_text: str,
    *,
    cwd: str | None = None,
    runner=subprocess.run,
) -> tuple[int, str, str]:
    cwd = cwd or os.getcwd()
    try:
        event = json.loads(stdin_text) if stdin_text.strip() else {}
    except json.JSONDecodeError:
        return 0, "", ""

    if event.get("tool_name") not in _EDIT_TOOLS:
        return 0, "", ""
    tool_input = event.get("tool_input") or {}
    path = str(tool_input.get("file_path") or "")
    if not path or not Path(path).is_absolute() and cwd:
        path = str(Path(cwd) / path) if path else ""
    if not path or not Path(path).exists():
        return 0, "", ""

    cmd = _formatter_for(path)
    if not cmd:
        return 0, "", ""
    try:
        proc = runner(cmd, cwd=cwd, capture_output=True, text=True, timeout=60.0)
    except (FileNotFoundError, subprocess.TimeoutExpired):
        # Formatter missing or hung — formatting is a convenience, stay silent + non-blocking.
        return 0, "", ""
    note = f"[awh] formatted {Path(path).name} ({cmd[0]})" if proc.returncode == 0 else ""
    return 0, note, ""


if __name__ == "__main__":  # pragma: no cover
    code, out, err = main(sys.stdin.read(), cwd=os.getcwd())
    if out:
        sys.stdout.write(out + "\n")
    if err:
        sys.stderr.write(err)
    sys.exit(code)
