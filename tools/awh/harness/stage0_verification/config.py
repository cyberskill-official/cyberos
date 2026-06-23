"""Config loading for Stage-0 hooks (kept separate so policy.py stays pure I/O-free).

Resolution order for the deny policy:
    1. $AWH_POLICY_FILE                      (explicit override)
    2. .awh/policy.json walking up from cwd  (per-project)
    3. built-in defaults                     (default_policy())

Resolution order for the Stop-gate command:
    1. $AWH_GATE_CMD                         (explicit shell command)
    2. .awh/gate.sh walking up from cwd      (executable script)
    3. None                                  (no gate configured -> Stop hook is a no-op)
"""

from __future__ import annotations

import json
import os
from pathlib import Path

from .policy import Policy, default_policy

__all__ = ["load_policy", "find_gate_command", "find_upwards"]


def find_upwards(name: str, start: str | os.PathLike[str]) -> Path | None:
    """Search ``start`` and each parent for a file/dir named ``name``."""
    cur = Path(start).resolve()
    for d in (cur, *cur.parents):
        candidate = d / name
        if candidate.exists():
            return candidate
    return None


def load_policy(cwd: str | os.PathLike[str] = ".") -> Policy:
    explicit = os.environ.get("AWH_POLICY_FILE")
    path = Path(explicit) if explicit else find_upwards(".awh/policy.json", cwd)
    if path and path.is_file():
        try:
            return Policy.from_dict(json.loads(path.read_text()))
        except (json.JSONDecodeError, OSError):
            # A broken policy file must fail safe to the strict defaults, never to "allow all".
            return default_policy()
    return default_policy()


def find_gate_command(cwd: str | os.PathLike[str] = ".") -> str | None:
    explicit = os.environ.get("AWH_GATE_CMD")
    if explicit:
        return explicit
    script = find_upwards(".awh/gate.sh", cwd)
    if script and script.is_file():
        return f"sh {script}"
    return None
