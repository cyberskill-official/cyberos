#!/usr/bin/env python3
"""check-docs-sync.py — every version / fixture-count surface must agree.

Mechanical enforcement of the "docs follow changes" invariant (AGENTS.md) and
the closure of BLINDSPOTS BS-11: README and index.html version strings were
the last surfaces kept in sync only by discipline. This script makes drift a
CI failure instead of a doc bug.

Surfaces checked against AUDIT.md's title version (the source of truth):
  package.json .version, package-lock.json .version (if present),
  pyproject.toml version (if present), evals/baseline.json .audit_md_version,
  CHANGELOG.md newest entry, improve/versions/AUDIT-<ver>.md existence,
  README.md "current release" marker, index.html badge/footer markers.
Fixture count on disk must equal baseline.json and every "N/N"-style count
mentioned in README.md / index.html / evals/README.md.

Stdlib only. Exit 0 = in sync; 1 = drift (each drift printed); 2 = usage.
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]   # repo root
CORE = ROOT / "core"
SITE = ROOT / "site"


def main():
    problems = []

    audit = (CORE / "AUDIT.md").read_text(encoding="utf-8")
    m = re.search(r"v\d+\.\d+\.\d+", audit.splitlines()[0])
    if not m:
        print("DRIFT: AUDIT.md title line carries no vX.Y.Z version")
        sys.exit(1)
    ver = m.group(0)            # e.g. v1.2.0
    bare = ver.lstrip("v")      # e.g. 1.2.0

    # --- version surfaces -------------------------------------------------
    pkg = json.loads((ROOT / "package.json").read_text(encoding="utf-8"))
    if pkg.get("version") != bare:
        problems.append(f"package.json version {pkg.get('version')} != {bare}")

    lock = ROOT / "package-lock.json"
    if lock.exists():
        lv = json.loads(lock.read_text(encoding="utf-8")).get("version")
        if lv != bare:
            problems.append(f"package-lock.json version {lv} != {bare}")

    pyproject = ROOT / "pyproject.toml"
    if pyproject.exists():
        pm = re.search(r'(?m)^version\s*=\s*"([^"]+)"', pyproject.read_text(encoding="utf-8"))
        if not pm or pm.group(1) != bare:
            problems.append(f"pyproject.toml version {pm.group(1) if pm else '(missing)'} != {bare}")

    validator = (CORE / "evals" / "code_audit_validator.py").read_text(encoding="utf-8")
    vm = re.search(r"CURRENT_PROTOCOL\s*=\s*\((\d+),\s*(\d+),\s*(\d+)\)", validator)
    if not vm or "v" + ".".join(vm.groups()) != ver:
        problems.append(f"code_audit_validator.py CURRENT_PROTOCOL {'v' + '.'.join(vm.groups()) if vm else '(missing)'} != {ver} — version-aware template gating out of lockstep")

    cff = ROOT / "CITATION.cff"
    if cff.exists():
        cm = re.search(r"(?m)^version:\s*(\S+)", cff.read_text(encoding="utf-8"))
        if not cm or cm.group(1) != bare:
            problems.append(f"CITATION.cff version {cm.group(1) if cm else '(missing)'} != {bare}")

    baseline = json.loads((CORE / "evals" / "baseline.json").read_text(encoding="utf-8"))
    if baseline.get("audit_md_version") != ver:
        problems.append(f"baseline.json audit_md_version {baseline.get('audit_md_version')} != {ver}")
    if not baseline.get("all_ok"):
        problems.append("baseline.json all_ok is false — do not ship a red baseline")

    ch = re.search(r"(?m)^## (v\d+\.\d+\.\d+)", (CORE / "CHANGELOG.md").read_text(encoding="utf-8"))
    if not ch or ch.group(1) != ver:
        problems.append(f"CHANGELOG.md newest entry {ch.group(1) if ch else '(none)'} != {ver}")

    if not (CORE / "improve" / "versions" / f"AUDIT-{ver}.md").exists():
        problems.append(f"core/improve/versions/AUDIT-{ver}.md missing — release was not snapshotted")

    readme = (ROOT / "README.md").read_text(encoding="utf-8")
    if f"current release **{ver}**" not in readme:
        problems.append(f"README.md lacks 'current release **{ver}**' marker")

    html = (SITE / "index.html").read_text(encoding="utf-8")
    for marker in (f"{ver} · evals", f"AUDIT.md {ver}"):
        if marker not in html:
            problems.append(f"index.html lacks current-version marker '{marker}'")

    # --- fixture-count surfaces -------------------------------------------
    n_disk = sum(1 for d in (CORE / "evals" / "fixtures").iterdir() if d.is_dir())
    if baseline.get("fixtures") != n_disk:
        problems.append(f"baseline.json fixtures {baseline.get('fixtures')} != {n_disk} on disk")
    count_re = re.compile(r"(\d+)\s*/\s*(\d+)\s*(?:fixtures|green|OK)|(\d+) of (\d+) fixtures|(\d+) fixtures")
    for name in ("README.md", "site/index.html", "core/evals/README.md"):
        text = (ROOT / name).read_text(encoding="utf-8")
        for mt in count_re.finditer(text):
            nums = [int(x) for x in mt.groups() if x]
            for x in nums:
                if x != n_disk:
                    problems.append(f"{name}: stale fixture count '{mt.group(0).strip()}' (suite has {n_disk})")

    # --- verdict -----------------------------------------------------------
    if problems:
        for p in problems:
            print(f"DRIFT: {p}")
        print(f"\n{len(problems)} doc-sync problem(s) — fix in the same commit as the change that caused them (AGENTS.md).")
        sys.exit(1)
    print(f"docs in sync — {ver}, {n_disk} fixtures, all surfaces agree")
    sys.exit(0)


if __name__ == "__main__":
    main()
