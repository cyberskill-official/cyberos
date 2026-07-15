#!/usr/bin/env python3
"""Obs triage as an MCP-gateway tool (TASK-OBS-007 x TASK-MCP-002 federation).

This exposes the `obs.triage-alert` path the obs-router already calls over HTTP as a tool on the
mcp-gateway, so an observability alert can be triaged through `tools/call` the same way any other
federated module tool is. It is the obs federation surface: the registering module identity is `obs`
and the tool is `cyberos.obs.execute_triage`, even though the code lives in the cuo package next to
`triage_server.py` - because triage is a CUO skill (`obs.triage-alert@1`) invoked in-process.

It adopts the reference-module contract verbatim (services/mcp-gateway/examples/reference_module.py):

  1. It serves JSON-RPC 2.0 over `POST /mcp` for the methods the gateway forwards
     (`initialize`, `tools/list`, `tools/call`).
  2. At startup it self-registers its one tool with the gateway by POSTing to `/v1/mcp/register`,
     so `cyberos.obs.execute_triage` shows up in the gateway's `tools/list` and the desktop Tools tab. The
     gateway then forwards `tools/call` for it back to this server's `/mcp` endpoint.

The tool body runs triage in-process through `triage_server.handle_triage_request` - the same pure
handler the HTTP triage endpoint uses - so there is no second hop and no duplicated logic. The
invoker is selected once at startup (`select_invoker`); when no invoker is available the handler
returns the skill's safe-degrade verdict (confidence 0.0) rather than failing, exactly as it does for
obs-router (SKILL.md section 5). A bad alert (missing `alert` or `alert.name`) is an in-band tool
error (`isError: true`), not a transport error.

Run it next to a gateway started with `MCP_DEV_REGISTRATION=1`:

    python3 -m cuo.triage_mcp_module --gateway http://127.0.0.1:8090 --listen 127.0.0.1:8101

Then refresh the desktop Tools tab: `cyberos.obs.execute_triage` appears, and calling it with an alert
forwards through the gateway to here and returns the triage verdict.
"""

from __future__ import annotations

import argparse
import json
import tempfile
import threading
import urllib.error
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

from cuo.triage_server import (
    SKILL_HANDLE,
    _resolve_skill_root,
    handle_triage_request,
)

MODULE_NAME = "obs"
TRIAGE_TOOL_NAME = "cyberos.obs.execute_triage"
HEARTBEAT_INTERVAL_SECS = 10  # match the gateway's DEC-2350 cadence

# The tool catalogue this module exposes. `inputSchema` and `annotations` follow the MCP
# 2025-11-25 wire form (camelCase) the gateway's registration endpoint accepts. The alert shape
# mirrors the obs-router contract that `triage_server.alert_to_inputs` reads.
TOOLS: list[dict[str, Any]] = [
    {
        "name": TRIAGE_TOOL_NAME,
        "description": (
            "Triage a fired observability alert with the obs.triage-alert skill. Returns a "
            "confidence-scored verdict (summary, suspected cause, suggested runbook url). Read-only: "
            "it assesses the alert and never mutates it or pages on its own."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "alert": {
                    "type": "object",
                    "description": "The fired alert to triage.",
                    "properties": {
                        "name": {"type": "string", "description": "alert name (required)"},
                        "severity": {"type": "string", "description": "e.g. sev1/sev2/sev3"},
                        "fingerprint": {"type": "string", "description": "Alertmanager fingerprint"},
                        "trace_id": {"type": "string", "description": "correlated trace id, if any"},
                        "summary": {"type": "string", "description": "one-line alert summary"},
                    },
                    "required": ["name"],
                }
            },
            "required": ["alert"],
        },
        "annotations": {
            "title": "Triage alert",
            "readOnlyHint": True,
            # An LLM-backed triage can vary between runs, so it is not idempotent.
            "idempotentHint": False,
        },
        "requiresScope": ["mcp:tools"],
    },
]


# ---- tool body ---------------------------------------------------------------------


def run_triage_tool(
    arguments: dict[str, Any],
    *,
    invoker,
    skill_root: Path,
    output_dir: Path,
) -> dict[str, Any]:
    """Execute `cyberos.obs.execute_triage`: run triage in-process and project the verdict to an MCP result.

    Pure: the invoker and roots are injected, so this is unit-tested with a fake invoker (no LLM, no
    network). It wraps the existing `handle_triage_request`, so the request contract and the
    safe-degrade behaviour are shared with the HTTP triage endpoint rather than re-implemented.

    A validation failure (no `alert`, or `alert.name` missing) maps to an in-band tool error
    (`isError: true`) - the caller's input was bad, which is not a transport failure. A successful
    triage (including the safe-degrade verdict) returns the verdict as both a text block and
    `structuredContent`.
    """
    alert = arguments.get("alert")
    payload = {"skill": SKILL_HANDLE, "alert": alert}
    status, body = handle_triage_request(
        payload, invoker=invoker, skill_root=skill_root, output_dir=output_dir
    )
    text = json.dumps(body, ensure_ascii=False, sort_keys=True)
    if status != 200:
        return {"content": [{"type": "text", "text": text}], "isError": True}
    return {
        "content": [{"type": "text", "text": text}],
        "structuredContent": body,
        "isError": False,
    }


def run_tool(
    name: str,
    arguments: dict[str, Any],
    *,
    invoker,
    skill_root: Path,
    output_dir: Path,
) -> dict[str, Any]:
    """Dispatch one tool call. Returns an MCP tools/call result (with isError on tool failure)."""
    if name == TRIAGE_TOOL_NAME:
        return run_triage_tool(
            arguments, invoker=invoker, skill_root=skill_root, output_dir=output_dir
        )
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


def handle_rpc(
    request: dict[str, Any],
    *,
    invoker,
    skill_root: Path,
    output_dir: Path,
) -> dict[str, Any] | None:
    """Dispatch one JSON-RPC request object. Returns the response, or None for a notification."""
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
        return _rpc_result(
            req_id,
            run_tool(name, arguments, invoker=invoker, skill_root=skill_root, output_dir=output_dir),
        )
    if req_id is None:
        # Notification (e.g. notifications/initialized): no response.
        return None
    return _rpc_error(req_id, METHOD_NOT_FOUND, f"method not found: {method}")


def _make_handler(invoker, skill_root: Path, output_dir: Path):
    """Build a request-handler class bound to the server's invoker + roots."""

    class _Handler(BaseHTTPRequestHandler):
        def log_message(self, fmt: str, *args: Any) -> None:  # noqa: A003 - stdlib signature
            print(f"[triage_mcp] {self.address_string()} {fmt % args}")

        def _send_json(self, status: int, payload: Any) -> None:
            data = json.dumps(payload).encode("utf-8")
            self.send_response(status)
            self.send_header("content-type", "application/json")
            self.send_header("content-length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

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
            if not isinstance(parsed, dict):
                self._send_json(
                    200, _rpc_error(None, INVALID_REQUEST, "expected a request object")
                )
                return
            resp = handle_rpc(
                parsed, invoker=invoker, skill_root=skill_root, output_dir=output_dir
            )
            if resp is None:
                self.send_response(204)
                self.end_headers()
                return
            self._send_json(200, resp)

    return _Handler


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
    """Send a heartbeat every interval until stopped (TASK-MCP-002 DEC-2350)."""
    while not stop.wait(HEARTBEAT_INTERVAL_SECS):
        try:
            _post_module_control(gateway, "/v1/mcp/heartbeat")
        except Exception as e:  # noqa: BLE001 - keep the loop alive across transient errors
            print(f"[triage_mcp] heartbeat failed: {e}")


def _select_startup_invoker():
    """Select an invoker once at startup, degrading to None (safe-degrade verdicts) when none exists."""
    from cuo.core.invoker import select_invoker

    try:
        return select_invoker(prefer="auto")
    except RuntimeError as e:
        print(f"[triage_mcp] warning: {e}\n[triage_mcp] serving safe-degrade verdicts (confidence 0.0)")
        return None


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(description="CyberOS obs triage MCP module server")
    parser.add_argument("--listen", default="127.0.0.1:8101", help="host:port to serve /mcp on")
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
    parser.add_argument(
        "--skill-root", default=None, help="path to modules/skill (default: autodetect)"
    )
    args = parser.parse_args(argv)

    host, _, port_s = args.listen.partition(":")
    port = int(port_s or "8101")
    reachable = args.public_host or args.listen
    endpoint = f"http://{reachable}/mcp"

    skill_root = _resolve_skill_root(args.skill_root)
    output_dir = Path(tempfile.gettempdir()) / "obs-triage-mcp-out"
    output_dir.mkdir(parents=True, exist_ok=True)
    invoker = _select_startup_invoker()

    handler = _make_handler(invoker, skill_root, output_dir)
    server = ThreadingHTTPServer((host, port), handler)
    print(
        f"[triage_mcp] serving MCP on http://{args.listen}/mcp "
        f"(module={MODULE_NAME}, tool={TRIAGE_TOOL_NAME}, skill_root={skill_root})"
    )

    stop = threading.Event()
    if args.gateway:
        ok, detail = self_register(args.gateway, endpoint)
        status = "registered" if ok else "registration FAILED"
        print(f"[triage_mcp] {status} with {args.gateway}: {detail}")
        if ok:
            threading.Thread(
                target=heartbeat_loop, args=(args.gateway, stop), daemon=True
            ).start()
            print(f"[triage_mcp] heartbeating every {HEARTBEAT_INTERVAL_SECS}s")
    else:
        print("[triage_mcp] --gateway not set; serving without self-registering")

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n[triage_mcp] shutting down")
        stop.set()
        if args.gateway:
            try:
                _post_module_control(args.gateway, "/v1/mcp/deregister")
                print("[triage_mcp] deregistered with the gateway")
            except Exception as e:  # noqa: BLE001 - best-effort on shutdown
                print(f"[triage_mcp] deregister failed: {e}")
        server.shutdown()


if __name__ == "__main__":
    main()
