"""Invoker tests — subprocess is mocked; we never actually run a skill."""

from __future__ import annotations

import subprocess
from pathlib import Path
from unittest.mock import patch

from cuo.core.invoker import InvocationResult, invoke


def test_invoke_passes_input_as_stdin(tmp_path):
    skill_root = tmp_path / "skill" / "skills"
    skill_root.mkdir(parents=True)

    fake_completed = subprocess.CompletedProcess(
        args=[], returncode=0, stdout='{"ok": true}', stderr=""
    )
    with patch("cuo.core.invoker.subprocess.run", return_value=fake_completed) as run:
        result = invoke("vn-mst-validate", "0312345678", skill_root)

    assert isinstance(result, InvocationResult)
    assert result.exit_code == 0
    assert result.output == '{"ok": true}'
    # subprocess.run got input="0312345678"
    call_kwargs = run.call_args.kwargs
    assert call_kwargs["input"] == "0312345678"
    # And the run subcommand was used with the right skill name.
    args = run.call_args.args[0]
    assert "run" in args
    assert "vn-mst-validate" in args


def test_invoke_serialises_dict_input(tmp_path):
    skill_root = tmp_path / "skill" / "skills"
    skill_root.mkdir(parents=True)

    fake_completed = subprocess.CompletedProcess(
        args=[], returncode=0, stdout="", stderr=""
    )
    with patch("cuo.core.invoker.subprocess.run", return_value=fake_completed) as run:
        invoke("some-skill", {"a": 1, "b": "two"}, skill_root)

    payload = run.call_args.kwargs["input"]
    assert '"a"' in payload and '"b"' in payload


def test_invoke_handles_none_input(tmp_path):
    skill_root = tmp_path / "skill" / "skills"
    skill_root.mkdir(parents=True)

    fake_completed = subprocess.CompletedProcess(
        args=[], returncode=2, stdout="", stderr="boom"
    )
    with patch("cuo.core.invoker.subprocess.run", return_value=fake_completed) as run:
        result = invoke("some-skill", None, skill_root)

    assert run.call_args.kwargs["input"] == ""
    assert result.exit_code == 2
    assert result.stderr == "boom"
