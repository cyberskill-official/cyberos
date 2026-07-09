"""Test suite for FR-CUO-200 — harness read-only report.

7 ACs covered. Each test function corresponds to one AC.
"""

from __future__ import annotations

import json
import os
import textwrap
import time
from datetime import datetime, timedelta, timezone
from pathlib import Path

import pytest

from cuo.core.harness import (
    HarnessReport,
    SignalBreach,
    WorkflowMetrics,
    compute_report,
    emit_report,
    format_markdown,
    parse_skill_signals,
    parse_window,
)


# ----------------------------------------------------------------------------
# Fixtures
# ----------------------------------------------------------------------------


@pytest.fixture()
def skill_root(tmp_path: Path) -> Path:
    """Build a minimal skill_root with two skills declaring self_audit signals."""
    sr = tmp_path / "skill"
    sr.mkdir()

    # Skill A — declares user_correction_streak: 2 and acceptance_rate_below: 0.6
    a = sr / "feature-request-audit"
    a.mkdir()
    (a / "SKILL.md").write_text(textwrap.dedent("""\
        ---
        name: feature-request-audit
        description: stub
        license: Apache-2.0
        metadata:
          version: 1.0.0
        self_audit:
          invariants_ref: ./INVARIANTS.md
          check_at:
            - on_completion
          anomaly_signals:
            user_correction_streak: {threshold: 2, window: 10}
            confidence_low_streak: {threshold: 3, window: 10}
        human_fine_tune:
          signals_to_initiate:
            - acceptance_rate_below: 0.6
            - hitl_pause_rate_above: 0.4
        ---

        # body
    """), encoding="utf-8")

    # Skill B — declares deterministic_drift: 1 only
    b = sr / "code-review-author"
    b.mkdir()
    (b / "SKILL.md").write_text(textwrap.dedent("""\
        ---
        name: code-review-author
        description: stub
        license: Apache-2.0
        metadata:
          version: 1.0.0
        self_audit:
          anomaly_signals:
            deterministic_drift: {threshold: 1, window: 1}
        ---

        # body
    """), encoding="utf-8")

    return sr


def _row(op: str, ts_ns: int, **extra) -> dict:
    """Build a minimal audit-row dict."""
    return {
        "op": op,
        "ts_ns": ts_ns,
        "seq": extra.pop("seq", 0),
        "row_id": extra.pop("row_id", f"row-{op}-{ts_ns}"),
        "extra": extra,
    }


# ----------------------------------------------------------------------------
# AC #1 — `harness report --since 7d` produces a non-empty markdown file
# ----------------------------------------------------------------------------


def test_report_emits_markdown(skill_root: Path, tmp_path: Path) -> None:
    """AC #1: report file is non-empty + structurally valid."""
    out_path = tmp_path / "harness-report-2026-05-19.md"
    report = compute_report(
        audit_dir=None,
        skill_root=skill_root,
        window=parse_window("7d"),
        rows_override=[],
    )
    written = emit_report(report, out_path)
    assert written.is_file()
    body = written.read_text(encoding="utf-8")
    assert "# Harness report" in body
    assert "## Skills with tripped signals" in body
    assert "## Workflows with elevated rework" in body
    assert "## Per-FR routed-back history" in body
    assert "## Summary" in body
    assert len(body) > 200


# ----------------------------------------------------------------------------
# AC #2 — seeded chain with 11 fr_routed_back rows trips acceptance_rate_below
# ----------------------------------------------------------------------------


def test_signal_thresholds_trip_correctly(skill_root: Path) -> None:
    """AC #2/#3: tripped signal carries skill/signal/value/threshold/evidence."""
    now_ns = int(time.time() * 1_000_000_000)
    rows: list[dict] = []
    # Seed 11 fr_routed_back rows for the skill — all terminals are non-done.
    for i in range(11):
        rows.append(_row(
            "memory.fr_routed_back", now_ns - i * 1_000_000_000,
            skill="feature-request-audit",
            fr_id=f"FR-X-{i:03d}",
            outcome="ROUTED_BACK",
            row_id=f"rb-{i}",
        ))
    # Plus 1 done row → done_rate = 1/12 = 8.3%, below 0.6 → trips
    rows.append(_row(
        "memory.fr_completed", now_ns,
        skill="feature-request-audit",
        fr_id="FR-X-999",
        outcome="done",
        row_id="rc-1",
    ))

    report = compute_report(
        audit_dir=None,
        skill_root=skill_root,
        window=timedelta(days=7),
        rows_override=rows,
    )
    acceptance_breaches = [
        b for b in report.breaches
        if b.signal_id == "acceptance_rate_below"
        and b.skill_name == "feature-request-audit"
    ]
    assert len(acceptance_breaches) == 1
    breach = acceptance_breaches[0]
    assert breach.value < 0.6
    assert breach.threshold == 0.6
    assert len(breach.evidence_row_ids) > 0


# ----------------------------------------------------------------------------
# AC #4 — Workflow rework rate section sorts descending
# ----------------------------------------------------------------------------


def test_workflow_rework_rate(skill_root: Path) -> None:
    """AC #4: workflows listed by rework rate descending."""
    now_ns = int(time.time() * 1_000_000_000)
    rows: list[dict] = []
    # Workflow A: 1 completed + 4 routed_back → rework_rate 80%
    for i in range(4):
        rows.append(_row("workflow_complete", now_ns - i * 1_000_000,
                         workflow_id="cto/ship-feature-requests",
                         outcome="ROUTED_BACK",
                         row_id=f"wfa-rb-{i}"))
    rows.append(_row("workflow_complete", now_ns,
                     workflow_id="cto/ship-feature-requests",
                     outcome="COMPLETED",
                     row_id="wfa-ok-1"))
    # Workflow B: 3 completed + 1 routed_back → rework_rate 25%
    for i in range(3):
        rows.append(_row("workflow_complete", now_ns - (10 + i) * 1_000_000,
                         workflow_id="ceo/quarterly-review",
                         outcome="COMPLETED",
                         row_id=f"wfb-ok-{i}"))
    rows.append(_row("workflow_complete", now_ns - 9_000_000,
                     workflow_id="ceo/quarterly-review",
                     outcome="ROUTED_BACK",
                     row_id="wfb-rb-1"))

    report = compute_report(
        audit_dir=None,
        skill_root=skill_root,
        window=timedelta(days=7),
        rows_override=rows,
    )
    assert len(report.workflow_metrics) >= 2
    # First entry has the highest rework rate
    assert report.workflow_metrics[0].rework_rate >= report.workflow_metrics[1].rework_rate
    # Verify it sorted to put cto/ship-feature-requests first
    assert report.workflow_metrics[0].workflow_id == "cto/ship-feature-requests"


# ----------------------------------------------------------------------------
# AC #5 — Atomic write in watch mode (write-to-temp then rename)
# ----------------------------------------------------------------------------


def test_watch_mode_atomic_write(skill_root: Path, tmp_path: Path) -> None:
    """AC #5: emit_report uses write-to-temp + rename — no truncation."""
    out_path = tmp_path / "out" / "harness-report.md"
    report1 = compute_report(audit_dir=None, skill_root=skill_root,
                             window=timedelta(days=7), rows_override=[])
    emit_report(report1, out_path)
    body1 = out_path.read_text(encoding="utf-8")
    size1 = out_path.stat().st_size

    # Second emission overwrites atomically (no .tmp leftover)
    now_ns = int(time.time() * 1_000_000_000)
    rows = [_row("workflow_complete", now_ns,
                 workflow_id="cto/ship-feature-requests",
                 outcome="COMPLETED", row_id="x-1")]
    report2 = compute_report(audit_dir=None, skill_root=skill_root,
                             window=timedelta(days=7), rows_override=rows)
    emit_report(report2, out_path)
    body2 = out_path.read_text(encoding="utf-8")

    assert body1 != body2  # content changed
    assert not (tmp_path / "out" / "harness-report.md.tmp").exists()
    # File is well-formed (no truncation)
    assert body2.startswith("# Harness report")
    assert "## Summary" in body2


# ----------------------------------------------------------------------------
# AC #6 — Per run, exactly one harness.report_emitted memory aux row
# ----------------------------------------------------------------------------


def test_emits_audit_row(skill_root: Path, tmp_path: Path, monkeypatch) -> None:
    """AC #6: exactly one `harness.report_emitted` audit row per emit_report call."""
    # Set up a real memory root so the emit happens through the Writer.
    memory_root = tmp_path / ".cyberos/memory/store"
    (memory_root / "audit").mkdir(parents=True)
    (memory_root / "memories").mkdir(parents=True)
    (memory_root / "manifest.json").write_text(json.dumps({
        "schema_version": 1,
        "audit_chain_head": "sha256:" + "0" * 64,
        "last_updated_at": "2026-05-19T00:00:00Z",
        "timezone": "UTC",
    }))
    monkeypatch.setenv("CYBEROS_HOST_MOUNT_PREFIX", str(tmp_path))

    out_path = tmp_path / "report.md"
    report = compute_report(audit_dir=None, skill_root=skill_root,
                            window=timedelta(days=7), rows_override=[])
    emit_report(report, out_path, memory_root=memory_root)

    # Walk the binlog and confirm exactly one harness.report_emitted row
    try:
        from cyberos.core.harness import load_audit_rows
        rows = load_audit_rows(memory_root / "audit")
    except ImportError:
        pytest.skip("cyberos.core not importable in this env")
    emitted = [r for r in rows if r.get("op") == "harness.report_emitted"]
    assert len(emitted) == 1
    payload = emitted[0].get("extra") or {}
    assert "report_path" in payload
    assert "skills_with_signals" in payload
    assert "workflows_with_signals" in payload
    assert "evidence_row_count" in payload


# ----------------------------------------------------------------------------
# AC #7 — Empty chain produces a report with all sections present but empty
# ----------------------------------------------------------------------------


def test_empty_chain_clean_exit(skill_root: Path, tmp_path: Path) -> None:
    """AC #7: zero rows → all sections present but empty, no crash."""
    out_path = tmp_path / "empty-report.md"
    report = compute_report(audit_dir=None, skill_root=skill_root,
                            window=timedelta(days=7), rows_override=[])
    emit_report(report, out_path)
    body = out_path.read_text(encoding="utf-8")
    # All 4 sections present
    assert "## Skills with tripped signals" in body
    assert "## Workflows with elevated rework" in body
    assert "## Per-FR routed-back history" in body
    assert "## Summary" in body
    # All empty
    assert "*(no signals tripped in this window)*" in body
    assert "*(no workflow runs in this window)*" in body
    assert "*(no rework events in this window)*" in body
    # Summary still has the structural lines
    assert "Workflow runs: **0**" in body


# ----------------------------------------------------------------------------
# Bonus — duration parser
# ----------------------------------------------------------------------------


def test_parse_window_durations() -> None:
    assert parse_window("24h") == timedelta(hours=24)
    assert parse_window("7d") == timedelta(days=7)
    assert parse_window("4w") == timedelta(weeks=4)
    # malformed → default 7d
    assert parse_window("nonsense") == timedelta(days=7)


# ----------------------------------------------------------------------------
# Bonus — frontmatter parser
# ----------------------------------------------------------------------------


def test_parse_skill_signals(skill_root: Path) -> None:
    skill_md = skill_root / "feature-request-audit" / "SKILL.md"
    signals = parse_skill_signals(skill_md)
    assert "user_correction_streak" in signals
    assert signals["user_correction_streak"]["threshold"] == 2
    assert "acceptance_rate_below" in signals
    assert signals["acceptance_rate_below"]["threshold"] == 0.6
