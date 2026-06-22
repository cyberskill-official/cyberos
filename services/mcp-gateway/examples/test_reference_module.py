"""Smoke tests for the reference module MCP server (FR-MCP-002 example).

These exercise the exact JSON-RPC surface the gateway forwards to: a `tools/call` POST
to `/mcp` must return a `{"result": {"content": [...], "isError": false}}` envelope, which
is what the gateway's `parse_forward_response` deserialises into a `ToolsCallResult`. The
server is stdlib-only, so this runs anywhere `python3` does (no gateway needed).
"""

from __future__ import annotations

import json
import socket
import threading
import urllib.request
from http.server import ThreadingHTTPServer

import reference_module as rm


def _free_port() -> int:
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    return port


def _serve() -> tuple[ThreadingHTTPServer, str]:
    port = _free_port()
    server = ThreadingHTTPServer(("127.0.0.1", port), rm._Handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server, f"http://127.0.0.1:{port}"


def _post(base: str, body: dict) -> dict:
    req = urllib.request.Request(
        base + "/mcp",
        data=json.dumps(body).encode(),
        headers={"content-type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=5) as resp:
        return json.loads(resp.read().decode())


def test_initialize_reports_protocol_version():
    server, base = _serve()
    try:
        r = _post(base, {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}})
        assert r["result"]["protocolVersion"] == rm.MCP_PROTOCOL_VERSION
    finally:
        server.shutdown()


def test_tools_list_returns_both_demo_tools():
    server, base = _serve()
    try:
        r = _post(base, {"jsonrpc": "2.0", "id": 1, "method": "tools/list"})
        names = {t["name"] for t in r["result"]["tools"]}
        assert names == {"cyberos.demo.echo", "cyberos.demo.now"}
    finally:
        server.shutdown()


def test_tools_call_echo_round_trips_arguments():
    server, base = _serve()
    try:
        r = _post(
            base,
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {"name": "cyberos.demo.echo", "arguments": {"message": "pong"}},
            },
        )
        result = r["result"]
        assert result["isError"] is False
        text = result["content"][0]["text"]
        assert json.loads(text) == {"message": "pong"}
    finally:
        server.shutdown()


def test_tools_call_now_is_iso_utc():
    server, base = _serve()
    try:
        r = _post(
            base,
            {"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "cyberos.demo.now"}},
        )
        text = r["result"]["content"][0]["text"]
        assert text.endswith("+00:00")  # ISO-8601 UTC offset
    finally:
        server.shutdown()


def test_unknown_tool_is_in_band_error_not_transport_error():
    server, base = _serve()
    try:
        r = _post(
            base,
            {"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "cyberos.demo.nope"}},
        )
        # The module is reachable, so this is a tool-side error (isError=true), not a
        # JSON-RPC error envelope. The gateway surfaces it as a normal result.
        assert "error" not in r
        assert r["result"]["isError"] is True
    finally:
        server.shutdown()


def test_unknown_method_is_jsonrpc_method_not_found():
    server, base = _serve()
    try:
        r = _post(base, {"jsonrpc": "2.0", "id": 1, "method": "no/such/method"})
        assert r["error"]["code"] == rm.METHOD_NOT_FOUND
    finally:
        server.shutdown()


def test_build_registration_shape_matches_gateway_contract():
    reg = rm.build_registration("http://127.0.0.1:8099/mcp")
    assert reg["module"] == "demo"
    assert reg["endpoint"] == "http://127.0.0.1:8099/mcp"
    assert len(reg["tools"]) == 2
    # Wire form the gateway's RegisterTool accepts (camelCase aliases).
    t = reg["tools"][0]
    assert {"name", "description", "inputSchema", "annotations", "requiresScope"} <= set(t)
