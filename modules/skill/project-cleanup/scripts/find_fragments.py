#!/usr/bin/env python3
"""Phase 1 — Inventory scanner.

Walks the project tree and identifies:
- Stale fragment files (small markdowns matching leftover patterns)
- Orphan audit files (audits without matching specs, or vice versa — cyberos-style)
- Broken markdown links
- Files in exclude paths are skipped

Output: structured JSON to stdout.

Usage:
    python3 find_fragments.py --project-root <path> [--threshold 80] [--scope auto|cyberos|generic]
"""
import argparse, fnmatch, json, os, re, sys
from datetime import datetime, timezone

DEFAULT_LEFTOVER_PATTERNS = [
    "*_SUMMARY.md", "*_PROGRESS.md", "*_NOTES.md",
    "*.md.bak", "*.old.md", "tmp_*.md", "draft_*.md",
]
DEFAULT_EXCLUDE_PATHS = [
    ".git/", "node_modules/", "target/", "dist/", "build/",
    ".venv/", "__pycache__/", ".cyberos/memory/store/",
]


def is_excluded(path: str, root: str, excludes: list[str]) -> bool:
    rel = os.path.relpath(path, root)
    return any(ex.rstrip("/") in rel.split(os.sep) for ex in excludes)


def matches_leftover(name: str, patterns: list[str]) -> bool:
    return any(fnmatch.fnmatch(name, p) for p in patterns)


def detect_scope(project_root: str) -> str:
    # Cyberos signal: BACKLOG.md plus the task-audit-co-located task-audit skill
    # (the latter moved 2026-05-18 from task-audit skill).
    has_backlog = os.path.exists(os.path.join(project_root, "docs/tasks/BACKLOG.md"))
    has_new_discipline = os.path.exists(os.path.join(project_root, "task-audit skill"))
    has_legacy_authoring = os.path.exists(os.path.join(project_root, "task-audit skill"))
    if has_backlog and (has_new_discipline or has_legacy_authoring):
        return "cyberos"
    return "generic"


def find_orphan_audits(project_root: str) -> list[dict]:
    """For cyberos repos: find TASK-*.audit.md files without matching TASK-*.md, and vice versa."""
    fr_dir = os.path.join(project_root, "docs/tasks")
    if not os.path.isdir(fr_dir):
        return []
    orphans = []
    for dirpath, _, files in os.walk(fr_dir):
        specs = {f[:-3] for f in files if f.startswith("TASK-") and f.endswith(".md") and not f.endswith(".audit.md")}
        audits = {f[:-len(".audit.md")] for f in files if f.endswith(".audit.md")}
        for spec_stem in specs - audits:
            orphans.append({"kind": "spec_no_audit", "path": os.path.join(dirpath, f"{spec_stem}.md")})
        for audit_stem in audits - specs:
            orphans.append({"kind": "audit_no_spec", "path": os.path.join(dirpath, f"{audit_stem}.audit.md")})
    return orphans


def _strip_code_blocks(text: str) -> str:
    """Remove fenced code blocks (```...``` and ~~~...~~~) so links inside
    example/source code aren't flagged as broken cross-doc references."""
    # Remove triple-backtick blocks (greedy across lines)
    text = re.sub(r"```.*?```", "", text, flags=re.DOTALL)
    # Remove triple-tilde blocks
    text = re.sub(r"~~~.*?~~~", "", text, flags=re.DOTALL)
    # Remove inline code spans (single backtick) — these are often e.g. `path.md`
    text = re.sub(r"`[^`\n]+`", "", text)
    return text


def find_broken_links(project_root: str, max_files: int = 1000) -> list[dict]:
    """Scan markdown files for [text](relative.md) links to non-existent files.

    Skips links found inside fenced code blocks (```...```, ~~~...~~~) and inline
    code spans so source-code examples don't produce false positives.
    """
    broken = []
    seen = 0
    md_link = re.compile(r"\[([^\]]+)\]\(([^)]+\.md)(#[^)]+)?\)")
    for dirpath, _, files in os.walk(project_root):
        if any(ex.rstrip("/") in dirpath for ex in DEFAULT_EXCLUDE_PATHS):
            continue
        for f in files:
            if not f.endswith(".md"):
                continue
            path = os.path.join(dirpath, f)
            seen += 1
            if seen > max_files:
                return broken
            try:
                with open(path) as fh:
                    text = fh.read()
            except (OSError, UnicodeDecodeError):
                continue
            text_no_code = _strip_code_blocks(text)
            for m in md_link.finditer(text_no_code):
                link_path = m.group(2)
                if link_path.startswith(("http://", "https://", "mailto:")):
                    continue
                # Skip obvious template placeholders (e.g. ./part-{}.md)
                if "{" in link_path or "<" in link_path:
                    continue
                resolved = os.path.normpath(os.path.join(dirpath, link_path))
                if not os.path.exists(resolved):
                    broken.append({
                        "from": os.path.relpath(path, project_root),
                        "to": link_path,
                        "resolved_missing": os.path.relpath(resolved, project_root),
                    })
    return broken


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--project-root", required=True)
    ap.add_argument("--threshold", type=int, default=80)
    ap.add_argument("--scope", default="auto", choices=["auto", "cyberos", "generic"])
    ap.add_argument("--leftover-pattern", action="append", default=None)
    ap.add_argument("--exclude", action="append", default=None)
    args = ap.parse_args()

    root = os.path.abspath(args.project_root)
    if not os.path.isdir(root):
        print(json.dumps({"error": f"project-root not a directory: {root}"}), file=sys.stderr)
        sys.exit(2)

    scope = args.scope if args.scope != "auto" else detect_scope(root)
    patterns = args.leftover_pattern or DEFAULT_LEFTOVER_PATTERNS
    excludes = args.exclude or DEFAULT_EXCLUDE_PATHS

    fragments = []     # small markdown files
    leftovers = []     # match leftover patterns
    files_scanned = 0

    for dirpath, dirs, files in os.walk(root):
        # prune excluded dirs in-place
        dirs[:] = [d for d in dirs if not is_excluded(os.path.join(dirpath, d), root, excludes)]
        for f in files:
            if not f.endswith(".md"):
                continue
            path = os.path.join(dirpath, f)
            files_scanned += 1
            try:
                stat = os.stat(path)
                with open(path) as fh:
                    line_count = sum(1 for _ in fh)
            except OSError:
                continue
            rel = os.path.relpath(path, root)
            if matches_leftover(f, patterns):
                leftovers.append({
                    "path": rel,
                    "lines": line_count,
                    "mtime": datetime.fromtimestamp(stat.st_mtime, tz=timezone.utc).isoformat(),
                    "reason": "matches_leftover_pattern",
                })
                continue
            if line_count < args.threshold and f not in ("README.md", "CHANGELOG.md", "task-audit skill", "task-audit skill", "LICENSE.md"):
                fragments.append({
                    "path": rel,
                    "lines": line_count,
                    "mtime": datetime.fromtimestamp(stat.st_mtime, tz=timezone.utc).isoformat(),
                })

    orphan_audits = find_orphan_audits(root) if scope == "cyberos" else []
    broken_links = find_broken_links(root)

    print(json.dumps({
        "project_root": root,
        "scope": scope,
        "files_scanned": files_scanned,
        "fragments_detected": len(fragments),
        "fragments": fragments[:50],   # cap report size
        "suspicious_leftovers": len(leftovers),
        "leftovers": leftovers,
        "orphan_audits": len(orphan_audits),
        "orphans": orphan_audits[:30],
        "broken_links": len(broken_links),
        "broken_links_sample": broken_links[:30],
    }, indent=2))


if __name__ == "__main__":
    main()
