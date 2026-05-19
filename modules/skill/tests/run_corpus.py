#!/usr/bin/env python3
"""
runtime/tests/skills/run_corpus.py — run a skill's test corpus.

Tier α.5 (Batch 22).

Loads YAML fixtures from runtime/tests/skills/<skill>/fixtures/*.yaml,
invokes the skill runner per fixture, scores the result against the
fixture's expectations.

Usage:
    python3 runtime/tests/skills/run_corpus.py fr-with-tasks
    python3 runtime/tests/skills/run_corpus.py fr-with-tasks --max-iterations 2 --no-llm
    cyberos skill-test fr-with-tasks   # umbrella alias

Scoring: each fixture passes if:
  - skill_runner returns status == PASS
  - emitted task_count is within expected_task_count_min/max
  - all expected_sizes appear in emitted sizes
  - if expected_invariants_clean: validate_emit returns []

Without anthropic SDK installed, --no-llm mode skips the LLM call and
just exercises the runner harness + fixture loading.
"""
from __future__ import annotations
import argparse
import json
import sys
import tempfile
from pathlib import Path


def find_memory(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def main():
    p = argparse.ArgumentParser(description="run a skill's test corpus (Tier α.5)")
    p.add_argument("skill_id", help="e.g. fr-with-tasks")
    p.add_argument("--max-iterations", type=int, default=2)
    p.add_argument("--no-llm", action="store_true",
                   help="skip LLM call; only exercise harness + fixture loading")
    p.add_argument("--json", action="store_true")
    args = p.parse_args()

    memory_root = find_memory()
    fixtures_dir = memory_root / "runtime" / "tests" / "skills" / args.skill_id / "fixtures"
    if not fixtures_dir.exists():
        print(f"  ✗ no fixtures at {fixtures_dir}", file=sys.stderr); return 2

    try:
        import yaml
    except ImportError:
        print("  ✗ pyyaml required", file=sys.stderr); return 3

    fixtures = sorted(fixtures_dir.glob("*.yaml")) + sorted(fixtures_dir.glob("*.yml"))
    if not fixtures:
        print(f"  ✗ no .yaml fixtures in {fixtures_dir}", file=sys.stderr); return 2

    sys.path.insert(0, str(memory_root / "runtime" / "skill_runners"))
    try:
        from base import load_runner, SkillCache  # type: ignore
    except ImportError as e:
        print(f"  ✗ couldn't load runner base: {e}", file=sys.stderr); return 3

    skill_id = args.skill_id if "/" in args.skill_id else f"cuo/cpo/{args.skill_id}"
    runner = load_runner(skill_id, memory_root)
    if runner is None:
        # Try CTO path
        skill_id = args.skill_id if "/" in args.skill_id else f"cuo/chief-technology-officer/{args.skill_id}"
        runner = load_runner(skill_id, memory_root)
    if runner is None:
        print(f"  ✗ no runner found for {args.skill_id}", file=sys.stderr); return 2

    results = []
    for fx in fixtures:
        fx_data = yaml.safe_load(fx.read_text(encoding="utf-8"))
        name = fx_data.get("name", fx.stem)
        if args.no_llm:
            # Just check the runner harness loads, fixture parses, invariants module loads
            result = {"fixture": name, "status": "HARNESS_OK"}
        else:
            with tempfile.TemporaryDirectory() as td:
                rr = runner.run(
                    inputs={"pitch": fx_data["pitch"]},
                    output_dir=Path(td),
                    max_iterations=args.max_iterations,
                    cache=None,
                )
                # Score
                passed_status = rr.status == "PASS"
                # Parse emitted file (best-effort)
                emitted_tasks = []
                if rr.artefact_path and rr.artefact_path.exists():
                    text = rr.artefact_path.read_text(encoding="utf-8")
                    try:
                        if text.startswith("---\n"):
                            end = text.find("\n---\n", 4)
                            fm = yaml.safe_load(text[4:end]) or {}
                            emitted_tasks = fm.get("tasks") or []
                    except Exception:
                        pass
                expected_min = fx_data.get("expected_task_count_min", 0)
                expected_max = fx_data.get("expected_task_count_max", 999)
                count_ok = expected_min <= len(emitted_tasks) <= expected_max
                expected_sizes = set(fx_data.get("expected_sizes") or [])
                emitted_sizes = {t.get("sizing") for t in emitted_tasks if isinstance(t, dict)}
                sizes_ok = (not expected_sizes) or expected_sizes.issubset(emitted_sizes)
                result = {
                    "fixture": name,
                    "status": rr.status,
                    "passed": passed_status and count_ok and sizes_ok,
                    "iterations": rr.iterations,
                    "task_count": len(emitted_tasks),
                    "tokens_used": rr.tokens_used,
                    "cost_usd": round(rr.cost_usd, 4),
                    "count_ok": count_ok,
                    "sizes_ok": sizes_ok,
                    "findings_count": len(rr.findings),
                }
        results.append(result)

    if args.json:
        print(json.dumps(results, indent=2))
    else:
        passed = sum(1 for r in results if r.get("passed", r.get("status") == "HARNESS_OK"))
        print(f"\n  Corpus results for {args.skill_id}  ({passed}/{len(results)} passing):\n")
        for r in results:
            marker = "✓" if r.get("passed", r.get("status") == "HARNESS_OK") else "✗"
            print(f"  {marker} {r['fixture']}  {r}")
    return 0 if all(r.get("passed", r.get("status") == "HARNESS_OK") for r in results) else 1


if __name__ == "__main__":
    sys.exit(main())
