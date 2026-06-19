#!/usr/bin/env python3
"""PreToolUse hook: deny destructive commands and secret reads BEFORE they run.

Wire-up (settings.json): a PreToolUse hook with matcher "Bash|Read|Edit|Write".
Contract: exit 2 => block the tool call; stderr is returned to Claude as the
reason. exit 0 => allow.
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

# Bootstrap: make `harness` importable when invoked as a standalone script path.
_ROOT = Path(__file__).resolve().parents[3]
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

from harness.stage0_verification.config import load_policy  # noqa: E402
from harness.stage0_verification.policy import evaluate_event  # noqa: E402


def main(stdin_text: str, *, cwd: str | None = None) -> tuple[int, str, str]:
    """Return (exit_code, stdout, stderr).

    Fails *open* only on malformed input (never silently blocks legitimate work on
    a parse error), but applies the strict default policy whenever it can read the
    event — deny always wins.
    """
    try:
        event = json.loads(stdin_text) if stdin_text.strip() else {}
    except json.JSONDecodeError:
        return 0, "", ""
    policy = load_policy(cwd or os.getcwd())
    decision = evaluate_event(event, policy)
    if decision.block:
        return 2, "", decision.reason + "\n"
    return 0, "", ""


if __name__ == "__main__":  # pragma: no cover
    code, out, err = main(sys.stdin.read(), cwd=os.getcwd())
    if out:
        sys.stdout.write(out)
    if err:
        sys.stderr.write(err)
    sys.exit(code)
