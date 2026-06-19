"""Golden-set task schema and loader.

A golden task is the smallest unit the eval gate scores. Per the recommended
workflow (Anthropic + production write-ups): start with ~20 real tasks, mine
production failures into new ones, and judge the *end state* (a deterministic
``check`` command) rather than each step, since valid paths diverge.

YAML shape (one task)::

    id: fix-null-deref
    description: NullPointer in parse_config when file missing
    cmd: "python3 solver.py {task_dir}"     # the system-under-test (agent or solver)
    check: "pytest -q tests/test_parse.py"  # end-state scorer; exit 0 == pass
    weight: 1.0
    timeout_sec: 120
    workdir: cases/fix-null-deref           # optional, relative to the YAML file

A file may contain a single task mapping or ``{tasks: [ ... ]}``. A directory is
loaded by globbing ``*.yaml``/``*.yml``.

Placeholders available in ``cmd``/``check``: ``{seed}``, ``{task_id}``,
``{task_dir}`` (absolute workdir). ``{seed}`` is also exported as ``$AWH_SEED``.
"""

from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path

import yaml

__all__ = ["GoldenTask", "load_tasks", "GoldenSetError"]


class GoldenSetError(ValueError):
    """Raised on a malformed or ambiguous golden set."""


@dataclass(frozen=True)
class GoldenTask:
    id: str
    cmd: str
    description: str = ""
    check: str | None = None
    weight: float = 1.0
    timeout_sec: float = 120.0
    workdir: str | None = None

    def resolved_workdir(self, base: Path) -> Path:
        if not self.workdir:
            return base
        wd = Path(self.workdir)
        return wd if wd.is_absolute() else (base / wd)

    @staticmethod
    def from_dict(data: dict, source: str = "<dict>") -> GoldenTask:
        if "id" not in data:
            raise GoldenSetError(f"{source}: task missing required field 'id'")
        if "cmd" not in data:
            raise GoldenSetError(f"{source}: task {data['id']!r} missing required field 'cmd'")
        weight = float(data.get("weight", 1.0))
        if weight <= 0:
            raise GoldenSetError(f"{source}: task {data['id']!r} weight must be > 0")
        timeout_sec = float(data.get("timeout_sec", 120.0))
        if timeout_sec <= 0:
            raise GoldenSetError(
                f"{source}: task {data['id']!r} timeout_sec must be > 0 "
                "(a zero/negative timeout makes the task always time out)"
            )
        return GoldenTask(
            id=str(data["id"]),
            cmd=str(data["cmd"]),
            description=str(data.get("description", "")),
            check=(str(data["check"]) if data.get("check") is not None else None),
            weight=weight,
            timeout_sec=timeout_sec,
            workdir=(str(data["workdir"]) if data.get("workdir") is not None else None),
        )


def _iter_task_dicts(doc, source: str):
    if doc is None:
        return
    if isinstance(doc, dict) and "tasks" in doc:
        yield from doc["tasks"]
    elif isinstance(doc, list):
        yield from doc
    elif isinstance(doc, dict):
        yield doc
    else:
        raise GoldenSetError(f"{source}: expected a task mapping or list, got {type(doc).__name__}")


def load_tasks(path: str | os.PathLike[str]) -> list[GoldenTask]:
    """Load tasks from a YAML file or a directory of YAML files.

    Raises GoldenSetError on duplicate ids or malformed tasks (fail loud — a
    silently-dropped eval task is a silently-weakened gate).
    """
    p = Path(path)
    files: list[Path]
    if p.is_dir():
        files = sorted([*p.glob("*.yaml"), *p.glob("*.yml")])
        if not files:
            raise GoldenSetError(f"{p}: no .yaml/.yml task files found")
    elif p.is_file():
        files = [p]
    else:
        raise GoldenSetError(f"{p}: not a file or directory")

    tasks: list[GoldenTask] = []
    seen: dict[str, str] = {}
    for f in files:
        doc = yaml.safe_load(f.read_text())
        for raw in _iter_task_dicts(doc, str(f)):
            task = GoldenTask.from_dict(raw, str(f))
            if task.id in seen:
                raise GoldenSetError(
                    f"duplicate task id {task.id!r} in {f} (already defined in {seen[task.id]})"
                )
            seen[task.id] = str(f)
            tasks.append(task)
    if not tasks:
        raise GoldenSetError(f"{p}: loaded zero tasks")
    return tasks


# Kept for symmetry / future use (resolving a task's base dir when running).
def task_base_dirs(path: str | os.PathLike[str]) -> dict[str, Path]:
    p = Path(path)
    base = p if p.is_dir() else p.parent
    return {t.id: t.resolved_workdir(base) for t in load_tasks(path)}
