#!/usr/bin/env python3
"""awh_promote.py — faithful, zero-LLM promotion of ready_to_test tasks to done.

The ship-tasks workflow gates testing->done at step 28 (awh-gate): it reruns
the task's MODULE goldenset (the real build+test suite vs the sealed baseline) out of band,
and step 29 flips testing->done only if that rerun is GREEN. This script applies exactly
that bar, minus the LLM doc-authoring steps (1-27), which regenerate artifacts that are not
the gate's acceptance criterion:

  - For each gated module, it RE-RUNS the gate now (not trusting the stored baseline -
    "agent self-certification is not trust").
  - Every ready_to_test task in a module whose gate is GREEN is promoted to done.
  - ready_to_test tasks in ungated modules (e.g. ai, red-deferred) are HELD with a reason.

It never regenerates the authoring artifacts (context map, ADR, code-review, coverage). If a
specific task needs those, run it through the full workflow with a real --invoker later.

  python3 scripts/awh_promote.py            # REPORT only - no file changes
  python3 scripts/awh_promote.py --apply    # flip status ready_to_test->done + write a ledger

Run on your Mac from the repo root. Needs the toolchain the suites use (cargo/pytest/make)
and pyyaml for awh. Never commits - review `git diff -- docs/tasks` and commit yourself.
"""
import json
import os
import re
import subprocess
import sys
import tempfile
import time
from pathlib import Path

REPO = Path(subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip())
# A module is "gated" if it has a golden set. Auto-detected so adding modules/<m>/.awh/
# goldenset.yaml (e.g. ai) includes it with no edit here. A goldenset without a captured
# baseline still counts as gated but reruns RED (run_gate returns "no goldenset/baseline").
GATED = sorted(p.parent.parent.name for p in (REPO / "modules").glob("*/.awh/goldenset.yaml"))
TASK_DIR = REPO / "docs" / "tasks"
LEDGER = REPO / ".awh" / "promotion-log.jsonl"
# Hardened for quoted/trailing values, same shape as scripts/rebaseline_task_status.py.
STATUS_RE = re.compile(r'^(status:\s*["\'`]?)(ready_to_test)(["\'`]?.*)$', re.M)


def have_yaml() -> bool:
    return subprocess.run([sys.executable, "-c", "import yaml"],
                          stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0


def run_gate(m: str):
    """Rerun module m's awh gate now. Returns (weighted_pass, verdict_str)."""
    gs = REPO / f"modules/{m}/.awh/goldenset.yaml"
    base = REPO / f"modules/{m}/.awh/eval-baseline.json"
    if not gs.exists() or not base.exists():
        return None, "no goldenset/baseline"
    fd, fresh = tempfile.mkstemp(suffix=".json")
    os.close(fd)
    env = dict(os.environ)
    env["PYTHONPATH"] = str(REPO / "tools" / "awh") + os.pathsep + env.get("PYTHONPATH", "")
    rc = subprocess.run(
        [sys.executable, "-m", "harness.cli", "eval", str(gs),
         "--base-dir", str(REPO), "--seeds", "1",
         "--baseline", str(base), "--max-regression", "0.0", "--out", fresh],
        env=env,
    ).returncode
    weighted, failing = None, ["<no report>"]
    try:
        d = json.loads(Path(fresh).read_text())
        ts = d.get("tasks", [])
        den = sum(t.get("weight", 1) for t in ts) or 1
        weighted = sum(t.get("pass_at_1", 0) * t.get("weight", 1) for t in ts) / den
        failing = [t.get("task_id", "?") for t in ts if t.get("pass_at_1", 0) < 1.0]
    except Exception as e:  # noqa: BLE001
        failing = [f"<unreadable report: {e}>"]
    finally:
        Path(fresh).unlink(missing_ok=True)
    green = rc == 0 and not failing
    return weighted, ("GREEN" if green else f"RED (rc={rc} failing={failing})")


def ready_frs(module_dir: str):
    d = TASK_DIR / module_dir
    if not d.is_dir():
        return []
    return [f for f in sorted(d.glob("*.md"))
            if re.search(r'^status:\s*["\'`]?ready_to_test', f.read_text(errors="replace")[:2000], re.M)]


def promote(f: Path, apply: bool) -> bool:
    txt = f.read_text()
    new, n = STATUS_RE.subn(lambda mm: f"{mm.group(1)}done{mm.group(3)}", txt, count=1)
    if n and apply:
        f.write_text(new)
    return n > 0


def main() -> int:
    apply = "--apply" in sys.argv[1:]
    if not have_yaml():
        print(f"awh needs pyyaml for this interpreter:\n    {sys.executable} -m pip install pyyaml")
        return 2

    print("== rerun each gated module's awh gate (out-of-band, not trusting the stored baseline) ==")
    gate = {}
    for m in GATED:
        w, verdict = run_gate(m)
        gate[m] = verdict
        print(f"  [gate] {m:7} -> {verdict}" + (f"  weighted={w:.3f}" if w is not None else ""))

    decisions = []  # (action, module, fr, reason)
    all_mods = sorted({p.parent.name for p in TASK_DIR.glob("*/*.md")})
    for mod in all_mods:
        tasks = ready_frs(mod)
        if not tasks:
            continue
        if mod in GATED and gate.get(mod) == "GREEN":
            for f in tasks:
                ok = promote(f, apply)
                decisions.append(("PROMOTE" if ok else "ERROR", mod, f.name, "module gate GREEN"))
        else:
            reason = "module gate RED" if mod in GATED else "module not gated (red-deferred)"
            for f in tasks:
                decisions.append(("HOLD", mod, f.name, reason))

    print(f"\n{'ACTION':8} {'MODULE':7} task  -- reason")
    for act, mod, fr, reason in decisions:
        print(f"{act:8} {mod:7} {fr}  -- {reason}")
    promoted = sum(1 for d in decisions if d[0] == "PROMOTE")
    held = sum(1 for d in decisions if d[0] == "HOLD")
    err = sum(1 for d in decisions if d[0] == "ERROR")
    print(f"\nsummary: {promoted} promote, {held} hold, {err} error   (apply={apply})")

    if apply:
        LEDGER.parent.mkdir(parents=True, exist_ok=True)
        with LEDGER.open("a") as L:
            for act, mod, fr, reason in decisions:
                L.write(json.dumps({"ts": time.time(), "action": act, "module": mod,
                                    "fr": fr, "reason": reason}) + "\n")
        print(f"ledger appended: {LEDGER}")
        print("review:  git --no-optional-locks diff -- docs/tasks   then rebuild docs + commit.")
    else:
        print("dry-run: no files changed. Re-run with --apply to flip status + write the ledger.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
