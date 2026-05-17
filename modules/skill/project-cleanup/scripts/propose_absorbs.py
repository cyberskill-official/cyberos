#!/usr/bin/env python3
"""Phase 2 — Auto-detect suitable absorb target for each fragment.

Reads JSON from find_fragments.py (via stdin or --inventory).
For each fragment, proposes the best parent doc to merge into:
1. Same directory's README.md (preferred)
2. Any doc in same dir that names the fragment topic
3. Closest larger doc in same dir
4. Else: no_parent_found

Output: JSON merge proposals to stdout.

Usage:
    python3 find_fragments.py --project-root <p> | python3 propose_absorbs.py
"""
import json, os, re, sys, argparse


def find_parent(fragment_path: str, project_root: str) -> dict:
    abs_frag = os.path.join(project_root, fragment_path)
    if not os.path.exists(abs_frag):
        return {"parent": None, "rationale": "fragment_not_found"}
    parent_dir = os.path.dirname(abs_frag)
    readme = os.path.join(parent_dir, "README.md")
    if os.path.exists(readme):
        return {
            "parent": os.path.relpath(readme, project_root),
            "rationale": "same_dir_readme",
        }
    # find largest other .md in same dir
    candidates = []
    for f in os.listdir(parent_dir):
        if not f.endswith(".md"):
            continue
        p = os.path.join(parent_dir, f)
        if p == abs_frag:
            continue
        try:
            with open(p) as fh:
                lines = sum(1 for _ in fh)
        except OSError:
            continue
        candidates.append((lines, p))
    if candidates:
        candidates.sort(reverse=True)
        return {
            "parent": os.path.relpath(candidates[0][1], project_root),
            "rationale": "largest_sibling_md",
        }
    return {"parent": None, "rationale": "no_parent_found"}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--inventory", help="path to JSON output from find_fragments.py (default stdin)")
    args = ap.parse_args()

    raw = open(args.inventory).read() if args.inventory else sys.stdin.read()
    inv = json.loads(raw)
    proposals = []
    for frag in inv.get("fragments", []) + inv.get("leftovers", []):
        sel = find_parent(frag["path"], inv["project_root"])
        proposals.append({
            "fragment": frag["path"],
            "lines": frag.get("lines"),
            "proposed_parent": sel["parent"],
            "rationale": sel["rationale"],
            "action": "absorb_then_delete" if sel["parent"] else "manual_review",
        })

    print(json.dumps({
        "project_root": inv["project_root"],
        "scope": inv.get("scope"),
        "proposals": proposals,
        "total_proposals": len(proposals),
        "with_parent": sum(1 for p in proposals if p["proposed_parent"]),
        "manual_review_needed": sum(1 for p in proposals if not p["proposed_parent"]),
    }, indent=2))


if __name__ == "__main__":
    main()
