"""Make test and scoring files read-only at the OS level (anti-reward-hacking).

ImpossibleBench (arXiv:2510.20270) found that read-only test files are the
pragmatic "middle ground": they kill the dominant cheat — test modification,
>79% of Claude's cheating strategies — without the capability hit of fully
hiding tests. METR's reward-hacking study reinforces that the model must never
see or write the scoring function.

This module provides the *OS speed bump* (chmod away write bits, then verify a
non-privileged write actually fails) and emits the matching ``deny_write_globs``
so the PreToolUse hook blocks edits too. Defense in depth:

  layer 1  chmod read-only            (this module — verifiable, same-uid speed bump)
  layer 2  PreToolUse deny_write_globs (policy — blocks Edit/Write/Bash mutations)
  layer 3  read-only bind mount / different uid  (sandbox/ — the strong guarantee)

Same-process chmod is not a security boundary against a determined same-uid
process (it can chmod back); layer 3 in ``sandbox/`` is. But layers 1+2 stop the
*observed* failure mode — a model lazily editing a failing test — cold.
"""

from __future__ import annotations

import fnmatch
import json
import os
import stat
from dataclasses import dataclass, field
from pathlib import Path

__all__ = [
    "DEFAULT_TEST_GLOBS",
    "DEFAULT_SCORING_GLOBS",
    "LockReport",
    "find_targets",
    "make_readonly",
    "make_writable",
    "is_readonly",
    "verify_unwritable",
    "lock_tests",
    "write_policy_augmentation",
]

DEFAULT_TEST_GLOBS: tuple[str, ...] = ("test_*.py", "*_test.py", "tests/**", "conftest.py")
DEFAULT_SCORING_GLOBS: tuple[str, ...] = ("score*.py", "grade*.py", "scoring/**", "scorer*.py")

_WRITE_BITS = stat.S_IWUSR | stat.S_IWGRP | stat.S_IWOTH


@dataclass
class LockReport:
    """Outcome of a lock_tests run."""

    locked: list[str] = field(default_factory=list)
    failed: list[str] = field(default_factory=list)
    recommended_write_globs: list[str] = field(default_factory=list)
    hidden: list[str] = field(default_factory=list)

    def summary(self) -> str:
        lines = [f"locked {len(self.locked)} file(s) read-only"]
        if self.hidden:
            lines.append(f"sealed {len(self.hidden)} scoring file(s) unreadable")
        if self.failed:
            lines.append(f"FAILED to verify {len(self.failed)}: {self.failed}")
        lines.append(f"recommended deny_write_globs: {self.recommended_write_globs}")
        return "; ".join(lines)


def _matches_any(rel: str, base: str, globs) -> bool:
    for g in globs:
        # support a simple "dir/**" form meaning "anything under dir/"
        if g.endswith("/**"):
            prefix = g[:-3].rstrip("/")
            if rel == prefix or rel.startswith(prefix + "/"):
                return True
        if fnmatch.fnmatch(rel, g) or fnmatch.fnmatch(base, g):
            return True
    return False


def find_targets(root: str | os.PathLike[str], globs=DEFAULT_TEST_GLOBS) -> list[Path]:
    """Return files under ``root`` matching any glob (relative path or basename)."""
    root_path = Path(root).resolve()
    out: list[Path] = []
    for dirpath, _dirnames, filenames in os.walk(root_path):
        for fn in filenames:
            full = Path(dirpath) / fn
            rel = str(full.relative_to(root_path))
            if _matches_any(rel, fn, globs):
                out.append(full)
    return sorted(out)


def make_readonly(path: str | os.PathLike[str]) -> int:
    """Clear all write bits on ``path``; return the new mode."""
    p = Path(path)
    mode = p.stat().st_mode
    new = mode & ~_WRITE_BITS
    os.chmod(p, new)
    return stat.S_IMODE(new)


def make_writable(path: str | os.PathLike[str]) -> int:
    """Restore the owner write bit on ``path`` (for the human operator); return new mode."""
    p = Path(path)
    new = p.stat().st_mode | stat.S_IWUSR
    os.chmod(p, new)
    return stat.S_IMODE(new)


def is_readonly(path: str | os.PathLike[str]) -> bool:
    return (Path(path).stat().st_mode & _WRITE_BITS) == 0


def verify_unwritable(path: str | os.PathLike[str]) -> bool:
    """Actually attempt to open ``path`` for writing; return True iff it is refused.

    This is *evidence*, not belief: we try ``os.open(..., O_WRONLY)`` and confirm a
    PermissionError. (Runs as a non-privileged uid — the only context an agent
    should ever run in; root would bypass file perms, which is what layer 3 prevents.)
    """
    try:
        fd = os.open(path, os.O_WRONLY)
    except PermissionError:
        return True
    except OSError:
        return False
    os.close(fd)
    return False


def _seal_unreadable(path: str | os.PathLike[str]) -> int:
    """Hidden mode: remove read+write so the agent cannot see the scoring logic."""
    os.chmod(path, 0o000)
    return 0o000


def lock_tests(
    root: str | os.PathLike[str],
    test_globs=DEFAULT_TEST_GLOBS,
    scoring_globs: tuple[str, ...] | None = None,
    hidden: bool = False,
) -> LockReport:
    """Lock test files read-only and (optionally) seal scoring files unreadable.

    Args:
        root: directory to scan.
        test_globs: globs for test files -> made read-only (still readable/runnable).
        scoring_globs: globs for scoring/grader files. If ``hidden`` is True these are
            chmod 0o000 (unreadable); otherwise they are made read-only like tests.
        hidden: escalate scoring files to fully unreadable. Escalation trigger per the
            roadmap: do this once you observe any test-file edit or operator-overload
            in trajectories.

    Returns:
        LockReport with locked/failed/hidden lists and recommended deny_write_globs.
    """
    root_path = Path(root).resolve()
    report = LockReport()

    for target in find_targets(root_path, test_globs):
        make_readonly(target)
        rel = str(target.relative_to(root_path))
        if verify_unwritable(target):
            report.locked.append(rel)
        else:
            report.failed.append(rel)
        report.recommended_write_globs.append(rel)

    if scoring_globs:
        for target in find_targets(root_path, scoring_globs):
            rel = str(target.relative_to(root_path))
            if hidden:
                _seal_unreadable(target)
                report.hidden.append(rel)
            else:
                make_readonly(target)
                report.locked.append(rel)
        # Deny the whole scoring/held-out PATTERN in policy — not just files that
        # exist now — so the agent cannot add, revive, or swap files in this area.
        # The held-out suite (SpecBench) must stay sealed even as the repo grows;
        # visible test files (test_globs) stay per-file so the agent can still add
        # its own unit tests.
        for g in scoring_globs:
            if g not in report.recommended_write_globs:
                report.recommended_write_globs.append(g)

    return report


def write_policy_augmentation(root: str | os.PathLike[str], write_globs: list[str]) -> Path:
    """Merge ``deny_write_globs`` into ``<root>/.awh/policy.json`` so the hook also
    refuses edits to the locked files. Returns the policy path."""
    root_path = Path(root).resolve()
    awh_dir = root_path / ".awh"
    awh_dir.mkdir(exist_ok=True)
    policy_path = awh_dir / "policy.json"
    data: dict = {}
    if policy_path.is_file():
        try:
            data = json.loads(policy_path.read_text())
        except json.JSONDecodeError:
            data = {}
    existing = set(data.get("deny_write_globs", []))
    existing.update(write_globs)
    data["deny_write_globs"] = sorted(existing)
    policy_path.write_text(json.dumps(data, indent=2) + "\n")
    return policy_path
