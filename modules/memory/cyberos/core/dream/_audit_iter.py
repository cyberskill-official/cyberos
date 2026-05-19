"""
cyberos.core.dream._audit_iter — small adapter that yields audit rows as
dicts (op, path, actor, ts_ns, extra, …) for the detectors.

Wraps :class:`cyberos.core.walker.MmapWalker.iter_records` and converts the
msgspec ``AuditRecord`` structs into plain dicts so detector code can use
``.get("op")`` / ``.get("extra", {})`` without depending on the writer's
internal types.
"""

from __future__ import annotations

from pathlib import Path
from typing import Iterator


def iter_audit_rows(store: Path) -> Iterator[dict]:
    """Yield audit rows from the store's binlog segments as dicts.

    Iterates the current binlog (``audit/current.binlog``). Slice-4 can
    add archived-segment walking once the dream pipeline needs longer
    history; today's detectors all operate on a windowed lookback that
    fits in the current segment.
    """
    from cyberos.core.walker import MmapWalker

    audit_dir = store / "audit"
    if not audit_dir.is_dir():
        return
    current = audit_dir / "current.binlog"
    if not current.exists():
        return
    with MmapWalker(current) as walker:
        for offset, rec in walker.iter_records():
            yield {
                "op": rec.op,
                "path": rec.path,
                "actor": rec.actor,
                "ts_ns": rec.ts_ns,
                "content_sha256": rec.content_sha256,
                "prev_chain": rec.prev_chain,
                "chain": rec.chain,
                "extra": dict(rec.extra) if rec.extra else {},
            }
