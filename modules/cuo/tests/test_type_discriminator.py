"""Type-discriminator wiring tests — 2026-07-14.

The dispatch is DATA: `task-author` loads `contracts/task/templates/{type}.md` and
HALTS if it is missing, rather than falling back to `feature`. That halt is
deliberate — a bug rendered as a feature has no reproduction, no root cause and no
regression test, and it sails through a gate that never knew to ask.

The cost of a loud halt is that the FM-108 enum and the templates directory MUST
agree. They did not, ten minutes after I wrote the rule: the enum admitted four types
and only two had templates, while 215 live tasks carried `type: improvement`. Every
one of them would have halted the author.

These tests are what stop that recurring.
"""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CONTRACT = ROOT / "modules" / "skill" / "contracts" / "task"
RUBRIC = ROOT / "modules" / "skill" / "task-audit" / "RUBRIC.md"
TEMPLATES = CONTRACT / "templates"
RUBRICS = CONTRACT / "rubrics"


def _enum_types() -> set[str]:
    """The `type` values FM-108 admits, read from the rubric row itself."""
    row = next((l for l in RUBRIC.read_text(encoding="utf-8").splitlines()
                if l.startswith("| `FM-108`")), "")
    return set(re.findall(r"`(feature|bug|improvement|chore)`", row))


def test_every_enum_type_has_a_template():
    """FM-108's enum and templates/ must agree, or task-author halts on a legal type."""
    types = _enum_types()
    assert types, "FM-108 row not found or admits no types — the enum is the source of truth"
    missing = {t for t in types if not (TEMPLATES / f"{t}.md").is_file()}
    assert not missing, (
        f"FM-108 admits {sorted(types)} but templates/ is missing {sorted(missing)}. "
        "task-author HALTS on a missing template (by design), so every enum member "
        "needs a file — even if that file only points at feature.md."
    )


def test_no_orphan_templates():
    """The reverse: a template for a type the enum rejects is dead weight."""
    types = _enum_types()
    on_disk = {p.stem for p in TEMPLATES.glob("*.md")}
    orphans = on_disk - types
    assert not orphans, (
        f"templates/ carries {sorted(orphans)} which FM-108 does not admit. "
        "Either add them to the enum or delete them."
    )


def test_bug_rubric_exists_and_is_loaded():
    """type: bug is the only type with an extra family today. It must be reachable."""
    assert (RUBRICS / "bug.md").is_file()
    assert (RUBRICS / "common.md").is_file(), "the composition contract must exist"
    audit_skill = (ROOT / "modules" / "skill" / "task-audit" / "SKILL.md").read_text(encoding="utf-8")
    assert "rubrics/{type}.md" in audit_skill or "rubrics/common" in audit_skill, (
        "task-audit does not compose per-type rubrics — the BUG-* family never fires"
    )


def test_absent_type_rubric_is_not_an_error():
    """feature/improvement/chore have no extra family. That is correct, not a gap.

    Pins rubrics/common.md §3: `rubrics/<type>.md` is OPTIONAL. If someone later
    'fixes' this by creating empty rubric files, that is noise, and this test says so.
    """
    for t in ("feature", "improvement", "chore"):
        p = RUBRICS / f"{t}.md"
        if p.is_file():
            body = p.read_text(encoding="utf-8").strip()
            assert len(body) > 200, (
                f"rubrics/{t}.md exists but is a stub. An absent per-type rubric "
                "already means 'common families only' — an empty file adds nothing."
            )


def test_regression_gate_is_wired_not_just_documented():
    """coverage-gate-author must RUN the proof, not describe it.

    REGRESSION-002 was verified by hand during Stage 3 and the gate did not implement
    it. A rule that only a human ever runs is a convention, not a gate.
    """
    gate = (ROOT / "modules" / "skill" / "coverage-gate-author" / "SKILL.md").read_text(encoding="utf-8")
    assert "red_at_broken" in gate, "coverage-gate@1 has no regression block"
    assert "worktree add" in gate, "no worktree checkout — REGRESSION-002 cannot be proven"
    assert "raw_terminal_red" in gate, "REGRESSION-003 needs both terminals captured"


def test_live_specs_only_use_enum_types():
    """No spec may carry a type the enum rejects."""
    types = _enum_types()
    bad: dict[str, str] = {}
    for spec in (ROOT / "docs" / "tasks").glob("*/TASK-*/spec.md"):
        m = re.search(r"^type:\s*(\S+)", spec.read_text(encoding="utf-8"), re.M)
        if m and m.group(1) not in types:
            bad[spec.parent.name] = m.group(1)
    assert not bad, f"specs carry types outside the FM-108 enum: {bad}"
