"""version_bump — rewrite metadata.version in a SKILL.md or workflow YAML.

TASK-CUO-202 §1 #2.

Semver bumps: patch | minor | major. Accepts a current version string like
"1.0.0" or "2.3.4-rc1" and returns the bumped form. The SKILL.md / workflow
file's frontmatter `metadata.version:` line is updated in place.
"""

from __future__ import annotations

import re
from pathlib import Path
from typing import Literal


BumpLevel = Literal["patch", "minor", "major"]

# Match `version: X.Y.Z` (and trailing -prerelease suffix) inside YAML frontmatter.
_VERSION_LINE_RE = re.compile(
    r"^(\s*version:\s*)(\d+)\.(\d+)\.(\d+)([\w.-]*)\s*$",
    re.MULTILINE,
)


class VersionBumpError(ValueError):
    """Raised when the file doesn't contain a parseable version line."""


def bump_version(current: str, level: BumpLevel) -> str:
    """Compute the bumped semver string. Doesn't touch any file."""
    m = re.match(r"^(\d+)\.(\d+)\.(\d+)([\w.-]*)$", current.strip())
    if not m:
        raise VersionBumpError(f"unparseable version {current!r}")
    major, minor, patch, suffix = m.groups()
    if level == "patch":
        return f"{major}.{minor}.{int(patch) + 1}"
    if level == "minor":
        return f"{major}.{int(minor) + 1}.0"
    if level == "major":
        return f"{int(major) + 1}.0.0"
    raise VersionBumpError(f"unknown bump level {level!r}")


def bump_file(path: Path, level: BumpLevel) -> str:
    """Read `path`, bump its `version:` line in-place, return the new version.

    Atomic: write-to-temp then rename.
    """
    text = path.read_text(encoding="utf-8")
    m = _VERSION_LINE_RE.search(text)
    if not m:
        raise VersionBumpError(f"no `version:` line found in {path}")
    current = f"{m.group(2)}.{m.group(3)}.{m.group(4)}{m.group(5)}"
    new = bump_version(current, level)
    new_line = f"{m.group(1)}{new}"
    new_text = text[:m.start()] + new_line + text[m.end():]
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_text(new_text, encoding="utf-8")
    import os
    os.replace(tmp, path)
    return new


def read_version(path: Path) -> str:
    """Return the current `version:` value from a SKILL.md / workflow file."""
    text = path.read_text(encoding="utf-8")
    m = _VERSION_LINE_RE.search(text)
    if not m:
        raise VersionBumpError(f"no `version:` line found in {path}")
    return f"{m.group(2)}.{m.group(3)}.{m.group(4)}{m.group(5)}"
