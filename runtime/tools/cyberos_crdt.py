#!/usr/bin/env python3
"""
cyberos_crdt.py — CRDT-style merge for sync conflicts.

Batch 13 (Tier C) of post-catalog improvements.

Today `cyberos sync conflicts --resolve` picks a winner per memory.
This adds structured field-level merges:

  - tags                    → union (LWW-element-set semantics)
  - relationships           → union of edges keyed by (kind, target)
  - last_updated_at         → max(local, remote)
  - version                 → max(local, remote)
  - body                    → multi-value-register: emit both with markers
  - classification          → REFUSED to auto-merge (needs human)
  - authority               → max along the authority hierarchy
  - sync_class              → tighter wins (local-only > shared > publishable > client-visible)

`cyberos crdt merge <conflict-marker>` reads a sync conflict file and
produces a merged frontmatter + body proposal at
`outputs/staged-memories/crdt-merge-<id>.md`. Operator reviews + commits.
"""
from __future__ import annotations
import argparse
import re
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

ICT = timezone(timedelta(hours=7))

AUTHORITY_ORDER = {"human-edited": 4, "human-confirmed": 3, "llm-explicit": 2, "llm-implicit": 1}
SYNC_TIGHTER = {"local-only": 4, "shared": 3, "publishable": 2, "client-visible": 1}


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


def merge_fm(local: dict, remote: dict) -> tuple[dict, list[str]]:
    """Return (merged_fm, notes_about_decisions)."""
    notes = []
    out = dict(local)  # start from local as base

    # tags: union, preserving order
    lt = local.get("tags") or []
    rt = remote.get("tags") or []
    merged_tags = list(dict.fromkeys(lt + [t for t in rt if t not in lt]))
    if merged_tags != lt:
        out["tags"] = merged_tags
        notes.append(f"tags: merged {len(lt)} local + {len(rt)} remote → {len(merged_tags)} union")

    # relationships: union by (kind, target)
    lr = local.get("relationships") or []
    rr = remote.get("relationships") or []
    seen = set()
    merged_rel = []
    for e in (lr + rr):
        if not isinstance(e, dict):
            continue
        key = (e.get("kind"), e.get("target"))
        if key in seen:
            continue
        seen.add(key); merged_rel.append(e)
    if merged_rel != lr:
        out["relationships"] = merged_rel
        notes.append(f"relationships: merged {len(lr)} + {len(rr)} → {len(merged_rel)}")

    # last_updated_at: max
    l_ts = local.get("last_updated_at", "")
    r_ts = remote.get("last_updated_at", "")
    if r_ts > l_ts:
        out["last_updated_at"] = r_ts
        notes.append(f"last_updated_at: took remote ({r_ts}) > local ({l_ts})")

    # version: max
    if int(remote.get("version", 1)) > int(local.get("version", 1)):
        out["version"] = int(remote.get("version", 1))
        notes.append(f"version: took remote ({remote.get('version')}) > local ({local.get('version')})")

    # authority: max
    la = AUTHORITY_ORDER.get(local.get("authority", ""), 0)
    ra = AUTHORITY_ORDER.get(remote.get("authority", ""), 0)
    if ra > la:
        out["authority"] = remote.get("authority")
        notes.append(f"authority: took remote ({remote.get('authority')}) > local")

    # sync_class: tighter wins
    ls = SYNC_TIGHTER.get(local.get("sync_class", ""), 0)
    rs = SYNC_TIGHTER.get(remote.get("sync_class", ""), 0)
    if rs > ls:
        out["sync_class"] = remote.get("sync_class")
        notes.append(f"sync_class: tightened to remote ({remote.get('sync_class')})")

    # classification: REFUSE to auto-merge
    if local.get("classification") != remote.get("classification"):
        notes.append(f"⚠ classification MISMATCH: local={local.get('classification')!r} remote={remote.get('classification')!r} — refusing auto-merge; manual decision required")

    return out, notes


def cmd_merge(args):
    brain_root = find_brain()
    conflict = Path(args.conflict)
    if not conflict.exists():
        conflict = brain_root / ".cyberos-memory" / args.conflict
    if not conflict.exists():
        print(f"  no such conflict: {args.conflict}", file=sys.stderr)
        return 2

    text = conflict.read_text(encoding="utf-8")
    # Extract local + remote paths from the marker (per cyberos_sync conflict format)
    local_m = re.search(r"Local:\s*`(\S+?)`", text)
    remote_m = re.search(r"Remote:\s*`(\S+?)`", text)
    if not local_m or not remote_m:
        print(f"  could not parse conflict marker", file=sys.stderr)
        return 2

    brain = brain_root / ".cyberos-memory"
    local_path = brain / local_m.group(1)
    remote_path = brain / remote_m.group(1)
    if not local_path.exists():
        print(f"  local missing: {local_path}", file=sys.stderr); return 2
    # remote may live in outputs/sync-staging/ if it was a real sync; for testing we assume local
    if not remote_path.exists():
        # Search staging
        for c in (brain_root / "outputs" / "sync-staging").rglob("*.md") if (brain_root / "outputs" / "sync-staging").exists() else []:
            if c.read_text(encoding="utf-8") and remote_m.group(1) in c.as_posix():
                remote_path = c
                break

    local_fm, local_body = parse_frontmatter(local_path.read_text())
    remote_fm, remote_body = parse_frontmatter(remote_path.read_text() if remote_path.exists() else local_path.read_text())

    merged_fm, notes = merge_fm(local_fm, remote_fm)

    # Body: multi-value register if different
    if local_body.strip() != remote_body.strip():
        merged_body = (
            "## CRDT merge — body diverged\n\n"
            "### Local body\n\n" + local_body.strip() + "\n\n"
            "### Remote body\n\n" + remote_body.strip() + "\n\n"
            "_(Operator: pick one or compose a new body, then commit via brain_writer.)_\n"
        )
        notes.append(f"body: diverged — multi-value-register output")
    else:
        merged_body = local_body

    # Render
    import yaml
    out_text = "---\n" + yaml.safe_dump(merged_fm, sort_keys=False) + "---\n" + merged_body
    out_dir = brain_root / "outputs" / "staged-memories"
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = out_dir / f"crdt-merge-{conflict.stem}.md"
    out_path.write_text(out_text, encoding="utf-8")

    print(f"  ✓ staged merge: {out_path.relative_to(brain_root)}")
    print(f"  Merge notes:")
    for n in notes:
        print(f"    {n}")
    return 0


def main():
    p = argparse.ArgumentParser(description="CRDT-style merge for sync conflicts (Batch 13 / Tier C)")
    sub = p.add_subparsers(dest="cmd", required=True)
    pm = sub.add_parser("merge"); pm.add_argument("conflict"); pm.set_defaults(func=cmd_merge)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
