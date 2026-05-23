"""Tests for CUO Rework Mode and status-aware phase restart logic."""

from __future__ import annotations

import json
from pathlib import Path
import pytest

from cuo.core.backlog_reader import FrRow, parse_backlog, next_eligible, list_eligible
from cuo.core.catalog import discover_personas
from tests.conftest import FakeInvoker
from cuo.core.supervisor import execute_chain


@pytest.fixture
def mock_backlog_file(tmp_path: Path) -> Path:
    backlog_content = """# Feature Request Backlog

| FR-ID | Title | Pri | Status | Depends on | Effort |
| :--- | :--- | :--- | :--- | :--- | :--- |
| FR-CUO-101 | Implement backlog reader | High | done | | 3 |
| FR-CUO-102 | Add rework mode | Medium | ready_to_implement | FR-CUO-101 | 5 |
| FR-CUO-103 | Legacy status update | Low | ready_to_review | FR-CUO-101 | 2 |
| FR-CUO-104 | Re-done testing | Low | done | FR-CUO-101 | 2 |
"""
    file_path = tmp_path / "BACKLOG.md"
    file_path.write_text(backlog_content, encoding="utf-8")
    return file_path


def test_parse_backlog_rows(mock_backlog_file: Path) -> None:
    rows = parse_backlog(mock_backlog_file)
    assert len(rows) == 4
    assert rows[0].fr_id == "FR-CUO-101"
    assert rows[0].status == "done"
    assert rows[1].fr_id == "FR-CUO-102"
    assert rows[1].status == "ready_to_implement"
    assert rows[1].deps == ["FR-CUO-101"]


def test_next_eligible_no_rework(mock_backlog_file: Path) -> None:
    rows = parse_backlog(mock_backlog_file)
    # FR-CUO-101 is done, so FR-CUO-102 is eligible
    eligible = next_eligible(rows, rework=False)
    assert eligible is not None
    assert eligible.fr_id == "FR-CUO-102"


def test_next_eligible_with_rework(mock_backlog_file: Path) -> None:
    rows = parse_backlog(mock_backlog_file)
    # With rework, "done" FRs (FR-CUO-101, FR-CUO-104) are eligible.
    # The first matching row in BACKLOG is FR-CUO-101.
    eligible = next_eligible(rows, rework=True)
    assert eligible is not None
    assert eligible.fr_id == "FR-CUO-101"


def test_list_eligible_with_rework(mock_backlog_file: Path) -> None:
    rows = parse_backlog(mock_backlog_file)
    eligible_list = list_eligible(rows, rework=True)
    eligible_ids = {r.fr_id for r in eligible_list}
    assert "FR-CUO-101" in eligible_ids
    assert "FR-CUO-102" in eligible_ids
    assert "FR-CUO-103" in eligible_ids
    assert "FR-CUO-104" in eligible_ids


def test_execute_chain_start_step_from_status(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    cuo_root = Path(__file__).resolve().parent.parent
    skill_root = cuo_root.parent / "skill"

    # Mock backlog_path.is_file() to return True for BACKLOG.md
    orig_is_file = Path.is_file
    def mock_is_file(self: Path) -> bool:
        if self.name == "BACKLOG.md":
            return True
        return orig_is_file(self)
    monkeypatch.setattr(Path, "is_file", mock_is_file)

    # Mock parse_backlog to return our target FR with status ready_to_review
    import cuo.core.backlog_reader
    monkeypatch.setattr(
        cuo.core.backlog_reader,
        "parse_backlog",
        lambda path: [
            FrRow(
                fr_id="FR-CUO-102",
                title="Add rework mode",
                priority="Medium",
                status="ready_to_review",
                deps=[],
                effort="5",
                line_number=1,
            )
        ]
    )

    personas = discover_personas(cuo_root)
    cto = next(p for p in personas if p.slug == "chief-technology-officer")

    result = execute_chain(
        persona=cto,
        workflow_slug="adr-quick-capture",
        skill_root=skill_root,
        output_dir=tmp_path / "out",
        inputs={"fr_id": "FR-CUO-102"},
        invoker=FakeInvoker(),
    )
    assert result.outcome == "COMPLETED"
    # Both steps (step 1 and 2) should be SKIPPED because step_num < 15.
    for r in result.step_results:
        assert r.status == "SKIPPED"
        assert "starting from step 15" in r.notes[0]


def test_smart_rework_skips_mock_outputs(tmp_path: Path) -> None:
    """In rework mode, MOCKED outputs on disk are regenerated (not reused)."""
    cuo_root = Path(__file__).resolve().parent.parent
    skill_root = cuo_root.parent / "skill"

    personas = discover_personas(cuo_root)
    cto = next(p for p in personas if p.slug == "chief-technology-officer")

    output_dir = tmp_path / "out"
    output_dir.mkdir()

    # Pre-populate a MOCKED output file for step 1 of adr-quick-capture's first skill.
    # The adr-quick-capture workflow uses: step 1 = architecture-decision-record-author.
    mock_output = {"skill": "architecture-decision-record-author", "synthetic": True}
    step_file = output_dir / "step01_architecture-decision-record-author.json"
    step_file.write_text(json.dumps(mock_output), encoding="utf-8")

    # Run WITHOUT rework → should reuse the mock output
    result_normal = execute_chain(
        persona=cto,
        workflow_slug="adr-quick-capture",
        skill_root=skill_root,
        output_dir=output_dir,
        inputs={},
        invoker=FakeInvoker(),
    )
    reused_step = next(s for s in result_normal.step_results if s.step == 1)
    assert reused_step.status == "MOCKED"
    assert reused_step.notes == ["Reused existing deliverable from disk (resume/rework)"]

    # Run WITH rework → should NOT reuse mock, should regenerate
    # (FakeInvoker will create a new MOCKED output, but it won't be "reused")
    result_rework = execute_chain(
        persona=cto,
        workflow_slug="adr-quick-capture",
        skill_root=skill_root,
        output_dir=output_dir,
        inputs={"rework": True},
        invoker=FakeInvoker(),
    )
    regenerated_step = next(s for s in result_rework.step_results if s.step == 1)
    assert regenerated_step.status == "MOCKED"
    # The note should NOT say "Reused" — it was freshly invoked
    assert "Reused existing deliverable" not in " ".join(regenerated_step.notes)


def test_smart_rework_reuses_real_llm_outputs(tmp_path: Path) -> None:
    """In rework mode, real LLM outputs (OK, not synthetic) are still reused."""
    cuo_root = Path(__file__).resolve().parent.parent
    skill_root = cuo_root.parent / "skill"

    personas = discover_personas(cuo_root)
    cto = next(p for p in personas if p.slug == "chief-technology-officer")

    output_dir = tmp_path / "out"
    output_dir.mkdir()

    # Pre-populate a real (non-synthetic) output file for step 1.
    real_output = {
        "skill": "architecture-decision-record-author",
        "some_field": "real value",
    }
    step_file = output_dir / "step01_architecture-decision-record-author.json"
    step_file.write_text(json.dumps(real_output), encoding="utf-8")

    # Run WITH rework → should reuse the real output
    result = execute_chain(
        persona=cto,
        workflow_slug="adr-quick-capture",
        skill_root=skill_root,
        output_dir=output_dir,
        inputs={"rework": True},
        invoker=FakeInvoker(),
    )
    reused_step = next(s for s in result.step_results if s.step == 1)
    assert reused_step.status == "OK"
    assert reused_step.notes == ["Reused existing deliverable from disk (resume/rework)"]
