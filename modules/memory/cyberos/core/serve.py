"""
cyberos.core.serve — local read-only HTTP REST mode (PROPOSAL.md P10).

The protocol stays local-first: ``cyberos serve`` binds to ``127.0.0.1``
by default, requires a bearer token on every request, and refuses to
serve write endpoints. Other tools on the same machine (companion apps,
Raycast extensions, future iOS Shortcuts via ssh-tunnel) can query the
memory without each tool re-implementing the binlog parser.

Endpoints (all GET unless noted):

  GET  /healthz                        — liveness probe (no auth)
  GET  /state                          — agent state (READY / FROZEN_*)
  GET  /audit/head                     — current HEAD seq
  GET  /memories?kind=&actor=&limit=   — list memory paths
  GET  /memories/<rel_path>            — one memory body
  GET  /digest?window=24h&format=text  — daily summary
  POST /search                         — body JSON: {query, limit, semantic}

Auth: every endpoint except /healthz requires the bearer token. Token
lives at ``<store>/.serve-token`` (auto-generated if missing, mode 0600).
Pass via ``Authorization: Bearer <token>`` header.

Design notes:

* stdlib only — no flask/uvicorn — to keep dependencies the same as
  Layer 1.
* Single-threaded ``HTTPServer`` is fine for the expected one-user load.
  Heavier loads can be reverse-proxied behind nginx etc.
* No CORS — local-only by design. Browsers asking from
  ``http://localhost:<port>`` to ``http://127.0.0.1:<port>`` should
  point at the same host:port; cross-origin is intentionally out of scope.
"""

from __future__ import annotations

import hashlib
import json
import os
import secrets
import stat
import sys
import time
from dataclasses import dataclass, field
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from typing import Callable
from urllib.parse import parse_qs, urlparse


# ---------------------------------------------------------------------------
# token management
# ---------------------------------------------------------------------------


def _token_path(store: Path) -> Path:
    return store / ".serve-token"


def get_or_create_token(store: Path) -> str:
    """Return the bearer token, generating one on first run.

    Token is a 32-byte URL-safe random string (~256 bits of entropy).
    File permissions are clamped to 0600 — owner-only — on POSIX. On
    Windows the file inherits the directory ACL.
    """
    path = _token_path(store)
    if path.is_file():
        return path.read_text(encoding="utf-8").strip()
    token = secrets.token_urlsafe(32)
    store.mkdir(parents=True, exist_ok=True)
    path.write_text(token, encoding="utf-8")
    try:
        os.chmod(path, stat.S_IRUSR | stat.S_IWUSR)
    except OSError:
        # On Windows, chmod has limited effect — accept.
        pass
    return token


def reset_token(store: Path) -> str:
    """Generate a fresh token, replacing any existing one."""
    path = _token_path(store)
    if path.exists():
        path.unlink()
    return get_or_create_token(store)


# ---------------------------------------------------------------------------
# handler factory
# ---------------------------------------------------------------------------


@dataclass
class ServeConfig:
    store: Path
    host: str = "127.0.0.1"
    port: int = 8765
    token: str | None = None


def make_handler(config: ServeConfig) -> type:
    """Return a BaseHTTPRequestHandler subclass closed over ``config``."""

    token = config.token or get_or_create_token(config.store)
    store = config.store

    class Handler(BaseHTTPRequestHandler):
        # Quiet down the default per-request stderr noise. We log via
        # log_message below so the host can pipe it where it wants.
        def log_message(self, format: str, *args) -> None:
            sys.stderr.write(
                "[serve] " + (format % args) + "\n"
            )

        # --- helpers ------------------------------------------------------

        def _json(self, status: int, payload: dict | list) -> None:
            body = json.dumps(payload, default=str).encode("utf-8")
            self.send_response(status)
            self.send_header("Content-Type", "application/json; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Cache-Control", "no-store")
            self.end_headers()
            self.wfile.write(body)

        def _text(self, status: int, body: str, ctype: str = "text/plain; charset=utf-8") -> None:
            data = body.encode("utf-8")
            self.send_response(status)
            self.send_header("Content-Type", ctype)
            self.send_header("Content-Length", str(len(data)))
            self.send_header("Cache-Control", "no-store")
            self.end_headers()
            self.wfile.write(data)

        def _check_auth(self) -> bool:
            auth = self.headers.get("Authorization", "")
            if not auth.startswith("Bearer "):
                return False
            supplied = auth[len("Bearer "):].strip()
            # Constant-time compare to avoid trivial timing oracle.
            if len(supplied) != len(token):
                return False
            return secrets.compare_digest(supplied, token)

        # --- routes -------------------------------------------------------

        def do_GET(self) -> None:
            try:
                self._route_get()
            except Exception as exc:  # noqa: BLE001
                self._json(500, {"error": f"{type(exc).__name__}: {exc}"})

        def do_POST(self) -> None:
            try:
                self._route_post()
            except Exception as exc:  # noqa: BLE001
                self._json(500, {"error": f"{type(exc).__name__}: {exc}"})

        def _route_get(self) -> None:
            url = urlparse(self.path)
            path = url.path
            query = parse_qs(url.query)

            # /healthz — unauthenticated liveness
            if path == "/healthz":
                self._json(200, {"status": "ok", "ts_ns": time.time_ns()})
                return

            if not self._check_auth():
                self._json(401, {"error": "missing or invalid bearer token"})
                return

            if path == "/state":
                from cyberos.core.invariants import run_all
                report = run_all(store)
                catastrophic_ids = {
                    "ledger-link-invariant", "ledger-hash-invariant",
                    "ledger-bridge-continuity", "ledger-mmr-cross-check",
                    "manifest-schema-version",
                }
                if report.ok:
                    state = "READY"
                elif any(r.id in catastrophic_ids for r in report.errors):
                    state = "FROZEN_HUMAN"
                else:
                    state = "FROZEN_RECOVERABLE"
                self._json(200, {
                    "state": state,
                    "errors": [r.id for r in report.errors],
                    "warnings": [r.id for r in report.warnings],
                })
                return

            if path == "/audit/head":
                import struct
                head = store / "HEAD"
                last_seq = 0
                if head.is_file():
                    buf = head.read_bytes()
                    if len(buf) == 8:
                        last_seq = struct.unpack("<Q", buf)[0]
                self._json(200, {"last_seq": last_seq})
                return

            if path == "/memories":
                kind = query.get("kind", [None])[0]
                actor = query.get("actor", [None])[0]
                limit = int(query.get("limit", ["50"])[0])
                results = _list_memories(store, kind=kind, actor=actor, limit=limit)
                self._json(200, results)
                return

            if path.startswith("/memories/"):
                rel = path[len("/memories/"):]
                target = (store / rel).resolve()
                # Path-traversal guard
                if not str(target).startswith(str(store.resolve())):
                    self._json(403, {"error": "path escapes store"})
                    return
                if not target.is_file():
                    self._json(404, {"error": "not found"})
                    return
                self._text(200, target.read_text(encoding="utf-8"),
                           ctype="text/markdown; charset=utf-8")
                return

            if path == "/digest":
                from cyberos.core.digest import build, format_text, parse_human_duration
                window = query.get("window", ["24h"])[0]
                fmt = query.get("format", ["json"])[0]
                try:
                    delta_ns = parse_human_duration(window)
                except ValueError as exc:
                    self._json(400, {"error": str(exc)})
                    return
                until = time.time_ns()
                d = build(store, since_ns=until - delta_ns, until_ns=until)
                if fmt == "text":
                    self._text(200, format_text(d))
                else:
                    self._text(200, d.to_json(),
                               ctype="application/json; charset=utf-8")
                return

            self._json(404, {"error": f"unknown route: {path}"})

        def _route_post(self) -> None:
            url = urlparse(self.path)
            path = url.path

            if not self._check_auth():
                self._json(401, {"error": "missing or invalid bearer token"})
                return

            if path != "/search":
                self._json(404, {"error": f"unknown route: {path}"})
                return

            length = int(self.headers.get("Content-Length", "0"))
            body = self.rfile.read(length) if length else b"{}"
            try:
                req = json.loads(body or b"{}")
            except json.JSONDecodeError as exc:
                self._json(400, {"error": f"invalid JSON: {exc}"})
                return
            query = req.get("query", "")
            limit = int(req.get("limit", 20))
            if not query:
                self._json(400, {"error": "missing 'query' in request body"})
                return

            if req.get("semantic"):
                from cyberos.core.semantic import available, search as semantic_search
                if not available():
                    self._json(503, {
                        "error": "semantic search unavailable; install sentence-transformers",
                    })
                    return
                hits = semantic_search(store, query, limit=limit)
                self._json(200, {
                    "mode": "semantic",
                    "hits": [
                        {"rel_path": h.rel_path, "score": h.score, "snippet": h.snippet}
                        for h in hits
                    ],
                })
                return

            from cyberos.core.index import open_index, search_memories
            fingerprint = hashlib.sha256(str(store).encode("utf-8")).hexdigest()[:16]
            conn = open_index(fingerprint)
            hits = list(search_memories(conn, query, limit=limit))
            self._json(200, {
                "mode": "fts5",
                "hits": [
                    {"rel_path": rp, "snippet": sn} for rp, sn in hits
                ],
            })

    return Handler


# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------


def _list_memories(
    store: Path, *,
    kind: str | None = None,
    actor: str | None = None,
    limit: int = 50,
) -> list[dict]:
    """Best-effort listing scanning canonical memory roots.

    No SQLite dependency — direct filesystem walk + frontmatter parse,
    so this endpoint works even before the FTS5 index is built.
    """
    from cyberos.core.frontmatter import looks_like_yaml, parse, parse_legacy_yaml
    roots = ("memories", "company", "module", "member", "client", "project", "persona")
    out: list[dict] = []
    for r in roots:
        base = store / r
        if not base.is_dir():
            continue
        for p in sorted(base.rglob("*.md")):
            if not p.is_file():
                continue
            try:
                raw = p.read_bytes()
                if looks_like_yaml(raw):
                    fm, _body = parse_legacy_yaml(raw)
                else:
                    fm, _body = parse(raw)
                import msgspec
                fm_dict = msgspec.to_builtins(fm)
            except Exception:  # noqa: BLE001
                continue
            if kind and fm_dict.get("kind") != kind:
                continue
            if actor and fm_dict.get("actor") != actor:
                continue
            out.append({
                "rel_path": p.relative_to(store).as_posix(),
                "id": fm_dict.get("id"),
                "kind": fm_dict.get("kind"),
                "actor": fm_dict.get("actor"),
                "ts_ns": fm_dict.get("ts_ns"),
                "tags": fm_dict.get("tags") or [],
            })
            if len(out) >= limit:
                return out
    return out


# ---------------------------------------------------------------------------
# entry point
# ---------------------------------------------------------------------------


def serve_forever(config: ServeConfig) -> None:
    """Block on a stdlib HTTPServer until interrupted."""
    handler_cls = make_handler(config)
    httpd = HTTPServer((config.host, config.port), handler_cls)
    sys.stderr.write(
        f"[serve] cyberos serve listening on http://{config.host}:{config.port}\n"
        f"[serve] token: {handler_cls.__dict__.get('_token_for_log', '(hidden)')}\n"
    )
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        sys.stderr.write("[serve] shutting down\n")
    finally:
        httpd.server_close()


__all__ = [
    "ServeConfig",
    "get_or_create_token",
    "make_handler",
    "reset_token",
    "serve_forever",
]
