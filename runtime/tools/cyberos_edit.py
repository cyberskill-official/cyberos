#!/usr/bin/env python3
"""
cyberos_edit.py — $EDITOR wrapper for memory edits.

Tier E.2 of post-catalog improvements (Batch 15).

Opens a memory in $EDITOR, validates on save, and commits the edit via
brain_writer as an op:str_replace (preserves audit history).

Usage:
    cyberos edit <memory-id-or-path>
    cyberos edit DEC-110
    cyberos edit memories/facts/FACT-015-*.md

Honors $EDITOR / $VISUAL; falls back to `vi` then `nano`.
"""
from __future__ import annotations
import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def resolve(brain_root: Path, ident: str) -> Path:
    brain = brain_root / ".cyberos-memory"
    p = Path(ident)
    if p.is_file():
        return p
    p2 = brain / ident
    if p2.is_file():
        return p2
    # PREFIX-NNN
    m = re.match(r"^([A-Z]+-\d+)", ident)
    if m:
        for md in brain.rglob(f"{m.group(1)}-*.md"):
            return md
    # memory_id
    if ident.startswith("mem_"):
        for md in brain.rglob("*.md"):
            try:
                if f"memory_id: {ident}" in md.read_text():
                    return md
            except Exception:
                continue
    raise SystemExit(f"could not resolve {ident!r}")


def pick_editor() -> list[str]:
    for env in ("VISUAL", "EDITOR"):
        v = os.environ.get(env)
        if v:
            return v.split()
    for bin_ in ("vi", "vim", "nano"):
        if shutil.which(bin_):
            return [bin_]
    raise SystemExit("no editor found (set $EDITOR or install vi/nano)")


def main():
    p = argparse.ArgumentParser(description="$EDITOR wrapper for memory edits (Tier E.2)")
    p.add_argument("memory", help="memory_id, full path, or PREFIX-NNN")
    p.add_argument("--no-validate", action="store_true", help="skip the post-edit validate")
    p.add_argument("--no-commit", action="store_true", help="edit in place; don't go through brain_writer")
    args = p.parse_args()

    brain_root = find_brain()
    target = resolve(brain_root, args.memory)
    print(f"  editing: {target.relative_to(brain_root)}")

    # Stage to a tmp file so we can detect actual changes
    with tempfile.NamedTemporaryFile(mode="w", suffix="-" + target.name, delete=False) as tf:
        tf.write(target.read_text(encoding="utf-8"))
        tmp_path = Path(tf.name)
    before = tmp_path.read_text(encoding="utf-8")

    editor = pick_editor()
    rc = subprocess.run([*editor, str(tmp_path)]).returncode
    if rc != 0:
        print(f"  editor exited rc={rc}; aborting", file=sys.stderr)
        tmp_path.unlink(missing_ok=True)
        return rc

    after = tmp_path.read_text(encoding="utf-8")
    if after == before:
        print(f"  no changes; aborting")
        tmp_path.unlink(missing_ok=True)
        return 0

    # Validate the new frontmatter parses
    if not args.no_validate:
        if not after.startswith("---\n") or "\n---\n" not in after[4:]:
            print(f"  ✗ post-edit frontmatter looks malformed; refusing to commit")
            print(f"  Stayed at: {tmp_path}")
            return 2

    if args.no_commit:
        target.write_text(after, encoding="utf-8")
        print(f"  ✓ wrote in place (no audit row)")
        tmp_path.unlink(missing_ok=True)
        return 0

    # Commit via brain_writer str-replace
    bw = brain_root / "outputs" / "brain_writer.py"
    if not bw.exists():
        target.write_text(after, encoding="utf-8")
        print(f"  ⚠ brain_writer.py not found; wrote in place")
        tmp_path.unlink(missing_ok=True)
        return 0

    actor = os.environ.get("CYBEROS_SUBJECT_ID", "subject:stephen-cheng")
    rel = target.relative_to(brain_root).as_posix()
    rc = subprocess.run(["python3", str(bw), "str-replace", actor, rel, str(tmp_path)]).returncode
    tmp_path.unlink(missing_ok=True)
    if rc == 0:
        print(f"  ✓ committed via brain_writer str-replace")
    return rc


if __name__ == "__main__":
    sys.exit(main())
