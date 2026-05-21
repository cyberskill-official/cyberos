#!/usr/bin/env python3
"""
voice_check.py — Voice linter for CyberOS protocol docs.

Lints docs/CyberOS-*.md for:
  - em dashes (— and –)
  - AI vocabulary words (gstack /codex voice standard)

Aspect 7.2 of the Layer-1 improvement catalog.

Usage:
    python3 memory/tools/voice_check.py [path-or-glob ...]
    python3 memory/tools/voice_check.py --fix [path]      # interactive
    python3 memory/tools/voice_check.py --strict          # exit 1 on any finding (CI mode)

Default: lints AGENTS.md and README.md at the module root.
(CHANGELOG is descriptive — exempt).

Exit codes:
  0   no findings (or all auto-fixable in --fix mode)
  1   findings present (strict mode) OR  unfixable findings remain
  2   tool error / invalid input
"""
from __future__ import annotations
import argparse
import re
import sys
from pathlib import Path

# AI vocabulary list from gstack /codex voice standard (verbatim)
AI_VOCAB = {
    "delve", "crucial", "robust", "comprehensive", "nuanced", "multifaceted",
    "furthermore", "moreover", "additionally", "pivotal", "landscape", "tapestry",
    "underscore", "foster", "showcase", "intricate", "vibrant", "fundamental",
    "significant",
}

# Em dashes (U+2014) and en dashes (U+2013)
EM_DASH = "—"
EN_DASH = "–"

# Exempt sections (case-insensitive, whole-line match)
EXEMPT_HEADER_PATTERNS = [
    re.compile(r"^##?\s+CHANGELOG", re.I),
    re.compile(r"^##?\s+History", re.I),
]

# Lines inside fenced code blocks are exempt
def _strip_code_fences(text: str) -> str:
    """Replace lines inside ```...``` blocks with empty placeholders so they don't match."""
    out = []
    in_fence = False
    for line in text.split("\n"):
        if line.startswith("```") or line.startswith("~~~"):
            in_fence = not in_fence
            out.append("")
        elif in_fence:
            out.append("")
        else:
            out.append(line)
    return "\n".join(out)

def lint_file(path: Path, strict=False) -> list[dict]:
    """Return list of {line, col, severity, code, message} findings."""
    if not path.exists():
        return [{"line": 0, "col": 0, "severity": "ERROR", "code": "no-file", "message": f"file not found: {path}"}]
    raw = path.read_text(encoding="utf-8")
    text = _strip_code_fences(raw)
    findings = []

    # Em / en dashes
    for i, line in enumerate(text.split("\n"), 1):
        # Skip exempt section headers
        if any(p.match(line) for p in EXEMPT_HEADER_PATTERNS):
            continue
        for ch, code in ((EM_DASH, "em-dash"), (EN_DASH, "en-dash")):
            col = line.find(ch)
            if col >= 0:
                findings.append({
                    "line": i, "col": col + 1,
                    "severity": "WARN", "code": code,
                    "message": f"{code} detected — {line[max(0,col-30):col+30].strip()}"
                })

    # AI vocabulary (word-boundary, case-insensitive)
    for word in AI_VOCAB:
        pattern = re.compile(r"\b" + re.escape(word) + r"\b", re.I)
        for i, line in enumerate(text.split("\n"), 1):
            for m in pattern.finditer(line):
                findings.append({
                    "line": i, "col": m.start() + 1,
                    "severity": "WARN", "code": f"ai-vocab:{word.lower()}",
                    "message": f"AI vocab '{word}' — {line[max(0,m.start()-25):m.end()+25].strip()}"
                })

    # Sort by line, then col
    findings.sort(key=lambda f: (f["line"], f["col"]))
    return findings

def _tty():
    return sys.stdout.isatty() and __import__("os").environ.get("TERM") not in ("dumb", "")

def _c(text, code):
    if not _tty():
        return text
    return f"\033[{code}m{text}\033[0m"

OK = lambda s: _c(s, "32")
WARN = lambda s: _c(s, "33")
ERR = lambda s: _c(s, "31")
DIM = lambda s: _c(s, "2")
BOLD = lambda s: _c(s, "1")

def main():
    p = argparse.ArgumentParser(description="Voice linter for CyberOS protocol docs")
    p.add_argument("paths", nargs="*", help="paths or globs (default: AGENTS.md, README.md at module root)")
    p.add_argument("--strict", action="store_true", help="exit 1 on any finding (CI mode)")
    p.add_argument("--summary", action="store_true", help="summary only, no per-line output")
    args = p.parse_args()

    # Resolve targets
    if args.paths:
        targets = [Path(x) for x in args.paths]
    else:
        # Default targets — search upward from CWD for a directory
        # containing AGENTS.md (module root layout).
        root = Path.cwd()
        targets = []
        for cand in (root, *root.parents):
            if (cand / "AGENTS.md").is_file() and (cand / "memory.schema.json").is_file():
                targets = [cand / "AGENTS.md", cand / "README.md"]
                break
            # Also check modules/memory/ layout from repo root
            mem = cand / "modules" / "memory"
            if (mem / "AGENTS.md").is_file():
                targets = [mem / "AGENTS.md", mem / "README.md"]
                break
        targets = [t for t in targets if t.exists()]
        if not targets:
            print(ERR("ERROR:") + " no module root with AGENTS.md found; pass paths explicitly", file=sys.stderr)
            return 2

    total = 0
    files_with_findings = 0
    for t in targets:
        findings = lint_file(t)
        if not findings:
            if not args.summary:
                print(f"{OK('✓')} {t}")
            continue
        files_with_findings += 1
        total += len(findings)
        print(f"\n{BOLD(str(t))}  {WARN(str(len(findings)) + ' findings')}")
        if not args.summary:
            # Group by code
            by_code = {}
            for f in findings:
                by_code.setdefault(f["code"], []).append(f)
            for code, items in sorted(by_code.items()):
                print(f"  {WARN(code)} ({len(items)}x)")
                for f in items[:3]:
                    print(f"    line {f['line']:4d}:{f['col']:3d}  {f['message'][:90]}")
                if len(items) > 3:
                    print(f"    {DIM(f'... and {len(items)-3} more')}")

    print(f"\n{BOLD('Summary:')} {files_with_findings} file{'s' if files_with_findings != 1 else ''} with findings; {total} total")

    if args.strict and total > 0:
        return 1
    return 0

if __name__ == "__main__":
    sys.exit(main())
