"""Tests for TASK-MEMORY-105 (watched-folder invariants) +
TASK-MEMORY-110 (daemon heartbeat) + TASK-MEMORY-111 (pre-ingest PII gate)."""

from __future__ import annotations

import json
import time
from pathlib import Path

import pytest

# ---------------------------------------------------------------------------
# TASK-MEMORY-105 — watched-folder invariants
# ---------------------------------------------------------------------------

from cyberos.core.watched_folders import (
    WatchedFolder,
    WatchedFolderError,
    check_invariants,
    doctor_check,
    load_watched_folders,
)


def _wf(path: str, **kw) -> WatchedFolder:
    return WatchedFolder(
        path=Path(path),
        added_at_ns=kw.get("added_at_ns", 0),
        include_globs=tuple(kw.get("include_globs", ("**/*",))),
        exclude_globs=tuple(kw.get("exclude_globs", ())),
        sync_class_default=kw.get("sync_class_default", "private"),
    )


def test_load_returns_empty_when_file_absent(tmp_path: Path) -> None:
    assert load_watched_folders(tmp_path) == []


def test_load_round_trips(tmp_path: Path) -> None:
    (tmp_path / "watched_folders.json").write_text(json.dumps({
        "version": 1,
        "folders": [
            {"path": str(tmp_path), "added_at_ns": 42},
        ],
    }))
    out = load_watched_folders(tmp_path)
    assert len(out) == 1
    assert out[0].added_at_ns == 42


def test_invariant_relative_path_fails(tmp_path: Path) -> None:
    errs = check_invariants([_wf("relative/path")])
    assert any("not absolute" in str(e) for e in errs)


def test_invariant_missing_path_fails(tmp_path: Path) -> None:
    errs = check_invariants([_wf(str(tmp_path / "does-not-exist"))])
    assert any("does not exist" in str(e) for e in errs)


def test_invariant_duplicate_fails(tmp_path: Path) -> None:
    errs = check_invariants([_wf(str(tmp_path)), _wf(str(tmp_path))])
    assert any("listed twice" in str(e) for e in errs)


def test_invariant_nesting_fails(tmp_path: Path) -> None:
    child = tmp_path / "sub"
    child.mkdir(parents=True, exist_ok=True)
    errs = check_invariants([_wf(str(tmp_path)), _wf(str(child))])
    assert any("nested inside" in str(e) for e in errs)


def test_invariant_bad_sync_class_fails(tmp_path: Path) -> None:
    errs = check_invariants([_wf(str(tmp_path), sync_class_default="public")])
    assert any("sync_class_default" in str(e) for e in errs)


def test_invariant_toxic_root_fails() -> None:
    errs = check_invariants([_wf("/")])
    assert any("filesystem root" in str(e) for e in errs)


def test_invariant_toxic_prefix_fails() -> None:
    errs = check_invariants([_wf("/etc/cyberos")])
    assert any("toxic location" in str(e) for e in errs)


def test_doctor_check_returns_errors(tmp_path: Path) -> None:
    (tmp_path / "watched_folders.json").write_text(json.dumps({
        "version": 1,
        "folders": [{"path": "/", "added_at_ns": 0}],
    }))
    errs = doctor_check(tmp_path)
    assert errs
    assert any("filesystem root" in str(e) for e in errs)


# ---------------------------------------------------------------------------
# TASK-MEMORY-110 — daemon heartbeat / supervisor
# ---------------------------------------------------------------------------

from cyberos.core.daemon_health import (
    HeartbeatProbe,
    HeartbeatWriter,
    RestartPolicy,
    Supervisor,
    daemon_status,
)


def test_heartbeat_writer_creates_file(tmp_path: Path) -> None:
    w = HeartbeatWriter(store=tmp_path, pid=42)
    # Force a fresh write by clearing the throttle.
    w.last_write_ns = 0
    w.heartbeat()
    p = w.path()
    assert p.exists()
    data = json.loads(p.read_text())
    assert data["pid"] == 42


def test_probe_returns_false_when_absent(tmp_path: Path) -> None:
    assert HeartbeatProbe(store=tmp_path).is_alive() is False


def test_probe_returns_true_for_fresh_beat(tmp_path: Path) -> None:
    w = HeartbeatWriter(store=tmp_path, pid=123)
    w.last_write_ns = 0
    w.heartbeat()
    assert HeartbeatProbe(store=tmp_path).is_alive() is True


def test_probe_returns_false_for_stale_beat(tmp_path: Path) -> None:
    w = HeartbeatWriter(store=tmp_path, pid=123)
    w.last_write_ns = 0
    w.heartbeat()
    # Forge an old ts.
    p = w.path()
    payload = json.loads(p.read_text())
    payload["ts_ns"] = int((time.time() - 600) * 1_000_000_000)
    p.write_text(json.dumps(payload))
    assert HeartbeatProbe(store=tmp_path, stale_secs=60).is_alive() is False


def test_restart_policy_backoff_grows() -> None:
    pol = RestartPolicy(base_secs=1.0, factor=2.0, cap_secs=10.0)
    assert pol.delay_for_attempt(0) == 1.0
    assert pol.delay_for_attempt(1) == 2.0
    assert pol.delay_for_attempt(2) == 4.0
    assert pol.delay_for_attempt(3) == 8.0
    assert pol.delay_for_attempt(4) == 10.0  # cap


def test_supervisor_records_and_resets() -> None:
    sup = Supervisor(store=Path("/tmp/x"))
    sup.record_crash("OOM")
    sup.record_crash("segfault")
    assert sup.attempt == 2
    sup.succeed()
    assert sup.attempt == 0


def test_supervisor_hit_ceiling() -> None:
    sup = Supervisor(store=Path("/tmp/x"), policy=RestartPolicy(max_attempts=2))
    sup.record_crash("a")
    sup.record_crash("b")
    assert sup.hit_ceiling() is True


def test_daemon_status_absent(tmp_path: Path) -> None:
    s = daemon_status(tmp_path)
    assert s["state"] == "absent"


def test_daemon_status_healthy(tmp_path: Path) -> None:
    w = HeartbeatWriter(store=tmp_path, pid=99)
    w.last_write_ns = 0
    w.heartbeat()
    s = daemon_status(tmp_path)
    assert s["state"] == "healthy"
    assert s["pid"] == 99


# ---------------------------------------------------------------------------
# TASK-MEMORY-111 — pre-ingest PII gate
# ---------------------------------------------------------------------------

from cyberos.core.pre_ingest_pii import (
    PiiBlockedError,
    PiiReport,
    pre_ingest_check,
    scan_pii,
)


def test_scan_detects_email() -> None:
    r = scan_pii("contact me at alice@example.com please", policy="log")
    assert any(h.kind == "email" for h in r.hits)


def test_scan_detects_cccd() -> None:
    r = scan_pii("CCCD 012345678901 expires soon", policy="log")
    assert any(h.kind == "cccd" for h in r.hits)


def test_scan_detects_mst_with_branch() -> None:
    r = scan_pii("invoice for MST 0123456789-001", policy="log")
    assert any(h.kind == "mst" for h in r.hits)


def test_scan_detects_e164_phone() -> None:
    r = scan_pii("call +84906878091 today", policy="log")
    assert any(h.kind == "phone" for h in r.hits)


def test_block_policy_raises() -> None:
    with pytest.raises(PiiBlockedError) as exc:
        scan_pii("email me at bob@example.com", policy="block")
    assert any(h.kind == "email" for h in exc.value.report.hits)


def test_redact_policy_replaces_with_sentinel() -> None:
    r = scan_pii("bob@example.com is the one", policy="redact")
    assert r.redacted_body == "[EMAIL] is the one"


def test_allowlist_drops_specific_kind() -> None:
    r = scan_pii("MST 0123456789 plus alice@example.com", policy="log", allowlist=("mst",))
    kinds = {h.kind for h in r.hits}
    assert "mst" not in kinds
    assert "email" in kinds


def test_pre_ingest_check_reads_frontmatter_policy() -> None:
    fm = {"pii_policy": "redact"}
    r = pre_ingest_check("memo.md", "send to alice@example.com", frontmatter=fm)
    assert r.redacted_body == "send to [EMAIL]"


def test_pre_ingest_check_reads_frontmatter_allowlist() -> None:
    fm = {"pii_allowlist": ["email"]}
    r = pre_ingest_check("memo.md", "hi bob@example.com", frontmatter=fm, default_policy="log")
    assert not r.has_hits
