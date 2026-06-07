"""run_parity.py — Cross-validate Python script path vs Rust host run path.

For each skill that has BOTH:
  - tests/fixtures.json (input corpus)
  - a primary script in PRIMARY_SCRIPT
invoke both paths with the same fixture input, normalise outputs (sort
JSON keys, strip whitespace), compare. Report per-skill pass/fail.

Run: python modules/skill/tests/parity/run_parity.py
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


MODULE_ROOT = Path(__file__).resolve().parents[2]
PUBLIC_SKILL_ROOT = MODULE_ROOT / "public"


def canonical(s: str) -> str:
    """Parse-then-reserialise so trailing whitespace + key order don't matter."""
    s = s.strip()
    if not s:
        return ""
    try:
        obj = json.loads(s)
        return json.dumps(obj, sort_keys=True, ensure_ascii=False)
    except json.JSONDecodeError:
        return s  # XML or other — compare verbatim


def invoke_script(script_path: Path, input_data: str) -> tuple[int, str]:
    out = subprocess.run(
        [sys.executable, str(script_path)],
        input=input_data, text=True, capture_output=True, timeout=10,
    )
    return out.returncode, out.stdout


def invoke_rust_host(skill_name: str, input_data: str, cwd: Path) -> tuple[int, str]:
    # Prefer the prebuilt release binary for speed; fall back to `cargo run`.
    release_bin = cwd / "target" / "release" / "cyberos-skill"
    debug_bin = cwd / "target" / "debug" / "cyberos-skill"
    if release_bin.is_file():
        argv = [
            str(release_bin),
            "--root",
            str(PUBLIC_SKILL_ROOT),
            "run",
            skill_name,
            "--executor",
            "script",
        ]
    elif debug_bin.is_file():
        argv = [
            str(debug_bin),
            "--root",
            str(PUBLIC_SKILL_ROOT),
            "run",
            skill_name,
            "--executor",
            "script",
        ]
    else:
        argv = [
            "cargo",
            "run",
            "-q",
            "-p",
            "cyberos-skill-cli",
            "--",
            "--root",
            str(PUBLIC_SKILL_ROOT),
            "run",
            skill_name,
            "--executor",
            "script",
        ]
    out = subprocess.run(
        argv, input=input_data, text=True, capture_output=True, timeout=60,
        cwd=str(cwd),
    )
    return out.returncode, out.stdout


def main() -> int:
    fixtures_seen = 0
    parity_pass = 0
    parity_fail = 0
    skills_seen = 0

    if not PUBLIC_SKILL_ROOT.is_dir():
        print(f"Missing public skill root: {PUBLIC_SKILL_ROOT}", file=sys.stderr)
        return 2

    for skill_md in PUBLIC_SKILL_ROOT.rglob("SKILL.md"):
        sd = skill_md.parent
        name = sd.name
        fp = sd / "tests" / "fixtures.json"
        if not fp.is_file():
            continue
        from_runner_map = {
            "vietnam-mst-validate": "scripts/validate_mst.py",
            "vietnam-vat-invoice": "scripts/generate_invoice.py",
            "vietnam-bank-transfer": "scripts/generate_qr.py",
            "vietnam-vneid-integration": "scripts/validate_cccd.py",
        }
        if name not in from_runner_map:
            continue
        script_path = sd / from_runner_map[name]
        if not script_path.is_file():
            continue
        skills_seen += 1
        fixtures = json.loads(fp.read_text(encoding="utf-8"))

        print(f"\n=== {name} ===")
        cases = fixtures.get("valid", []) + fixtures.get("invalid", [])
        for case in cases:
            inp = case.get("input")
            if isinstance(inp, (dict, list)):
                inp = json.dumps(inp)
            elif inp is None:
                # Whole-case payload — strip auxiliary fields starting with '_'.
                payload = {k: v for k, v in case.items() if not k.startswith("_")}
                inp = json.dumps(payload, ensure_ascii=False)
            fixtures_seen += 1
            r_code, r_out = invoke_script(script_path, str(inp))
            h_code, h_out = invoke_rust_host(name, str(inp), MODULE_ROOT)
            if r_code == h_code and canonical(r_out) == canonical(h_out):
                parity_pass += 1
                print(f"  OK    {str(inp)[:40]:<40} -> exit={r_code}, equal outputs")
            else:
                parity_fail += 1
                print(f"  FAIL  {str(inp)[:40]:<40}")
                print(f"        script: exit={r_code} out={r_out[:80]!r}")
                print(f"        host:   exit={h_code} out={h_out[:80]!r}")

    print(f"\nTotal: {parity_pass}/{fixtures_seen} parity-match")
    if skills_seen == 0 or fixtures_seen == 0:
        print("No script-backed public skill fixtures were exercised.", file=sys.stderr)
        return 2
    return 0 if parity_fail == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
