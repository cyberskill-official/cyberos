"""Router scoring + fixture tests."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from cuo.core.catalog import discover
from cuo.core.router import RoutingDecision, route

REPO = Path(__file__).resolve().parents[2]
SKILLS_ROOT = REPO / "skill" / "skills"
FIXTURES = Path(__file__).parent / "fixtures" / "routing-cases.json"


@pytest.fixture(scope="module")
def catalog():
    return discover(SKILLS_ROOT)


@pytest.fixture(scope="module")
def cases():
    return json.loads(FIXTURES.read_text(encoding="utf-8"))


def test_route_returns_none_for_no_match(catalog):
    out = route("How's the weather today?", catalog)
    assert out is None


def test_route_returns_decision_for_mst_query(catalog):
    out = route("Validate MST 0312345678", catalog)
    assert out is not None
    assert isinstance(out, RoutingDecision)
    assert out.skill_name == "vn-mst-validate"
    assert out.arguments.get("input") == "0312345678"
    assert 0.0 < out.confidence <= 1.0


def test_route_handles_vietnamese_diacritics(catalog):
    out = route("Kiểm tra mã số thuế 0312345678", catalog)
    assert out is not None
    assert out.skill_name == "vn-mst-validate"


def test_route_fixtures_all_pass(catalog, cases):
    failures: list[str] = []
    for case in cases:
        out = route(case["query"], catalog)
        actual = out.skill_name if out else None
        expected = case["expect_skill"]
        if actual != expected:
            failures.append(f"[{case['name']}] expected {expected!r}, got {actual!r}")
            continue
        if out and "expect_args" in case:
            for key, expected_value in case["expect_args"].items():
                actual_value = out.arguments.get(key)
                if actual_value != expected_value:
                    failures.append(
                        f"[{case['name']}] arg {key!r}: expected {expected_value!r}, "
                        f"got {actual_value!r}"
                    )
    assert not failures, "\n".join(failures)


def test_route_includes_alternatives(catalog):
    # A query that hits multiple skills should surface alternatives.
    out = route("validate the tax invoice MST", catalog)
    assert out is not None
    # alternative_skills is a list, may be empty if only one skill scored.
    assert isinstance(out.alternative_skills, list)
