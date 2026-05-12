#!/usr/bin/env python3
"""
cyberos_proj.py — sync tasks from an FR into a project tracker (S5.4).

Subcommands:

    cyberos proj sync FR-NNN [--backend linear|jira|github] [--dry-run]
        Read FR-NNN's embedded tasks; for each task, create / update a
        ticket in the target project tracker.

    cyberos proj backends
        List supported backends + whether each is configured

    cyberos proj pull FR-NNN
        Pull ticket status back from the tracker; update task status in
        the FR markdown.

For now this is a SCAFFOLD: ticket-creation outputs a deterministic
JSON envelope per task. Operator pipes the envelope to the real backend
via `linear-cli`, `jira-cli`, or `gh issue create`. The cyberos side
records the mapping in a `<FR>.proj-sync.json` sidecar.

Reason for scaffold-first: backend credentials live elsewhere; we never
want to surprise operators by creating real tickets unexpectedly.
"""
from __future__ import annotations
import argparse
import json
import re
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

ICT = timezone(timedelta(hours=7))


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def parse_frontmatter(text: str) -> tuple[dict, str]:
    if not text.startswith("---\n"):
        return {}, text
    end = text.find("\n---\n", 4)
    if end < 0:
        return {}, text
    try:
        import yaml
        return yaml.safe_load(text[4:end]) or {}, text[end + 5:]
    except Exception:
        return {}, text[end + 5:]


def resolve_fr(brain_root: Path, fr_id: str) -> Path:
    """Find FR-NNN-*.md under planning/ or memories/projects/."""
    for d in (brain_root / "planning", brain_root / ".cyberos-memory" / "memories" / "projects"):
        if not d.exists():
            continue
        for md in d.rglob(f"{fr_id}-*.md"):
            return md
        for md in d.rglob(f"FR-{fr_id}*.md"):
            return md
    raise SystemExit(f"no FR matching {fr_id!r}")


def task_to_envelope(task: dict, fr_id: str, backend: str) -> dict:
    """Render a task into a backend-specific ticket payload."""
    body_lines = [
        task.get("description", ""),
        "",
        "## Preconditions",
        "\n".join(f"- {x}" for x in (task.get("preconditions") or [])) or "_(none)_",
        "",
        "## Deliverables",
        "\n".join(f"- {x}" for x in (task.get("deliverables") or [])),
        "",
        "## Acceptance test",
        "```",
        (task.get("acceptance_test", {}).get("shell")
         or task.get("acceptance_test", {}).get("assertion")
         or "TBD"),
        "```",
    ]
    body = "\n".join(body_lines)

    tid = task.get("id", "T??")
    title = f"[{tid}] {task.get('title', '?')}"
    labels = [f"sizing:{task.get('sizing','?')}",
              f"assignable:{','.join(task.get('assignable_to') or [])}",
              fr_id, "cyberos-chain"]

    if backend == "linear":
        return {
            "backend": "linear",
            "command": f"linear-cli issue create --title \"{title}\" --priority 3 --labels \"{','.join(labels)}\"",
            "title": title,
            "body": body,
            "labels": labels,
        }
    elif backend == "jira":
        return {
            "backend": "jira",
            "command": f"jira-cli issue create --summary \"{title}\" --labels \"{','.join(labels)}\"",
            "title": title,
            "body": body,
            "labels": labels,
        }
    elif backend == "github":
        return {
            "backend": "github",
            "command": f"gh issue create --title \"{title}\" --label \"{','.join(labels)}\" --body-file -",
            "title": title,
            "body": body,
            "labels": labels,
        }
    else:
        return {"backend": "unknown", "title": title, "body": body, "labels": labels}


def cmd_backends(_args):
    print("\n  Backend support:")
    for b in ("linear", "jira", "github"):
        # Detect CLI presence
        import shutil
        clis = {"linear": ["linear", "linear-cli"], "jira": ["jira", "jira-cli"], "github": ["gh"]}
        present = next((c for c in clis[b] if shutil.which(c)), None)
        flag = f"✓ ({present})" if present else "✗ no CLI on PATH"
        print(f"    {b:10s}  {flag}")
    print()
    print("  This tool produces backend-specific envelopes; operator pipes to the CLI.")
    return 0


def cmd_sync(args):
    brain_root = find_brain()
    fr_path = resolve_fr(brain_root, args.fr_id)
    text = fr_path.read_text(encoding="utf-8")
    fm, body = parse_frontmatter(text)
    tasks = fm.get("tasks") or []
    if not tasks:
        print(f"  {args.fr_id} has no embedded tasks")
        return 0

    envelopes = [task_to_envelope(t, args.fr_id, args.backend) for t in tasks]
    out_path = fr_path.with_suffix(".proj-sync.json")
    out_path.write_text(json.dumps({
        "fr_id": args.fr_id,
        "fr_path": str(fr_path.relative_to(brain_root)),
        "backend": args.backend,
        "generated_at": datetime.now(ICT).isoformat(timespec="seconds"),
        "task_count": len(envelopes),
        "envelopes": envelopes,
        "applied": False,
    }, indent=2))

    print(f"\n  {args.backend} sync envelopes for {args.fr_id}:")
    print(f"  Generated {len(envelopes)} envelope(s) → {out_path.relative_to(brain_root)}")
    if args.dry_run:
        print(f"  (dry-run; review the envelopes before piping to the backend CLI)")
        return 0

    print(f"\n  Run these commands to create tickets (review first):\n")
    for env in envelopes:
        print(f"  # {env['title']}")
        print(f"  echo {json.dumps(env['body'])[:80]}… | {env['command']}")
        print()
    return 0


def cmd_pull(args):
    brain_root = find_brain()
    fr_path = resolve_fr(brain_root, args.fr_id)
    sync_path = fr_path.with_suffix(".proj-sync.json")
    if not sync_path.exists():
        print(f"  no sync record at {sync_path}", file=sys.stderr); return 2
    sync = json.loads(sync_path.read_text())
    print(f"\n  Backend: {sync.get('backend')}")
    print(f"  Task envelopes: {sync.get('task_count')}")
    print(f"  Applied: {sync.get('applied')}")
    print(f"\n  Pull is a scaffold — implement by adding linked_pr / status from the tracker into the FR's `tasks[]` list.")
    return 0


def main():
    p = argparse.ArgumentParser(description="proj-tracker sync (S5.4)")
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("backends").set_defaults(func=cmd_backends)
    ps = sub.add_parser("sync")
    ps.add_argument("fr_id")
    ps.add_argument("--backend", choices=["linear", "jira", "github"], default="github")
    ps.add_argument("--dry-run", action="store_true")
    ps.set_defaults(func=cmd_sync)
    pp = sub.add_parser("pull"); pp.add_argument("fr_id"); pp.set_defaults(func=cmd_pull)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
