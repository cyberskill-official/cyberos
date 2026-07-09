"""
cyberos.core.import_ — cross-memory import (PROPOSAL.md P6).

Implements AGENTS.md v2 §14 "cross-agent interop" for the team-merge
workflow: pull memory files from someone else's memory into your own,
preserving provenance, honouring privacy classes, and recording the
import boundary in your local audit chain.

Design (per PROPOSAL.md P6):

* The foreign chain is NOT merged directly. Each imported memory becomes
  a fresh ``put`` row in the local chain, with ``extra.imported_from``
  pointing at the foreign record's content_sha256 and the source
  store's fingerprint. This keeps the local chain single-writer and
  byte-deterministic.
* Filter pipeline: ``--filter kind=decision sync_class=shareable``
  predicates can be stacked.
* Path-collision policy via ``--on-conflict {skip,overwrite,branch}``:
  - ``skip`` (default): leave the local copy untouched
  - ``overwrite``: replace with the foreign copy
  - ``branch``: import as ``<path>.from-<short-fingerprint>.md``
* Idempotent: every import records its high-water-mark
  ``last_imported_seq`` under ``manifest.imports[<source-fingerprint>]``;
  re-running only pulls the delta.
* Bracketed by ``session.start`` / ``session.end`` audit rows on the
  local chain so the import boundary is itself auditable.

Source store ingestion accepts:

* a directory (``--source /path/to/other/.cyberos/memory/store``), or
* a deterministic-export zip (``--source other.zip``); auto-extracted.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import sys
import tempfile
import time
import zipfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable, Iterable

import msgspec

from cyberos.core.frontmatter import (
    Frontmatter, looks_like_yaml, parse, parse_legacy_yaml, serialize,
)
from cyberos.core.fsync import durable_dir_sync, durable_sync
from cyberos.core.ops import _atomic_write, _check_rel_path, _sha256
from cyberos.core.walker import MmapWalker
from cyberos.core.writer import AuditRecord, Writer


@dataclass
class ImportPlan:
    """One importable item from the foreign store."""
    rel_path: str          # path inside <memory-root>
    raw_bytes: bytes       # current bytes from the foreign store
    foreign_actor: str
    foreign_seq: int
    foreign_chain: str
    op_class: str          # "put" or "delete-tombstone" (deletes get propagated)
    foreign_meta: dict = field(default_factory=dict)


@dataclass
class ImportReport:
    target_store: Path
    source_fingerprint: str
    plans: list[ImportPlan] = field(default_factory=list)
    imported: list[str] = field(default_factory=list)
    skipped: list[tuple[str, str]] = field(default_factory=list)
    branched: list[tuple[str, str]] = field(default_factory=list)  # (orig, new)
    errors: list[str] = field(default_factory=list)
    last_imported_seq: int = 0

    @property
    def ok(self) -> bool:
        return not self.errors


# --- filter parsing ----------------------------------------------------------


_FILTER_TOKEN_RE = re.compile(r"^([a-z_]+)=(.+)$")


def parse_filters(specs: list[str] | None) -> list[Callable[[ImportPlan], bool]]:
    """Compile a list of ``key=value`` filter predicates.

    Supported keys:

    * ``kind`` — match ``foreign_meta['kind']``
    * ``sync_class`` — match ``foreign_meta['sync_class']``
      (``private`` / ``shareable`` per AGENTS.md v2 §15, OR legacy
      ``local-only`` / ``publishable`` / ``shared`` / ``client-visible``)
    * ``actor`` — match ``foreign_actor`` (substring match, case-sensitive)
    * ``classification`` — match ``foreign_meta['classification']``
    """
    if not specs:
        return []
    preds: list[Callable[[ImportPlan], bool]] = []
    for spec in specs:
        m = _FILTER_TOKEN_RE.match(spec.strip())
        if not m:
            raise ValueError(f"unrecognised filter: {spec!r} (use key=value)")
        key, want = m.group(1), m.group(2)
        if key == "kind":
            preds.append(lambda p, w=want: p.foreign_meta.get("kind") == w)
        elif key == "sync_class":
            def _sc(p, w=want):
                v = p.foreign_meta.get("sync_class")
                v1 = p.foreign_meta.get("sync_class_v1")
                return v == w or v1 == w
            preds.append(_sc)
        elif key == "actor":
            preds.append(lambda p, w=want: w in p.foreign_actor)
        elif key == "classification":
            preds.append(
                lambda p, w=want: p.foreign_meta.get("classification") == w,
            )
        else:
            raise ValueError(f"unknown filter key: {key!r}")
    return preds


# --- source resolution -------------------------------------------------------


def _resolve_source(source: Path) -> tuple[Path, Path | None]:
    """Return (store-root, tempdir-or-None). If source is a zip, extract."""
    if source.is_dir():
        if (source / "manifest.json").is_file():
            return source, None
        # Allow pointing at the project root that contains the store.
        nested = source / ".cyberos" / "memory" / "store"
        if (nested / "manifest.json").is_file():
            return nested, None
        raise ValueError(
            f"{source} doesn't look like a memory store (.cyberos/memory/store)"
        )
    if source.suffix == ".zip" and source.is_file():
        td = Path(tempfile.mkdtemp(prefix="cyberos-import-"))
        with zipfile.ZipFile(source) as z:
            z.extractall(td)
        # The zip may contain the store contents at top level, OR
        # wrapped under a single dir.
        if (td / "manifest.json").is_file():
            return td, td
        for child in td.iterdir():
            if child.is_dir() and (child / "manifest.json").is_file():
                return child, td
        raise ValueError(f"zip {source} doesn't contain a manifest.json")
    raise ValueError(f"unrecognised source: {source}")


def _fingerprint(store: Path) -> str:
    """Stable identifier for a foreign store — first 16 hex of sha256(absolute path)."""
    return hashlib.sha256(str(store.resolve()).encode("utf-8")).hexdigest()[:16]


def _read_foreign_manifest(store: Path) -> dict:
    p = store / "manifest.json"
    if not p.is_file():
        return {}
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}


# --- plan construction ------------------------------------------------------


def build_plans(
    source_store: Path,
    *,
    since_seq: int = 0,
    filters: list[Callable[[ImportPlan], bool]] | None = None,
) -> list[ImportPlan]:
    """Walk the foreign binlog; emit one ImportPlan per latest-state record.

    The latest state per ``rel_path`` is what we import — earlier
    rewrites in the foreign chain don't matter; only the current file's
    bytes do.
    """
    audit_dir = source_store / "audit"
    segs = sorted(p for p in audit_dir.glob("*.binlog") if p.name != "current.binlog")
    current = audit_dir / "current.binlog"
    if current.exists():
        segs.append(current)

    # last-touch-wins per rel_path
    latest_rec: dict[str, tuple[AuditRecord, int]] = {}
    for seg in segs:
        with MmapWalker(seg) as walker:
            for _o, rec in walker.iter_records():
                seq = int(rec.extra.get("_seq", 0))
                if seq <= since_seq:
                    continue
                if rec.op in ("session.start", "session.end", "view"):
                    continue
                # rename: foreign chain tells us the path changed; we
                # follow the new path. We don't try to replay the rename
                # locally — the latest-state policy already produces the
                # right outcome.
                if rec.op in ("rename", "move"):
                    new_path = rec.extra.get("to", rec.path)
                    if rec.path in latest_rec:
                        # Replace old key with new.
                        old_rec, old_seq = latest_rec.pop(rec.path)
                        latest_rec[new_path] = (
                            msgspec.structs.replace(old_rec, path=new_path),
                            seq,
                        )
                    continue
                latest_rec[rec.path] = (rec, seq)

    plans: list[ImportPlan] = []
    for rel_path, (rec, seq) in latest_rec.items():
        abs_path = source_store / rel_path
        # delete tombstones: include them; importer can decide whether to apply
        if rec.op == "delete":
            plans.append(ImportPlan(
                rel_path=rel_path,
                raw_bytes=b"",
                foreign_actor=rec.actor,
                foreign_seq=seq,
                foreign_chain=rec.chain,
                op_class="delete-tombstone",
                foreign_meta=dict(rec.extra),
            ))
            continue
        if not abs_path.is_file():
            # File was created then renamed; should have been collapsed above.
            continue
        raw = abs_path.read_bytes()
        # Extract frontmatter for filter evaluation. Tolerant of both
        # legacy YAML and v2 JSON.
        meta: dict = {}
        if raw.startswith(b"---\n"):
            try:
                if looks_like_yaml(raw):
                    fm, _body = parse_legacy_yaml(raw)
                else:
                    fm, _body = parse(raw)
                meta = msgspec.to_builtins(fm)
                # Flatten extra so filter keys like `sync_class=shareable`
                # match values nested under frontmatter.extra.
                if isinstance(meta.get("extra"), dict):
                    for k, v in meta["extra"].items():
                        meta.setdefault(k, v)
            except Exception:  # noqa: BLE001 — schema mismatch is OK
                meta = {}
        # Merge in any tag info from the audit row's extra (kind, etc.)
        for k in ("kind", "sync_class", "sync_class_v1", "classification"):
            v = rec.extra.get(k)
            if v is not None and k not in meta:
                meta[k] = v
        plans.append(ImportPlan(
            rel_path=rel_path,
            raw_bytes=raw,
            foreign_actor=rec.actor,
            foreign_seq=seq,
            foreign_chain=rec.chain,
            op_class="put",
            foreign_meta=meta,
        ))

    # Apply filters
    if filters:
        plans = [p for p in plans if all(pred(p) for pred in filters)]
    return plans


# --- execution --------------------------------------------------------------


def execute(
    target_store: Path,
    plans: list[ImportPlan],
    *,
    fingerprint: str,
    on_conflict: str = "skip",
    map_actor: dict[str, str] | None = None,
    dry_run: bool = False,
) -> ImportReport:
    """Apply ``plans`` to ``target_store``. See :func:`run` for the full pipeline."""
    if on_conflict not in ("skip", "overwrite", "branch"):
        raise ValueError(f"on_conflict must be skip/overwrite/branch, got {on_conflict!r}")

    report = ImportReport(
        target_store=target_store,
        source_fingerprint=fingerprint,
    )
    if not plans:
        return report

    if dry_run:
        for plan in plans:
            local = target_store / plan.rel_path
            if local.exists():
                report.skipped.append(
                    (plan.rel_path, f"local exists (would {on_conflict})"),
                )
            else:
                report.imported.append(plan.rel_path)
        return report

    # Open writer; bracket import in session.start / session.end.
    with Writer(target_store) as writer:
        writer.submit(AuditRecord(
            op="session.start",
            path="",
            actor=f"cyberos-import:{fingerprint}",
            content_sha256="",
            extra={"import_source": fingerprint},
        ))

        for plan in plans:
            actor = plan.foreign_actor
            if map_actor and actor in map_actor:
                actor = map_actor[actor]

            local = target_store / plan.rel_path
            if plan.op_class == "delete-tombstone":
                if not local.is_file():
                    report.skipped.append((plan.rel_path, "no local file to tombstone"))
                    continue
                # Mirror the delete locally — emit a delete audit row.
                writer.submit(AuditRecord(
                    op="delete",
                    path=plan.rel_path,
                    actor=actor,
                    content_sha256=_sha256(local.read_bytes()),
                    extra={
                        "mode": "tombstone",
                        "imported_from": fingerprint,
                        "foreign_seq": plan.foreign_seq,
                        "foreign_chain": plan.foreign_chain,
                    },
                ))
                report.imported.append(plan.rel_path)
                report.last_imported_seq = max(report.last_imported_seq, plan.foreign_seq)
                continue

            # put
            target_path = plan.rel_path
            if local.exists():
                if on_conflict == "skip":
                    report.skipped.append((plan.rel_path, "conflict; skipped"))
                    continue
                if on_conflict == "branch":
                    target_path = _branch_name(plan.rel_path, fingerprint)
                    report.branched.append((plan.rel_path, target_path))
                # overwrite falls through

            try:
                _check_rel_path(target_path)
            except Exception as exc:  # noqa: BLE001
                report.errors.append(f"{target_path}: {exc}")
                continue

            abs_target = target_store / target_path
            try:
                _atomic_write(abs_target, plan.raw_bytes)
            except OSError as exc:
                report.errors.append(f"write {target_path}: {exc}")
                continue

            writer.submit(AuditRecord(
                op="put",
                path=target_path,
                actor=actor,
                content_sha256=_sha256(plan.raw_bytes),
                extra={
                    "kind": plan.foreign_meta.get("kind", "unknown"),
                    "imported_from": fingerprint,
                    "foreign_seq": plan.foreign_seq,
                    "foreign_chain": plan.foreign_chain,
                    "foreign_actor": plan.foreign_actor,
                },
            ))
            report.imported.append(target_path)
            report.last_imported_seq = max(report.last_imported_seq, plan.foreign_seq)

        writer.submit(AuditRecord(
            op="session.end",
            path="",
            actor=f"cyberos-import:{fingerprint}",
            content_sha256="",
            extra={
                "import_source": fingerprint,
                "imported": len(report.imported),
                "skipped": len(report.skipped),
                "branched": len(report.branched),
                "last_imported_seq": report.last_imported_seq,
            },
        ))

    return report


def _branch_name(rel_path: str, fingerprint: str) -> str:
    p = Path(rel_path)
    short_fp = fingerprint[:8]
    return str(p.with_name(f"{p.stem}.from-{short_fp}{p.suffix}"))


# --- manifest bookkeeping ---------------------------------------------------


def update_manifest_imports(target_store: Path, fingerprint: str, last_seq: int) -> None:
    """Record the high-water-mark for idempotent re-import."""
    manifest_path = target_store / "manifest.json"
    if not manifest_path.is_file():
        return
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return
    imports = manifest.setdefault("imports", {})
    entry = imports.setdefault(fingerprint, {})
    if last_seq <= entry.get("last_imported_seq", 0):
        return
    entry["last_imported_seq"] = last_seq
    entry["last_imported_at_ns"] = time.time_ns()
    body = json.dumps(manifest, sort_keys=True, indent=2).encode("utf-8") + b"\n"
    tmp = manifest_path.with_name(manifest_path.name + ".tmp")
    flags = os.O_WRONLY | os.O_CREAT | os.O_TRUNC | getattr(os, "O_CLOEXEC", 0)
    fd = os.open(tmp, flags, 0o600)
    try:
        os.write(fd, body)
        durable_sync(fd)
    finally:
        os.close(fd)
    os.replace(tmp, manifest_path)
    durable_dir_sync(target_store)


def get_last_imported_seq(target_store: Path, fingerprint: str) -> int:
    """Return the last seq imported from this source, or 0."""
    manifest_path = target_store / "manifest.json"
    if not manifest_path.is_file():
        return 0
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return 0
    return int(
        manifest.get("imports", {}).get(fingerprint, {}).get("last_imported_seq", 0)
    )


# --- top-level driver -------------------------------------------------------


def run(
    target_store: Path,
    source: Path,
    *,
    filters: list[str] | None = None,
    on_conflict: str = "skip",
    map_actor: dict[str, str] | None = None,
    since: int | None = None,
    dry_run: bool = False,
) -> ImportReport:
    """Top-level import driver. See :func:`build_plans` + :func:`execute`."""
    source_store, cleanup_dir = _resolve_source(source)
    try:
        fingerprint = _fingerprint(source_store)
        last_seq = since if since is not None else get_last_imported_seq(
            target_store, fingerprint,
        )
        compiled = parse_filters(filters)
        plans = build_plans(source_store, since_seq=last_seq, filters=compiled)
        report = execute(
            target_store, plans,
            fingerprint=fingerprint,
            on_conflict=on_conflict,
            map_actor=map_actor,
            dry_run=dry_run,
        )
        if not dry_run and report.ok:
            update_manifest_imports(
                target_store, fingerprint, report.last_imported_seq,
            )
        return report
    finally:
        if cleanup_dir is not None:
            shutil.rmtree(cleanup_dir, ignore_errors=True)


def format_report(report: ImportReport, *, dry_run: bool = False) -> str:
    lines = [f"cyberos import — {report.target_store} ← (source fp {report.source_fingerprint})"]
    label = "would import" if dry_run else "imported"
    for path in report.imported:
        lines.append(f"  {label}: {path}")
    for orig, new in report.branched:
        lines.append(f"  branched: {orig} → {new}")
    for path, reason in report.skipped:
        lines.append(f"  skip: {path} ({reason})")
    for err in report.errors:
        lines.append(f"  ERROR: {err}")
    lines.append(
        f"  total: {len(report.imported)} {label.split()[-1]}, "
        f"{len(report.skipped)} skipped, "
        f"{len(report.branched)} branched, "
        f"{len(report.errors)} errors"
    )
    if not dry_run and report.last_imported_seq:
        lines.append(f"  last_imported_seq: {report.last_imported_seq}")
    return "\n".join(lines)


__all__ = [
    "ImportPlan", "ImportReport",
    "build_plans", "execute", "run", "format_report",
    "parse_filters",
    "get_last_imported_seq", "update_manifest_imports",
]
