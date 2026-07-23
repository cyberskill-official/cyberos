"""
Schema single-source conformance (TASK-MEMORY-303 §1.1 / §1.2).

One generator, one content, N copies:

* ``modules/memory/memory.schema.json``               (root copy — vendored)
* ``modules/memory/cyberos/data/memory.schema.json``  (package-data copy —
  what the installed Python package loads)

Both MUST be byte-identical to the generator output and MUST carry the
StoreAcl / StoreAclEntry / StoreAclMode definitions AGENTS.md §14.4.7
makes normative (AC 1). The companion drift test MUST be un-skippable:
a missing committed schema is a FAIL (AC 2).
"""

from __future__ import annotations

import hashlib
import importlib.util
import json
import re
import subprocess
import sys
from pathlib import Path

import pytest

_MEMORY = Path(__file__).resolve().parent.parent          # modules/memory/
_REPO = _MEMORY.parent.parent                              # repo root
_GENERATOR = _MEMORY / "tools" / "cyberos_generate_schema.py"
_ROOT_COPY = _MEMORY / "memory.schema.json"
_DATA_COPY = _MEMORY / "cyberos" / "data" / "memory.schema.json"
_BUILD_SH = _REPO / "tools" / "install" / "build.sh"
_DRIFT_TEST = _MEMORY / "tests" / "test_schema_drift.py"

_ACL_DEFINITIONS = ("StoreAcl", "StoreAclEntry", "StoreAclMode")


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def test_all_copies_identical_and_acl_bearing() -> None:
    """AC 1 — generator --check green; copies byte-identical; ACL defs present.

    The vendored-copy leg is proven at the source: every
    ``memory.schema.json`` path that ``tools/install/build.sh`` vendors
    from must resolve to a file byte-identical to the canonical
    package-data copy. This holds regardless of which tracked copy the
    build script references, because unification makes them one content —
    and it keeps failing if a future edit forks any referenced copy.
    (Executing a full scratch payload build is the final verification
    pass's job; the source-level check pins the same contract.)
    """
    # (a) generator --check exits 0 against the root copy
    result = subprocess.run(
        [sys.executable, str(_GENERATOR), "--check", "--out", str(_ROOT_COPY)],
        cwd=str(_MEMORY),
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, (
        f"generator --check failed against root copy:\n{result.stderr}"
    )

    # (b) root and package-data copies are byte-identical
    root_hash = _sha256(_ROOT_COPY)
    data_hash = _sha256(_DATA_COPY)
    assert root_hash == data_hash, (
        "schema fork: root copy and package-data copy differ\n"
        f"  {_ROOT_COPY}: {root_hash}\n"
        f"  {_DATA_COPY}: {data_hash}"
    )

    # (c) every schema source build.sh vendors from is the canonical content
    assert _BUILD_SH.is_file(), f"build.sh not found at {_BUILD_SH}"
    build_text = _BUILD_SH.read_text(encoding="utf-8")
    referenced = re.findall(r'\$repo/([^"\s]*memory\.schema\.json)', build_text)
    assert referenced, (
        "build.sh no longer references any memory.schema.json vendoring "
        "source — payload schema vendoring was silently dropped"
    )
    for rel in set(referenced):
        src = _REPO / rel
        assert src.is_file(), (
            f"build.sh vendors the schema from {rel}, which does not exist"
        )
        assert _sha256(src) == data_hash, (
            f"build.sh vendors the schema from {rel}, whose content "
            "differs from the canonical package-data copy — the payload "
            "would ship a forked schema"
        )

    # (d) the root copy carries the three §14.4.7 StoreAcl definitions
    defs = json.loads(_ROOT_COPY.read_text(encoding="utf-8"))["definitions"]
    for name in _ACL_DEFINITIONS:
        assert name in defs, (
            f"root copy missing normative definition {name!r} (§14.4.7)"
        )


def test_drift_test_cannot_skip() -> None:
    """AC 2 — the drift test executes (0 skips) and FAILs when the schema
    is missing, rather than skipping."""
    # (a) the drift test collects and runs with zero skips on this repo
    result = subprocess.run(
        [sys.executable, "-m", "pytest", str(_DRIFT_TEST), "-v", "--tb=no"],
        cwd=str(_MEMORY),
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, (
        f"drift test suite did not pass:\n{result.stdout[-2000:]}"
    )
    assert "skipped" not in result.stdout.lower(), (
        f"drift test suite skipped at least one test:\n{result.stdout[-2000:]}"
    )

    # (b) monkeypatching _COMMITTED to a missing path makes it FAIL, not skip
    spec = importlib.util.spec_from_file_location(
        "_drift_under_test", _DRIFT_TEST,
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module._COMMITTED = _MEMORY / "does" / "not" / "exist.schema.json"

    raised: BaseException | None = None
    try:
        module.test_committed_schema_matches_generator_output()
    except BaseException as exc:  # noqa: BLE001 — need the outcome class
        raised = exc
    assert raised is not None, (
        "drift test did not fail on a missing committed schema"
    )
    assert isinstance(raised, pytest.fail.Exception), (
        f"drift test raised {type(raised).__name__} instead of failing — "
        "a skip (or any non-failure outcome) on the trigger condition "
        "rebuilds the silent-green defect"
    )
