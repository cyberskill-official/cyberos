"""Tests for applier path resolution — verifies output_dir-based file discovery.

Ensures that appliers find FR files and resolve artifact paths by walking up
from the output directory (where step JSON files live), not just from
_cyberos_root which may point to a different project.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from cuo.core.applier import (
    _find_fr_file,
    _find_repo_root,
    _resolve_artifact_path,
    _resolve_cuo_artifact_dir,
)


@pytest.fixture()
def mock_project(tmp_path: Path):
    """Create a minimal project directory tree with an FR file.

    Structure:
        project/
        ├── docs/
        │   └── feature-requests/
        │       └── auth/
        │           └── FR-TEST-001-test-feature.md
        └── outputs/
            └── step03_architecture-decision-record-author.json
    """
    project = tmp_path / "my-project"
    fr_dir = project / "docs" / "feature-requests" / "auth"
    fr_dir.mkdir(parents=True)
    fr_file = fr_dir / "FR-TEST-001-test-feature.md"
    fr_file.write_text("# FR-TEST-001\n\nA test feature request.\n")

    output_dir = project / "outputs"
    output_dir.mkdir()
    output_file = output_dir / "step03_architecture-decision-record-author.json"
    output_file.write_text(json.dumps({"skill": "architecture-decision-record-author"}))

    return {
        "project": project,
        "fr_file": fr_file,
        "output_dir": output_dir,
        "output_file": output_file,
    }


class TestFindFrFile:
    """_find_fr_file should discover FR files via output_dir walk-up."""

    def test_finds_fr_from_output_dir(self, mock_project):
        """When output_dir is inside the project, walk-up finds the FR."""
        result = _find_fr_file(
            "FR-TEST-001",
            repo_root=None,
            output_dir=mock_project["output_dir"],
        )
        assert result is not None
        assert result == mock_project["fr_file"]

    def test_finds_fr_from_nested_output_dir(self, tmp_path):
        """Even deeply nested output dirs should walk up to find docs/."""
        project = tmp_path / "deep-project"
        fr_dir = project / "docs" / "feature-requests" / "auth"
        fr_dir.mkdir(parents=True)
        fr_file = fr_dir / "FR-DEEP-001-deep-feature.md"
        fr_file.write_text("# FR-DEEP-001\n")

        # Output dir is 3 levels deep under project
        deep_output = project / "runs" / "2026-05-23" / "attempt-1"
        deep_output.mkdir(parents=True)
        (deep_output / "step01.json").write_text("{}")

        result = _find_fr_file("FR-DEEP-001", output_dir=deep_output)
        assert result is not None
        assert result == fr_file

    def test_repo_root_used_when_output_dir_missing_docs(self, mock_project):
        """When output_dir has no docs/ ancestor, falls back to repo_root."""
        # Create a separate cyberos-like root with its own docs/
        other_root = mock_project["project"].parent / "cyberos"
        (other_root / "docs" / "feature-requests").mkdir(parents=True)

        # The FR is in mock_project, not in other_root
        result = _find_fr_file(
            "FR-TEST-001",
            repo_root=other_root,
            output_dir=mock_project["output_dir"],
        )
        # Should find via output_dir walk-up, NOT from other_root
        assert result is not None
        assert result == mock_project["fr_file"]

    def test_returns_none_when_fr_not_found(self, tmp_path):
        """Returns None when the FR doesn't exist in any search root."""
        empty_project = tmp_path / "empty"
        (empty_project / "docs" / "feature-requests").mkdir(parents=True)
        output_dir = empty_project / "outputs"
        output_dir.mkdir()

        result = _find_fr_file("FR-NONEXISTENT-999", output_dir=output_dir)
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
    """_resolve_artifact_path should place artifacts next to FR files."""

    def test_sibling_path_when_fr_found(self, mock_project):
        """When the FR file is found, artifact goes next to it."""
        result = _resolve_artifact_path(
            output={},
            fr_id="FR-TEST-001",
            hand_off={},
            filename_prefix="impl-plan",
            default_dir="docs/feature-requests",
            output_dir=mock_project["output_dir"],
        )
        assert result is not None
        assert result.parent == mock_project["fr_file"].parent
        assert result.name == "impl-plan-FR-TEST-001.md"

    def test_fallback_to_default_dir(self, tmp_path):
        """When FR is not found, falls back to <repo_root>/<default_dir>/."""
        project = tmp_path / "fallback-project"
        (project / "docs").mkdir(parents=True)
        output_dir = project / "outputs"
        output_dir.mkdir()

        result = _resolve_artifact_path(
            output={},
            fr_id="FR-MISSING-999",
            hand_off={},
            filename_prefix="code-review",
            default_dir="docs/feature-requests",
            output_dir=output_dir,
        )
        assert result is not None
        assert result.parent == project / "docs" / "feature-requests"
        assert "code-review" in result.name

    def test_cuo_default_artifact_dir_uses_target(self, mock_project):
        """CUO workflow artifacts should not create top-level BRAIN dirs."""
        artifact_dir = _resolve_cuo_artifact_dir(
            "impl-plans",
            hand_off={},
            output_dir=mock_project["output_dir"],
        )
        assert artifact_dir == (
            mock_project["project"]
            / "target"
            / "cuo-workflow"
            / "artifacts"
            / "impl-plans"
        )

        result = _resolve_artifact_path(
            output={},
            fr_id="FR-MISSING-999",
            hand_off={},
            filename_prefix="impl-plan",
            default_dir=str(artifact_dir),
            output_dir=mock_project["output_dir"],
            force_default_dir=True,
        )
        assert result is not None
        assert result.parent == artifact_dir

    def test_cuo_artifact_dir_honors_relative_env_override(self, mock_project, monkeypatch):
        """Operators can redirect artifacts without using .cyberos-memory."""
        monkeypatch.setenv("CYBEROS_CUO_ARTIFACT_ROOT", "tmp/cuo-artifacts")
        artifact_dir = _resolve_cuo_artifact_dir(
            "audits",
            hand_off={},
            output_dir=mock_project["output_dir"],
        )
        assert artifact_dir == mock_project["project"] / "tmp" / "cuo-artifacts" / "audits"
