"""Structured tracing for routing decisions.

A minimal logger that emits one JSON line per routing event to stderr
(default) or an explicit file path. The intent is replayability: every
routing decision should be reproducible from its trace row.

Phase 2 will tie traces into the BRAIN audit chain via memory_bridge;
Phase 1 keeps the local on-disk JSONL.
"""

from __future__ import annotations

import json
import sys
import time
from pathlib import Path
from typing import IO


class Tracer:
    def __init__(self, sink: IO | None = None, path: Path | None = None):
        self._owned_handle: IO | None = None
        if path is not None:
            path.parent.mkdir(parents=True, exist_ok=True)
            self._owned_handle = path.open("a", encoding="utf-8")
            self.sink: IO = self._owned_handle
        elif sink is not None:
            self.sink = sink
        else:
            self.sink = sys.stderr

    def emit(self, event: str, **fields) -> None:
        record = {"ts_ns": time.time_ns(), "event": event, **fields}
        self.sink.write(json.dumps(record, ensure_ascii=False) + "\n")
        self.sink.flush()

    def close(self) -> None:
        if self._owned_handle is not None:
            self._owned_handle.close()
            self._owned_handle = None
