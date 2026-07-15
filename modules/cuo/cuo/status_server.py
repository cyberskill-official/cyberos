#!/usr/bin/env python3
"""CUO status server (TASK-APP-006) - a small read-only HTTP surface for the console's CUO Workflows &
GENIE tile. It exposes two things the CUO module already computes, with no LLM call and no mutation:

  - the dream-loop evolution envelope (TASK-CUO-204): enabled / mode / idle window / allowlist / denylist;
  - a summary of the task backlog (docs/tasks/BACKLOG.md): totals by status and by module.

Run from modules/cuo so the `cuo` package imports:

    CYBEROS_ROOT=/path/to/cyberos CUO_DEV_CORS=1 python3 -m cuo.status_server --listen 127.0.0.1:7740

In production a fronting proxy would serve it under one origin; CUO_DEV_CORS adds permissive CORS for the
local browser console only.
"""
from __future__ import annotations

import argparse
import json
import os
from collections import Counter
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

from cuo.core.backlog_reader import parse_backlog
from cuo.core.evolution_envelope import EvolutionEnvelope


def _root() -> Path:
    """Resolve the repo root (holds docs/tasks and modules/cuo)."""
    env = os.environ.get("CYBEROS_ROOT")
    if env:
        return Path(env)
    here = Path.cwd()
    for cand in [here, *here.parents]:
        if (cand / "docs" / "tasks" / "BACKLOG.md").exists():
            return cand
    return here


def _status(root: Path) -> dict:
    env_path = root / "modules" / "cuo" / "config" / "dream.yaml"
    env = EvolutionEnvelope.load(env_path)
    osenv = dict(os.environ)
    envelope = {
        "enabled": env.is_enabled(osenv),
        "effective_mode": env.effective_mode(osenv),
        "configured_mode": env.mode,
        "idle_window_minutes": env.idle_window_minutes,
        "denylist": env.denylist,
        "allowlist": env.allowlist,
    }
    backlog_path = root / "docs" / "tasks" / "BACKLOG.md"
    rows = parse_backlog(backlog_path) if backlog_path.exists() else []
    by_status = Counter(r.status for r in rows)
    by_module = Counter(r.module for r in rows)
    backlog = {
        "total": len(rows),
        "by_status": dict(sorted(by_status.items(), key=lambda kv: -kv[1])),
        "by_module": dict(sorted(by_module.items(), key=lambda kv: -kv[1])),
    }
    return {"envelope": envelope, "backlog": backlog}


def _make_handler(root: Path):
    cors = bool(os.environ.get("CUO_DEV_CORS"))

    class Handler(BaseHTTPRequestHandler):
        def _send(self, code: int, body: dict) -> None:
            data = json.dumps(body).encode()
            self.send_response(code)
            self.send_header("content-type", "application/json")
            if cors:
                self.send_header("access-control-allow-origin", "*")
            self.send_header("content-length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def do_GET(self) -> None:  # noqa: N802 - stdlib signature
            path = self.path.rstrip("/")
            if path in ("", "/healthz"):
                self._send(200, {"status": "ok", "service": "cuo-status"})
            elif path == "/v1/cuo/status":
                try:
                    self._send(200, _status(root))
                except Exception as exc:  # noqa: BLE001 - surface the error to the operator
                    self._send(500, {"error": str(exc)})
            else:
                self._send(404, {"error": "not found"})

        def log_message(self, *_args) -> None:  # silence default logging
            return

    return Handler


def main() -> None:
    ap = argparse.ArgumentParser(description="CUO read-only status server")
    ap.add_argument("--listen", default="127.0.0.1:7740", help="host:port to serve on")
    ap.add_argument("--root", default=None, help="repo root (defaults to CYBEROS_ROOT or a CWD walk)")
    args = ap.parse_args()
    root = Path(args.root) if args.root else _root()
    host, port = args.listen.split(":")
    httpd = ThreadingHTTPServer((host, int(port)), _make_handler(root))
    print(f"[cuo-status] serving on http://{args.listen}  (root: {root})", flush=True)
    httpd.serve_forever()


if __name__ == "__main__":
    main()
