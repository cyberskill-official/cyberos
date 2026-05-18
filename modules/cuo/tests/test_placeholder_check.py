"""Tests for cuo.placeholder_check — FR-SKILL-115 SKB-030 validator."""

from __future__ import annotations

from pathlib import Path

from cuo.placeholder_check import (
    PlaceholderHit,
    ScanResult,
    run_all,
    scan,
    summarize,
)


# ─── Single-skill scan tests ────────────────────────────────────────────────────

def test_detects_metadata_stage_placeholder(tmp_path: Path):
    skill = tmp_path / "SKILL.md"
    skill.write_text("""---
name: foo-author
description: "Generate a foo from bar"
metadata:
  stage: <SDP §2 stage letter or "cross">
  version: 1.0.0
wrap_in_marker: "untrusted_content"
---
body
""", encoding="utf-8")
    result = scan(skill)
    assert result.exempt is False
    assert result.error is None
    assert len(result.hits) >= 1
    stage_hits = [h for h in result.hits if h.field_path.endswith(".stage")]
    assert len(stage_hits) == 1
    assert "SDP" in stage_hits[0].token


def test_detects_description_placeholder(tmp_path: Path):
    skill = tmp_path / "SKILL.md"
    skill.write_text("""---
name: foo-author
description: "Author a <artifact> from <input>"
metadata:
  stage: b
wrap_in_marker: "untrusted_content"
---
""", encoding="utf-8")
    result = scan(skill)
    assert len(result.hits) == 2  # <artifact> + <input>
    tokens = sorted([h.token for h in result.hits])
    assert tokens == ["artifact", "input"]


def test_template_path_exempt(tmp_path: Path):
    template_dir = tmp_path / "_template" / "author"
    template_dir.mkdir(parents=True)
    skill = template_dir / "SKILL.md"
    skill.write_text("""---
name: <artefact>-author
description: "Author a <ARTEFACT>"
metadata:
  stage: <SDP §2 stage letter>
---
""", encoding="utf-8")
    result = scan(skill)
    assert result.exempt is True
    assert result.hits == []  # exempt → don't report hits even though placeholders exist


def test_template_path_exempt_via_underscore_prefix(tmp_path: Path):
    """EXEMPT_PATH_PARTS matches `_template/` and `/_template`."""
    nested = tmp_path / "skill" / "_template" / "audit"
    nested.mkdir(parents=True)
    skill = nested / "SKILL.md"
    skill.write_text("""---
name: <artifact>-audit
metadata:
  stage: <SDP §2 stage letter>
---
""", encoding="utf-8")
    result = scan(skill)
    assert result.exempt is True


def test_safe_html_tags_not_flagged(tmp_path: Path):
    """`<br>`, `<strong>`, `<sub>`, `<sup>`, `<span>`, `<b>`, `<i>`, `<em>` are HTML — not placeholders."""
    skill = tmp_path / "SKILL.md"
    skill.write_text("""---
name: foo-author
description: "Line one<br>Line two<strong>important</strong>"
metadata:
  stage: b
---
""", encoding="utf-8")
    result = scan(skill)
    assert result.hits == []


def test_wrap_in_marker_not_flagged(tmp_path: Path):
    """Post-FR-SKILL-113 form: wrap_in_marker is a plain string, never flagged."""
    skill = tmp_path / "SKILL.md"
    skill.write_text("""---
name: foo-author
description: "Author foo"
metadata:
  stage: b
wrap_in_marker: "untrusted_content"
---
""", encoding="utf-8")
    result = scan(skill)
    assert result.hits == []


def test_missing_file(tmp_path: Path):
    result = scan(tmp_path / "nonexistent.md")
    assert result.error == "file_missing"


def test_no_frontmatter(tmp_path: Path):
    skill = tmp_path / "SKILL.md"
    skill.write_text("# Body only\nNo frontmatter\n", encoding="utf-8")
    result = scan(skill)
    assert result.error == "no_frontmatter"


def test_no_closing_delimiter(tmp_path: Path):
    skill = tmp_path / "SKILL.md"
    skill.write_text("---\nname: foo\n# no closing\n", encoding="utf-8")
    result = scan(skill)
    assert result.error == "no_closing_frontmatter_delimiter"


def test_yaml_parse_error(tmp_path: Path):
    skill = tmp_path / "SKILL.md"
    skill.write_text("---\nname: foo\n  bad: indent\n nope\n---\n", encoding="utf-8")
    result = scan(skill)
    assert result.error is not None
    assert "yaml_parse" in result.error or result.error == "frontmatter_not_dict"


# ─── run_all walk tests ─────────────────────────────────────────────────────────

def test_run_all_walks_catalog(tmp_path: Path):
    # Two skills: one with placeholder, one clean
    (tmp_path / "foo-author").mkdir()
    (tmp_path / "foo-author" / "SKILL.md").write_text("""---
name: foo-author
metadata:
  stage: <SDP §2 stage letter>
---
""", encoding="utf-8")
    (tmp_path / "bar-author").mkdir()
    (tmp_path / "bar-author" / "SKILL.md").write_text("""---
name: bar-author
metadata:
  stage: b
---
""", encoding="utf-8")
    results = run_all(tmp_path)
    assert len(results) == 2
    # Find the one with hits + the clean one
    hits_count = sum(1 for r in results.values() if r.hits)
    assert hits_count == 1
    clean_count = sum(1 for r in results.values() if not r.hits and not r.error and not r.exempt)
    assert clean_count == 1


def test_run_all_skips_template_directory(tmp_path: Path):
    (tmp_path / "_template" / "author").mkdir(parents=True)
    (tmp_path / "_template" / "author" / "SKILL.md").write_text("""---
name: <artifact>-author
metadata:
  stage: <SDP §2 stage letter>
---
""", encoding="utf-8")
    (tmp_path / "real-skill").mkdir()
    (tmp_path / "real-skill" / "SKILL.md").write_text("""---
name: real-skill
metadata:
  stage: b
---
""", encoding="utf-8")
    results = run_all(tmp_path)
    assert len(results) == 2
    exempt_count = sum(1 for r in results.values() if r.exempt)
    assert exempt_count == 1
    hits_count = sum(1 for r in results.values() if r.hits)
    assert hits_count == 0  # template's hits don't surface; real skill is clean


# ─── Summary tests ──────────────────────────────────────────────────────────────

def test_summarize_output_shape(tmp_path: Path):
    (tmp_path / "a").mkdir()
    (tmp_path / "a" / "SKILL.md").write_text("""---
name: a
metadata:
  stage: <SDP stage>
---
""", encoding="utf-8")
    results = run_all(tmp_path)
    summary = summarize(results)
    assert "Scanned: 1" in summary
    assert "placeholder hits: 1" in summary
