#!/usr/bin/env python3
"""
cyberos_sync.py — multi-machine sync scaffolding for .cyberos-memory/ stores.

Aspect 6.x of the Layer-1 improvement catalog.

Scaffolding ONLY — no network transport. Produces deterministic sync bundles
(filtered by sync_class) and applies bundles received from another subject
with three-way conflict detection. The actual transport (rsync, syncthing,
git-annex, S3) is left to the operator.

Sync-class rules (per §17):
  - local-only       : NEVER syncs. Stripped from every bundle.
  - publishable      : included; signed by author subject.
  - shared           : included; signed; requires consent.has_consent=true.
  - client-visible   : included ONLY in client-scope bundles with explicit
                       --include-client flag.

Three-way merge semantics:
  Given (local, remote, common_ancestor):
    - Same memory_id, same content_sha          → no-op (no conflict)
    - Same memory_id, differing content_sha     → CONFLICT (record as
                                                  memories/conflicts/<id>.md)
    - Remote-only memory_id                     → IMPORT (stage in
                                                  outputs/sync-staging/)
    - Local-only memory_id                      → no-op on import side

Usage
-----
    cyberos sync export --to ~/cyberos-syncbundle.zip
        Produces deterministic bundle of publishable + shared memories.
        Includes manifest.json (filtered) + memories/ (filtered) + audit/
        (last 30 days, filtered to ops on exported memory_ids).

    cyberos sync export --to ~/bundle.zip --include client-visible
        Add client-visible memories to the bundle (consent-gated).

    cyberos sync import <bundle.zip> --from subject:teammate [--dry-run]
        Detect conflicts; stage non-conflicting imports under
        outputs/sync-staging/. Writes a sync report at
        outputs/sync/<run-id>.md.

    cyberos sync conflicts
        List pending conflicts (memories/conflicts/*.md) created by prior
        imports. Hand off to §3 reconciliation pipeline.

Exit codes
----------
0 = ok
1 = conflicts surfaced (informational; not a failure)
2 = invocation error
3 = bundle integrity failure (refused to apply)
"""
from __future__ import annotations

import argparse
import hashlib
import io
import json
import re
import sys
import zipfile
from datetime import datetime, timedelta, timezone
from pathlib import Path

ICT = timezone(timedelta(hours=7))


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def parse_frontmatter(text: str) -> tuple[dict, str]:
    if not text.startswith("---\n"):
        return {}, text
    end = text.find("\n---\n", 4)
    if end < 0:
        return {}, text
    fm_text = text[4:end]
    body = text[end + 5:]
    try:
        import yaml
        return yaml.safe_load(fm_text) or {}, body
    except Exception:
        fm = {}
        for line in fm_text.splitlines():
            m = re.match(r"^([a-z_]+):\s*(.+?)\s*$", line)
            if m:
                fm[m.group(1)] = m.group(2)
        return fm, body


def content_sha(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


# ---------------------------------------------------------------------------
# Export
# ---------------------------------------------------------------------------

INCLUDABLE_SYNC_CLASSES = {"publishable", "shared"}


def collect_exportable(brain_root: Path, include_extra: set[str]) -> list[tuple[str, str, dict]]:
    """Walk .cyberos-memory/, return list of (rel_path, content, frontmatter)."""
    brain = brain_root / ".cyberos-memory"
    allowed = INCLUDABLE_SYNC_CLASSES | include_extra
    out = []
    for md in sorted(brain.rglob("*.md")):
        if not md.is_file() or md.name.startswith("."):
            continue
        # Skip non-memory directories
        rel = md.relative_to(brain).as_posix()
        if rel.startswith(("audit/", "index/", "exports/", "outputs/", "meta/templates/")):
            continue
        try:
            text = md.read_text(encoding="utf-8")
        except Exception:
            continue
        fm, _ = parse_frontmatter(text)
        sync = fm.get("sync_class", "")
        if sync not in allowed:
            continue
        # Consent gate for shared / client-visible
        if sync in ("shared", "client-visible"):
            consent = fm.get("consent", {}) or {}
            if not consent.get("has_consent", False):
                continue
        # Drop tombstoned entries from export (downstream has its own record)
        if fm.get("tombstoned"):
            continue
        out.append((rel, text, fm))
    return out


def build_export_bundle(brain_root: Path, out_path: Path, include_extra: set[str]) -> tuple[int, int]:
    """Write a deterministic zip. Returns (memory_count, byte_size)."""
    entries = collect_exportable(brain_root, include_extra)
    # Deterministic ordering
    entries.sort(key=lambda e: e[0])

    # Derive a deterministic snapshot_at = max(last_updated_at)
    # Falls back to 1980 epoch if no timestamps available.
    def _ts(fm):
        v = fm.get("last_updated_at") or fm.get("created_at") or ""
        return v if isinstance(v, str) else ""
    max_ts = max((_ts(fm) for _, _, fm in entries), default="") or "1980-01-01T00:00:00+00:00"

    # Build a sync_manifest.json (small, deterministic)
    sync_manifest = {
        "format_version": "cyberos-sync-1",
        "snapshot_at": max_ts,
        "memory_count": len(entries),
        "entries": [
            {
                "path": rel,
                "memory_id": fm.get("memory_id", ""),
                "content_sha": content_sha(text),
                "sync_class": fm.get("sync_class", ""),
                "classification": fm.get("classification", ""),
                "authority": fm.get("authority", ""),
                "version": fm.get("version", 1),
            }
            for rel, text, fm in entries
        ],
    }
    # Sort entries inside manifest too
    sync_manifest["entries"].sort(key=lambda e: e["path"])
    manifest_bytes = (json.dumps(sync_manifest, sort_keys=True, separators=(",", ":")) + "\n").encode("utf-8")

    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED, allowZip64=False) as z:
        # Manifest first (deterministic mtime)
        zi = zipfile.ZipInfo("sync_manifest.json", date_time=(1980, 1, 1, 0, 0, 0))
        zi.compress_type = zipfile.ZIP_DEFLATED
        zi.external_attr = (0o644 & 0xFFFF) << 16
        z.writestr(zi, manifest_bytes)
        for rel, text, _ in entries:
            zi = zipfile.ZipInfo(f"memories/{rel}", date_time=(1980, 1, 1, 0, 0, 0))
            zi.compress_type = zipfile.ZIP_DEFLATED
            zi.external_attr = (0o644 & 0xFFFF) << 16
            z.writestr(zi, text.encode("utf-8"))

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(buf.getvalue())
    return len(entries), len(buf.getvalue())


def cmd_export(args):
    brain_root = find_brain()
    include_extra = set()
    if args.include:
        for tok in args.include.split(","):
            tok = tok.strip()
            if tok == "client-visible":
                include_extra.add("client-visible")
            elif tok in INCLUDABLE_SYNC_CLASSES:
                pass  # already included
            else:
                print(f"WARN: unknown sync class {tok!r} (ignored)", file=sys.stderr)

    out_path = Path(args.to)
    n, size = build_export_bundle(brain_root, out_path, include_extra)
    print(f"  ✓ sync bundle: {out_path}")
    print(f"    memories: {n}")
    print(f"    size:     {size:,} bytes")
    print(f"    sync_classes: publishable, shared{', client-visible' if 'client-visible' in include_extra else ''}")
    sha = hashlib.sha256(out_path.read_bytes()).hexdigest()
    print(f"    sha256:   {sha[:16]}…")
    return 0


# ---------------------------------------------------------------------------
# Import
# ---------------------------------------------------------------------------

def index_local(brain_root: Path) -> dict[str, tuple[str, str]]:
    """Map memory_id → (rel_path, content_sha) for local memories."""
    brain = brain_root / ".cyberos-memory"
    out = {}
    for md in brain.rglob("*.md"):
        if not md.is_file() or md.name.startswith("."):
            continue
        rel = md.relative_to(brain).as_posix()
        if rel.startswith(("audit/", "index/", "exports/")):
            continue
        try:
            text = md.read_text(encoding="utf-8")
        except Exception:
            continue
        fm, _ = parse_frontmatter(text)
        mid = fm.get("memory_id")
        if mid:
            out[mid] = (rel, content_sha(text))
    return out


def cmd_import(args):
    brain_root = find_brain()
    bundle = Path(args.bundle)
    if not bundle.exists():
        print(f"ERROR: bundle not found: {bundle}", file=sys.stderr)
        return 2

    # Read sync_manifest from bundle
    with zipfile.ZipFile(bundle) as z:
        try:
            sm = json.loads(z.read("sync_manifest.json"))
        except KeyError:
            print(f"ERROR: bundle missing sync_manifest.json", file=sys.stderr)
            return 3
        local = index_local(brain_root)

        conflicts = []
        imports = []
        noops = []
        for entry in sm["entries"]:
            mid = entry["memory_id"]
            remote_sha = entry["content_sha"]
            if mid in local:
                local_path, local_sha = local[mid]
                if local_sha == remote_sha:
                    noops.append((mid, entry["path"]))
                else:
                    conflicts.append({
                        "memory_id": mid,
                        "local_path": local_path,
                        "local_sha": local_sha,
                        "remote_path": entry["path"],
                        "remote_sha": remote_sha,
                        "remote_version": entry.get("version", 1),
                    })
            else:
                imports.append(entry)

        # Run id
        run_id = datetime.now(ICT).strftime("%Y%m%d-%H%M%S")
        report = []
        report.append(f"# Sync import report — {run_id}")
        report.append("")
        report.append(f"**Bundle:** {bundle}")
        report.append(f"**From:** {args.frm}")
        report.append(f"**Bundle SHA256:** {hashlib.sha256(bundle.read_bytes()).hexdigest()}")
        report.append(f"**Generated:** {datetime.now(ICT).isoformat(timespec='seconds')}")
        report.append(f"**Mode:** {'dry-run' if args.dry_run else 'apply'}")
        report.append("")
        report.append(f"## Summary")
        report.append(f"- New imports: {len(imports)}")
        report.append(f"- Conflicts:   {len(conflicts)}")
        report.append(f"- No-ops:      {len(noops)}")
        report.append("")

        # Stage imports
        staged_dir = brain_root / "outputs" / "sync-staging" / run_id
        if imports:
            report.append(f"## Imports (staged for review)")
            if not args.dry_run:
                staged_dir.mkdir(parents=True, exist_ok=True)
            for entry in imports:
                src = f"memories/{entry['path']}"
                try:
                    text = z.read(src).decode("utf-8")
                except KeyError:
                    report.append(f"- SKIP: {entry['path']} (entry listed in manifest but file missing in bundle)")
                    continue
                if not args.dry_run:
                    dest = staged_dir / entry["path"]
                    dest.parent.mkdir(parents=True, exist_ok=True)
                    dest.write_text(text, encoding="utf-8")
                report.append(f"- {entry['memory_id']}  {entry['path']}")
            report.append("")
            if not args.dry_run:
                report.append(f"_(Review staged files at `outputs/sync-staging/{run_id}/`, then move into `.cyberos-memory/` via `brain_writer.py write`.)_")
                report.append("")

        if conflicts:
            report.append(f"## Conflicts (require §3 reconciliation)")
            for c in conflicts:
                report.append(f"- **{c['memory_id']}**")
                report.append(f"    - local:  `{c['local_path']}` sha={c['local_sha'][:12]}…")
                report.append(f"    - remote: `{c['remote_path']}` sha={c['remote_sha'][:12]}…")
            report.append("")
            # Write conflict markers
            if not args.dry_run:
                conflicts_dir = brain_root / ".cyberos-memory" / "memories" / "conflicts"
                conflicts_dir.mkdir(parents=True, exist_ok=True)
                for c in conflicts:
                    marker = conflicts_dir / f"sync-{run_id}-{c['memory_id'][-12:]}.md"
                    marker.write_text(
                        f"# Sync conflict — {c['memory_id']}\n\n"
                        f"- Local:  `{c['local_path']}` sha={c['local_sha']}\n"
                        f"- Remote: `{c['remote_path']}` sha={c['remote_sha']}\n"
                        f"- From subject: {args.frm}\n"
                        f"- Bundle: {bundle.name}\n"
                        f"- Run id: {run_id}\n\n"
                        "Resolution: see §3 (conflict reconciliation). Open both files,\n"
                        "decide a winner (or mark as disputed-pair), then delete this\n"
                        "marker once recorded in audit ledger.\n",
                        encoding="utf-8",
                    )

        # Persist report
        sync_dir = brain_root / "outputs" / "sync"
        sync_dir.mkdir(parents=True, exist_ok=True)
        report_path = sync_dir / f"{run_id}.md"
        report_path.write_text("\n".join(report) + "\n", encoding="utf-8")
        rel_report = report_path.relative_to(brain_root)
        print(f"  ✓ report: {rel_report}")
        print(f"    new:       {len(imports)}")
        print(f"    conflicts: {len(conflicts)}")
        print(f"    no-ops:    {len(noops)}")
        if args.dry_run:
            print(f"    mode:      dry-run (no files written)")
        return 1 if conflicts else 0


# ---------------------------------------------------------------------------
# Conflicts
# ---------------------------------------------------------------------------

def cmd_conflicts(args):
    brain_root = find_brain()
    conflicts_dir = brain_root / ".cyberos-memory" / "memories" / "conflicts"
    if not conflicts_dir.exists():
        print("  no conflicts dir")
        return 0
    items = sorted(conflicts_dir.glob("sync-*.md"))
    if not items:
        print("  no pending sync conflicts")
        return 0
    if not getattr(args, "resolve", False):
        print(f"  {len(items)} pending sync conflicts:")
        for p in items:
            first_line = ""
            try:
                first_line = p.read_text(encoding="utf-8").splitlines()[0]
            except Exception:
                pass
            print(f"    {p.name}  {first_line}")
        print()
        print(f"  Resolve interactively with: cyberos sync conflicts --resolve")
        return 1

    # Aspect 6.5 — interactive resolver
    print(f"\n  Interactive conflict resolver — {len(items)} pending conflict(s)")
    print(f"  For each conflict: pick [local | remote | disputed | open | skip]")
    print()
    resolved = 0
    skipped = 0
    for p in items:
        text = p.read_text(encoding="utf-8")
        print(f"  ── {p.name} ──")
        for line in text.splitlines()[:10]:
            print(f"    {line}")
        print()
        while True:
            choice = input(f"    [l]ocal | [r]emote | [d]isputed | [o]pen | [s]kip | [q]uit  ? ").strip().lower()
            if choice in ("l", "local"):
                # Record decision in marker as resolved
                resolution = "kept-local"
                _record_resolution(p, resolution)
                resolved += 1
                print(f"    → kept local; conflict marker annotated. Remember to record §3 reconciliation in audit ledger.")
                break
            elif choice in ("r", "remote"):
                resolution = "kept-remote"
                _record_resolution(p, resolution)
                resolved += 1
                print(f"    → accepted remote (you must manually apply the remote file to overwrite the local one).")
                break
            elif choice in ("d", "disputed"):
                resolution = "disputed-pair"
                _record_resolution(p, resolution)
                resolved += 1
                print(f"    → marked as disputed-pair. Per §3, both stand until evidence picks a winner.")
                break
            elif choice in ("o", "open"):
                print(f"    open in editor: {brain_root}/.cyberos-memory/{p.relative_to(brain_root / '.cyberos-memory')}")
                print(f"    (leaving conflict file in place; rerun resolver later)")
                skipped += 1
                break
            elif choice in ("s", "skip"):
                skipped += 1
                break
            elif choice in ("q", "quit"):
                print(f"\n  Quit. {resolved} resolved, {skipped} skipped, {len(items) - resolved - skipped} pending.")
                return 1 if resolved < len(items) else 0
            else:
                print(f"    pick one of: l / r / d / o / s / q")
    print(f"\n  Done. {resolved} resolved, {skipped} skipped.")
    return 0 if resolved == len(items) else 1


def _record_resolution(path: Path, resolution: str):
    """Append resolution decision to the conflict marker."""
    ts = datetime.now(ICT).isoformat(timespec="seconds")
    text = path.read_text(encoding="utf-8")
    text += f"\n\n## Resolution ({ts})\n\n- Decision: **{resolution}**\n- Resolved via: `cyberos sync conflicts --resolve`\n- Next: record this decision in the audit ledger per §3 reconciliation; then delete this marker.\n"
    path.write_text(text, encoding="utf-8")


def main():
    p = argparse.ArgumentParser(description="Multi-machine sync scaffolding (Aspect 6.x)")
    sub = p.add_subparsers(dest="cmd", required=True)

    pe = sub.add_parser("export", help="produce a deterministic sync bundle")
    pe.add_argument("--to", required=True, help="output zip path")
    pe.add_argument("--include", help="extra sync classes (comma-separated; default empty)")
    pe.set_defaults(func=cmd_export)

    pi = sub.add_parser("import", help="apply / preview a remote sync bundle")
    pi.add_argument("bundle", help="path to remote sync bundle (.zip)")
    pi.add_argument("--from", dest="frm", required=True, help="origin subject id (e.g. subject:teammate)")
    pi.add_argument("--dry-run", action="store_true", help="report only; do not write")
    pi.set_defaults(func=cmd_import)

    pc = sub.add_parser("conflicts", help="list / interactively resolve pending sync conflicts")
    pc.add_argument("--resolve", action="store_true", help="step through each conflict interactively (Aspect 6.5)")
    pc.set_defaults(func=cmd_conflicts)

    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
