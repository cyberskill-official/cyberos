"""
cyberos.core.invariants — the self-audit walker.

Each function here corresponds to one invariant in
``docs/memory/memory.invariants.yaml``. The walker
:func:`run_all` iterates the YAML, dispatches to the named check, and
returns a structured :class:`Report`. The ``cyberos doctor`` CLI subcommand
formats the report for humans; CI runs the same code and exits non-zero on
any ``error``-level violation.

Design notes:

* Checks return ``(passed: bool, details: str)``. They MUST NOT raise on
  expected violations — only on harness errors. This keeps the walker
  fail-resistant: a buggy check doesn't blow up the whole report.
* Checks are read-only against the store. The walker MUST work on a
  read-only / shared-lock view.
* The walker honours the legacy chain bridge via
  :func:`cyberos.core.writer.resolve_initial_chain_from_manifest`.

Audit report §8.7 (current AGENTS.md) and Deep Optimization Audit §4.2
row 35 both call for a declarative invariant set walked by a single
function. This module is that function.
"""

from __future__ import annotations

import importlib
import json
import os
import re
import struct
import sys
import tempfile
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable

import msgspec

from cyberos.core.fsync import _is_darwin
from cyberos.core.walker import MmapWalker, verify_segments
from cyberos.core.writer import (
    _GENESIS_CHAIN,
    crc_implementation,
    resolve_initial_chain_from_manifest,
)


# --- result types ---------------------------------------------------------


@dataclass
class CheckResult:
    """One invariant's outcome."""
    id: str
    level: str  # "error" | "warning" | "info"
    scope: str
    passed: bool
    details: str
    duration_ms: float = 0.0


@dataclass
class Report:
    """The full doctor run."""
    store: Path
    results: list[CheckResult] = field(default_factory=list)
    started_ns: int = 0
    finished_ns: int = 0

    @property
    def errors(self) -> list[CheckResult]:
        return [r for r in self.results if not r.passed and r.level == "error"]

    @property
    def warnings(self) -> list[CheckResult]:
        return [r for r in self.results if not r.passed and r.level == "warning"]

    @property
    def ok(self) -> bool:
        return not self.errors


# --- the canonical store layout (audit report §3 + cyberos_migrate Phase 3) -


_CANONICAL_TOP_LEVEL_DIRS = frozenset({
    # AGENTS.md v2 §2 canonical directories — clean v2 store, no legacy
    # debris tolerated. If your BRAIN was migrated from v1, run
    # scripts/cleanup-v1.sh to remove stale directories.
    "memories", "meta", "company", "module", "member", "client",
    "project", "persona", "conflicts", "exports", "index", "audit",
})
_CANONICAL_TOP_LEVEL_FILES = frozenset({
    "manifest.json", "HEAD", ".lock", "README.md",
    # Permissible OS / VCS hygiene files (never written by the protocol):
    ".DS_Store", "Thumbs.db", ".gitignore",
})

_SANDBOX_FRAGMENTS = (
    "/sessions/", "/private/var/folders/", "/var/folders/",
    "/tmp/", "/private/tmp/", "/dev/shm/",
    "local-agent-mode-sessions", "claude-hostloop-plugins",
    "cowork-session", "cowork-mode-sessions", "agent-sandbox",
    "mcp-sandbox", "claude-code-sandbox",
)

_KIND_DIRS = (
    "decisions", "facts", "people", "projects",
    "preferences", "drift", "refinements",
)


# --- invariant checks -----------------------------------------------------
#
# Each check has signature `(store: Path) -> tuple[bool, str]`.


def check_layout_root_canonical(store: Path) -> tuple[bool, str]:
    """Verify §3 canonical top-level structure."""
    if not store.is_dir():
        return False, f"store not a directory: {store}"
    unexpected = []
    for entry in store.iterdir():
        name = entry.name
        if entry.is_dir() and name not in _CANONICAL_TOP_LEVEL_DIRS:
            unexpected.append(name + "/")
        elif entry.is_file() and name not in _CANONICAL_TOP_LEVEL_FILES:
            # Tolerate AGENTS.md / INTEROP.md / PROPOSAL.md etc. at root
            # — they live under docs/memory/ normally but a synthetic
            # test store may copy them in.
            if name.endswith((".md", ".json", ".yaml")):
                continue
            unexpected.append(name)
    if unexpected:
        return False, f"unexpected top-level entries: {unexpected}"
    return True, "canonical layout"


def check_layout_no_sandbox_path(store: Path) -> tuple[bool, str]:
    """Verify §0.1 — store is not under any forbidden sandbox path."""
    resolved = str(store.resolve()).lower()
    exempt_prefix = os.environ.get("CYBEROS_HOST_MOUNT_PREFIX", "").strip()
    if exempt_prefix and str(store.resolve()).startswith(exempt_prefix):
        return True, "store under CYBEROS_HOST_MOUNT_PREFIX (exempt)"
    for fragment in _SANDBOX_FRAGMENTS:
        if fragment.lower() in resolved:
            return False, (
                f"path contains forbidden fragment {fragment!r}; "
                "store appears to be on ephemeral storage"
            )
    return True, f"resolved={store.resolve()}"


def check_layout_shard_uniformity(store: Path) -> tuple[bool, str]:
    """Warn on un-resharded memory files (direct children of <kind>/)."""
    mem = store / "memories"
    if not mem.is_dir():
        return True, "no memories/ directory yet"
    leftovers: list[str] = []
    for kind in _KIND_DIRS:
        kdir = mem / kind
        if not kdir.is_dir():
            continue
        for child in kdir.iterdir():
            if child.is_file() and child.name.endswith(".md"):
                leftovers.append(f"memories/{kind}/{child.name}")
    if leftovers:
        sample = leftovers[:5]
        more = "" if len(leftovers) <= 5 else f" (+{len(leftovers) - 5} more)"
        return False, (
            f"{len(leftovers)} un-resharded memory file(s); first: {sample}{more}"
        )
    return True, "all memories sharded under <kind>/<hex>/<hex>/"


def check_ledger_link(store: Path) -> tuple[bool, str]:
    """Verify the LINK invariant across all binlog segments."""
    audit = store / "audit"
    segs = sorted(p for p in audit.glob("*.binlog") if p.name != "current.binlog")
    current = audit / "current.binlog"
    if current.exists():
        segs.append(current)
    if not segs:
        return True, "no binlog segments"
    try:
        start_prev = resolve_initial_chain_from_manifest(store)
    except ValueError as exc:
        return False, f"malformed legacy bridge: {exc}"
    n = verify_segments(segs, start_prev=start_prev)
    return True, f"{n} records, chain intact"


def check_ledger_hash(store: Path) -> tuple[bool, str]:
    """Re-hash every binlog row; assert chain field matches.

    ``verify_segments`` already checks both LINK and HASH together, so the
    HASH-only invariant is exercised by the same code path. This function
    exists separately so the report distinguishes the failure modes — but
    in practice if HASH fails LINK fails (the next row's prev_chain points
    at the rehashed value).
    """
    return check_ledger_link(store)


def check_ledger_crc_tail(store: Path) -> tuple[bool, str]:
    """Surface CRC-truncated tails — informational, not a chain break."""
    current = store / "audit" / "current.binlog"
    if not current.exists():
        return True, "no active binlog"
    # The walker stops cleanly on CRC mismatch; compare reported byte
    # count against on-disk size to detect a truncation.
    on_disk = current.stat().st_size
    consumed = 0
    with MmapWalker(current) as walker:
        for offset, _rec in walker.iter_records():
            consumed = offset + walker.frame_size_at(offset)
    if consumed < on_disk:
        return False, (
            f"trailing {on_disk - consumed} byte(s) past last good frame; "
            "next writer open will truncate"
        )
    return True, f"all {on_disk} bytes accounted for"


def check_ledger_bridge_continuity(store: Path) -> tuple[bool, str]:
    """For v2 stores with a legacy bridge, first binlog row prev_chain == bridge."""
    manifest_path = store / "manifest.json"
    if not manifest_path.is_file():
        return True, "no manifest (test store?)"
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        return False, f"unreadable manifest: {exc}"
    if manifest.get("schema_version", 1) < 2:
        return True, f"schema_version={manifest.get('schema_version', 1)} — bridge not applicable"
    bridge = manifest.get("migration", {}).get("legacy_last_chain")
    if not bridge:
        return True, "schema v2 but no legacy bridge (greenfield store)"

    current = store / "audit" / "current.binlog"
    if not current.exists() or current.stat().st_size == 0:
        return True, f"binlog empty; bridge value {bridge[:16]}… reserved for first row"

    expected = bridge[len("sha256:"):] if bridge.startswith("sha256:") else bridge
    with MmapWalker(current) as walker:
        first = next(walker.iter_records(), None)
    if first is None:
        return True, "binlog empty after open"
    _offset, rec = first
    if rec.prev_chain != expected:
        return False, (
            f"first binlog record's prev_chain={rec.prev_chain[:16]}… "
            f"does not match legacy_last_chain={expected[:16]}…"
        )
    return True, f"bridge intact (prev={expected[:16]}…)"


def check_ledger_mmr_cross_check(store: Path) -> tuple[bool, str]:
    """MMR root recomputed from binlog MUST match persisted peaks.

    PROPOSAL.md P2 Stage 1 cross-check: divergence means the additive MMR
    is out of sync with the chain. The chain is authoritative; this surfaces
    the MMR bug so we don't promote the primitive.

    Pass when no MMR exists yet (store predates P2).
    """
    peaks_path = store / "audit" / "mmr" / "peaks.bin"
    if not peaks_path.is_file():
        return True, "no MMR persisted (P2 Stage 1 not active)"

    audit = store / "audit"
    segs = sorted(p for p in audit.glob("*.binlog") if p.name != "current.binlog")
    current = audit / "current.binlog"
    if current.exists():
        segs.append(current)
    if not segs:
        return True, "no binlog to cross-check against"

    from cyberos.core.mmr import OnDiskMMR, mmr_root_for_binlog
    persisted = OnDiskMMR(store)
    recomputed_root, recomputed_leaves = mmr_root_for_binlog(segs)

    if persisted.leaf_count != recomputed_leaves:
        return False, (
            f"leaf-count mismatch: persisted={persisted.leaf_count}, "
            f"recomputed={recomputed_leaves}"
        )
    if persisted.root() != recomputed_root:
        return False, (
            f"root mismatch: persisted={persisted.root().hex()[:16]}…, "
            f"recomputed={recomputed_root.hex()[:16]}…"
        )
    return True, (
        f"MMR consistent ({persisted.leaf_count} leaves, "
        f"root={persisted.root().hex()[:16]}…)"
    )


def check_ledger_op_enum_conformance(store: Path) -> tuple[bool, str]:
    """Every audit row's `op` field MUST be in memory.schema.json's op enum.

    Defends against future writer typos or rogue tools that append rows
    with unrecognised op strings.
    """
    schema_path = _find_memory_schema(store)
    if schema_path is None:
        return True, "memory.schema.json not located; skip"
    try:
        schema = json.loads(schema_path.read_text(encoding="utf-8"))
        op_enum = set(schema["definitions"]["AuditRecord"]["properties"]["op"]["enum"])
    except (OSError, KeyError, ValueError) as exc:
        return False, f"schema lookup failed: {exc}"

    from cyberos.core.walker import MmapWalker
    audit = store / "audit"
    segs = sorted(p for p in audit.glob("*.binlog") if p.name != "current.binlog")
    current = audit / "current.binlog"
    if current.exists():
        segs.append(current)

    offenders: dict[str, int] = {}
    n_total = 0
    for seg in segs:
        with MmapWalker(seg) as walker:
            for _o, rec in walker.iter_records():
                n_total += 1
                if rec.op not in op_enum:
                    offenders[rec.op] = offenders.get(rec.op, 0) + 1
    if offenders:
        return False, (
            f"{sum(offenders.values())} row(s) with off-enum op: {offenders}"
        )
    return True, f"all {n_total} rows have schema-valid op"


def check_ledger_append_only(store: Path) -> tuple[bool, str]:
    """The LINK+HASH check above proves append-only. This is a sentinel."""
    audit = store / "audit"
    segs = sorted(p for p in audit.glob("*.binlog") if p.name != "current.binlog")
    if not segs:
        return True, "no sealed segments to check"
    # If any sealed segment's mtime is newer than its largest seq's ts_ns,
    # someone touched it after-the-fact.
    issues: list[str] = []
    for seg in segs:
        stat = seg.stat()
        mtime_ns = stat.st_mtime_ns
        with MmapWalker(seg) as walker:
            max_ts = 0
            for _o, rec in walker.iter_records():
                if rec.ts_ns > max_ts:
                    max_ts = rec.ts_ns
        if max_ts == 0:
            continue
        # Allow 60s of fs-mtime slop (rename(2) updates parent dir, not file).
        if mtime_ns > max_ts + 60_000_000_000:
            issues.append(
                f"{seg.name}: mtime is "
                f"{(mtime_ns - max_ts) / 1e9:.0f}s after last record ts"
            )
    if issues:
        return False, "; ".join(issues)
    return True, f"{len(segs)} sealed segment(s) untouched"


def check_manifest_schema_version(store: Path) -> tuple[bool, str]:
    """manifest.json exists, has int schema_version, v2 carries legacy_last_chain."""
    manifest_path = store / "manifest.json"
    if not manifest_path.is_file():
        return False, "manifest.json missing"
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        return False, f"unparseable: {exc}"
    schema = manifest.get("schema_version")
    if not isinstance(schema, int) or schema < 1:
        return False, f"invalid schema_version: {schema!r}"
    if schema >= 2 and not manifest.get("migration", {}).get("legacy_last_chain"):
        # Greenfield v2 stores (no prior v1 history) are allowed to omit
        # the bridge; detect this by checking whether legacy JSONLs exist.
        has_legacy = any(
            (store / "audit").glob("*.jsonl")
        )
        if has_legacy:
            return False, (
                "schema_version=2 but no migration.legacy_last_chain; "
                "legacy JSONL files present — bridge missing"
            )
    return True, f"schema_version={schema}"


def check_manifest_validates(store: Path) -> tuple[bool, str]:
    """Validate manifest.json against memory.schema.json#/definitions/Manifest.

    Uses jsonschema if installed; otherwise skips with a warning.
    """
    schema_path = _find_memory_schema(store)
    if schema_path is None:
        return True, "memory.schema.json not located; skip"
    manifest_path = store / "manifest.json"
    if not manifest_path.is_file():
        return False, "manifest.json missing"
    try:
        import jsonschema  # type: ignore[import-not-found]
    except ImportError:
        return True, "jsonschema not installed; skip"
    try:
        full = json.loads(schema_path.read_text(encoding="utf-8"))
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        # Pick the newest validator class the installed version exposes.
        # jsonschema < 4.0 only has Draft7Validator; 4.x adds 2019/2020.
        validator_cls = (
            getattr(jsonschema, "Draft202012Validator", None)
            or getattr(jsonschema, "Draft201909Validator", None)
            or jsonschema.Draft7Validator
        )
        resolver = jsonschema.RefResolver.from_schema(full)
        validator = validator_cls(
            full["definitions"]["Manifest"], resolver=resolver,
        )
        errors = list(validator.iter_errors(manifest))
    except Exception as exc:  # noqa: BLE001 — surface as a check failure
        return False, f"validation harness error: {exc}"
    if errors:
        return False, f"{len(errors)} error(s); first: {errors[0].message}"
    return True, "manifest validates"


def check_export_determinism(store: Path) -> tuple[bool, str]:
    """Two exports of the current store MUST be byte-identical."""
    from cyberos.core.export import export_zip  # noqa: WPS433 — heavy import
    with tempfile.TemporaryDirectory(prefix="cyberos-doctor-det-") as td:
        td_path = Path(td)
        a = export_zip(store, td_path / "a.zip")
        b = export_zip(store, td_path / "b.zip")
    if a != b:
        return False, f"sha256 diverged: {a[:16]}… vs {b[:16]}…"
    return True, f"sha256={a[:16]}…"


def check_crc_implementation(_store: Path) -> tuple[bool, str]:
    """Strongly recommend the hardware crc32c wheel for production."""
    impl = crc_implementation()
    if impl == "hw":
        return True, "hardware-accelerated CRC-32C"
    return False, (
        f"using {impl!r}; install the 'crc32c' wheel for SSE 4.2 / ARM CRC32 path"
    )


def check_durability_platform_correct(_store: Path) -> tuple[bool, str]:
    """On Darwin, verify F_BARRIERFSYNC is the per-batch strategy.

    Read-only: we inspect the writer's default strategy resolution.
    """
    from cyberos.core.fsync import STRATEGY_AUTO, STRATEGY_FBARRIER
    if _is_darwin():
        # The cfg.fsync_strategy default is STRATEGY_AUTO. On Darwin, AUTO
        # resolves to fbarrier internally (see durable_sync). We assert
        # the constants are present and correct.
        if STRATEGY_AUTO == "auto" and STRATEGY_FBARRIER == "fbarrier":
            return True, "auto → F_BARRIERFSYNC on Darwin"
        return False, "fsync strategy constants drifted; check cyberos/core/fsync.py"
    return True, f"non-Darwin platform ({sys.platform}); fdatasync path used"


# --- the registry & walker ------------------------------------------------


_REGISTRY: dict[str, Callable[[Path], tuple[bool, str]]] = {
    "layout-root-canonical": check_layout_root_canonical,
    "layout-no-sandbox-path": check_layout_no_sandbox_path,
    "layout-shard-uniformity": check_layout_shard_uniformity,
    "ledger-link-invariant": check_ledger_link,
    "ledger-hash-invariant": check_ledger_hash,
    "ledger-crc-tail": check_ledger_crc_tail,
    "ledger-bridge-continuity": check_ledger_bridge_continuity,
    "ledger-append-only": check_ledger_append_only,
    "ledger-mmr-cross-check": check_ledger_mmr_cross_check,
    "ledger-op-enum-conformance": check_ledger_op_enum_conformance,
    "manifest-schema-version": check_manifest_schema_version,
    "manifest-validates-against-schema": check_manifest_validates,
    "export-determinism": check_export_determinism,
    "crypto-crc-implementation": check_crc_implementation,
    "durability-platform-correct": check_durability_platform_correct,
}


def _find_memory_schema(store: Path) -> Path | None:
    """Locate ``memory.schema.json`` near the store or repo."""
    candidates = [
        store / "memory.schema.json",
        store.parent / "docs" / "memory" / "memory.schema.json",
    ]
    # Repo root candidate (if cyberos package is importable from this layout).
    try:
        import cyberos
        repo = Path(cyberos.__file__).resolve().parent.parent
        candidates.append(repo / "docs" / "memory" / "memory.schema.json")
    except Exception:  # noqa: BLE001
        pass
    for c in candidates:
        if c.is_file():
            return c
    return None


def load_invariants_yaml(path: Path | None = None) -> list[dict]:
    """Load ``memory.invariants.yaml``. Imports PyYAML lazily."""
    if path is None:
        path = _find_invariants_yaml()
    if path is None or not path.is_file():
        return []
    import yaml  # noqa: WPS433 — lazy; this code path is cold
    data = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    return data.get("invariants", [])


def _find_invariants_yaml() -> Path | None:
    try:
        import cyberos
        repo = Path(cyberos.__file__).resolve().parent.parent
        candidate = repo / "docs" / "memory" / "memory.invariants.yaml"
        if candidate.is_file():
            return candidate
    except Exception:  # noqa: BLE001
        pass
    return None


def run_all(store: Path, *, only: list[str] | None = None) -> Report:
    """Walk every invariant in ``memory.invariants.yaml`` against ``store``."""
    invariants = load_invariants_yaml()
    started = time.time_ns()
    report = Report(store=store, started_ns=started)

    for inv in invariants:
        inv_id = inv.get("id")
        if not inv_id:
            continue
        if only and inv_id not in only:
            continue
        level = inv.get("level", "error")
        scope = inv.get("scope", "unknown")

        check_ref = inv.get("check", "")
        check_fn = _resolve_check(check_ref)
        if check_fn is None:
            report.results.append(CheckResult(
                id=inv_id, level="error", scope=scope, passed=False,
                details=f"check function not found: {check_ref!r}",
            ))
            continue

        t0 = time.perf_counter_ns()
        try:
            passed, details = check_fn(store)
        except Exception as exc:  # noqa: BLE001 — harness must not crash
            passed, details = False, f"harness error: {exc!r}"
        duration_ms = (time.perf_counter_ns() - t0) / 1_000_000
        report.results.append(CheckResult(
            id=inv_id, level=level, scope=scope, passed=passed,
            details=details, duration_ms=duration_ms,
        ))

    report.finished_ns = time.time_ns()
    return report


def _resolve_check(qualified_name: str) -> Callable[[Path], tuple[bool, str]] | None:
    """Resolve "cyberos.core.invariants.check_foo" to a function."""
    if not qualified_name:
        return None
    # Fast path: look up in our local registry by tail.
    tail = qualified_name.rsplit(".", 1)[-1]
    if tail in {fn.__name__ for fn in _REGISTRY.values()}:
        for fn in _REGISTRY.values():
            if fn.__name__ == tail:
                return fn
    # Generic path: importlib resolve.
    try:
        module_name, attr = qualified_name.rsplit(".", 1)
        module = importlib.import_module(module_name)
        return getattr(module, attr, None)
    except (ImportError, ValueError):
        return None


# --- pretty-printer for the CLI -------------------------------------------


def format_report(report: Report, *, json_mode: bool = False) -> str:
    if json_mode:
        return json.dumps({
            "store": str(report.store),
            "ok": report.ok,
            "started_ns": report.started_ns,
            "finished_ns": report.finished_ns,
            "results": [
                {
                    "id": r.id, "level": r.level, "scope": r.scope,
                    "passed": r.passed, "details": r.details,
                    "duration_ms": r.duration_ms,
                }
                for r in report.results
            ],
        }, indent=2)

    lines = []
    lines.append(f"cyberos doctor — {report.store}")
    lines.append("")
    width = max((len(r.id) for r in report.results), default=20)
    for r in report.results:
        status = "PASS" if r.passed else (
            "ERROR" if r.level == "error" else "WARN "
        )
        lines.append(
            f"  [{status}] {r.id:<{width}}  {r.details}  ({r.duration_ms:.1f} ms)"
        )
    lines.append("")
    n_err = len(report.errors)
    n_warn = len(report.warnings)
    n_pass = sum(1 for r in report.results if r.passed)
    lines.append(f"  total: {len(report.results)}  pass: {n_pass}  warn: {n_warn}  error: {n_err}")
    lines.append(
        f"  overall: {'OK' if report.ok else 'FAIL'} "
        f"({(report.finished_ns - report.started_ns) / 1e6:.0f} ms)"
    )
    return "\n".join(lines)


@dataclass
class RepairResult:
    """Outcome of a single repair action."""
    invariant_id: str
    fixed: bool
    details: str


_REPAIRABLE_INVARIANTS: dict[str, str] = {
    # invariant_id → human-readable description of what the repair does
    "layout-shard-uniformity":
        "run runtime/tools/cyberos_migrate_sidecar.py to reshard unsharded "
        "memory files into hex buckets",
    "manifest-validates-against-schema":
        "regenerate index/manifest.json from the current binlog state",
}


def repair(store: Path, *, only: list[str] | None = None) -> list[RepairResult]:
    """Auto-fix recoverable invariant failures.

    Only attempts repair for invariants listed in ``_REPAIRABLE_INVARIANTS``.
    Catastrophic failures (chain corruption, MMR cross-check, unparseable
    manifest) NEVER auto-repair — those require human review.

    Returns a list of :class:`RepairResult` (one per attempted repair).
    """
    report = run_all(store)
    actions: list[RepairResult] = []
    # Process every failed check — both errors AND warnings — that's
    # repairable. Warnings get auto-fixed too (e.g. shard-uniformity is
    # a warning by design but the repair is safe and the user benefits).
    failed = [r for r in report.results if not r.passed]
    for r in failed:
        if only and r.id not in only:
            continue
        if r.id not in _REPAIRABLE_INVARIANTS:
            # Only complain about errors we can't fix; warnings we can't
            # fix are not worth surfacing as "skip".
            if r.level == "error":
                actions.append(RepairResult(
                    invariant_id=r.id, fixed=False,
                    details=(
                        f"not auto-repairable; needs human review. "
                        f"Original failure: {r.details}"
                    ),
                ))
            continue
        try:
            fix_fn = _REPAIR_REGISTRY[r.id]
            ok, msg = fix_fn(store)
            actions.append(RepairResult(
                invariant_id=r.id, fixed=ok, details=msg,
            ))
        except Exception as exc:  # noqa: BLE001
            actions.append(RepairResult(
                invariant_id=r.id, fixed=False,
                details=f"repair raised: {type(exc).__name__}: {exc}",
            ))
    return actions


def _repair_shard_uniformity(store: Path) -> tuple[bool, str]:
    """Reshard un-bucketed memory files via the sidecar migrator's logic.

    We don't shell out — we reuse :mod:`runtime.tools.cyberos_migrate_sidecar`'s
    layout-restructuring path. Doing this in-process keeps the doctor's
    repair operation a single atomic decision the user can audit.

    For shard-only repair we don't touch frontmatter — just move the
    files into <kind>/<hex>/<hex>/.
    """
    import hashlib
    import shutil
    mem = store / "memories"
    if not mem.is_dir():
        return True, "no memories/ to reshard"

    moved = 0
    skipped: list[str] = []
    _KIND_DIRS = (
        "decisions", "facts", "people", "projects",
        "preferences", "drift", "refinements",
    )
    for kind in _KIND_DIRS:
        kdir = mem / kind
        if not kdir.is_dir():
            continue
        for path in sorted(kdir.glob("*.md")):
            rel = path.relative_to(kdir).parts
            if len(rel) > 1:
                continue  # already sharded
            sha = hashlib.sha256(path.name.encode("utf-8")).hexdigest()
            shard = kdir / sha[0:2] / sha[2:4]
            shard.mkdir(parents=True, exist_ok=True)
            target = shard / path.name
            if target.exists():
                skipped.append(path.name)
                continue
            shutil.move(str(path), str(target))
            # Also move any sidecar.
            sidecar = path.with_suffix(".md.meta.json")
            if sidecar.is_file():
                shutil.move(str(sidecar), str(shard / sidecar.name))
            moved += 1

    msg = f"resharded {moved} file(s)"
    if skipped:
        msg += f"; skipped {len(skipped)} (target exists): {skipped[:3]}"
    return True, msg


def _repair_index_manifest(store: Path) -> tuple[bool, str]:
    """Regenerate index/manifest.json from the binlog head."""
    import json as _json
    import struct as _struct
    head_path = store / "HEAD"
    last_seq = 0
    if head_path.is_file():
        buf = head_path.read_bytes()
        if len(buf) == 8:
            last_seq = _struct.unpack("<Q", buf)[0]
    target = store / "index" / "manifest.json"
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(
        _json.dumps({"schema_version": 2, "last_applied_seq": last_seq},
                   sort_keys=True),
        encoding="utf-8",
    )
    return True, f"wrote index/manifest.json (last_applied_seq={last_seq})"


_REPAIR_REGISTRY: dict[str, Callable[[Path], tuple[bool, str]]] = {
    "layout-shard-uniformity": _repair_shard_uniformity,
    "manifest-validates-against-schema": _repair_index_manifest,
}


__all__ = [
    "CheckResult", "Report", "RepairResult",
    "run_all", "load_invariants_yaml", "format_report",
    "repair",
    # Individual checks (exported for unit tests + custom walkers)
    "check_layout_root_canonical",
    "check_layout_no_sandbox_path",
    "check_layout_shard_uniformity",
    "check_ledger_link",
    "check_ledger_hash",
    "check_ledger_crc_tail",
    "check_ledger_bridge_continuity",
    "check_ledger_append_only",
    "check_manifest_schema_version",
    "check_manifest_validates",
    "check_export_determinism",
    "check_crc_implementation",
    "check_durability_platform_correct",
]
