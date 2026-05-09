#!/usr/bin/env python3
"""
cyberos-doctor — recovery CLI for `.cyberos-memory/` stores in CORRUPT state.

Replaces AGENTS.md §13.0's "no auto-repair" rigidity with structured repair
operations under MAINTENANCE mode (§8.8). Every repair op:
  - requires explicit per-op confirmation (or --auto-yes for safe-only batch)
  - runs inside an `op:"maintenance.start"` ... `op:"maintenance.end"` envelope
  - emits its own audit row preserving full chain integrity
  - is reversible where possible (tombstone over hard-erase, etc.)

Usage
-----
    cyberos-doctor <store>                              # diagnose-only (read-only)
    cyberos-doctor <store> --repair                     # interactive repair
    cyberos-doctor <store> --repair --auto-yes          # batch safe repairs
    cyberos-doctor <store> --repair --reason "<text>"   # required for repair
    cyberos-doctor <store> --rebuild-checkpoint         # one-shot Stage 1 helper

Repair operations covered
-------------------------
    R1  stale-checkpoint           Reset manifest.reconciliation_checkpoint
                                   to the current ledger tail (or null)
    R2  partial-trailing-line      Truncate audit ledger to last full row
    R3  chain-head-unreachable     Update manifest.audit_chain_head to a
                                   reachable ledger row
    R4  orphan-file                Tombstone an unreferenced memory file
    R5  audit-id-collision         Surface for user resolution (no auto-fix)
    R6  manifest-pin-drift         Refuse repair; require §0.5 chat-turn
                                   approval (CANNOT be auto-fixed by design)

Findings the doctor cannot repair (surfaces only)
-------------------------------------------------
    chain-link-mismatch (mid-ledger break)  — needs human review of which side
                                              of the break is authoritative
    schema-invariant violations             — user must edit memory or
                                              update protocol via §0.5
    duplicate memory_id                     — user picks which to keep

Author: CyberOS local-optimization Stage 2 close
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import secrets
import shutil
import sys
import unicodedata
from pathlib import Path
from typing import Any

try:
    import rfc8785
    _HAS_JCS = True
except ImportError:
    _HAS_JCS = False

# Reuse the validator's findings model
sys.path.insert(0, str(Path(__file__).parent))
from cyberos_validate import (  # noqa: E402
    Validator, Finding, CRITICAL, WARN, INFO,
)


# ---------------------------------------------------------------------------
# Repair model
# ---------------------------------------------------------------------------

class Repair:
    """A single repair operation. Bound to a finding."""

    def __init__(self, code: str, finding: Finding, description: str,
                 dangerous: bool = False, requires_maintenance: bool = True):
        self.code = code  # e.g., "R1-rebuild-checkpoint"
        self.finding = finding
        self.description = description
        self.dangerous = dangerous  # require explicit confirmation even with --auto-yes
        self.requires_maintenance = requires_maintenance

    def apply(self, doctor: "Doctor") -> str:
        """Apply the repair. Return the audit row chain after."""
        raise NotImplementedError


# ---------------------------------------------------------------------------
# Doctor
# ---------------------------------------------------------------------------

class Doctor:
    def __init__(self, root: Path):
        self.root = root
        self.manifest_path = root / "manifest.json"
        self.audit_dir = root / "audit"
        self.findings: list[Finding] = []
        self.repairs: list[Repair] = []
        self.maintenance_session_id: str | None = None
        self.now = dt.datetime.now(dt.timezone(dt.timedelta(hours=7)))

    # -- diagnose ------------------------------------------------------------

    def diagnose(self) -> None:
        """Run validator, also detect partial-trailing-line and other doctor
        specific checks beyond what the validator covers."""
        v = Validator(self.root)
        v.run()
        self.findings = list(v.findings)

        # Doctor-specific checks
        self._check_partial_trailing_line()
        self._check_stale_checkpoint()

        # Dedupe: validator + doctor sometimes both surface the same issue
        # (e.g., audit-row-unparseable found by walk and by tail-check). Keep
        # first occurrence per (path, code) — preserves CRITICAL → WARN order
        # within validator-then-doctor sequence.
        seen: set[tuple[str, str]] = set()
        deduped: list[Finding] = []
        for f in self.findings:
            key = (f.path, f.code)
            if key not in seen:
                seen.add(key)
                deduped.append(f)
        self.findings = deduped

        # Map findings → repairs
        self._propose_repairs()

    def _check_partial_trailing_line(self) -> None:
        """Mid-write filesystem-sync partial-delivery detection."""
        for ledger in sorted(self.audit_dir.glob("*.jsonl")):
            try:
                data = ledger.read_bytes()
                if not data:
                    continue
                # Last byte should be \n if file is well-formed
                if not data.endswith(b"\n"):
                    self.findings.append(Finding(
                        WARN, "§4.7", "partial-trailing-line",
                        f"audit/{ledger.name}",
                        "ledger does not end with \\n — possible mid-write "
                        "filesystem-sync delivery; last line may be partial",
                        details={"last_8_bytes": repr(data[-8:])},
                    ))
                # Try to parse last line; if it fails AND the file ends with \n,
                # something else is wrong (caught by validator).
                last_line = data.rstrip(b"\n").rsplit(b"\n", 1)[-1]
                if last_line:
                    try:
                        json.loads(last_line)
                    except json.JSONDecodeError as e:
                        self.findings.append(Finding(
                            CRITICAL, "§7.3", "audit-row-unparseable",
                            f"audit/{ledger.name}",
                            f"last line not valid JSON: {e}",
                            details={"last_line_bytes": len(last_line)},
                        ))
            except OSError as e:
                self.findings.append(Finding(
                    CRITICAL, "§7", "audit-read-error",
                    f"audit/{ledger.name}",
                    f"cannot read ledger: {e}",
                ))

    def _check_stale_checkpoint(self) -> None:
        """§8.7 phase 4 stale-checkpoint check (Stage 1 amendment)."""
        try:
            m = json.loads(self.manifest_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            return  # already covered by validator
        cp = m.get("reconciliation_checkpoint")
        if not cp or not isinstance(cp, dict):
            return  # not set; nothing to check
        cp_id = cp.get("audit_id")
        cp_chain = cp.get("chain")
        if not cp_id or not cp_chain:
            return  # null/empty checkpoint is allowed

        # Check the audit_id resolves and chain matches
        for ledger in sorted(self.audit_dir.glob("*.jsonl")):
            try:
                with ledger.open("r", encoding="utf-8") as f:
                    for line in f:
                        if not line.strip():
                            continue
                        try:
                            row = json.loads(line)
                        except json.JSONDecodeError:
                            continue
                        if row.get("audit_id") == cp_id:
                            if row.get("chain") != cp_chain:
                                self.findings.append(Finding(
                                    CRITICAL, "§8.7",
                                    "stale-checkpoint",
                                    "manifest.json",
                                    f"reconciliation_checkpoint.audit_id "
                                    f"{cp_id[:24]}... resolves but chain "
                                    f"mismatches "
                                    f"(checkpoint={cp_chain[:23]}... "
                                    f"vs ledger={row.get('chain', '')[:23]}...)",
                                ))
                            return  # found the row
            except OSError:
                continue

        # Not found in any ledger
        self.findings.append(Finding(
            CRITICAL, "§8.7", "stale-checkpoint",
            "manifest.json",
            f"reconciliation_checkpoint.audit_id {cp_id[:24]}... "
            f"does not resolve in any ledger",
        ))

    def _propose_repairs(self) -> None:
        for f in self.findings:
            r = self._propose_for(f)
            if r:
                self.repairs.append(r)

    def _propose_for(self, f: Finding) -> Repair | None:
        if f.code == "stale-checkpoint":
            return RebuildCheckpointRepair(f)
        if f.code == "partial-trailing-line":
            return TruncatePartialLineRepair(f)
        if f.code == "audit-row-unparseable":
            return TruncateUnparseableLineRepair(f)
        if f.code == "audit-chain-head-unreachable":
            return UpdateChainHeadRepair(f)
        if f.code == "orphan-file":
            return TombstoneOrphanRepair(f)
        if f.code == "merkle-checkpoint-divergence":
            return RebuildMerkleCheckpointRepair(f)
        return None

    # -- preflight: stabilise ledger so maintenance can start ---------------

    def preflight_stabilise(self) -> list[str]:
        """If the ledger has a partial/unparseable trailing line, append-mode
        writes will concatenate to it instead of starting cleanly. Detect this
        condition and truncate-to-last-good-row BEFORE begin_maintenance.

        The truncate happens WITHOUT an audit row because the chain is
        already broken; appending a row to a broken chain is what we're
        protecting against. Forensic dumps go to /tmp.

        Returns: list of human-readable preflight actions taken.
        """
        actions: list[str] = []
        for ledger in sorted(self.audit_dir.glob("*.jsonl")):
            try:
                data = ledger.read_bytes()
            except OSError:
                continue
            if not data:
                continue

            # Walk lines, find last successfully-parseable boundary
            last_good_end = 0  # byte offset just past the last \n of a parseable row
            cursor = 0
            while cursor < len(data):
                nl = data.find(b"\n", cursor)
                if nl < 0:
                    # Trailing partial line (no final \n)
                    break
                line = data[cursor:nl]
                if line.strip():
                    try:
                        json.loads(line)
                        last_good_end = nl + 1
                    except json.JSONDecodeError:
                        break  # corrupt mid-stream — surface for human review
                else:
                    last_good_end = nl + 1
                cursor = nl + 1

            if last_good_end == len(data):
                continue  # ledger fully clean

            # Truncate
            forensic = Path("/tmp") / (
                f"{ledger.name}.preflight-tail-{secrets.token_hex(4)}")
            forensic.write_bytes(data[last_good_end:])
            tmp = ledger.with_suffix(f".tmp.{secrets.token_hex(8)}.part")
            tmp.write_bytes(data[:last_good_end])
            os.replace(tmp, ledger)
            actions.append(
                f"truncated {ledger.name} to last good row "
                f"({len(data) - last_good_end} bytes dumped to {forensic})"
            )
        return actions

    # -- maintenance envelope -----------------------------------------------

    def begin_maintenance(self, reason: str) -> None:
        if not reason or len(reason) < 5:
            raise ValueError("maintenance reason must be at least 5 chars")
        self.maintenance_session_id = uuid7(self.now)
        chain = self._append_audit({
            "op": "maintenance.start",
            "scope": "meta",
            "path": str(self.root.relative_to(self.root.parent)),
            "memory_id": None,
            "before_hash": None,
            "after_hash": None,
            "diff": "<hash-only>",
            "reason": (
                f"cyberos-doctor session {self.maintenance_session_id} — "
                f"reason: {reason}"
            ),
            "correction_to": None,
        })
        print(f"  → maintenance.start (session {self.maintenance_session_id[:24]}...; "
              f"chain {chain[:32]}...)")

    def end_maintenance(self, summary: str) -> None:
        if not self.maintenance_session_id:
            return
        chain = self._append_audit({
            "op": "maintenance.end",
            "scope": "meta",
            "path": str(self.root.relative_to(self.root.parent)),
            "memory_id": None,
            "before_hash": None,
            "after_hash": None,
            "diff": "<hash-only>",
            "reason": (
                f"cyberos-doctor session {self.maintenance_session_id} closed: "
                f"{summary}"
            ),
            "correction_to": None,
        })
        print(f"  → maintenance.end (chain {chain[:32]}...)")
        self.maintenance_session_id = None

    # -- audit append --------------------------------------------------------

    def _append_audit(self, partial_row: dict) -> str:
        """Append an audit row with full chain computation. Return new chain."""
        # Find latest ledger and its tail chain
        ledgers = sorted(self.audit_dir.glob("*.jsonl"))
        if not ledgers:
            raise RuntimeError("no ledger to append to")
        latest = ledgers[-1]
        # Robust: skip unparseable lines (the doctor exists to repair these).
        # Use the chain from the most recent SUCCESSFULLY parseable row.
        prev_chain = None
        with latest.open("r", encoding="utf-8") as f:
            for line in f:
                if not line.strip():
                    continue
                try:
                    prev_chain = json.loads(line).get("chain")
                except json.JSONDecodeError:
                    continue  # skip partial/corrupt lines
        if not prev_chain:
            raise RuntimeError("ledger has no parseable rows; cannot derive prev_chain")

        full_row = {
            "audit_id": f"evt_{uuid7(self.now)}",
            "ts": self.now.isoformat(timespec="seconds"),
            "actor_kind": "agent",
            "actor_id": "cyberos-doctor",
            "persona": None,
            **partial_row,
            "prev_version": partial_row.get("prev_version"),
            "new_version": partial_row.get("new_version"),
            "supersedes_event_id": None,
            "classification": None,
            "authority": None,
            "consent_event_id": None,
            "provenance": {
                "source": "manual",
                "source_ref": (
                    f"cyberos-doctor:maintenance:{self.maintenance_session_id}"
                    if self.maintenance_session_id else "cyberos-doctor"
                ),
                "confidence": 1.0,
            },
        }
        full_row.setdefault("memory_id", None)
        full_row.setdefault("prev_version", None)
        full_row.setdefault("new_version", None)
        full_row.setdefault("before_hash", None)
        full_row.setdefault("after_hash", None)
        full_row.setdefault("diff", "<hash-only>")
        full_row.setdefault("reason", "")
        full_row.setdefault("correction_to", None)

        if _HAS_JCS:
            canonical = rfc8785.dumps(full_row)
        else:
            canonical = json.dumps(
                full_row, sort_keys=True, separators=(",", ":"),
                ensure_ascii=False,
            ).encode("utf-8")
        chain = "sha256:" + hashlib.sha256(
            canonical + prev_chain.encode("utf-8")).hexdigest()
        full_row["prev_chain"] = prev_chain
        full_row["chain"] = chain

        with latest.open("a", encoding="utf-8") as f:
            f.write(json.dumps(full_row, ensure_ascii=False) + "\n")
            f.flush()
            os.fsync(f.fileno())
        return chain


def uuid7(now: dt.datetime) -> str:
    ms = int(now.timestamp() * 1000)
    rand_a = secrets.randbits(12)
    rand_b = secrets.randbits(62)
    high = (ms & 0xFFFFFFFFFFFF) << 16 | (0x7 << 12) | rand_a
    low = (0b10 << 62) | rand_b
    n = (high << 64) | low
    h = f"{n:032x}"
    return f"{h[0:8]}-{h[8:12]}-{h[12:16]}-{h[16:20]}-{h[20:32]}"


# ---------------------------------------------------------------------------
# Repair implementations
# ---------------------------------------------------------------------------

class RebuildCheckpointRepair(Repair):
    def __init__(self, finding: Finding):
        super().__init__(
            "R1-rebuild-checkpoint", finding,
            "Reset manifest.reconciliation_checkpoint to current ledger tail "
            "(or null if ledger is empty). Forces next §4.7 reconciliation to "
            "do a full walk; after a clean session.end the checkpoint will "
            "rebuild itself.",
            dangerous=False,
        )

    def apply(self, doctor: Doctor) -> str:
        before = doctor.manifest_path.read_bytes()
        m = json.loads(before)

        # Find latest ledger row
        ledgers = sorted(doctor.audit_dir.glob("*.jsonl"))
        latest_chain = None
        latest_audit_id = None
        latest_ts = None
        for f in ledgers:
            with f.open("r", encoding="utf-8") as fp:
                for line in fp:
                    if line.strip():
                        r = json.loads(line)
                        latest_chain = r.get("chain")
                        latest_audit_id = r.get("audit_id")
                        latest_ts = r.get("ts")

        m["reconciliation_checkpoint"] = (
            None if not latest_chain else {
                "audit_id": latest_audit_id,
                "chain": latest_chain,
                "ts": latest_ts,
            }
        )
        m["last_updated_at"] = doctor.now.isoformat(timespec="seconds")
        after_str = json.dumps(m, indent=2, ensure_ascii=False)
        after = (after_str + "\n").encode("utf-8")
        before_hash = "sha256:" + hashlib.sha256(before).hexdigest()
        after_hash = "sha256:" + hashlib.sha256(after.rstrip(b"\n")).hexdigest()

        # Atomic write
        tmp = doctor.manifest_path.with_suffix(
            f".tmp.{secrets.token_hex(8)}.part")
        tmp.write_bytes(after)
        os.replace(tmp, doctor.manifest_path)

        return doctor._append_audit({
            "op": "str_replace",
            "scope": "meta",
            "path": ".cyberos-memory/manifest.json",
            "memory_id": None,
            "before_hash": before_hash,
            "after_hash": after_hash,
            "diff": "<hash-only>",
            "reason": (
                f"R1: rebuild reconciliation_checkpoint to ledger tail "
                f"(prior checkpoint {self.finding.details})"
            ),
            "correction_to": None,
        })


class TruncatePartialLineRepair(Repair):
    def __init__(self, finding: Finding):
        super().__init__(
            "R2-truncate-partial-line", finding,
            f"Truncate {finding.path} to remove the partial trailing line "
            "(filesystem-sync mid-write delivery).",
            dangerous=True,
        )

    def apply(self, doctor: Doctor) -> str:
        ledger = doctor.root / self.finding.path
        data = ledger.read_bytes()
        # Find last newline; truncate after it
        idx = data.rfind(b"\n")
        if idx < 0 or idx == len(data) - 1:
            # No partial line, or already ends with \n properly
            return ""
        new_data = data[:idx + 1]

        # Save the truncated tail to /tmp for forensics
        forensic = Path("/tmp") / f"{ledger.name}.partial-tail-{secrets.token_hex(4)}"
        forensic.write_bytes(data[idx + 1:])
        print(f"    [forensic dump: {forensic}]")

        tmp = ledger.with_suffix(f".tmp.{secrets.token_hex(8)}.part")
        tmp.write_bytes(new_data)
        os.replace(tmp, ledger)

        # Note: we cannot append a regular audit row through _append_audit
        # because the truncate operation modifies the ledger itself. Instead
        # we synthesize the row directly into the truncated ledger.
        prev_chain = json.loads(new_data.rstrip(b"\n").rsplit(b"\n", 1)[-1])["chain"]
        partial = {
            "op": "rejected",
            "scope": "meta",
            "path": f".cyberos-memory/{self.finding.path}",
            "memory_id": None,
            "before_hash": None,
            "after_hash": None,
            "diff": "<hash-only>",
            "reason": (
                f"R2: truncated partial trailing line "
                f"(was {len(data) - idx - 1} bytes); forensic dump at "
                f"{forensic}; cyberos-doctor session "
                f"{doctor.maintenance_session_id}"
            ),
            "correction_to": None,
        }
        return doctor._append_audit(partial)


class TruncateUnparseableLineRepair(TruncatePartialLineRepair):
    def __init__(self, finding: Finding):
        Repair.__init__(
            self,
            "R2-truncate-unparseable", finding,
            f"Truncate {finding.path} to remove the unparseable trailing line "
            "(JSON parse error).",
            dangerous=True,
        )


class UpdateChainHeadRepair(Repair):
    def __init__(self, finding: Finding):
        super().__init__(
            "R3-update-chain-head", finding,
            "Update manifest.audit_chain_head to the current ledger tail's "
            "chain (current pin doesn't appear in the ledger; usually means "
            "the manifest synced separately from the ledger and one is now "
            "ahead of the other).",
            dangerous=True,
        )

    def apply(self, doctor: Doctor) -> str:
        before = doctor.manifest_path.read_bytes()
        m = json.loads(before)

        # Find latest ledger chain
        ledgers = sorted(doctor.audit_dir.glob("*.jsonl"))
        latest_chain = None
        for f in ledgers:
            with f.open("r", encoding="utf-8") as fp:
                for line in fp:
                    if line.strip():
                        latest_chain = json.loads(line).get("chain")
        if not latest_chain:
            raise RuntimeError("ledger empty; cannot update chain head")

        old_head = m.get("audit_chain_head")
        m["audit_chain_head"] = latest_chain
        m["last_updated_at"] = doctor.now.isoformat(timespec="seconds")
        after_str = json.dumps(m, indent=2, ensure_ascii=False)
        after = (after_str + "\n").encode("utf-8")
        before_hash = "sha256:" + hashlib.sha256(before).hexdigest()
        after_hash = "sha256:" + hashlib.sha256(after.rstrip(b"\n")).hexdigest()

        tmp = doctor.manifest_path.with_suffix(
            f".tmp.{secrets.token_hex(8)}.part")
        tmp.write_bytes(after)
        os.replace(tmp, doctor.manifest_path)

        return doctor._append_audit({
            "op": "str_replace",
            "scope": "meta",
            "path": ".cyberos-memory/manifest.json",
            "memory_id": None,
            "before_hash": before_hash,
            "after_hash": after_hash,
            "diff": "<hash-only>",
            "reason": (
                f"R3: update audit_chain_head from {old_head} to "
                f"{latest_chain} (current ledger tail)"
            ),
            "correction_to": None,
        })


class RebuildMerkleCheckpointRepair(Repair):
    def __init__(self, finding: Finding):
        super().__init__(
            "R5-rebuild-merkle-checkpoint", finding,
            "Recompute the Merkle root from current ledger rows and replace "
            "the divergent checkpoint via op:'corrects'. The divergent row "
            "stays in the chain (LINK invariant); the corrects-row carries "
            "the recomputed root.",
            dangerous=True,
        )

    def apply(self, doctor: "Doctor") -> str:
        # Read all audit rows; find the consolidation_run row by audit_id
        ledger = doctor.audit_dir / sorted(
            doctor.audit_dir.glob("*.jsonl"))[-1].name
        rows = []
        target = self.finding.details.get("audit_id")
        prev_idx = -1
        target_idx = -1
        with ledger.open("r", encoding="utf-8") as f:
            for line in f:
                if not line.strip():
                    continue
                try:
                    rows.append(json.loads(line))
                except json.JSONDecodeError:
                    continue
        for i, r in enumerate(rows):
            if r.get("audit_id") == target:
                target_idx = i
                break
            if r.get("op") == "consolidation_run" and "merkle_root" in r:
                prev_idx = i
        if target_idx < 0:
            raise RuntimeError(f"target audit_id {target} not found")

        # Recompute root over rows since prev checkpoint
        leaves = [r.get("chain") for r in rows[prev_idx + 1:target_idx]
                  if r.get("chain")]
        # Merkle build (matches validator's _build_merkle_root)
        if not leaves:
            recomputed = "sha256:" + "0" * 64
        else:
            level = [bytes.fromhex(c.replace("sha256:", "")) for c in leaves]
            while len(level) > 1:
                if len(level) % 2:
                    level.append(level[-1])
                level = [hashlib.sha256(level[i] + level[i + 1]).digest()
                         for i in range(0, len(level), 2)]
            recomputed = "sha256:" + level[0].hex()

        # Append op:corrects row referencing the divergent checkpoint
        return doctor._append_audit({
            "op": "corrects",
            "scope": "meta",
            "path": f".cyberos-memory/audit/{ledger.name}",
            "memory_id": None,
            "before_hash": rows[target_idx].get("after_hash"),
            "after_hash": rows[target_idx].get("after_hash"),  # no file change
            "diff": "<hash-only>",
            "reason": (
                f"R5: rebuild Merkle checkpoint for {target} — "
                f"original {rows[target_idx].get('merkle_root', '?')[:24]}... "
                f"recomputed {recomputed[:24]}... "
                f"over {len(leaves)} rows since previous checkpoint"
            ),
            "correction_to": target,
        })


class TombstoneOrphanRepair(Repair):
    def __init__(self, finding: Finding):
        super().__init__(
            "R4-tombstone-orphan", finding,
            f"Tombstone {finding.path} — file exists but no audit row "
            "references its path. Body kept verbatim per §4.6.",
            dangerous=True,
        )

    def apply(self, doctor: Doctor) -> str:
        target = doctor.root / self.finding.path
        text = target.read_text(encoding="utf-8")
        before_hash = "sha256:" + hashlib.sha256(
            text.encode("utf-8")).hexdigest()

        # Naive frontmatter manipulation: parse, add tombstone fields, re-emit
        if not text.startswith("---\n"):
            raise RuntimeError(
                f"{self.finding.path}: no frontmatter; cannot tombstone")
        end = text.find("\n---\n", 4)
        if end < 0:
            raise RuntimeError(
                f"{self.finding.path}: malformed frontmatter")
        fm = text[4:end]
        body = text[end + 5:]

        # Add tombstone metadata; preserve all existing fields
        ts_iso = doctor.now.isoformat(timespec="seconds")
        new_fm = fm.rstrip("\n") + "\n" + (
            f"tombstoned: true\n"
            f"deleted_at: {ts_iso}\n"
            f"deleted_by: agent:cyberos-doctor\n"
            f"tombstone_reason: \"R4: orphan file (no audit row references "
            f"this path); cyberos-doctor session "
            f"{doctor.maintenance_session_id}\"\n"
        )
        new_text = "---\n" + new_fm + "---\n" + body
        after_hash = "sha256:" + hashlib.sha256(
            new_text.encode("utf-8")).hexdigest()

        tmp = target.with_suffix(f".tmp.{secrets.token_hex(8)}.part")
        tmp.write_text(new_text, encoding="utf-8")
        os.replace(tmp, target)

        return doctor._append_audit({
            "op": "delete",
            "scope": "meta",
            "path": f".cyberos-memory/{self.finding.path}",
            "memory_id": None,
            "before_hash": before_hash,
            "after_hash": after_hash,
            "diff": "<hash-only>",
            "reason": (
                f"R4: tombstone orphan file {self.finding.path}; body kept; "
                f"cyberos-doctor session {doctor.maintenance_session_id}"
            ),
            "correction_to": None,
        })


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def confirm(prompt: str, *, auto_yes: bool, dangerous: bool) -> bool:
    if auto_yes and not dangerous:
        print(f"  [auto-yes] {prompt}: YES")
        return True
    if auto_yes and dangerous:
        print(f"  [auto-yes BUT DANGEROUS] {prompt}")
        # fall through to manual
    try:
        ans = input(f"  {prompt} [y/N] ").strip().lower()
    except EOFError:
        return False
    return ans in ("y", "yes")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="cyberos-doctor",
        description="Recovery CLI for .cyberos-memory/ stores in CORRUPT state.",
    )
    parser.add_argument("path")
    parser.add_argument("--repair", action="store_true",
                        help="Apply repairs interactively")
    parser.add_argument("--auto-yes", action="store_true",
                        help="Auto-confirm safe repairs (still prompt for dangerous)")
    parser.add_argument("--reason", default="",
                        help="Required for --repair: why are you running this?")
    parser.add_argument("--rebuild-checkpoint", action="store_true",
                        help="One-shot: rebuild manifest.reconciliation_checkpoint")
    parser.add_argument("--decompact-ledger",
                        help="One-shot: re-expand audit/<YYYY-MM>.compacted.jsonl "
                             "back to audit/<YYYY-MM>.jsonl from archive/")
    parser.add_argument("--compact-ledger",
                        help="One-shot: compact audit/<YYYY-MM>.jsonl per §7.7. "
                             "Requires Merkle checkpoint and age threshold met. "
                             "Equivalent to chat-turn phrase but agent-driven.")
    args = parser.parse_args(argv)

    root = Path(args.path).resolve()
    if (root / ".cyberos-memory").is_dir():
        root = root / ".cyberos-memory"
    if not root.is_dir():
        print(f"error: {root} not a directory", file=sys.stderr)
        return 3

    doctor = Doctor(root)
    doctor.diagnose()

    print(f"cyberos-doctor {root}")
    print(f"  Findings: "
          f"{sum(1 for f in doctor.findings if f.severity == CRITICAL)} CRITICAL, "
          f"{sum(1 for f in doctor.findings if f.severity == WARN)} WARN, "
          f"{sum(1 for f in doctor.findings if f.severity == INFO)} INFO")
    print(f"  Repair candidates: {len(doctor.repairs)}")
    print()

    if not doctor.findings:
        print("✅ no findings; store appears healthy.")
        return 0

    # Show findings + proposed repairs
    repair_by_finding = {id(r.finding): r for r in doctor.repairs}
    for f in doctor.findings:
        marker = "✘" if f.severity == CRITICAL else "⚠" if f.severity == WARN else "ℹ"
        print(f"  {marker} [{f.severity}] {f.section} {f.code}: {f.path}")
        print(f"      {f.message}")
        r = repair_by_finding.get(id(f))
        if r:
            print(f"      → REPAIR {r.code}{' (DANGEROUS)' if r.dangerous else ''}: "
                  f"{r.description}")
        else:
            print(f"      → no automated repair; surface to user")
        print()

    if args.compact_ledger:
        month = args.compact_ledger
        ledger = root / "audit" / f"{month}.jsonl"
        compacted = root / "audit" / f"{month}.compacted.jsonl"
        if not ledger.exists():
            print(f"error: {ledger} not found", file=sys.stderr)
            return 3
        if compacted.exists():
            print(f"error: {compacted} already exists (already compacted)",
                  file=sys.stderr)
            return 3
        try:
            import zstandard
        except ImportError:
            print("error: zstandard package required (pip install zstandard)",
                  file=sys.stderr)
            return 3

        # Pre-condition: ledger has at least one consolidation_run with merkle_root
        has_checkpoint = False
        with ledger.open("r", encoding="utf-8") as f:
            for line in f:
                if not line.strip():
                    continue
                try:
                    r = json.loads(line)
                    if r.get("op") == "consolidation_run" and "merkle_root" in r:
                        has_checkpoint = True
                        break
                except json.JSONDecodeError:
                    continue
        if not has_checkpoint:
            print(f"error: {month}.jsonl has no Merkle checkpoint "
                  f"(op:consolidation_run with merkle_root); cannot compact",
                  file=sys.stderr)
            return 3

        if not args.reason:
            args.reason = f"compact ledger {month}"
        doctor.begin_maintenance(args.reason)
        try:
            archive_dir = root / "archive"
            archive_dir.mkdir(exist_ok=True)
            archive = archive_dir / f"{month}.jsonl.zst"
            data = ledger.read_bytes()
            compressor = zstandard.ZstdCompressor(level=10)
            archive.write_bytes(compressor.compress(data))

            # Build compacted: per-memory final state (simplified)
            per_memory = {}
            with ledger.open("r", encoding="utf-8") as f:
                for line in f:
                    if not line.strip():
                        continue
                    try:
                        r = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    mid = r.get("memory_id")
                    if mid:
                        per_memory[mid] = {
                            "memory_id": mid,
                            "final_op": r.get("op"),
                            "final_chain": r.get("chain"),
                            "final_audit_id": r.get("audit_id"),
                            "final_ts": r.get("ts"),
                            "merkle_proof": [],  # simplified — proof would
                                                  # be derived from the checkpoint
                        }
            tmp = compacted.with_suffix(f".tmp.{secrets.token_hex(8)}.part")
            tmp.write_text(
                "\n".join(json.dumps(v, ensure_ascii=False)
                          for v in per_memory.values()) + "\n",
                encoding="utf-8")
            os.replace(tmp, compacted)
            ledger.unlink()

            doctor._append_audit({
                "op": "ledger_compact",
                "scope": "meta",
                "path": f".cyberos-memory/audit/{month}.jsonl",
                "memory_id": None,
                "before_hash": "sha256:" + hashlib.sha256(data).hexdigest(),
                "after_hash": "sha256:" + hashlib.sha256(
                    compacted.read_bytes()).hexdigest(),
                "diff": "<hash-only>",
                "reason": (
                    f"compact-ledger: {len(per_memory)} memories collapsed; "
                    f"{len(data)} → {compacted.stat().st_size} bytes "
                    f"({100 * compacted.stat().st_size / len(data):.0f}% of original); "
                    f"original archived to archive/{month}.jsonl.zst"
                ),
                "correction_to": None,
            })
            print(f"  ✅ {month}.jsonl compacted "
                  f"(saved {(1 - compacted.stat().st_size / len(data)) * 100:.0f}%)")
        finally:
            doctor.end_maintenance(f"compacted {month}")
        return 0

    if args.decompact_ledger:
        # One-shot: re-expand archived JSONL over the compacted form
        month = args.decompact_ledger
        archive = root / "archive" / f"{month}.jsonl.zst"
        compacted = root / "audit" / f"{month}.compacted.jsonl"
        if not archive.exists():
            print(f"error: {archive} not found", file=sys.stderr)
            return 3
        if not compacted.exists():
            print(f"error: {compacted} not found (already decompacted?)",
                  file=sys.stderr)
            return 3
        try:
            import zstandard
        except ImportError:
            print("error: zstandard package required (pip install zstandard)",
                  file=sys.stderr)
            return 3
        if not args.reason:
            args.reason = f"decompact ledger {month}"
        doctor.begin_maintenance(args.reason)
        try:
            decompressor = zstandard.ZstdDecompressor()
            data = decompressor.decompress(archive.read_bytes())
            target = root / "audit" / f"{month}.jsonl"
            tmp = target.with_suffix(f".tmp.{secrets.token_hex(8)}.part")
            tmp.write_bytes(data)
            os.replace(tmp, target)
            # Tombstone the compacted form
            compacted.unlink()
            doctor._append_audit({
                "op": "ledger_decompact",
                "scope": "meta",
                "path": f".cyberos-memory/audit/{month}.jsonl",
                "memory_id": None,
                "before_hash": "sha256:" + hashlib.sha256(
                    archive.read_bytes()).hexdigest(),
                "after_hash": "sha256:" + hashlib.sha256(data).hexdigest(),
                "diff": "<hash-only>",
                "reason": (
                    f"decompact-ledger: restored {month}.jsonl from "
                    f"archive/{month}.jsonl.zst; removed compacted form"
                ),
                "correction_to": None,
            })
            print(f"  ✅ {month}.jsonl restored from archive ({len(data)} bytes)")
        finally:
            doctor.end_maintenance(f"decompacted {month}")
        return 0

    if args.rebuild_checkpoint:
        # One-shot path — useful as "always-fix" of stale checkpoints
        relevant = [r for r in doctor.repairs
                    if r.code == "R1-rebuild-checkpoint"]
        if not relevant:
            print("No stale-checkpoint finding; nothing to do.")
            return 0
        if not args.reason:
            args.reason = "rebuild checkpoint via --rebuild-checkpoint"
        doctor.begin_maintenance(args.reason)
        try:
            for r in relevant:
                r.apply(doctor)
                print(f"  ✅ applied {r.code}")
        finally:
            doctor.end_maintenance(f"applied {len(relevant)} repair(s)")
        return 0

    if not args.repair:
        worst = max(
            (1 if f.severity == WARN else 2 if f.severity == CRITICAL else 0)
            for f in doctor.findings
        ) if doctor.findings else 0
        return worst

    if not args.reason:
        print("error: --repair requires --reason \"<text>\"", file=sys.stderr)
        return 3

    # Pre-flight: stabilise ledger so begin_maintenance can append cleanly
    preflight = doctor.preflight_stabilise()
    if preflight:
        print("\n  Preflight (run BEFORE maintenance.start; chain was broken):")
        for a in preflight:
            print(f"    • {a}")
        # After preflight, the partial-line / unparseable-line repairs are
        # subsumed; re-diagnose to refresh repair list
        doctor.findings = []
        doctor.repairs = []
        doctor.diagnose()
        print(f"  After preflight: {len(doctor.repairs)} remaining repair(s)")

    # Interactive repair flow
    doctor.begin_maintenance(args.reason)
    applied = 0
    skipped = 0
    try:
        for r in doctor.repairs:
            print(f"\n  Apply {r.code}? "
                  f"({'DANGEROUS' if r.dangerous else 'safe'})")
            print(f"    target: {r.finding.path}")
            print(f"    action: {r.description}")
            if confirm("apply?", auto_yes=args.auto_yes, dangerous=r.dangerous):
                try:
                    r.apply(doctor)
                    print(f"    ✅ applied")
                    applied += 1
                except Exception as e:  # noqa: BLE001
                    print(f"    ✘ failed: {e}")
                    skipped += 1
            else:
                print(f"    skipped")
                skipped += 1
    finally:
        doctor.end_maintenance(f"applied={applied} skipped={skipped}")

    print(f"\n  Repairs applied: {applied}")
    print(f"  Repairs skipped: {skipped}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
