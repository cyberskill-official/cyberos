"""memory bridge — emits CUO supervisor decisions to the memory audit chain.

Per cuo/docs/AGENTS.md §1.1 step 4: "Records the chain decision in the memory
audit chain per the memory module protocol (AGENTS.md §6, §11)."

This module is a thin wrapper around `cyberos.core.writer.Writer` from the
sibling memory module. It opens the writer against the local memory
(`<cyberos_root>/.cyberos/memory/store/`) and emits two row classes per workflow
execution:

  1. One **`view`** row per `StepResult` — records that the supervisor invoked
     `<skill_name>` and captured an output of hash `<output_hash>`. The op is
     `view` because the supervisor isn't writing memory files itself; it's
     witnessing skill execution. The path field encodes
     `cuo/<persona>/workflows/<workflow>.md#step-N` and `extra` carries the
     persona, workflow, step, skill, status, duration_ms, output_path.

  2. One **`session.end`** row per `ChainResult` — records that the supervisor
     completed (or BLOCKED/FAILED) a workflow chain with the per-step output
     hashes summarised. `extra` carries chain outcome, total duration,
     invoker kind, validation summary.

The writer's chain-sealing logic is preserved unchanged — every row gets
`prev_chain` linking to the prior tip + `chain` computed over the row's
canonical-JSON. The doctor invariant `ledger-mmr-cross-check` will still
pass against rows the CUO supervisor emits.

If the memory module isn't importable (sandbox without cyberos.core install,
or memory dir missing), `emit_chain_result()` becomes a no-op and returns a
NoMemoryResult with the reason. Callers should treat memory emission as best-
effort — the supervisor's primary job is workflow execution, not audit.
"""

from __future__ import annotations

import os
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from cuo.core.supervisor import ChainResult


@dataclass
class MemoryEmitResult:
    """Outcome of emitting a ChainResult to the memory audit chain."""

    emitted: bool
    rows_written: int = 0
    chain_head_after: str = ""
    notes: list[str] = field(default_factory=list)
    reason_skipped: str = ""

    def __repr__(self) -> str:
        if not self.emitted:
            return f"MemoryEmitResult(SKIPPED, reason={self.reason_skipped!r})"
        return f"MemoryEmitResult(emitted, rows={self.rows_written}, head={self.chain_head_after[:8]}...)"


def _try_import_memory_writer():
    """Best-effort import of cyberos.core.writer.

    Returns (Writer, WriterConfig, AuditRecord) tuple OR None if not importable.
    Tries:
      1. cyberos.core.writer  (when memory module installed as package)
      2. Adding <cyberos_root>/modules/memory/ to sys.path  (post-2026-05-18 layout)
      3. Adding <cyberos_root>/memory/ to sys.path           (legacy flat layout)
    """
    try:
        from cyberos.core.writer import AuditRecord, Writer, WriterConfig
        return Writer, WriterConfig, AuditRecord
    except ImportError:
        pass

    # Walk up looking for memory module under either layout
    here = Path(__file__).resolve()
    # cuo file lives at modules/cuo/cuo/core/memory_bridge.py
    # ancestors: core(0) cuo-py(1) cuo-mod(2) modules(3) cyberos-root(4) — go up generously
    candidates = []
    for ancestor in [
        here.parent.parent.parent.parent,         # cuo/ root  OR  modules/ (legacy/new)
        here.parent.parent.parent.parent.parent,  # cyberos-root  OR  legacy ancestor
        here.parent.parent.parent.parent.parent.parent,  # legacy grandparent
    ]:
        # Try modules/memory/ first (current layout)
        candidates.append(ancestor / "modules" / "memory")
        # Fallback to flat memory/ (legacy)
        candidates.append(ancestor / "memory")

    for memory_dir in candidates:
        if memory_dir.is_dir() and (memory_dir / "cyberos" / "core" / "writer.py").is_file():
            if str(memory_dir) not in sys.path:
                sys.path.insert(0, str(memory_dir))
            try:
                from cyberos.core.writer import AuditRecord, Writer, WriterConfig
                return Writer, WriterConfig, AuditRecord
            except ImportError:
                pass
    return None


def _find_memory_root(skill_root: Path) -> Path | None:
    """Locate `<cyberos_root>/.cyberos/memory/store/`.

    Under the modules/ layout (post-2026-05-18), skill_root is
    `<cyberos_root>/modules/skill/`, so memory is at `skill_root.parent.parent`.
    Under the legacy flat layout, skill_root is `<cyberos_root>/skill/`, so
    memory is at `skill_root.parent`.

    Falls back to ``CYBEROS_ROOT`` env var if neither resolves.
    """
    import os

    for candidate in (skill_root.parent.parent, skill_root.parent):
        memory = candidate / ".cyberos/memory/store"
        if memory.is_dir():
            return memory

    # Fallback: CYBEROS_ROOT env var
    env_root = os.environ.get("CYBEROS_ROOT")
    if env_root:
        memory = Path(env_root).resolve() / ".cyberos/memory/store"
        if memory.is_dir():
            return memory

    return None


def emit_chain_result(
    chain_result: "ChainResult",
    skill_root: Path,
    *,
    actor: str = "cuo-supervisor",
    memory_root: Path | None = None,
) -> MemoryEmitResult:
    """Emit a completed workflow chain to the memory audit chain.

    Writes:
      - One `view` row per StepResult (skipped steps are skipped here too).
      - One `session.end` row summarising the chain.

    Args:
        chain_result: the supervisor's ChainResult to record.
        skill_root: path to `skill/` (used to locate the sibling memory root).
        actor: actor name attached to every row. Default "cuo-supervisor".
        memory_root: optional override for `.cyberos/memory/store/` location.
            If None, auto-locates via `_find_memory_root(skill_root)`.

    Returns:
        MemoryEmitResult — `emitted=True` on success, `emitted=False` with
        `reason_skipped` if the writer is unavailable or memory missing.
    """
    # Late import to avoid hard dependency for non-memory callers.
    imported = _try_import_memory_writer()
    if imported is None:
        return MemoryEmitResult(
            emitted=False,
            reason_skipped="cyberos.core.writer not importable — memory emission requires the memory module",
        )
    Writer, WriterConfig, AuditRecord = imported

    if memory_root is None:
        memory_root = _find_memory_root(skill_root)
        if memory_root is None:
            return MemoryEmitResult(
                emitted=False,
                reason_skipped=f"memory root not found near {skill_root}",
            )

    # Make sure audit/ exists so Writer can open.
    audit_dir = memory_root / "audit"
    if not audit_dir.is_dir():
        return MemoryEmitResult(
            emitted=False,
            reason_skipped=f"memory audit/ subdir missing at {audit_dir}",
        )

    notes: list[str] = []
    rows_written = 0

    # Open writer with default config — defaults are production-safe.
    try:
        cfg = WriterConfig()
        writer = Writer(memory_root, config=cfg)
        writer.open()
    except Exception as e:  # noqa: BLE001 — writer can raise many things
        return MemoryEmitResult(
            emitted=False,
            reason_skipped=f"Writer.open() failed: {e}",
        )

    workflow_id = chain_result.workflow_id
    workflow_path = f"modules/cuo/{workflow_id}.md"

    try:
        # Per-step rows.
        for step in chain_result.step_results:
            try:
                writer.submit(AuditRecord(
                    op="view",
                    path=f"{workflow_path}#step-{step.step}",
                    actor=actor,
                    ts_ns=time.time_ns(),
                    content_sha256=step.output_hash,
                    extra={
                        "kind": "cuo.skill.invoke",
                        "workflow_id": workflow_id,
                        "step": step.step,
                        "skill": step.skill,
                        "status": step.status,
                        "duration_ms": step.duration_ms,
                        "output_path": str(step.output_path) if step.output_path else None,
                        "stderr_head": step.stderr[:200] if step.stderr else "",
                        "notes": list(step.notes),
                    },
                ))
                rows_written += 1
            except Exception as e:  # noqa: BLE001
                notes.append(f"step {step.step} row failed: {e}")

        # Workflow-complete summary row.
        per_step_summary = [
            {
                "step": s.step,
                "skill": s.skill,
                "status": s.status,
                "output_hash": s.output_hash,
            }
            for s in chain_result.step_results
        ]
        try:
            writer.submit(AuditRecord(
                op="session.end",
                path=workflow_path,
                actor=actor,
                ts_ns=time.time_ns(),
                content_sha256="0" * 64,  # no single content for the chain summary
                extra={
                    "kind": "cuo.workflow.complete",
                    "workflow_id": workflow_id,
                    "outcome": chain_result.outcome,
                    "total_duration_ms": chain_result.total_duration_ms,
                    "invoker_kind": chain_result.invoker_kind,
                    "chain_length": chain_result.validation.chain_length,
                    "steps_run": len(chain_result.step_results),
                    "missing_skills": list(chain_result.validation.missing_skills),
                    "planned_skills": list(chain_result.validation.planned_skills),
                    "per_step": per_step_summary,
                    "supervisor_version": "3.0.0a3",
                },
            ))
            rows_written += 1
        except Exception as e:  # noqa: BLE001
            notes.append(f"workflow_complete row failed: {e}")
    finally:
        try:
            writer.close()
        except Exception as e:  # noqa: BLE001
            notes.append(f"writer.close() failed: {e}")

    # Read chain head after.
    head_after = ""
    head_file = memory_root / "HEAD"
    if head_file.is_file():
        try:
            head_bytes = head_file.read_bytes()
            if len(head_bytes) >= 8:
                head_after = head_bytes[:8].hex()
        except OSError as e:
            notes.append(f"could not read HEAD: {e}")

    return MemoryEmitResult(
        emitted=rows_written > 0,
        rows_written=rows_written,
        chain_head_after=head_after,
        notes=notes,
        reason_skipped="" if rows_written > 0 else "no rows written — see notes",
    )


def memory_is_available(skill_root: Path) -> bool:
    """Check whether memory emission would succeed for this skill_root.

    Returns True only when both:
      - cyberos.core.writer is importable
      - <cyberos_root>/.cyberos/memory/store/audit/ exists
    """
    if _try_import_memory_writer() is None:
        return False
    memory = _find_memory_root(skill_root)
    if memory is None:
        return False
    return (memory / "audit").is_dir()
