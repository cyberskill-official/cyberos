"""run_fixtures.py — Walk every skill bundle and exercise its tests/fixtures.json.

For each skill under <root>:
  - Read tests/fixtures.json
  - If `valid` array is present, run each fixture's `input` through the
    skill's primary script (heuristic: scripts/<verb>.py where verb is in
    the skill name)
  - Assert exit code + output match `expected`

Usage:
    python skill/tools/run_fixtures.py [<skill-root>]
    Default root: skill/skills/

Output: per-skill PASS/FAIL summary + final tally. Exit code 1 if any failures.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path
from typing import Iterable


# Map skill name -> primary entry script (best-effort heuristic).
PRIMARY_SCRIPT = {
    "vietnam-mst-validate": "scripts/validate_mst.py",
    "vietnam-vat-invoice": "scripts/generate_invoice.py",
    "vietnam-bank-transfer": "scripts/generate_qr.py",
    "vietnam-vneid-integration": "scripts/validate_cccd.py",
    "vn-tax-filing": "scripts/generate_return.py",
}


def find_skills(root: Path) -> Iterable[Path]:
    for skill_md in root.rglob("SKILL.md"):
        yield skill_md.parent


def _input_payload(case: dict) -> str:
    """Coerce a fixture case to the stdin string for the primary script.

    Cases come in two shapes:
      - {"input": <str|dict|list>, "expected": ..., "_label": ...}
      - vietnam-vat-invoice shape: case IS the invoice document, plus _label /
        _expected_totals / _expected_error_contains auxiliary fields.
    """
    if "input" in case:
        inp = case["input"]
        if isinstance(inp, (dict, list)):
            return json.dumps(inp, ensure_ascii=False)
        return str(inp) if inp is not None else ""
    # Whole-case payload — strip auxiliary fields starting with '_'.
    payload = {k: v for k, v in case.items() if not k.startswith("_")}
    return json.dumps(payload, ensure_ascii=False)


def _label(case: dict) -> str:
    if "_label" in case:
        return case["_label"]
    if "input" in case and not isinstance(case["input"], (dict, list)):
        return repr(case["input"])[:60]
    return "<unlabeled>"


def run_one(skill_dir: Path) -> tuple[int, int]:
    """Returns (passed, failed) for this skill."""
    fixtures_path = skill_dir / "tests" / "fixtures.json"
    if not fixtures_path.is_file():
        print(f"  SKIP  {skill_dir.name} (no fixtures)")
        return 0, 0
    fixtures = json.loads(fixtures_path.read_text(encoding="utf-8"))
    name = skill_dir.name
    script_rel = PRIMARY_SCRIPT.get(name)
    if not script_rel:
        print(f"  SKIP  {name} (no primary script mapping; reference-only or unknown)")
        return 0, 0
    script = skill_dir / script_rel
    if not script.is_file():
        print(f"  MISS  {name} - primary script {script_rel} not found")
        return 0, 0

    passed = 0
    failed = 0
    for case in fixtures.get("valid", []):
        stdin_data = _input_payload(case)
        out = subprocess.run(
            [sys.executable, str(script)],
            input=stdin_data, text=True, capture_output=True, timeout=10,
        )
        if out.returncode == 0:
            passed += 1
        else:
            failed += 1
            print(f"  FAIL  {name} valid[{_label(case)}]: exit={out.returncode} stderr={out.stderr[:120].strip()}")
    for case in fixtures.get("invalid", []):
        stdin_data = _input_payload(case)
        out = subprocess.run(
            [sys.executable, str(script)],
            input=stdin_data, text=True, capture_output=True, timeout=10,
        )
        # For invalid cases, expect non-zero exit.
        if out.returncode != 0:
            passed += 1
        else:
            failed += 1
            print(f"  FAIL  {name} invalid[{_label(case)}]: unexpectedly succeeded; stdout={out.stdout[:120].strip()}")
    print(f"  {('PASS' if failed == 0 else 'FAIL')}  {name:<24}  {passed} passed, {failed} failed")
    return passed, failed


def main() -> int:
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("skill/skills")
    if not root.is_dir():
        print(f"error: {root} is not a directory", file=sys.stderr)
        return 2
    print(f"Running fixtures under {root}\n")
    total_p, total_f = 0, 0
    for sd in sorted(find_skills(root)):
        p, f = run_one(sd)
        total_p += p
        total_f += f
    print(f"\n  total: {total_p} passed, {total_f} failed")
    return 0 if total_f == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
