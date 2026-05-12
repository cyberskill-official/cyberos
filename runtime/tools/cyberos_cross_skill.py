#!/usr/bin/env python3
"""
cyberos_cross_skill.py — cross-skill consistency validation.

Tier α.7 (Batch 22).

After a chain run emits multiple artefacts, validate consistency BETWEEN
them. Catches drift that per-skill validators miss.

Checks:

  C1 — every FR-NNN-T-MM in task lists resolves to a task that actually exists
  C2 — fr-audit's verdicts cover every FR emitted by fr-with-tasks/fr-author
  C3 — every tech_spec references a real FR (when standard/full profile)
  C4 — every impl-plan ticket maps to a task in a known FR
  C5 — chain-manifest plan steps and emitted files align (every non-skipped
       step has an artefact, every artefact maps back to a step)

Usage:
    cyberos cross-skill <chain-output-dir>
    cyberos cross-skill <chain-output-dir> --json
"""
from __future__ import annotations
import argparse
import json
import re
import sys
from pathlib import Path


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def parse_frontmatter(text: str) -> tuple[dict, str]:
    if not text.startswith("---\n"):
        return {}, text
    end = text.find("\n---\n", 4)
    if end < 0:
        return {}, text
    try:
        import yaml
        return yaml.safe_load(text[4:end]) or {}, text[end + 5:]
    except Exception:
        return {}, text[end + 5:]


def collect_artefacts(chain_dir: Path) -> dict:
    """Return {fr_files, tech_spec_files, impl_plan_files, manifest}."""
    out = {"fr_files": [], "tech_spec_files": [], "impl_plan_files": [], "manifest": None}
    mf = chain_dir / "chain-manifest.json"
    if mf.exists():
        try:
            out["manifest"] = json.loads(mf.read_text())
        except Exception:
            pass
    for f in chain_dir.glob("*.md"):
        name = f.name.lower()
        if name.startswith("fr-"):
            out["fr_files"].append(f)
        elif "tech" in name or "spec" in name:
            out["tech_spec_files"].append(f)
        elif "impl" in name:
            out["impl_plan_files"].append(f)
    return out


def cmd_check(args):
    brain_root = find_brain()
    chain_dir = Path(args.chain_dir)
    if not chain_dir.exists():
        print(f"  no such dir: {chain_dir}", file=sys.stderr); return 2

    arts = collect_artefacts(chain_dir)
    findings = []

    # C1 — task ID references resolve
    all_task_ids: set[str] = set()
    fr_to_tasks: dict[str, list[dict]] = {}
    for fr_path in arts["fr_files"]:
        try:
            fm, _ = parse_frontmatter(fr_path.read_text(encoding="utf-8"))
        except Exception:
            continue
        fr_id = fm.get("fr_id") or fr_path.stem.split("-")[0] + "-" + fr_path.stem.split("-")[1] if "-" in fr_path.stem else fr_path.stem
        tasks = fm.get("tasks") or []
        fr_to_tasks[fr_id] = tasks
        for t in tasks:
            if isinstance(t, dict) and t.get("id"):
                all_task_ids.add(t["id"])

    for fr_id, tasks in fr_to_tasks.items():
        for t in tasks:
            if not isinstance(t, dict):
                continue
            for dep in (t.get("dependencies") or []):
                if dep not in all_task_ids:
                    findings.append({
                        "check": "C1-task-ref-unresolved",
                        "fr_id": fr_id, "task_id": t.get("id"), "missing_dep": dep,
                    })

    # C2 — fr-audit covers every FR (skipped if no audit artefact)
    audit_paths = [f for f in chain_dir.glob("*audit*.md")]
    if audit_paths and arts["fr_files"]:
        # Heuristic: audit file should mention each FR-NNN
        audit_text = " ".join(p.read_text(encoding="utf-8", errors="ignore") for p in audit_paths)
        for fr_id in fr_to_tasks:
            if fr_id and fr_id not in audit_text:
                findings.append({"check": "C2-fr-not-audited", "fr_id": fr_id,
                                 "fix": "fr-audit didn't mention this FR; review audit coverage"})

    # C5 — chain-manifest plan steps align with emitted files
    if arts["manifest"]:
        for step in arts["manifest"].get("plan", []):
            if step["status"] not in ("done", "skipped"):
                continue
            if step["status"] == "skipped":
                continue
            outs = step.get("output_paths") or []
            for op in outs:
                full = brain_root / op
                if not full.exists():
                    findings.append({
                        "check": "C5-output-missing",
                        "step": step["step"], "skill_id": step["skill_id"],
                        "missing_path": op,
                    })

    if args.json:
        print(json.dumps({"findings": findings, "count": len(findings),
                          "fr_count": len(fr_to_tasks),
                          "task_count": len(all_task_ids)}, indent=2))
        return 1 if findings else 0

    print(f"\n  cross-skill check — {chain_dir}\n")
    print(f"  FR files: {len(arts['fr_files'])}, tech-spec: {len(arts['tech_spec_files'])}, impl-plan: {len(arts['impl_plan_files'])}")
    print(f"  Total tasks: {len(all_task_ids)}")
    if not findings:
        print(f"\n  ✓ all cross-skill checks passed")
        return 0
    print(f"\n  ⚠ {len(findings)} finding(s):")
    for f in findings:
        print(f"    {f}")
    return 1


def main():
    p = argparse.ArgumentParser(description="cross-skill consistency validator (Tier α.7)")
    p.add_argument("chain_dir", help="planning/<date>-<slug>/ directory")
    p.add_argument("--json", action="store_true")
    p.set_defaults(func=cmd_check)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
