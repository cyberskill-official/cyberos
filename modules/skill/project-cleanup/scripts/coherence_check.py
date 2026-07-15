#!/usr/bin/env python3
"""Phase 4 (cyberos) — task DAG coherence + audit-score check.

For repos with a cyberos-style docs/tasks/ task catalog:
- Every depends_on edge has reciprocal blocks edge
- Every task has matching .audit.md
- Every audit shows score_post_revision: 10/10
- (Optional) per-module spec/audit balance

Usage:
    python3 coherence_check.py --project-root <path>

Exit code: 0 if clean, 1 if any issue.
"""
import argparse, glob, json, os, re, sys


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
        elif k == "id":
            fm["id"] = v
    return fm


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--project-root", required=True)
    ap.add_argument("--json", action="store_true", help="emit JSON output")
    args = ap.parse_args()

    task_root = os.path.join(args.project_root, "docs/tasks")
    if not os.path.isdir(task_root):
        print(json.dumps({"error": "no docs/tasks/ — not a cyberos repo"}), file=sys.stderr)
        sys.exit(2)

    all_frs = {}
    for path in sorted(glob.glob(f"{task_root}/**/TASK-*.md", recursive=True)):
        if path.endswith(".audit.md"):
            continue
        fm = parse_fm(path)
        if not fm or "id" not in fm:
            continue
        all_frs[fm["id"]] = {
            "path": path,
            "depends_on": fm.get("depends_on", []),
            "blocks": fm.get("blocks", []),
        }

    errors = []
    for task_id, info in all_frs.items():
        for dep in info["depends_on"]:
            if dep not in all_frs:
                errors.append(f"{task_id} depends_on missing task {dep}")
                continue
            if task_id not in all_frs[dep]["blocks"]:
                errors.append(f"RECIP: {dep}.blocks must include {task_id}")
        for blk in info["blocks"]:
            if blk not in all_frs:
                errors.append(f"{task_id} blocks missing task {blk}")
                continue
            if task_id not in all_frs[blk]["depends_on"]:
                errors.append(f"RECIP: {blk}.depends_on must include {task_id}")

    missing_audits = []
    not_perfect = []
    for task_id, info in all_frs.items():
        audit = info["path"].replace(".md", ".audit.md")
        if not os.path.exists(audit):
            missing_audits.append(task_id)
            continue
        with open(audit) as f:
            txt = f.read()
        if "score_post_revision: 10/10" not in txt:
            not_perfect.append(task_id)

    result = {
        "total_frs": len(all_frs),
        "reciprocity_errors": len(errors),
        "errors_sample": errors[:30],
        "missing_audits": len(missing_audits),
        "missing_audits_sample": missing_audits[:20],
        "not_10_10": len(not_perfect),
        "not_10_10_sample": not_perfect[:20],
        "overall": "PASS" if (not errors and not missing_audits and not not_perfect) else "FAIL",
    }
    if args.json:
        print(json.dumps(result, indent=2))
    else:
        print(f"Total tasks: {result['total_frs']}")
        print(f"Reciprocity errors: {result['reciprocity_errors']}")
        for e in result["errors_sample"]:
            print(f"  - {e}")
        print(f"Missing audits: {result['missing_audits']}")
        for m in result["missing_audits_sample"]:
            print(f"  - {m}")
        print(f"Not 10/10: {result['not_10_10']}")
        for n in result["not_10_10_sample"]:
            print(f"  - {n}")
        print(f"\nOverall: {result['overall']}")
    sys.exit(0 if result["overall"] == "PASS" else 1)


if __name__ == "__main__":
    main()
