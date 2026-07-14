"""Tests for BriefGenerator — execution brief generation for host LLMs."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from cuo.core.catalog import discover_personas, discover_workflows
from cuo.core.brief_generator import (
    BriefGenerator,
    _detect_project_context,
    _has_project_markers,
    _read_skill_frontmatter,
    _resolve_project_root,
    _strip_frontmatter,
    _APPLIER_INSTRUCTIONS,
)


@pytest.fixture(scope="module")
def cuo_root() -> Path:
    """Resolve the cuo module root (contains MODULE.md)."""
    # Walk up from this test file to find MODULE.md
    cur = Path(__file__).resolve().parent
    for _ in range(8):
        if (cur / "MODULE.md").is_file():
            return cur
        if cur.parent == cur:
            break
        cur = cur.parent
    pytest.skip("MODULE.md not found — running outside cuo module")


@pytest.fixture(scope="module")
def skill_root(cuo_root) -> Path:
    """Resolve the skill/ root (sibling to cuo/)."""
    candidate = cuo_root.parent.parent / "modules" / "skill"
    if candidate.is_dir():
        return candidate
    # Legacy flat layout
    candidate = cuo_root.parent / "skill"
    if candidate.is_dir():
        return candidate
    pytest.skip("skill/ directory not found")


@pytest.fixture(scope="module")
def cto_persona(cuo_root):
    """Get the chief-technology-officer persona."""
    personas = discover_personas(cuo_root)
    persona = next((p for p in personas if p.slug == "chief-technology-officer"), None)
    if persona is None:
        pytest.skip("chief-technology-officer persona not found")
    return persona


@pytest.fixture(scope="module")
def ship_wf(cto_persona, skill_root):
    """Get the ship-tasks workflow."""
    workflows = discover_workflows(cto_persona)
    wf = next((w for w in workflows if w.slug == "ship-tasks"), None)
    if wf is None:
        pytest.skip("ship-tasks workflow not found")
    return wf


class TestDetectProjectContext:
    """Project context detection from CWD markers."""

    def test_detects_python_project(self, tmp_path):
        (tmp_path / "pyproject.toml").write_text("[project]\nname = 'test'\n")
        ctx = _detect_project_context(tmp_path)
        assert ctx["language"] == "python"

    def test_detects_rust_project(self, tmp_path):
        (tmp_path / "Cargo.toml").write_text("[package]\nname = 'test'\n")
        ctx = _detect_project_context(tmp_path)
        assert ctx["language"] == "rust"

    def test_empty_project(self, tmp_path):
        ctx = _detect_project_context(tmp_path)
        assert ctx["language"] == "unknown"

    def test_detects_node_project(self, tmp_path):
        pkg = {"name": "test", "dependencies": {"react": "^18.0.0"}}
        (tmp_path / "package.json").write_text(json.dumps(pkg))
        ctx = _detect_project_context(tmp_path)
        assert ctx["language"] == "javascript"
        assert ctx["framework"] == "react"


class TestStripFrontmatter:
    """YAML frontmatter stripping."""

    def test_strips_frontmatter(self):
        text = "---\ntitle: test\n---\n\nBody content\n"
        assert _strip_frontmatter(text) == "Body content"

    def test_no_frontmatter(self):
        text = "Just body content\n"
        assert _strip_frontmatter(text) == "Just body content"


class TestApplierInstructions:
    """Applier instructions are defined for side-effect skills."""

    def test_backlog_update_has_instructions(self):
        assert "backlog-state-update-author" in _APPLIER_INSTRUCTIONS
        assert "BACKLOG.md" in _APPLIER_INSTRUCTIONS["backlog-state-update-author"]

    def test_impl_plan_has_instructions(self):
        assert "implementation-plan-author" in _APPLIER_INSTRUCTIONS
        assert "code_changes" in _APPLIER_INSTRUCTIONS["implementation-plan-author"]

    def test_coverage_gate_has_instructions(self):
        assert "coverage-gate-author" in _APPLIER_INSTRUCTIONS
        assert "pytest" in _APPLIER_INSTRUCTIONS["coverage-gate-author"]

    def test_code_review_has_instructions(self):
        assert "code-review-author" in _APPLIER_INSTRUCTIONS

    def test_obs_injection_has_instructions(self):
        assert "observability-injection-author" in _APPLIER_INSTRUCTIONS


class TestBriefGenerator:
    """Brief generation against real workflow definitions."""

    def test_generates_brief_structure(
        self, cto_persona, ship_wf, skill_root, tmp_path
    ):
        output_dir = tmp_path / "output"
        output_dir.mkdir()

        generator = BriefGenerator(
            persona=cto_persona,
            workflow=ship_wf,
            skill_root=skill_root,
            output_dir=output_dir,
            task_id="TASK-OBS-002",
        )
        brief = generator.generate()

        # Verify key sections exist
        assert "# Execution Brief — TASK-OBS-002" in brief
        assert "## 1. Project Context" in brief
        assert "## 2. Task" in brief
        assert "## 5. Execution Plan" in brief
        assert "## 6. Completion" in brief

    def test_includes_step_instructions(
        self, cto_persona, ship_wf, skill_root, tmp_path
    ):
        output_dir = tmp_path / "output"
        output_dir.mkdir()

        generator = BriefGenerator(
            persona=cto_persona,
            workflow=ship_wf,
            skill_root=skill_root,
            output_dir=output_dir,
            task_id="TASK-OBS-002",
        )
        brief = generator.generate()

        # Verify steps are present
        assert "### Step 1: repo-context-map-author" in brief
        assert "### Step 2: repo-context-map-audit" in brief

    def test_includes_applier_instructions(
        self, cto_persona, ship_wf, skill_root, tmp_path
    ):
        output_dir = tmp_path / "output"
        output_dir.mkdir()

        generator = BriefGenerator(
            persona=cto_persona,
            workflow=ship_wf,
            skill_root=skill_root,
            output_dir=output_dir,
            task_id="TASK-OBS-002",
        )
        brief = generator.generate()

        # Verify applier instructions appear
        assert "Side Effects" in brief
        assert "BACKLOG.md" in brief

    def test_includes_conditional_steps(
        self, cto_persona, ship_wf, skill_root, tmp_path
    ):
        output_dir = tmp_path / "output"
        output_dir.mkdir()

        generator = BriefGenerator(
            persona=cto_persona,
            workflow=ship_wf,
            skill_root=skill_root,
            output_dir=output_dir,
            task_id="TASK-OBS-002",
        )
        brief = generator.generate()

        # The workflow has conditional steps
        assert "Conditional Steps" in brief
        assert "condition" in brief.lower()

    def test_handles_missing_fr_gracefully(
        self, cto_persona, ship_wf, skill_root, tmp_path
    ):
        output_dir = tmp_path / "output"
        output_dir.mkdir()

        generator = BriefGenerator(
            persona=cto_persona,
            workflow=ship_wf,
            skill_root=skill_root,
            output_dir=output_dir,
            task_id="FR-NONEXISTENT-999",
        )
        brief = generator.generate()

        # Should still generate a brief (with FR content unavailable)
        assert "# Execution Brief" in brief
        assert "FR-NONEXISTENT-999" in brief


class TestBriefChain:
    """brief_chain() supervisor function."""

    def test_brief_chain_validates_workflow(
        self, cto_persona, skill_root, tmp_path
    ):
        from cuo.core.supervisor import brief_chain

        output_dir = tmp_path / "output"
        output_dir.mkdir()

        result = brief_chain(
            persona=cto_persona,
            workflow_slug="nonexistent-workflow",
            skill_root=skill_root,
            output_dir=output_dir,
            task_id="TASK-TEST-001",
        )
        assert "ERROR" in result
        assert "not found" in result

    def test_brief_chain_requires_fr_id(
        self, cto_persona, skill_root, tmp_path
    ):
        from cuo.core.supervisor import brief_chain

        output_dir = tmp_path / "output"
        output_dir.mkdir()

        result = brief_chain(
            persona=cto_persona,
            workflow_slug="ship-tasks",
            skill_root=skill_root,
            output_dir=output_dir,
            task_id=None,
        )
        assert "ERROR" in result
        assert "task_id" in result


class TestProjectRootResolution:
    """Project root resolution from output_dir fallback."""

    def test_returns_explicit_when_given(self, tmp_path):
        result = _resolve_project_root(tmp_path / "output", tmp_path / "explicit")
        assert result == tmp_path / "explicit"

    def test_falls_back_to_output_dir_parent(self, tmp_path):
        project = tmp_path / "my-project"
        project.mkdir()
        (project / "pyproject.toml").write_text("[project]\nname='x'\n")
        output_dir = project / "output"
        output_dir.mkdir()
        result = _resolve_project_root(output_dir, None)
        assert result == project

    def test_falls_back_to_cwd(self, tmp_path, monkeypatch):
        # No markers in output_dir.parent — should fall through to CWD
        monkeypatch.chdir(tmp_path)
        (tmp_path / "Cargo.toml").write_text("[package]\nname='x'\n")
        result = _resolve_project_root(tmp_path / "nonexistent" / "output", None)
        assert result == tmp_path

    def test_no_markers_anywhere_uses_cwd(self, tmp_path, monkeypatch):
        monkeypatch.chdir(tmp_path)
        result = _resolve_project_root(tmp_path / "nonexistent" / "output", None)
        assert result == tmp_path


class TestHasProjectMarkers:
    """Marker file detection."""

    def test_finds_pyproject(self, tmp_path):
        (tmp_path / "pyproject.toml").touch()
        assert _has_project_markers(tmp_path) is True

    def test_finds_package_json(self, tmp_path):
        (tmp_path / "package.json").touch()
        assert _has_project_markers(tmp_path) is True

    def test_finds_cargo_toml(self, tmp_path):
        (tmp_path / "Cargo.toml").touch()
        assert _has_project_markers(tmp_path) is True

    def test_finds_go_mod(self, tmp_path):
        (tmp_path / "go.mod").touch()
        assert _has_project_markers(tmp_path) is True

    def test_empty_dir(self, tmp_path):
        assert _has_project_markers(tmp_path) is False


class TestReadSkillFrontmatter:
    """Skill frontmatter parsing."""

    def test_reads_outputs_from_real_skill(self, skill_root):
        fm = _read_skill_frontmatter("repo-context-map-author", skill_root)
        assert "outputs" in fm
        outputs = fm["outputs"]
        assert len(outputs) >= 1
        assert outputs[0]["name"] == "context_map"
        assert "format" in outputs[0]

    def test_returns_empty_for_missing_skill(self, skill_root):
        fm = _read_skill_frontmatter("nonexistent-skill-999", skill_root)
        assert fm == {}

    def test_reads_audit_fields(self, skill_root):
        fm = _read_skill_frontmatter("repo-context-map-author", skill_root)
        audit = fm.get("audit", {})
        fields = audit.get("required_fields", [])
        assert "task_id" in fields


class TestBriefOutputFormat:
    """Structured output format in brief."""

    def test_includes_output_format_reference(
        self, cto_persona, ship_wf, skill_root, tmp_path
    ):
        output_dir = tmp_path / "output"
        output_dir.mkdir()

        generator = BriefGenerator(
            persona=cto_persona,
            workflow=ship_wf,
            skill_root=skill_root,
            output_dir=output_dir,
            task_id="TASK-OBS-002",
        )
        brief = generator.generate()

        assert "## 7. Output Format Reference" in brief
        assert "repo-context-map@1" in brief

    def test_includes_audit_fields_per_step(
        self, cto_persona, ship_wf, skill_root, tmp_path
    ):
        output_dir = tmp_path / "output"
        output_dir.mkdir()

        generator = BriefGenerator(
            persona=cto_persona,
            workflow=ship_wf,
            skill_root=skill_root,
            output_dir=output_dir,
            task_id="TASK-OBS-002",
        )
        brief = generator.generate()

        # Step 1 should show audit fields
        assert "Audit fields" in brief

    def test_includes_hand_off_keys(
        self, cto_persona, ship_wf, skill_root, tmp_path
    ):
        output_dir = tmp_path / "output"
        output_dir.mkdir()

        generator = BriefGenerator(
            persona=cto_persona,
            workflow=ship_wf,
            skill_root=skill_root,
            output_dir=output_dir,
            task_id="TASK-OBS-002",
        )
        brief = generator.generate()

        # Should show hand-off key references
        assert "Hand-off key" in brief
