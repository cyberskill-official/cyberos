"""Tests for cyberos.core.serve (PROPOSAL.md P10).

We spin up an HTTPServer on an ephemeral loopback port in a thread per
test, hit it with urllib, and tear down. The handler is closed over its
config so each test gets a fresh store + token.
"""

from __future__ import annotations

import json
import threading
import urllib.error
import urllib.request
from http.server import HTTPServer
from pathlib import Path

import pytest

from cyberos.core import serve as serve_mod


# ---------------------------------------------------------------------------
# fixtures
# ---------------------------------------------------------------------------


def _empty_store(tmp_path: Path) -> Path:
    s = tmp_path / ".cyberos/memory/store"
    s.mkdir(parents=True, exist_ok=True)
    (s / "audit").mkdir(parents=True, exist_ok=True)
    (s / "manifest.json").write_text('{}', encoding="utf-8")
    return s


def _write_memory(store: Path, rel: str, *, kind: str, actor: str,
                  body: str, mid: str | None = None) -> None:
    import msgspec
    fm = {
        "id": mid or rel, "kind": kind, "ts_ns": 1,
        "actor": actor, "tags": [],
    }
    fm_bytes = msgspec.json.encode(fm, order="sorted")
    path = store / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"---\n" + fm_bytes + b"\n---\n" + body.encode("utf-8"))


@pytest.fixture
def running_server(tmp_path):
    """Spin up the serve handler on an ephemeral port. Yields (url, token)."""
    store = _empty_store(tmp_path)
    _write_memory(store, "memories/facts/a.md",
                  kind="fact", actor="stephen",
                  body="alpha body content")
    _write_memory(store, "memories/decisions/d.md",
                  kind="decision", actor="agent",
                  body="decided to ship")

    cfg = serve_mod.ServeConfig(store=store, host="127.0.0.1", port=0)
    token = serve_mod.get_or_create_token(store)
    cfg.token = token  # use the token we just minted

    handler = serve_mod.make_handler(cfg)
    httpd = HTTPServer((cfg.host, 0), handler)  # port=0 → ephemeral
    port = httpd.server_address[1]
    base = f"http://127.0.0.1:{port}"

    t = threading.Thread(target=httpd.serve_forever, daemon=True)
    t.start()
    try:
        yield base, token, store
    finally:
        httpd.shutdown()
        httpd.server_close()
        t.join(timeout=2)


def _get(url: str, *, token: str | None = None, want_json: bool = True):
    req = urllib.request.Request(url)
    if token is not None:
        req.add_header("Authorization", f"Bearer {token}")
    with urllib.request.urlopen(req, timeout=5) as resp:
        body = resp.read()
        if want_json:
            return resp.status, json.loads(body)
        return resp.status, body.decode("utf-8")


def _post(url: str, payload: dict, *, token: str | None = None):
    req = urllib.request.Request(url, method="POST")
    req.add_header("Content-Type", "application/json")
    if token is not None:
        req.add_header("Authorization", f"Bearer {token}")
    data = json.dumps(payload).encode("utf-8")
    with urllib.request.urlopen(req, data=data, timeout=5) as resp:
        return resp.status, json.loads(resp.read())


# ---------------------------------------------------------------------------
# token management
# ---------------------------------------------------------------------------


def test_token_persists_across_calls(tmp_path):
    store = _empty_store(tmp_path)
    t1 = serve_mod.get_or_create_token(store)
    t2 = serve_mod.get_or_create_token(store)
    assert t1 == t2
    assert len(t1) >= 32


def test_reset_token_changes_value(tmp_path):
    store = _empty_store(tmp_path)
    t1 = serve_mod.get_or_create_token(store)
    t2 = serve_mod.reset_token(store)
    assert t1 != t2
    # And it's now persisted
    assert serve_mod.get_or_create_token(store) == t2


# ---------------------------------------------------------------------------
# /healthz (no auth)
# ---------------------------------------------------------------------------


def test_healthz_no_auth(running_server):
    base, _token, _store = running_server
    status, body = _get(f"{base}/healthz")
    assert status == 200
    assert body["status"] == "ok"
    assert "ts_ns" in body


# ---------------------------------------------------------------------------
# auth gate
# ---------------------------------------------------------------------------


def test_state_requires_auth(running_server):
    base, _token, _store = running_server
    with pytest.raises(urllib.error.HTTPError) as exc:
        _get(f"{base}/state")
    assert exc.value.code == 401


def test_wrong_token_rejected(running_server):
    base, _token, _store = running_server
    with pytest.raises(urllib.error.HTTPError) as exc:
        _get(f"{base}/state", token="not-the-real-token")
    assert exc.value.code == 401


def test_right_token_accepted(running_server):
    base, token, _store = running_server
    status, body = _get(f"{base}/state", token=token)
    assert status == 200
    assert body["state"] in ("READY", "FROZEN_RECOVERABLE", "FROZEN_HUMAN")


# ---------------------------------------------------------------------------
# /audit/head
# ---------------------------------------------------------------------------


def test_audit_head_empty_store(running_server):
    base, token, _store = running_server
    status, body = _get(f"{base}/audit/head", token=token)
    assert status == 200
    assert body["last_seq"] == 0


# ---------------------------------------------------------------------------
# /memories
# ---------------------------------------------------------------------------


def test_memories_list(running_server):
    base, token, _store = running_server
    status, body = _get(f"{base}/memories", token=token)
    assert status == 200
    rel_paths = {m["rel_path"] for m in body}
    assert "memories/facts/a.md" in rel_paths
    assert "memories/decisions/d.md" in rel_paths


def test_memories_filter_by_kind(running_server):
    base, token, _store = running_server
    status, body = _get(f"{base}/memories?kind=decision", token=token)
    assert status == 200
    assert all(m["kind"] == "decision" for m in body)
    assert len(body) == 1


def test_memories_filter_by_actor(running_server):
    base, token, _store = running_server
    status, body = _get(f"{base}/memories?actor=agent", token=token)
    assert status == 200
    assert all(m["actor"] == "agent" for m in body)


def test_memories_get_one(running_server):
    base, token, _store = running_server
    status, body = _get(f"{base}/memories/memories/facts/a.md",
                        token=token, want_json=False)
    assert status == 200
    assert "alpha body content" in body


def test_memories_get_nonexistent(running_server):
    base, token, _store = running_server
    with pytest.raises(urllib.error.HTTPError) as exc:
        _get(f"{base}/memories/memories/facts/nope.md", token=token)
    assert exc.value.code == 404


def test_memories_get_path_traversal_blocked(running_server):
    base, token, _store = running_server
    # Even though urllib normalises some paths, the server should still
    # validate via .resolve() check.
    with pytest.raises(urllib.error.HTTPError) as exc:
        _get(
            f"{base}/memories/../../etc/passwd",
            token=token,
        )
    # Either 403 (escape detected) or 404 (resolve fails to find).
    assert exc.value.code in (403, 404)


# ---------------------------------------------------------------------------
# /digest
# ---------------------------------------------------------------------------


def test_digest_text_format(running_server):
    base, token, _store = running_server
    status, body = _get(f"{base}/digest?window=24h&format=text",
                        token=token, want_json=False)
    assert status == 200
    assert "memory digest" in body


def test_digest_json_format(running_server):
    base, token, _store = running_server
    status, body = _get(f"{base}/digest?window=24h", token=token, want_json=False)
    decoded = json.loads(body)
    assert "total_rows" in decoded
    assert "op_counts" in decoded


def test_digest_invalid_window(running_server):
    base, token, _store = running_server
    with pytest.raises(urllib.error.HTTPError) as exc:
        _get(f"{base}/digest?window=garbage", token=token)
    assert exc.value.code == 400


# ---------------------------------------------------------------------------
# /search
# ---------------------------------------------------------------------------


def test_search_missing_query(running_server):
    base, token, _store = running_server
    with pytest.raises(urllib.error.HTTPError) as exc:
        _post(f"{base}/search", {}, token=token)
    assert exc.value.code == 400


def test_search_invalid_json(running_server):
    base, token, _store = running_server
    # We can't use _post for invalid JSON — go raw
    req = urllib.request.Request(f"{base}/search", method="POST")
    req.add_header("Authorization", f"Bearer {token}")
    req.add_header("Content-Type", "application/json")
    with pytest.raises(urllib.error.HTTPError) as exc:
        urllib.request.urlopen(req, data=b"not valid json", timeout=5)
    assert exc.value.code == 400


def test_search_fts5_returns_payload(running_server):
    base, token, _store = running_server
    status, body = _post(f"{base}/search",
                         {"query": "alpha", "limit": 5},
                         token=token)
    assert status == 200
    assert body["mode"] == "fts5"
    # The FTS5 index may be empty in test environments — accept any list
    assert isinstance(body["hits"], list)


def test_search_semantic_returns_503_when_unavailable(running_server, monkeypatch):
    from cyberos.core import semantic as semantic_mod
    monkeypatch.setattr(semantic_mod, "_AVAILABLE", False)
    monkeypatch.setattr(semantic_mod, "_MODEL_CACHE", None)
    base, token, _store = running_server
    with pytest.raises(urllib.error.HTTPError) as exc:
        _post(f"{base}/search",
              {"query": "x", "semantic": True},
              token=token)
    assert exc.value.code == 503
    semantic_mod._AVAILABLE = None


# ---------------------------------------------------------------------------
# unknown routes
# ---------------------------------------------------------------------------


def test_unknown_get_route(running_server):
    base, token, _store = running_server
    with pytest.raises(urllib.error.HTTPError) as exc:
        _get(f"{base}/no-such-route", token=token)
    assert exc.value.code == 404


def test_unknown_post_route(running_server):
    base, token, _store = running_server
    with pytest.raises(urllib.error.HTTPError) as exc:
        _post(f"{base}/no-such-route", {}, token=token)
    assert exc.value.code == 404
