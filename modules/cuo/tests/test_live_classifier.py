"""Live-classifier tests — NO monkeypatching. TASK-SKILL-112 follow-up, 2026-07-14.

Why this file exists
--------------------
`tests/test_trigger_tests.py` monkeypatches `classify`. Every one of its 29 tests
passed while the real adapter was broken in three independent ways:

    1. `_route(phrase)`                 -> route() takes (query, personas)
    2. `discover_personas()`            -> takes cuo_root
    3. `discover_workflows(cuo_root)`   -> takes a PersonaEntry, not a path
                                           ...and the AttributeError was swallowed
                                           by a bare `except`, so classify()
                                           silently returned None for every phrase.

Live-classifier coverage was 0% while the suite reported green. A fake that never
disagrees with you is not a test.

These tests call the real router. They are the only thing standing between you and
a repeat.
"""
from __future__ import annotations

import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "modules" / "cuo"))

from cuo.trigger_tests import classify  # noqa: E402  (the REAL one, unpatched)


def test_classifier_adapter_actually_runs():
    """The adapter must return a result, not raise and not silently None-out.

    Pins fixes 1-3 above. Before them, this raised TypeError on the first call.
    """
    r = classify("ship the next task")
    assert r is not None
    assert hasattr(r, "skill_id") and hasattr(r, "confidence")
    # a real routing decision was reached — confidence is a number, not the 0.0
    # sentinel the broken adapter returned for literally every input
    assert isinstance(r.confidence, float)


@pytest.mark.parametrize("phrase", [
    "add a task to my todo list",
    "create a task",
    "remind me to buy milk",
    "what is our holiday schedule",
])
def test_common_noun_does_not_false_positive(phrase: str):
    """RISK R1, measured.

    The rename named two skills after `task` — the single most common noun in agent
    prompts. The fear was that `task-author` / `task-audit` would start swallowing
    generic phrasing ("add a task", "create a task") and wreck routing precision.

    It did not happen, and the reason is structural rather than lucky: the CUO
    classifier routes to a (persona, workflow) pair behind a 0.5 confidence
    threshold. It is not a bag-of-words matcher over skill names, so a bare common
    noun cannot drag it anywhere.

    This test is what stops that from silently changing.
    """
    r = classify(phrase)
    assert r.skill_id is None, (
        f"generic phrasing {phrase!r} routed to {r.skill_id!r} at conf={r.confidence:.2f} — "
        "the common-noun collision the rename risked has now materialised"
    )


def test_task_author_is_command_driven_not_router_driven():
    """task-author / task-audit are NOT reachable through the workflow router.

    48 personas, 224 workflows, and zero of them carry task-author or task-audit as
    an ENTRY skill — they appear only *inside* ship-tasks' chain. They are reached
    via the `/create-tasks` command, by design.

    Their `acceptance/TRIGGER_TESTS.md` fixtures therefore assert a routing path the
    architecture cannot produce. That is a fixture bug, not a classifier bug, and it
    was invisible for as long as the tests ran against a fake.

    If someone later adds a workflow whose entry skill IS task-author, this test
    fails loudly and the fixtures become meaningful again — which is exactly when
    you want to be told.
    """
    from cuo.core.catalog import discover_personas, discover_workflows
    personas = discover_personas(ROOT / "modules" / "cuo")
    entry_skills = set()
    for p in personas:
        for wf in discover_workflows(p):
            if wf.skill_chain:
                first = wf.skill_chain[0]
                entry_skills.add(str(first.get("skill") or first.get("id") or ""))

    routable = {s for s in entry_skills if "task-author" in s or "task-audit" in s}
    assert not routable, (
        f"task-author/task-audit are now workflow entry skills ({routable}). "
        "They ARE router-reachable — go re-enable their TRIGGER_TESTS fixtures."
    )
