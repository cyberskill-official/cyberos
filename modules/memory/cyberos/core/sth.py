"""
cyberos.core.sth — Signed Tree Heads over the MMR (PROPOSAL.md P2 Stage 1).

A Signed Tree Head (STH) is the auditor-facing primitive that Sigstore
Rekor, Google CT, and DataTrails all converge on. Each STH commits to:

* a tree size (leaf count),
* the MMR root at that size,
* a UTC timestamp,
* the signer key id,
* an Ed25519 signature over the canonical serialisation of the above.

Per AGENTS.md v2 §6.4 and PROPOSAL.md Appendix, Stage 1 is **additive**:
STHs are produced alongside the per-row chain at every consolidation,
but the chain remains the source of truth. The chain primitive switch
(§6 P2 Stage 3) requires a separate chat-turn approval.

Key management (PROPOSAL.md Appendix Q2):

* Key file at ``~/.config/cyberos/sth_signing_key`` (raw 32-byte
  Ed25519 seed) — Stage 1 simplification. Passphrase-wrapping via
  ``age``-style scrypt is the Stage 2 hardening.
* Public key persisted alongside as ``sth_signing_key.pub`` so
  verifiers can check without the private material.
* Key rotation produces a `rotation` STH whose ``previous_signer``
  field pins the old key's last STH by file path.

Dependencies: ``cryptography`` (BSD; widely available). Falls back to
:class:`P2NotActive` if not installed so the rest of the package keeps
working on bare-stdlib hosts.
"""

from __future__ import annotations

import base64
import hashlib
import json
import os
import struct
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Final

try:
    from cryptography.hazmat.primitives.asymmetric.ed25519 import (
        Ed25519PrivateKey,
        Ed25519PublicKey,
    )
    from cryptography.hazmat.primitives import serialization
    from cryptography.hazmat.primitives.ciphers.aead import ChaCha20Poly1305
    from cryptography.hazmat.primitives.kdf.scrypt import Scrypt
    from cryptography.exceptions import InvalidSignature
    _CRYPTO_AVAILABLE: bool = True
except ImportError:  # pragma: no cover
    _CRYPTO_AVAILABLE = False


class P2NotActive(RuntimeError):
    """Raised when STH operations are attempted without `cryptography` installed."""


# --- key storage ----------------------------------------------------------


_KEY_DIR_DEFAULT: Final[Path] = Path.home() / ".config" / "cyberos"
_KEY_FILE_DEFAULT: Final[str] = "sth_signing_key"

# --- passphrase-wrapping (PROPOSAL.md P2 Stage 2) -------------------------
#
# Wrapped key file format:
#   magic (16 bytes)  = b"CYBEROS-WRAPKEY1\n" -- detects wrapped vs raw
#   scrypt_salt (16)  = random per-file
#   scrypt_n (u32 BE) = scrypt cost parameter (default 2**15)
#   scrypt_r (u32 BE) = block size (default 8)
#   scrypt_p (u32 BE) = parallelization (default 1)
#   nonce (12 bytes)  = ChaCha20-Poly1305 nonce, random per-file
#   ciphertext        = encrypted 32-byte Ed25519 seed + auth tag
#
# Raw key files (Stage 1) are still loadable: detection is by header.

_WRAP_MAGIC: Final[bytes] = b"CYBEROS-WRAPKEY1\n"
_SCRYPT_N_DEFAULT: Final[int] = 1 << 15
_SCRYPT_R_DEFAULT: Final[int] = 8
_SCRYPT_P_DEFAULT: Final[int] = 1
_NONCE_LEN: Final[int] = 12
_SALT_LEN: Final[int] = 16


def _wrap_seed(seed: bytes, passphrase: bytes) -> bytes:
    """Wrap a 32-byte Ed25519 seed with passphrase-derived ChaCha20-Poly1305."""
    if not _CRYPTO_AVAILABLE:
        raise P2NotActive("STH key wrapping requires 'cryptography'")
    salt = os.urandom(_SALT_LEN)
    nonce = os.urandom(_NONCE_LEN)
    kdf = Scrypt(
        salt=salt, length=32,
        n=_SCRYPT_N_DEFAULT, r=_SCRYPT_R_DEFAULT, p=_SCRYPT_P_DEFAULT,
    )
    derived = kdf.derive(passphrase)
    cipher = ChaCha20Poly1305(derived)
    ciphertext = cipher.encrypt(nonce, seed, associated_data=_WRAP_MAGIC)
    n_bytes = struct.pack(">I", _SCRYPT_N_DEFAULT)
    r_bytes = struct.pack(">I", _SCRYPT_R_DEFAULT)
    p_bytes = struct.pack(">I", _SCRYPT_P_DEFAULT)
    return _WRAP_MAGIC + salt + n_bytes + r_bytes + p_bytes + nonce + ciphertext


def _unwrap_seed(blob: bytes, passphrase: bytes) -> bytes:
    """Reverse of :func:`_wrap_seed`. Raises on bad passphrase or tamper."""
    if not _CRYPTO_AVAILABLE:
        raise P2NotActive("STH key unwrapping requires 'cryptography'")
    if not blob.startswith(_WRAP_MAGIC):
        raise ValueError("not a wrapped key file (missing magic)")
    offset = len(_WRAP_MAGIC)
    salt = blob[offset:offset + _SALT_LEN]
    offset += _SALT_LEN
    n, = struct.unpack_from(">I", blob, offset); offset += 4
    r, = struct.unpack_from(">I", blob, offset); offset += 4
    p, = struct.unpack_from(">I", blob, offset); offset += 4
    nonce = blob[offset:offset + _NONCE_LEN]
    offset += _NONCE_LEN
    ciphertext = blob[offset:]
    kdf = Scrypt(salt=salt, length=32, n=n, r=r, p=p)
    derived = kdf.derive(passphrase)
    cipher = ChaCha20Poly1305(derived)
    return cipher.decrypt(nonce, ciphertext, associated_data=_WRAP_MAGIC)


def _read_passphrase() -> bytes | None:
    """Resolve passphrase from env (preferred) or interactive prompt.

    Order of precedence:
    1. ``CYBEROS_STH_PASSPHRASE`` env var (preferred for CI / scripts).
    2. ``getpass.getpass()`` if stdin is a TTY.
    3. None — signals "use raw-key path" (stage-1 compat).
    """
    env = os.environ.get("CYBEROS_STH_PASSPHRASE")
    if env:
        return env.encode("utf-8")
    if sys.stdin.isatty():
        import getpass
        try:
            return getpass.getpass("STH signing key passphrase: ").encode("utf-8")
        except (EOFError, KeyboardInterrupt):
            return None
    return None


@dataclass
class KeyPaths:
    private: Path
    public: Path

    @classmethod
    def default(cls, base: Path | None = None) -> "KeyPaths":
        d = base or _KEY_DIR_DEFAULT
        return cls(private=d / _KEY_FILE_DEFAULT, public=d / f"{_KEY_FILE_DEFAULT}.pub")


def ensure_key(
    paths: KeyPaths | None = None,
    *,
    passphrase: bytes | None = None,
) -> KeyPaths:
    """Generate the Ed25519 signing key if it doesn't exist; return paths.

    If ``passphrase`` (or ``CYBEROS_STH_PASSPHRASE`` env, or interactive
    prompt on a TTY) is available, the key is wrapped via scrypt +
    ChaCha20-Poly1305 (PROPOSAL.md P2 Stage 2). Otherwise the raw seed
    is written, matching Stage-1 semantics.

    The file's first 16 bytes distinguish wrapped (``CYBEROS-WRAPKEY1\\n``)
    from raw, so :func:`load_signing_key` reads either.
    """
    if not _CRYPTO_AVAILABLE:
        raise P2NotActive(
            "STH signing requires the 'cryptography' package. "
            "Install: pip install cryptography --break-system-packages"
        )
    p = paths or KeyPaths.default()
    if p.private.is_file() and p.public.is_file():
        return p
    p.private.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    private = Ed25519PrivateKey.generate()
    seed = private.private_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PrivateFormat.Raw,
        encryption_algorithm=serialization.NoEncryption(),
    )
    public_bytes = private.public_key().public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw,
    )

    if passphrase is None:
        passphrase = _read_passphrase()
    if passphrase:
        body = _wrap_seed(seed, passphrase)
    else:
        body = seed
    # Atomic write the private key file with 0o600.
    tmp = p.private.with_suffix(".key.tmp")
    fd = os.open(tmp, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    try:
        os.write(fd, body)
    finally:
        os.close(fd)
    os.replace(tmp, p.private)
    p.public.write_bytes(public_bytes)
    return p


def load_signing_key(
    paths: KeyPaths | None = None,
    *,
    passphrase: bytes | None = None,
) -> "Ed25519PrivateKey":
    """Load the Ed25519 signing key.

    Auto-detects wrapped vs raw via the file's magic header. Wrapped
    keys require ``passphrase`` (or the env var / interactive prompt
    fallback). Raises if a wrapped file has no passphrase available.
    """
    if not _CRYPTO_AVAILABLE:
        raise P2NotActive("STH signing requires 'cryptography'")
    p = paths or KeyPaths.default()
    if not p.private.is_file():
        raise FileNotFoundError(
            f"signing key missing at {p.private}; "
            "run sth.ensure_key() to generate"
        )
    blob = p.private.read_bytes()
    if blob.startswith(_WRAP_MAGIC):
        if passphrase is None:
            passphrase = _read_passphrase()
        if not passphrase:
            raise ValueError(
                f"signing key at {p.private} is passphrase-wrapped but "
                "no passphrase was provided. Set CYBEROS_STH_PASSPHRASE "
                "or run interactively."
            )
        seed = _unwrap_seed(blob, passphrase)
    else:
        seed = blob
    return Ed25519PrivateKey.from_private_bytes(seed)


def wrap_existing_key(
    paths: KeyPaths | None = None,
    *,
    passphrase: bytes,
) -> None:
    """Convert a stage-1 raw key file to a stage-2 wrapped one in place.

    Idempotent: already-wrapped files are left alone. Atomic: a partial
    write does not corrupt the key — the rename is the commit point.
    Caller MUST supply ``passphrase`` explicitly (no env/prompt fallback,
    so the operation is unambiguous).
    """
    if not _CRYPTO_AVAILABLE:
        raise P2NotActive("STH key wrapping requires 'cryptography'")
    p = paths or KeyPaths.default()
    if not p.private.is_file():
        raise FileNotFoundError(p.private)
    blob = p.private.read_bytes()
    if blob.startswith(_WRAP_MAGIC):
        return  # already wrapped
    if len(blob) != 32:
        raise ValueError(
            f"raw key at {p.private} has length {len(blob)}, expected 32"
        )
    wrapped = _wrap_seed(blob, passphrase)
    tmp = p.private.with_suffix(".key.tmp")
    fd = os.open(tmp, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    try:
        os.write(fd, wrapped)
    finally:
        os.close(fd)
    os.replace(tmp, p.private)


def load_public_key(paths: KeyPaths | None = None) -> "Ed25519PublicKey":
    if not _CRYPTO_AVAILABLE:
        raise P2NotActive("STH verification requires 'cryptography'")
    p = paths or KeyPaths.default()
    return Ed25519PublicKey.from_public_bytes(p.public.read_bytes())


def key_id(public_key: "Ed25519PublicKey") -> str:
    """Stable identifier for a public key — SHA-256 of its raw bytes."""
    raw = public_key.public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw,
    )
    return hashlib.sha256(raw).hexdigest()[:16]  # short form; full hex stored elsewhere


# --- STH record schema ----------------------------------------------------


def canonical_sign_input(record: dict) -> bytes:
    """Deterministic bytes that the Ed25519 signature commits to.

    Excludes the ``signature`` field (which is what we are computing).
    The ``previous_sth`` field IS included — it's part of the chain
    of signed heads and the signer's commitment to it matters.
    Sorted keys, no whitespace.
    """
    payload = {k: v for k, v in record.items() if k != "signature"}
    return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")


def sign_tree_head(
    *,
    tree_size: int,
    root_hash_hex: str,
    paths: KeyPaths | None = None,
    previous_sth_relpath: str | None = None,
    passphrase: bytes | None = None,
) -> dict:
    """Produce a Signed Tree Head record.

    Returns the dict; the caller writes it to
    ``audit/sth/<timestamp>-<root>.json``. Raises :class:`P2NotActive`
    if the cryptography package is unavailable.
    """
    if not _CRYPTO_AVAILABLE:
        raise P2NotActive(
            "STH signing requires the 'cryptography' package. "
            "Install: pip install cryptography --break-system-packages"
        )
    if len(root_hash_hex) != 64 or any(c not in "0123456789abcdef" for c in root_hash_hex):
        raise ValueError(f"root_hash_hex must be 64-hex lowercase, got {root_hash_hex!r}")

    p = paths or KeyPaths.default()
    ensure_key(p, passphrase=passphrase)
    private = load_signing_key(p, passphrase=passphrase)
    public = private.public_key()
    kid = key_id(public)

    record = {
        "tree_size": tree_size,
        "root_hash": root_hash_hex,
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "signer": kid,
        "previous_sth": previous_sth_relpath,
    }
    sig = private.sign(canonical_sign_input(record))
    record["signature"] = base64.b64encode(sig).decode("ascii")
    return record


def verify_tree_head(
    record: dict,
    paths: KeyPaths | None = None,
    public_key: "Ed25519PublicKey | None" = None,
) -> bool:
    """Verify an STH's Ed25519 signature.

    Pass ``public_key`` directly to verify against an arbitrary key
    (useful for cross-host verification); otherwise the local public
    key file is used.
    """
    if not _CRYPTO_AVAILABLE:
        raise P2NotActive("STH verification requires 'cryptography'")
    if "signature" not in record:
        return False
    try:
        sig = base64.b64decode(record["signature"], validate=True)
    except (ValueError, TypeError):
        return False
    pk = public_key or load_public_key(paths)
    try:
        pk.verify(sig, canonical_sign_input(record))
        return True
    except InvalidSignature:
        return False


# --- on-disk STH filing ---------------------------------------------------


def write_sth(store: Path, record: dict) -> Path:
    """Persist ``record`` to ``<store>/audit/sth/<timestamp>-<root>.json``."""
    sth_dir = store / "audit" / "sth"
    sth_dir.mkdir(parents=True, exist_ok=True)
    # Filename: replace ':' (Windows-hostile) with '-'; truncate root to 16 hex.
    ts = record["timestamp"].replace(":", "-")
    fname = f"{ts}-{record['root_hash'][:16]}.json"
    out = sth_dir / fname
    body = json.dumps(record, sort_keys=True, indent=2).encode("utf-8") + b"\n"
    tmp = out.with_suffix(".json.tmp")
    flags = os.O_WRONLY | os.O_CREAT | os.O_TRUNC | getattr(os, "O_CLOEXEC", 0)
    fd = os.open(tmp, flags, 0o600)
    try:
        os.write(fd, body)
        from cyberos.core.fsync import durable_sync
        durable_sync(fd)
    finally:
        os.close(fd)
    os.replace(tmp, out)
    from cyberos.core.fsync import durable_dir_sync
    durable_dir_sync(sth_dir)
    return out


def latest_sth(store: Path) -> tuple[Path, dict] | None:
    """Return the path + parsed record of the most recent STH in ``<store>``."""
    sth_dir = store / "audit" / "sth"
    if not sth_dir.is_dir():
        return None
    files = sorted(sth_dir.glob("*.json"))
    if not files:
        return None
    last = files[-1]
    return last, json.loads(last.read_text(encoding="utf-8"))


# --- end-to-end helper ----------------------------------------------------


def sign_and_publish(
    store: Path,
    *,
    tree_size: int,
    root_hash_hex: str,
    paths: KeyPaths | None = None,
) -> Path:
    """Sign an STH for the current MMR state and write it to the store.

    Stage 1 usage: called from ``cyberos consolidate``. Stage 3 (post-
    primitive-swap) usage: called by the writer at every batch flush —
    but Stage 3 requires the user's separate approval.
    """
    prev = latest_sth(store)
    prev_relpath = None if prev is None else str(prev[0].relative_to(store))
    record = sign_tree_head(
        tree_size=tree_size,
        root_hash_hex=root_hash_hex,
        paths=paths,
        previous_sth_relpath=prev_relpath,
    )
    return write_sth(store, record)


__all__ = [
    "KeyPaths",
    "P2NotActive",
    "canonical_sign_input",
    "ensure_key",
    "key_id",
    "latest_sth",
    "load_public_key",
    "load_signing_key",
    "sign_and_publish",
    "sign_tree_head",
    "verify_tree_head",
    "wrap_existing_key",
    "write_sth",
]
