#!/usr/bin/env python3
"""
scripts/unwrap-md.py — join paragraph-internal line breaks.

The CyberOS docs use ~80-column hard wraps for source readability, but
some markdown renderers (VS Code's preview, certain web viewers) honour
those hard breaks instead of collapsing them like the CommonMark spec
expects — sentences get cut mid-line.

This script unwraps **paragraphs** while preserving every other block
type: code fences, tables, lists, blockquotes, headings, YAML/TOML
frontmatter, HTML, and horizontal rules stay byte-identical.

Usage:
    python scripts/unwrap-md.py docs/**/*.md
    python scripts/unwrap-md.py --check docs/**/*.md     # exit 1 if any file would change
    python scripts/unwrap-md.py --diff docs/**/*.md      # show diffs without writing
"""

from __future__ import annotations

import argparse
import difflib
import re
import sys
from pathlib import Path

# Block-start patterns that we MUST NOT touch (and that suspend the
# paragraph-join behaviour for the lines they cover).
_CODE_FENCE = re.compile(r"^(\s*)(```|~~~)")
_HEADING    = re.compile(r"^\s*#{1,6}\s")
_HRULE      = re.compile(r"^\s*([-*_])(\s*\1){2,}\s*$")
_LIST_ITEM  = re.compile(r"^\s*([-*+]|\d+[.)])\s")
_BLOCKQUOTE = re.compile(r"^\s*>")
_TABLE_ROW  = re.compile(r"^\s*\|")
_HTML_TAG   = re.compile(r"^\s*</?[a-zA-Z][^>]*>")
# Frontmatter delimiter (`---` or `+++`) when at the very top of the file.
_FRONTMATTER_DELIM = re.compile(r"^(---|\+\+\+)\s*$")
# Lines ending in two-or-more trailing spaces are a hard <br> in
# CommonMark — preserve.
_HARD_BREAK = re.compile(r"  +$")


def _is_block_marker(line: str) -> bool:
    """A non-paragraph line that should never join with neighbours."""
    if not line.strip():
        return True
    if (_HEADING.match(line) or _HRULE.match(line) or _LIST_ITEM.match(line)
            or _BLOCKQUOTE.match(line) or _TABLE_ROW.match(line)
            or _HTML_TAG.match(line)):
        return True
    return False


def unwrap(source: str) -> str:
    """Return ``source`` with paragraph-internal hard wraps joined."""
    lines = source.split("\n")
    out: list[str] = []
    i = 0
    n = len(lines)

    # Optional YAML/TOML frontmatter at the top: copy verbatim.
    if n > 0 and _FRONTMATTER_DELIM.match(lines[0]):
        out.append(lines[0])
        i = 1
        while i < n and not _FRONTMATTER_DELIM.match(lines[i]):
            out.append(lines[i])
            i += 1
        if i < n:
            out.append(lines[i])
            i += 1

    in_fence: str | None = None  # the fence marker string, or None
    while i < n:
        line = lines[i]

        # --- code-fence handling --------------------------------------
        if in_fence is None:
            m = _CODE_FENCE.match(line)
            if m:
                in_fence = m.group(2)
                out.append(line)
                i += 1
                continue
        else:
            out.append(line)
            # Close fence on a matching marker line.
            if line.lstrip().startswith(in_fence):
                in_fence = None
            i += 1
            continue

        # --- block markers stay as-is ---------------------------------
        if _is_block_marker(line):
            out.append(line)
            i += 1
            continue

        # --- paragraph: join until the next block marker or fence -----
        buf = [line.rstrip()]
        i += 1
        while i < n:
            nxt = lines[i]
            # Stop on blank, fence, or any block marker.
            if not nxt.strip():
                break
            if _CODE_FENCE.match(nxt):
                break
            if _is_block_marker(nxt):
                break
            # CommonMark hard-break (two trailing spaces) is meaningful —
            # break the paragraph here so the renderer keeps the <br>.
            if _HARD_BREAK.search(buf[-1]):
                break
            buf.append(nxt.strip())
            i += 1
        out.append(" ".join(buf))

    return "\n".join(out)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("paths", nargs="+", help="markdown files to process")
    ap.add_argument("--check", action="store_true",
                    help="exit 1 if any file would be rewritten")
    ap.add_argument("--diff", action="store_true",
                    help="print unified diffs without writing")
    args = ap.parse_args(argv)

    n_changed = 0
    for path_str in args.paths:
        path = Path(path_str)
        if not path.is_file():
            print(f"  skip (not a file): {path}", file=sys.stderr)
            continue
        original = path.read_text(encoding="utf-8")
        rewritten = unwrap(original)
        if rewritten == original:
            continue
        n_changed += 1
        if args.diff:
            diff = difflib.unified_diff(
                original.splitlines(keepends=True),
                rewritten.splitlines(keepends=True),
                fromfile=str(path), tofile=str(path) + " (unwrapped)",
            )
            sys.stdout.writelines(diff)
        elif args.check:
            print(f"  would unwrap: {path}")
        else:
            path.write_text(rewritten, encoding="utf-8")
            print(f"  unwrapped: {path}")

    if args.check and n_changed > 0:
        print(f"\n{n_changed} file(s) need unwrapping", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
