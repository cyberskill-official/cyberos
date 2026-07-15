"""Tests for cuo.trigger_tests — TASK-SKILL-112 trigger-test runner."""

from __future__ import annotations

from pathlib import Path

import pytest

from cuo.trigger_tests import (
    SkillRoutingResult,
    TriggerTestRow,
    are_paraphrase_distinct,
    check_paraphrase_distinct,
    classify,
    load_fixture,
    run_for_skill,
    run_all,
    validate_confidence_relationship,
    _levenshtein,
    _unquote,
)


# ─── Fixture-loader tests ───────────────────────────────────────────────────────

def test_load_well_formed_fixture(tmp_path: Path):
    fixture = tmp_path / "TRIGGER_TESTS.md"
    fixture.write_text("""---
skill_id: foo-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for foo-author

## Positive triggers (MUST route here)

- "draft a foo"
- "turn this bar into a foo"
- "generate the foo backlog"

## Negative triggers (MUST NOT route here)

- "audit my foo" → foo-audit
- "draft a quux" → quux-author
- "what is the weather?" → none
""", encoding="utf-8")
    fm, rows = load_fixture(fixture)
    assert fm["skill_id"] == "foo-author"
    assert fm["min_confidence"] == 0.7
    assert fm["classifier_version"] == "3.0.0-a4"
    assert len(rows) == 6
    positives = [r for r in rows if r.is_positive]
    negatives = [r for r in rows if not r.is_positive]
    assert len(positives) == 3
    assert len(negatives) == 3
    assert positives[0].phrase == "draft a foo"
    assert positives[0].expected_skill == "foo-author"
    assert negatives[0].expected_skill == "foo-audit"
    assert negatives[2].expected_skill is None  # "→ none"


def test_load_missing_file():
    with pytest.raises(FileNotFoundError):
        load_fixture(Path("/nonexistent/TRIGGER_TESTS.md"))


def test_load_missing_frontmatter(tmp_path: Path):
    fixture = tmp_path / "TRIGGER_TESTS.md"
    fixture.write_text("# No frontmatter\n## Positive triggers\n- \"foo\"\n", encoding="utf-8")
    with pytest.raises(ValueError, match="missing leading"):
        load_fixture(fixture)


def test_load_missing_closing_frontmatter(tmp_path: Path):
    fixture = tmp_path / "TRIGGER_TESTS.md"
    fixture.write_text("---\nskill_id: foo\n# never closes\n", encoding="utf-8")
    with pytest.raises(ValueError, match="missing closing"):
        load_fixture(fixture)


def test_load_missing_required_frontmatter_key(tmp_path: Path):
    fixture = tmp_path / "TRIGGER_TESTS.md"
    fixture.write_text("""---
skill_id: foo
min_confidence: 0.7
---

## Positive triggers
- "foo"
""", encoding="utf-8")
    with pytest.raises(ValueError, match="classifier_version"):
        load_fixture(fixture)


# ─── Paraphrase-distinct tests ──────────────────────────────────────────────────

def test_paraphrase_distinct_close_variants_rejected():
    # "draft a task" vs "draft a task" — 1 char diff
    assert not are_paraphrase_distinct("draft a task", "draft a task")
    # "draft task" vs "draft tasks" — 1 char diff
    assert not are_paraphrase_distinct("draft task", "draft tasks")
    # Identical — distance 0, not distinct
    assert not are_paraphrase_distinct("draft a task", "draft a task")


def test_paraphrase_distinct_real_paraphrases_accepted():
    assert are_paraphrase_distinct(
        "draft a task",
        "turn this PRD into a backlog",
    )
    assert are_paraphrase_distinct(
        "audit this task",
        "check the rubric on this task",
    )


def test_check_paraphrase_distinct_finds_duplicates():
    phrases = ["draft a task", "draft a task", "turn this PRD into a backlog"]
    failures = check_paraphrase_distinct(phrases)
    assert len(failures) == 1
    a, b, d = failures[0]
    assert a == "draft a task" and b == "draft a task"
    assert d <= 3


def test_levenshtein_basic():
    assert _levenshtein("", "") == 0
    assert _levenshtein("a", "") == 1
    assert _levenshtein("", "a") == 1
    assert _levenshtein("abc", "abd") == 1
    assert _levenshtein("kitten", "sitting") == 3


# ─── Confidence-relationship tests ──────────────────────────────────────────────

def test_confidence_relationship_pass():
    assert validate_confidence_relationship(min_confidence=0.7, defer_below=0.5) is True
    assert validate_confidence_relationship(min_confidence=0.5, defer_below=0.5) is True  # equality OK


def test_confidence_relationship_fail():
    assert validate_confidence_relationship(min_confidence=0.3, defer_below=0.5) is False


# ─── Classifier tests (with monkeypatched classify) ─────────────────────────────

def test_run_for_skill_all_pass(monkeypatch, tmp_path: Path):
    def fake_classify(phrase: str) -> SkillRoutingResult:
        if "audit" in phrase:
            return SkillRoutingResult(skill_id="foo-audit", workflow_slug="foo-audit-wf", confidence=0.9)
        if "quux" in phrase:
            return SkillRoutingResult(skill_id="quux-author", workflow_slug="quux-wf", confidence=0.85)
        if "weather" in phrase:
            return SkillRoutingResult(skill_id=None, workflow_slug=None, confidence=0.0)
        return SkillRoutingResult(skill_id="foo-author", workflow_slug="foo-author-wf", confidence=0.85)

    monkeypatch.setattr("cuo.trigger_tests.classify", fake_classify)

    skill_dir = tmp_path / "foo-author"
    (skill_dir / "acceptance").mkdir(parents=True)
    (skill_dir / "acceptance" / "TRIGGER_TESTS.md").write_text("""---
skill_id: foo-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

## Positive triggers (MUST route here)
- "draft a foo"
- "turn this bar into a foo"
- "generate the foo backlog"

## Negative triggers (MUST NOT route here)
- "audit my foo" → foo-audit
- "draft a quux" → quux-author
- "what is the weather?" → none
""", encoding="utf-8")

    result = run_for_skill(skill_dir)
    assert result.passed is True, f"Expected pass; got failures: {result.failures}"
    assert len(result.rows) == 6
    assert len(result.failures) == 0


def test_run_for_skill_positive_misroute(monkeypatch, tmp_path: Path):
    def fake_classify(phrase: str) -> SkillRoutingResult:
        # Bug: positive phrase routes to wrong skill
        return SkillRoutingResult(skill_id="bar-author", workflow_slug="bar-wf", confidence=0.95)

    monkeypatch.setattr("cuo.trigger_tests.classify", fake_classify)

    skill_dir = tmp_path / "foo-author"
    (skill_dir / "acceptance").mkdir(parents=True)
    (skill_dir / "acceptance" / "TRIGGER_TESTS.md").write_text("""---
skill_id: foo-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

## Positive triggers (MUST route here)
- "draft a foo"
- "another phrase"
- "third phrase"

## Negative triggers (MUST NOT route here)
- "irrelevant" → none
- "totally different" → none
- "third negative" → none
""", encoding="utf-8")

    result = run_for_skill(skill_dir)
    assert result.passed is False
    # 3 positives should fail (route to wrong skill)
    # 3 negatives should pass (don't route to foo-author)
    positive_failures = [r for r, _, p in result.rows if r.is_positive and not p]
    assert len(positive_failures) == 3


def test_run_for_skill_negative_leak(monkeypatch, tmp_path: Path):
    def fake_classify(phrase: str) -> SkillRoutingResult:
        # Bug: every phrase leaks into foo-author
        return SkillRoutingResult(skill_id="foo-author", workflow_slug="foo-wf", confidence=0.95)

    monkeypatch.setattr("cuo.trigger_tests.classify", fake_classify)

    skill_dir = tmp_path / "foo-author"
    (skill_dir / "acceptance").mkdir(parents=True)
    (skill_dir / "acceptance" / "TRIGGER_TESTS.md").write_text("""---
skill_id: foo-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

## Positive triggers (MUST route here)
- "draft a foo"
- "another phrase"
- "third phrase"

## Negative triggers (MUST NOT route here)
- "audit my foo" → foo-audit
- "draft a quux" → quux-author
- "weather?" → none
""", encoding="utf-8")

    result = run_for_skill(skill_dir)
    assert result.passed is False
    # Positives: pass (correct routing)
    # Negatives: fail (all leak into foo-author)
    negative_failures = [r for r, _, p in result.rows if not r.is_positive and not p]
    assert len(negative_failures) == 3


def test_run_for_skill_missing_fixture(tmp_path: Path):
    skill_dir = tmp_path / "foo-author"
    skill_dir.mkdir()
    with pytest.raises(FileNotFoundError):
        run_for_skill(skill_dir)


def test_run_all_walks_catalog(monkeypatch, tmp_path: Path):
    def fake_classify(phrase: str) -> SkillRoutingResult:
        return SkillRoutingResult(skill_id="foo-author", workflow_slug="foo-wf", confidence=0.85)

    monkeypatch.setattr("cuo.trigger_tests.classify", fake_classify)

    # Two skill dirs, one with TRIGGER_TESTS.md and one without
    (tmp_path / "foo-author" / "acceptance").mkdir(parents=True)
    (tmp_path / "foo-author" / "acceptance" / "TRIGGER_TESTS.md").write_text("""---
skill_id: foo-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

## Positive triggers
- "a"
- "b"
- "c"

## Negative triggers
- "x" → none
- "y" → none
- "z" → none
""", encoding="utf-8")
    (tmp_path / "bar-author").mkdir()

    results = run_all(tmp_path)
    assert "foo-author" in results
    assert "bar-author" not in results  # gracefully skipped


def test_unquote_helper():
    assert _unquote('"hello"') == "hello"
    assert _unquote('  "hello"  ') == "hello"
    assert _unquote('hello') == "hello"
    assert _unquote('"') == '"'  # too short to strip
