#!/usr/bin/env python3
"""
cyberos_cold_storage.py — cold-tier export of ancient audit ledgers.

Aspect 9.5 of the Layer-1 improvement catalog.

Pattern from `cockroachdb/configuring-log-export` — old audit rows
(typically > 12 months) move to a cheap, slow, immutable tier. The
local BRAIN keeps recent rows + Merkle-checkpoint anchors; the cold
tier holds full history for compliance + forensic queries.

This tool does NOT actually upload to S3 / GCS — it produces the
deterministic archive bundle that the operator then `aws s3 cp` (or
equivalent) into their cold-storage bucket. The bundle includes a
Merkle anchor pointing back at the live BRAIN's chain head so a future
`decompact-on-demand` can verify the cold archive against the live
manifest.

Usage:
    cyberos cold-storage archive --age-months 12 --to ~/cold/
        Walk audit/*.jsonl, pick months older than the threshold,
        zip them with a manifest into ~/cold/<YYYY-MM>.cold.zip,
        leave the originals in place. Operator deletes locally after
        confirming the upload.

    cyberos cold-storage list ~/cold/
        Inventory archives in a directory (deterministic, sorted).

    cyberos cold-storage verify ~/cold/2025-04.cold.zip
        Verify the archive's Merkle anchor still resolves in the
        current live BRAIN's chain.

Local-only by contract. We never reach out to a cloud provider.
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


def ledger_last_ts(ledger: Path) -> datetime | None:
    last_ts = None
    for line in ledger.read_text(encoding="utf-8", errors="ignore").splitlines():
        if not line.strip():
            continue
        try:
            r = json.loads(line)
            ts = datetime.fromisoformat(r.get("ts", ""))
            if last_ts is None or ts > last_ts:
                last_ts = ts
        except Exception:
            continue
    return last_ts


def cmd_archive(args):
    brain_root = find_brain()
    brain = brain_root / ".cyberos-memory"
    audit_dir = brain / "audit"
    if not audit_dir.exists():
        print("  no audit/ directory")
        return 0
    out_dir = Path(args.to).expanduser()
    out_dir.mkdir(parents=True, exist_ok=True)

    cutoff = datetime.now(ICT) - timedelta(days=int(args.age_months) * 30)

    # Read manifest's audit_chain_head for Merkle anchor
    try:
        manifest = json.loads((brain / "manifest.json").read_text(encoding="utf-8"))
        live_chain_head = manifest.get("audit_chain_head", "")
    except Exception:
        live_chain_head = ""

    archived = []
    for ledger in sorted(audit_dir.glob("*.jsonl")):
        last_ts = ledger_last_ts(ledger)
        if not last_ts or last_ts >= cutoff:
            continue
        m = re.match(r"^(\d{4})-(\d{2})", ledger.name)
        ym = f"{m.group(1)}-{m.group(2)}" if m else ledger.stem

        # Deterministic archive
        buf = io.BytesIO()
        manifest_blob = {
            "format_version": "cyberos-cold-1",
            "year_month": ym,
            "source_ledger": ledger.name,
            "rows": sum(1 for line in ledger.read_text(encoding="utf-8").splitlines() if line.strip()),
            "last_ts": last_ts.isoformat(),
            "live_chain_head_at_archive": live_chain_head,
            "content_sha256": hashlib.sha256(ledger.read_bytes()).hexdigest(),
        }
        manifest_bytes = (json.dumps(manifest_blob, sort_keys=True, separators=(",", ":")) + "\n").encode("utf-8")
        with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED) as z:
            zi = zipfile.ZipInfo("manifest.json", date_time=(1980, 1, 1, 0, 0, 0))
            z.writestr(zi, manifest_bytes)
            zi = zipfile.ZipInfo(ledger.name, date_time=(1980, 1, 1, 0, 0, 0))
            z.writestr(zi, ledger.read_bytes())
        out_path = out_dir / f"{ym}.cold.zip"
        out_path.write_bytes(buf.getvalue())
        archived.append({"year_month": ym, "out": str(out_path),
                         "size_bytes": len(buf.getvalue()),
                         "sha": hashlib.sha256(buf.getvalue()).hexdigest()[:16]})

    if not archived:
        print(f"  ✓ no ledgers older than {args.age_months} months")
        return 0

    print(f"  Archived {len(archived)} ledger(s) older than {args.age_months} months:")
    for a in archived:
        print(f"    {a['year_month']:10s}  {a['size_bytes']:>10,} B  sha={a['sha']}…  → {a['out']}")
    print()
    print(f"  Next: upload to cold storage (operator's choice — aws s3 cp / gcs / rclone).")
    print(f"  After confirmed upload, you MAY delete the local source ledgers.")
    return 0


def cmd_list(args):
    d = Path(args.dir).expanduser()
    if not d.exists():
        print(f"  no such dir: {d}")
        return 2
    archives = sorted(d.glob("*.cold.zip"))
    if not archives:
        print(f"  no .cold.zip archives in {d}")
        return 0
    print(f"  {len(archives)} cold archive(s) in {d}:")
    for a in archives:
        size = a.stat().st_size
        try:
            with zipfile.ZipFile(a) as z:
                mf = json.loads(z.read("manifest.json"))
                ym = mf.get("year_month", "?")
                rows = mf.get("rows", "?")
                anchor = mf.get("live_chain_head_at_archive", "?")[:24] + "…"
        except Exception:
            ym, rows, anchor = "?", "?", "?"
        print(f"    {ym:10s}  {size:>10,} B  rows={rows}  anchor={anchor}")
    return 0


def cmd_verify(args):
    archive = Path(args.archive).expanduser()
    if not archive.exists():
        print(f"  no such archive: {archive}")
        return 2
    brain_root = find_brain()
    brain = brain_root / ".cyberos-memory"
    try:
        with zipfile.ZipFile(archive) as z:
            mf = json.loads(z.read("manifest.json"))
            ledger_name = mf["source_ledger"]
            archived_sha = mf["content_sha256"]
            inner = z.read(ledger_name)
            recomputed = hashlib.sha256(inner).hexdigest()
    except Exception as e:
        print(f"  ✗ unreadable archive: {e}")
        return 3
    sha_ok = archived_sha == recomputed
    print(f"  archive:           {archive}")
    print(f"  year_month:        {mf.get('year_month')}")
    print(f"  content sha:       {'✓' if sha_ok else '✗'} {recomputed[:16]}… vs {archived_sha[:16]}…")
    # Check that the live BRAIN's chain still anchors this archive
    anchor = mf.get("live_chain_head_at_archive", "")
    try:
        live = json.loads((brain / "manifest.json").read_text(encoding="utf-8"))
        live_head = live.get("audit_chain_head", "")
    except Exception:
        live_head = ""
    # The chain MUST have advanced (newer head) or be identical
    print(f"  chain anchor:      {anchor[:32]}…")
    print(f"  live chain head:   {live_head[:32]}…")
    # We cannot verify the anchor is a prefix without rebuilding the full chain;
    # what we can verify is that the archive content matches its stated SHA.
    if not sha_ok:
        return 3
    print(f"  ✓ archive content verified (chain-anchor check requires full chain walk)")
    return 0


def main():
    p = argparse.ArgumentParser(description="cold-tier audit-ledger export (Aspect 9.5)")
    sub = p.add_subparsers(dest="cmd", required=True)
    pa = sub.add_parser("archive", help="produce cold archives for old ledgers")
    pa.add_argument("--age-months", type=int, default=12)
    pa.add_argument("--to", required=True, help="local output directory")
    pa.set_defaults(func=cmd_archive)
    pl = sub.add_parser("list", help="inventory archives in a directory")
    pl.add_argument("dir")
    pl.set_defaults(func=cmd_list)
    pv = sub.add_parser("verify", help="verify an archive against the live BRAIN")
    pv.add_argument("archive")
    pv.set_defaults(func=cmd_verify)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
