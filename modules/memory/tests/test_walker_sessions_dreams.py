"""
Walker hardening (TASK-MEMORY-303 §1.4 / §1.5, AC 4 + AC 5).

* ``sessions/`` (§18.2) and ``dreams/`` (§7.7.4) are canonical top-level
  dirs — a store exercising the protocol's own features stays doctor-green.
* The three declared-but-missing invariants exist and bite: each of the
  three constructed violating fixtures fails exactly its own invariant,
  and a clean store passes all three.
* The §1.5 store repair (stray ``adrs/`` + ``impl-plans/`` relocated via
  canonical ledger-recorded ``move`` ops) is demonstrated on a fixture
  store cloned from the live layout: doctor layout goes red → green and
  the audit chain stays intact. (The live-store execution is
  operator-gated — see the task's store-repair-plan.md; this test is the
  mechanical proof per AC 5's split verification.)
* Sandbox non-interference (audit ISS-006): the ``/sessions/`` entry in
  ``_SANDBOX_FRAGMENTS`` tests the STORE'S OWN path, not entries inside
  it; adding a ``sessions/`` child does not trip it, and a store under a
  path containing ``/sessions/`` remains rejected.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

import pytest

from cyberos.core.invariants import (
    check_dream_applied_provenance,
    check_layout_no_sandbox_path,
    check_layout_root_canonical,
    check_session_lifecycle,
    check_store_yaml_acl_valid,
)
from cyberos.core.ops import move
from cyberos.core.transcript import _append_frame, _locate_binlog, append, end, start
from cyberos.core.walker import verify_segments
from cyberos.core.writer import _GENESIS_CHAIN, AuditRecord, Writer


@pytest.fixture(autouse=True)
def _exempt_sandbox_path(monkeypatch, tmp_path):
    """Exempt the test's tmp_path from AGENTS.md §0.1 sandbox-fragment check."""
    monkeypatch.setenv("CYBEROS_HOST_MOUNT_PREFIX", str(tmp_path))


def _init_store(tmp_path: Path, name: str = "store") -> Path:
    store = tmp_path / ".cyberos" / "memory" / name
    (store / "audit").mkdir(parents=True)
    (store / "memories" / "facts").mkdir(parents=True)
    (store / "manifest.json").write_text(json.dumps({"schema_version": 1}))
    (store / "AGENTS.md").write_text(
        "# stub\n## §18  Session transcript ledger\n"
    )
    return store


def _shard(filename: str) -> tuple[str, str]:
    """The canonical two-level hex bucket for a memory filename."""
    sha = hashlib.sha256(filename.encode("utf-8")).hexdigest()
    return sha[0:2], sha[2:4]


# ---------------------------------------------------------------------------
# AC 4 — allowlist + the three invariants
# ---------------------------------------------------------------------------


def test_new_invariants_pass_and_fail_correctly(tmp_path: Path) -> None:
    """AC 4 — sessions/ + dreams/ pass layout; three violating fixtures
    each fail exactly their invariant; a clean store passes all three."""
    # --- seeded store with the protocol's own feature dirs passes layout ---
    seeded = _init_store(tmp_path, "seeded")
    (seeded / "sessions").mkdir()
    (seeded / "dreams" / "20260723T000000Z").mkdir(parents=True)
    passed, details = check_layout_root_canonical(seeded)
    assert passed, f"sessions/ + dreams/ must be canonical: {details}"

    # --- clean store passes all three new invariants ---
    clean = _init_store(tmp_path, "clean")
    with Writer(clean) as w:
        sess = start(w, session_id="clean-sess")
        append(w, session_id="clean-sess", role="user", content="hi")
        append(w, session_id="clean-sess", role="assistant", content="hello")
        end(w, session_id="clean-sess", seal_binlog=False)
        # A well-formed dream-applier pair: mutation row + aux row.
        w.submit(AuditRecord(
            op="put", path="memories/facts/f.md", actor="dream-applier",
            content_sha256="0" * 64,
            extra={"kind": "fact", "dream_id": "01D", "proposal_id": "P1"},
        ))
        w.submit(AuditRecord(
            op="dream.proposal_applied", path="memories/facts/f.md",
            actor="dream-applier",
            extra={"dream_id": "01D", "proposal_id": "P1"},
        ))
    (clean / "memories" / "STORE.yaml").write_text(
        "store_id: clean-store\n"
        "default_mode: read-write\n"
        "acl:\n"
        "  - actor: '*'\n"
        "    mode: read-write\n"
    )
    for check in (
        check_dream_applied_provenance,
        check_store_yaml_acl_valid,
        check_session_lifecycle,
    ):
        passed, details = check(clean)
        assert passed, f"clean store failed {check.__name__}: {details}"

    # --- fixture A: dream row missing proposal_id ---
    fix_a = _init_store(tmp_path, "fix-a")
    with Writer(fix_a) as w:
        w.submit(AuditRecord(
            op="dream.proposal_applied", path="memories/facts/a.md",
            actor="dream-applier",
            extra={"dream_id": "01DREAM"},          # proposal_id missing
        ))
    passed, details = check_dream_applied_provenance(fix_a)
    assert not passed and "proposal_id" in details, details
    for other in (check_store_yaml_acl_valid, check_session_lifecycle):
        passed, details = other(fix_a)
        assert passed, f"fixture A tripped {other.__name__}: {details}"

    # --- fixture B: malformed STORE.yaml ---
    fix_b = _init_store(tmp_path, "fix-b")
    (fix_b / "memories" / "STORE.yaml").write_text(
        "default_mode: write-anything\n"            # bad enum, no store_id,
        "acl:\n"                                     # bad entry shape
        "  - actor: ''\n"
        "    mode: admin\n"
        "extra_key: nope\n"
    )
    passed, details = check_store_yaml_acl_valid(fix_b)
    assert not passed and "STORE.yaml" in details, details
    for other in (check_dream_applied_provenance, check_session_lifecycle):
        passed, details = other(fix_b)
        assert passed, f"fixture B tripped {other.__name__}: {details}"

    # --- fixture C: session turn appended after session.end ---
    fix_c = _init_store(tmp_path, "fix-c")
    with Writer(fix_c) as w:
        start(w, session_id="sess-c")
        append(w, session_id="sess-c", role="user", content="hi")
        end(w, session_id="sess-c", seal_binlog=False)
    binlog = _locate_binlog(fix_c, "sess-c")
    assert binlog is not None
    _append_frame(                                   # rogue turn, ts after end
        binlog,
        json.dumps({
            "session_id": "sess-c", "role": "user",
            "turn_seq": 1, "content": "zombie turn",
        }).encode("utf-8"),
        turn_seq=1,
    )
    passed, details = check_session_lifecycle(fix_c)
    assert not passed and "follows session.end" in details, details
    for other in (check_dream_applied_provenance, check_store_yaml_acl_valid):
        passed, details = other(fix_c)
        assert passed, f"fixture C tripped {other.__name__}: {details}"


def test_sessions_allowlist_does_not_touch_sandbox_check(
    tmp_path: Path, monkeypatch,
) -> None:
    """ISS-006 — `/sessions/` in `_SANDBOX_FRAGMENTS` checks the store's
    OWN path, not its children (and vice versa)."""
    # A store WITH a sessions/ child dir passes both layout and (exempted)
    # sandbox checks — the child does not trip the path-fragment test.
    store = _init_store(tmp_path, "with-sessions-child")
    (store / "sessions").mkdir()
    passed, details = check_layout_root_canonical(store)
    assert passed, details
    passed, details = check_layout_no_sandbox_path(store)
    assert passed, details

    # A store UNDER a /sessions/ path segment remains rejected as before
    # once the exemption is lifted — the fragment check still fires.
    monkeypatch.delenv("CYBEROS_HOST_MOUNT_PREFIX", raising=False)
    nested = tmp_path / "sessions" / "store"
    nested.mkdir(parents=True)
    passed, details = check_layout_no_sandbox_path(nested)
    assert not passed and "sessions" in details, details


# ---------------------------------------------------------------------------
# AC 5 (fixture leg) — stray-dir relocation preserves the chain
# ---------------------------------------------------------------------------


def test_repair_fixture_relocation_preserves_chain(tmp_path: Path) -> None:
    """AC 5 — on a fixture store cloned from the live layout, the §1.5
    repair (canonical `move` ops + empty-dir removal) turns
    layout-root-canonical red → green with the chain intact."""
    store = _init_store(tmp_path, "live-clone")
    (store / "memories" / "decisions").mkdir(parents=True)
    (store / "memories" / "projects").mkdir(parents=True)
    # The live store's two strays, synthetic-equivalent bodies.
    adr_body = b"---\ntemplate: architecture-decision-record@1\n---\n# ADR-0001\n"
    plan_body = b"---\ntemplate: impl_plan@1\n---\n## Implementation Plan\n"
    (store / "adrs").mkdir()
    (store / "adrs" / "ADR-0001-untitled.md").write_bytes(adr_body)
    (store / "impl-plans").mkdir()
    (store / "impl-plans" / "impl-plan-untitled.md").write_bytes(plan_body)

    # BEFORE: layout red, both strays named.
    passed, details = check_layout_root_canonical(store)
    assert not passed and "adrs/" in details and "impl-plans/" in details, details

    # THE REPAIR — canonical ledger-recorded moves (same op sequence the
    # operator-gated store-repair-plan.md specifies for the live store).
    a1, a2 = _shard("ADR-0001-untitled.md")
    p1, p2 = _shard("impl-plan-untitled.md")
    adr_dst = f"memories/decisions/{a1}/{a2}/ADR-0001-untitled.md"
    plan_dst = f"memories/projects/{p1}/{p2}/impl-plan-untitled.md"
    with Writer(store) as w:
        move(w, "adrs/ADR-0001-untitled.md", adr_dst, actor="operator-repair")
        move(w, "impl-plans/impl-plan-untitled.md", plan_dst,
             actor="operator-repair")
    (store / "adrs").rmdir()          # empty after the move — plan step 3
    (store / "impl-plans").rmdir()

    # AFTER: layout green; bodies intact at the canonical homes.
    passed, details = check_layout_root_canonical(store)
    assert passed, f"layout still red after repair: {details}"
    assert (store / adr_dst).read_bytes() == adr_body
    assert (store / plan_dst).read_bytes() == plan_body

    # Chain intact: exactly the two move rows, LINK+HASH verified.
    current = store / "audit" / "current.binlog"
    n = verify_segments([current], start_prev=_GENESIS_CHAIN)
    assert n == 2, f"expected exactly the 2 relocation rows, got {n}"

    from cyberos.core.dream._audit_iter import iter_audit_rows
    rows = list(iter_audit_rows(store))
    assert [r["op"] for r in rows] == ["move", "move"]
    assert rows[0]["path"] == "adrs/ADR-0001-untitled.md"
    assert rows[0]["extra"]["to"] == adr_dst
    assert rows[0]["content_sha256"] == hashlib.sha256(adr_body).hexdigest()
    assert rows[1]["path"] == "impl-plans/impl-plan-untitled.md"
    assert rows[1]["extra"]["to"] == plan_dst
    assert rows[1]["content_sha256"] == hashlib.sha256(plan_body).hexdigest()
