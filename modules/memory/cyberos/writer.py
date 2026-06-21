"""cyberos.writer - the FR-AI-003 audit-bridge Writer CLI.

The AI Gateway's `memory_writer.rs` (FR-AI-003) spawns `python3 -m cyberos.writer put`, pipes one line of
canonical JSON `{path, body, meta}` on stdin, and reads back one line of `{seq, ts_ns, chain, prev_chain}`
on stdout. The gateway then recomputes the chain as `SHA-256(payload_bytes || prev_chain)` and rejects the
row if it does not match (FR-AI-003 §1 #5). So the chain MUST be computed exactly that way, over the exact
payload bytes that arrived on stdin, with `prev_chain` taken from this ledger's head.

This module is deliberately stdlib-only and self-contained: it does not import `cyberos.core.writer` (the
group-commit binlog), because that ledger hashes the decomposed `AuditRecord`, whereas FR-AI-003 hashes
the wire payload - two different chain definitions. Keeping the AI-audit chain separate makes the bridge
functional and verifiable now; unifying it with the memory module's main L1 ledger (one chain model for
both) is a follow-up architecture decision tracked in docs/KNOWN-ISSUES.md.

Exit codes (FR-AI-003 subprocess handshake): 0 success, 1 schema rejection, 2 lock contention, 3 path
traversal. On a non-zero exit, stderr carries `{"code": "<id>", "detail": "<text>"}`.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
import time
from pathlib import Path

__version__ = "1.0.0"

GENESIS_CHAIN = "00" * 32
LOCK_TIMEOUT_S = 4.0
RESERVED_PREFIXES = ("audit/", "index/")


def _ledger_root() -> Path:
    """The AI-audit ledger root. Configurable so tests and deploys can isolate it."""
    root = os.environ.get("CYBEROS_AI_AUDIT_ROOT") or os.environ.get("CYBEROS_MEMORY_ROOT")
    if root:
        return Path(root)
    return Path.home() / ".cyberos-ai-audit"


def _err(code: str, detail: str, exit_code: int) -> int:
    """Print a JSON error to stderr and return the process exit code."""
    sys.stderr.write(json.dumps({"code": code, "detail": detail}) + "\n")
    return exit_code


def _validate_path(path: str) -> str | None:
    """Return a rejection reason if `path` is unsafe, else None (FR-AI-003 §1 #7 / AC #7)."""
    if not path:
        return "empty path"
    if path.startswith("/"):
        return "absolute path"
    if ".." in Path(path).parts:
        return "traversal"
    if any(path.startswith(p) for p in RESERVED_PREFIXES):
        return "reserved directory"
    return None


def _cmd_put(payload_bytes: bytes) -> int:
    # 1. Parse + validate the payload (schema rejection -> exit 1).
    try:
        payload = json.loads(payload_bytes.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as e:
        return _err("schema", f"invalid JSON payload: {e}", 1)
    if not isinstance(payload, dict) or "path" not in payload or "body" not in payload:
        return _err("schema", "payload must be an object with 'path' and 'body'", 1)

    path = str(payload["path"])
    reason = _validate_path(path)
    if reason is not None:
        return _err("path_rejected", reason, 3)

    root = _ledger_root()
    try:
        root.mkdir(parents=True, exist_ok=True)
        (root / "memories").mkdir(parents=True, exist_ok=True)
    except OSError as e:
        return _err("io", f"cannot create ledger root {root}: {e}", 1)

    # 2. Single-writer lock for serialised, contiguous seq under concurrency (AC #2).
    import fcntl

    lock_path = root / ".writer.lock"
    deadline = time.monotonic() + LOCK_TIMEOUT_S
    lock_fd = os.open(str(lock_path), os.O_CREAT | os.O_RDWR, 0o644)
    try:
        while True:
            try:
                fcntl.flock(lock_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except BlockingIOError:
                if time.monotonic() >= deadline:
                    return _err("lock", "could not acquire writer lock", 2)
                time.sleep(0.01)

        # 3. Read head (seq, chain) or genesis.
        head_path = root / "head.json"
        if head_path.exists():
            head = json.loads(head_path.read_text(encoding="utf-8"))
            prev_seq = int(head["seq"])
            prev_chain = str(head["chain"])
        else:
            prev_seq = 0
            prev_chain = GENESIS_CHAIN

        # 4. Compute the chain EXACTLY as the gateway recomputes it: SHA-256(payload || prev_chain_bytes).
        seq = prev_seq + 1
        ts_ns = time.time_ns()
        chain = hashlib.sha256(payload_bytes + bytes.fromhex(prev_chain)).hexdigest()

        # 5. Append the record and the memory file, then advance head atomically.
        record = {
            "seq": seq,
            "ts_ns": ts_ns,
            "chain": chain,
            "prev_chain": prev_chain,
            "path": path,
            "meta": payload.get("meta"),
        }
        with (root / "ledger.jsonl").open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(record, sort_keys=True) + "\n")

        body = payload.get("body", "")
        body_file = root / "memories" / path
        try:
            body_file.parent.mkdir(parents=True, exist_ok=True)
            body_file.write_text(body if isinstance(body, str) else str(body), encoding="utf-8")
        except OSError:
            pass  # the chain row is the source of truth; the body file is a convenience mirror.

        tmp = head_path.with_suffix(".json.tmp")
        tmp.write_text(json.dumps({"seq": seq, "chain": chain}), encoding="utf-8")
        os.replace(tmp, head_path)
    finally:
        os.close(lock_fd)

    # 6. Emit the contract response on stdout.
    sys.stdout.write(
        json.dumps({"seq": seq, "ts_ns": ts_ns, "chain": chain, "prev_chain": prev_chain}) + "\n"
    )
    sys.stdout.flush()
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="cyberos.writer", description="FR-AI-003 audit-bridge Writer")
    parser.add_argument("--version", action="version", version=f"cyberos.writer {__version__}")
    sub = parser.add_subparsers(dest="cmd")
    sub.add_parser("put", help="append one audit row; reads {path,body,meta} JSON on stdin")
    args = parser.parse_args(argv)

    if args.cmd == "put":
        data = sys.stdin.buffer.read()
        if data.endswith(b"\n"):
            data = data[:-1]
        return _cmd_put(data)

    parser.print_help(sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
