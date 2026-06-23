#!/usr/bin/env python3
"""Reference CyberOS module MCP server (FR-MCP-002 federation example).

This is the smallest honest example of how a CyberOS module joins the mcp-gateway:

  1. It serves JSON-RPC 2.0 over `POST /mcp` (the methods the gateway forwards:
     `initialize`, `tools/list`, `tools/call`).
  2. At startup it self-registers its tool catalogue with the gateway by POSTing to
     `/v1/mcp/register`, so the tools show up in the gateway's `tools/list` and in the
     desktop Tools tab. The gateway then forwards `tools/call` for those tools back to
     this server's `/mcp` endpoint and returns the result.

It is stdlib-only (no third-party deps) so any module author can copy it as a starting
point and so it runs anywhere `python3` does. Real modules (cuo, obs, memory, ...) adopt
this same contract; the tool bodies here are deliberately trivial (echo / now).

Run it next to a gateway started with `MCP_DEV_REGISTRATION=1`:

    python3 reference_module.py --gateway http://127.0.0.1:8090 --listen 127.0.0.1:8099

Then refresh the desktop Tools tab: `cyberos.demo.echo` and `cyberos.demo.now` appear,
and running echo forwards through the gateway to here and returns your arguments.
"""

from __future__ import annotations

import argparse
import datetime as _dt
import json
import threading
import urllib.error
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Any

MODULE_NAME = "demo"
HEARTBEAT_INTERVAL_SECS = 10  # match the gateway's DEC-2350 cadence

# The tool catalogue this module exposes. `inputSchema` and `annotations` follow the MCP
# 2025-11-25 wire form (camelCase) the gateway's registration endpoint accepts.
TOOLS: list[dict[str, Any]] = [
    {
        "name": "cyberos.demo.echo",
        "description": "Echo the arguments back as text. A read-only smoke-test tool.",
        "inputSchema": {
            "type": "object",
            "properties": {"message": {"type": "string", "description": "anything"}},
        },
        "annotations": {
            "title": "Echo",
            "readOnlyHint": True,
            "idempotentHint": True,
        },
        "requiresScope": ["mcp:tools"],
    },
    {
        "name": "cyberos.demo.now",
        "description": "Return the current UTC time in ISO-8601. Read-only.",
        "inputSchema": {"type": "object", "properties": {}},
        "annotations": {
            "title": "Now",
            "readOnlyHint": True,
            "idempotentHint": False,
        },
        "requiresScope": ["mcp:tools"],
    },
]


# ---- tool bodies -------------------------------------------------------------------


def _text_result(text: str) -> dict[str, Any]:
    """A successful MCP tools/call result carrying a single text content block."""
    return {"content": [{"type": "text", "text": text}], "isError": False}


def run_tool(name: str, arguments: dict[str, Any]) -> dict[str, Any]:
    """Execute one tool. Returns an MCP tools/call result (with isError on tool failure)."""
    if name == "cyberos.demo.echo":
        return _text_result(json.dumps(arguments, ensure_ascii=False, sort_keys=True))
    if name == "cyberos.demo.now":
        now = _dt.datetime.now(_dt.timezone.utc).replace(microsecond=0).isoformat()
        return _text_result(now)
    # Unknown tool name: in-band tool error, not a transport error.
    return {
        "content": [{"type": "text", "text": f"unknown tool: {name}"}],
        "isError": True,
    }


# ---- JSON-RPC handling -------------------------------------------------------------

# JSON-RPC 2.0 / MCP error codes (subset).
PARSE_ERROR = -32700
INVALID_REQUEST = -32600
METHOD_NOT_FOUND = -32601
INVALID_PARAMS = -32602

MCP_PROTOCOL_VERSION = "2025-11-25"


def _rpc_result(req_id: Any, result: Any) -> dict[str, Any]:
    return {"jsonrpc": "2.0", "id": req_id, "result": result}


def _rpc_error(req_id: Any, code: int, message: str) -> dict[str, Any]:
    return {"jsonrpc": "2.0", "id": req_id, "error": {"code": code, "message": message}}


def handle_rpc(request: dict[str, Any]) -> dict[str, Any] | None:
    """Dispatch one JSON-RPC request object. Returns the response, or None for a
    notification (a request with no `id`)."""
    req_id = request.get("id")
    method = request.get("method")
    params = request.get("params") or {}

    if method == "initialize":
        return _rpc_result(
            req_id,
            {
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "serverInfo": {"name": f"cyberos-module-{MODULE_NAME}", "version": "0.1.0"},
                "capabilities": {"tools": {}},
            },
        )
    if method == "tools/list":
        descriptors = [
            {
                "name": t["name"],
                "description": t["description"],
                "inputSchema": t["inputSchema"],
                "annotations": t["annotations"],
            }
            for t in TOOLS
        ]
        return _rpc_result(req_id, {"tools": descriptors})
    if method == "tools/call":
        name = params.get("name")
        if not isinstance(name, str) or not name:
            return _rpc_error(req_id, INVALID_PARAMS, "tools/call: missing name")
        arguments = params.get("arguments") or {}
        if not isinstance(arguments, dict):
            arguments = {}
        return _rpc_result(req_id, run_tool(name, arguments))
    if req_id is None:
        # Notification (e.g. notifications/initialized): no response.
        return None
    return _rpc_error(req_id, METHOD_NOT_FOUND, f"method not found: {method}")


class _Handler(BaseHTTPRequestHandler):
    # Quieter logging; one line per call is enough for a reference server.
    def log_message(self, fmt: str, *args: Any) -> None:  # noqa: A003 - stdlib signature
        print(f"[reference_module] {self.address_string()} {fmt % args}")

    def _send_json(self, status: int, payload: Any) -> None:
        body = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self) -> None:  # noqa: N802 - stdlib signature
        if self.path.rstrip("/") in ("", "/healthz", "/mcp/healthz"):
            self._send_json(200, {"status": "ok", "module": MODULE_NAME})
            return
        self._send_json(404, {"error": "not_found"})

    def do_POST(self) -> None:  # noqa: N802 - stdlib signature
        length = int(self.headers.get("content-length", "0") or "0")
        raw = self.rfile.read(length) if length else b""
        try:
            parsed = json.loads(raw or b"{}")
        except json.JSONDecodeError:
            self._send_json(200, _rpc_error(None, PARSE_ERROR, "parse error"))
            return

        # Support a single request object (what the gateway sends). Batch is out of scope
        # for this reference.
        if not isinstance(parsed, dict):
            self._send_json(200, _rpc_error(None, INVALID_REQUEST, "expected a request object"))
            return

        resp = handle_rpc(parsed)
        if resp is None:
            # Notification: 204 No Content.
            self.send_response(204)
            self.end_headers()
            return
        self._send_json(200, resp)


# ---- self-registration -------------------------------------------------------------


def build_registration(endpoint: str) -> dict[str, Any]:
    """The body POSTed to the gateway's /v1/mcp/register."""
    return {"module": MODULE_NAME, "endpoint": endpoint, "tools": TOOLS}


def self_register(gateway: str, endpoint: str, timeout: float = 5.0) -> tuple[bool, str]:
    """POST this module's catalogue to the gateway. Returns (ok, detail)."""
    url = gateway.rstrip("/") + "/v1/mcp/register"
    data = json.dumps(build_registration(endpoint)).encode("utf-8")
    req = urllib.request.Request(
        url, data=data, headers={"content-type": "application/json"}, method="POST"
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            body = resp.read().decode("utf-8", "replace")
            return True, f"HTTP {resp.status}: {body}"
    except urllib.error.HTTPError as e:
        detail = e.read().decode("utf-8", "replace")
        hint = ""
        if e.code == 403:
            hint = "  (start the gateway with MCP_DEV_REGISTRATION=1 to enable registration)"
        return False, f"HTTP {e.code}: {detail}{hint}"
    except urllib.error.URLError as e:
        return False, f"could not reach gateway at {url}: {e.reason}"


def _post_module_control(gateway: str, path: str, timeout: float = 5.0) -> int:
    """POST {"module": MODULE_NAME} to a control-plane path (heartbeat/deregister)."""
    url = gateway.rstrip("/") + path
    data = json.dumps({"module": MODULE_NAME}).encode("utf-8")
    req = urllib.request.Request(
        url, data=data, headers={"content-type": "application/json"}, method="POST"
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return resp.status


def heartbeat_loop(gateway: str, stop: threading.Event) -> None:
    """Send a heartbeat every interval until stopped (FR-MCP-002 DEC-2350)."""
    while not stop.wait(HEARTBEAT_INTERVAL_SECS):
        try:
            _post_module_control(gateway, "/v1/mcp/heartbeat")
        except Exception as e:  # noqa: BLE001 - keep the loop alive across transient errors
            print(f"[reference_module] heartbeat failed: {e}")


def main() -> None:
    parser = argparse.ArgumentParser(description="CyberOS reference module MCP server")
    parser.add_argument("--listen", default="127.0.0.1:8099", help="host:port to serve /mcp on")
    parser.add_argument(
        "--gateway",
        default=None,
        help="gateway base URL to self-register with (e.g. http://127.0.0.1:8090); omit to skip",
    )
    parser.add_argument(
        "--public-host",
        default=None,
        help="host:port the gateway should use to reach this module (defaults to --listen)",
    )
    args = parser.parse_args()

    host, _, port_s = args.listen.partition(":")
    port = int(port_s or "8099")
    reachable = args.public_host or args.listen
    endpoint = f"http://{reachable}/mcp"

    server = ThreadingHTTPServer((host, port), _Handler)
    print(f"[reference_module] serving MCP on http://{args.listen}/mcp (module={MODULE_NAME})")

    stop = threading.Event()
    if args.gateway:
        ok, detail = self_register(args.gateway, endpoint)
        status = "registered" if ok else "registration FAILED"
        print(f"[reference_module] {status} with {args.gateway}: {detail}")
        if ok:
            threading.Thread(
                target=heartbeat_loop, args=(args.gateway, stop), daemon=True
            ).start()
            print(f"[reference_module] heartbeating every {HEARTBEAT_INTERVAL_SECS}s")
    else:
        print("[reference_module] --gateway not set; serving without self-registering")

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n[reference_module] shutting down")
        stop.set()
        if args.gateway:
            try:
                _post_module_control(args.gateway, "/v1/mcp/deregister")
                print("[reference_module] deregistered with the gateway")
            except Exception as e:  # noqa: BLE001 - best-effort on shutdown
                print(f"[reference_module] deregister failed: {e}")
        server.shutdown()


if __name__ == "__main__":
    main()
