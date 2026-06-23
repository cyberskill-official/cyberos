#!/usr/bin/env python3
"""Stop / SubagentStop hook: refuse to end the turn while the evidence gate is red.

This upgrades the AUTO_WORK "evidence gate" from a prompt-enforced convention to a
*structural* one: when the agent tries to stop, run the project's full
test+lint+typecheck command; if it exits non-zero, block the stop and feed the
failing output back so the agent must fix it (Claude Code hooks: a Stop hook that
runs the suite and blocks completion on failure).

Loop guard: Claude Code sets ``stop_hook_active=true`` on the event once a Stop
hook has already fired this turn; we honor it and do not re-block, preventing an
infinite test->fix->test loop from wedging the session.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

_ROOT = Path(__file__).resolve().parents[3]
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

from harness.stage0_verification.config import find_gate_command  # noqa: E402
from harness.stage1_measurement.runner import _safe_run  # noqa: E402

_MAX_OUTPUT = 4000  # chars of gate output to feed back


def _tail(text: str, limit: int = _MAX_OUTPUT) -> str:
    return text if len(text) <= limit else "…(truncated)…\n" + text[-limit:]


def main(
    stdin_text: str,
    *,
    cwd: str | None = None,
    runner=_safe_run,
    timeout: float = 900.0,
) -> tuple[int, str, str]:
    """Return (exit_code, stdout, stderr).

    ``runner`` is injectable so tests can drive the gate without spawning processes.
    Defaults to the process-group-isolating runner with a timeout so a hung gate
    (a test that waits on stdin, or a watcher holding the output pipe) can't wedge
    the turn forever — the same failure mode that bit the Stage-1 eval.
    """
    cwd = cwd or os.getcwd()
    try:
        event = json.loads(stdin_text) if stdin_text.strip() else {}
    except json.JSONDecodeError:
        event = {}

    if event.get("stop_hook_active"):
        # A previous Stop hook already fired this turn — do not loop.
        return 0, "", ""

    gate = find_gate_command(cwd)
    if not gate:
        # No gate configured: nothing to enforce, allow stop.
        return 0, "", ""

    try:
        proc = runner(
            gate,
            shell=True,  # nosec B604 - runs the operator's gate command, never agent input
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        return (
            2,
            "",
            (
                f"Stop blocked: evidence gate `{gate}` did not finish within {timeout:.0f}s "
                "(a hung test or watcher). Make the checks terminate, then try again."
            ),
        )
    out = (proc.stdout or "") + (proc.stderr or "")
    if proc.returncode == 0:
        return 0, "", ""

    reason = (
        f"Stop blocked by Stage-0 evidence gate: `{gate}` exited {proc.returncode}.\n"
        "Do NOT stop. Fix the failing checks (never weaken/skip a test) and try again.\n"
        "----- gate output (tail) -----\n" + _tail(out)
    )
    return 2, "", reason


if __name__ == "__main__":  # pragma: no cover
    code, out, err = main(sys.stdin.read(), cwd=os.getcwd())
    if out:
        sys.stdout.write(out)
    if err:
        sys.stderr.write(err)
    sys.exit(code)
