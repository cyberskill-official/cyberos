"""TASK-CUO-301 regression tests — the ship queue must see the real backlog.

REGRESSION-002: `test_parses_live_backlog` MUST fail at HEAD~1 and pass at HEAD.
Before the fix, `parse_backlog` matched a markdown table against a bullet file and
returned 0 rows — silently, because "no regex matched any line" is indistinguishable
from "the backlog is empty".

These tests deliberately assert against the LIVE backlog rather than a fixture. A
fixture would have kept passing through the entire outage, which is precisely how the
bug survived: the table-shaped fixture in test_rework_mode.py was green the whole time.
"""
from __future__ import annotations

import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "modules" / "cuo"))

from cuo.core.backlog_reader import parse_backlog, parse_specs, next_eligible  # noqa: E402

BACKLOG = ROOT / "docs" / "tasks" / "BACKLOG.md"
TASKS_ROOT = ROOT / "docs" / "tasks"


@pytest.mark.skipif(not BACKLOG.exists(), reason="no backlog in this checkout")
def test_parses_live_backlog():
    """THE regression test cited by TASK-CUO-301.

    Red at HEAD~1: table regex vs bullet file -> 0 rows.
    Green at HEAD: spec-frontmatter fallback -> one row per spec.
    """
    rows = parse_backlog(BACKLOG)
    specs = list(TASKS_ROOT.glob("*/TASK-*/spec.md"))
    assert len(rows) > 0, (
        "parse_backlog returned 0 rows against the live backlog. This is the "
        "TASK-CUO-301 failure mode: an empty parse is not an error, so the ship "
        "queue reports 'no eligible task' forever."
    )
    assert len(rows) == len(specs), (
        f"queue sees {len(rows)} tasks but {len(specs)} spec.md files exist on disk"
    )


@pytest.mark.skipif(not BACKLOG.exists(), reason="no backlog in this checkout")
def test_queue_is_not_dead():
    """next_eligible must be able to pick something. A permanently-None queue is the bug."""
    rows = parse_backlog(BACKLOG)
    active = [r for r in rows if r.status in (
        "ready_to_implement", "implementing", "ready_to_review",
        "reviewing", "ready_to_test", "testing")]
    if not active:
        pytest.skip("no active tasks in this backlog — nothing to pick")
    assert next_eligible(rows) is not None, "queue has active tasks but picks none"


def test_strips_yaml_comment(tmp_path: Path):
    """A trailing YAML comment must not become part of the value.

    Real spec found in the wild:
        status: on_hold   # was "blocked" (not a valid status per STATUS-REFERENCE §1)
    Without stripping, `status` is the whole line and matches no enum member, so the
    task silently drops out of every status filter.
    """
    d = tmp_path / "mod" / "TASK-MOD-001-x"
    d.mkdir(parents=True)
    (d / "spec.md").write_text(
        '---\n'
        'id: TASK-MOD-001\n'
        'title: x\n'
        'status: on_hold   # was "blocked" (not a valid status)\n'
        'priority: p0\n'
        'depends_on: []\n'
        '---\nbody\n', encoding="utf-8")
    rows = parse_specs(tmp_path)
    assert len(rows) == 1
    assert rows[0].status == "on_hold", f"comment leaked into status: {rows[0].status!r}"


def test_skips_dir_without_spec(tmp_path: Path):
    (tmp_path / "mod" / "TASK-MOD-002-empty").mkdir(parents=True)
    assert parse_specs(tmp_path) == []


def test_table_mode_still_works(tmp_path: Path):
    """Back-compat: a genuinely table-shaped backlog still parses in table mode."""
    b = tmp_path / "BACKLOG.md"
    b.write_text(
        "| TASK-ID | Title | Pri | Status | Depends on | Effort |\n"
        "| --- | --- | --- | --- | --- | --- |\n"
        "| TASK-AAA-001 | t | p0 | ready_to_implement |  | 1 |\n", encoding="utf-8")
    rows = parse_backlog(b)
    assert len(rows) == 1
    assert rows[0].task_id == "TASK-AAA-001"
    assert rows[0].line_number == 3, "table mode must keep the line number for the applier"
