"""Skill invoker — shells out to `cyberos-skill run`.

Phase 1 dispatch path. The skill module owns the actual execution
(WASM / script / Python runner); CUO is just the orchestrator. We
prefer a release-built binary, fall back to a debug build, fall back
to `cargo run`. None of those are required to be present for the
router itself to work — `route()` runs without ever invoking anything.
"""

from __future__ import annotations

import json
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass
class InvocationResult:
    skill_name: str
    exit_code: int
    output: str
    stderr: str


def _resolve_cmd(skill_module: Path) -> list[str]:
    release_bin = skill_module / "target" / "release" / "cyberos-skill"
    debug_bin = skill_module / "target" / "debug" / "cyberos-skill"
    if release_bin.is_file():
        return [str(release_bin)]
    if debug_bin.is_file():
        return [str(debug_bin)]
    return ["cargo", "run", "-q", "-p", "cyberos-skill-cli", "--"]


def invoke(
    skill_name: str,
    input_data: Any,
    skill_root: Path,
    timeout: float = 30.0,
) -> InvocationResult:
    """Run a skill via `cyberos-skill run --executor script`.

    `skill_root` is the *skills/* directory (e.g. `skill/skills/`); the
    skill module's package root is its parent.
    """
    if isinstance(input_data, (dict, list)):
        input_str = json.dumps(input_data)
    else:
        input_str = str(input_data) if input_data is not None else ""

    skill_module = skill_root.parent
    cmd = _resolve_cmd(skill_module)

    proc = subprocess.run(
        cmd + ["run", skill_name, "--executor", "script"],
        input=input_str,
        text=True,
        capture_output=True,
        timeout=timeout,
        cwd=str(skill_module),
    )
    return InvocationResult(
        skill_name=skill_name,
        exit_code=proc.returncode,
        output=proc.stdout,
        stderr=proc.stderr,
    )
