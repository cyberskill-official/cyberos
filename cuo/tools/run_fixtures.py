#!/usr/bin/env python3
"""Routing-fixture parity harness.

Walks `tests/fixtures/routing-cases.json`, runs each query through
`route()`, asserts the expected skill_name and (where provided)
expected_args. Prints PASS/FAIL per case and a summary.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO_ROOT = HERE.parent  # cuo/ project root
sys.path.insert(0, str(REPO_ROOT))

from cuo.core.catalog import discover                       # noqa: E402
from cuo.core.router import route                            # noqa: E402

SKILLS_ROOT = REPO_ROOT.parent / "skill" / "skills"
FIXTURES = REPO_ROOT / "tests" / "fixtures" / "routing-cases.json"


def main() -> int:
    if not SKILLS_ROOT.exists():
        print(f"FATAL: skill root not found: {SKILLS_ROOT}", file=sys.stderr)
        return 2
    catalog = discover(SKILLS_ROOT)
    cases = json.loads(FIXTURES.read_text(encoding="utf-8"))

    passed = 0
    failed = 0
    for case in cases:
        name = case["name"]
        query = case["query"]
        expected = case.get("expect_skill")
        expected_args = case.get("expect_args", {})

        out = route(query, catalog)
        actual = out.skill_name if out else None

        if actual != expected:
            print(f"FAIL  {name}: expected={expected!r} actual={actual!r}")
            failed += 1
            continue

        # If expected is None and actual is None, it's a pass.
        if out is None:
            print(f"PASS  {name}: (correctly unrouted)")
            passed += 1
            continue

        # Check expected_args.
        arg_failures = []
        for key, expected_value in expected_args.items():
            actual_value = out.arguments.get(key)
            if actual_value != expected_value:
                arg_failures.append(f"{key}: want={expected_value!r} got={actual_value!r}")

        if arg_failures:
            print(f"FAIL  {name}: routed to {actual!r} but args mismatch: {arg_failures}")
            failed += 1
        else:
            print(
                f"PASS  {name}: → {actual} "
                f"(conf={out.confidence:.2f}; rationale={out.rationale})"
            )
            passed += 1

    total = passed + failed
    print()
    print(f"Summary: {passed}/{total} passed, {failed} failed")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
