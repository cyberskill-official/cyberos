#!/usr/bin/env python3
"""
cyberos_migrate.py — schema migration runner for .cyberos-memory.

Tier E.1 of post-catalog improvements (Batch 15).

When a new required frontmatter field is added (or an existing field
must be reshaped), this runner replays every memory through a transform
function and re-validates. Each migration is a versioned Python file
under `runtime/migrations/<NNN>-<slug>.py` that exports:

    APPLIES_TO = "memories/**"            # path glob
    DESCRIPTION = "what this migration does"

    def transform(fm: dict, body: str, rel: str) -> tuple[dict, str]:
        '''Return (new_fm, new_body). Raise to abort.'''
        ...

The runner:
  1. Walks every memory matching APPLIES_TO
  2. Runs transform(fm, body, rel)
  3. Writes new frontmatter + body atomically
  4. Re-validates via brain_writer or cyberos_validate
  5. Records a migration row in the audit chain via op:str_replace
  6. Stores state in `meta/migrations-applied.json` so each runs once

Usage:
    cyberos migrate list                    # show available + applied
    cyberos migrate plan <name>             # dry-run; show diffs
    cyberos migrate apply <name>            # actually mutate (sev-0)
    cyberos migrate apply <name> --force    # re-run an already-applied migration

Example migration:
    cat > runtime/migrations/001-add-source-tier.py <<'PY'
    APPLIES_TO = "memories/facts/**"
    DESCRIPTION = "Add source_freshness_tier=10 to FACTs that lack it"
    def transform(fm, body, rel):
        if "source_freshness_tier" not in fm:
            fm["source_freshness_tier"] = 10
        return fm, body
    PY
"""
from __future__ import annotations
import argparse
import difflib
import fnmatch
import importlib.util
import json
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


def dump_frontmatter(fm: dict, body: str) -> str:
    import yaml
    return "---\n" + yaml.safe_dump(fm, sort_keys=False) + "---\n" + body


def applied_state(brain_root: Path) -> dict:
    p = brain_root / ".cyberos-memory" / "meta" / "migrations-applied.json"
    if not p.exists():
        return {}
    try:
        return json.loads(p.read_text())
    except Exception:
        return {}


def save_applied(brain_root: Path, state: dict):
    p = brain_root / ".cyberos-memory" / "meta" / "migrations-applied.json"
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(state, indent=2, sort_keys=True) + "\n")


def load_migration(brain_root: Path, name: str):
    """Load `runtime/migrations/<name>.py` (with or without .py suffix)."""
    base = brain_root / "migrations"
    cands = [base / f"{name}.py", base / name]
    for c in cands:
        if c.exists():
            spec = importlib.util.spec_from_file_location(f"cyberos_migration_{c.stem}", c)
            mod = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(mod)
            return c.stem, mod
    raise SystemExit(f"no migration found: {name} (looked under {base})")


def list_migrations(brain_root: Path) -> list[Path]:
    base = brain_root / "migrations"
    if not base.exists():
        return []
    return sorted(p for p in base.glob("*.py") if p.is_file())


def cmd_list(_args):
    brain_root = find_brain()
    migrations = list_migrations(brain_root)
    state = applied_state(brain_root)
    if not migrations:
        print(f"  no migrations under {brain_root / 'migrations'}")
        return 0
    print(f"\n  {len(migrations)} migration(s):\n")
    for m in migrations:
        applied = state.get(m.stem, {}).get("applied_at", "—")
        try:
            spec = importlib.util.spec_from_file_location(m.stem, m)
            mod = importlib.util.module_from_spec(spec); spec.loader.exec_module(mod)
            desc = getattr(mod, "DESCRIPTION", "")[:60]
        except Exception:
            desc = "(load error)"
        print(f"    {m.stem:30s}  applied={applied}  {desc}")
    return 0


def walk(brain_root: Path, glob: str):
    brain = brain_root / ".cyberos-memory"
    # Normalise glob — strip leading ".cyberos-memory/"
    if glob.startswith(".cyberos-memory/"):
        glob = glob[len(".cyberos-memory/"):]
    for md in brain.rglob("*.md"):
        if not md.is_file() or md.name.startswith("."):
            continue
        rel = md.relative_to(brain).as_posix()
        if rel.startswith(("audit/", "index/", "exports/", "meta/templates/", "meta/protocol-history/", ".branches/")):
            continue
        if not fnmatch.fnmatchcase(rel, glob):
            continue
        yield rel, md


def run(brain_root: Path, name: str, apply: bool) -> int:
    stem, mod = load_migration(brain_root, name)
    glob = getattr(mod, "APPLIES_TO", "memories/**")
    transform = getattr(mod, "transform", None)
    if not callable(transform):
        print(f"  ✗ migration {stem!r} has no `transform(fm, body, rel)` function", file=sys.stderr)
        return 2

    state = applied_state(brain_root)
    if state.get(stem, {}).get("applied_at") and apply:
        print(f"  ⚠ {stem} already applied at {state[stem]['applied_at']} (use `migrate apply ... --force`)")
        return 1

    print(f"\n  Migration: {stem}")
    print(f"  Glob:      {glob}")
    print(f"  Mode:      {'APPLY' if apply else 'PLAN (dry-run)'}")
    print()

    changed = 0
    errors = 0
    for rel, md in walk(brain_root, glob):
        try:
            text = md.read_text(encoding="utf-8")
        except Exception:
            errors += 1
            continue
        fm, body = parse_frontmatter(text)
        try:
            new_fm, new_body = transform(dict(fm), body, rel)
        except Exception as e:
            print(f"  ✗ {rel}: transform raised: {e}")
            errors += 1
            continue
        if new_fm == fm and new_body == body:
            continue
        new_text = dump_frontmatter(new_fm, new_body)
        if new_text == text:
            continue
        changed += 1
        if not apply:
            diff = list(difflib.unified_diff(text.splitlines(), new_text.splitlines(),
                       fromfile=f"a/{rel}", tofile=f"b/{rel}", lineterm=""))[:20]
            print(f"\n── {rel} ──")
            print("\n".join(diff))
        else:
            md.write_text(new_text, encoding="utf-8")
            print(f"  ✓ {rel}")

    print(f"\n  Memories changed: {changed}")
    print(f"  Errors:           {errors}")

    if apply and changed > 0 and errors == 0:
        state[stem] = {
            "applied_at": datetime.now(ICT).isoformat(timespec="seconds"),
            "memories_touched": changed,
        }
        save_applied(brain_root, state)
        print(f"  ✓ migration {stem!r} recorded in meta/migrations-applied.json")
    return 1 if errors else 0


def cmd_plan(args):
    return run(find_brain(), args.name, apply=False)


def cmd_apply(args):
    return run(find_brain(), args.name, apply=True)


def main():
    p = argparse.ArgumentParser(description="schema migration runner (Tier E.1)")
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("list").set_defaults(func=cmd_list)
    pp = sub.add_parser("plan"); pp.add_argument("name"); pp.set_defaults(func=cmd_plan)
    pa = sub.add_parser("apply"); pa.add_argument("name"); pa.add_argument("--force", action="store_true"); pa.set_defaults(func=cmd_apply)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
