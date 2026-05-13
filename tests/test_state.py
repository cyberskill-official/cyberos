"""
State-machine transition tests for ``cyberos state``.

AGENTS.md v2 §12 defines three states:

* ``READY`` — all invariants pass.
* ``FROZEN_RECOVERABLE`` — at least one error-level invariant failed,
  but the failure can be repaired by tooling (e.g. layout mismatch).
* ``FROZEN_HUMAN`` — catastrophic: chain corrupt, manifest unparseable,
  MMR cross-check failed. Requires explicit human steps.

Each test synthesises a store that triggers exactly one failure mode,
then runs ``cyberos state`` via subprocess and asserts the classification.
"""

from __future__ import annotations

import hashlib
import json
import os
import struct
import subprocess
import sys
from pathlib import Path

import pytest


_REPO = Path(__file__).resolve().parent.parent


def _cyberos_state(store: Path) -> tuple[int, str, str]:
    """Run `cyberos state --json` and return (rc, stdout, stderr)."""
    env = {
        **os.environ,
        "CYBEROS_HOST_MOUNT_PREFIX": "/",  # exempt sandbox paths
    }
    proc = subprocess.run(
        [sys.executable, "-m", "cyberos", "--store", str(store), "state", "--json"],
        cwd=str(_REPO),
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )
    return proc.returncode, proc.stdout, proc.stderr


def _bootstrap_v2_store(tmp_path: Path) -> Path:
    """Make a minimal v2 store that the doctor's full invariant set passes on."""
    store = tmp_path / ".cyberos-memory"
    (store / "audit").mkdir(parents=True)
    store.joinpath("manifest.json").write_text(
        json.dumps({"schema_version": 2}), encoding="utf-8",
    )
    return store


# --- READY ---------------------------------------------------------------


def test_ready_on_pristine_v2_store(tmp_path: Path) -> None:
    store = _bootstrap_v2_store(tmp_path)
    rc, stdout, _stderr = _cyberos_state(store)
    assert rc == 0, f"expected READY (rc=0), got rc={rc}"
    result = json.loads(stdout)
    assert result["state"] == "READY"


# --- FROZEN_HUMAN — catastrophic invariants ------------------------------


def test_frozen_human_when_manifest_missing(tmp_path: Path) -> None:
    store = tmp_path / ".cyberos-memory"
    (store / "audit").mkdir(parents=True)
    # No manifest.json → manifest-schema-version invariant fails.
    rc, stdout, _stderr = _cyberos_state(store)
    assert rc == 1
    result = json.loads(stdout)
    assert result["state"] == "FROZEN_HUMAN", (
        f"expected FROZEN_HUMAN for missing manifest, got {result['state']}: {result['reason']}"
    )
    assert "manifest" in result["reason"].lower()


def test_frozen_human_when_manifest_unparseable(tmp_path: Path) -> None:
    store = _bootstrap_v2_store(tmp_path)
    store.joinpath("manifest.json").write_text("not valid json", encoding="utf-8")
    rc, stdout, _stderr = _cyberos_state(store)
    assert rc == 1
    result = json.loads(stdout)
    assert result["state"] == "FROZEN_HUMAN"


def test_frozen_human_when_bridge_tampered(tmp_path: Path) -> None:
    """Bridge cross-check is catastrophic — silent chain divergence."""
    store = _bootstrap_v2_store(tmp_path)
    # Set a legacy_last_chain, then write a binlog whose first record's
    # prev_chain doesn't match it.
    bridge = "a" * 64
    store.joinpath("manifest.json").write_text(
        json.dumps({
            "schema_version": 2,
            "migration": {"legacy_last_chain": bridge},
        }),
        encoding="utf-8",
    )
    # Use the Writer to populate; its first prev_chain WILL be `bridge`
    # (correct). Then tamper the manifest to point at a DIFFERENT bridge.
    from cyberos.core.writer import AuditRecord, Writer
    with Writer(store) as w:
        w.submit(AuditRecord(op="view", path="m.md", actor="t",
                             content_sha256="0" * 64))
    # Now tamper the bridge — the first row's stored prev_chain no longer
    # matches what the manifest claims.
    m = json.loads(store.joinpath("manifest.json").read_text(encoding="utf-8"))
    m["migration"]["legacy_last_chain"] = "b" * 64
    store.joinpath("manifest.json").write_text(
        json.dumps(m), encoding="utf-8",
    )
    rc, stdout, _stderr = _cyberos_state(store)
    assert rc == 1
    result = json.loads(stdout)
    assert result["state"] == "FROZEN_HUMAN", result
    assert "bridge" in result["reason"].lower()


def test_frozen_human_when_chain_link_broken(tmp_path: Path) -> None:
    """Splice a record into the binlog whose prev_chain is wrong."""
    store = _bootstrap_v2_store(tmp_path)
    from cyberos.core.writer import AuditRecord, Writer, _FRAME_HDR, _crc32c
    with Writer(store) as w:
        for i in range(3):
            w.submit(AuditRecord(op="view", path=f"x{i}.md", actor="t",
                                 content_sha256="0" * 64))

    # Append a frame whose payload claims a wrong prev_chain.
    import msgspec
    bad_rec = AuditRecord(
        op="view", path="bad.md", actor="t", content_sha256="0" * 64,
        prev_chain="z" * 64,  # wrong
        chain="z" * 64,  # also wrong but the LINK check fires first
    )
    payload = msgspec.json.Encoder(order="sorted").encode(bad_rec)
    header = _FRAME_HDR.pack(len(payload), _crc32c(payload), 99, 0)
    with open(store / "audit" / "current.binlog", "ab") as fh:
        fh.write(header + payload)

    rc, stdout, _stderr = _cyberos_state(store)
    assert rc == 1
    result = json.loads(stdout)
    assert result["state"] == "FROZEN_HUMAN"


# --- FROZEN_RECOVERABLE — fixable invariants -----------------------------


def test_frozen_recoverable_on_layout_drift_alone(tmp_path: Path) -> None:
    """The layout-shard-uniformity is a WARN not an ERROR — so on its own
    it should NOT push to FROZEN_*. This test pins the level."""
    store = _bootstrap_v2_store(tmp_path)
    # Put a memory file directly under memories/decisions/ (un-resharded).
    target = store / "memories" / "decisions" / "DEC-x.md"
    target.parent.mkdir(parents=True)
    target.write_text(
        '---\n{"id":"DEC-x","kind":"decision","ts_ns":1,"actor":"t","tags":[],"extra":{}}\n'
        '---\n# body\n',
        encoding="utf-8",
    )
    rc, stdout, _stderr = _cyberos_state(store)
    # WARN doesn't push to FROZEN — state should still be READY.
    result = json.loads(stdout)
    assert result["state"] == "READY", (
        f"layout WARN alone should leave state READY, got {result}"
    )


# --- error-but-recoverable -----------------------------------------------


def test_frozen_recoverable_when_only_op_enum_violation(tmp_path: Path) -> None:
    """An op-enum violation is error-level but recoverable via tooling.

    To isolate the op-enum invariant from MMR / chain failures, this test
    builds a binlog from scratch via the writer with the MMR disabled,
    then splices in a single bad-op record whose chain LINK + HASH still
    verify. The result fails ONLY ledger-op-enum-conformance.
    """
    store = _bootstrap_v2_store(tmp_path)
    from cyberos.core.writer import (
        AuditRecord, Writer, WriterConfig, _FRAME_HDR, _crc32c,
    )

    # Append a normal row with MMR disabled — no peaks.bin will exist,
    # so ledger-mmr-cross-check skips with "no MMR persisted".
    with Writer(store, config=WriterConfig(enable_mmr=False)) as w:
        w.submit(AuditRecord(op="view", path="x.md", actor="t",
                             content_sha256="0" * 64))

    # Splice a record with an off-enum `op` — but chain-valid.
    import msgspec
    from cyberos.core.writer import _canonical, _chain_hash
    from cyberos.core.walker import MmapWalker

    with MmapWalker(store / "audit" / "current.binlog") as walker:
        last_chain = ""
        for _o, rec in walker.iter_records():
            last_chain = rec.chain

    bad_rec = AuditRecord(
        op="DEFINITELY_NOT_AN_OP",
        path="x.md", actor="t", content_sha256="0" * 64,
        prev_chain=last_chain,
    )
    chain = _chain_hash(last_chain, bad_rec)
    final = msgspec.structs.replace(bad_rec, chain=chain)
    payload = _canonical(final)
    header = _FRAME_HDR.pack(len(payload), _crc32c(payload), 99, 0)
    with open(store / "audit" / "current.binlog", "ab") as fh:
        fh.write(header + payload)

    rc, stdout, _stderr = _cyberos_state(store)
    assert rc == 1
    result = json.loads(stdout)
    assert result["state"] == "FROZEN_RECOVERABLE", (
        f"op-enum violation should be FROZEN_RECOVERABLE, got {result}"
    )
    assert "op-enum" in result["reason"] or "DEFINITELY_NOT_AN_OP" in result["reason"]
