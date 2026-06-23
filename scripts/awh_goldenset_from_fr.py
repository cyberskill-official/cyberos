#!/usr/bin/env python3
"""Derive an awh golden-set acceptance task from an FR's cited tests, and audit
cited-test path drift across the whole backlog.

Two findings this operationalizes:
1. FR specs cite test paths that are systematically stale (e.g. `tests/X.py` where the
   real file is `tests/core/X.py`, plus some renames). The awh gate needs a cited->real
   mapping pass rather than trusting the cited path verbatim.
2. Some cited tests do not exist on disk at all (the FR is a draft or the test was never
   written). Those FRs cannot be auto-gated until the test lands.

Usage:
  awh_goldenset_from_fr.py FR-MEMORY-116        # emit a golden-set acceptance task
  awh_goldenset_from_fr.py --audit              # scan every FR, report cited-test drift
  awh_goldenset_from_fr.py --audit --json       # machine-readable audit
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(".")
FR_ROOT = Path("docs/feature-requests")
# A test-file path mentioned anywhere in an FR: rooted (modules/.. , services/..) or
# module-relative (tests/..). Captures the .py / .rs file.
TEST_RE = re.compile(r"(?:modules/[\w.-]+/|services/[\w.-]+/)?tests/[\w./-]+\.(?:py|rs)")

CRATE = {  # module -> Rust crate package (where the module's service tests live)
    "memory": "cyberos-memory", "auth": "cyberos-auth", "proj": "cyberos-proj",
    "skill": "cyberos-skill-broker",
}


def fr_files():
    return sorted(p for p in FR_ROOT.rglob("FR-*.md") if not p.name.endswith(".audit.md"))


def fr_module(text: str, path: Path) -> str:
    m = re.search(r"^module:\s*([A-Za-z]+)", text, re.M)
    if m:
        return m.group(1).lower()
    parts = path.relative_to(FR_ROOT).parts
    return parts[0].lower() if parts else "?"


def cited_tests(text: str) -> list[str]:
    seen, out = set(), []
    for m in TEST_RE.findall(text):
        if m not in seen:
            seen.add(m)
            out.append(m)
    return out


def resolve(cited: str, module: str) -> tuple[str | None, str]:
    """Return (real_path, how) or (None, 'unresolved')."""
    p = Path(cited)
    if p.exists():
        return cited, "exact"
    # module-relative `tests/...` -> anchor under modules/<module>/
    if cited.startswith("tests/"):
        anchored = Path("modules") / module / cited
        if anchored.exists():
            return str(anchored), "anchored"
        # try inserting core/
        core = Path("modules") / module / "tests" / "core" / Path(cited).relative_to("tests")
        if core.exists():
            return str(core), "core-insert"
    # try inserting core/ for rooted modules/<m>/tests/<rest>
    mm = re.match(r"(modules/[\w.-]+/tests)/(.+)$", cited)
    if mm and not cited.startswith(mm.group(1) + "/core/"):
        core = Path(mm.group(1)) / "core" / mm.group(2)
        if core.exists():
            return str(core), "core-insert"
    # last resort: unique basename match within the module's test trees
    base = Path(cited).name
    hits = list(Path("modules").glob(f"{module}/tests/**/{base}")) + \
        list(Path("services").glob(f"*/tests/**/{base}"))
    hits = [h for h in hits if h.is_file()]
    if len(hits) == 1:
        return str(hits[0]), "basename"
    return None, "unresolved"


def emit_task(fr_id: str) -> int:
    files = [p for p in fr_files() if p.name.startswith(fr_id + "-") or p.stem == fr_id]
    if not files:
        print(f"error: no spec for {fr_id}", file=sys.stderr)
        return 2
    path = files[0]
    text = path.read_text(encoding="utf-8")
    module = fr_module(text, path)
    resolved, unresolved = [], []
    for c in cited_tests(text):
        real, how = resolve(c, module)
        (resolved if real else unresolved).append((c, real, how))
    print(f"# {fr_id}  module={module}  spec={path}")
    if unresolved:
        print(f"# WARNING unresolved cited tests: {[c for c, _, _ in unresolved]}")
    if not resolved:
        print("# no resolvable cited tests; cannot emit an acceptance task yet")
        return 1
    py = list(dict.fromkeys(r for _, r, _ in resolved if r.endswith(".py")))
    rs = list(dict.fromkeys(r for _, r, _ in resolved if r.endswith(".rs")))
    print("  # add to modules/%s/.awh/goldenset.yaml :" % module)
    if py:
        rel = " ".join(str(Path(p).relative_to(Path('modules') / module)) for p in py)
        print(f"""  - id: acceptance-{fr_id.lower()}
    description: held-out cited test(s) for {fr_id}
    cmd: "cd modules/{module} && python -m pytest {rel} -q"
    weight: 5.0
    timeout_sec: 300""")
    for r in rs:
        crate = CRATE.get(module, f"cyberos-{module}")
        stem = Path(r).stem
        print(f"""  - id: acceptance-{fr_id.lower()}-{stem}
    description: held-out cited test for {fr_id}
    cmd: "cd services && cargo test -p {crate} --test {stem}"
    weight: 5.0
    timeout_sec: 600""")
    return 0


def audit(as_json: bool) -> int:
    per_mod: dict[str, dict[str, int]] = {}
    unresolved_frs: list[dict] = []
    for path in fr_files():
        text = path.read_text(encoding="utf-8")
        module = fr_module(text, path)
        fid = re.search(r"^id:\s*(FR-[A-Z]+-\d+)", text, re.M)
        fid = fid.group(1) if fid else path.stem
        cites = cited_tests(text)
        d = per_mod.setdefault(module, {"frs": 0, "with_cites": 0, "cites": 0,
                                        "resolved": 0, "unresolved": 0, "mapped": 0})
        d["frs"] += 1
        if cites:
            d["with_cites"] += 1
        miss = []
        for c in cites:
            real, how = resolve(c, module)
            d["cites"] += 1
            if real:
                d["resolved"] += 1
                if how != "exact":
                    d["mapped"] += 1
            else:
                d["unresolved"] += 1
                miss.append(c)
        if miss:
            unresolved_frs.append({"fr": fid, "module": module, "unresolved": miss})
    if as_json:
        print(json.dumps({"per_module": per_mod, "unresolved_frs": unresolved_frs}, indent=2))
        return 0
    print("cited-test drift audit (per module):")
    print(f"  {'module':10s} {'FRs':>4} {'w/cites':>7} {'cites':>6} {'exact':>6} {'mapped':>6} {'unresolved':>10}")
    tot = {"frs": 0, "cites": 0, "resolved": 0, "mapped": 0, "unresolved": 0}
    for m in sorted(per_mod):
        d = per_mod[m]
        exact = d["resolved"] - d["mapped"]
        print(f"  {m:10s} {d['frs']:>4} {d['with_cites']:>7} {d['cites']:>6} {exact:>6} {d['mapped']:>6} {d['unresolved']:>10}")
        for k in tot:
            tot[k] += d.get(k, 0)
    print(f"  {'TOTAL':10s} {tot['frs']:>4} {'':>7} {tot['cites']:>6} {tot['resolved']-tot['mapped']:>6} {tot['mapped']:>6} {tot['unresolved']:>10}")
    print(f"\nFRs with at least one unresolved cited test: {len(unresolved_frs)}")
    for u in unresolved_frs[:20]:
        print(f"  {u['fr']:18s} {u['unresolved']}")
    if len(unresolved_frs) > 20:
        print(f"  ... {len(unresolved_frs) - 20} more")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("fr", nargs="?", help="FR id, e.g. FR-MEMORY-116")
    ap.add_argument("--audit", action="store_true", help="scan all FRs for cited-test drift")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    if args.audit:
        return audit(args.json)
    if not args.fr:
        ap.print_help()
        return 2
    return emit_task(args.fr)


if __name__ == "__main__":
    raise SystemExit(main())
