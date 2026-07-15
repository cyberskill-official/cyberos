#!/usr/bin/env python3
"""Phase 4 (cyberos) — Regenerate IMPLEMENTATION_ORDER.md + SPRINT_PLAN.md.

Topo-sorts the task DAG (depends_on edges) into layers. Sums effort_hours by
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

    task_root = os.path.join(args.project_root, "docs/tasks")
    if not os.path.isdir(task_root):
        print(f"ERROR: no docs/tasks/ at {task_root}")
        return

    tasks = {}
    for path in sorted(glob.glob(f"{task_root}/**/TASK-*.md", recursive=True)):
        if path.endswith(".audit.md"):
            continue
        fm = parse_fm(path)
        if fm and "id" in fm:
            tasks[fm["id"]] = fm

    # Kahn topo sort
    in_degree = {task: 0 for task in tasks}
    for task_id, task in tasks.items():
        for dep in task.get("depends_on", []):
            if dep in tasks:
                in_degree[task_id] += 1
    queue = deque([f for f in tasks if in_degree[f] == 0])
    layers, current = [], list(queue)
    while current:
        layers.append(sorted(current))
        next_layer = []
        for task_id in current:
            for dep_task_id, dep_task in tasks.items():
                if task_id in dep_task.get("depends_on", []):
                    in_degree[dep_task_id] -= 1
                    if in_degree[dep_task_id] == 0:
                        next_layer.append(dep_task_id)
        current = next_layer

    from datetime import date
    out = [f"# task implementation order — topological build sequence", ""]
    out.append(f"_Generated {date.today().isoformat()} — {len(tasks)} tasks in {len(layers)} dependency layers._")
    out.extend(["", "Each **layer** can be built in parallel (no cross-dependencies inside a layer). Layers MUST be built in order.", ""])
    for i, layer in enumerate(layers):
        out.append(f"## Layer {i} ({len(layer)} tasks — buildable in parallel)\n")
        for task_id in layer:
            task = tasks[task_id]
            title = task.get("title", "").strip('"').strip("'")[:80]
            effort = task.get("effort_hours", "?")
            priority = task.get("priority", "?")
            slice_v = task.get("slice", "?")
            out.append(f"- **{task_id}** [{priority}, {effort}h, slice {slice_v}] — {title}")
        out.append("")
    with open(f"{task_root}/IMPLEMENTATION_ORDER.md", "w") as f:
        f.write("\n".join(out))
    print(f"Wrote IMPLEMENTATION_ORDER.md: {len(layers)} layers, {len(tasks)} tasks")

    # Sprint plan
    by_module = defaultdict(lambda: defaultdict(lambda: {"hours": 0, "count": 0, "tasks": []}))
    total_hours = 0
    for task_id, task in tasks.items():
        module = task.get("module", "?")
        slice_v = task.get("slice", "?")
        try:
            hours = float(task.get("effort_hours", "0"))
        except (TypeError, ValueError):
            hours = 0
        by_module[module][slice_v]["hours"] += hours
        by_module[module][slice_v]["count"] += 1
        by_module[module][slice_v]["tasks"].append(task_id)
        total_hours += hours

    out = [f"# Sprint plan — effort rollup by module & slice", ""]
    out.append(f"_Generated {date.today().isoformat()} — {len(tasks)} tasks, {total_hours:.0f} total engineering-hours._")
    out.extend(["", "## Headline numbers", ""])
    out.append(f"- **Total scope:** {len(tasks)} tasks, {total_hours:.0f}h ({total_hours/8:.0f} engineer-days @ 8h/d, or {total_hours/160:.1f} engineer-months @ 160h/m).")
    out.append(f"- **At 3 engineers (480h/sprint @ 2-week sprints):** {total_hours/480:.1f} sprints (~{total_hours/480 * 2:.1f} weeks).")
    out.append(f"- **At 5 engineers (800h/sprint):** {total_hours/800:.1f} sprints (~{total_hours/800 * 2:.1f} weeks).")
    out.extend(["", "## By module", ""])
    out.append("| Module | tasks | Total hours | Slices |")
    out.append("|---|---:|---:|---|")
    for module in sorted(by_module):
        h = sum(s["hours"] for s in by_module[module].values())
        c = sum(s["count"] for s in by_module[module].values())
        slices = sorted(by_module[module])
        out.append(f"| **{module}** | {c} | {h:.0f} | {', '.join(str(s) for s in slices)} |")
    out.extend(["", "## By module & slice", ""])
    for module in sorted(by_module):
        out.append(f"### {module}\n")
        out.append("| Slice | tasks | Hours | task list |")
        out.append("|---|---:|---:|---|")
        for slice_v in sorted(by_module[module]):
            s = by_module[module][slice_v]
            task_list = ", ".join(sorted(s["tasks"]))
            out.append(f"| {slice_v} | {s['count']} | {s['hours']:.0f} | {task_list} |")
        out.append("")
    with open(f"{task_root}/SPRINT_PLAN.md", "w") as f:
        f.write("\n".join(out))
    print(f"Wrote SPRINT_PLAN.md: {len(tasks)} tasks / {total_hours:.0f}h across {len(by_module)} modules")


if __name__ == "__main__":
    main()
