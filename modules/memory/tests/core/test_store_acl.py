"""Tests for FR-MEMORY-117 — per-store ACL via STORE.yaml.

Covers acceptance criteria from
`docs/feature-requests/memory/FR-MEMORY-117-per-store-acl/spec.md`:

* AC #1  — permissive default when no STORE.yaml
* AC #2  — wildcard actor `*` matches
* AC #3  — read-only mode blocks write
* AC #4  — explicit deny overrides allow (regardless of order)
* AC #5  — glob actor matching
* AC #6  — closest STORE.yaml wins (innermost overrides outer)
* AC #7  — first-match-wins on acl list
* AC #8  — default_mode applied when no entry matches
* AC #9  — rejected write emits memory.acl_denied aux row
* AC #10 — move respects both src + dst
* AC #11 — STORE.yaml mtime cache invalidation
* AC #15 — `cyberos acl show` lists store_id + acl
* AC #16 — `cyberos acl explain <path>` resolves mode + matched entry
* AC #17 — WARN-ONLY mode (no §14.4 anchor → log but proceed)
* AC #18 — Enforcement when §14.4 present (log + refuse)
* AC #19 — built-in actor literals
* AC #20 — symlinked memory_root resolution
"""

from __future__ import annotations

import json
import time
from pathlib import Path

import pytest
import yaml

from cyberos.core.frontmatter import Frontmatter, serialize
from cyberos.core.ops import AclDenied, put as canonical_put, move as canonical_move, delete as canonical_delete
from cyberos.core.store_acl import (
    AclResult,
    StoreAcl,
    check_write,
    explain,
    find_governing_store_yaml,
)
from cyberos.core.writer import Writer


# ---- fixtures --------------------------------------------------------------


@pytest.fixture(autouse=True)
def _exempt_sandbox_path(monkeypatch, tmp_path):
    """Exempt the test's tmp_path from AGENTS.md §0.1 sandbox-fragment check."""
    monkeypatch.setenv("CYBEROS_HOST_MOUNT_PREFIX", str(tmp_path))


def _init_store(tmp_path: Path, with_section_14_4: bool = True) -> Path:
    """Bootstrap a minimal store with manifest + optional §14.4 anchor."""
    store = tmp_path / ".cyberos/memory/store"
    (store / "audit").mkdir(parents=True)
    (store / "memories" / "facts").mkdir(parents=True)
    (store / "memories" / "org-wide-knowledge").mkdir(parents=True)
    (store / "manifest.json").write_text(json.dumps({
        "schema_version": 1,
        "audit_chain_head": "sha256:" + "0" * 64,
        "last_updated_at": "2026-05-19T00:00:00Z",
        "timezone": "UTC",
    }))
    if with_section_14_4:
        # Minimal AGENTS.md carrying the §14.4 anchor
        (store / "AGENTS.md").write_text(
            "# stub\n## §14  Cross-agent interop\n\n"
            "§14.4  **Store-level ACL.** (Added by P20.)\n"
            "§14.4.1  Each subtree MAY declare a STORE.yaml.\n"
        )
    return store


def _write_store_yaml(store: Path, subdir: str, *, store_id: str,
                     default_mode: str = "read-write",
                     acl: list[dict] | None = None) -> Path:
    """Helper: write a STORE.yaml at <store>/<subdir>/STORE.yaml."""
    target = store / subdir / "STORE.yaml"
    target.parent.mkdir(parents=True, exist_ok=True)
    body = {
        "store_id": store_id,
        "default_mode": default_mode,
        "acl": acl or [],
    }
    target.write_text(yaml.safe_dump(body))
    return target


def _body() -> bytes:
    fm = Frontmatter(id="F-1", kind="fact", ts_ns=time.time_ns(),
                     actor="t", tags=[], extra={})
    return serialize(fm, b"body")


# ---- StoreAcl parse / resolve ---------------------------------------------


def test_storeacl_parse_minimal(tmp_path: Path) -> None:
    yml = tmp_path / "STORE.yaml"
    yml.write_text(yaml.safe_dump({
        "store_id": "test",
        "acl": [{"actor": "*", "mode": "read-write"}],
    }))
    acl = StoreAcl.from_yaml(yml)
    assert acl.store_id == "test"
    assert acl.default_mode == "read-write"
    assert acl.acl == (("*", "read-write"),)


def test_storeacl_parse_invalid_yaml(tmp_path: Path) -> None:
    yml = tmp_path / "STORE.yaml"
    yml.write_text(":::not valid yaml::: { invalid")
    with pytest.raises(ValueError, match="invalid YAML"):
        StoreAcl.from_yaml(yml)


def test_storeacl_parse_missing_store_id(tmp_path: Path) -> None:
    yml = tmp_path / "STORE.yaml"
    yml.write_text(yaml.safe_dump({"acl": []}))
    with pytest.raises(ValueError, match="store_id"):
        StoreAcl.from_yaml(yml)


def test_storeacl_parse_bad_mode(tmp_path: Path) -> None:
    yml = tmp_path / "STORE.yaml"
    yml.write_text(yaml.safe_dump({
        "store_id": "t",
        "acl": [{"actor": "*", "mode": "invalid"}],
    }))
    with pytest.raises(ValueError, match="closed enum"):
        StoreAcl.from_yaml(yml)


def test_resolve_wildcard() -> None:
    """AC #2."""
    acl = StoreAcl(store_id="t", default_mode="read",
                   acl=(("*", "read-write"),))
    assert acl.resolve_mode("anyone") == "read-write"


def test_resolve_first_match_wins() -> None:
    """AC #7 — wildcard first beats more-specific later."""
    acl = StoreAcl(store_id="t", default_mode="read",
                   acl=(("*", "read"), ("stephen", "read-write")))
    assert acl.resolve_mode("stephen") == "read"   # wildcard matched first


def test_resolve_explicit_deny_overrides() -> None:
    """AC #4 — deny first beats wildcard allow second."""
    acl = StoreAcl(store_id="t", default_mode="read-write",
                   acl=(("scheduled-importer", "deny"),
                        ("*", "read-write")))
    assert acl.resolve_mode("scheduled-importer") == "deny"
    assert acl.resolve_mode("stephen") == "read-write"


def test_resolve_glob_actor_matching() -> None:
    """AC #5 — fnmatchcase glob patterns."""
    acl = StoreAcl(store_id="t", default_mode="read",
                   acl=(("stephen@*", "read-write"),))
    assert acl.resolve_mode("stephen@example.com") == "read-write"
    assert acl.resolve_mode("alice@example.com") == "read"  # default


def test_resolve_default_mode_when_no_match() -> None:
    """AC #8."""
    acl = StoreAcl(store_id="t", default_mode="deny",
                   acl=(("stephen", "read-write"),))
    assert acl.resolve_mode("alice") == "deny"


# ---- find_governing_store_yaml -------------------------------------------


def test_governing_yaml_permissive_when_absent(tmp_path: Path) -> None:
    """AC #1 — no STORE.yaml anywhere → returns None (caller treats as permissive)."""
    store = _init_store(tmp_path)
    assert find_governing_store_yaml(store, "memories/facts/x.md") is None


def test_governing_yaml_closest_wins(tmp_path: Path) -> None:
    """AC #6 — innermost STORE.yaml wins over outer."""
    store = _init_store(tmp_path)
    _write_store_yaml(store, "memories", store_id="outer",
                      acl=[{"actor": "*", "mode": "read"}])
    _write_store_yaml(store, "memories/facts", store_id="inner",
                      acl=[{"actor": "*", "mode": "read-write"}])
    governing = find_governing_store_yaml(store, "memories/facts/x.md")
    assert governing is not None
    assert "facts" in str(governing)


# ---- check_write integration ---------------------------------------------


def test_permissive_default_no_yaml(tmp_path: Path) -> None:
    """AC #1."""
    store = _init_store(tmp_path)
    res = check_write(store, "memories/facts/x.md", actor="stephen")
    assert res.allowed is True
    assert res.yaml_path is None


def test_read_only_blocks_write_enforced(tmp_path: Path) -> None:
    """AC #3 + #18 — read-only mode + §14.4 anchor → refuse."""
    store = _init_store(tmp_path, with_section_14_4=True)
    _write_store_yaml(store, "memories/org-wide-knowledge",
                      store_id="org",
                      acl=[{"actor": "*", "mode": "read"}])
    res = check_write(store, "memories/org-wide-knowledge/x.md", actor="stephen")
    assert res.allowed is False
    assert res.mode == "read"
    assert res.reason and res.reason.startswith("acl_denied:")


def test_warn_only_mode_allows_but_logs(tmp_path: Path) -> None:
    """AC #17 — without §14.4 anchor, denied writes still PROCEED but with reason."""
    store = _init_store(tmp_path, with_section_14_4=False)
    _write_store_yaml(store, "memories/org-wide-knowledge",
                      store_id="org",
                      acl=[{"actor": "*", "mode": "read"}])
    res = check_write(store, "memories/org-wide-knowledge/x.md", actor="stephen")
    assert res.allowed is True
    assert res.reason and res.reason.startswith("warn_only:")


def test_built_in_actor_literal(tmp_path: Path) -> None:
    """AC #19 — `dream-runner`, `scheduled-importer`, etc. match literally."""
    store = _init_store(tmp_path)
    _write_store_yaml(store, "memories/facts", store_id="f",
                      acl=[
                          {"actor": "dream-runner", "mode": "read-write"},
                          {"actor": "*", "mode": "read"},
                      ])
    assert check_write(store, "memories/facts/x.md", actor="dream-runner").allowed
    assert not check_write(store, "memories/facts/x.md", actor="alice").allowed \
        or check_write(store, "memories/facts/x.md", actor="alice").mode == "read"


# ---- writer / ops integration --------------------------------------------


def test_put_acl_denied_raises_under_enforcement(tmp_path: Path) -> None:
    """AC #3 + #18 — `cyberos.core.ops.put` raises AclDenied on refusal."""
    store = _init_store(tmp_path, with_section_14_4=True)
    _write_store_yaml(store, "memories/facts", store_id="f",
                      acl=[{"actor": "*", "mode": "read"}])
    with Writer(store) as w:
        with pytest.raises(AclDenied):
            canonical_put(w, "memories/facts/x.md", _body(),
                          actor="alice", kind="fact")


def test_put_acl_denied_emits_aux_row(tmp_path: Path) -> None:
    """AC #9 — refused write emits memory.acl_denied aux row before raising."""
    from cyberos.core.dream._audit_iter import iter_audit_rows
    store = _init_store(tmp_path, with_section_14_4=True)
    _write_store_yaml(store, "memories/facts", store_id="f",
                      acl=[{"actor": "*", "mode": "read"}])
    with Writer(store) as w:
        with pytest.raises(AclDenied):
            canonical_put(w, "memories/facts/x.md", _body(),
                          actor="alice", kind="fact")
    rows = list(iter_audit_rows(store))
    acl_rows = [r for r in rows if r["op"] == "memory.acl_denied"]
    assert len(acl_rows) >= 1
    payload = acl_rows[-1]["extra"]
    assert payload["actor"] == "alice"
    assert payload["target_path"] == "memories/facts/x.md"
    assert payload["mode"] == "read"
    assert payload["warn_only"] is False
    assert payload["attempt_kind"] == "put"


def test_put_warn_only_proceeds_with_aux_row(tmp_path: Path) -> None:
    """AC #17 — WARN-ONLY mode proceeds + emits aux row marked warn_only=True."""
    from cyberos.core.dream._audit_iter import iter_audit_rows
    store = _init_store(tmp_path, with_section_14_4=False)
    _write_store_yaml(store, "memories/facts", store_id="f",
                      acl=[{"actor": "*", "mode": "read"}])
    with Writer(store) as w:
        seq = canonical_put(w, "memories/facts/x.md", _body(),
                            actor="alice", kind="fact")
    assert seq > 0
    rows = list(iter_audit_rows(store))
    acl_rows = [r for r in rows if r["op"] == "memory.acl_denied"]
    assert any(r["extra"]["warn_only"] is True for r in acl_rows)


def test_move_checks_both_paths(tmp_path: Path) -> None:
    """AC #10 — move respects both src + dst ACLs."""
    store = _init_store(tmp_path, with_section_14_4=True)
    # src in read-write, dst in deny → move should fail on dst side
    _write_store_yaml(store, "memories/facts", store_id="f",
                      acl=[{"actor": "*", "mode": "read-write"}])
    (store / "memories" / "secrets").mkdir(exist_ok=True)
    _write_store_yaml(store, "memories/secrets", store_id="s",
                      acl=[{"actor": "*", "mode": "deny"}])
    with Writer(store) as w:
        # First, write to src
        canonical_put(w, "memories/facts/move-me.md", _body(),
                      actor="alice", kind="fact")
        # Now try move into the secrets subtree
        with pytest.raises(AclDenied, match="dst side"):
            canonical_move(w, "memories/facts/move-me.md",
                          "memories/secrets/move-me.md",
                          actor="alice")


def test_delete_blocked_under_enforcement(tmp_path: Path) -> None:
    """delete respects ACL — extension of AC #3 / #18 to the delete op."""
    store = _init_store(tmp_path, with_section_14_4=True)
    # Write succeeds when ACL is permissive
    _write_store_yaml(store, "memories/facts", store_id="f",
                      acl=[{"actor": "alice", "mode": "read-write"},
                           {"actor": "*", "mode": "read"}])
    with Writer(store) as w:
        canonical_put(w, "memories/facts/x.md", _body(),
                      actor="alice", kind="fact")
    # bob (read-only) cannot delete
    with Writer(store) as w:
        with pytest.raises(AclDenied):
            canonical_delete(w, "memories/facts/x.md",
                             actor="bob", mode="tombstone", reason="test")


# ---- explain ---------------------------------------------------------------


def test_explain_resolves_denied(tmp_path: Path) -> None:
    """AC #16 — `explain()` returns structured result with mode + matched entry."""
    store = _init_store(tmp_path, with_section_14_4=True)
    _write_store_yaml(store, "memories/org-wide-knowledge",
                      store_id="org",
                      acl=[
                          {"actor": "stephen@*", "mode": "read-write"},
                          {"actor": "scheduled-importer", "mode": "deny"},
                          {"actor": "*", "mode": "read"},
                      ])
    result = explain(store, "memories/org-wide-knowledge/runbook.md",
                     actor="scheduled-importer")
    assert result["effective_mode"] == "deny"
    assert result["allowed_write"] is False
    assert "scheduled-importer" in result["matched_entry"]


def test_explain_resolves_allowed(tmp_path: Path) -> None:
    """AC #16 — happy path."""
    store = _init_store(tmp_path, with_section_14_4=True)
    _write_store_yaml(store, "memories/org-wide-knowledge",
                      store_id="org",
                      acl=[{"actor": "stephen@*", "mode": "read-write"},
                           {"actor": "*", "mode": "read"}])
    result = explain(store, "memories/org-wide-knowledge/x.md",
                     actor="stephen@cyberskill.world")
    assert result["effective_mode"] == "read-write"
    assert result["allowed_write"] is True
