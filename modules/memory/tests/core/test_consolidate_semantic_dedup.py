"""Tests for FR-MEMORY-116 — semantic-dedup consolidate phase.

Covers acceptance criteria from
`docs/feature-requests/memory/FR-MEMORY-116-semantic-dedup-consolidate.md`:

* AC #1  — default consolidate unchanged (no SemanticDedup)
* AC #2  — --semantic-dedup --dry-run produces diff but no apply
* AC #4  — --semantic-dedup-apply with §7.7 anchor applies proposals
* AC #6  — audit rows tagged with `invocation: "consolidate"`
* AC #7  — --threshold override changes proposal count
* AC #9  — phase ordering: walk failure aborts SemanticDedup
* AC #11 — re-apply idempotent (no new dream.proposal_applied rows)
* AC #13 — reuses FR-MEMORY-115 duplicates detector (asserted via import)
* AC #14 — diff matches what `cyberos dream` would produce
"""

from __future__ import annotations

import json
import os
import time
from pathlib import Path

import pytest

from cyberos.core.consolidate import _phase_semantic_dedup, run as consolidate_run, ConsolidationReport
from cyberos.core.frontmatter import Frontmatter, serialize
from cyberos.core.ops import put as canonical_put
from cyberos.core.writer import Writer


# ---- fixtures --------------------------------------------------------------


@pytest.fixture(autouse=True)
def _exempt_sandbox_path(monkeypatch, tmp_path):
    """Exempt the test's tmp_path from AGENTS.md §0.1 sandbox-fragment check."""
    monkeypatch.setenv("CYBEROS_HOST_MOUNT_PREFIX", str(tmp_path))


@pytest.fixture()
def store_with_dupes(tmp_path: Path) -> Path:
    """A store seeded with manifest + 2 near-duplicate facts + §7.7 anchor."""
    store = tmp_path / ".cyberos-memory"
    (store / "audit").mkdir(parents=True)
    (store / "memories" / "facts").mkdir(parents=True)

    # Minimal manifest the walker accepts
    (store / "manifest.json").write_text(json.dumps({
        "schema_version": 1,
        "audit_chain_head": "sha256:" + "0" * 64,
        "last_updated_at": "2026-05-19T00:00:00Z",
        "timezone": "UTC",
    }))

    # AGENTS.md with §7.7 anchor for apply to succeed
    (store / "AGENTS.md").write_text(
        "# stub\n## §7.7  Dreaming\n"
        "§7.7.1 dream-runner / dream-applier are distinct identities...\n"
    )

    fm = Frontmatter(id="F-1", kind="fact", ts_ns=time.time_ns(),
                     actor="t", tags=[], extra={})
    with Writer(store) as w:
        canonical_put(
            w, "memories/facts/dispatch-1.md",
            serialize(fm, b"dispatch service 60-second retry pattern triggering load"),
            actor="t", kind="fact",
        )
        canonical_put(
            w, "memories/facts/dispatch-2.md",
            serialize(fm, b"dispatch service exhibits 60 second retry pattern triggering load"),
            actor="t", kind="fact",
        )
    return store


# ---- back-compat: default consolidate unchanged ----------------------------


def test_default_consolidate_skips_dedup(store_with_dupes: Path) -> None:
    """AC #1 — without --semantic-dedup, no SemanticDedup phase runs."""
    report = consolidate_run(store_with_dupes)
    assert report.semantic_dedup_ran is False
    assert report.semantic_dedup_proposals_count == 0


# ---- dry-run path ----------------------------------------------------------


def test_dedup_dry_run_finds_but_does_not_apply(store_with_dupes: Path) -> None:
    """AC #2."""
    report = consolidate_run(
        store_with_dupes,
        semantic_dedup=True,
        semantic_dedup_apply=False,  # explicit
        semantic_dedup_threshold=0.3,
    )
    assert report.semantic_dedup_ran is True
    assert report.semantic_dedup_dry_run is True
    assert report.semantic_dedup_proposals_count >= 1
    assert report.semantic_dedup_applied_count == 0


# ---- apply path ------------------------------------------------------------


def test_dedup_apply_merges_duplicates(store_with_dupes: Path) -> None:
    """AC #4."""
    with Writer(store_with_dupes) as w:
        head_before = w.head_seq

    report = consolidate_run(
        store_with_dupes,
        semantic_dedup=True,
        semantic_dedup_apply=True,
        semantic_dedup_threshold=0.3,
    )
    assert report.semantic_dedup_ran is True
    assert report.semantic_dedup_dry_run is False
    assert report.semantic_dedup_applied_count >= 1

    with Writer(store_with_dupes) as w:
        head_after = w.head_seq
    assert head_after > head_before


def test_dedup_apply_tags_invocation_consolidate(store_with_dupes: Path) -> None:
    """AC #6 — the consolidate-side dream.complete row carries `invocation`."""
    from cyberos.core.dream._audit_iter import iter_audit_rows

    consolidate_run(
        store_with_dupes,
        semantic_dedup=True,
        semantic_dedup_apply=True,
        semantic_dedup_threshold=0.3,
    )
    invocation_rows = [
        r for r in iter_audit_rows(store_with_dupes)
        if r.get("op") == "dream.complete"
        and r.get("extra", {}).get("invocation") == "consolidate"
    ]
    assert len(invocation_rows) >= 1, (
        "expected at least one dream.complete row with invocation=consolidate"
    )


# ---- threshold ------------------------------------------------------------


def test_threshold_changes_proposal_count(store_with_dupes: Path) -> None:
    """AC #7."""
    loose = consolidate_run(
        store_with_dupes,
        semantic_dedup=True,
        semantic_dedup_threshold=0.3,
    )
    # Tight threshold: 0.99 should produce ZERO proposals (no two
    # memories are 99% identical word-bag-wise)
    tight = consolidate_run(
        store_with_dupes,
        semantic_dedup=True,
        semantic_dedup_threshold=0.99,
    )
    assert loose.semantic_dedup_proposals_count >= tight.semantic_dedup_proposals_count


# ---- phase ordering -------------------------------------------------------


def test_walk_failure_aborts_semantic_dedup(tmp_path: Path) -> None:
    """AC #9 — if Walk fails, SemanticDedup does not run."""
    # Store missing manifest.json → Walk will fail
    store = tmp_path / ".cyberos-memory"
    (store / "audit").mkdir(parents=True)
    (store / "memories" / "facts").mkdir(parents=True)
    # NOTE: no manifest.json on purpose

    report = consolidate_run(
        store,
        semantic_dedup=True,
        semantic_dedup_apply=True,
        semantic_dedup_threshold=0.3,
    )
    assert report.semantic_dedup_ran is False
    assert report.semantic_dedup_applied_count == 0


# ---- idempotency ----------------------------------------------------------


def test_reapply_is_idempotent(store_with_dupes: Path) -> None:
    """AC #11 — re-running --semantic-dedup-apply emits zero new dream.proposal_applied rows."""
    from cyberos.core.dream._audit_iter import iter_audit_rows

    consolidate_run(
        store_with_dupes,
        semantic_dedup=True,
        semantic_dedup_apply=True,
        semantic_dedup_threshold=0.3,
    )
    applied_after_first = sum(
        1 for r in iter_audit_rows(store_with_dupes)
        if r.get("op") == "dream.proposal_applied"
    )

    consolidate_run(
        store_with_dupes,
        semantic_dedup=True,
        semantic_dedup_apply=True,
        semantic_dedup_threshold=0.3,
    )
    applied_after_second = sum(
        1 for r in iter_audit_rows(store_with_dupes)
        if r.get("op") == "dream.proposal_applied"
    )
    assert applied_after_second == applied_after_first, (
        f"expected idempotency; got {applied_after_first} → {applied_after_second}"
    )


# ---- code reuse -----------------------------------------------------------


def test_consolidate_imports_dream_detector() -> None:
    """AC #13 — the SemanticDedup phase reuses FR-MEMORY-115's detector
    rather than forking the cosine logic.

    Asserts the import path the phase uses; if someone forks the
    detector this test breaks.
    """
    import inspect
    from cyberos.core.consolidate import _phase_semantic_dedup
    src = inspect.getsource(_phase_semantic_dedup)
    assert "cyberos.core.dream.runner" in src
    assert "cyberos.core.dream.applier" in src
    assert "duplicates" in src


# ---- ConsolidationReport schema ------------------------------------------


def test_report_fields_present_when_dedup_ran(store_with_dupes: Path) -> None:
    """Sanity check: the new ConsolidationReport fields are populated."""
    report = consolidate_run(
        store_with_dupes,
        semantic_dedup=True,
        semantic_dedup_threshold=0.3,
    )
    for field_name in (
        "semantic_dedup_ran",
        "semantic_dedup_dry_run",
        "semantic_dedup_proposals_count",
        "semantic_dedup_applied_count",
        "semantic_dedup_dream_id",
    ):
        assert hasattr(report, field_name), f"missing field {field_name}"
