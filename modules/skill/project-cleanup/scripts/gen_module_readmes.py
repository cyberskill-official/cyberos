#!/usr/bin/env python3
"""Phase 4 (cyberos) — Regenerate per-module README index files.

For each docs/tasks/<module>/ folder, write a fresh README.md with:
- Task table (id, priority, slice, hours, title)
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

    task_root = os.path.join(args.project_root, "docs/tasks")
    if not os.path.isdir(task_root):
        print(f"ERROR: no docs/tasks/ at {task_root}")
        return

    tasks = {}
    for path in sorted(glob.glob(f"{task_root}/**/TASK-*.md", recursive=True)):
        if path.endswith(".audit.md"):
            continue
        fm = parse_fm(path)
        if not fm or "id" not in fm:
            continue
        fm["_path"] = path
        tasks[fm["id"]] = fm

    by_folder = defaultdict(list)
    for task in tasks.values():
        by_folder[os.path.dirname(task["_path"])].append(task)

    for folder, task_list in sorted(by_folder.items()):
        module_name = os.path.basename(folder).upper()
        task_list_sorted = sorted(task_list, key=lambda x: x["id"])
        total_hours = sum(float(task.get("effort_hours", 0) or 0) for task in task_list_sorted)

        out = []
        from datetime import date
        out.append(f"# {module_name} module — task index")
        out.append("")
        out.append(f"_Generated {date.today().isoformat()} — {len(task_list_sorted)} tasks, {total_hours:.0f} engineering-hours total._")
        out.append("")
        out.append("## tasks")
        out.append("")
        out.append("| Task | Priority | Slice | Hours | Title |")
        out.append("|---|---|---|---:|---|")
        for task in task_list_sorted:
            task_id = task["id"]
            priority = task.get("priority", "?")
            slice_v = task.get("slice", "?")
            hours = task.get("effort_hours", "?")
            title = task.get("title", "").strip('"').strip("'")
            path = os.path.basename(task["_path"])
            out.append(f"| [{task_id}]({path}) | {priority} | {slice_v} | {hours} | {title[:100]} |")
        out.append("")
        out.append("## Cross-module dependencies")
        out.append("")
        own_ids = set(task["id"] for task in task_list_sorted)
        deps_in = defaultdict(list)
        deps_out = defaultdict(list)
        for task in task_list_sorted:
            for dep in task.get("depends_on", []):
                if dep in tasks and dep not in own_ids:
                    dep_module = os.path.basename(os.path.dirname(tasks[dep]["_path"])).upper()
                    deps_in[dep_module].append((task["id"], dep))
        for task_id, task in tasks.items():
            if task_id in own_ids:
                continue
            for dep in task.get("depends_on", []):
                if dep in own_ids:
                    task_module = os.path.basename(os.path.dirname(task["_path"])).upper()
                    deps_out[task_module].append((task["id"], dep))
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
        print(f"Wrote {folder}/README.md ({len(task_list_sorted)} tasks, {total_hours:.0f}h)")


if __name__ == "__main__":
    main()
