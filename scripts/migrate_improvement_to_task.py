#!/usr/bin/env python3
"""One-time migration (2026-07-08): fold the three docs/improvement backlogs
(memory MEM-*, chat T-*, deep-audit IMP-*) into tasks as tasks with
class: improvement, renumbered to fresh ids (no legacy_id kept, per operator choice).

Generates task spec files + a migration map, remaps dependencies, and regenerates
docs/tasks/BACKLOG.md from all task frontmatter. Does NOT delete the old
docs/improvement/ dirs (done as a separate, verified step).

Usage: python3 scripts/migrate_improvement_to_task.py            # write task files + map
       python3 scripts/migrate_improvement_to_task.py --backlog  # also regenerate BACKLOG.md
"""
import re, sys, yaml
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
task = ROOT / "docs/tasks"
TODAY = "2026-07-08"
STATUS_ORDER = ["draft", "ready_to_implement", "implementing", "ready_to_review",
                "reviewing", "ready_to_test", "testing", "done", "on_hold", "closed"]

def slug(title):
    s = re.sub(r"[^a-z0-9]+", "-", title.lower()).strip("-")
    return (s[:48].rstrip("-")) or "task"

def map_status(s):
    # Migrated stubs are unaudited specs -> draft. done stays done; superseded -> closed.
    # "blocked" in the improvement backlogs means dependency/decision-blocked (still to-do,
    # captured by depends_on), NOT deliberately parked, so it maps to draft, not on_hold.
    s = (s or "").strip().lower().split(":")[0]
    if s == "done": return "done"
    if s == "superseded": return "closed"
    return "draft"

def map_pri(p):
    p = (p or "").strip().lower()
    if p in ("critical", "p0"): return "MUST"
    if p in ("low", "p3", "could"): return "COULD"
    return "SHOULD"

def parse_memory():
    data = yaml.safe_load((ROOT / "docs/improvement/memory/backlog.yaml").read_text())
    out = []
    for t in data.get("tasks", []):
        out.append(dict(old=t["id"], title=t["title"], phase=str(t.get("phase", "")),
                        status=t.get("status", ""), pri=t.get("priority", ""),
                        refs=[str(r) for r in (t.get("refs") or [])],
                        deps=[str(d) for d in (t.get("deps") or [])],
                        accept=t.get("accept", ""), program="memory", module="memory"))
    return out

def parse_md_table(path, id_prefix, cols, program, module):
    tasks, phase = [], ""
    for line in Path(path).read_text().splitlines():
        mh = re.match(r"^##\s+(Phase[^\n]*|Wave[^\n]*)", line)
        if mh:
            phase = mh.group(1).split("(")[0].strip(); continue
        if not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) < len(cols):
            continue
        row = dict(zip(cols, cells))
        oid = row.get("id", "")
        if not re.match(rf"^{id_prefix}-\d+$", oid):
            continue
        raw_deps = row.get("depends", "") or row.get("depends_on", "")
        deps = [] if raw_deps in ("-", "") else [d for d in re.split(r"[ ,]+", raw_deps) if re.match(rf"^{id_prefix}-\d+$", d)]
        raw_refs = row.get("crefs", "") or row.get("refs", "")
        refs = [] if raw_refs in ("-", "") else [r for r in re.split(r"[ ,]+", raw_refs) if r]
        tasks.append(dict(old=oid, title=row.get("title", ""), phase=phase,
                          status=row.get("status", ""), pri=row.get("pri", "") or row.get("prio", ""),
                          refs=refs, deps=deps, accept="", program=program, module=module))
    return tasks

def num(old):
    return int(re.search(r"(\d+)$", old).group(1))

def main():
    mem = parse_memory()
    chat = parse_md_table(ROOT / "docs/improvement/chat/BACKLOG.md", "T",
                          ["id", "pri", "eff", "title", "crefs", "depends", "status"], "chat", "chat")
    imp = parse_md_table(ROOT / "docs/improvement/BACKLOG.md", "IMP",
                         ["id", "title", "refs", "prio", "effort", "depends_on", "status"], "deep-audit", "improvement")

    groups = [("memory", "TASK-MEMORY", 201, mem), ("chat", "TASK-CHAT", 201, chat), ("improvement", "TASK-IMP", 1, imp)]
    idmap = {}
    for _, prefix, start, tasks in groups:
        for i, t in enumerate(sorted(tasks, key=lambda x: num(x["old"])), start=start):
            t["new"] = f"{prefix}-{i:03d}"
            idmap[t["old"]] = t["new"]

    written = 0
    for module_dir, prefix, _, tasks in groups:
        outdir = task / module_dir
        outdir.mkdir(parents=True, exist_ok=True)
        for t in tasks:
            nid = t["new"]
            st = map_status(t["status"])
            deps = [idmap.get(d, d) for d in t["deps"]]
            title = t["title"].replace('"', "'")
            body_accept = t["accept"].strip()
            refs_str = ", ".join(t["refs"]) if t["refs"] else "see the source improvement report"
            fname = f"{nid}-{slug(t['title'])}.md"
            fm = [
                "---",
                f"id: {nid}",
                f'title: "{title}"',
                f"module: {t['module']}",
                f"priority: {map_pri(t['pri'])}",
                f"status: {st}",
                "class: improvement",
                f"phase: {t['phase'] or 'n/a'}",
                f"refs: [{', '.join(t['refs'])}]",
                f"depends_on: [{', '.join(deps)}]",
                f"created: {TODAY}",
                "verify: N   # awh N/A until a goldenset is sealed for this area",
                "---",
            ]
            body = [
                f"# {nid}: {t['title']}",
                "",
                "## 1. Description",
                "",
                (body_accept or "Author the normative clauses when this task is picked up; it was migrated as a draft stub."),
                "",
                f"Migrated {TODAY} from the {t['program']} improvement backlog, folded into the task system as `class: improvement`. Source report refs: {refs_str}.",
                "",
                "## Acceptance criteria",
                "",
                (f"- [ ] {body_accept}" if body_accept else "- [ ] (to be authored from the source report before this task leaves draft)"),
                "",
            ]
            (outdir / fname).write_text("\n".join(fm) + "\n" + "\n".join(body))
            written += 1

    # migration map (human record; not a legacy_id field on the tasks)
    lines = ["# Improvement backlog migration map (2026-07-08)", "",
             "One-time renumber of the three docs/improvement backlogs into tasks as",
             "`class: improvement` tasks. No `legacy_id` was kept on the tasks (operator choice); this",
             "table is the only record linking old ids to new. Use it to reconcile the in-flight",
             "`auto/memory-enterprise` and `auto/chat-enterprise` branches after merge.", ""]
    for label, _, _, tasks in groups:
        lines += [f"## {label}", "", "| old | new | status | title |", "|---|---|---|---|"]
        for t in sorted(tasks, key=lambda x: num(x["old"])):
            lines.append(f"| {t['old']} | {t['new']} | {map_status(t['status'])} | {t['title'][:70]} |")
        lines.append("")
    (task / "improvement" / "MIGRATION-MAP-2026-07-08.md").write_text("\n".join(lines))

    print(f"wrote {written} task files (memory={len(mem)} chat={len(chat)} deep-audit={len(imp)})")
    print(f"id ranges: TASK-MEMORY-201..{200+len(mem)}, TASK-CHAT-201..{200+len(chat)}, TASK-IMP-001..{len(imp):03d}")
    if "--backlog" in sys.argv:
        regen_backlog()

def read_fm(p):
    txt = p.read_text()
    m = re.match(r"\A---\n(.*?)\n---\n", txt, re.S)
    if not m:
        return None
    try:
        d = yaml.safe_load(m.group(1))
        return d if isinstance(d, dict) else None
    except yaml.YAMLError:
        return None

def status_line(tally):
    # TASK-IMP-091 §1 #1.2: count lines come from the frontmatter tally alone -
    # STATUS_ORDER first (the committed convention), then any legal-but-unlisted status
    # (STATUS-REFERENCE.md §1.2 off-ramps: cannot_reproduce, duplicate) sorted, so no
    # status a task actually carries can silently drop out of a header or Totals line.
    order = STATUS_ORDER + sorted(s for s in tally if s not in STATUS_ORDER and s)
    return ", ".join(f"{tally[s]} {s}" for s in order if tally.get(s))

def regen_backlog():
    mods = {}
    unparseable = []
    for f in sorted(task.glob("*/TASK-*/spec.md")):
        if "_audits" in f.parts or "_archive" in f.parts:
            continue
        fm = read_fm(f)
        if not fm:
            unparseable.append(str(f))   # collected; the halt below names every file
            continue
        mod = f.parent.parent.name
        klass = str(fm.get("class", "product")).strip() or "product"
        mods.setdefault(mod, []).append(
            (f.parent.name, str(fm.get("status", "")).strip(), str(fm.get("title", "")), klass)
        )
    if unparseable:
        # TASK-IMP-091 §3: unparseable frontmatter HALTS the regen BEFORE any write,
        # naming every offending file - never a guessed row, and never the silent-skip
        # variant of the drift class TASK-IMP-086 backfilled (its gate-log E1).
        for u in unparseable:
            print(f"regen_backlog: unparseable frontmatter: {u}", file=sys.stderr)
        sys.exit(f"regen_backlog: {len(unparseable)} unparseable spec.md file(s); BACKLOG.md NOT written")
    totals = {}
    out = ["# CyberOS task backlog (regenerated 2026-07-09)", "",
           "Source of truth = task frontmatter. This file lists EVERY task folder - one row per",
           "task in every status (TASK-IMP-091); per-status counts sit in each module header. ONE backlog for both classes:",
           "`class: improvement` rows are tagged `(improvement)`, untagged rows are `class: product`.",
           "The `improvement` section below is the module folder for cross-cutting hardening tasks",
           "(docs/tasks/improvement/), indexed here like any other module. Regenerated by",
           "scripts/migrate_improvement_to_task.py --backlog.", ""]
    body = []
    for mod in sorted(mods):
        rows = mods[mod]
        counts = {}
        for _, st, _, _ in rows:
            counts[st] = counts.get(st, 0) + 1
            totals[st] = totals.get(st, 0) + 1
        body.append(f"## {mod}  ({status_line(counts)})")
        body.append("")
        # TASK-IMP-091 §1 #1.1: one row per task folder for EVERY frontmatter status,
        # stem-ascending. The ACTIVE filter that sat here dropped every terminal row on
        # regen - the recorded root cause of the drift TASK-IMP-086 backfilled (its
        # gate-log E1: zero rows for the fourteen done tasks, Totals 155 vs 158).
        for stem, st, title, kl in sorted(rows):
            tag = " (improvement)" if kl == "improvement" else ""
            body.append(f"- [{st}] {stem} - {title}{tag}")
        body.append("")
    out.append(f"Totals: {status_line(totals)}")
    out.append("")
    (task / "BACKLOG.md").write_text("\n".join(out + body))
    print(f"regenerated BACKLOG.md: {sum(len(v) for v in mods.values())} tasks across {len(mods)} modules")

if __name__ == "__main__":
    import sys
    if "--backlog" in sys.argv:
        # Regenerate docs/tasks/BACKLOG.md from task frontmatter only.
        # (The one-time migration in main() needs the retired docs/improvement/
        # tree and cannot run again after its deletion.)
        regen_backlog()
    else:
        main()
