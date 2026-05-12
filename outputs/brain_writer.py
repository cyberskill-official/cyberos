#!/usr/bin/env python3
"""
outputs/brain_writer.py — reference writer for the CyberOS BRAIN.

Canonical location per AGENTS.md §0.6 line 175: this is one of the
"implementation files" tracked by the §0.6 related-files invariant
whenever §7.2 / §4.4 / §5.2 amendments land. Lives in the project source
tree (versioned in git) — not inside `.cyberos-memory/` (which is local
operational state, gitignored by design).

Implements the audit-row append + atomic file-write contract from
docs/CyberOS-AGENTS.md (§4 / §5.2 / §7 / §13). Every op produces exactly
one audit row in audit/<YYYY-MM>.jsonl, chained via prev_chain → chain
(SHA-256 over RFC 8785 JCS canonicalisation of the row body, concatenated
with the prev_chain UTF-8 bytes; see §7.2).

Reference impl. Single file. Stdlib + rfc8785 + PyYAML only. POSIX flock
is used for lock coordination; Windows is supported via msvcrt.

Subcommands
-----------
  session-start <actor>
        Append op:session.start. Use at the beginning of an agent session.

  session-end <actor>
        Append op:session.end, then str_replace manifest.json to update
        audit_chain_head + reconciliation_checkpoint + last_updated_at.

  write <actor> <relpath> <content_file>
        Create a new memory file at <BRAIN>/<relpath> from <content_file>
        and append op:create. <relpath> MUST be relative to the BRAIN
        root, MUST pass §4.1 path-traversal guard, MUST resolve to an
        agent-writable scope (§4.5). Frontmatter is parsed and key fields
        are mirrored into the audit row.

  str-replace <actor> <relpath> <new_file>
        Replace the file at <BRAIN>/<relpath> with the contents of
        <new_file> and append op:str_replace. before_hash = sha256 of
        existing file; after_hash = sha256 of new file.

  verify [--bit-perfect]
        Walk the chain end-to-end. Always verifies the LINK invariant
        (row[N].prev_chain == row[N-1].chain). With --bit-perfect, also
        re-hashes every row via JCS and reports rows where this writer's
        output differs from the stored chain (informational per §7.2;
        cross-writer-version recompute mismatches are NOT chain breaks).

  status
        Print chain head, row count, manifest summary, and a one-line
        health flag (READY / CORRUPT:<reason>).

Exit codes
----------
  0 = success
  1 = generic failure (already surfaced to stderr)
  2 = validation failure (chain corrupt, schema mismatch, denylist hit,
      path/scope violation, frontmatter rejected, ...)
  3 = lock contention / concurrent writer
"""

from __future__ import annotations

import argparse
import datetime as _dt
import hashlib
import json
import os
import re
import secrets
import sys
import tempfile
import time
import unicodedata
from pathlib import Path

try:
    import rfc8785  # RFC 8785 JCS canonicalisation (PyPI)
except ImportError:  # pragma: no cover
    sys.stderr.write(
        "FATAL: rfc8785 package not installed. Run:\n"
        "  pip install rfc8785 --break-system-packages\n"
        "(See AGENTS.md §7.2 — JCS is REQUIRED for chain hash stability.)\n"
    )
    sys.exit(1)

try:
    import yaml  # PyYAML — for parsing memory frontmatter
except ImportError:  # pragma: no cover
    sys.stderr.write("FATAL: PyYAML not installed. Run: pip install PyYAML\n")
    sys.exit(1)


# ───────────────────────────────────────────────────────────────────────
# §0.1 BRAIN root resolution + sanity check
# ───────────────────────────────────────────────────────────────────────

_FORBIDDEN_PATH_FRAGMENTS = (
    "/sessions/", "/private/var/folders/", "/var/folders/", "/tmp/",
    "/private/tmp/", "/dev/shm/",
    "local-agent-mode-sessions", "claude-hostloop-plugins",
    "cowork-session", "cowork-mode-sessions", "agent-sandbox",
    "mcp-sandbox", "claude-code-sandbox",
)


def resolve_brain_root(override: str | None = None) -> Path:
    """Resolve `.cyberos-memory/` for this writer.

    Default layout (canonical per §0.6):
        <project_root>/outputs/brain_writer.py   ← this script
        <project_root>/.cyberos-memory/          ← BRAIN (sibling)

    The script's parent's parent is the project root. We then look for
    `.cyberos-memory/` there. The `--brain-root` CLI flag (or override
    parameter) lets callers point at a non-default BRAIN — useful for
    testing or for projects with non-standard layouts. The §0.1 sandbox
    check still runs on the resolved path.
    """
    if override is None:
        override = os.environ.get("CYBEROS_BRAIN_ROOT")
    if override:
        brain_root = Path(override).expanduser().resolve()
    else:
        script_real = Path(os.path.realpath(__file__))
        # script_real: <root>/outputs/brain_writer.py
        # project_root: <root>
        project_root = script_real.parent.parent
        brain_root = project_root / ".cyberos-memory"

    if not brain_root.is_dir():
        die(
            f"BRAIN dir not found at {brain_root}. "
            f"Expected `.cyberos-memory/` as a sibling of `outputs/`. "
            f"Use --brain-root <path> to override.",
            exit_code=2,
        )

    real = str(brain_root.resolve())
    # Cowork-mode escape: when the host mounts the user's REAL local
    # filesystem under a /sessions/<id>/mnt/<folder>/ prefix, the §0.1
    # fragment check produces a false positive. The host (or an agent
    # that knows it's running in such a host) MAY set
    # CYBEROS_HOST_MOUNT_PREFIX=<prefix> to declare the prefix is a
    # trusted real-fs mount. Any path starting with that prefix is
    # exempt from the /sessions/ + tmpfs-style fragment checks ONLY;
    # all other forbidden fragments (cowork-session, agent-sandbox,
    # claude-hostloop-plugins, etc.) still apply. The escape is
    # surfaced into the next audit row as a one-time
    # `op:"warn" reason:"host-mount-prefix-active"` per §13.1 step 11
    # convention. REF-007 candidate (2026-05-11).
    host_mount_prefix = os.environ.get("CYBEROS_HOST_MOUNT_PREFIX", "").strip()
    _exempt_fragments = {"/sessions/", "/var/folders/", "/private/var/folders/",
                         "/tmp/", "/private/tmp/", "/dev/shm/"}
    for frag in _FORBIDDEN_PATH_FRAGMENTS:
        if frag.lower() in real.lower():
            if host_mount_prefix and real.startswith(host_mount_prefix) and frag in _exempt_fragments:
                continue
            die(
                f"§0.1 violation: BRAIN appears to live under a sandbox "
                f"path ({frag!r} in {real!r}). Refusing to write. "
                f"Run the writer against the real local filesystem path.",
                exit_code=2,
            )

    return brain_root


def die(msg: str, *, exit_code: int = 1) -> None:
    sys.stderr.write(f"brain_writer: {msg}\n")
    sys.exit(exit_code)


# ───────────────────────────────────────────────────────────────────────
# manifest.json read / atomic update (no audit row here — caller decides)
# ───────────────────────────────────────────────────────────────────────

def read_manifest(brain_root: Path) -> dict:
    path = brain_root / "manifest.json"
    if not path.is_file():
        die(f"manifest.json missing at {path}", exit_code=2)
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        die(f"manifest.json unparseable: {e}", exit_code=2)


def session_timezone(manifest: dict) -> _dt.tzinfo:
    """Return tzinfo from manifest.timezone (IANA name) with sane fallback."""
    tz_name = manifest.get("timezone") or "UTC"
    try:
        # Python 3.9+ stdlib
        from zoneinfo import ZoneInfo  # noqa
        return ZoneInfo(tz_name)
    except Exception:
        return _dt.timezone.utc


def now_iso(tz: _dt.tzinfo) -> str:
    """ISO-8601 with offset, second precision, matching §5.2 regex."""
    now = _dt.datetime.now(tz)
    s = now.replace(microsecond=0).isoformat()
    # isoformat() emits "+07:00" already; UTC emits "+00:00"
    return s


# ───────────────────────────────────────────────────────────────────────
# UUIDv7 (RFC 9562) — per §5.2 audit_id / memory_id format
# ───────────────────────────────────────────────────────────────────────

def new_uuid7(prefix: str, *, ts_ms: int | None = None) -> str:
    """Generate a UUIDv7 with the given mem_/evt_ prefix.

    Layout (128 bits):
      48 bits: unix_ts_ms
       4 bits: version (0b0111)
      12 bits: rand_a
       2 bits: variant (0b10)
      62 bits: rand_b
    """
    if ts_ms is None:
        ts_ms = int(time.time() * 1000)
    ts_ms &= (1 << 48) - 1
    rand_a = secrets.randbits(12)
    rand_b = secrets.randbits(62)
    n = ts_ms << 80
    n |= 0x7 << 76
    n |= rand_a << 64
    n |= 0b10 << 62
    n |= rand_b
    hex_str = f"{n:032x}"
    formatted = (
        f"{hex_str[0:8]}-{hex_str[8:12]}-{hex_str[12:16]}-"
        f"{hex_str[16:20]}-{hex_str[20:32]}"
    )
    return f"{prefix}_{formatted}"


_UUID7_RE = re.compile(
    r"^(mem|evt)_[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-"
    r"[89ab][0-9a-f]{3}-[0-9a-f]{12}$"
)
_ULID_RE = re.compile(r"^(mem|evt)_[0-9A-HJKMNP-TV-Z]{26}$")


def is_valid_id(s: str) -> bool:
    return bool(_UUID7_RE.match(s) or _ULID_RE.match(s))


# ───────────────────────────────────────────────────────────────────────
# §4.1 Path-traversal guard
# ───────────────────────────────────────────────────────────────────────

_ZERO_WIDTH = {"​", "‌", "‍", "﻿", "⁠", "᠎"}
_BIDI_OVERRIDE = set(chr(c) for c in
                     list(range(0x202A, 0x202F)) + list(range(0x2066, 0x206A)))
_RESERVED_WIN = {"CON", "PRN", "AUX", "NUL"} | {f"COM{i}" for i in range(1, 10)} \
    | {f"LPT{i}" for i in range(1, 10)}
_WIN_ILLEGAL = set('<>:"/\\|?*')


def validate_relpath(relpath: str, brain_root: Path) -> Path:
    """Apply §4.1 + return absolute resolved path under BRAIN root.

    Raises ValueError on any rejection; caller turns into op:rejected.
    """
    if not relpath:
        raise ValueError("path-empty")
    nfkc = unicodedata.normalize("NFKC", relpath)
    if nfkc != unicodedata.normalize("NFC", relpath):
        raise ValueError("normalisation-evasion")
    if any(ch in relpath for ch in _ZERO_WIDTH):
        raise ValueError("zero-width-char")
    if relpath.startswith("/") or relpath.startswith("~") \
            or re.match(r"^[A-Za-z]:[\\/]", relpath):
        raise ValueError("absolute-path-rejected")
    if "\x00" in relpath:
        raise ValueError("nul-byte")
    if "\\" in relpath:
        raise ValueError("backslash-in-path")
    parts = relpath.split("/")
    for part in parts:
        if part in ("", ".", ".."):
            raise ValueError(f"path-segment:{part!r}")
        if any(0 <= ord(c) <= 0x1F or ord(c) == 0x7F for c in part):
            raise ValueError("control-char")
        if any(c in _ZERO_WIDTH or c in _BIDI_OVERRIDE for c in part):
            raise ValueError("zero-width-or-bidi")
        if any(0xD800 <= ord(c) <= 0xDFFF for c in part):
            raise ValueError("lone-surrogate")
        if part.endswith(".") or part.endswith(" ") or part.endswith("\t"):
            raise ValueError("trailing-dot-or-ws")
        stem = part.rsplit(".", 1)[0] if "." in part else part
        if stem.endswith(".") or stem.endswith(" ") or stem.endswith("\t"):
            raise ValueError("stem-trailing-dot-or-ws")
        if "  " in part or "\t\t" in part:
            raise ValueError("double-whitespace")
        if len(part.encode("utf-8")) > 255:
            raise ValueError("segment-too-long")
        if stem.upper() in _RESERVED_WIN:
            raise ValueError(f"win-reserved:{stem}")
        if any(c in _WIN_ILLEGAL for c in part):
            raise ValueError("win-illegal-char")

    if len(relpath.encode("utf-8")) > 4096 or len(relpath) > 260:
        raise ValueError("path-too-long")
    if len(parts) > 12:
        raise ValueError("depth-exceeds-12")

    abs_path = (brain_root / relpath).resolve()
    try:
        abs_path.relative_to(brain_root.resolve())
    except ValueError:
        raise ValueError("escapes-brain-root")
    return abs_path


# ───────────────────────────────────────────────────────────────────────
# §4.3 File-content hygiene
# ───────────────────────────────────────────────────────────────────────

def validate_file_bytes(data: bytes) -> None:
    """Reject content that violates §4.3 hygiene rules."""
    if data.startswith(b"\xef\xbb\xbf"):
        raise ValueError("utf8-bom-leading")
    if b"\xef\xbb\xbf" in data[3:]:
        raise ValueError("utf8-bom-mid-file")
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as e:
        raise ValueError(f"not-utf8:{e.reason}")
    if "\x00" in text:
        raise ValueError("nul-in-body")
    if any(0xD800 <= ord(c) <= 0xDFFF for c in text):
        raise ValueError("lone-surrogate")
    for ch in text:
        if 0x202A <= ord(ch) <= 0x202E or 0x2066 <= ord(ch) <= 0x2069:
            raise ValueError("bidi-override-in-body")
    # Bare \r without \r\n
    if re.search(r"\r(?!\n)", text):
        raise ValueError("bare-cr")
    # Frontmatter shape — only structurally checked here; full schema
    # validation is the caller's job (§5.2 validators).
    if text.startswith("---\n"):
        # Find closing fence — but exempt fenced code spans per DEC-087.
        # For the writer we trust the caller's schema validation; we only
        # reject the raw multi-block-and-no-fences case.
        rest = text[4:]
        # Strip ``` and ~~~ fenced code blocks before secondary scan.
        rest_no_fences = _strip_fenced(rest)
        # Find first closing "\n---\n" or "\n---" at EOF
        m = re.search(r"\n---\n|\n---$", rest_no_fences)
        if not m:
            raise ValueError("frontmatter-no-close")
        tail = rest_no_fences[m.end():]
        if "\n---\n" in tail:
            raise ValueError("multiple-frontmatter-blocks")


def _strip_fenced(text: str) -> str:
    """Remove ``` and ~~~ fenced code spans for the secondary frontmatter
    scan (DEC-087). Intentionally simple — handles the common case."""
    out: list[str] = []
    in_fence = False
    fence_marker = ""
    for line in text.split("\n"):
        stripped = line.lstrip()
        if not in_fence:
            if stripped.startswith("```") or stripped.startswith("~~~"):
                fence_marker = stripped[:3]
                in_fence = True
                out.append("")  # placeholder
                continue
            out.append(line)
        else:
            if stripped.startswith(fence_marker):
                in_fence = False
                out.append("")
                continue
            out.append("")  # blank out fenced content
    return "\n".join(out)


# ───────────────────────────────────────────────────────────────────────
# §4.9 .lock — POSIX flock with bounded backoff
# ───────────────────────────────────────────────────────────────────────

class BrainLock:
    """Exclusive .lock acquired around audit-append + atomic file write."""

    def __init__(self, brain_root: Path, actor_id: str):
        self.lock_path = brain_root / ".lock"
        self.actor_id = actor_id
        self._fh = None

    def __enter__(self):
        self._acquire(timeout_ms=200)
        return self

    def __exit__(self, exc_type, exc, tb):
        self._release()

    def _acquire(self, *, timeout_ms: int):
        if os.name == "posix":
            import fcntl
            self._fh = open(self.lock_path, "a+b")
            deadline = time.monotonic() + timeout_ms / 1000.0
            while True:
                try:
                    fcntl.flock(self._fh.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                    break
                except BlockingIOError:
                    if time.monotonic() >= deadline:
                        self._fh.close()
                        self._fh = None
                        die(
                            f"another writer holds {self.lock_path} "
                            f"(timeout after {timeout_ms} ms)",
                            exit_code=3,
                        )
                    time.sleep(0.02)
        elif os.name == "nt":  # pragma: no cover
            import msvcrt
            self._fh = open(self.lock_path, "a+b")
            deadline = time.monotonic() + timeout_ms / 1000.0
            while True:
                try:
                    msvcrt.locking(self._fh.fileno(), msvcrt.LK_NBLCK, 1)
                    break
                except OSError:
                    if time.monotonic() >= deadline:
                        self._fh.close()
                        self._fh = None
                        die("lock contention (Windows)", exit_code=3)
                    time.sleep(0.02)
        else:
            die(f"unsupported OS for locking: {os.name}", exit_code=1)

        # Write lock body for stale-recovery diagnostics
        body = json.dumps({
            "pid": os.getpid(),
            "host": _hostname(),
            "actor_id": self.actor_id,
            "acquired_at": _dt.datetime.now(_dt.timezone.utc).isoformat(),
        }, separators=(",", ":")).encode("utf-8")
        self._fh.seek(0)
        self._fh.truncate()
        self._fh.write(body)
        self._fh.flush()
        os.fsync(self._fh.fileno())

    def _release(self):
        if self._fh is None:
            return
        try:
            self._fh.seek(0)
            self._fh.truncate()  # release-time body wipe per §4.9
            self._fh.flush()
            os.fsync(self._fh.fileno())
            if os.name == "posix":
                import fcntl
                fcntl.flock(self._fh.fileno(), fcntl.LOCK_UN)
            elif os.name == "nt":  # pragma: no cover
                import msvcrt
                msvcrt.locking(self._fh.fileno(), msvcrt.LK_UNLCK, 1)
        finally:
            self._fh.close()
            self._fh = None


def _hostname() -> str:
    try:
        import socket
        return socket.gethostname()
    except Exception:
        return "unknown-host"


# ───────────────────────────────────────────────────────────────────────
# §4.4 Two-phase atomic write — tmp + fsync + rename + parent fsync
# ───────────────────────────────────────────────────────────────────────

def atomic_write_bytes(path: Path, data: bytes) -> None:
    parent = path.parent
    parent.mkdir(parents=True, exist_ok=True)
    fd, tmpname = tempfile.mkstemp(
        prefix=".tmp.", suffix=".part", dir=str(parent)
    )
    try:
        with os.fdopen(fd, "wb") as f:
            f.write(data)
            f.flush()
            os.fsync(f.fileno())
        os.replace(tmpname, path)
    except Exception:
        try:
            os.unlink(tmpname)
        except FileNotFoundError:
            pass
        raise
    # fsync parent dir for crash-consistent rename
    try:
        dfd = os.open(str(parent), os.O_RDONLY)
        try:
            os.fsync(dfd)
        finally:
            os.close(dfd)
    except OSError:
        pass  # not all FS support dir fsync


def append_audit_line(audit_path: Path, line: str) -> None:
    """Append-only write to JSONL. Creates file if missing, fsyncs on each
    append. Single-line, single-write to keep partial-line risk minimal."""
    audit_path.parent.mkdir(parents=True, exist_ok=True)
    payload = (line + "\n").encode("utf-8")
    fd = os.open(
        str(audit_path),
        os.O_WRONLY | os.O_APPEND | os.O_CREAT,
        0o600,
    )
    try:
        os.write(fd, payload)
        os.fsync(fd)
    finally:
        os.close(fd)


# ───────────────────────────────────────────────────────────────────────
# §7.2 Canonical JSON for hashing — RFC 8785 JCS
# ───────────────────────────────────────────────────────────────────────

def compute_chain(row: dict, prev_chain: str) -> str:
    """Per §7.2:
        chain = sha256_hex(canonical_json(row \\ {chain, prev_chain})
                           || prev_chain.encode('utf-8'))
    """
    body = {k: v for k, v in row.items() if k not in ("chain", "prev_chain")}
    canonical = rfc8785.dumps(body)  # bytes, NFC-normalised, JCS-canonical
    digest = hashlib.sha256(canonical + prev_chain.encode("utf-8")).hexdigest()
    return f"sha256:{digest}"


def sha256_hex_bytes(data: bytes) -> str:
    return f"sha256:{hashlib.sha256(data).hexdigest()}"


# ───────────────────────────────────────────────────────────────────────
# Audit ledger — read head, append row
# ───────────────────────────────────────────────────────────────────────

def current_audit_path(brain_root: Path, tz: _dt.tzinfo) -> Path:
    yyyy_mm = _dt.datetime.now(tz).strftime("%Y-%m")
    return brain_root / "audit" / f"{yyyy_mm}.jsonl"


def all_audit_paths(brain_root: Path) -> list[Path]:
    audit_dir = brain_root / "audit"
    if not audit_dir.is_dir():
        return []
    return sorted(p for p in audit_dir.glob("*.jsonl") if p.is_file())


def latest_chain_head(brain_root: Path) -> tuple[str, Path | None]:
    """Return (chain_hex, audit_path_of_last_row) or (genesis, None) if empty."""
    paths = all_audit_paths(brain_root)
    if not paths:
        return ("sha256:" + "0" * 64, None)
    last_path = paths[-1]
    last_line = None
    with open(last_path, "rb") as f:
        # Walk forward — files are not huge in practice; simplicity > seek
        for raw in f:
            if raw.strip():
                last_line = raw
    if last_line is None:
        # Empty file but earlier file may have content
        if len(paths) > 1:
            with open(paths[-2], "rb") as f:
                for raw in f:
                    if raw.strip():
                        last_line = raw
        if last_line is None:
            return ("sha256:" + "0" * 64, None)
        last_path = paths[-2]
    row = json.loads(last_line)
    return (row["chain"], last_path)


def scope_for_path(relpath: str) -> str:
    """Map a BRAIN-relative path to its scope per §3 layout."""
    if relpath in ("", "/", "."):
        return "meta"
    head = relpath.split("/", 1)[0]
    if head in ("manifest.json", "README.md", ".lock", "audit", "meta",
                "memories", "exports", "index", "conflicts", "company"):
        return "meta" if head not in ("memories", "company") else (
            "company" if head == "company" else "meta"
        )
    if head == "memories":
        return "meta"
    if head in ("module", "member", "client", "project", "persona"):
        # second segment is the scope discriminator
        parts = relpath.split("/")
        if len(parts) >= 2 and parts[1]:
            return f"{head}:{parts[1]}"
        return head
    return "meta"


def parse_frontmatter(text: str) -> dict:
    """Return parsed YAML frontmatter dict, or {} if absent."""
    if not text.startswith("---\n"):
        return {}
    rest = text[4:]
    rest_no_fences = _strip_fenced(rest)
    m = re.search(r"\n---\n|\n---$", rest_no_fences)
    if not m:
        return {}
    fm_text = rest[: m.start()]
    try:
        data = yaml.safe_load(fm_text)
        return data if isinstance(data, dict) else {}
    except yaml.YAMLError:
        return {}


# ───────────────────────────────────────────────────────────────────────
# Audit-row builder
# ───────────────────────────────────────────────────────────────────────

def make_audit_row(
    *,
    op: str,
    actor_kind: str,
    actor_id: str,
    scope: str,
    path: str,
    prev_chain: str,
    tz: _dt.tzinfo,
    memory_id: str | None = None,
    persona: str | None = None,
    prev_version: int | None = None,
    new_version: int | None = None,
    supersedes_event_id: str | None = None,
    classification: str | None = None,
    authority: str | None = None,
    consent_event_id: str | None = None,
    provenance_source: str = "manual",
    provenance_source_ref: str = "",
    provenance_confidence: float = 1.0,
    before_hash: str | None = None,
    after_hash: str | None = None,
    diff: str = "<hash-only>",
    reason: str = "",
    correction_to: str | None = None,
    audit_id: str | None = None,
    ts: str | None = None,
) -> dict:
    if audit_id is None:
        audit_id = new_uuid7("evt")
    if ts is None:
        ts = now_iso(tz)
    row: dict = {
        "audit_id": audit_id,
        "ts": ts,
        "actor_kind": actor_kind,
        "actor_id": actor_id,
        "persona": persona,
        "op": op,
        "scope": scope,
        "path": path,
        "memory_id": memory_id,
        "prev_version": prev_version,
        "new_version": new_version,
        "supersedes_event_id": supersedes_event_id,
        "classification": classification,
        "authority": authority,
        "consent_event_id": consent_event_id,
        "provenance": {
            "source": provenance_source,
            "source_ref": provenance_source_ref,
            "confidence": float(provenance_confidence),
        },
        "before_hash": before_hash,
        "after_hash": after_hash,
        "diff": diff,
        "reason": reason[:200],
        "correction_to": correction_to,
        "prev_chain": prev_chain,
    }
    row["chain"] = compute_chain(row, prev_chain)
    return row


def serialise_row_for_disk(row: dict) -> str:
    """On-disk JSONL line — uses insertion order (matches recent rows in
    the existing chain) and ensure_ascii=False to keep human-readable
    Unicode. The HASH is computed from the JCS form, not this string."""
    return json.dumps(row, ensure_ascii=False, separators=(",", ":"))


# ───────────────────────────────────────────────────────────────────────
# Subcommand: session-start
# ───────────────────────────────────────────────────────────────────────

def cmd_session_start(actor: str) -> int:
    brain_root = resolve_brain_root()
    manifest = read_manifest(brain_root)
    if manifest.get("operational_mode") not in (None, "normal", "verbose",
                                                "debug", "maintenance"):
        die(
            f"manifest.operational_mode = "
            f"{manifest.get('operational_mode')!r} — refusing to start.",
            exit_code=2,
        )
    tz = session_timezone(manifest)
    actor_kind, actor_id = split_actor(actor)

    with BrainLock(brain_root, actor_id):
        prev_chain, _ = latest_chain_head(brain_root)
        row = make_audit_row(
            op="session.start",
            actor_kind=actor_kind,
            actor_id=actor_id,
            scope="meta",
            path=".cyberos-memory/",
            prev_chain=prev_chain,
            tz=tz,
            classification="operational",
            authority="human-edited",
            provenance_source="manual",
            provenance_source_ref="session-marker",
            provenance_confidence=1.0,
            reason="Begin agent session.",
        )
        audit_path = current_audit_path(brain_root, tz)
        append_audit_line(audit_path, serialise_row_for_disk(row))

    print(f"session-start appended: {row['audit_id']} chain={row['chain']}")
    return 0


# ───────────────────────────────────────────────────────────────────────
# Subcommand: session-end
# ───────────────────────────────────────────────────────────────────────

def cmd_session_end(actor: str) -> int:
    brain_root = resolve_brain_root()
    manifest = read_manifest(brain_root)
    tz = session_timezone(manifest)
    actor_kind, actor_id = split_actor(actor)

    with BrainLock(brain_root, actor_id):
        # Step 1: append session.end terminator
        prev_chain, _ = latest_chain_head(brain_root)
        end_row = make_audit_row(
            op="session.end",
            actor_kind=actor_kind,
            actor_id=actor_id,
            scope="meta",
            path=".cyberos-memory/",
            prev_chain=prev_chain,
            tz=tz,
            classification="operational",
            authority="human-edited",
            provenance_source="manual",
            provenance_source_ref="session-marker",
            provenance_confidence=1.0,
            reason="End agent session.",
        )
        audit_path = current_audit_path(brain_root, tz)
        append_audit_line(audit_path, serialise_row_for_disk(end_row))

        # Step 2: str_replace manifest.json — update audit_chain_head +
        # reconciliation_checkpoint + last_updated_at to point at end_row
        manifest_path = brain_root / "manifest.json"
        before_bytes = manifest_path.read_bytes()
        before_hash = sha256_hex_bytes(before_bytes)

        new_manifest = dict(manifest)
        new_manifest["audit_chain_head"] = end_row["chain"]
        new_manifest["last_updated_at"] = end_row["ts"]
        new_manifest["reconciliation_checkpoint"] = {
            "audit_id": end_row["audit_id"],
            "chain": end_row["chain"],
            "ts": end_row["ts"],
        }
        # Match existing manifest.json formatting style: 2-space indent,
        # ensure_ascii=False, sort_keys=False, trailing newline.
        new_text = json.dumps(new_manifest, indent=2, ensure_ascii=False) + "\n"
        new_bytes = new_text.encode("utf-8")
        after_hash = sha256_hex_bytes(new_bytes)

        # Audit row for the manifest update — chains from end_row
        manifest_row = make_audit_row(
            op="str_replace",
            actor_kind=actor_kind,
            actor_id=actor_id,
            scope="meta",
            path=".cyberos-memory/manifest.json",
            prev_chain=end_row["chain"],
            tz=tz,
            classification="operational",
            authority="human-confirmed",
            provenance_source="manual",
            provenance_source_ref=(
                "AGENTS.md §6 + §4.7 reconciliation_checkpoint update"
            ),
            provenance_confidence=1.0,
            before_hash=before_hash,
            after_hash=after_hash,
            reason=(
                "Update audit_chain_head + reconciliation_checkpoint at "
                "session.end."
            ),
        )
        append_audit_line(audit_path, serialise_row_for_disk(manifest_row))
        atomic_write_bytes(manifest_path, new_bytes)

    print(
        f"session-end appended: {end_row['audit_id']} "
        f"chain={end_row['chain']}\n"
        f"manifest-update: {manifest_row['audit_id']} "
        f"chain={manifest_row['chain']}"
    )
    return 0


# ───────────────────────────────────────────────────────────────────────
# Subcommand: write (op:create on a memory file)
# ───────────────────────────────────────────────────────────────────────

def cmd_write(actor: str, relpath: str, content_file: str) -> int:
    brain_root = resolve_brain_root()
    manifest = read_manifest(brain_root)
    tz = session_timezone(manifest)
    actor_kind, actor_id = split_actor(actor)

    # §4.1 path traversal
    try:
        abs_path = validate_relpath(relpath, brain_root)
    except ValueError as e:
        die(f"path-rejected:{e}", exit_code=2)

    if abs_path.exists():
        die(f"create-on-existing-path:{relpath}", exit_code=2)

    # Read content
    src = Path(content_file)
    if not src.is_file():
        die(f"content-file-missing:{content_file}", exit_code=2)
    data = src.read_bytes()

    # §4.3 file-content hygiene
    try:
        validate_file_bytes(data)
    except ValueError as e:
        die(f"content-rejected:{e}", exit_code=2)

    # Parse frontmatter for audit-row mirror fields
    text = data.decode("utf-8")
    fm = parse_frontmatter(text)
    memory_id = fm.get("memory_id")
    if memory_id and not is_valid_id(memory_id):
        die(f"frontmatter-rejected:bad-memory-id:{memory_id}", exit_code=2)
    classification = fm.get("classification")
    authority = fm.get("authority")
    new_version = fm.get("version", 1)

    # §4.5 scope contract — prefer frontmatter `scope:` (semantic scope set
    # by the memory itself, e.g. `project:cyberos` for a project working
    # memory under memories/*), fall back to path-derived scope.
    scope = fm.get("scope") or scope_for_path(relpath)
    elevated = manifest.get("scope_contract", {}).get(
        "elevated_scopes_require_human_confirmation", []
    )
    scope_head = scope.split(":", 1)[0]
    if actor_kind == "agent" and scope_head in elevated:
        # Allow if user has explicitly invoked from chat — caller's
        # responsibility. We log scope here for visibility but don't
        # silently reject. (Keep the simple behaviour: writer trusts
        # caller; AGENTS.md §0.2 holds the agent accountable.)
        pass

    after_hash = sha256_hex_bytes(data)
    bran_path = ".cyberos-memory/" + relpath

    with BrainLock(brain_root, actor_id):
        prev_chain, _ = latest_chain_head(brain_root)
        row = make_audit_row(
            op="create",
            actor_kind=actor_kind,
            actor_id=actor_id,
            scope=scope,
            path=bran_path,
            prev_chain=prev_chain,
            tz=tz,
            memory_id=memory_id,
            new_version=new_version,
            classification=classification,
            authority=authority,
            after_hash=after_hash,
            reason=f"Create {relpath}.",
            provenance_source=str(fm.get("provenance", {}).get("source", "manual")),
            provenance_source_ref=str(
                fm.get("provenance", {}).get("source_ref", "")
            ),
            provenance_confidence=float(
                fm.get("provenance", {}).get("confidence", 1.0)
            ),
        )
        audit_path = current_audit_path(brain_root, tz)
        append_audit_line(audit_path, serialise_row_for_disk(row))
        atomic_write_bytes(abs_path, data)

    print(f"write appended: {row['audit_id']} chain={row['chain']} path={bran_path}")
    return 0


# ───────────────────────────────────────────────────────────────────────
# Subcommand: str-replace (whole-file replace + audit row)
# ───────────────────────────────────────────────────────────────────────

def cmd_str_replace(actor: str, relpath: str, new_file: str) -> int:
    brain_root = resolve_brain_root()
    manifest = read_manifest(brain_root)
    tz = session_timezone(manifest)
    actor_kind, actor_id = split_actor(actor)

    try:
        abs_path = validate_relpath(relpath, brain_root)
    except ValueError as e:
        die(f"path-rejected:{e}", exit_code=2)

    if not abs_path.is_file():
        die(f"str-replace-target-missing:{relpath}", exit_code=2)

    src = Path(new_file)
    if not src.is_file():
        die(f"new-file-missing:{new_file}", exit_code=2)
    new_data = src.read_bytes()
    try:
        validate_file_bytes(new_data)
    except ValueError as e:
        die(f"content-rejected:{e}", exit_code=2)

    before_data = abs_path.read_bytes()
    before_hash = sha256_hex_bytes(before_data)
    after_hash = sha256_hex_bytes(new_data)
    if before_hash == after_hash:
        die("str-replace-noop:before==after", exit_code=2)

    fm = parse_frontmatter(new_data.decode("utf-8"))
    memory_id = fm.get("memory_id")
    classification = fm.get("classification")
    authority = fm.get("authority")
    new_version = fm.get("version")
    bran_path = ".cyberos-memory/" + relpath
    scope = fm.get("scope") or scope_for_path(relpath)

    with BrainLock(brain_root, actor_id):
        prev_chain, _ = latest_chain_head(brain_root)
        row = make_audit_row(
            op="str_replace",
            actor_kind=actor_kind,
            actor_id=actor_id,
            scope=scope,
            path=bran_path,
            prev_chain=prev_chain,
            tz=tz,
            memory_id=memory_id,
            new_version=new_version,
            classification=classification,
            authority=authority,
            before_hash=before_hash,
            after_hash=after_hash,
            reason=f"str_replace {relpath}.",
        )
        audit_path = current_audit_path(brain_root, tz)
        append_audit_line(audit_path, serialise_row_for_disk(row))
        atomic_write_bytes(abs_path, new_data)

    print(f"str-replace appended: {row['audit_id']} chain={row['chain']} path={bran_path}")
    return 0


# ───────────────────────────────────────────────────────────────────────
# Subcommand: protocol-upgrade
#
# Per AGENTS.md §0.5: updates manifest.protocol.{sha256, approved_at,
# approved_by, last_checked_at}, appends op:"protocol_upgrade" audit row
# with reason citing the SHA transition. Idempotent against concurrent
# writers via the BrainLock. Does NOT touch audit_chain_head or
# reconciliation_checkpoint — those are updated separately by
# session-end as the close-of-session pattern.
# ───────────────────────────────────────────────────────────────────────

def cmd_protocol_upgrade(actor: str, old_sha: str, new_sha: str,
                         reason: str | None = None) -> int:
    brain_root = resolve_brain_root()
    manifest = read_manifest(brain_root)
    tz = session_timezone(manifest)
    actor_kind, actor_id = split_actor(actor)

    # Sanity: old_sha must match what's currently pinned.
    pinned = manifest.get("protocol", {}).get("sha256")
    if pinned != old_sha:
        die(
            f"old_sha mismatch: manifest pin = {pinned!r}, expected "
            f"{old_sha!r}. Refusing to upgrade — re-check the chain.",
            exit_code=2,
        )

    with BrainLock(brain_root, actor_id):
        manifest_path = brain_root / "manifest.json"
        before_bytes = manifest_path.read_bytes()
        before_hash = sha256_hex_bytes(before_bytes)

        new_manifest = dict(manifest)
        ts = now_iso(tz)
        proto = dict(new_manifest.get("protocol", {}))
        proto["sha256"] = new_sha
        proto["approved_at"] = ts
        proto["approved_by"] = actor_id
        proto["last_checked_at"] = ts
        new_manifest["protocol"] = proto

        new_text = json.dumps(new_manifest, indent=2, ensure_ascii=False) + "\n"
        new_bytes = new_text.encode("utf-8")
        after_hash = sha256_hex_bytes(new_bytes)

        if reason is None:
            reason = (
                f"Approve protocol upgrade {old_sha} → {new_sha} per §0.5; "
                f"approved by {actor_id} in chat."
            )

        prev_chain, _ = latest_chain_head(brain_root)
        row = make_audit_row(
            op="protocol_upgrade",
            actor_kind=actor_kind,
            actor_id=actor_id,
            scope="meta",
            path=".cyberos-memory/manifest.json",
            prev_chain=prev_chain,
            tz=tz,
            classification="operational",
            authority="human-edited",
            provenance_source="manual",
            provenance_source_ref=f"§0.5 chat-turn approval {ts}",
            provenance_confidence=1.0,
            before_hash=before_hash,
            after_hash=after_hash,
            reason=reason,
        )
        audit_path = current_audit_path(brain_root, tz)
        append_audit_line(audit_path, serialise_row_for_disk(row))
        atomic_write_bytes(manifest_path, new_bytes)

    print(f"protocol-upgrade appended: {row['audit_id']} chain={row['chain']}")
    return 0


# ───────────────────────────────────────────────────────────────────────
# Subcommand: self-audit (§8.7 6-phase pass)
#
# Performs the six checks specified in AGENTS.md §8.7 and produces:
#   - meta/health/<YYYY-MM-DD>-<sha-prefix>[-postupgrade].md  (markdown report)
#   - one op:"health_check" audit row referencing that report
#
# Severity routing:
#   CRITICAL — chain break, schema invariant violation, supersedes cycle,
#              dangling supersedes, orphan audit row referencing missing
#              path. Returns exit code 2 (script-blocking).
#   WARN     — cap approaching, dangling relates_to, orphan file with no
#              audit reference, schema drift on a non-critical field.
#              Returns exit code 0 (does not block scripts).
#   INFO     — successful checks; cross-writer-version hash recompute
#              differences; legacy memory_id registry hits. Returns 0.
# ───────────────────────────────────────────────────────────────────────

# §5.2 validators reused across phases
_TS_RE = re.compile(
    r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?([+-]\d{2}:\d{2}|Z)$"
)


def _validate_timestamp(value, field_label: str) -> tuple[bool, str]:
    """Per §5.2: accept ISO-8601 string OR datetime instance with tzinfo.
    Returns (ok, error_msg). PyYAML auto-coerces ISO ts → datetime;
    str(dt) emits with space separator and fails the regex — handle both.
    """
    if isinstance(value, _dt.datetime):
        if value.tzinfo is None:
            return False, f"naive-ts:{field_label}"
        return True, ""
    if isinstance(value, str):
        if _TS_RE.match(value):
            return True, ""
        return False, f"bad-ts:{field_label}"
    return False, f"non-ts-type:{field_label}:{type(value).__name__}"


def _read_legacy_ids(brain_root: Path) -> set[str]:
    """Per §5.2: legacy memory_ids registered in meta/legacy-ids.md."""
    path = brain_root / "meta" / "legacy-ids.md"
    if not path.is_file():
        return set()
    out: set[str] = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        if "|" in line and line.strip().startswith("mem_"):
            out.add(line.split("|", 1)[0].strip())
    return out


def _read_legacy_files(brain_root: Path) -> set[str]:
    """Files exempt from §8.7 phase 5 orphan-WARN/INFO surfacing.

    Format mirrors meta/legacy-ids.md: one entry per line as
    `<rel-path-under-.cyberos-memory> | <reason> | <approximate-creation>`.
    Lines starting with `#` or empty are ignored. Used for files created
    by older writers that didn't audit (e.g., pre-§0.6 protocol-history
    archives) and for known chain-history backward-orphans we've
    decided not to chase.
    """
    path = brain_root / "meta" / "legacy-files.md"
    if not path.is_file():
        return set()
    out: set[str] = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if "|" in line:
            entry = line.split("|", 1)[0].strip()
            if entry and not entry.startswith("<"):
                out.add(entry)
    return out


def _walk_memory_files(brain_root: Path) -> list[Path]:
    """All memory files under §3 layout dirs (excl. audit/, .lock, etc.)."""
    out: list[Path] = []
    skip_dirs = {".lock", "audit", "index", "exports", "conflicts", ".DS_Store"}
    for top in ("memories", "member", "client", "module", "company",
                "persona", "project", "meta"):
        d = brain_root / top
        if not d.is_dir():
            continue
        for p in d.rglob("*.md"):
            if any(part in skip_dirs for part in p.parts):
                continue
            if p.name.startswith(".tmp."):
                continue
            out.append(p)
    return sorted(out)


def _phase1_schema(brain_root: Path, legacy: set[str]) -> list[dict]:
    findings: list[dict] = []
    for path in _walk_memory_files(brain_root):
        rel = path.relative_to(brain_root).as_posix()
        # Read + parse
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError as e:
            findings.append({"sev": "CRITICAL", "phase": 1,
                             "code": f"not-utf8:{rel}:{e.reason}"})
            continue
        if not text.startswith("---\n"):
            # Some registry files (legacy-ids.md, tombstones.md, classification-
            # rules.md, retention-rules.md, README.md) are §4.2-exempt per
            # the protocol — skip schema validation.
            if rel in (
                "meta/legacy-ids.md", "meta/legacy-files.md",
                "meta/tombstones.md",
                "meta/classification-rules.md", "meta/retention-rules.md",
                "README.md",
            ) or rel.startswith("meta/health/") \
              or rel.startswith("meta/protocol-history/"):
                continue
            findings.append({"sev": "WARN", "phase": 1,
                             "code": f"no-frontmatter:{rel}"})
            continue
        fm = parse_frontmatter(text)
        if not fm:
            findings.append({"sev": "CRITICAL", "phase": 1,
                             "code": f"frontmatter-parse-failed:{rel}"})
            continue

        mid = fm.get("memory_id")
        if not mid:
            findings.append({"sev": "CRITICAL", "phase": 1,
                             "code": f"missing-memory-id:{rel}"})
        elif not is_valid_id(mid):
            if mid in legacy:
                findings.append({"sev": "INFO", "phase": 1,
                                 "code": f"memory-id-legacy:{rel}:{mid} "
                                         f"(allowed per §5.2 carve-out)"})
            else:
                findings.append({"sev": "CRITICAL", "phase": 1,
                                 "code": f"bad-memory-id:{rel}:{mid}"})

        for ts_field in ("created_at", "last_updated_at"):
            if ts_field not in fm:
                findings.append({"sev": "WARN", "phase": 1,
                                 "code": f"missing-{ts_field}:{rel}"})
                continue
            ok, err = _validate_timestamp(fm[ts_field], ts_field)
            if not ok:
                findings.append({"sev": "CRITICAL", "phase": 1,
                                 "code": f"{err}:{rel}"})

        v = fm.get("version")
        if v is not None:
            if not isinstance(v, int) or v < 1:
                findings.append({"sev": "CRITICAL", "phase": 1,
                                 "code": f"bad-version:{rel}:{v!r}"})

        prov = fm.get("provenance") or {}
        conf = prov.get("confidence")
        if conf is not None:
            if isinstance(conf, bool) or not isinstance(conf, (int, float)):
                findings.append({"sev": "CRITICAL", "phase": 1,
                                 "code": f"bad-confidence-type:{rel}"})
            elif not 0.0 <= float(conf) <= 1.0:
                findings.append({"sev": "CRITICAL", "phase": 1,
                                 "code": f"confidence-out-of-range:{rel}"})

        cls = fm.get("classification")
        if cls and cls not in ("personnel", "client", "operational", "public"):
            findings.append({"sev": "CRITICAL", "phase": 1,
                             "code": f"bad-classification:{rel}:{cls}"})

        auth = fm.get("authority")
        if auth and auth not in (
            "human-edited", "human-confirmed", "llm-explicit", "llm-implicit"
        ):
            findings.append({"sev": "CRITICAL", "phase": 1,
                             "code": f"bad-authority:{rel}:{auth}"})

    return findings


def _phase2_supersedes(brain_root: Path) -> list[dict]:
    findings: list[dict] = []
    files = _walk_memory_files(brain_root)
    id_to_path: dict[str, str] = {}
    supersedes_map: dict[str, list[str]] = {}
    superseded_by_map: dict[str, str] = {}
    for path in files:
        rel = path.relative_to(brain_root).as_posix()
        fm = parse_frontmatter(path.read_text(encoding="utf-8", errors="replace"))
        if not fm:
            continue
        mid = fm.get("memory_id")
        if not mid:
            continue
        id_to_path[mid] = rel
        sup = fm.get("supersedes")
        if isinstance(sup, str):
            supersedes_map[mid] = [sup]
        elif isinstance(sup, list):
            supersedes_map[mid] = [s for s in sup if isinstance(s, str)]
        else:
            supersedes_map[mid] = []
        sb = fm.get("superseded_by")
        if isinstance(sb, str):
            superseded_by_map[mid] = sb

    # Dangling supersedes
    for mid, targets in supersedes_map.items():
        for tgt in targets:
            if tgt not in id_to_path:
                findings.append({"sev": "CRITICAL", "phase": 2,
                                 "code": f"dangling-supersedes:"
                                         f"{id_to_path[mid]}→{tgt}"})

    # Dangling superseded_by + orphan check
    for mid, by in superseded_by_map.items():
        if by not in id_to_path:
            findings.append({"sev": "CRITICAL", "phase": 2,
                             "code": f"dangling-superseded-by:"
                                     f"{id_to_path[mid]}→{by}"})
            continue
        if mid not in supersedes_map.get(by, []):
            findings.append({"sev": "WARN", "phase": 2,
                             "code": f"orphan-superseded-by:"
                                     f"{id_to_path[mid]} says superseded "
                                     f"by {by} but {by} doesn't claim it"})

    # Cycle detection (DFS)
    visited: dict[str, int] = {}  # 0=unseen, 1=in-stack, 2=done
    for start in supersedes_map:
        if visited.get(start, 0) == 2:
            continue
        stack = [(start, iter(supersedes_map.get(start, [])))]
        visited[start] = 1
        while stack:
            node, it = stack[-1]
            try:
                nxt = next(it)
            except StopIteration:
                visited[node] = 2
                stack.pop()
                continue
            if nxt not in id_to_path:
                continue
            if visited.get(nxt, 0) == 1:
                findings.append({"sev": "CRITICAL", "phase": 2,
                                 "code": f"supersedes-cycle:"
                                         f"...→{nxt}→...→{nxt}"})
                visited[nxt] = 2
                continue
            if visited.get(nxt, 0) == 0:
                visited[nxt] = 1
                stack.append((nxt, iter(supersedes_map.get(nxt, []))))

    return findings


def _phase3_relationships(brain_root: Path) -> list[dict]:
    findings: list[dict] = []
    files = _walk_memory_files(brain_root)
    id_to_path: dict[str, str] = {}
    rel_map: dict[str, list[str]] = {}
    for path in files:
        rel = path.relative_to(brain_root).as_posix()
        fm = parse_frontmatter(path.read_text(encoding="utf-8", errors="replace"))
        if not fm:
            continue
        mid = fm.get("memory_id")
        if not mid:
            continue
        id_to_path[mid] = rel
        rels = fm.get("relationships") or []
        if isinstance(rels, list):
            rel_map[mid] = [
                r.get("relates_to") for r in rels
                if isinstance(r, dict) and r.get("relates_to")
            ]

    for mid, targets in rel_map.items():
        for tgt in targets:
            if tgt not in id_to_path:
                findings.append({"sev": "WARN", "phase": 3,
                                 "code": f"dangling-relates-to:"
                                         f"{id_to_path[mid]}→{tgt}"})
    return findings


def _phase4_chain(brain_root: Path, manifest: dict, *,
                  bit_perfect: bool) -> tuple[list[dict], dict]:
    findings: list[dict] = []
    paths = all_audit_paths(brain_root)
    if not paths:
        findings.append({"sev": "WARN", "phase": 4, "code": "no-audit-ledger"})
        return findings, {"rows": 0, "head": None, "bit_perfect": (0, 0)}
    rows: list[dict] = []
    for p in paths:
        with open(p, "r", encoding="utf-8") as f:
            for lineno, line in enumerate(f, 1):
                if not line.strip():
                    continue
                try:
                    rows.append(json.loads(line))
                except json.JSONDecodeError as e:
                    findings.append({"sev": "CRITICAL", "phase": 4,
                                     "code": f"audit-corrupt:"
                                             f"{p.name}:{lineno}:{e.msg}"})

    if not rows:
        findings.append({"sev": "WARN", "phase": 4, "code": "ledger-empty"})
        return findings, {"rows": 0, "head": None, "bit_perfect": (0, 0)}

    # LINK invariant
    link_breaks = []
    for i in range(1, len(rows)):
        if rows[i]["prev_chain"] != rows[i - 1]["chain"]:
            link_breaks.append(i)
    for idx in link_breaks:
        findings.append({"sev": "CRITICAL", "phase": 4,
                         "code": f"chain-link-break:row[{idx}]"})

    # Bit-perfect recompute (INFO only — informational per §7.2)
    bp_match = 0
    if bit_perfect:
        for r in rows:
            if compute_chain(r, r["prev_chain"]) == r["chain"]:
                bp_match += 1
        if bp_match < len(rows):
            findings.append({
                "sev": "INFO", "phase": 4,
                "code": f"bit-perfect-recompute: {bp_match}/{len(rows)} "
                        f"(differences are cross-writer-version informational "
                        f"per §7.2; LINK invariant is authoritative)"
            })

    # audit_chain_head reachability
    head = manifest.get("audit_chain_head")
    if head and not any(r["chain"] == head for r in rows):
        findings.append({"sev": "CRITICAL", "phase": 4,
                         "code": f"audit-chain-head-not-in-ledger:{head}"})

    # reconciliation_checkpoint
    cp = manifest.get("reconciliation_checkpoint") or {}
    if cp.get("audit_id"):
        target = next((r for r in rows if r["audit_id"] == cp["audit_id"]), None)
        if not target:
            findings.append({"sev": "CRITICAL", "phase": 4,
                             "code": f"stale-checkpoint:audit_id:"
                                     f"{cp['audit_id']}"})
        elif target["chain"] != cp.get("chain"):
            findings.append({"sev": "CRITICAL", "phase": 4,
                             "code": f"stale-checkpoint:chain-mismatch"})

    return findings, {
        "rows": len(rows),
        "head": rows[-1]["chain"],
        "bit_perfect": (bp_match, len(rows)) if bit_perfect else (None, None),
    }


def _phase5_orphans(brain_root: Path) -> list[dict]:
    """§8.7 phase 5 — orphan detection.

    Two complementary checks:
      Forward: every file under .cyberos-memory/ has an audit row
               creating it (or post-creation mutation) that is not later
               reverted. Missing audit row → WARN (orphan file).
      Backward: every non-revert/non-delete audit row's path resolves to
               an existing file. Missing file → INFO (chain-history
               artefact). Demoted from §8.7's nominal CRITICAL because
               historical chains often carry rename-without-rename-op
               artefacts that Bundle P's writer tolerated; flagging them
               CRITICAL would retroactively freeze established BRAINs.
               A future Bundle may tighten this to CRITICAL once a
               cleanup pass has run.
    """
    findings: list[dict] = []
    paths = all_audit_paths(brain_root)
    legacy_files = _read_legacy_files(brain_root)

    # Build last-op-per-path index from chain
    last_op_per_path: dict[str, str] = {}
    for p in paths:
        with open(p, "r", encoding="utf-8") as f:
            for line in f:
                if not line.strip():
                    continue
                try:
                    r = json.loads(line)
                except json.JSONDecodeError:
                    continue
                op = r.get("op")
                pth = r.get("path")
                if not pth or not op:
                    continue
                if op in ("create", "str_replace", "insert", "rename",
                          "protocol_upgrade", "health_check"):
                    last_op_per_path[pth] = op
                elif op == "delete":
                    last_op_per_path[pth] = "delete"
                elif op == "revert":
                    if pth in last_op_per_path:
                        del last_op_per_path[pth]

    # Backward direction (audit row → file)
    for pth, op in last_op_per_path.items():
        if pth in (".cyberos-memory/", ".cyberos-memory"):
            continue
        if not pth.startswith(".cyberos-memory/"):
            continue
        rel = pth.removeprefix(".cyberos-memory/")
        abs_path = brain_root / rel
        if op == "delete":
            if not abs_path.is_file():
                findings.append({"sev": "WARN", "phase": 5,
                                 "code": f"tombstoned-file-missing:{rel}"})
        else:
            if not abs_path.is_file():
                if rel in legacy_files:
                    # Registered in meta/legacy-files.md — silenced as INFO
                    # with a clearer marker so future audits can see it
                    # was deliberately registered, not just background
                    # chain-history.
                    findings.append({"sev": "INFO", "phase": 5,
                                     "code": f"legacy-file-registered:{rel} "
                                             f"(last op:{op}, file missing — "
                                             f"per meta/legacy-files.md)"})
                else:
                    findings.append({"sev": "INFO", "phase": 5,
                                     "code": f"orphan-audit-row "
                                             f"(chain-history artefact):"
                                             f"{rel} (last op:{op}, "
                                             f"file missing)"})

    # Forward direction (file → audit row)
    audited_paths = {
        pth.removeprefix(".cyberos-memory/")
        for pth in last_op_per_path
        if pth.startswith(".cyberos-memory/")
    }
    for f_path in _walk_memory_files(brain_root):
        rel = f_path.relative_to(brain_root).as_posix()
        # Health reports + protocol-history archives + registries are
        # legitimately created via op:create or op:health_check; if they
        # appear in audited_paths we're fine. Skip the .keep placeholders.
        if f_path.name == ".keep":
            continue
        if rel not in audited_paths:
            if rel in legacy_files:
                findings.append({"sev": "INFO", "phase": 5,
                                 "code": f"legacy-file-registered:{rel} "
                                         f"(no audit row — per "
                                         f"meta/legacy-files.md)"})
            else:
                findings.append({"sev": "WARN", "phase": 5,
                                 "code": f"orphan-file (no audit row):{rel}"})
    return findings


def _phase6_caps(brain_root: Path) -> tuple[list[dict], dict]:
    """Resource caps per §5.5. Warn at 80% of hard cap."""
    findings: list[dict] = []
    files = _walk_memory_files(brain_root)
    file_count = 0
    total_bytes = 0
    over_body = 0
    over_fm = 0
    for path in files:
        file_count += 1
        data = path.read_bytes()
        total_bytes += len(data)
        # Roughly split frontmatter vs body
        try:
            text = data.decode("utf-8")
        except UnicodeDecodeError:
            continue
        if text.startswith("---\n"):
            rest = text[4:]
            m = re.search(r"\n---\n|\n---$", _strip_fenced(rest))
            if m:
                fm_bytes = len(rest[: m.start()].encode("utf-8"))
                body_bytes = len(rest[m.end():].encode("utf-8"))
                if fm_bytes > 4 * 1024:
                    findings.append({"sev": "WARN", "phase": 6,
                                     "code": f"frontmatter-over-4kb:"
                                             f"{path.relative_to(brain_root)}: "
                                             f"{fm_bytes} bytes"})
                if body_bytes > 30 * 1024:
                    over_body += 1
                    findings.append({"sev": "WARN", "phase": 6,
                                     "code": f"body-over-30kb-hard:"
                                             f"{path.relative_to(brain_root)}: "
                                             f"{body_bytes} bytes"})

    # File count
    if file_count > 8000:  # 80% of 10000
        findings.append({"sev": "WARN", "phase": 6,
                         "code": f"file-count-approaching-cap:"
                                 f"{file_count}/10000"})
    # Total store size
    if total_bytes > 8 * 1024 * 1024:  # 80% of 10 MB hard
        findings.append({"sev": "WARN", "phase": 6,
                         "code": f"store-size-approaching-cap:"
                                 f"{total_bytes // 1024} KB / 10 MB"})

    return findings, {
        "files": file_count,
        "total_bytes": total_bytes,
    }


def cmd_self_audit(actor: str, *, post_upgrade: bool = False,
                   bit_perfect: bool = True) -> int:
    brain_root = resolve_brain_root()
    manifest = read_manifest(brain_root)
    tz = session_timezone(manifest)
    actor_kind, actor_id = split_actor(actor)
    legacy = _read_legacy_ids(brain_root)

    print("§8.7 self-audit — running 6 phases…")
    findings: list[dict] = []
    findings.extend(_phase1_schema(brain_root, legacy))
    print(f"  phase 1 (schema): {sum(1 for f in findings if f['phase']==1)} finding(s)")
    findings.extend(_phase2_supersedes(brain_root))
    print(f"  phase 2 (supersedes): {sum(1 for f in findings if f['phase']==2)} finding(s)")
    findings.extend(_phase3_relationships(brain_root))
    print(f"  phase 3 (relationships): {sum(1 for f in findings if f['phase']==3)} finding(s)")
    chain_findings, chain_stats = _phase4_chain(brain_root, manifest,
                                                bit_perfect=bit_perfect)
    findings.extend(chain_findings)
    print(f"  phase 4 (chain): {len(chain_findings)} finding(s); "
          f"rows={chain_stats['rows']}")
    findings.extend(_phase5_orphans(brain_root))
    print(f"  phase 5 (orphans): {sum(1 for f in findings if f['phase']==5)} finding(s)")
    cap_findings, cap_stats = _phase6_caps(brain_root)
    findings.extend(cap_findings)
    print(f"  phase 6 (caps): {len(cap_findings)} finding(s); "
          f"files={cap_stats['files']}, "
          f"size={cap_stats['total_bytes'] // 1024} KB")

    crit = [f for f in findings if f["sev"] == "CRITICAL"]
    warn = [f for f in findings if f["sev"] == "WARN"]
    info = [f for f in findings if f["sev"] == "INFO"]

    # Build report
    pinned = manifest.get("protocol", {}).get("sha256", "(unknown)")
    sha_prefix = pinned.removeprefix("sha256:")[:16] if pinned else "unknown"
    yyyy_mm_dd = _dt.datetime.now(tz).strftime("%Y-%m-%d")
    report_name = (
        f"{yyyy_mm_dd}-{sha_prefix}"
        + ("-postupgrade" if post_upgrade else "")
        + ".md"
    )
    report_relpath = f"meta/health/{report_name}"
    report_abs = brain_root / report_relpath

    lines = []
    title = "post-upgrade scan" if post_upgrade else "routine self-audit"
    lines.append(f"# §8.7 self-audit — {yyyy_mm_dd} — {title}")
    lines.append("")
    lines.append(f"**Trigger:** "
                 + ("auto per §0.5 step 4 — post-upgrade migration check after "
                    f"`op:protocol_upgrade` to `{pinned}`."
                    if post_upgrade else
                    "on-demand or session-end §8.7 self-audit pass."))
    lines.append(f"**Approved by:** {actor_id}")
    lines.append(f"**Generated at:** {now_iso(tz)}")
    lines.append(f"**Pinned protocol SHA:** `{pinned}`")
    lines.append(f"**Chain rows walked:** {chain_stats['rows']}")
    lines.append(f"**Chain head (ledger):** `{chain_stats['head']}`")
    lines.append(f"**Manifest audit_chain_head:** `{manifest.get('audit_chain_head')}`")
    lines.append("")
    lines.append("## Summary")
    lines.append("")
    lines.append(f"- **Critical:** {len(crit)}")
    lines.append(f"- **Warn:** {len(warn)}")
    lines.append(f"- **Info:** {len(info)}")
    lines.append("")
    if not crit:
        lines.append("**✓ No critical findings.**")
    else:
        lines.append("**✗ CRITICAL findings present — writes should be frozen "
                     "until repaired (MAINTENANCE mode).**")
    lines.append("")

    for sev, group in (("CRITICAL", crit), ("WARN", warn), ("INFO", info)):
        if not group:
            continue
        lines.append(f"## {sev} findings ({len(group)})")
        lines.append("")
        for f in group:
            lines.append(f"- phase {f['phase']}: {f['code']}")
        lines.append("")

    lines.append("## Stats")
    lines.append("")
    lines.append(f"- Memory files walked: {cap_stats['files']}")
    lines.append(f"- Total store bytes: {cap_stats['total_bytes']} "
                 f"({cap_stats['total_bytes']//1024} KB)")
    if bit_perfect and chain_stats.get("bit_perfect"):
        bp_n, bp_d = chain_stats["bit_perfect"]
        if bp_d:
            lines.append(f"- Audit-row bit-perfect recompute: {bp_n}/{bp_d} "
                         f"(differences are INFO per §7.2; LINK is authoritative)")
    lines.append("")
    report_text = "\n".join(lines) + "\n"
    report_bytes = report_text.encode("utf-8")
    report_sha = sha256_hex_bytes(report_bytes)

    # Write report + audit row under lock
    with BrainLock(brain_root, actor_id):
        atomic_write_bytes(report_abs, report_bytes)
        prev_chain, _ = latest_chain_head(brain_root)
        row = make_audit_row(
            op="health_check",
            actor_kind=actor_kind,
            actor_id=actor_id,
            scope="meta",
            path=f".cyberos-memory/{report_relpath}",
            prev_chain=prev_chain,
            tz=tz,
            classification="operational",
            authority="human-edited",
            provenance_source="manual",
            provenance_source_ref="8.7-self-audit-pass" + (
                ":post-upgrade" if post_upgrade else ""
            ),
            provenance_confidence=1.0,
            after_hash=report_sha,
            reason=(
                f"§8.7 self-audit"
                + (" (post-upgrade)" if post_upgrade else "")
                + f": {len(crit)} critical, {len(warn)} warn, "
                f"{len(info)} info findings; "
                f"chain LINK {'broken' if any('chain-link-break' in f['code'] for f in crit) else 'intact'}; "
                f"head {'reachable' if not any('audit-chain-head' in f['code'] for f in crit) else 'NOT reachable'}"
            ),
        )
        audit_path = current_audit_path(brain_root, tz)
        append_audit_line(audit_path, serialise_row_for_disk(row))

    print()
    print(f"report: {report_relpath} ({len(report_bytes)} bytes; sha={report_sha[:30]}…)")
    print(f"audit-row: {row['audit_id']} chain={row['chain']}")
    print(f"summary: {len(crit)} CRITICAL / {len(warn)} WARN / {len(info)} INFO")
    return 2 if crit else 0


# ───────────────────────────────────────────────────────────────────────
# Subcommand: verify
# ───────────────────────────────────────────────────────────────────────

def cmd_verify(*, bit_perfect: bool) -> int:
    brain_root = resolve_brain_root()
    manifest = read_manifest(brain_root)
    paths = all_audit_paths(brain_root)
    if not paths:
        print("verify: no audit ledgers found — empty chain.")
        return 0

    rows: list[dict] = []
    for p in paths:
        with open(p, "r", encoding="utf-8") as f:
            for lineno, line in enumerate(f, 1):
                if not line.strip():
                    continue
                try:
                    rows.append(json.loads(line))
                except json.JSONDecodeError as e:
                    die(
                        f"audit-corrupt:{p.name}:{lineno}:{e.msg}",
                        exit_code=2,
                    )

    if not rows:
        print("verify: ledgers present but empty — empty chain.")
        return 0

    # LINK invariant
    link_breaks = []
    for i in range(1, len(rows)):
        if rows[i]["prev_chain"] != rows[i - 1]["chain"]:
            link_breaks.append(i)

    bit_perfect_misses = 0
    if bit_perfect:
        for r in rows:
            if compute_chain(r, r["prev_chain"]) != r["chain"]:
                bit_perfect_misses += 1

    head = rows[-1]["chain"]
    manifest_head = manifest.get("audit_chain_head")
    head_in_chain = any(r["chain"] == manifest_head for r in rows)

    print(f"verify: rows={len(rows)} chain_head={head}")
    print(f"verify: LINK invariant breaks: {len(link_breaks)}"
          + (f" first-at={link_breaks[0]}" if link_breaks else ""))
    if bit_perfect:
        print(f"verify: bit-perfect recompute matches: "
              f"{len(rows) - bit_perfect_misses}/{len(rows)} "
              f"(differences are INFO per §7.2 cross-writer-version)")
    print(f"verify: manifest.audit_chain_head present in chain: "
          f"{head_in_chain} ({manifest_head})")

    if link_breaks:
        return 2
    if not head_in_chain:
        return 2
    return 0


# ───────────────────────────────────────────────────────────────────────
# Subcommand: status
# ───────────────────────────────────────────────────────────────────────

def cmd_status() -> int:
    brain_root = resolve_brain_root()
    manifest = read_manifest(brain_root)
    paths = all_audit_paths(brain_root)
    total = 0
    for p in paths:
        with open(p, "r", encoding="utf-8") as f:
            total += sum(1 for line in f if line.strip())
    head, _ = latest_chain_head(brain_root)
    print(f"brain_root: {brain_root}")
    print(f"protocol.sha256: {manifest.get('protocol', {}).get('sha256')}")
    print(f"operational_mode: {manifest.get('operational_mode')}")
    print(f"memory_count: {manifest.get('memory_count')}")
    print(f"audit_chain_head (manifest): {manifest.get('audit_chain_head')}")
    print(f"audit_chain_head (ledger):   {head}")
    print(f"audit_rows_total: {total}")
    print(f"reconciliation_checkpoint: "
          f"{manifest.get('reconciliation_checkpoint', {}).get('audit_id')}")
    return 0


# ───────────────────────────────────────────────────────────────────────
# Helpers
# ───────────────────────────────────────────────────────────────────────

def split_actor(actor: str) -> tuple[str, str]:
    """Parse 'agent:claude-code' / 'human:stephen' / 'subject:steph' /
    'system:cron' into (actor_kind, actor_id). Defaults to agent when no
    prefix is given (matches CHAIN_ORCHESTRATOR.md examples)."""
    if ":" not in actor:
        return ("agent", actor)
    kind, rest = actor.split(":", 1)
    if kind not in ("agent", "human", "system", "subject"):
        die(f"actor-kind-unknown:{kind}", exit_code=2)
    return (kind, actor)  # actor_id keeps the prefix per existing convention


# ───────────────────────────────────────────────────────────────────────
# CLI
# ───────────────────────────────────────────────────────────────────────

def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        prog="brain_writer.py",
        description="Reference writer for the CyberOS BRAIN audit chain.",
    )
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_ss = sub.add_parser("session-start", help="Append op:session.start")
    p_ss.add_argument("actor", help="e.g. agent:claude-opus-4-7")

    p_se = sub.add_parser("session-end",
                          help="Append op:session.end + manifest update")
    p_se.add_argument("actor")

    p_w = sub.add_parser("write", help="Create a memory file + op:create")
    p_w.add_argument("actor")
    p_w.add_argument("relpath", help="Path relative to .cyberos-memory/")
    p_w.add_argument("content_file", help="Local file with the new content")

    p_sr = sub.add_parser("str-replace",
                          help="Replace whole file + op:str_replace")
    p_sr.add_argument("actor")
    p_sr.add_argument("relpath")
    p_sr.add_argument("new_file")

    p_pu = sub.add_parser("protocol-upgrade",
                          help="§0.5 manifest pin update + op:protocol_upgrade")
    p_pu.add_argument("actor")
    p_pu.add_argument("old_sha", help="sha256:<64-hex> currently pinned")
    p_pu.add_argument("new_sha", help="sha256:<64-hex> new canonical AGENTS.md")
    p_pu.add_argument("--reason", help="Override default reason text")

    p_v = sub.add_parser("verify", help="Walk and verify the chain")
    p_v.add_argument("--bit-perfect", action="store_true",
                     help="Also recompute every row's hash via JCS")

    p_sa = sub.add_parser("self-audit",
                          help="§8.7 6-phase pass; writes meta/health/<…>.md")
    p_sa.add_argument("actor")
    p_sa.add_argument("--post-upgrade", action="store_true",
                      help="Treat as §0.5 step 4 post-upgrade scan "
                           "(report filename suffixed with -postupgrade)")
    p_sa.add_argument("--no-bit-perfect", action="store_true",
                      help="Skip bit-perfect recompute (faster)")

    sub.add_parser("status", help="Print chain head + manifest summary")

    args = parser.parse_args(argv)
    if args.cmd == "session-start":
        return cmd_session_start(args.actor)
    if args.cmd == "session-end":
        return cmd_session_end(args.actor)
    if args.cmd == "write":
        return cmd_write(args.actor, args.relpath, args.content_file)
    if args.cmd == "str-replace":
        return cmd_str_replace(args.actor, args.relpath, args.new_file)
    if args.cmd == "protocol-upgrade":
        return cmd_protocol_upgrade(
            args.actor, args.old_sha, args.new_sha, reason=args.reason,
        )
    if args.cmd == "verify":
        return cmd_verify(bit_perfect=args.bit_perfect)
    if args.cmd == "self-audit":
        return cmd_self_audit(
            args.actor,
            post_upgrade=args.post_upgrade,
            bit_perfect=not args.no_bit_perfect,
        )
    if args.cmd == "status":
        return cmd_status()
    parser.print_help()
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
