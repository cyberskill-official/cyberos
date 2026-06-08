"""Health state for the BGE-M3 sidecar."""

from __future__ import annotations

from dataclasses import dataclass
from threading import Lock


@dataclass(frozen=True)
class HealthSnapshot:
    status: str
    device: str | None
    sidecar_version: str
    model_sha256: str | None
    error: str | None = None


class HealthState:
    def __init__(self, sidecar_version: str) -> None:
        self._sidecar_version = sidecar_version
        self._lock = Lock()
        self._snapshot = HealthSnapshot(
            status="warming",
            device=None,
            sidecar_version=sidecar_version,
            model_sha256=None,
        )

    def set_ready(self, *, device: str, model_sha256: str) -> None:
        with self._lock:
            self._snapshot = HealthSnapshot(
                status="ok",
                device=device,
                sidecar_version=self._sidecar_version,
                model_sha256=model_sha256[:16],
            )

    def set_error(self, error: Exception | str) -> None:
        with self._lock:
            self._snapshot = HealthSnapshot(
                status="error",
                device=None,
                sidecar_version=self._sidecar_version,
                model_sha256=None,
                error=str(error),
            )

    def snapshot(self) -> HealthSnapshot:
        with self._lock:
            return self._snapshot
