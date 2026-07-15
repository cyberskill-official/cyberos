"""Tests for applier path resolution — verifies output_dir-based file discovery.

Ensures that appliers find task files and resolve artifact paths by walking up
from the output directory (where step JSON files live), not just from
_cyberos_root which may point to a different project.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from cuo.core.applier import _find_task_file, _find_repo_root, _resolve_artifact_path


@pytest.fixture()
def mock_project(tmp_path: Path):
    """Create a minimal project directory tree with a task file.

    Structure:
        project/
        ├── docs/
        │   └── tasks/
        │       └── auth/
        │           └── TASK-TEST-001-test-feature.md
        └── outputs/
            └── step03_architecture-decision-record-author.json
    """
    project = tmp_path / "my-project"
    task_dir = project / "docs" / "tasks" / "auth"
    task_dir.mkdir(parents=True)
    task_file = task_dir / "TASK-TEST-001-test-feature.md"
    task_file.write_text("# TASK-TEST-001\n\nA test task.\n")

    output_dir = project / "outputs"
    output_dir.mkdir()
    output_file = output_dir / "step03_architecture-decision-record-author.json"
    output_file.write_text(json.dumps({"skill": "architecture-decision-record-author"}))

    return {
        "project": project,
        "task_file": task_file,
        "output_dir": output_dir,
        "output_file": output_file,
    }


class TestFindTaskFile:
    """_find_fr_file should discover task files via output_dir walk-up."""

    def test_finds_task_from_output_dir(self, mock_project):
        """When output_dir is inside the project, walk-up finds the task."""
        result = _find_task_file(
            "TASK-TEST-001",
            repo_root=None,
            output_dir=mock_project["output_dir"],
        )
        assert result is not None
        assert result == mock_project["task_file"]

    def test_finds_task_from_nested_output_dir(self, tmp_path):
        """Even deeply nested output dirs should walk up to find docs/."""
        project = tmp_path / "deep-project"
        task_dir = project / "docs" / "tasks" / "auth"
        task_dir.mkdir(parents=True)
        task_file = task_dir / "TASK-DEEP-001-deep-feature.md"
        task_file.write_text("# TASK-DEEP-001\n")

        # Output dir is 3 levels deep under project
        deep_output = project / "runs" / "2026-05-23" / "attempt-1"
        deep_output.mkdir(parents=True)
        (deep_output / "step01.json").write_text("{}")

        result = _find_task_file("TASK-DEEP-001", output_dir=deep_output)
        assert result is not None
        assert result == task_file

    def test_repo_root_used_when_output_dir_missing_docs(self, mock_project):
        """When output_dir has no docs/ ancestor, falls back to repo_root."""
        # Create a separate cyberos-like root with its own docs/
        other_root = mock_project["project"].parent / "cyberos"
        (other_root / "docs" / "tasks").mkdir(parents=True)

        # The task is in mock_project, not in other_root
        result = _find_task_file(
            "TASK-TEST-001",
            repo_root=other_root,
            output_dir=mock_project["output_dir"],
        )
        # Should find via output_dir walk-up, NOT from other_root
        assert result is not None
        assert result == mock_project["task_file"]

    def test_returns_none_when_task_not_found(self, tmp_path):
        """Returns None when the task doesn't exist in any search root."""
        empty_project = tmp_path / "empty"
        (empty_project / "docs" / "tasks").mkdir(parents=True)
        output_dir = empty_project / "outputs"
        output_dir.mkdir()

        result = _find_task_file("TASK-NONEXISTENT-999", output_dir=output_dir)
        assert result is None


class TestFindRepoRoot:
    """_find_repo_root should discover the project root via output_dir."""

    def test_finds_root_from_output_dir(self, mock_project):
        result = _find_repo_root({}, output_dir=mock_project["output_dir"])
        assert result is not None
        assert result == mock_project["project"]

    def test_output_dir_takes_precedence_over_cyberos_root(self, mock_project):
        """output_dir walk-up should find the actual project, not _cyberos_root."""
        fake_cyberos = mock_project["project"].parent / "fake-cyberos"
        (fake_cyberos / "docs").mkdir(parents=True)

        hand_off = {"_cyberos_root": str(fake_cyberos)}
        result = _find_repo_root(hand_off, output_dir=mock_project["output_dir"])
        # Should find mock_project via output_dir, not fake_cyberos
        assert result == mock_project["project"]


class TestResolveArtifactPath:
    """_resolve_artifact_path should place artifacts next to task files."""

    def test_sibling_path_when_task_found(self, mock_project):
        """When the task file is found, artifact goes next to it."""
        result = _resolve_artifact_path(
            output={},
            task_id="TASK-TEST-001",
            hand_off={},
            filename_prefix="impl-plan",
            default_dir="docs/tasks",
            output_dir=mock_project["output_dir"],
        )
        assert result is not None
        assert result.parent == mock_project["task_file"].parent
        assert result.name == "impl-plan-TASK-TEST-001.md"

    def test_fallback_to_default_dir(self, tmp_path):
        """When task is not found, falls back to <repo_root>/<default_dir>/."""
        project = tmp_path / "fallback-project"
        (project / "docs").mkdir(parents=True)
        output_dir = project / "outputs"
        output_dir.mkdir()

        result = _resolve_artifact_path(
            output={},
            task_id="TASK-MISSING-999",
            hand_off={},
            filename_prefix="code-review",
            default_dir="docs/tasks",
            output_dir=output_dir,
        )
        assert result is not None
        assert result.parent == project / "docs" / "tasks"
        assert "code-review" in result.name
