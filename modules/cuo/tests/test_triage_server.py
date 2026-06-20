"""Tests for the CUO obs.triage-alert HTTP endpoint (FR-OBS-007 §1 #2).

These exercise the pure request handler with a fake invoker, so no LLM and no network are needed. They
pin the request/response contract obs-router's `cuo_triage.rs` depends on, and the safe-degradation
behaviour the skill's guardrail (SKILL.md §5) prescribes.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from cuo.core.invoker import StepResult
from cuo.triage_server import (
    SKILL_HANDLE,
    alert_to_inputs,
    extract_triage,
    handle_triage_request,
    safe_degrade,
)


class FakeInvoker:
    """Records the invoke call and returns a canned StepResult."""

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


def test_happy_path_returns_the_obs_router_contract():
    out = {
        "confidence": 0.82,
        "summary": "api-gateway 5xx jumped after deploy v0.4.7.",
        "suspected_cause": "Regression in deploy v0.4.7.",
        "suggested_runbook": {"title": "Roll back gateway", "url": "https://kb/rollback"},
    }
    inv = FakeInvoker(_ok_result(out))
    status, body = handle_triage_request(
        {"skill": SKILL_HANDLE, "alert": _alert()}, invoker=inv, skill_root=SKILL_ROOT, output_dir=OUT
    )
    assert status == 200
    assert body["confidence"] == 0.82
    assert body["summary"].startswith("api-gateway 5xx")
    assert body["suspected_cause"] == "Regression in deploy v0.4.7."
    assert body["suggested_runbook"] == {"url": "https://kb/rollback", "title": "Roll back gateway"}
    # The skill was invoked by its directory name with the alert passed through.
    assert inv.calls[0][0] == "obs-triage-alert"


def test_wrong_skill_handle_is_rejected():
    status, body = handle_triage_request(
        {"skill": "obs.something-else@1", "alert": _alert()},
        invoker=FakeInvoker(_ok_result({})),
        skill_root=SKILL_ROOT,
        output_dir=OUT,
    )
    assert status == 400
    assert "error" in body


def test_missing_alert_is_rejected():
    status, body = handle_triage_request(
        {"skill": SKILL_HANDLE}, invoker=FakeInvoker(_ok_result({})), skill_root=SKILL_ROOT, output_dir=OUT
    )
    assert status == 400


def test_alert_without_name_is_rejected():
    status, _ = handle_triage_request(
        {"skill": SKILL_HANDLE, "alert": {"severity": "sev2"}},
        invoker=FakeInvoker(_ok_result({})),
        skill_root=SKILL_ROOT,
        output_dir=OUT,
    )
    assert status == 400


def test_no_invoker_degrades_to_a_paging_verdict():
    status, body = handle_triage_request(
        {"skill": SKILL_HANDLE, "alert": _alert()}, invoker=None, skill_root=SKILL_ROOT, output_dir=OUT
    )
    assert status == 200
    assert body["confidence"] == 0.0  # obs-router pages on < 0.70
    assert "without its inputs" in body["summary"]


def test_failed_invocation_degrades_to_a_paging_verdict():
    failed = StepResult(step=1, skill="obs-triage-alert", status="FAILED", notes=["binary not on PATH"])
    status, body = handle_triage_request(
        {"skill": SKILL_HANDLE, "alert": _alert()},
        invoker=FakeInvoker(failed),
        skill_root=SKILL_ROOT,
        output_dir=OUT,
    )
    assert status == 200
    assert body["confidence"] == 0.0
    assert "binary not on PATH" in body["summary"]


def test_invoker_exception_degrades_rather_than_500():
    status, body = handle_triage_request(
        {"skill": SKILL_HANDLE, "alert": _alert()},
        invoker=FakeInvoker(raises=RuntimeError("boom")),
        skill_root=SKILL_ROOT,
        output_dir=OUT,
    )
    assert status == 200
    assert body["confidence"] == 0.0
    assert "RuntimeError" in body["summary"]


@pytest.mark.parametrize(
    "raw,expected",
    [(1.5, 1.0), (-0.3, 0.0), ("nan-ish", 0.0), (None, 0.0), (0.7, 0.7)],
)
def test_confidence_is_clamped_to_unit_interval(raw, expected):
    body = extract_triage({"confidence": raw})
    assert body["confidence"] == expected


def test_runbook_without_a_url_becomes_null():
    assert extract_triage({"confidence": 0.9, "suggested_runbook": {"title": "x"}})["suggested_runbook"] is None
    assert extract_triage({"confidence": 0.9, "suggested_runbook": {"url": "  "}})["suggested_runbook"] is None
    assert extract_triage({"confidence": 0.9})["suggested_runbook"] is None


def test_alert_to_inputs_passes_the_five_fields_through():
    inputs = alert_to_inputs(_alert(name="DiskFull", severity="sev1"))
    assert inputs["alert"]["name"] == "DiskFull"
    assert inputs["alert"]["severity"] == "sev1"
    assert set(inputs["alert"]) == {"name", "severity", "fingerprint", "trace_id", "summary"}


def test_safe_degrade_shape():
    d = safe_degrade("metrics unreachable")
    assert d["confidence"] == 0.0
    assert d["suspected_cause"] == ""
    assert d["suggested_runbook"] is None
    assert "metrics unreachable" in d["summary"]
