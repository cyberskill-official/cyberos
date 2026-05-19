"""
cyberos.core.decay — pluggable recency-decay profiles (FR-MEMORY-113 §1 #4).

A *decay profile* is a stateless callable that maps `hours_old` (delta
between "now" and a memory's `last_seen_at`) to a recency value in
``[0.0, 1.0]``. The protocol ships two built-in profiles:

1. **Exponential** (Park et al. 2023 "Generative Agents", validated on
   their benchmark simulation): ``recency = decay_factor ** hours_old``.
   Default ``decay_factor = 0.995`` ⇒ half-life ≈ 138 hours ≈ 5.76 days.
2. **Ebbinghaus** (MARS framework alignment, classic spaced-repetition
   curve): ``recency = exp(-hours_old / strength)``. Default
   ``strength = 240.0`` ⇒ characteristic time ~10 days.

Both are bounded in ``[0.0, 1.0]`` for non-negative ``hours_old``; negative
``hours_old`` (clock skew / future-dated `last_seen_at`) clamps to 1.0.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
from math import exp, log
from typing import Optional, Protocol, runtime_checkable


@runtime_checkable
class DecayProfile(Protocol):
    """Stateless callable ``recency(hours_old) -> float in [0.0, 1.0]``.

    MUST be monotonically non-increasing on non-negative inputs.
    """

    def __call__(self, hours_old: float) -> float: ...


@dataclass(frozen=True)
class Exponential:
    """Park-et-al exponential decay.

    ``recency = decay_factor ** hours_old``.

    Default ``decay_factor = 0.995`` ⇒ half-life ≈ 138.3 hours
    (≈ 5.76 days).
    """

    decay_factor: float = 0.995

    def __post_init__(self) -> None:
        if not isinstance(self.decay_factor, (int, float)):
            raise ValueError(
                f"decay_factor must be a number; got {type(self.decay_factor).__name__}"
            )
        if not (0.0 < float(self.decay_factor) < 1.0):
            raise ValueError(
                f"decay_factor must be in (0.0, 1.0); got {self.decay_factor}"
            )

    def __call__(self, hours_old: float) -> float:
        if hours_old < 0:
            return 1.0
        return float(self.decay_factor) ** float(hours_old)

    @property
    def half_life_hours(self) -> float:
        """``log(0.5) / log(decay_factor)`` — operator-readable lifetime."""
        return log(0.5) / log(self.decay_factor)


@dataclass(frozen=True)
class Ebbinghaus:
    """Classic forgetting curve (MARS framework alignment).

    ``recency = exp(-hours_old / strength)``.

    Default ``strength = 240.0`` ⇒ characteristic time = 10 days
    (recency drops to ``1/e ≈ 0.368`` at 10 days).
    """

    strength: float = 240.0

    def __post_init__(self) -> None:
        if not isinstance(self.strength, (int, float)):
            raise ValueError(
                f"strength must be a number; got {type(self.strength).__name__}"
            )
        if float(self.strength) <= 0:
            raise ValueError(f"strength must be > 0; got {self.strength}")

    def __call__(self, hours_old: float) -> float:
        if hours_old < 0:
            return 1.0
        return exp(-float(hours_old) / float(self.strength))


def build_profile(name: str, params: Optional[dict] = None) -> DecayProfile:
    """Construct a profile by name.

    Slice-4 will replace this with an ``entry_points``-based plug-in
    mechanism (see FR-MEMORY-113 §1 #13). Slice-3 ships the two built-ins.
    """
    params = params or {}
    if name == "exponential":
        return Exponential(decay_factor=params.get("decay_factor", 0.995))
    if name == "ebbinghaus":
        return Ebbinghaus(strength=params.get("strength", 240.0))
    raise ValueError(
        f"unknown decay profile {name!r}; expected one of: exponential, ebbinghaus"
    )


def hours_between(
    now: datetime, last_seen: Optional[datetime]
) -> Optional[float]:
    """Return the delta in hours, or None when ``last_seen`` is None.

    Both datetimes are coerced to UTC if naïve. Negative deltas (future-
    dated `last_seen_at` from clock skew) are returned as-is; profile
    implementations clamp negatives to recency=1.0.
    """
    if last_seen is None:
        return None
    if last_seen.tzinfo is None:
        last_seen = last_seen.replace(tzinfo=timezone.utc)
    if now.tzinfo is None:
        now = now.replace(tzinfo=timezone.utc)
    return (now - last_seen).total_seconds() / 3600.0
