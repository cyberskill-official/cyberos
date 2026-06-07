"""Regression tests for `cyberos init` AGENTS.md wiring."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


_REPO = Path(__file__).resolve().parent.parent


def _run_init(store: Path) -> subprocess.CompletedProcess[str]:
    """Run `cyberos --store <path> init` against the local source tree."""
    env = {
        **os.environ,
        "CYBEROS_HOST_MOUNT_PREFIX": str(store.parent),
    }
    return subprocess.run(
        [sys.executable, "-m", "cyberos", "--store", str(store), "init"],
        cwd=str(_REPO),
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )


def test_init_writes_project_agents_when_missing(tmp_path: Path) -> None:
    """`cyberos init` should create AGENTS.md in project root when absent."""
    project = tmp_path / "project"
    project.mkdir()
    store = project / ".cyberos-memory"

    proc = _run_init(store)

    assert proc.returncode == 0, proc.stderr
    assert (store / "AGENTS.md").is_file()
    assert (project / "AGENTS.md").is_file()
    assert (project / "AGENTS.md").read_bytes() == (store / "AGENTS.md").read_bytes()


def test_init_preserves_existing_project_agents(tmp_path: Path) -> None:
    """`cyberos init` should not overwrite a pre-existing project AGENTS.md."""
    project = tmp_path / "project"
    project.mkdir()
    store = project / ".cyberos-memory"
    project_agents = project / "AGENTS.md"
    project_agents.write_text("custom-agent-protocol\n", encoding="utf-8")

    proc = _run_init(store)

    assert proc.returncode == 0, proc.stderr
    assert (store / "AGENTS.md").is_file()
    assert project_agents.read_text(encoding="utf-8") == "custom-agent-protocol\n"
