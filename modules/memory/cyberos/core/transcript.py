"""
cyberos.core.transcript — session transcript ledger
(TASK-MEMORY-119, AGENTS.md §18).

The transcript ledger is an OPT-IN turn-level audit trail for agent-user
conversations. Operators opt in per conversation via the lifecycle CLI;
without `cyberos transcript start ...` no transcript rows are produced.

Storage:

* Session bodies live at ``<memory-root>/sessions/<YYYY-MM-DD>/<id>.binlog.zst``.
  Date partition is the SESSION START date — sessions spanning midnight
  remain in their original date directory.
* The active session is indicated by a pointer file at
  ``<memory-root>/sessions/.active`` (contents = session id). Single-active
  enforcement (§18.7 / §18.8).
* Frame format mirrors §6.2 (length-prefixed binary, msgspec canonical JSON).
  Compressed with zstd on `session.end`.

Audit chain:

* `session.start` / `session.end` / `session.purged` are summary rows on
  the main audit chain so TASK-MEMORY-115 dream / TASK-MEMORY-120 history can
  discover sessions without reading the binlog.
* Memory writes during an active session carry `extra.session_id` per
  AGENTS.md §18.7 — wired via a small writer-side hook.

Classification:

* `confidential` (default per Stephen's 2026-05-19 decision) — encryption
  recommended but not required.
* `restricted` — encryption envelope §5.4 REQUIRED. The `content` field
  in turn payloads is replaced with `content_cipher`.
* `public` / `internal` are NOT permitted on sessions (§18.3).
"""

from __future__ import annotations

import hashlib
import json
import struct
import time
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Iterator, Literal, Optional

Classification = Literal["confidential", "restricted"]
Role = Literal["user", "assistant", "system", "tool"]

_ALLOWED_CLASSIFICATIONS: frozenset[str] = frozenset({"confidential", "restricted"})
_ALLOWED_ROLES: frozenset[str] = frozenset({"user", "assistant", "system", "tool"})

# Frame format — mirrors §6.2: [u32 length BE][u64 turn_seq BE][u64 ts_ns BE]
# then a UTF-8 JSON payload of length `length`.
_FRAME_HDR = struct.Struct(">IQQ")  # 20 bytes


class TranscriptError(RuntimeError):
    """Generic transcript lifecycle error."""


class ProtocolAmendmentMissing(RuntimeError):
    """Raised when AGENTS.md §18 is not yet anchored (TASK-MEMORY-119 §1 #12)."""


def _has_section_18(store: Path) -> bool:
    """Anchor check for the §18 amendment."""
    candidates: list[Path] = [store / "AGENTS.md"]
    for parent in [store, *store.parents][:6]:
        candidates.append(parent / "modules" / "memory" / "AGENTS.md")
    for c in candidates:
        if c.exists():
            try:
                body = c.read_text(encoding="utf-8", errors="ignore")
            except Exception:
                continue
            if "§18" in body and ("Session transcript ledger" in body or "transcript ledger" in body.lower()):
                return True
    return False


@dataclass
class Session:
    """In-memory representation of an active or recently-ended session."""

    id: str
    started_at: datetime
    classification: Classification
    retention_days: int = 30
    actor: str = "agent"
    ended_at: Optional[datetime] = None
    ended_reason: Optional[str] = None
    binlog_path: Optional[Path] = None


# ────────────────────────────────────────────────────────────────────
# helpers — disk layout
# ────────────────────────────────────────────────────────────────────


def _sessions_root(store: Path) -> Path:
    return store / "sessions"


def _active_pointer(store: Path) -> Path:
    return _sessions_root(store) / ".active"


def _date_dir(store: Path, dt: datetime) -> Path:
    return _sessions_root(store) / dt.strftime("%Y-%m-%d")


def _binlog_path_for(store: Path, session_id: str, started_at: datetime) -> Path:
    return _date_dir(store, started_at) / f"{session_id}.binlog"


def _locate_binlog(store: Path, session_id: str) -> Optional[Path]:
    """Find an existing session's binlog by scanning date dirs (newest first)."""
    root = _sessions_root(store)
    if not root.is_dir():
        return None
    for date_dir in sorted(root.iterdir(), reverse=True):
        if not date_dir.is_dir() or date_dir.name == ".active":
            continue
        cand = date_dir / f"{session_id}.binlog"
        if cand.exists():
            return cand
        cand_zst = date_dir / f"{session_id}.binlog.zst"
        if cand_zst.exists():
            return cand_zst
    return None


def _append_frame(binlog: Path, payload: bytes, turn_seq: int) -> None:
    """Append one length-prefixed frame to the binlog (no fsync — slice-3 lazy)."""
    ts_ns = time.time_ns()
    header = _FRAME_HDR.pack(len(payload), turn_seq, ts_ns)
    with open(binlog, "ab") as f:
        f.write(header)
        f.write(payload)


def _count_frames(binlog: Path) -> int:
    """Count frames in a binlog by walking the length-prefix headers."""
    if not binlog.exists():
        return 0
    count = 0
    with open(binlog, "rb") as f:
        while True:
            hdr = f.read(_FRAME_HDR.size)
            if len(hdr) < _FRAME_HDR.size:
                break
            length, _seq, _ts = _FRAME_HDR.unpack(hdr)
            f.seek(length, 1)
            count += 1
    return count


def _iter_frames(binlog: Path) -> Iterator[tuple[int, int, dict]]:
    """Yield (turn_seq, ts_ns, payload_dict) for every frame.

    Handles both ``.binlog`` (active) and ``.binlog.zst`` (sealed) inputs.
    """
    if str(binlog).endswith(".zst"):
        import zstandard as zstd
        with open(binlog, "rb") as f:
            data = zstd.ZstdDecompressor().decompress(f.read())
        buf = data
        offset = 0
        while offset + _FRAME_HDR.size <= len(buf):
            hdr = buf[offset:offset + _FRAME_HDR.size]
            length, seq, ts_ns = _FRAME_HDR.unpack(hdr)
            offset += _FRAME_HDR.size
            payload = buf[offset:offset + length]
            offset += length
            try:
                yield seq, ts_ns, json.loads(payload.decode("utf-8"))
            except Exception:
                continue
        return
    if not binlog.exists():
        return
    with open(binlog, "rb") as f:
        while True:
            hdr = f.read(_FRAME_HDR.size)
            if len(hdr) < _FRAME_HDR.size:
                break
            length, seq, ts_ns = _FRAME_HDR.unpack(hdr)
            payload = f.read(length)
            if len(payload) < length:
                break
            try:
                yield seq, ts_ns, json.loads(payload.decode("utf-8"))
            except Exception:
                continue


def _sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


# ────────────────────────────────────────────────────────────────────
# encryption (restricted classification)
# ────────────────────────────────────────────────────────────────────


def _encrypt_content(content: str, store: Path) -> dict:
    """Minimal §5.4 envelope wrapper.

    Slice-3 ships a placeholder envelope (the cipher/key handling lives in
    `cyberos.core.crypto_mode` and TASK-MEMORY-117's encryption pipeline; for
    transcripts the body is stored encrypted-at-the-payload level so the
    meta-frame remains plaintext per §18.4). When `cryptography` package
    is unavailable, this falls back to a marker that the operator can
    spot in `cyberos transcript read --decrypt` output.
    """
    try:
        from cryptography.hazmat.primitives.ciphers.aead import AESGCM  # noqa: F401
    except ImportError:
        # Fallback: structured marker; operator can pip-install
        # cryptography to engage real envelope.
        return {
            "cipher": "aes-256-gcm",
            "key_id": "_unconfigured",
            "nonce": "0" * 24,
            "ciphertext_hash": hashlib.sha256(content.encode("utf-8")).hexdigest(),
            "_warning": "cryptography package not installed; payload not encrypted",
            "_placeholder_content": content,
        }
    # When cryptography IS available, encrypt with a per-store key
    # derived from the manifest's fingerprint (slice-3 convenience).
    # Production deployments should provision proper KMS keys.
    key_material = hashlib.sha256(
        (str(store) + ":transcript-key").encode("utf-8")
    ).digest()
    nonce = hashlib.sha256(
        (content + str(time.time_ns())).encode("utf-8")
    ).digest()[:12]
    cipher = AESGCM(key_material)
    ct = cipher.encrypt(nonce, content.encode("utf-8"), associated_data=None)
    return {
        "cipher": "aes-256-gcm",
        "key_id": "store-derived",
        "nonce": nonce.hex(),
        "ciphertext_hash": hashlib.sha256(ct).hexdigest(),
        "_ct_hex": ct.hex(),
    }


def _decrypt_content(envelope: dict, store: Path) -> str:
    """Reverse of `_encrypt_content`."""
    if envelope.get("_placeholder_content") is not None:
        return envelope["_placeholder_content"]
    try:
        from cryptography.hazmat.primitives.ciphers.aead import AESGCM
    except ImportError:
        return "[encrypted content; install cryptography to decrypt]"
    key_material = hashlib.sha256(
        (str(store) + ":transcript-key").encode("utf-8")
    ).digest()
    nonce = bytes.fromhex(envelope["nonce"])
    ct = bytes.fromhex(envelope["_ct_hex"])
    cipher = AESGCM(key_material)
    pt = cipher.decrypt(nonce, ct, associated_data=None)
    return pt.decode("utf-8")


# ────────────────────────────────────────────────────────────────────
# lifecycle: start / append / end / purge
# ────────────────────────────────────────────────────────────────────


def start(
    writer,  # cyberos.core.writer.Writer
    *,
    session_id: str,
    classification: Classification = "confidential",
    retention_days: int = 30,
    actor: str = "agent",
) -> Session:
    """Open a new session (TASK-MEMORY-119 §1 #1)."""
    if not _has_section_18(writer.store):
        raise ProtocolAmendmentMissing(
            "AGENTS.md §18 not anchored. Approve via:\n"
            "  APPROVE protocol change P22 §18\n"
            "and ensure AGENTS.md contains §18 Session transcript ledger."
        )
    if classification not in _ALLOWED_CLASSIFICATIONS:
        raise ValueError(
            f"classification={classification!r} not permitted on sessions; "
            f"choose one of {sorted(_ALLOWED_CLASSIFICATIONS)}"
        )
    if not session_id or not isinstance(session_id, str):
        raise ValueError("session_id must be a non-empty string")
    if not retention_days >= 1:
        raise ValueError(f"retention_days must be ≥ 1; got {retention_days}")

    active = _active_pointer(writer.store)
    if active.exists():
        raise TranscriptError(
            f"a session is already active ({active.read_text(encoding='utf-8').strip()!r}); "
            f"end it before starting another"
        )

    started_at = datetime.now(timezone.utc)
    date_dir = _date_dir(writer.store, started_at)
    date_dir.mkdir(parents=True, exist_ok=True)

    binlog = _binlog_path_for(writer.store, session_id, started_at)
    if binlog.exists() or (binlog.parent / f"{session_id}.binlog.zst").exists():
        raise TranscriptError(
            f"session id {session_id!r} already exists on {started_at.date()}"
        )

    binlog.write_bytes(b"")                           # create empty binlog
    active.parent.mkdir(parents=True, exist_ok=True)
    active.write_text(session_id, encoding="utf-8")

    # Emit summary row on main chain
    from cyberos.core.writer import AuditRecord
    writer.submit(AuditRecord(
        op="session.start",
        path=str(binlog.relative_to(writer.store)),
        actor=actor,
        extra={
            "session_id": session_id,
            "started_at": started_at.isoformat(),
            "classification": classification,
            "retention_days": retention_days,
            "actor": actor,
        },
    ))
    return Session(
        id=session_id,
        started_at=started_at,
        classification=classification,
        retention_days=retention_days,
        actor=actor,
        binlog_path=binlog,
    )


def append(
    writer,
    *,
    session_id: str,
    role: Role,
    content: str,
    redactions_applied: Optional[bool] = None,
) -> int:
    """Append one turn to an active session (TASK-MEMORY-119 §1 #1).

    Returns the assigned turn_seq (starts at 0).
    """
    if role not in _ALLOWED_ROLES:
        raise ValueError(
            f"role={role!r} not in {sorted(_ALLOWED_ROLES)}"
        )
    active = _active_pointer(writer.store)
    if not active.exists():
        raise TranscriptError("no active session; start one first")
    current = active.read_text(encoding="utf-8").strip()
    if current != session_id:
        raise TranscriptError(
            f"active session is {current!r}, not {session_id!r}"
        )

    # Read classification from the most recent session.start row's
    # extras. Slice-3 keeps this simple by walking the main chain.
    classification = _classification_for(writer, session_id)

    # Locate the binlog
    binlog = _locate_binlog(writer.store, session_id)
    if binlog is None:
        raise TranscriptError(f"binlog for session {session_id!r} not found")
    if binlog.suffix == ".zst":
        raise TranscriptError(
            f"session {session_id!r} is sealed; cannot append after end"
        )

    turn_seq = _count_frames(binlog)
    ts = datetime.now(timezone.utc).isoformat()

    payload: dict = {
        "session_id": session_id,
        "role": role,
        "turn_seq": turn_seq,
        "ts": ts,
    }
    if classification == "restricted":
        payload["content_cipher"] = _encrypt_content(content, writer.store)
    else:
        payload["content"] = content
    if redactions_applied is not None:
        payload["redactions_applied"] = redactions_applied

    _append_frame(binlog, json.dumps(payload).encode("utf-8"), turn_seq)
    return turn_seq


def end(
    writer,
    *,
    session_id: str,
    reason: Optional[str] = None,
    seal_binlog: bool = True,
) -> Session:
    """End an active session (TASK-MEMORY-119 §1 #1).

    On `seal_binlog=True`, compresses the .binlog with zstd → .binlog.zst
    and removes the original. Slice-3 default; operators can keep the raw
    binlog via `seal_binlog=False`.
    """
    active = _active_pointer(writer.store)
    if not active.exists():
        raise TranscriptError(f"session {session_id!r} is not active (no active pointer)")
    current = active.read_text(encoding="utf-8").strip()
    if current != session_id:
        raise TranscriptError(
            f"active session is {current!r}, not {session_id!r}"
        )

    binlog = _locate_binlog(writer.store, session_id)
    if binlog is None:
        raise TranscriptError(f"binlog for session {session_id!r} not found")

    ended_at = datetime.now(timezone.utc)
    turns_count = _count_frames(binlog) if binlog.suffix == ".binlog" else 0
    final_hash = _sha256_file(binlog)

    # Seal (compress) the binlog
    if seal_binlog and binlog.suffix == ".binlog":
        try:
            import zstandard as zstd
            raw = binlog.read_bytes()
            ct = zstd.ZstdCompressor(level=10).compress(raw)
            sealed = binlog.with_suffix(".binlog.zst")
            sealed.write_bytes(ct)
            binlog.unlink()
            binlog = sealed
        except ImportError:
            pass  # leave raw if zstandard not installed

    active.unlink()

    from cyberos.core.writer import AuditRecord
    writer.submit(AuditRecord(
        op="session.end",
        path=str(binlog.relative_to(writer.store)),
        actor="transcript-end",
        extra={
            "session_id": session_id,
            "ended_at": ended_at.isoformat(),
            "ended_reason": reason or "explicit",
            "turns_count": turns_count,
            "binlog_hash": final_hash,
        },
    ))

    return Session(
        id=session_id,
        started_at=ended_at,  # not tracked here; caller knows
        classification=_classification_for(writer, session_id) or "confidential",
        ended_at=ended_at,
        ended_reason=reason,
        binlog_path=binlog,
    )


def purge_expired(
    writer,
    *,
    retention_days: Optional[int] = None,
    dry_run: bool = False,
    actor: str = "retention-purger",
) -> dict:
    """Scan ``sessions/<date>/`` dirs older than retention_days; tombstone
    each session's binlog + emit `session.purged` rows."""
    from cyberos.core.writer import AuditRecord

    root = _sessions_root(writer.store)
    if not root.is_dir():
        return {"purged_count": 0, "dry_run": dry_run, "purged": []}

    retention = retention_days if retention_days is not None else 30
    now = datetime.now(timezone.utc)
    purged: list[dict] = []

    for date_dir in sorted(root.iterdir()):
        if not date_dir.is_dir() or date_dir.name == ".active":
            continue
        try:
            date = datetime.strptime(date_dir.name, "%Y-%m-%d").replace(tzinfo=timezone.utc)
        except ValueError:
            continue
        age_days = (now - date).days
        if age_days <= retention:
            continue
        for binlog in list(date_dir.glob("*.binlog*")):
            session_id = binlog.stem.replace(".binlog", "")
            purged.append({
                "session_id": session_id,
                "date": date_dir.name,
                "age_days": age_days,
            })
            if not dry_run:
                # Tombstone marker replaces the binlog body
                tombstone = json.dumps({
                    "session_id": session_id,
                    "original_started_at": date.isoformat(),
                    "purged_at": now.isoformat(),
                    "tombstone": True,
                }).encode("utf-8")
                binlog.write_bytes(tombstone)
                writer.submit(AuditRecord(
                    op="session.purged",
                    path=str(binlog.relative_to(writer.store)),
                    actor=actor,
                    extra={
                        "session_id": session_id,
                        "original_started_at": date.isoformat(),
                        "purged_at": now.isoformat(),
                        "reason": "retention_expired",
                    },
                ))
    return {"purged_count": len(purged), "dry_run": dry_run, "purged": purged}


# ────────────────────────────────────────────────────────────────────
# read + list (operator surfaces)
# ────────────────────────────────────────────────────────────────────


def read(store: Path, session_id: str, *, decrypt: bool = False) -> list[dict]:
    """Render a session's turns as a list of payload dicts.

    For `restricted` sessions, without `decrypt=True` the content is replaced
    with a placeholder string. With `decrypt=True`, attempts to decrypt and
    returns the plaintext.
    """
    binlog = _locate_binlog(store, session_id)
    if binlog is None:
        return []
    out: list[dict] = []
    for seq, ts_ns, payload in _iter_frames(binlog):
        if payload.get("tombstone"):
            out.append({
                "session_id": payload.get("session_id"),
                "tombstone": True,
                "purged_at": payload.get("purged_at"),
            })
            continue
        if "content_cipher" in payload:
            if decrypt:
                payload["content"] = _decrypt_content(payload["content_cipher"], store)
            else:
                payload["content"] = "[encrypted content; --decrypt to read]"
        out.append(payload)
    return out


def list_sessions(store: Path, *, since: Optional[timedelta] = None) -> list[dict]:
    """Enumerate sessions in the store.

    Returns a list of `{session_id, started_date, classification?, state?}` dicts.
    """
    root = _sessions_root(store)
    if not root.is_dir():
        return []
    cutoff = (datetime.now(timezone.utc) - since) if since else None
    ended_ids: set[str] = set()
    purged_ids: set[str] = set()
    try:
        from cyberos.core.dream._audit_iter import iter_audit_rows

        for row in iter_audit_rows(store):
            session_id = row.get("extra", {}).get("session_id")
            if not session_id:
                continue
            if row.get("op") == "session.end":
                ended_ids.add(str(session_id))
            elif row.get("op") == "session.purged":
                purged_ids.add(str(session_id))
    except Exception:
        # Listing should remain best-effort even when the main chain is not
        # readable; the binlog suffix/tombstone checks below still provide a
        # useful local view.
        pass
    out: list[dict] = []
    for date_dir in sorted(root.iterdir()):
        if not date_dir.is_dir() or date_dir.name == ".active":
            continue
        try:
            date = datetime.strptime(date_dir.name, "%Y-%m-%d").replace(tzinfo=timezone.utc)
        except ValueError:
            continue
        if cutoff and date < cutoff:
            continue
        for binlog in sorted(date_dir.glob("*.binlog*")):
            session_id = binlog.stem.replace(".binlog", "")
            sealed = binlog.suffix == ".zst"
            state = "ended" if sealed or session_id in ended_ids else "active"
            # Detect tombstone
            try:
                head = binlog.read_bytes()[:300]
                if (
                    session_id in purged_ids
                    or b'"tombstone": true' in head
                    or b'"tombstone":true' in head
                ):
                    state = "purged"
            except Exception:
                pass
            out.append({
                "session_id": session_id,
                "started_date": date_dir.name,
                "state": state,
                "binlog_path": str(binlog.relative_to(store)),
            })
    return out


def _classification_for(writer, session_id: str) -> Optional[Classification]:
    """Walk the main chain backward to find this session's `session.start` row.

    Slice-3 implementation — linear scan; acceptable for moderate chains.
    Slice-4 can cache classification in `.active` alongside the session id.
    """
    try:
        from cyberos.core.dream._audit_iter import iter_audit_rows
    except Exception:
        return None
    for row in iter_audit_rows(writer.store):
        if row.get("op") == "session.start":
            ex = row.get("extra") or {}
            if ex.get("session_id") == session_id:
                cls = ex.get("classification")
                if cls in _ALLOWED_CLASSIFICATIONS:
                    return cls
    return None


def active_session_id(store: Path) -> Optional[str]:
    """Return the currently-active session id, or None."""
    ptr = _active_pointer(store)
    if not ptr.exists():
        return None
    s = ptr.read_text(encoding="utf-8").strip()
    return s or None
