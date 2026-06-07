from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

from cyberos.core.walker import MmapWalker


def test_writer_version_cli() -> None:
    out = subprocess.run(
        [sys.executable, "-m", "cyberos.writer", "--version"],
        check=True,
        capture_output=True,
        text=True,
    )

    assert out.stdout.startswith("cyberos.writer ")
    assert "schema=1" in out.stdout


def test_writer_put_cli_routes_through_canonical_writer(tmp_path: Path) -> None:
    store = tmp_path / ".cyberos-memory"
    (store / "audit").mkdir(parents=True)
    payload = {
        "body": "---\nkind: ai.precheck\nactor: agent:cyberos-ai-gateway\n---\n",
        "meta": {
            "actor": "agent:cyberos-ai-gateway",
            "extra": {"tenant_id": "org:cyberskill"},
            "kind": "ai.precheck",
        },
        "path": "memories/decisions/ai-invocations/test.md",
    }

    out = subprocess.run(
        [sys.executable, "-m", "cyberos.writer", "--store", str(store), "put"],
        input=json.dumps(payload),
        check=True,
        capture_output=True,
        text=True,
    )

    emitted = json.loads(out.stdout)
    assert emitted["seq"] == 1
    assert len(emitted["chain"]) == 64
    assert len(emitted["prev_chain"]) == 64
    assert (store / payload["path"]).read_text(encoding="utf-8") == payload["body"]

    with MmapWalker(store / "audit" / "current.binlog") as walker:
        records = [rec for _offset, rec in walker.iter_records()]
    assert len(records) == 1
    assert records[0].op == "put"
    assert records[0].path == payload["path"]
    assert records[0].actor == "agent:cyberos-ai-gateway"
    assert records[0].extra["kind"] == "ai.precheck"
    assert records[0].extra["tenant_id"] == "org:cyberskill"
