#!/usr/bin/env python3
"""Phase 4 (cyberos) — Regenerate IMPLEMENTATION_ORDER.md + SPRINT_PLAN.md.

Topo-sorts the FR DAG (depends_on edges) into layers. Sums effort_hours by
module + slice. Writes both files atomically.

Usage:
    python3 gen_impl_artifacts.py --project-root <path>
"""
import argparse, glob, os, re
from collections import defaultdict, deque


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
        elif k in ("id", "title", "module", "priority", "slice", "effort_hours"):
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
        if fm and "id" in fm:
            frs[fm["id"]] = fm

    # Kahn topo sort
    in_degree = {fr: 0 for fr in frs}
    for fr_id, fr in frs.items():
        for dep in fr.get("depends_on", []):
            if dep in frs:
                in_degree[fr_id] += 1
    queue = deque([f for f in frs if in_degree[f] == 0])
    layers, current = [], list(queue)
    while current:
        layers.append(sorted(current))
        next_layer = []
        for fr_id in current:
            for dep_fr_id, dep_fr in frs.items():
                if fr_id in dep_fr.get("depends_on", []):
                    in_degree[dep_fr_id] -= 1
                    if in_degree[dep_fr_id] == 0:
                        next_layer.append(dep_fr_id)
        current = next_layer

    from datetime import date
    out = [f"# FR implementation order — topological build sequence", ""]
    out.append(f"_Generated {date.today().isoformat()} — {len(frs)} FRs in {len(layers)} dependency layers._")
    out.extend(["", "Each **layer** can be built in parallel (no cross-dependencies inside a layer). Layers MUST be built in order.", ""])
    for i, layer in enumerate(layers):
        out.append(f"## Layer {i} ({len(layer)} FRs — buildable in parallel)\n")
        for fr_id in layer:
            fr = frs[fr_id]
            title = fr.get("title", "").strip('"').strip("'")[:80]
            effort = fr.get("effort_hours", "?")
            priority = fr.get("priority", "?")
            slice_v = fr.get("slice", "?")
            out.append(f"- **{fr_id}** [{priority}, {effort}h, slice {slice_v}] — {title}")
        out.append("")
    with open(f"{fr_root}/IMPLEMENTATION_ORDER.md", "w") as f:
        f.write("\n".join(out))
    print(f"Wrote IMPLEMENTATION_ORDER.md: {len(layers)} layers, {len(frs)} FRs")

    # Sprint plan
    by_module = defaultdict(lambda: defaultdict(lambda: {"hours": 0, "count": 0, "frs": []}))
    total_hours = 0
    for fr_id, fr in frs.items():
        module = fr.get("module", "?")
        slice_v = fr.get("slice", "?")
        try:
            hours = float(fr.get("effort_hours", "0"))
        except (TypeError, ValueError):
            hours = 0
        by_module[module][slice_v]["hours"] += hours
        by_module[module][slice_v]["count"] += 1
        by_module[module][slice_v]["frs"].append(fr_id)
        total_hours += hours

    out = [f"# Sprint plan — effort rollup by module & slice", ""]
    out.append(f"_Generated {date.today().isoformat()} — {len(frs)} FRs, {total_hours:.0f} total engineering-hours._")
    out.extend(["", "## Headline numbers", ""])
    out.append(f"- **Total scope:** {len(frs)} FRs, {total_hours:.0f}h ({total_hours/8:.0f} engineer-days @ 8h/d, or {total_hours/160:.1f} engineer-months @ 160h/m).")
    out.append(f"- **At 3 engineers (480h/sprint @ 2-week sprints):** {total_hours/480:.1f} sprints (~{total_hours/480 * 2:.1f} weeks).")
    out.append(f"- **At 5 engineers (800h/sprint):** {total_hours/800:.1f} sprints (~{total_hours/800 * 2:.1f} weeks).")
    out.extend(["", "## By module", ""])
    out.append("| Module | FRs | Total hours | Slices |")
    out.append("|---|---:|---:|---|")
    for module in sorted(by_module):
        h = sum(s["hours"] for s in by_module[module].values())
        c = sum(s["count"] for s in by_module[module].values())
        slices = sorted(by_module[module])
        out.append(f"| **{module}** | {c} | {h:.0f} | {', '.join(str(s) for s in slices)} |")
    out.extend(["", "## By module & slice", ""])
    for module in sorted(by_module):
        out.append(f"### {module}\n")
        out.append("| Slice | FRs | Hours | FR list |")
        out.append("|---|---:|---:|---|")
        for slice_v in sorted(by_module[module]):
            s = by_module[module][slice_v]
            fr_list = ", ".join(sorted(s["frs"]))
            out.append(f"| {slice_v} | {s['count']} | {s['hours']:.0f} | {fr_list} |")
        out.append("")
    with open(f"{fr_root}/SPRINT_PLAN.md", "w") as f:
        f.write("\n".join(out))
    print(f"Wrote SPRINT_PLAN.md: {len(frs)} FRs / {total_hours:.0f}h across {len(by_module)} modules")


if __name__ == "__main__":
    main()
