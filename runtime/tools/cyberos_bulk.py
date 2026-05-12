#!/usr/bin/env python3
"""
cyberos_bulk.py — bulk frontmatter edits across many memories.

Tier E.3 of post-catalog improvements (Batch 15).

Change a single frontmatter field across many memories matching a
filter. Safer than `cyberos migrate` for ad-hoc one-offs.

Refuses to bulk-set:
  - memory_id  (immutable)
  - audit_chain_head (manifest-only)
  - classification (manual decision)
  - authority (manual decision)

Usage:
    cyberos bulk-set source_freshness_tier=12 --filter tag:tech-stack
    cyberos bulk-set sync_class=publishable --filter scope:memories/decisions
    cyberos bulk-set tags+=migrated-batch-15 --filter "scope:memories/facts"
    cyberos bulk-unset expires_at --filter "tag:cyberos"

Filter syntax (any combination, AND):
    scope:<prefix>    classification:<value>
    tag:<tag>         authority:<value>
    sync_class:<v>    tombstoned:true|false
"""
from __future__ import annotations
import argparse
import re
import sys
from pathlib import Path

REFUSED_FIELDS = {"memory_id", "audit_chain_head", "created_at", "created_by"}


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


def parse_filters(filters: list[str]) -> dict:
    out = {}
    for f in filters:
        if ":" not in f:
            raise SystemExit(f"bad filter {f!r}; expect key:value")
        k, v = f.split(":", 1)
        out[k.strip()] = v.strip()
    return out


def memory_matches(fm: dict, rel: str, flt: dict) -> bool:
    if "scope" in flt and not (rel.startswith(flt["scope"]) or fm.get("scope") == flt["scope"]):
        return False
    if "classification" in flt and fm.get("classification") != flt["classification"]:
        return False
    if "authority" in flt and fm.get("authority") != flt["authority"]:
        return False
    if "sync_class" in flt and fm.get("sync_class") != flt["sync_class"]:
        return False
    if "tag" in flt:
        tags = fm.get("tags") or []
        if isinstance(tags, str):
            tags = [tags]
        if flt["tag"] not in tags:
            return False
    if "tombstoned" in flt:
        want = flt["tombstoned"].lower() in ("true", "1", "yes")
        if bool(fm.get("tombstoned")) != want:
            return False
    return True


def parse_assign(expr: str) -> tuple[str, str, str]:
    """Return (key, op, value). op is '=', '+=', or '-='."""
    for op in ("+=", "-=", "="):
        if op in expr:
            k, v = expr.split(op, 1)
            return k.strip(), op, v.strip()
    raise SystemExit(f"bad assignment {expr!r}; use key=value or key+=value")


def apply_set(fm: dict, key: str, op: str, value: str) -> bool:
    """Mutate fm in place. Returns True if changed."""
    if op == "=":
        # Try int / bool coercion
        new = value
        try:
            new = int(value)
        except ValueError:
            if value.lower() in ("true", "false"):
                new = value.lower() == "true"
            elif value.lower() == "null":
                new = None
        if fm.get(key) == new:
            return False
        fm[key] = new
        return True
    elif op == "+=":
        # List append (e.g. tags+=foo)
        cur = fm.get(key) or []
        if not isinstance(cur, list):
            return False
        if value in cur:
            return False
        fm[key] = list(cur) + [value]
        return True
    elif op == "-=":
        cur = fm.get(key) or []
        if not isinstance(cur, list) or value not in cur:
            return False
        fm[key] = [x for x in cur if x != value]
        return True
    return False


def write_back(md: Path, fm: dict, body: str):
    import yaml
    text = "---\n" + yaml.safe_dump(fm, sort_keys=False) + "---\n" + body
    md.write_text(text, encoding="utf-8")


def cmd_set(args):
    brain_root = find_brain()
    brain = brain_root / ".cyberos-memory"
    key, op, value = parse_assign(args.expression)
    if key in REFUSED_FIELDS:
        print(f"  ✗ refusing to bulk-set {key!r} (immutable / manual-decision)", file=sys.stderr)
        return 2
    if key in ("classification", "authority") and not args.allow_protected:
        print(f"  ✗ refusing to bulk-set {key!r} without --allow-protected", file=sys.stderr)
        return 2
    flt = parse_filters(args.filter)

    candidates = []
    for md in brain.rglob("*.md"):
        if not md.is_file() or md.name.startswith("."):
            continue
        rel = md.relative_to(brain).as_posix()
        if rel.startswith(("audit/", "index/", "exports/", "meta/templates/", "meta/protocol-history/", ".branches/")):
            continue
        try:
            text = md.read_text(encoding="utf-8")
        except Exception:
            continue
        fm, body = parse_frontmatter(text)
        if not memory_matches(fm, rel, flt):
            continue
        if apply_set(fm, key, op, value):
            candidates.append((md, fm, body, rel))

    if not candidates:
        print(f"  no memories match filter or already at target value")
        return 0

    print(f"\n  Would change {len(candidates)} memory(s):  {key} {op} {value!r}")
    for md, fm, body, rel in candidates[:10]:
        print(f"    {rel}")
    if len(candidates) > 10:
        print(f"    … +{len(candidates) - 10} more")

    if not args.apply:
        print(f"\n  (dry-run; pass --apply to write)")
        return 0

    n = 0
    for md, fm, body, rel in candidates:
        write_back(md, fm, body)
        n += 1
    print(f"\n  ✓ applied to {n} memories")
    return 0


def cmd_unset(args):
    # Implement as bulk-set with =null
    args.expression = f"{args.field}=null"
    return cmd_set(args)


def main():
    p = argparse.ArgumentParser(description="bulk frontmatter edits (Tier E.3)")
    sub = p.add_subparsers(dest="cmd", required=True)
    ps = sub.add_parser("set")
    ps.add_argument("expression", help="e.g. tags+=migrated, sync_class=publishable")
    ps.add_argument("--filter", action="append", default=[], help="key:value filter (repeatable)")
    ps.add_argument("--apply", action="store_true")
    ps.add_argument("--allow-protected", action="store_true")
    ps.set_defaults(func=cmd_set)
    pu = sub.add_parser("unset")
    pu.add_argument("field")
    pu.add_argument("--filter", action="append", default=[])
    pu.add_argument("--apply", action="store_true")
    pu.add_argument("--allow-protected", action="store_true")
    pu.set_defaults(func=cmd_unset)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
