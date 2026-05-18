#!/usr/bin/env python3
"""Escape HTML-eating placeholder angle brackets inside mermaid diagrams.

Browsers parse <div class="mermaid"> content as HTML before mermaid sees it.
Tokens like `<fixture>` are treated as malformed tags and stripped from the
DOM textContent. Mermaid then sees a corrupted diagram and renders a syntax
error.

This script surgically rewrites every `<placeholder>` inside mermaid blocks
to `&lt;placeholder&gt;` while preserving:
- `<br>` and `<br/>` (valid mermaid line-break syntax)
- `<b>`, `<i>`, `<em>`, `<strong>`, `<sub>`, `<sup>`, `<span>` (HTML formatting
  mermaid v11 supports in node labels)

Usage:
    python3 escape-placeholders.py --dry-run
    python3 escape-placeholders.py --apply
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
WEB_ROOT = REPO_ROOT / "website" / "docs"

# Tags mermaid + browsers handle safely inside .mermaid divs.
SAFE_TAGS = frozenset({"br", "b", "i", "em", "strong", "sub", "sup", "span", "div"})


def fix_block(block: str) -> tuple[str, list[str]]:
    """Escape unsafe placeholder-style <tag> patterns inside one mermaid block.

    Returns (new_block, list_of_escaped_tokens).
    """
    escaped: list[str] = []

    def repl(m: re.Match) -> str:
        full = m.group(0)
        tag = m.group(1).lower()
        if tag in SAFE_TAGS:
            return full
        escaped.append(m.group(1))
        # Escape the brackets only — preserve any attributes and trailing slash
        inner = full[1:-1]  # strip outer < >
        return f"&lt;{inner}&gt;"

    new_block = re.sub(
        r"<([a-zA-Z][a-zA-Z0-9_-]*)(?:\s+[^>]*)?\s*/?>",
        repl,
        block,
    )
    return new_block, escaped


def fix_file(path: Path, apply: bool) -> tuple[int, list[str]]:
    """Return (num_blocks_changed, list_of_escaped_tokens)."""
    text = path.read_text(encoding="utf-8")
    total_escaped: list[str] = []
    blocks_changed = 0

    def block_repl(m: re.Match) -> str:
        nonlocal blocks_changed
        prefix = m.group(1)
        body = m.group(2)
        suffix = m.group(3)
        new_body, escaped = fix_block(body)
        if escaped:
            blocks_changed += 1
            total_escaped.extend(escaped)
        return prefix + new_body + suffix

    new_text = re.sub(
        r'(<div class="mermaid">\s*)(.*?)(\s*</div>)',
        block_repl,
        text,
        flags=re.DOTALL,
    )

    if blocks_changed and apply:
        path.write_text(new_text, encoding="utf-8")
    return blocks_changed, total_escaped


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--apply", action="store_true", help="Write changes to disk")
    p.add_argument("--dry-run", action="store_true", help="Show what would change")
    args = p.parse_args()

    if not args.apply and not args.dry_run:
        print("Use --dry-run or --apply", file=sys.stderr)
        return 2

    apply = bool(args.apply)
    total_blocks = 0
    total_tokens: list[str] = []
    affected_files = []

    for f in sorted(WEB_ROOT.glob("**/*.html")):
        blocks, tokens = fix_file(f, apply)
        if blocks:
            total_blocks += blocks
            total_tokens.extend(tokens)
            affected_files.append((f.relative_to(REPO_ROOT), blocks, list(set(tokens))))

    action = "Applied" if apply else "Would change"
    print(f"{action}: {total_blocks} mermaid block(s) across {len(affected_files)} file(s)")
    print(f"Total tokens escaped: {len(total_tokens)} (distinct: {len(set(total_tokens))})")
    print()
    print("Per file:")
    for rel, blocks, tokens in affected_files:
        print(f"  {rel}: {blocks} block(s), tokens: {sorted(set(tokens))}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
