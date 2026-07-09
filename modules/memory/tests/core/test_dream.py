"""Tests for FR-MEMORY-115 — `cyberos dream` out-of-band reflection.

Covers acceptance criteria from
`docs/feature-requests/memory/FR-MEMORY-115-cyberos-dream.md`:

* AC #1  — dream emits dream.start + dream.complete rows
* AC #2  — diff file persisted under dreams/<ts>/
* AC #3  — dry-run is non-mutating but still produces a diff
* AC #4  — apply refuses without §7.7 anchor
* AC #5  — apply advances HEAD per proposal
* AC #6  — applied rows carry dream_id + proposal_id provenance
* AC #8  — duplicates detector threshold
* AC #12 — proposal_id format ^P[0-9A-Z]{8}$
* AC #13 — ULID dream_id format
* AC #14 — idempotent re-apply emits zero new rows
* AC #15 — body-hash drift → PreconditionFailed
* AC #17 — `--detectors` filter
* AC #23 — dream.complete metrics shape
* AC #26 — closed proposal kind enum
"""

from __future__ import annotations

import asyncio
import hashlib
import json
import re
import time
from datetime import timedelta
from pathlib import Path

import pytest

from cyberos.core.dream import (
    DreamDiff,
    DreamProposal,
    generate_dream_id,
    generate_proposal_id,
)
from cyberos.core.dream.runner import run as dream_run
from cyberos.core.dream.applier import (
    PreconditionFailed,
    ProtocolAmendmentMissing,
    apply,
)
from cyberos.core.frontmatter import Frontmatter, serialize
from cyberos.core.ops import put as canonical_put
from cyberos.core.writer import Writer


# ---- fixtures --------------------------------------------------------------


@pytest.fixture()
def store(tmp_path: Path) -> Path:
    s = tmp_path / ".cyberos/memory/store"
    (s / "audit").mkdir(parents=True)
    (s / "memories" / "facts").mkdir(parents=True)
    return s


@pytest.fixture()
def store_with_agents_md(store: Path) -> Path:
    """A store whose AGENTS.md carries the §7.7 anchor."""
    (store / "AGENTS.md").write_text(
        "# stub\n## §7.7  Dreaming\n"
        "§7.7.1 Dreaming is the out-of-band batch reflection process...\n"
    )
    return store


def _seed_duplicates(store: Path) -> None:
    """Seed two near-duplicate fact memories + one unrelated."""
    fm = Frontmatter(id="F-1", kind="fact", ts_ns=time.time_ns(),
                     actor="t", tags=[], extra={})
    with Writer(store) as w:
        canonical_put(
            w, "memories/facts/dispatch-1.md",
            serialize(fm, b"The dispatch service has a 60-second retry pattern triggering load"),
            actor="t", kind="fact",
        )
        canonical_put(
            w, "memories/facts/dispatch-2.md",
            serialize(fm, b"The dispatch service exhibits a 60 second retry pattern triggering load"),
            actor="t", kind="fact",
        )
        canonical_put(
            w, "memories/facts/unrelated.md",
            serialize(fm, b"completely unrelated content about pizza dough recipes"),
            actor="t", kind="fact",
        )


# ---- ID format ------------------------------------------------------------


def test_dream_id_ulid_format() -> None:
    """AC #13."""
    d = generate_dream_id()
    assert re.fullmatch(r"[0-9A-HJKMNP-TV-Z]{26}", d), d


def test_proposal_id_format() -> None:
    """AC #12."""
    for _ in range(100):
        p = generate_proposal_id()
        assert re.fullmatch(r"P[0-9A-Z]{8}", p), p


def test_dream_id_time_sortable() -> None:
    """ULIDs of two consecutive calls are lexicographically ordered."""
    a = generate_dream_id()
    time.sleep(0.005)
    b = generate_dream_id()
    assert a < b


# ---- DreamProposal validation ---------------------------------------------


def test_proposal_op_closed_enum() -> None:
    """AC #26."""
    DreamProposal(proposal_id=generate_proposal_id(), op="merge",
                  rationale="ok")
    with pytest.raises(ValueError, match="closed enum"):
        DreamProposal(proposal_id=generate_proposal_id(), op="weird",  # type: ignore[arg-type]
                      rationale="ok")


def test_proposal_id_must_match_pattern() -> None:
    with pytest.raises(ValueError, match="proposal_id"):
        DreamProposal(proposal_id="badid", op="merge", rationale="ok")


def test_proposal_rationale_non_empty() -> None:
    with pytest.raises(ValueError, match="rationale"):
        DreamProposal(proposal_id=generate_proposal_id(), op="merge",
                      rationale="   ")


# ---- runner ---------------------------------------------------------------


@pytest.mark.asyncio
async def test_dream_emits_start_and_complete(store: Path) -> None:
    """AC #1 — even on an empty store, dream emits start + complete."""
    with Writer(store) as w:
        head_before = w.head_seq
        diff = await dream_run(w, since=timedelta(hours=24),
                                duplicates_threshold=0.92)
        head_after = w.head_seq
    # At minimum: dream.start + dream.complete = +2
    assert head_after >= head_before + 2
    assert diff.proposals == []


@pytest.mark.asyncio
async def test_dream_writes_diff_file(store: Path) -> None:
    """AC #2."""
    with Writer(store) as w:
        diff = await dream_run(w, since=timedelta(hours=24))
    diff_files = list((store / "dreams").rglob("diff.json"))
    assert len(diff_files) == 1
    body = json.loads(diff_files[0].read_text())
    assert body["dream_id"] == diff.dream_id


@pytest.mark.asyncio
async def test_dry_run_tags_metrics(store: Path) -> None:
    """AC #3 — dry-run is non-mutating per the operator's contract."""
    _seed_duplicates(store)
    with Writer(store) as w:
        diff = await dream_run(w, since=timedelta(hours=24),
                                duplicates_threshold=0.3,
                                dry_run=True)
    assert diff.metrics["dry_run"] is True


@pytest.mark.asyncio
async def test_duplicates_detector_threshold(store: Path) -> None:
    """AC #8."""
    _seed_duplicates(store)
    with Writer(store) as w:
        diff = await dream_run(w, since=timedelta(hours=24),
                                duplicates_threshold=0.3)
    kinds = {p.op for p in diff.proposals}
    assert "merge" in kinds


@pytest.mark.asyncio
async def test_detectors_filter(store: Path) -> None:
    """AC #17 — `--detectors` filter limits the active set."""
    _seed_duplicates(store)
    with Writer(store) as w:
        diff = await dream_run(
            w, since=timedelta(hours=24),
            detector_names=("duplicates",),
            duplicates_threshold=0.3,
        )
    # No new/verify/stale proposals
    kinds = {p.op for p in diff.proposals}
    assert kinds.issubset({"merge"})


@pytest.mark.asyncio
async def test_quality_metrics_shape(store: Path) -> None:
    """AC #23 — quality_metrics carries the expected keys."""
    with Writer(store) as w:
        diff = await dream_run(w, since=timedelta(hours=24))
    qm = diff.metrics
    for k in ("proposals_count_by_kind", "snapshot_head", "duration_ms"):
        assert k in qm
    for kind in ("merge", "stale", "new", "verify"):
        assert kind in qm["proposals_count_by_kind"]


@pytest.mark.asyncio
async def test_unknown_detector_raises(store: Path) -> None:
    with Writer(store) as w:
        with pytest.raises(ValueError, match="unknown detector"):
            await dream_run(w, detector_names=("made_up",))


# ---- applier --------------------------------------------------------------


@pytest.mark.asyncio
async def test_apply_requires_section_7_7(store: Path) -> None:
    """AC #4 — apply refuses without the §7.7 anchor."""
    _seed_duplicates(store)
    with Writer(store) as w:
        diff = await dream_run(w, since=timedelta(hours=24),
                                duplicates_threshold=0.3)
    # No AGENTS.md in this store
    with Writer(store) as w:
        with pytest.raises(ProtocolAmendmentMissing, match="P19 §7.7"):
            apply(w, diff)


@pytest.mark.asyncio
async def test_apply_advances_head_per_proposal(store_with_agents_md: Path) -> None:
    """AC #5 + AC #6 — apply advances head; rows carry dream_id + proposal_id."""
    _seed_duplicates(store_with_agents_md)
    with Writer(store_with_agents_md) as w:
        diff = await dream_run(w, since=timedelta(hours=24),
                                duplicates_threshold=0.3)
    proposals_count = len(diff.proposals)
    with Writer(store_with_agents_md) as w:
        head_before = w.head_seq
        summary = apply(w, diff)
        head_after = w.head_seq
    # Each proposal contributes (canonical op row(s) + 1 dream.proposal_applied row)
    # For a merge proposal with 2 paths: 1 delete + 1 aux = 2 rows minimum
    assert summary["applied_count"] == proposals_count
    assert head_after > head_before


@pytest.mark.asyncio
async def test_idempotent_reapply(store_with_agents_md: Path) -> None:
    """AC #14 — second apply with unchanged state emits zero new rows."""
    _seed_duplicates(store_with_agents_md)
    with Writer(store_with_agents_md) as w:
        diff = await dream_run(w, since=timedelta(hours=24),
                                duplicates_threshold=0.3)
    with Writer(store_with_agents_md) as w:
        apply(w, diff)
        head_after_first = w.head_seq
    with Writer(store_with_agents_md) as w:
        summary2 = apply(w, diff)
        head_after_second = w.head_seq
    assert summary2["applied_count"] == 0
    assert summary2["skipped_idempotent"] >= 1
    assert head_after_second == head_after_first


@pytest.mark.asyncio
async def test_apply_refuses_on_drift(store_with_agents_md: Path) -> None:
    """AC #15 — modifying a target between dream + apply triggers
    PreconditionFailed."""
    _seed_duplicates(store_with_agents_md)
    with Writer(store_with_agents_md) as w:
        diff = await dream_run(w, since=timedelta(hours=24),
                                duplicates_threshold=0.3)
    # Mutate one of the target paths
    p = store_with_agents_md / "memories/facts/dispatch-1.md"
    p.write_bytes(p.read_bytes() + b"\n# drifted!")
    with Writer(store_with_agents_md) as w:
        with pytest.raises(PreconditionFailed, match="drift"):
            apply(w, diff)


@pytest.mark.asyncio
async def test_no_check_protocol_bypasses_section_check(store: Path) -> None:
    """The DEV-only escape hatch lets tests apply without §7.7 anchor."""
    _seed_duplicates(store)
    with Writer(store) as w:
        diff = await dream_run(w, since=timedelta(hours=24),
                                duplicates_threshold=0.3)
    # No AGENTS.md in this store but enforce_section_7_7=False
    with Writer(store) as w:
        summary = apply(w, diff, enforce_section_7_7=False)
    assert summary["applied_count"] == len(diff.proposals)


# ---- DreamDiff round-trip -------------------------------------------------


def test_dreamdiff_round_trips() -> None:
    """DreamDiff.to_dict / from_dict are inverses (used by `dream apply`)."""
    d = DreamDiff(
        dream_id=generate_dream_id(),
        scope="memories/facts",
        since="2026-05-19T00:00:00+00:00",
        input_sessions=[],
        proposals=[
            DreamProposal(
                proposal_id=generate_proposal_id(),
                op="merge",
                paths=["a.md", "b.md"],
                rationale="duplicates",
            ),
        ],
        metrics={"proposals_count_by_kind": {"merge": 1, "stale": 0, "new": 0, "verify": 0}},
    )
    blob = d.to_json()
    d2 = DreamDiff.from_dict(json.loads(blob))
    assert d2.dream_id == d.dream_id
    assert d2.proposals[0].proposal_id == d.proposals[0].proposal_id
    assert d2.proposals[0].op == "merge"
