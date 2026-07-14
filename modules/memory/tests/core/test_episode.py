"""Tests for TASK-MEMORY-112 — episodic memory + recall-similar.

Covers acceptance criteria from
`docs/tasks/memory/TASK-MEMORY-112-episodic-memory/spec.md`:

* AC #1 – schema accepts `kind: episode` (verified via Frontmatter round-trip)
* AC #2 – missing required Episode extras rejected by validate_episode_extras
* AC #3 – existing memory kinds unchanged
* AC #5 – outcome enum closed
* AC #6 – quality_score range enforced
* AC #7 – duration non-negative
* AC #8 – error required on non-success
* AC #9 – searchable_document() shape exact
* AC #10 – `cyberos episode log` happy path
* AC #11 – recall-similar filters to episode kind
* AC #12 – combined-score ordering
* AC #13 – default quality_score = 0.5
* AC #21 – structured no-match response
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from cyberos.core.episode import (
    Episode,
    log as episode_log,
    recall_similar,
    validate_episode_extras,
)
from cyberos.core.frontmatter import Frontmatter, parse, serialize
from cyberos.core.writer import Writer


# ---- conftest-style helper ------------------------------------------------


@pytest.fixture()
def store(tmp_path: Path) -> Path:
    s = tmp_path / ".cyberos/memory/store"
    (s / "audit").mkdir(parents=True)
    (s / "memories" / "episodes").mkdir(parents=True)
    return s


# ---- Episode dataclass validation ----------------------------------------


def test_outcome_closed_enum_success() -> None:
    """AC #5 — success accepted."""
    Episode(task="t", approach="a", outcome="success", duration_ms=10)


def test_outcome_closed_enum_partial_requires_error() -> None:
    """AC #5 + AC #8 — partial accepted only with error."""
    with pytest.raises(ValueError, match="non-empty error"):
        Episode(task="t", approach="a", outcome="partial", duration_ms=10)
    Episode(
        task="t", approach="a", outcome="partial", duration_ms=10,
        error="dispatcher returned 502 once",
    )


def test_outcome_closed_enum_failure_requires_error() -> None:
    """AC #5 + AC #8."""
    with pytest.raises(ValueError, match="non-empty error"):
        Episode(task="t", approach="a", outcome="failure", duration_ms=10)
    Episode(
        task="t", approach="a", outcome="failure", duration_ms=10,
        error="tokens exhausted",
    )


def test_outcome_outside_enum_rejected() -> None:
    """AC #5."""
    with pytest.raises(ValueError, match="closed enum"):
        Episode(task="t", approach="a", outcome="kinda", duration_ms=10)


@pytest.mark.parametrize("qs,ok", [
    (0.0, True), (0.5, True), (1.0, True), (None, True),
    (-0.0001, False), (1.0001, False),
])
def test_quality_score_range(qs: float | None, ok: bool) -> None:
    """AC #6."""
    if ok:
        Episode(
            task="t", approach="a", outcome="success", duration_ms=10,
            quality_score=qs,
        )
    else:
        with pytest.raises(ValueError, match="quality_score"):
            Episode(
                task="t", approach="a", outcome="success", duration_ms=10,
                quality_score=qs,
            )


def test_duration_non_negative() -> None:
    """AC #7."""
    Episode(task="t", approach="a", outcome="success", duration_ms=0)
    with pytest.raises(ValueError, match="duration_ms"):
        Episode(task="t", approach="a", outcome="success", duration_ms=-1)


def test_task_non_empty_after_trim() -> None:
    """Constructor invariant — `task` must be non-empty."""
    with pytest.raises(ValueError, match="task"):
        Episode(task="   ", approach="a", outcome="success", duration_ms=1)


def test_searchable_document_shape() -> None:
    """AC #9 — exact format."""
    ep = Episode(
        task="X", approach="Y", outcome="success", duration_ms=10, notes="N",
    )
    assert (
        ep.searchable_document()
        == "Task: X\nApproach: Y\nOutcome: success\nNotes: N"
    )


def test_extra_fields_omits_absent_optionals() -> None:
    """The extras dict only carries the fields the Episode actually set."""
    ep = Episode(task="t", approach="a", outcome="success", duration_ms=1)
    extras = ep.extra_fields()
    assert set(extras) == {"task", "approach", "outcome", "duration_ms"}


def test_extra_fields_carries_optionals() -> None:
    ep = Episode(
        task="t", approach="a", outcome="success", duration_ms=1,
        token_cost=1200, quality_score=0.9, notes="N", error=None,
    )
    extras = ep.extra_fields()
    assert extras["token_cost"] == 1200
    assert extras["quality_score"] == 0.9
    assert extras["notes"] == "N"
    # success ⇒ no error
    assert "error" not in extras


# ---- validate_episode_extras walker invariant ------------------------------


def test_validate_extras_accepts_well_formed() -> None:
    """AC #2 reciprocal."""
    validate_episode_extras({
        "task": "x", "approach": "y", "outcome": "success", "duration_ms": 1,
    })


@pytest.mark.parametrize("missing", ["task", "approach", "outcome", "duration_ms"])
def test_validate_extras_missing_required(missing: str) -> None:
    """AC #2 — walker rejects missing required field."""
    base = {"task": "x", "approach": "y", "outcome": "success", "duration_ms": 1}
    bad = {k: v for k, v in base.items() if k != missing}
    with pytest.raises(ValueError):
        validate_episode_extras(bad)


def test_validate_extras_outcome_closed_enum() -> None:
    with pytest.raises(ValueError, match="outcome-closed-enum"):
        validate_episode_extras({
            "task": "x", "approach": "y", "outcome": "kinda", "duration_ms": 1,
        })


def test_validate_extras_quality_score_range() -> None:
    with pytest.raises(ValueError, match="quality-score-range"):
        validate_episode_extras({
            "task": "x", "approach": "y", "outcome": "success",
            "duration_ms": 1, "quality_score": 1.5,
        })


def test_validate_extras_duration_non_negative() -> None:
    with pytest.raises(ValueError, match="duration-non-negative"):
        validate_episode_extras({
            "task": "x", "approach": "y", "outcome": "success", "duration_ms": -1,
        })


def test_validate_extras_error_required_on_non_success() -> None:
    with pytest.raises(ValueError, match="error-on-non-success"):
        validate_episode_extras({
            "task": "x", "approach": "y", "outcome": "failure", "duration_ms": 1,
        })


# ---- end-to-end log + recall ----------------------------------------------


def test_episode_log_writes_audit_row(store: Path) -> None:
    """AC #4 — `log()` advances HEAD via canonical writer."""
    ep = Episode(
        task="ship FR", approach="audit-revise loop", outcome="success",
        duration_ms=1800_000, quality_score=0.92,
    )
    with Writer(store) as writer:
        seq, rel_path = episode_log(writer, ep, actor="stephen")
    assert seq == 1
    abs_path = store / rel_path
    assert abs_path.exists()
    # Frontmatter round-trips
    raw = abs_path.read_bytes()
    fm, body = parse(raw)
    assert fm.kind == "episode"
    assert fm.actor == "stephen"
    assert fm.extra["task"] == "ship FR"
    assert fm.extra["outcome"] == "success"
    assert fm.extra["quality_score"] == 0.92
    # Body matches the searchable document
    assert body.startswith(b"Task: ship FR\n")


def test_recall_similar_filters_to_episode_kind(store: Path) -> None:
    """AC #11 — recall-similar returns only kind=episode hits."""
    # Seed a non-episode file with overlapping vocabulary
    fact_path = store / "memories" / "facts" / "ab" / "cd" / "abcdefgh.md"
    fact_path.parent.mkdir(parents=True, exist_ok=True)
    fact_fm = Frontmatter(
        id="FACT-1", kind="fact", ts_ns=1, actor="t", tags=[], extra={},
    )
    fact_path.write_bytes(serialize(fact_fm, b"ship a task together"))
    # Seed two episodes
    with Writer(store) as writer:
        episode_log(
            writer,
            Episode(
                task="ship a task",
                approach="audit-revise loop",
                outcome="success", duration_ms=10,
            ),
            actor="t",
        )
        episode_log(
            writer,
            Episode(
                task="completely unrelated topic about pizza",
                approach="pizza approach",
                outcome="success", duration_ms=10,
            ),
            actor="t",
        )
    result = recall_similar(store, "ship a feature", k=10, min_relevance=0.0)
    # All returned matches must be episodes (no fact file)
    paths = [m["path"] for m in result["matches"]]
    assert all("episodes" in p for p in paths)
    assert "facts" not in " ".join(paths)


def test_recall_similar_combined_score_orders_by_quality(store: Path) -> None:
    """AC #12 — higher quality_score wins on tied relevance."""
    with Writer(store) as writer:
        episode_log(
            writer,
            Episode(
                task="ship the task",
                approach="loop",
                outcome="success", duration_ms=10, quality_score=0.4,
            ),
            actor="t",
        )
        episode_log(
            writer,
            Episode(
                task="ship the task",
                approach="loop",
                outcome="success", duration_ms=10, quality_score=0.9,
            ),
            actor="t",
        )
    result = recall_similar(store, "ship the task", k=2, min_relevance=0.0)
    assert len(result["matches"]) == 2
    qs = [m["quality_score"] for m in result["matches"]]
    # The higher quality_score must come first
    assert qs[0] == 0.9
    assert qs[1] == 0.4


def test_recall_similar_default_quality_acts_as_half(store: Path) -> None:
    """AC #13 — absent quality_score ranks identically to 0.5."""
    with Writer(store) as writer:
        episode_log(
            writer,
            Episode(
                task="task with no quality",
                approach="x",
                outcome="success", duration_ms=10,
            ),
            actor="t",
        )
        episode_log(
            writer,
            Episode(
                task="task with explicit half",
                approach="x",
                outcome="success", duration_ms=10, quality_score=0.5,
            ),
            actor="t",
        )
    no_qs = recall_similar(store, "task with no quality", k=1, min_relevance=0.0)
    half = recall_similar(store, "task with explicit half", k=1, min_relevance=0.0)
    # Both should have the same combined_score component for quality
    assert abs(no_qs["matches"][0]["combined_score"]
               - half["matches"][0]["combined_score"]) < 0.01


def test_recall_similar_no_episodes_in_store(store: Path) -> None:
    """AC #21 — structured response on empty store."""
    result = recall_similar(store, "anything", k=3, min_relevance=0.0)
    assert result["matches"] == []
    assert result["reason"] == "no_episodes_in_store"


def test_recall_similar_no_matches_above_threshold(store: Path) -> None:
    """AC #21 — structured response when filter rejects everything."""
    with Writer(store) as writer:
        episode_log(
            writer,
            Episode(
                task="pizza dough recipe",
                approach="ferment",
                outcome="success", duration_ms=10,
            ),
            actor="t",
        )
    # Query is totally unrelated → relevance 0 (filtered out by min)
    result = recall_similar(
        store, "ship a task", k=3, min_relevance=0.65,
    )
    assert result["matches"] == []
    assert result["reason"] == "no_episodes_above_min_relevance"


def test_legacy_kinds_still_round_trip(store: Path) -> None:
    """AC #3 — existing non-episode kinds unchanged."""
    for kind in ("decision", "fact", "person", "project", "preference",
                 "drift", "refinement", "unknown"):
        fm = Frontmatter(
            id=f"X-{kind}", kind=kind, ts_ns=1, actor="t", tags=[], extra={},
        )
        # Round-trip via serialize/parse — accept the legacy shape
        raw = serialize(fm, b"body bytes")
        fm2, body2 = parse(raw)
        assert fm2.kind == kind
        assert body2 == b"body bytes"
