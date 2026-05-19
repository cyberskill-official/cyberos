"""FR-MEMORY-110 — Capture-daemon health + auto-restart supervisor.

The capture daemon (FR-MEMORY-110 production target) runs as a background
process that watches the file system + writes memory rows. It MUST survive:

* Crashes (segfault on a malformed file, OOM, etc.) → exponential-backoff
  auto-restart up to 5 attempts, then a hard-fail with a desktop notification.
* Wedge (event-loop stops without crashing) → liveness probe; if the heartbeat
  file hasn't been touched in ``HEARTBEAT_STALE_SECS`` (default 90s), kill
  the process and restart.
* Watched-folder disappearing (USB unplugged, network mount dropped) → soft
  error, surface to UI, keep daemon alive.

This module ships the supervisor primitives:

* :class:`HeartbeatWriter` — daemon code calls ``heartbeat()`` every tick.
* :class:`HeartbeatProbe` — supervisor reads the heartbeat file + decides
  whether the daemon is alive.
* :class:`RestartPolicy` — exponential-backoff calculator with a cap.
* :class:`Supervisor` — orchestrator that spawns + monitors + restarts.

Platform-specific service registration (launchd / systemd / Task Scheduler)
lives in :mod:`cyberos.core.serve` already; this module is the supervisor
LOGIC, not the OS-wiring.
"""

from __future__ import annotations

import json
import time
from dataclasses import dataclass, field
from pathlib import Path

DEFAULT_HEARTBEAT_PATH_REL = "audit/.daemon-heartbeat.json"
HEARTBEAT_STALE_SECS_DEFAULT = 90
MAX_BACKOFF_RESTART_ATTEMPTS = 5


class DaemonHealthError(RuntimeError):
    """Supervisor decided the daemon is unrecoverable in this session."""


@dataclass
class HeartbeatWriter:
    """Daemon-side helper. Call :meth:`heartbeat` every tick."""

    store: Path
    pid: int
    rel_path: str = DEFAULT_HEARTBEAT_PATH_REL
    last_write_ns: int = 0

    def path(self) -> Path:
        return self.store / self.rel_path

    def heartbeat(self) -> None:
        now_ns = time.time_ns()
        # Throttle disk writes — only fsync at most once per second.
        if now_ns - self.last_write_ns < 1_000_000_000:
            return
        p = self.path()
        p.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "pid": self.pid,
            "ts_ns": now_ns,
            "ts_iso": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "schema_version": 1,
        }
        p.write_text(json.dumps(payload), encoding="utf-8")
        self.last_write_ns = now_ns


@dataclass
class HeartbeatProbe:
    """Supervisor-side helper. Returns ``True`` if the daemon is healthy."""

    store: Path
    stale_secs: int = HEARTBEAT_STALE_SECS_DEFAULT
    rel_path: str = DEFAULT_HEARTBEAT_PATH_REL

    def path(self) -> Path:
        return self.store / self.rel_path

    def is_alive(self, now_ns: int | None = None) -> bool:
        now_ns = now_ns if now_ns is not None else time.time_ns()
        p = self.path()
        if not p.exists():
            return False
        try:
            data = json.loads(p.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            return False
        ts_ns = int(data.get("ts_ns", 0))
        age_secs = (now_ns - ts_ns) / 1_000_000_000
        return age_secs <= self.stale_secs


@dataclass
class RestartPolicy:
    """Exponential-backoff with cap. Used by Supervisor on every crash."""

    base_secs: float = 1.0
    factor: float = 2.0
    cap_secs: float = 60.0
    max_attempts: int = MAX_BACKOFF_RESTART_ATTEMPTS

    def delay_for_attempt(self, attempt: int) -> float:
        """Seconds to sleep before restart attempt N (0-indexed)."""
        if attempt < 0:
            return 0.0
        d = self.base_secs * (self.factor ** attempt)
        return min(d, self.cap_secs)

    def should_give_up(self, attempt: int) -> bool:
        return attempt >= self.max_attempts


@dataclass
class Supervisor:
    """Tying it together. The actual subprocess launch lives in
    :mod:`cyberos.core.serve`; this class is the policy + state machine."""

    store: Path
    probe: HeartbeatProbe = field(default_factory=lambda: HeartbeatProbe(store=Path(".")))
    policy: RestartPolicy = field(default_factory=RestartPolicy)
    attempt: int = 0
    last_crash_ts_ns: int = 0
    last_crash_reason: str | None = None

    def record_crash(self, reason: str) -> None:
        self.attempt += 1
        self.last_crash_ts_ns = time.time_ns()
        self.last_crash_reason = reason

    def restart_after_secs(self) -> float:
        """How long to wait before the next launch. 0 = launch now."""
        return self.policy.delay_for_attempt(self.attempt)

    def hit_ceiling(self) -> bool:
        return self.policy.should_give_up(self.attempt)

    def succeed(self) -> None:
        """Call once the daemon has reported a healthy heartbeat post-restart.
        Resets the backoff counter."""
        self.attempt = 0
        self.last_crash_reason = None


# ---------------------------------------------------------------------------
# Diagnostic helper used by `cyberos doctor`
# ---------------------------------------------------------------------------

def daemon_status(store: Path) -> dict[str, object]:
    """Returns a JSON-able summary of the daemon's recent state. Used by
    ``cyberos doctor`` to surface 'daemon hasn't beat in 2 hours' to the user.
    """
    probe = HeartbeatProbe(store=store)
    p = probe.path()
    if not p.exists():
        return {
            "state": "absent",
            "detail": f"no heartbeat file at {p}",
        }
    try:
        data = json.loads(p.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as e:
        return {"state": "malformed", "detail": str(e)}
    ts_ns = int(data.get("ts_ns", 0))
    age_secs = max(0.0, (time.time_ns() - ts_ns) / 1_000_000_000)
    return {
        "state": "healthy" if probe.is_alive() else "stale",
        "pid": data.get("pid"),
        "ts_iso": data.get("ts_iso"),
        "age_secs": age_secs,
    }
