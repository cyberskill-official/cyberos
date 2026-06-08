"""FR-CUO-101 in-memory checkpointer scaffold."""

from __future__ import annotations

import time
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class Checkpoint:
    run_id: str
    tenant_id: str
    seq: int
    state: dict[str, Any]
    ts_ns: int


class InMemoryCheckpointer:
    """LangGraph-compatible checkpointer contract for slice 2."""

    def __init__(self) -> None:
        self._rows: dict[str, list[Checkpoint]] = {}

    def save(self, run_id: str, tenant_id: str, state: dict[str, Any]) -> Checkpoint:
        rows = self._rows.setdefault(run_id, [])
        cp = Checkpoint(run_id, tenant_id, len(rows) + 1, dict(state), time.time_ns())
        rows.append(cp)
        return cp

    def latest(self, run_id: str) -> Checkpoint | None:
        rows = self._rows.get(run_id) or []
        return rows[-1] if rows else None


def state_version_supported(version: int, *, current: int = 1) -> bool:
    return current <= version <= current + 2
