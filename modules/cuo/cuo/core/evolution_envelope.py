"""evolution_envelope - FR-CUO-204 safety boundary for the idle-time dream loop.

The dream loop (`dream_loop.py`) may AUTOMATICALLY apply a self-improvement only when the change clears
three independent gates: it is gate-green (the AWH test gate), it is classified low-risk (FR-CUO-202
`proposal_applier`), and its TARGET sits inside this envelope. This module is that third gate - a
path-based allow/deny boundary that is orthogonal to the content-based risk classifier, so a change that
looks benign in its body but touches a security-critical file still halts for a human.

Two design rules make the envelope safe by construction:

1. Default-deny. A target is allowed only if it matches an allowlist pattern AND matches no denylist
   pattern. Anything unrecognised is denied. The allowlist is the small set of human-readable,
   test-covered, reversible artifacts the loop may touch (skill prompt bodies, workflow step ordering,
   thresholds). The denylist names the invariants the loop must NEVER auto-modify: auth and RBAC, the
   audit chain, cross-tenant isolation, PII redaction, and cost-ledger math. Denylist beats allowlist.

2. Disabled by default. `enabled` is false in the shipped config, and a kill-switch environment variable
   overrides it to off regardless. The loop applies nothing until an operator both sets `enabled: true`
   and clears the kill switch, after reviewing this envelope.

Patterns are matched with `fnmatch`, whose `*` spans path separators, so `services/auth/*` matches
`services/auth/src/rbac.rs` and `*audit*` matches any path containing "audit".
"""

from __future__ import annotations

import os
from dataclasses import dataclass
from fnmatch import fnmatch
from pathlib import Path
from typing import Optional

import yaml

# The environment variable that force-disables the loop no matter what the config says.
KILL_SWITCH_ENV = "CYBEROS_DREAM_KILL"
_TRUE_VALUES = {"1", "true", "yes", "on"}

# The enablement ladder (FR-CUO-204). `off` never runs; `propose` runs + records but applies nothing;
# `auto` may auto-apply within the gates (and only with the runner's explicit opt-in).
VALID_MODES = ("off", "propose", "auto")


@dataclass(frozen=True)
class EnvelopeVerdict:
    """The envelope's decision for one target path."""

    decision: str  # "allow" | "deny_halt"
    reason: str
    matched: Optional[str] = None  # the pattern that decided it, when applicable

    @property
    def allowed(self) -> bool:
        return self.decision == "allow"


@dataclass
class EvolutionEnvelope:
    """The allow/deny boundary plus the loop's bounding knobs. Loaded from `config/dream.yaml`."""

    allowlist: list[str]
    denylist: list[str]
    enabled: bool = False
    mode: str = "off"
    idle_window_minutes: int = 30
    max_changes_per_window: int = 5
    max_wall_clock_seconds: int = 600

    @classmethod
    def from_dict(cls, data: dict) -> "EvolutionEnvelope":
        return cls(
            allowlist=list(data.get("allowlist", []) or []),
            denylist=list(data.get("denylist", []) or []),
            enabled=bool(data.get("enabled", False)),
            mode=str(data.get("mode", "off")).strip().lower(),
            idle_window_minutes=int(data.get("idle_window_minutes", 30)),
            max_changes_per_window=int(data.get("max_changes_per_window", 5)),
            max_wall_clock_seconds=int(data.get("max_wall_clock_seconds", 600)),
        )

    @classmethod
    def load(cls, path: Path) -> "EvolutionEnvelope":
        """Load from a YAML config. A missing file yields a safe, fully-disabled, deny-everything envelope."""
        p = Path(path)
        if not p.is_file():
            return cls(allowlist=[], denylist=[], enabled=False)
        data = yaml.safe_load(p.read_text(encoding="utf-8")) or {}
        return cls.from_dict(data)

    def is_enabled(self, env: Optional[dict] = None) -> bool:
        """The loop is enabled only if the config flag is true AND the kill switch is not set."""
        env = os.environ if env is None else env
        if str(env.get(KILL_SWITCH_ENV, "")).strip().lower() in _TRUE_VALUES:
            return False
        return self.enabled

    def effective_mode(self, env: Optional[dict] = None) -> str:
        """Resolve the operative mode after the kill switch and the master `enabled` flag.

        Returns `off` (the loop must not run) when the kill switch is set, when `enabled` is false, or
        when the configured mode is not a recognised running mode. Otherwise returns `propose` or `auto`.
        The runner reads this to decide whether to run, and whether auto-apply is even a possibility; the
        dream loop itself still gates on `is_enabled`, so a stale caller can never accidentally auto-apply.
        """
        if not self.is_enabled(env):
            return "off"
        mode = self.mode if self.mode in VALID_MODES else "off"
        return mode

    def classify_target(self, target: str) -> EnvelopeVerdict:
        """Decide whether the loop may auto-modify `target`. Default-deny; denylist beats allowlist."""
        norm = _normalize(target)

        for pat in self.denylist:
            if _match(norm, pat):
                return EnvelopeVerdict(
                    "deny_halt",
                    f"target matches denylist invariant '{pat}' - holds for human review",
                    pat,
                )

        for pat in self.allowlist:
            if _match(norm, pat):
                return EnvelopeVerdict("allow", f"target matches allowlist entry '{pat}'", pat)

        return EnvelopeVerdict(
            "deny_halt",
            "target matches no allowlist entry (default-deny) - holds for human review",
            None,
        )


def _normalize(target: str) -> str:
    """Normalise to forward-slash, no leading './', for stable matching."""
    s = str(target).replace("\\", "/").strip()
    while s.startswith("./"):
        s = s[2:]
    return s.lstrip("/")


def _match(target: str, pattern: str) -> bool:
    """fnmatch where `*` spans separators; also treat a bare token as a substring (`*token*`)."""
    pat = pattern.replace("\\", "/")
    if any(ch in pat for ch in "*?[]"):
        return fnmatch(target, pat)
    # A plain string is treated as a path-substring match for convenience.
    return pat in target
