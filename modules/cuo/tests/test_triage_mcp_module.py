"""Tests for the obs triage MCP module (FR-OBS-007 x FR-MCP-002 federation surface).

These exercise the tool body and JSON-RPC dispatch with a fake invoker, so no LLM, no network, and
no live gateway are needed. They pin: the `tools/list` descriptor the gateway registers, the verdict
projection a successful triage returns through `tools/call`, the in-band tool error a bad alert maps
to, the safe-degrade verdict when no invoker is available, the unknown-tool error, and the
registration body posted to the gateway.
"""

from __future__ import annotations

from pathlib import Path

from cuo.core.invoker import StepResult
from cuo.triage_mcp_module import (
    MODULE_NAME,
    TRIAGE_TOOL_NAME,
    build_registration,
    handle_rpc,
    run_tool,
    run_triage_tool,
)


class FakeInvoker:
    """Records the invoke call and returns a canned StepResult (mirrors test_triage_server)."""

    def __init__(self, result: StepResult | None = None, raises: Exception | None = None):
        self._result = result
        self._raises = raises
        self.calls: list[tuple] = []

    def invoke(self, skill_name, inputs, skill_root, output_dir, step_num, *, file_prefix=""):
        self.calls.append((skill_name, inputs, skill_root, output_dir, step_num, file_prefix))
        if self._raises is not None:
            raise self._raises
        return self._result


def _ok_result(output: dict) -> StepResult:
    return StepResult(step=1, skill="obs-triage-alert", status="OK", output=output)


def _alert(**over) -> dict:
    base = {
        "name": "HighErrorRate",
        "severity": "sev2",
        "fingerprint": "fp-1",
        "trace_id": "abc123",
        "summary": "5xx above 2% for api-gateway",
    }
    base.update(over)
    return base


SKILL_ROOT = Path("/tmp/skill")
OUT = Path("/tmp/out")


def _call(arguments, invoker):
    return run_triage_tool(arguments, invoker=invoker, skill_root=SKILL_ROOT, output_dir=OUT)


def test_tools_list_advertises_the_obs_triage_tool():
    resp = handle_rpc(
        {"jsonrpc": "2.0", "id": 1, "method": "tools/list"},
        invoker=None,
        skill_root=SKILL_ROOT,
        output_dir=OUT,
    )
    tools = resp["result"]["tools"]
    assert [t["name"] for t in tools] == [TRIAGE_TOOL_NAME]
    schema = tools[0]["inputSchema"]
    assert schema["required"] == ["alert"]
    assert schema["properties"]["alert"]["required"] == ["name"]
    assert tools[0]["annotations"]["readOnlyHint"] is True


def test_successful_triage_returns_the_verdict_as_text_and_structured_content():
    out = {
        "confidence": 0.82,
        "summary": "api-gateway 5xx jumped after deploy v0.4.7.",
        "suspected_cause": "Regression in deploy v0.4.7.",
        "suggested_runbook": {"title": "Roll back gateway", "url": "https://kb/rollback"},
    }
    res = _call({"alert": _alert()}, FakeInvoker(_ok_result(out)))
    assert res["isError"] is False
    # The verdict is carried both as the structured object and as a text block.
    assert res["structuredContent"]["confidence"] == 0.82
    assert res["structuredContent"]["suggested_runbook"] == {"url": "https://kb/rollback", "title": "Roll back gateway"}
    assert '"confidence": 0.82' in res["content"][0]["text"]


def test_triage_runs_through_tools_call_dispatch():
    out = {"confidence": 0.5, "summary": "s", "suspected_cause": "c", "suggested_runbook": None}
    resp = handle_rpc(
        {
            "jsonrpc": "2.0",
            "id": 7,
            "method": "tools/call",
            "params": {"name": TRIAGE_TOOL_NAME, "arguments": {"alert": _alert()}},
        },
        invoker=FakeInvoker(_ok_result(out)),
        skill_root=SKILL_ROOT,
        output_dir=OUT,
    )
    assert resp["result"]["isError"] is False
    assert resp["result"]["structuredContent"]["confidence"] == 0.5


def test_missing_alert_is_an_in_band_tool_error():
    res = _call({}, FakeInvoker(_ok_result({})))
    assert res["isError"] is True
    assert "alert" in res["content"][0]["text"]


def test_alert_without_name_is_an_in_band_tool_error():
    res = _call({"alert": _alert(name="")}, FakeInvoker(_ok_result({})))
    assert res["isError"] is True
    assert "name" in res["content"][0]["text"]


def test_no_invoker_returns_the_safe_degrade_verdict_not_an_error():
    # No invoker available: the shared handler returns confidence 0.0 at HTTP 200, which is a
    # successful tool result (obs would page), not a transport/tool error.
    res = _call({"alert": _alert()}, None)
    assert res["isError"] is False
    assert res["structuredContent"]["confidence"] == 0.0
    assert "without its inputs" in res["structuredContent"]["summary"]


def test_unknown_tool_name_is_an_in_band_error():
    res = run_tool("cyberos.obs.nope", {}, invoker=None, skill_root=SKILL_ROOT, output_dir=OUT)
    assert res["isError"] is True
    assert "unknown tool" in res["content"][0]["text"]


def test_registration_body_advertises_module_and_tool():
    body = build_registration("http://127.0.0.1:8101/mcp")
    assert body["module"] == MODULE_NAME == "obs"
    assert body["endpoint"] == "http://127.0.0.1:8101/mcp"
    assert [t["name"] for t in body["tools"]] == [TRIAGE_TOOL_NAME]
