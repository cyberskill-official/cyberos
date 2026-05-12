#!/usr/bin/env python3
"""
cyberos_fr_migrate.py — convert legacy `feature_request@1` (tasks inlined in
YAML frontmatter) to the new shape (Batch A, 2026-05-12) where tasks live as
body H2 sections with fenced `task-meta` blocks.

Usage:
  python3 runtime/tools/cyberos_fr_migrate.py <FR-file.md> [--in-place] [--check]

Modes:
  default       Write migrated artefact to stdout.
  --in-place    Overwrite the input file (backup at <file>.legacy.bak).
  --check       Exit 1 if file is legacy shape (needs migration), 0 if new.

Safety:
  - Idempotent: running on already-new shape is a no-op.
  - Preserves Source-attribution stripping per `feedback_fr_doc_style` memory.
"""
from __future__ import annotations
import argparse
import sys
import re
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from cyberos_fr_parser import split_frontmatter, parse_body_tasks


# Keys removed from frontmatter when moved into body / task-meta fences
_FRONTMATTER_DROP = {"tasks", "task_count"}
# Keys that move into a per-task `task-meta` fence
_TASK_META_KEYS = [
    "sizing", "dependencies", "parallelisable",
    "assignable_to", "agent_profile",
    "estimated_tokens", "estimated_hours",
    "status", "runbook_hint",
]
# Body sections that should be stripped (BRAIN-meta, per feedback_fr_doc_style)
_STRIP_SECTIONS = ("Source attribution", "Provenance")


def yaml_dump(d) -> str:
    """Minimal, stable YAML dumper — uses PyYAML if present."""
    import yaml
    return yaml.safe_dump(d, default_flow_style=False, sort_keys=False, allow_unicode=True).rstrip()


def render_task_section(task: dict) -> str:
    """Render one task as a body H2 section."""
    tid = task.get("id", "FR-???-T-??")
    title = task.get("title", "")
    description = (task.get("description") or "").strip()
    preconditions = task.get("preconditions") or []
    deliverables = task.get("deliverables") or []
    deps = task.get("dependencies") or []
    at = task.get("acceptance_test") or {}

    lines = [f"## {tid} — {title}", ""]
    if description:
        lines.append(description)
        lines.append("")

    if preconditions:
        lines.append("**Preconditions:**")
        lines.append("")
        for p in preconditions:
            lines.append(f"- {p}")
        lines.append("")
    else:
        lines.append("**Preconditions:**")
        lines.append("")
        lines.append("- none")
        lines.append("")

    if deliverables:
        lines.append("**Deliverables:**")
        lines.append("")
        for d in deliverables:
            lines.append(f"- {d}")
        lines.append("")

    if deps:
        lines.append("**Dependencies:**")
        lines.append("")
        for d in deps:
            lines.append(f"- {d}")
        lines.append("")

    lines.append("**Acceptance test:**")
    lines.append("")
    if at.get("shell"):
        lines.append("```shell")
        lines.append(at["shell"])
        lines.append("```")
    elif at.get("assertion"):
        lines.append("```assertion")
        lines.append(at["assertion"])
        lines.append("```")
    else:
        lines.append("```assertion")
        lines.append("TBD")
        lines.append("```")
    lines.append("")

    # task-meta fenced block
    meta = {}
    for k in _TASK_META_KEYS:
        if k in task:
            meta[k] = task[k]
    if meta:
        lines.append("```task-meta")
        lines.append(yaml_dump(meta))
        lines.append("```")
        lines.append("")

    return "\n".join(lines)


def strip_meta_sections(body: str) -> str:
    """Drop `## Source attribution` / `## Provenance` sections from FR body.

    Per `feedback_fr_doc_style` memory: FR is implementation spec, not meta-narration.
    """
    out_lines = []
    skipping = False
    for line in body.split("\n"):
        m = re.match(r'^## (.+?)\s*$', line)
        if m:
            section = m.group(1).strip()
            if section in _STRIP_SECTIONS:
                skipping = True
                continue
            skipping = False
        if not skipping:
            out_lines.append(line)
    # Drop trailing `## Tasks` reference section if present (we now have per-task H2s).
    # Matches `## Tasks` followed by ANY prose up to the next H2 heading.
    text = "\n".join(out_lines)
    text = re.sub(r'\n## Tasks\b[^\n]*\n(?:(?!\n## ).*\n)*',
                  "\n", text, flags=re.M)
    return text


def migrate(text: str) -> tuple[str, dict]:
    """Migrate one FR text. Returns (new_text, stats)."""
    fm, body = split_frontmatter(text)
    if not fm:
        return text, {"status": "no-frontmatter", "changed": False}

    body_tasks = parse_body_tasks(body)
    legacy_tasks = fm.get("tasks") or []
    if body_tasks and not legacy_tasks:
        # Already migrated
        return text, {"status": "already-new", "changed": False, "task_count": len(body_tasks)}

    tasks = legacy_tasks or body_tasks
    if not tasks:
        return text, {"status": "no-tasks", "changed": False}

    # Build new frontmatter
    new_fm = {k: v for k, v in fm.items() if k not in _FRONTMATTER_DROP}
    new_fm["task_index"] = [
        {"id": t.get("id"), "title": t.get("title", "")} for t in tasks
    ]
    # Drop provenance block too (it was the "Source attribution" prose driver)
    new_fm.pop("provenance", None)

    # Strip meta sections from body
    new_body = strip_meta_sections(body).rstrip() + "\n"

    # Append task H2 sections
    task_sections = "\n".join(render_task_section(t) for t in tasks)

    new_text = "---\n" + yaml_dump(new_fm) + "\n---\n\n" + new_body + "\n" + task_sections
    return new_text, {
        "status": "migrated",
        "changed": True,
        "task_count": len(tasks),
        "frontmatter_lines_before": len(text[:text.find("\n---\n", 4)].split("\n")) if "---" in text else 0,
    }


def main():
    p = argparse.ArgumentParser(description="Migrate legacy FR (frontmatter tasks) → new shape (body H2 tasks)")
    p.add_argument("file", help="FR markdown file")
    p.add_argument("--in-place", action="store_true", help="Overwrite input (creates .legacy.bak)")
    p.add_argument("--check", action="store_true", help="Exit 1 if migration needed, 0 if already new shape")
    args = p.parse_args()

    src = Path(args.file)
    if not src.exists():
        print(f"error: {src} not found", file=sys.stderr)
        return 2

    text = src.read_text(encoding="utf-8")
    new_text, stats = migrate(text)

    if args.check:
        if stats["status"] == "migrated":
            print(f"{src}: legacy shape, needs migration ({stats['task_count']} tasks)")
            return 1
        print(f"{src}: {stats['status']}")
        return 0

    if args.in_place:
        if stats["changed"]:
            bak = src.with_suffix(src.suffix + ".legacy.bak")
            bak.write_text(text, encoding="utf-8")
            src.write_text(new_text, encoding="utf-8")
            print(f"migrated {src} → new shape ({stats['task_count']} tasks). backup: {bak.name}")
        else:
            print(f"{src}: {stats['status']} (no changes)")
        return 0

    # default: stdout
    sys.stdout.write(new_text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
