"""Tests for FR-MEMORY-113 — recency-decay recall ranking.

Covers acceptance criteria from
`docs/feature-requests/memory/FR-MEMORY-113-recency-decay-recall.md`:

* AC #1 — default weights are (0.4, 0.3, 0.3)
* AC #2 — sum-to-1.0 constraint
* AC #3 — weights bounded to [0, 1]
* AC #5 — absent importance defaults to 0.5
* AC #7 — Exponential half-life ≈ 138 h
* AC #8 — Exponential at hours_old=0 → 1.0
* AC #9 — Exponential monotonic decreasing
* AC #10 — Ebbinghaus at 0 → 1.0; finite for large h
* AC #11 — decay parameter validation
* AC #12 — last_seen_at absent → recency=1.0
* AC #13 — last_seen_at in future → recency=1.0
* AC #14 — combined-score formula
* AC #15 — ordering
* AC #19 — pure function — `now` injected
* AC #20 — ScoredHit annotations preserved
* AC #21 — relevance-only mode
"""

from __future__ import annotations

from datetime import datetime, timedelta, timezone
from types import SimpleNamespace

import pytest

from cyberos.core.decay import Ebbinghaus, Exponential, build_profile, hours_between
from cyberos.core.ranking import RecallWeights, ScoredHit, score_hits


# ---- RecallWeights ---------------------------------------------------------


def test_default_weights() -> None:
    """AC #1."""
    w = RecallWeights()
    assert (w.relevance, w.importance, w.recency) == (0.4, 0.3, 0.3)


@pytest.mark.parametrize("triple,raises", [
    ((0.4, 0.3, 0.3), False),
    ((0.5, 0.3, 0.3), True),   # sum 1.1
    ((0.4, 0.3, 0.2), True),   # sum 0.9
    ((-0.1, 0.55, 0.55), True),  # negative
    ((1.1, -0.05, -0.05), True),  # out of range
    ((1.0, 0.0, 0.0), False),  # relevance-only
])
def test_weight_validation(triple, raises) -> None:
    """AC #2 + AC #3."""
    if raises:
        with pytest.raises(ValueError):
            RecallWeights(*triple)
    else:
        RecallWeights(*triple)


def test_relevance_only_classmethod() -> None:
    """AC #21."""
    w = RecallWeights.relevance_only()
    assert (w.relevance, w.importance, w.recency) == (1.0, 0.0, 0.0)


# ---- Exponential decay -----------------------------------------------------


def test_exponential_zero() -> None:
    """AC #8."""
    assert Exponential()(0.0) == 1.0


def test_exponential_half_life() -> None:
    """AC #7 — Park-et-al exponential half-life ≈ 138 h."""
    e = Exponential(0.995)
    assert 137.8 < e.half_life_hours < 138.5


def test_exponential_monotonic() -> None:
    """AC #9."""
    e = Exponential()
    for h1, h2 in [(0, 1), (1, 5), (5, 20), (20, 100)]:
        assert e(h1) > e(h2), f"Exponential({h1}) > Exponential({h2}) failed"


def test_exponential_negative_caps_at_one() -> None:
    """AC #13."""
    assert Exponential()(-10.0) == 1.0


@pytest.mark.parametrize("kwargs", [
    {"decay_factor": 1.5},
    {"decay_factor": -0.1},
    {"decay_factor": 0.0},
    {"decay_factor": 1.0},
])
def test_exponential_param_validation(kwargs) -> None:
    """AC #11."""
    with pytest.raises(ValueError):
        Exponential(**kwargs)


# ---- Ebbinghaus decay ------------------------------------------------------


def test_ebbinghaus_zero() -> None:
    """AC #10."""
    assert Ebbinghaus()(0.0) == 1.0


def test_ebbinghaus_finite_at_large_h() -> None:
    """AC #10."""
    assert 0 < Ebbinghaus()(10000.0) < 1.0


def test_ebbinghaus_monotonic() -> None:
    eb = Ebbinghaus()
    for h1, h2 in [(0, 1), (1, 100), (100, 1000)]:
        assert eb(h1) > eb(h2)


@pytest.mark.parametrize("kwargs", [
    {"strength": 0},
    {"strength": -1.0},
])
def test_ebbinghaus_param_validation(kwargs) -> None:
    """AC #11."""
    with pytest.raises(ValueError):
        Ebbinghaus(**kwargs)


# ---- build_profile dispatch ------------------------------------------------


def test_build_profile_known() -> None:
    e1 = build_profile("exponential", {"decay_factor": 0.99})
    e2 = build_profile("ebbinghaus", {"strength": 100})
    assert e1(0) == 1.0
    assert e2(0) == 1.0


def test_build_profile_unknown_raises() -> None:
    with pytest.raises(ValueError, match="unknown decay profile"):
        build_profile("made_up")


# ---- hours_between ---------------------------------------------------------


def test_hours_between_none_last_seen() -> None:
    assert hours_between(datetime.now(timezone.utc), None) is None


def test_hours_between_naive_coerces_utc() -> None:
    now = datetime(2026, 5, 19, 12, 0, 0, tzinfo=timezone.utc)
    last_seen = datetime(2026, 5, 19, 10, 0, 0)  # naïve
    assert hours_between(now, last_seen) == pytest.approx(2.0)


# ---- score_hits ------------------------------------------------------------


def _hit(relevance, importance=None, last_seen_at=None, path="p"):
    fm = {} if importance is None else {"importance": importance}
    return SimpleNamespace(
        path=path, relevance=relevance, frontmatter=fm, last_seen_at=last_seen_at,
    )


def test_importance_defaults_to_half() -> None:
    """AC #5."""
    now = datetime.now(timezone.utc)
    s = score_hits([_hit(0.8, importance=None, last_seen_at=now)],
                   RecallWeights(), Exponential(), now=now)
    assert s[0].importance == 0.5


def test_last_seen_none_recency_one() -> None:
    """AC #12."""
    now = datetime.now(timezone.utc)
    s = score_hits([_hit(0.5, last_seen_at=None)],
                   RecallWeights(), Exponential(), now=now)
    assert s[0].recency == 1.0


def test_last_seen_future_recency_one() -> None:
    """AC #13."""
    now = datetime.now(timezone.utc)
    future = now + timedelta(hours=24)
    s = score_hits([_hit(0.5, last_seen_at=future)],
                   RecallWeights(), Exponential(), now=now)
    assert s[0].recency == 1.0


def test_combined_score_formula() -> None:
    """AC #14 — exact math at hours_old=0 (recency=1.0)."""
    now = datetime.now(timezone.utc)
    s = score_hits([_hit(0.8, importance=0.6, last_seen_at=now)],
                   RecallWeights(0.4, 0.3, 0.3), Exponential(), now=now)
    # 0.8*0.4 + 0.6*0.3 + 1.0*0.3 = 0.32 + 0.18 + 0.30 = 0.80
    assert s[0].combined_score == pytest.approx(0.80)


def test_ordering_descending() -> None:
    """AC #15."""
    now = datetime.now(timezone.utc)
    hits = [
        _hit(0.1, importance=0.1, last_seen_at=now, path="low"),
        _hit(0.9, importance=0.9, last_seen_at=now, path="high"),
        _hit(0.5, importance=0.5, last_seen_at=now, path="mid"),
    ]
    s = score_hits(hits, RecallWeights(), Exponential(), now=now)
    assert [h.path for h in s] == ["high", "mid", "low"]


def test_pure_function_now_injected() -> None:
    """AC #19 — score_hits doesn't call datetime.now() internally."""
    now = datetime(2026, 5, 19, tzinfo=timezone.utc)
    hit = _hit(0.5, last_seen_at=now - timedelta(hours=10))
    s1 = score_hits([hit], RecallWeights(), Exponential(), now=now)
    s2 = score_hits([hit], RecallWeights(), Exponential(), now=now)
    assert s1[0].combined_score == s2[0].combined_score


def test_relevance_only_mode_preserves_order() -> None:
    """AC #21 — weights=(1, 0, 0) ranks by relevance only."""
    now = datetime.now(timezone.utc)
    hits = [
        _hit(0.3, importance=0.9, last_seen_at=now),
        _hit(0.7, importance=0.1, last_seen_at=now),
    ]
    s = score_hits(hits, RecallWeights.relevance_only(), Exponential(), now=now)
    assert s[0].relevance == 0.7  # higher relevance wins despite lower importance


def test_scored_hit_annotations() -> None:
    """AC #20."""
    now = datetime.now(timezone.utc)
    s = score_hits([_hit(0.5, importance=0.7, last_seen_at=now)],
                   RecallWeights(), Exponential(), now=now)
    for field in ("relevance", "importance", "recency", "combined_score"):
        assert hasattr(s[0], field)


def test_score_hits_accepts_dict_hits() -> None:
    """Duck-typed accessor: dict-shaped hits work alongside objects."""
    now = datetime.now(timezone.utc)
    dict_hit = {
        "path": "x",
        "relevance": 0.5,
        "frontmatter": {"importance": 0.8},
        "last_seen_at": now,
    }
    s = score_hits([dict_hit], RecallWeights(), Exponential(), now=now)
    assert s[0].importance == 0.8


def test_iso_last_seen_string_coerced() -> None:
    """Accept ISO strings for last_seen_at — common from JSON sources."""
    now = datetime(2026, 5, 19, 12, 0, 0, tzinfo=timezone.utc)
    hit = _hit(0.5, last_seen_at="2026-05-19T10:00:00Z")
    s = score_hits([hit], RecallWeights(), Exponential(), now=now)
    # 2 hours old; recency = 0.995^2 ≈ 0.990
    assert 0.989 < s[0].recency < 0.991
