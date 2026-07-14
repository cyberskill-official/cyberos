#!/usr/bin/env python3
"""Suggest (and optionally apply) corrections for FR cited-test paths that do not exist.

Most FR cited tests do not resolve on disk: specs name granular per-clause tests that the
real suites consolidated or renamed. For FRs that claim completion (status ready_to_test) a
stale path is a real spec defect. This tool fuzzy-matches each unresolved cited test against
the real test files in that module's trees and proposes the most likely replacement (its full
repo-relative path).

  awh_cited_fixups.py                 # ready_to_test FRs (default), human report
  awh_cited_fixups.py --status all    # every FR
  awh_cited_fixups.py --json
  awh_cited_fixups.py --apply          # rewrite confident cited paths -> real paths (reviewable diff)

--apply replaces only confident suggestions, only the exact cited string, and is idempotent
(after a fix the path resolves exactly, so a second run is a no-op). It writes files but never
commits; review with git diff.
"""
from __future__ import annotations

import argparse
import difflib
import importlib.util
import json
import re
from pathlib import Path

_here = Path(__file__).parent
_spec = importlib.util.spec_from_file_location("agf", _here / "awh_goldenset_from_fr.py")
agf = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(agf)

SERVICE_OF = {
    "memory": ["memory"], "auth": ["auth"], "skill": ["skill-broker"], "proj": ["proj"],
    "chat": ["chat"], "email": ["email"], "ai": ["ai-gateway"], "mcp": ["mcp-gateway"],
    "obs": ["obs-collector", "obs-router", "obs-compliance-view"],
}
_cache: dict[str, dict[str, str]] = {}


def candidates(module: str) -> dict[str, str]:
    """basename -> full repo-relative path of real test files in this module's trees."""
    if module in _cache:
        return _cache[module]
    files: list[Path] = []
    td = Path("modules") / module / "tests"
    if td.is_dir():
        files += list(td.rglob("test_*.py"))
    for svc in SERVICE_OF.get(module, []):
        d = Path("services") / svc / "tests"
        if d.is_dir():
            files += list(d.rglob("*.rs"))
    out: dict[str, str] = {}
    for p in sorted(files):
        out.setdefault(p.name, str(p))  # first wins on rare basename collisions
    _cache[module] = out
    return out


def status_of(text: str) -> str:
    m = re.search(r"^status:\s*['\"`]?([A-Za-z_]+)", text, re.M)
    return m.group(1) if m else "?"


def suggest(cited: str, module: str, cutoff: float = 0.6) -> str | None:
    cand = candidates(module)
    hit = difflib.get_close_matches(Path(cited).name, list(cand), n=1, cutoff=cutoff)
    return cand[hit[0]] if hit else None


def run(status_filter: str, as_json: bool, apply: bool) -> int:
    rows = []
    files_changed = 0
    repl_count = 0
    for path in agf.fr_files():
        text = path.read_text(encoding="utf-8")
        st = status_of(text)
        if status_filter != "all" and st != status_filter:
            continue
        module = agf.fr_module(text, path)
        fid = re.search(r"^id:\s*(FR-[A-Z]+-\d+)", text, re.M)
        fid = fid.group(1) if fid else path.stem
        unresolved = []
        new_text = text
        for c in agf.cited_tests(text):
            real, _ = agf.resolve(c, module)
            if real:
                continue
            s = suggest(c, module)
            unresolved.append({"cited": c, "suggest": s})
            if apply and s and c != s:
                new_text = new_text.replace(c, s)
        if unresolved:
            rows.append({"fr": fid, "module": module, "status": st, "unresolved": unresolved})
        if apply and new_text != text:
            path.write_text(new_text, encoding="utf-8")
            files_changed += 1
            repl_count += sum(1 for u in unresolved if u["suggest"])

    if apply:
        print(f"applied: {repl_count} cited-path corrections across {files_changed} FR spec(s).")
        print("review with: git --no-optional-locks diff -- docs/tasks")
        return 0
    if as_json:
        print(json.dumps(rows, indent=2))
        return 0
    n_unres = sum(len(r["unresolved"]) for r in rows)
    n_sugg = sum(1 for r in rows for u in r["unresolved"] if u["suggest"])
    print(f"FRs (status={status_filter}) with unresolved cited tests: {len(rows)}")
    print(f"unresolved cited tests: {n_unres} | with a confident suggestion: {n_sugg} "
          f"| no match: {n_unres - n_sugg}")
    print()
    for r in rows:
        print(f"{r['fr']} [{r['module']}, {r['status']}]")
        for u in r["unresolved"]:
            arrow = f"-> {u['suggest']}" if u["suggest"] else "-> (no confident match; may be unwritten)"
            print(f"    {u['cited']}  {arrow}")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--status", default="ready_to_test",
                    help="FR status to scan (default ready_to_test; 'all' for every FR)")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--apply", action="store_true",
                    help="rewrite confident cited paths to real paths (reviewable, no commit)")
    args = ap.parse_args()
    return run(args.status, args.json, args.apply)


if __name__ == "__main__":
    raise SystemExit(main())
