"""Skill catalog — discovers what's available by walking ../skill/skills/.

Reads YAML frontmatter from every SKILL.md under the skill-root and returns
a list of `SkillEntry` records. The router consumes this list to score
candidate skills against a natural-language query.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import yaml


@dataclass
class SkillEntry:
    name: str
    description: str
    capabilities: list[str]
    region: str | None
    collection: str | None
    skill_dir: Path


def discover(skill_root: Path) -> list[SkillEntry]:
    """Walk skill_root for SKILL.md files; return parsed entries.

    Skips any SKILL.md without a frontmatter block, or with a malformed
    one. Frontmatter is YAML between ``---`` markers at the top of the file.
    """
    out: list[SkillEntry] = []
    if not skill_root.exists():
        return out
    for skill_md in sorted(skill_root.rglob("SKILL.md")):
        raw = skill_md.read_text(encoding="utf-8")
        if not raw.startswith("---\n"):
            continue
        end = raw.find("\n---\n", 4)
        if end < 0:
            continue
        try:
            fm = yaml.safe_load(raw[4:end])
        except yaml.YAMLError:
            continue
        if not isinstance(fm, dict):
            continue
        if "name" not in fm or "description" not in fm:
            continue
        caps_raw = fm.get("allowed-tools", "")
        if isinstance(caps_raw, str):
            caps = caps_raw.split() if caps_raw else []
        else:
            caps = list(caps_raw or [])
        metadata = fm.get("metadata", {}) or {}
        out.append(
            SkillEntry(
                name=str(fm["name"]),
                description=" ".join(str(fm["description"]).split()),
                capabilities=caps,
                region=metadata.get("region"),
                collection=metadata.get("collection"),
                skill_dir=skill_md.parent,
            )
        )
    return out
