#!/usr/bin/env python3
"""Phase 4 (cyberos) — Regenerate per-module README index files.

For each docs/feature-requests/<module>/ folder, write a fresh README.md with:
- FR table (id, priority, slice, hours, title)
- Cross-module dep summary (depends on / depended on by)

Usage:
    python3 gen_module_readmes.py --project-root <path>
"""
import argparse, glob, os, re
from collections import defaultdict


def parse_fm(path):
    with open(path) as f:
        text = f.read()
    fm = {}
    m = re.match(r"---\n(.*?)\n---", text, re.DOTALL)
    if not m:
        return None
    for line in m.group(1).split("\n"):
        if ":" not in line:
            continue
        k, _, v = line.partition(":")
        k, v = k.strip(), v.strip()
        if k in ("depends_on", "blocks"):
            mm = re.search(r"\[(.*?)\]", v)
            if mm:
                inner = mm.group(1).strip()
                fm[k] = [x.strip() for x in inner.split(",") if x.strip()] if inner else []
            else:
                fm[k] = []
        elif k in ("id", "title", "module", "priority", "slice", "effort_hours", "status"):
            fm[k] = v.strip().strip('"').strip("'")
    return fm


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--project-root", required=True)
    args = ap.parse_args()

    fr_root = os.path.join(args.project_root, "docs/feature-requests")
    if not os.path.isdir(fr_root):
        print(f"ERROR: no docs/feature-requests/ at {fr_root}")
        return

    frs = {}
    for path in sorted(glob.glob(f"{fr_root}/**/FR-*.md", recursive=True)):
        if path.endswith(".audit.md"):
            continue
        fm = parse_fm(path)
        if not fm or "id" not in fm:
            continue
        fm["_path"] = path
        frs[fm["id"]] = fm

    by_folder = defaultdict(list)
    for fr in frs.values():
        by_folder[os.path.dirname(fr["_path"])].append(fr)

    for folder, fr_list in sorted(by_folder.items()):
        module_name = os.path.basename(folder).upper()
        fr_list_sorted = sorted(fr_list, key=lambda x: x["id"])
        total_hours = sum(float(fr.get("effort_hours", 0) or 0) for fr in fr_list_sorted)

        out = []
        from datetime import date
        out.append(f"# {module_name} module — feature request index")
        out.append("")
        out.append(f"_Generated {date.today().isoformat()} — {len(fr_list_sorted)} FRs, {total_hours:.0f} engineering-hours total._")
        out.append("")
        out.append("## FRs")
        out.append("")
        out.append("| FR | Priority | Slice | Hours | Title |")
        out.append("|---|---|---|---:|---|")
        for fr in fr_list_sorted:
            fr_id = fr["id"]
            priority = fr.get("priority", "?")
            slice_v = fr.get("slice", "?")
            hours = fr.get("effort_hours", "?")
            title = fr.get("title", "").strip('"').strip("'")
            path = os.path.basename(fr["_path"])
            out.append(f"| [{fr_id}]({path}) | {priority} | {slice_v} | {hours} | {title[:100]} |")
        out.append("")
        out.append("## Cross-module dependencies")
        out.append("")
        own_ids = set(fr["id"] for fr in fr_list_sorted)
        deps_in = defaultdict(list)
        deps_out = defaultdict(list)
        for fr in fr_list_sorted:
            for dep in fr.get("depends_on", []):
                if dep in frs and dep not in own_ids:
                    dep_module = os.path.basename(os.path.dirname(frs[dep]["_path"])).upper()
                    deps_in[dep_module].append((fr["id"], dep))
        for fr_id, fr in frs.items():
            if fr_id in own_ids:
                continue
            for dep in fr.get("depends_on", []):
                if dep in own_ids:
                    fr_module = os.path.basename(os.path.dirname(fr["_path"])).upper()
                    deps_out[fr_module].append((fr["id"], dep))
        if deps_in:
            out.append("**This module depends on:**\n")
            for mod, edges in sorted(deps_in.items()):
                out.append(f"- **{mod}**: {', '.join(f'{s}→{d}' for s, d in sorted(edges))}")
            out.append("")
        if deps_out:
            out.append("**This module is depended on by:**\n")
            for mod, edges in sorted(deps_out.items()):
                out.append(f"- **{mod}**: {', '.join(f'{s}→{d}' for s, d in sorted(edges))}")
            out.append("")
        out.append("---\n")
        out.append("_See `../IMPLEMENTATION_ORDER.md` for the full topological build sequence._")

        with open(os.path.join(folder, "README.md"), "w") as f:
            f.write("\n".join(out))
        print(f"Wrote {folder}/README.md ({len(fr_list_sorted)} FRs, {total_hours:.0f}h)")


if __name__ == "__main__":
    main()
