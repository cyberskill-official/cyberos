"""Compatibility CLI for subprocess memory writes.

This module backs ``python3 -m cyberos.writer`` for Rust services that need a
small, stable subprocess surface over the canonical Layer-1 writer. It never
touches ``audit/`` or ``HEAD`` directly; all mutations route through
``cyberos.core.ops`` and ``cyberos.core.writer.Writer``.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any


SEMVER = "0.1.0"
SCHEMA_VERSION = 1


def _store(args: argparse.Namespace) -> Path:
    explicit = args.store
    if explicit:
        return Path(explicit).resolve()
    env_store = os.environ.get("CYBEROS_STORE")
    if env_store:
        return Path(env_store).resolve()
    cwd = Path.cwd().resolve()
    for parent in (cwd, *cwd.parents):
        candidate = parent / ".cyberos-memory"
        if candidate.is_dir():
            return candidate
    return (cwd / ".cyberos-memory").resolve()


def _last_record(store: Path, seq: int):
    from cyberos.core.walker import MmapWalker

    with MmapWalker(store / "audit" / "current.binlog") as walker:
        for _offset, rec in walker.iter_records():
            if int(rec.extra.get("_seq", -1)) == seq:
                return rec
    raise RuntimeError(f"committed seq {seq} not found in current.binlog")


def _payload_from_stdin() -> dict[str, Any]:
    try:
        payload = json.load(sys.stdin)
    except json.JSONDecodeError as exc:
        raise ValueError(f"stdin is not valid JSON: {exc}") from exc
    if not isinstance(payload, dict):
        raise ValueError("stdin payload must be a JSON object")
    return payload


def _parse_payload(payload: dict[str, Any], args: argparse.Namespace):
    path = payload.get("path")
    body = payload.get("body")
    meta = payload.get("meta")
    if not isinstance(path, str) or not path:
        raise ValueError("payload.path must be a non-empty string")
    if not isinstance(body, str):
        raise ValueError("payload.body must be a string")
    if not isinstance(meta, dict):
        raise ValueError("payload.meta must be an object")

    actor = meta.get("actor") or args.actor or "agent:cyberos-writer"
    if not isinstance(actor, str):
        raise ValueError("payload.meta.actor must be a string when present")
    kind = meta.get("kind") or "unknown"
    if not isinstance(kind, str):
        raise ValueError("payload.meta.kind must be a string when present")
    extra = meta.get("extra")
    if extra is not None and not isinstance(extra, dict):
        raise ValueError("payload.meta.extra must be an object when present")
    return path, body, actor, kind, extra


def _emit_payload(writer, payload: dict[str, Any], args: argparse.Namespace) -> dict[str, Any]:
    from cyberos.core.ops import put_with_record

    path, body, actor, kind, extra = _parse_payload(payload, args)
    seq, rec = put_with_record(
        writer,
        path,
        body.encode("utf-8"),
        actor=actor,
        kind=kind,
        extra=extra,
    )
    return {
        "seq": seq,
        "ts_ns": rec.ts_ns,
        "prev_chain": rec.prev_chain,
        "chain": rec.chain,
    }


def _cmd_put(args: argparse.Namespace) -> int:
    from cyberos.core.writer import Writer

    payload = _payload_from_stdin()

    store = _store(args)
    with Writer(store) as writer:
        row = _emit_payload(writer, payload, args)

    sys.stdout.write(
        json.dumps(
            row,
            sort_keys=True,
            separators=(",", ":"),
        )
        + "\n"
    )
    return 0


def _cmd_stream(args: argparse.Namespace) -> int:
    from cyberos.core.writer import Writer

    store = _store(args)
    with Writer(store) as writer:
        for raw in sys.stdin:
            raw = raw.strip()
            if not raw:
                continue
            try:
                payload = json.loads(raw)
                if not isinstance(payload, dict):
                    raise ValueError("stdin payload must be a JSON object")
                row = _emit_payload(writer, payload, args)
                out = row
            except BaseException as exc:  # noqa: BLE001 - surface structured subprocess errors
                out = {
                    "error": f"{type(exc).__name__}: {exc}",
                }
            sys.stdout.write(json.dumps(out, sort_keys=True, separators=(",", ":")) + "\n")
            sys.stdout.flush()
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="cyberos.writer")
    parser.add_argument("--store", default=None, help="path to .cyberos-memory")
    parser.add_argument("--actor", default=None, help="fallback actor for audit rows")
    parser.add_argument(
        "--version",
        action="store_true",
        help="print writer subprocess interface version",
    )
    sub = parser.add_subparsers(dest="cmd")
    put_parser = sub.add_parser("put", help="read one canonical put payload from stdin")
    put_parser.set_defaults(fn=_cmd_put)
    stream_parser = sub.add_parser(
        "stream",
        help="read canonical put payloads as JSON Lines from stdin",
    )
    stream_parser.set_defaults(fn=_cmd_stream)
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.version:
        print(f"cyberos.writer {SEMVER} sha=unknown schema={SCHEMA_VERSION}")
        return 0
    if not args.cmd:
        parser.error("a subcommand is required unless --version is used")
    try:
        return args.fn(args)
    except Exception as exc:  # noqa: BLE001 - subprocess callers need stderr text.
        sys.stderr.write(f"cyberos.writer: {type(exc).__name__}: {exc}\n")
        return 1


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())
