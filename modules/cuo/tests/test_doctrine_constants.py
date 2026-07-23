"""Doctrine-constant pins — TASK-CUO-304.

ship-tasks.md §11b is the normative home of the route-back ceiling ("At
`routed_back_count >= 3`, ship-tasks MUST HALT"; "a task at routed_back_count: 2
re-enters normally"). The machine encodes the same number twice: the
``halt_on_repeat_rework`` default in ``cuo.api.run()`` and the
``--halt-on-repeat-rework`` click option (default + help text) in ``cuo/cli.py``.

These tests parse the ceiling OUT OF THE DOCTRINE TEXT — never a hardcoded
literal. A literal would be a third copy of the constant (the fork surface the
task audit's ISS-001 names): when the doctrine changes deliberately, a
literal-carrying test fails with a message that reads "the test expects 3",
inviting a test-side fix while api.py keeps drifting. Parsing makes the doc the
single source and this module a conformance check: a deliberate ceiling change
edits exactly one normative home (ship-tasks.md §11b) and these tests then name
every stale machine surface.

This module is also the designated future home for further doctrine-constant
pins — e.g. the 5-fail debugging circuit breaker, which today exists only as
workflow/skill prose with no Python constant to pin (measured 2026-07-23; see
the task spec's Alternatives Considered).
"""

from __future__ import annotations

import inspect
import re
from pathlib import Path
from types import SimpleNamespace

import pytest

import cuo.api
import cuo.core.backlog_reader
from cuo.cli import main as cli_main

_MODULE_ROOT = Path(__file__).resolve().parents[1]  # modules/cuo
_REPO_ROOT = Path(__file__).resolve().parents[3]    # repo root
SHIP_TASKS = _MODULE_ROOT / "chief-technology-officer" / "workflows" / "ship-tasks.md"
CHANGELOG = _REPO_ROOT / "CHANGELOG.md"

# The MUST-HALT bullet's shape, pinned including the `>=` comparator: if the
# doctrine is ever reworded to `>` (or the bullet moves/disappears), the parser
# MISSES and fails loud — forcing the editor to update doc, parser, and machine
# surfaces together instead of letting them fork (spec 1.4 + audit ISS-002/003).
_CEILING_RE = re.compile(r"routed_back_count\s*>=\s*(\d+)")


class DoctrineParseError(Exception):
    """§11b's MUST-HALT bullet could not be parsed — fail LOUD, never skip."""


def doctrine_ceiling(text: str) -> int:
    """Extract the route-back ceiling from ship-tasks.md §11b's MUST-HALT bullet.

    Raises DoctrineParseError (never returns a default, never skips) when the
    pattern is absent or ambiguous — a silently-skipping conformance test is the
    schema-drift failure mode this batch is also fixing (TASK-MEMORY-303), and
    it must not be reproduced here (spec 1.4).
    """
    hits: list[int] = []
    for line in text.splitlines():
        if "MUST HALT" not in line:
            continue
        m = _CEILING_RE.search(line)
        if m:
            hits.append(int(m.group(1)))
    if len(hits) != 1:
        raise DoctrineParseError(
            f"expected exactly one 'routed_back_count >= N' MUST-HALT bullet in "
            f"ship-tasks.md §11b, found {len(hits)} ({hits!r}). The doctrine wording "
            f"drifted: update this parser AND every machine surface together "
            f"(cuo/api.py run() halt_on_repeat_rework default; cuo/cli.py "
            f"--halt-on-repeat-rework default + help text)."
        )
    return hits[0]


# ── per-surface assertions (shared by the conformance tests and the AC-3
#    mismatch-message test; every failure message names BOTH sides) ───────────

def _assert_api_default(n: int) -> None:
    default = inspect.signature(cuo.api.run).parameters["halt_on_repeat_rework"].default
    assert default == n, (
        f"cuo.api.run() defaults halt_on_repeat_rework={default}, but the doctrine "
        f"ceiling (ship-tasks.md §11b) is {n} — the machine surface drifted from "
        f"the doctrine (TASK-CUO-304)"
    )


def _drain_option():
    cmd = cli_main.commands["drain"]
    return next(p for p in cmd.params if p.name == "halt_on_repeat_rework")


def _assert_cli_default(n: int) -> None:
    default = _drain_option().default
    assert default == n, (
        f"cuo/cli.py --halt-on-repeat-rework defaults to {default}, but the doctrine "
        f"ceiling (ship-tasks.md §11b) is {n} — the machine surface drifted from "
        f"the doctrine (TASK-CUO-304)"
    )


def _assert_cli_help(n: int) -> None:
    help_text = _drain_option().help or ""
    assert f"default {n}" in help_text, (
        f"cuo/cli.py --halt-on-repeat-rework help must quote the doctrine ceiling "
        f"'default {n}' (ship-tasks.md §11b) but says: {help_text!r}"
    )
    assert "Set 0 to disable" in help_text, (
        f"cuo/cli.py --halt-on-repeat-rework help no longer documents 'Set 0 to "
        f"disable' — 0-disable is contract (spec edge case) and must stay "
        f"documented; help says: {help_text!r}"
    )


def _run_default_drain(monkeypatch: pytest.MonkeyPatch, out_dir: Path, rbc: int) -> int:
    """Drive the REAL cuo.api.run() halt logic with its SIGNATURE DEFAULT ceiling.

    One eligible task; the chain outcome is stubbed to ROUTED_BACK and the
    task's routed_back_count to ``rbc``. Returns the drain's exit code:
    0 = the task re-entered (drain completed), 2 = the ceiling halted it.
    halt_on_repeat_rework is deliberately NOT passed — the default is under test.
    """
    out_dir.mkdir(parents=True, exist_ok=True)
    backlog = out_dir / "BACKLOG.md"
    backlog.write_text(
        "| TASK-ID | Title | Pri | Status | Depends on | Effort |\n"
        "| :--- | :--- | :--- | :--- | :--- | :--- |\n"
        "| TASK-CUO-901 | Ceiling fixture | High | ready_to_implement | | 1 |\n",
        encoding="utf-8",
    )
    stub = SimpleNamespace(outcome="ROUTED_BACK", step_results=[], total_duration_ms=0,
                           notes=["doctrine-constants fixture"])
    monkeypatch.setattr(cuo.api, "execute_chain", lambda **kw: stub)
    monkeypatch.setattr(cuo.api, "select_invoker", lambda name: SimpleNamespace())
    monkeypatch.setattr(cuo.core.backlog_reader, "routed_back_count",
                        lambda task_id, audit_dir: rbc)
    with pytest.raises(SystemExit) as exc:
        cuo.api.run(
            "chief-technology-officer/ship-tasks",
            output_dir=out_dir,
            backlog_path=backlog,
            memory_emit=False,
            cuo_root=_MODULE_ROOT,
            skill_root=_MODULE_ROOT.parent / "skill",
        )
    return exc.value.code or 0


# ── AC 1 (spec 1.1): api default == doctrine; rbc N-1 re-enters, rbc N halts ─

def test_api_default_matches_doctrine(monkeypatch: pytest.MonkeyPatch, tmp_path: Path,
                                      capsys: pytest.CaptureFixture) -> None:
    n = doctrine_ceiling(SHIP_TASKS.read_text(encoding="utf-8"))
    _assert_api_default(n)

    # Behavioral half (audit ISS-003: pin the `rbc >= ceiling` comparison, not just
    # the signature): one under the ceiling re-enters, at the ceiling halts.
    under = tmp_path / "under-ceiling"
    assert _run_default_drain(monkeypatch, under, rbc=n - 1) == 0, (
        f"a task at routed_back_count {n - 1} must re-enter normally "
        f"(ship-tasks.md §11b), but the default drain halted"
    )
    assert not (under / "DRAIN_HALT.md").exists(), (
        "no halt brief may be written under the ceiling"
    )
    capsys.readouterr()

    at = tmp_path / "at-ceiling"
    assert _run_default_drain(monkeypatch, at, rbc=n) == 2, (
        f"a task at routed_back_count {n} must HALT at the operator gate "
        f"(ship-tasks.md §11b), but the default drain did not"
    )
    assert (at / "DRAIN_HALT.md").exists(), "the ceiling halt must write DRAIN_HALT.md"
    out = capsys.readouterr().out
    assert f"--halt-on-repeat-rework={n}" in out, (
        f"the halt line must print the effective doctrine ceiling {n}; got: {out!r}"
    )


# ── AC 2 (spec 1.2): click default == doctrine; help quotes it + keeps 0-disable ─

def test_cli_default_and_help_match_doctrine() -> None:
    n = doctrine_ceiling(SHIP_TASKS.read_text(encoding="utf-8"))
    _assert_cli_default(n)
    _assert_cli_help(n)


# ── AC 3 (spec 1.3): the ceiling is DERIVED, and a mismatch names both sides ─

def test_mismatch_names_both_sides() -> None:
    real_text = SHIP_TASKS.read_text(encoding="utf-8")
    n_real = doctrine_ceiling(real_text)
    patched = real_text.replace(f"routed_back_count >= {n_real}",
                                f"routed_back_count >= {n_real + 1}")
    n_patched = doctrine_ceiling(patched)
    assert n_patched == n_real + 1, (
        "derivation proof failed: the parser did not pick up the in-memory doc patch "
        "— it must read the doctrine text, never a literal"
    )
    for check in (_assert_api_default, _assert_cli_default, _assert_cli_help):
        with pytest.raises(AssertionError) as exc:
            check(n_patched)
        msg = str(exc.value)
        assert str(n_patched) in msg and str(n_real) in msg, (
            f"{check.__name__} mismatch message must name BOTH the doctrine value "
            f"({n_patched}) and the stale machine value ({n_real}); got: {msg}"
        )


# ── AC 4 (spec 1.4): a parser miss raises — never skips, never defaults ──────

def test_missing_doctrine_pattern_fails_loud() -> None:
    real_text = SHIP_TASKS.read_text(encoding="utf-8")

    # (a) the MUST-HALT bullet deleted outright
    gutted = "\n".join(l for l in real_text.splitlines() if "MUST HALT" not in l)
    with pytest.raises(DoctrineParseError) as exc:
        doctrine_ceiling(gutted)
    msg = str(exc.value)
    assert "update this parser" in msg and "machine surface" in msg, (
        f"the parse failure must instruct the editor to update the parser and the "
        f"machine surfaces together; got: {msg}"
    )

    # (b) the bullet reworded past the pinned `>=` comparator (audit ISS-003):
    # a comparator change is a semantics change and must also fail loud.
    reworded = real_text.replace("routed_back_count >= ", "routed_back_count > ")
    with pytest.raises(DoctrineParseError):
        doctrine_ceiling(reworded)


# ── AC 5 (spec 1.5): the CHANGELOG top entry records the default change ──────

def test_changelog_entry_present() -> None:
    text = CHANGELOG.read_text(encoding="utf-8")
    first = re.search(r"^## .*$", text, flags=re.MULTILINE)
    assert first, "CHANGELOG.md carries no '## ' release section at all"
    rest = text[first.end():]
    nxt = re.search(r"^## ", rest, flags=re.MULTILINE)
    top_entry = text[first.start():first.end() + (nxt.start() if nxt else len(rest))]

    assert "halt-on-repeat-rework" in top_entry, (
        "CHANGELOG.md's top entry must mention `halt-on-repeat-rework` — TASK-CUO-304 "
        "1.5 requires an entry noting the default change (2 -> 3: default drains now "
        f"permit the third cycle before halting). Top entry begins: {top_entry[:120]!r}"
    )
    assert re.search(r"2\s*(?:->|→|to)\s*3", top_entry), (
        "CHANGELOG.md's top entry must state the 2-to-3 default change for "
        f"halt-on-repeat-rework (TASK-CUO-304 1.5). Top entry begins: {top_entry[:120]!r}"
    )
