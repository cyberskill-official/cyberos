"""Tests for the cross-agent FR drain handoff helper."""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
from pathlib import Path
from types import SimpleNamespace

import pytest


REPO_ROOT = Path(__file__).resolve().parents[3]
SCRIPT = REPO_ROOT / "scripts" / "agent_handoff.py"


def load_handoff_module(monkeypatch: pytest.MonkeyPatch, tmp_path: Path):
    spec = importlib.util.spec_from_file_location("agent_handoff_for_test", SCRIPT)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)

    repo = tmp_path / "repo"
    backlog_dir = repo / "docs" / "feature-requests"
    backlog_dir.mkdir(parents=True)
    (repo / ".gitignore").write_text("target/\n", encoding="utf-8")
    (backlog_dir / "BACKLOG.md").write_text(
        """# Feature Request Backlog

| FR-ID | Title | Pri | Status | Depends on | Effort |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **FR-TEST-001** | Base feature | P0 | done | - | 1 |
| **FR-TEST-002** | Relay feature | P0 | ready_to_implement | FR-TEST-001 | 2 |
""",
        encoding="utf-8",
    )
    subprocess.run(["git", "init"], cwd=repo, check=True, stdout=subprocess.PIPE)
    subprocess.run(
        ["git", "config", "user.email", "agent-handoff@example.test"],
        cwd=repo,
        check=True,
    )
    subprocess.run(
        ["git", "config", "user.name", "Agent Handoff Test"],
        cwd=repo,
        check=True,
    )
    subprocess.run(["git", "add", "."], cwd=repo, check=True)
    subprocess.run(
        ["git", "commit", "-m", "test fixture"],
        cwd=repo,
        check=True,
        stdout=subprocess.PIPE,
    )

    monkeypatch.setattr(module, "REPO_ROOT", repo)
    monkeypatch.setattr(module, "BACKLOG", backlog_dir / "BACKLOG.md")
    monkeypatch.setattr(module, "DEFAULT_OUT", repo / "target" / "cuo-workflow" / "handoffs")
    monkeypatch.setattr(module, "CLAIM_DIR", repo / "target" / "cuo-workflow" / "agent-session")
    monkeypatch.setattr(module, "CLAIM_FILE", module.CLAIM_DIR / "CLAIM.json")
    return module, repo


def test_create_packet_contains_resume_and_claim_commands(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    handoff, repo = load_handoff_module(monkeypatch, tmp_path)
    args = SimpleNamespace(
        reason="usage-limit",
        agent="codex",
        handoff_to="claude-code",
        active_fr="FR-TEST-002",
        next_fr=None,
        note=["testing handoff"],
        out_dir=handoff.DEFAULT_OUT,
        release_claim=False,
        force=False,
    )

    packet = handoff.write_packet(args)
    state = json.loads((packet / "STATE.json").read_text(encoding="utf-8"))
    handoff_md = (packet / "HANDOFF.md").read_text(encoding="utf-8")

    assert state["schema"] == "cyberos.agent-handoff@1"
    assert state["active_fr"] == "FR-TEST-002"
    assert state["recommended_next_fr"]["fr_id"] == "FR-TEST-002"
    assert "resume --agent claude-code" in state["resume_command"]
    assert "claim --agent claude-code" in handoff_md
    assert (handoff.DEFAULT_OUT / "LATEST").read_text(encoding="utf-8").strip() == (
        str(packet.relative_to(repo))
    )

    validation = handoff.validate_packet_dir(packet)
    assert validation.ok, validation.errors


def test_claim_blocks_other_agent_until_released(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    handoff, _repo = load_handoff_module(monkeypatch, tmp_path)
    args = SimpleNamespace(
        reason="manual",
        agent="codex",
        handoff_to="claude-code",
        active_fr="FR-TEST-002",
        next_fr=None,
        note=[],
        out_dir=handoff.DEFAULT_OUT,
        release_claim=False,
        force=False,
    )
    packet = handoff.write_packet(args)

    handoff.write_claim(
        agent="codex",
        active_fr=None,
        packet_dir=packet,
        ttl_hours=1,
        force=False,
        reason="resume",
        notes=[],
    )
    claim = json.loads(handoff.CLAIM_FILE.read_text(encoding="utf-8"))
    assert claim["agent"] == "codex"
    assert claim["active_fr"] == "FR-TEST-002"

    with pytest.raises(handoff.HandoffError, match="active agent claim"):
        handoff.write_claim(
            agent="claude-code",
            active_fr=None,
            packet_dir=packet,
            ttl_hours=1,
            force=False,
            reason="resume",
            notes=[],
        )

    assert handoff.release_claim("codex", force=False, reason="handoff-complete") is True
    assert not handoff.CLAIM_FILE.exists()
    assert list(handoff.CLAIM_DIR.glob("RELEASED-*.json"))
