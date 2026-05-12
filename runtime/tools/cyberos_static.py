#!/usr/bin/env python3
"""
cyberos_static.py — render BRAIN as a static HTML site.

Batch 14 (Tier D) of post-catalog improvements.

Walks `.cyberos-memory/` and emits a static HTML tree under
`.cyberos-memory/cache/site/`. Mobile-friendly. Read your BRAIN from a phone via a
local file server or by syncing the folder to a phone-accessible drive.

Each memory becomes a page; the index lists all memories grouped by
scope. No JavaScript; pure HTML + CSS.

Usage:
    cyberos static
    cyberos static --out ~/cyberos-mobile/
"""
from __future__ import annotations
import argparse
import html
import re
import sys
from pathlib import Path


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


CSS = """body{font:15px/1.5 -apple-system,BlinkMacSystemFont,sans-serif;max-width:42rem;margin:0 auto;padding:1rem;color:#222;background:#fff}
@media (prefers-color-scheme:dark){body{background:#111;color:#eee}a{color:#6bf}pre{background:#222!important}}
h1,h2,h3{line-height:1.2}h1{font-size:1.5rem}h2{font-size:1.2rem}
pre{background:#f5f5f5;padding:.6rem;border-radius:6px;overflow-x:auto;font-size:13px;white-space:pre-wrap}
nav{margin-bottom:1rem}nav a{margin-right:.8rem}
.scope{font-size:.85rem;color:#888}
"""


def md_to_html(text: str) -> str:
    """Minimal Markdown → HTML. Not a full parser; handles headings, lists, code, paragraphs."""
    out = []
    in_pre = False
    for line in text.splitlines():
        if line.startswith("```"):
            out.append("<pre>" if not in_pre else "</pre>")
            in_pre = not in_pre
            continue
        if in_pre:
            out.append(html.escape(line))
            continue
        if line.startswith("# "):
            out.append(f"<h1>{html.escape(line[2:])}</h1>")
        elif line.startswith("## "):
            out.append(f"<h2>{html.escape(line[3:])}</h2>")
        elif line.startswith("### "):
            out.append(f"<h3>{html.escape(line[4:])}</h3>")
        elif line.startswith("- "):
            out.append(f"<li>{html.escape(line[2:])}</li>")
        elif not line.strip():
            out.append("")
        else:
            out.append(f"<p>{html.escape(line)}</p>")
    return "\n".join(out)


def main():
    p = argparse.ArgumentParser(description="render BRAIN as static HTML (Batch 14 / Tier D)")
    p.add_argument("--out", default=None, help="output directory (default: .cyberos-memory/cache/site/)")
    args = p.parse_args()

    brain_root = find_brain()
    out_dir = Path(args.out).expanduser() if args.out else (brain_root / "outputs" / "site")
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "memories").mkdir(exist_ok=True)

    brain = brain_root / ".cyberos-memory"
    by_scope: dict[str, list] = {}
    pages = 0
    for md in sorted(brain.rglob("*.md")):
        if not md.is_file() or md.name.startswith("."):
            continue
        rel = md.relative_to(brain).as_posix()
        if rel.startswith(("audit/", "index/", "exports/", "meta/templates/", "meta/protocol-history/", ".branches/")):
            continue
        try:
            text = md.read_text(encoding="utf-8")
        except Exception:
            continue
        # Strip frontmatter for the body render
        body = text
        if text.startswith("---\n"):
            end = text.find("\n---\n", 4)
            if end > 0:
                body = text[end + 5:]
        slug = rel.replace("/", "_").replace(".md", "")
        page = f"""<!doctype html><html><head><meta charset=utf-8><meta name=viewport content="width=device-width,initial-scale=1">
<title>{html.escape(rel)}</title><style>{CSS}</style></head><body>
<nav><a href="../index.html">← all memories</a></nav>
<p class=scope>{html.escape(rel)}</p>
{md_to_html(body)}
</body></html>"""
        (out_dir / "memories" / f"{slug}.html").write_text(page, encoding="utf-8")
        scope = rel.split("/")[0] if "/" in rel else "root"
        by_scope.setdefault(scope, []).append((rel, slug))
        pages += 1

    # Index page
    sections = []
    for scope in sorted(by_scope):
        rows = "".join(f"<li><a href='memories/{slug}.html'>{html.escape(rel)}</a></li>"
                       for rel, slug in sorted(by_scope[scope]))
        sections.append(f"<h2>{html.escape(scope)} ({len(by_scope[scope])})</h2><ul>{rows}</ul>")
    index = f"""<!doctype html><html><head><meta charset=utf-8><meta name=viewport content="width=device-width,initial-scale=1">
<title>CyberOS BRAIN</title><style>{CSS}</style></head><body>
<h1>CyberOS BRAIN</h1>
<p class=scope>{pages} memories</p>
{"".join(sections)}
</body></html>"""
    (out_dir / "index.html").write_text(index, encoding="utf-8")

    print(f"  ✓ rendered {pages} pages → {out_dir}")
    print(f"  open: file://{out_dir / 'index.html'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
