#!/usr/bin/env python3
"""
cyberos_fr_parser.py — shared parser for `feature_request@1` artefacts.

Handles BOTH the legacy shape (tasks inlined in frontmatter as a YAML list)
and the new shape (Batch A, 2026-05-12) where each task is a body H2 section
with a fenced ``task-meta`` block for structured fields.

Consumers: cyberos_fr.py (browse / show / graph), runtime/skill_runners/fr_with_tasks.py
(validation), cyberos_fr_migrate.py (legacy→new conversion).

Public surface
--------------
parse_fr(path: Path) -> FR
  Returns a dict with: fr_id, title, frontmatter, body_prose, tasks (list of dicts).
  Each task dict follows task@1: id, title, description, preconditions, deliverables,
  acceptance_test (dict with shell or assertion), sizing, dependencies,
  parallelisable, assignable_to, agent_profile?, estimated_tokens?, estimated_hours?,
  status, runbook_hint?, subtasks? (Batch B).

split_frontmatter(text: str) -> (dict, str)
  YAML frontmatter + remaining body. Empty dict if no frontmatter.

parse_body_tasks(body: str) -> list[dict]
  Extract per-task H2 sections from FR body. Returns [] if no body tasks found
  (caller should fall back to frontmatter tasks).
"""
from __future__ import annotations
import re
from pathlib import Path
from typing import Tuple


def split_frontmatter(text: str) -> Tuple[dict, str]:
    if not text.startswith("---\n"):
        return {}, text
    end = text.find("\n---\n", 4)
    if end < 0:
        return {}, text
    try:
        import yaml
        fm = yaml.safe_load(text[4:end]) or {}
    except Exception:
        fm = {}
    return fm, text[end + 5:]


# ──────────────────────────────────────────────────────────────────────
# Body-task parser (new shape)
# ──────────────────────────────────────────────────────────────────────

# Matches:  ## FR-001-T-01 — Title text
#   OR:     ## FR-001-T-01 - Title text
#   OR:     ## FR-001-T-01: Title text
_TASK_HEADING = re.compile(r'^## (FR-\d+-T-\d+)\s*[—\-:]\s*(.+?)\s*$', re.M)

# Batch B — subtask headings inside a task section.
# Matches:  ### FR-001-T-01-ST-01 — Subtask title
_SUBTASK_HEADING = re.compile(r'^### (FR-\d+-T-\d+-ST-\d+)\s*[—\-:]\s*(.+?)\s*$', re.M)

# Matches  ```task-meta  or  ```yaml task-meta
_TASK_META_FENCE = re.compile(
    r'```(?:yaml\s+)?task-meta\s*\n(.*?)\n```',
    re.S
)

# Matches `**Label:**`  (e.g. **Preconditions:**)
def _section_after_label(section: str, label: str) -> str:
    """Return text after `**{label}:**` up to the next `**X:**` or end."""
    m = re.search(rf'\*\*{re.escape(label)}:\*\*\s*\n(.*?)(?=\n\*\*[A-Z][^*]*:\*\*|\n```task-meta|\Z)',
                  section, re.S)
    return m.group(1).strip() if m else ""


def _parse_bullets(text: str) -> list[str]:
    """Pull `- item` bullets from text. 'none' yields [].."""
    if not text or text.strip().lower() in ("none", "n/a", "-"):
        return []
    out = []
    for line in text.split("\n"):
        s = line.strip()
        if s.startswith(("- ", "* ", "+ ")):
            out.append(s[2:].strip().strip('"').strip("'"))
    return out


def _parse_acceptance_test(text: str) -> dict:
    """Extract first code fence after **Acceptance test:** label.

    Code fence with `shell` info-string → {"shell": "..."}.
    Code fence with `assertion` info-string OR no info → {"assertion": "..."}.
    Plain text (no fence) → {"assertion": "<text>"}.
    """
    if not text.strip():
        return {}
    m = re.search(r'```(\w*)\s*\n(.*?)\n```', text, re.S)
    if m:
        info, body = m.group(1).strip(), m.group(2).strip()
        if info == "shell":
            return {"shell": body}
        if info == "assertion":
            return {"assertion": body}
        return {"assertion": body or text.strip()}
    return {"assertion": text.strip()}


def parse_body_tasks(body: str) -> list[dict]:
    """Return per-task dicts extracted from H2 sections in FR body.

    Returns [] when no `## FR-NNN-T-MM —` headings are found (caller falls back
    to legacy frontmatter `tasks:` list).
    """
    matches = list(_TASK_HEADING.finditer(body))
    if not matches:
        return []
    out = []
    for i, m in enumerate(matches):
        tid = m.group(1)
        title = m.group(2).strip()
        start = m.end()
        end = matches[i + 1].start() if i + 1 < len(matches) else len(body)
        section = body[start:end].strip()

        # Structured metadata from task-meta fence
        meta = {}
        meta_match = _TASK_META_FENCE.search(section)
        if meta_match:
            try:
                import yaml
                meta = yaml.safe_load(meta_match.group(1)) or {}
            except Exception:
                meta = {}
            # Strip the fence from section for description parsing
            section_no_meta = section[:meta_match.start()] + section[meta_match.end():]
        else:
            section_no_meta = section

        # Pull labelled subsections from prose
        preconditions = _parse_bullets(_section_after_label(section_no_meta, "Preconditions"))
        deliverables = _parse_bullets(_section_after_label(section_no_meta, "Deliverables"))
        acceptance = _parse_acceptance_test(_section_after_label(section_no_meta, "Acceptance test"))
        dependencies = _parse_bullets(_section_after_label(section_no_meta, "Dependencies"))

        # Description = prose before first **X:** label or end
        first_label = re.search(r'\*\*[A-Z][^*]*:\*\*', section_no_meta)
        if first_label:
            description = section_no_meta[:first_label.start()].strip()
        else:
            description = section_no_meta.strip()

        # Build task dict — start from task-meta, overlay parsed fields
        task = dict(meta) if isinstance(meta, dict) else {}
        task["id"] = tid
        task.setdefault("title", title)
        if description and not task.get("description"):
            task["description"] = description
        if preconditions and not task.get("preconditions"):
            task["preconditions"] = preconditions
        if deliverables and not task.get("deliverables"):
            task["deliverables"] = deliverables
        if acceptance and not task.get("acceptance_test"):
            task["acceptance_test"] = acceptance
        if dependencies and not task.get("dependencies"):
            # Comma-separated string IDs from prose dependency bullets
            task["dependencies"] = [d.strip() for d in dependencies if d.strip().startswith("FR-")]
        # Defaults
        task.setdefault("preconditions", [])
        task.setdefault("deliverables", [])
        task.setdefault("dependencies", [])
        task.setdefault("acceptance_test", {})
        task.setdefault("status", "draft")

        # Batch B — extract subtasks from H3 headings inside the section
        subtasks = _parse_subtasks(section, tid)
        if subtasks:
            task["subtasks"] = subtasks
        out.append(task)
    return out


def _parse_subtasks(section: str, parent_tid: str) -> list[dict]:
    """Extract `### FR-NNN-T-MM-ST-XX —` subtask sections inside a task section.

    Subtask has minimal shape: id, title, prose description, optional fenced
    `subtask-meta` block with sizing + estimated_hours/tokens + status.
    Returns [] when none found.
    """
    matches = list(_SUBTASK_HEADING.finditer(section))
    if not matches:
        return []
    out = []
    for i, m in enumerate(matches):
        stid = m.group(1)
        title = m.group(2).strip()
        # Validate prefix matches parent
        if not stid.startswith(parent_tid + "-ST-"):
            continue
        start = m.end()
        end = matches[i + 1].start() if i + 1 < len(matches) else len(section)
        sub_section = section[start:end].strip()

        # Optional subtask-meta fence
        meta_match = re.search(r'```(?:yaml\s+)?subtask-meta\s*\n(.*?)\n```', sub_section, re.S)
        meta = {}
        if meta_match:
            try:
                import yaml
                meta = yaml.safe_load(meta_match.group(1)) or {}
            except Exception:
                meta = {}
            sub_no_meta = sub_section[:meta_match.start()] + sub_section[meta_match.end():]
        else:
            sub_no_meta = sub_section
        description = sub_no_meta.strip()

        subtask = dict(meta) if isinstance(meta, dict) else {}
        subtask["id"] = stid
        subtask.setdefault("title", title)
        if description and not subtask.get("description"):
            subtask["description"] = description
        subtask.setdefault("status", "draft")
        out.append(subtask)
    return out


# ──────────────────────────────────────────────────────────────────────
# Top-level FR parser
# ──────────────────────────────────────────────────────────────────────

def parse_fr(path: Path) -> dict:
    """Parse an FR file, preferring body-task shape over legacy frontmatter tasks."""
    text = path.read_text(encoding="utf-8")
    fm, body = split_frontmatter(text)
    body_tasks = parse_body_tasks(body)
    if body_tasks:
        tasks = body_tasks
        shape = "body-h2"
    else:
        tasks = fm.get("tasks") or []
        shape = "legacy-frontmatter"
    return {
        "path": str(path),
        "fr_id": fm.get("fr_id") or _guess_fr_id_from_filename(path),
        "title": fm.get("title", ""),
        "profile": fm.get("profile", "?"),
        "status": fm.get("status", "draft"),
        "shape": shape,
        "frontmatter": fm,
        "body": body,
        "tasks": tasks,
    }


def _guess_fr_id_from_filename(path: Path) -> str:
    m = re.match(r'^(FR-\d+)', path.stem)
    return m.group(1) if m else path.stem


if __name__ == "__main__":
    # Quick test
    import sys
    if len(sys.argv) > 1:
        fr = parse_fr(Path(sys.argv[1]))
        print(f"fr_id: {fr['fr_id']}")
        print(f"title: {fr['title']}")
        print(f"shape: {fr['shape']}")
        print(f"task count: {len(fr['tasks'])}")
        for t in fr["tasks"]:
            print(f"  {t.get('id'):14s}  [{t.get('sizing','?')}]  {t.get('title','')[:50]}")
            print(f"    desc={len(t.get('description') or ''):>4}c  precs={len(t.get('preconditions') or [])}  "
                  f"delivs={len(t.get('deliverables') or [])}  at={list((t.get('acceptance_test') or {}).keys())}")
