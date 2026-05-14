"""Memory bridge — verify the on-disk write contract."""

from __future__ import annotations

import json
from pathlib import Path

from cuo.core.memory_bridge import record_decision


def test_record_decision_writes_under_cuo_decisions(tmp_path):
    decision = {
        "skill_name": "vn-mst-validate",
        "confidence": 0.8,
        "arguments": {"input": "0312345678"},
        "rationale": "keyword `mst`",
        "alternative_skills": [],
    }
    result = {
        "skill_name": "vn-mst-validate",
        "exit_code": 0,
        "output": '{"ok": true, "kind": "entity"}',
        "stderr": "",
    }

    written = record_decision(decision, result, tmp_path)
    assert written.exists()
    expected_dir = tmp_path / ".cyberos-memory" / "meta" / "cuo-decisions"
    assert written.parent == expected_dir

    body = written.read_text(encoding="utf-8")
    assert "CUO routing decision" in body
    assert "vn-mst-validate" in body
    assert "0312345678" in body


def test_record_decision_preserves_unicode(tmp_path):
    decision = {
        "skill_name": "vn-vat-invoice",
        "confidence": 0.5,
        "arguments": {"buyer": "Công ty XYZ"},
        "rationale": "keyword `hoá đơn`",
        "alternative_skills": [],
    }
    result = {"skill_name": "vn-vat-invoice", "exit_code": 0, "output": "", "stderr": ""}
    written = record_decision(decision, result, tmp_path)
    body = written.read_text(encoding="utf-8")
    assert "Công ty XYZ" in body
