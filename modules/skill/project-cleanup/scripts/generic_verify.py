#!/usr/bin/env python3
"""Phase 4 (generic) — Repo health checks for non-cyberos projects.

Verifies:
- Top-level README.md present
- CHANGELOG.md present + last-entry not stale (>30 days warn, >90 days fail)
- No broken markdown links
- No orphan markdown files (no incoming refs)
- License file present

Usage:
    python3 generic_verify.py --project-root <path> [--max-stale-days 30]
"""
import argparse, json, os, re, sys
from datetime import datetime, timedelta, timezone


def has_file(root, name):
    return os.path.exists(os.path.join(root, name))


def last_entry_date(changelog_path):
    """Heuristic: find the most recent ISO date or 'v.. YYYY-MM-DD' in the file."""
    if not os.path.exists(changelog_path):
        return None
    with open(changelog_path) as f:
        text = f.read()
    dates = re.findall(r"\b(\d{4}-\d{2}-\d{2})\b", text)
    if not dates:
        return None
    parsed = []
    for d in dates:
        try:
            parsed.append(datetime.strptime(d, "%Y-%m-%d").replace(tzinfo=timezone.utc))
        except ValueError:
            continue
    return max(parsed) if parsed else None


def find_broken_links(root):
    md_link = re.compile(r"\[([^\]]+)\]\(([^)]+\.md)(#[^)]+)?\)")
    broken = []
    excludes = (".git", "node_modules", "target", "dist", ".venv", "__pycache__")
    for dirpath, dirs, files in os.walk(root):
        dirs[:] = [d for d in dirs if d not in excludes]
        for f in files:
            if not f.endswith(".md"):
                continue
            path = os.path.join(dirpath, f)
            try:
                with open(path) as fh:
                    text = fh.read()
            except (OSError, UnicodeDecodeError):
                continue
            for m in md_link.finditer(text):
                link = m.group(2)
                if link.startswith(("http://", "https://", "mailto:")):
                    continue
                resolved = os.path.normpath(os.path.join(dirpath, link))
                if not os.path.exists(resolved):
                    broken.append({"from": os.path.relpath(path, root), "to": link})
    return broken


def find_orphan_mds(root, max_check=500):
    """Markdowns not referenced from any other markdown."""
    md_ref = re.compile(r"\[([^\]]+)\]\(([^)]+\.md)")
    all_mds = []
    excludes = (".git", "node_modules", "target", "dist", ".venv", "__pycache__")
    for dirpath, dirs, files in os.walk(root):
        dirs[:] = [d for d in dirs if d not in excludes]
        for f in files:
            if f.endswith(".md"):
                all_mds.append(os.path.join(dirpath, f))
                if len(all_mds) > max_check:
                    return []
    refs = set()
    for path in all_mds:
        try:
            with open(path) as fh:
                text = fh.read()
        except (OSError, UnicodeDecodeError):
            continue
        dirpath = os.path.dirname(path)
        for m in md_ref.finditer(text):
            link = m.group(2)
            if link.startswith(("http://", "https://")):
                continue
            resolved = os.path.normpath(os.path.join(dirpath, link))
            refs.add(resolved)

    orphans = []
    for md in all_mds:
        if md not in refs and os.path.basename(md) not in ("README.md", "CHANGELOG.md", "LICENSE.md", "CONTRIBUTING.md"):
            orphans.append(os.path.relpath(md, root))
    return orphans


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--project-root", required=True)
    ap.add_argument("--max-stale-days", type=int, default=30)
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    root = os.path.abspath(args.project_root)
    issues = []

    if not has_file(root, "README.md"):
        issues.append("missing top-level README.md")

    cl_path = os.path.join(root, "CHANGELOG.md")
    cl_age_days = None
    if not os.path.exists(cl_path):
        issues.append("missing CHANGELOG.md")
    else:
        last = last_entry_date(cl_path)
        if last:
            cl_age_days = (datetime.now(timezone.utc) - last).days
            if cl_age_days > 90:
                issues.append(f"CHANGELOG.md last entry is {cl_age_days} days old (>90 fail threshold)")
            elif cl_age_days > args.max_stale_days:
                issues.append(f"CHANGELOG.md last entry is {cl_age_days} days old (>{args.max_stale_days} warn threshold)")

    license_found = any(has_file(root, n) for n in ("LICENSE", "LICENSE.md", "LICENSE.txt"))
    if not license_found:
        issues.append("missing LICENSE file")

    broken = find_broken_links(root)
    orphans = find_orphan_mds(root)

    result = {
        "project_root": root,
        "issues": issues,
        "broken_links": len(broken),
        "broken_links_sample": broken[:20],
        "orphan_mds": len(orphans),
        "orphan_mds_sample": orphans[:20],
        "changelog_stale_days": cl_age_days,
        "overall": "PASS" if (not issues and not broken) else ("FAIL" if any("fail" in i for i in issues) or broken else "WARN"),
    }
    if args.json:
        print(json.dumps(result, indent=2))
    else:
        print(f"Issues: {len(issues)}")
        for i in issues:
            print(f"  - {i}")
        print(f"Broken links: {len(broken)}")
        for b in broken[:10]:
            print(f"  - {b['from']} → {b['to']}")
        print(f"Orphan markdowns: {len(orphans)}")
        print(f"\nOverall: {result['overall']}")
    sys.exit(0 if result["overall"] == "PASS" else 1)


if __name__ == "__main__":
    main()
