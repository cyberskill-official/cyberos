"""Tests for cuo.baseline — TASK-SKILL-114 BASELINE.md validator."""

from __future__ import annotations

from pathlib import Path

from cuo.baseline import validate


# Minimum valid baseline (all required sections + valid frontmatter)
_VALID_BODY = """
## Workflow under test

The workflow is "turn a PRD into a task backlog".

## Without-skill baseline

**Measurement window:** 2026-04-15 → 2026-05-15. **Sample size:** n=12.

## With-skill measurements

Tool-call ratio 0.23, token ratio 0.26, failure-rate ratio 0.24.

## Token-budget transparency

Per-invocation budget: 40,000 tokens.

## Trust calibration

`confidence_band.default: 0.7` chosen because the skill is judgement work.

## Authoring notes

- Sample size n=12 is small.
- Attestation chain: gathered by claude-opus-4-7; reviewed by cuo-cpo.
"""


def _write_fixture(tmp_path: Path, frontmatter: str, body: str = _VALID_BODY) -> Path:
    path = tmp_path / "BASELINE.md"
    path.write_text(f"---\n{frontmatter}\n---\n{body}", encoding="utf-8")
    return path


def test_valid_baseline_passes(tmp_path: Path):
    path = _write_fixture(tmp_path, """skill_id: task-author
baseline_version: 1.0.0
baseline_measured_at: 2026-05-19T15:30:00+07:00
attested_by: cuo-cpo
next_review_due: 2099-05-19T00:00:00+07:00""")
    result = validate(path)
    assert result.valid is True
    assert result.skill_id == "task-author"
    assert result.issues == []
    assert result.warnings == []


def test_missing_file(tmp_path: Path):
    result = validate(tmp_path / "nonexistent.md")
    assert result.valid is False
    assert "file_missing" in result.issues


def test_missing_leading_delimiter(tmp_path: Path):
    path = tmp_path / "BASELINE.md"
    path.write_text("# No frontmatter\n", encoding="utf-8")
    result = validate(path)
    assert result.valid is False
    assert any("missing leading" in i for i in result.issues)


def test_missing_closing_delimiter(tmp_path: Path):
    path = tmp_path / "BASELINE.md"
    path.write_text("---\nskill_id: foo\n# no closing\n", encoding="utf-8")
    result = validate(path)
    assert result.valid is False
    assert any("missing closing" in i for i in result.issues)


def test_missing_required_attested_by(tmp_path: Path):
    path = _write_fixture(tmp_path, """skill_id: foo
baseline_version: 1.0.0
baseline_measured_at: 2026-05-19T15:30:00+07:00
next_review_due: 2027-05-19T00:00:00+07:00""")
    result = validate(path)
    assert result.valid is False
    assert any("attested_by" in i for i in result.issues)


def test_invalid_attested_by_form(tmp_path: Path):
    path = _write_fixture(tmp_path, """skill_id: foo
baseline_version: 1.0.0
baseline_measured_at: 2026-05-19T15:30:00+07:00
attested_by: random-person
next_review_due: 2099-05-19T00:00:00+07:00""")
    result = validate(path)
    assert result.valid is False
    assert any("attested_by_invalid" in i for i in result.issues)


def test_persona_attestation_accepted(tmp_path: Path):
    path = _write_fixture(tmp_path, """skill_id: foo
baseline_version: 1.0.0
baseline_measured_at: 2026-05-19T15:30:00+07:00
attested_by: cuo-cpo
next_review_due: 2099-05-19T00:00:00+07:00""")
    result = validate(path)
    assert result.valid is True


def test_human_attestation_accepted(tmp_path: Path):
    path = _write_fixture(tmp_path, """skill_id: foo
baseline_version: 1.0.0
baseline_measured_at: 2026-05-19T15:30:00+07:00
attested_by: human:stephen-cheng
next_review_due: 2099-05-19T00:00:00+07:00""")
    result = validate(path)
    assert result.valid is True


def test_missing_body_section(tmp_path: Path):
    # Body lacks `## Token-budget transparency`
    short_body = """
## Workflow under test
## Without-skill baseline
## With-skill measurements
## Trust calibration
## Authoring notes
"""
    path = _write_fixture(tmp_path, """skill_id: foo
baseline_version: 1.0.0
baseline_measured_at: 2026-05-19T15:30:00+07:00
attested_by: cuo-cpo
next_review_due: 2099-05-19T00:00:00+07:00""", body=short_body)
    result = validate(path)
    assert result.valid is False
    assert any("Token-budget transparency" in i for i in result.issues)


def test_review_overdue_within_year_warns(tmp_path: Path):
    # 30 days overdue (clearly within 365)
    path = _write_fixture(tmp_path, """skill_id: foo
baseline_version: 1.0.0
baseline_measured_at: 2025-04-01T00:00:00+07:00
attested_by: cuo-cpo
next_review_due: 2026-04-01T00:00:00+07:00""")
    result = validate(path)
    # Whether this is valid depends on whether other rules pass; the warning is what matters
    assert any("review_overdue" in w for w in result.warnings) or any("review_overdue" in i for i in result.issues)


def test_review_stale_over_year_fails(tmp_path: Path):
    # > 365 days overdue
    path = _write_fixture(tmp_path, """skill_id: foo
baseline_version: 1.0.0
baseline_measured_at: 2020-01-01T00:00:00+07:00
attested_by: cuo-cpo
next_review_due: 2020-06-01T00:00:00+07:00""")
    result = validate(path)
    assert result.valid is False
    assert any("review_overdue" in i and "stale" in i for i in result.issues)


def test_invalid_next_review_due_parse_error(tmp_path: Path):
    path = _write_fixture(tmp_path, """skill_id: foo
baseline_version: 1.0.0
baseline_measured_at: 2026-05-19T15:30:00+07:00
attested_by: cuo-cpo
next_review_due: 'not-a-date'""")
    result = validate(path)
    assert result.valid is False
    assert any("next_review_due" in i for i in result.issues)
