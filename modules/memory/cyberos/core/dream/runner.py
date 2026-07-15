"""
cyberos.core.dream.runner — orchestrates one dream pass
(TASK-MEMORY-115 §1 #1, §1 #5, §1 #6).

Lifecycle (synchronous from the operator's POV; async for detector
implementations):

1. Generate a ULID ``dream_id``.
2. Capture the HEAD seq at start (snapshot isolation per §1 #2 / §7.7.6).
3. Emit ``dream.start`` aux row on the main chain.
4. Run each requested detector (default: all four) sequentially.
   Detector failures are captured as ``dream.detector_failed`` rows but
   do not abort the run.
5. Aggregate proposals; dedup by (op, sorted-paths) to avoid two
   detectors firing on the same path.
6. Persist ``DreamDiff`` JSON at
   ``dreams/<YYYYMMDDTHHMMSSZ>/diff.json``.
7. Emit ``dream.complete`` aux row carrying the proposals_count_by_kind
   and snapshot_head metric.

No memory state is changed by the runner. Apply requires a separate
``cyberos dream apply <id>`` invocation (TASK-MEMORY-115 §1 #4).
"""

from __future__ import annotations

import asyncio
import time
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Optional, Sequence

from cyberos.core.dream.proposals import (
    DreamDiff,
    DreamProposal,
    generate_dream_id,
)


DEFAULT_DETECTORS = ("duplicates", "stale", "patterns", "verify")


async def run(
    writer,  # cyberos.core.writer.Writer
    *,
    since: timedelta = timedelta(hours=24),
    scope: str = "",
    detector_names: Sequence[str] = DEFAULT_DETECTORS,
    invoker_name: Optional[str] = None,
    dry_run: bool = False,
    duplicates_threshold: float = 0.92,
) -> DreamDiff:
    """Execute one dream pass.

    Parameters mirror the CLI surface in TASK-MEMORY-115 §1 #1 / #8.
    """
    from cyberos.core.writer import AuditRecord
    from cyberos.core.dream.detectors import (
        run_duplicates, run_stale, run_patterns, run_verify,
    )

    dream_id = generate_dream_id()
    started_at = datetime.now(timezone.utc)
    snapshot_head = writer.head_seq if hasattr(writer, "head_seq") else 0

    # dream.start aux row
    writer.submit(AuditRecord(
        op="dream.start",
        path="",
        actor="dream-runner",
        extra={
            "dream_id": dream_id,
            "scope": scope or "*",
            "since": (started_at - since).isoformat(),
            "detectors": list(detector_names),
            "invoker": invoker_name or "default",
            "started_at": started_at.isoformat(),
            "snapshot_head": snapshot_head,
            "dry_run": bool(dry_run),
        },
    ))

    registry = {
        "duplicates": lambda: run_duplicates(
            writer.store, since, scope,
            threshold=duplicates_threshold, invoker_name=invoker_name,
        ),
        "stale": lambda: run_stale(
            writer.store, since, scope, invoker_name=invoker_name,
        ),
        "patterns": lambda: run_patterns(
            writer.store, since, scope, invoker_name=invoker_name,
        ),
        "verify": lambda: run_verify(
            writer.store, since, scope, invoker_name=invoker_name,
        ),
    }

    all_proposals: list[DreamProposal] = []
    for name in detector_names:
        if name not in registry:
            raise ValueError(
                f"unknown detector {name!r}; expected one of {sorted(registry)}"
            )
        try:
            proposals = await registry[name]()
        except Exception as e:  # noqa: BLE001 — surface to chain but don't abort
            writer.submit(AuditRecord(
                op="dream.detector_failed",
                path="",
                actor="dream-runner",
                extra={
                    "dream_id": dream_id,
                    "detector": name,
                    "error": f"{type(e).__name__}: {e}",
                },
            ))
            continue
        all_proposals.extend(proposals)

    # Dedup by (op, sorted(paths)) so two detectors firing on the same
    # target don't generate duplicate proposals
    seen: set[tuple[str, tuple[str, ...]]] = set()
    dedup: list[DreamProposal] = []
    for p in all_proposals:
        key = (p.op, tuple(sorted(p.paths)))
        if key in seen:
            continue
        seen.add(key)
        dedup.append(p)

    by_kind = {"merge": 0, "stale": 0, "new": 0, "verify": 0}
    for p in dedup:
        by_kind[p.op] = by_kind.get(p.op, 0) + 1

    diff = DreamDiff(
        dream_id=dream_id,
        scope=scope or "*",
        since=(started_at - since).isoformat(),
        input_sessions=[],  # TASK-MEMORY-119 will populate when transcripts land
        proposals=dedup,
        metrics={
            "proposals_count_by_kind": by_kind,
            "snapshot_head": snapshot_head,
            "duration_ms": int((time.time() - started_at.timestamp()) * 1000),
            "dry_run": bool(dry_run),
        },
    )

    # Persist diff to disk
    out_dir = writer.store / "dreams" / started_at.strftime("%Y%m%dT%H%M%SZ")
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "diff.json").write_text(diff.to_json(), encoding="utf-8")

    # dream.complete aux row
    writer.submit(AuditRecord(
        op="dream.complete",
        path="",
        actor="dream-runner",
        extra={
            "dream_id": dream_id,
            "proposals_count": len(dedup),
            "applied_count": 0,                     # apply is a separate command
            "duration_ms": diff.metrics["duration_ms"],
            "quality_metrics": diff.metrics,
            "diff_path": str((out_dir / "diff.json").relative_to(writer.store)),
            "dry_run": bool(dry_run),
        },
    ))

    return diff


def run_sync(writer, **kwargs) -> DreamDiff:
    """Synchronous wrapper for CLI callers."""
    return asyncio.run(run(writer, **kwargs))
