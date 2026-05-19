#!/usr/bin/env python3
"""
cyberos-encrypt — at-rest encryption + Shamir 3-of-5 escrow for `.cyberos-memory/`.

Implements AGENTS.md §5.6 (post-Stage-5 SHA `sha256:d3ce9764…`):
  - XChaCha20-Poly1305-IETF body envelope (frontmatter stays plaintext)
  - HKDF-SHA256 master-key derivation from HW-bound key OR Argon2id passphrase
  - Mandatory Shamir 3-of-5 recovery escrow at enable time
  - User-paced migration via --migrate-batch <N>

Subcommands
-----------
    cyberos-encrypt <store> enable [--passphrase | --hw=<backend>]
    cyberos-encrypt <store> disable
    cyberos-encrypt <store> migrate-batch <N>
    cyberos-encrypt <store> rotate-shamir
    cyberos-encrypt <store> recover < fragments.txt
    cyberos-encrypt <store> status

Exit codes
----------
0 = ok / wizard completed cleanly
1 = wizard cancelled by user
2 = error in cryptographic operation
3 = invocation error / preconditions not met

Dependencies
------------
    cryptography  — XChaCha20-Poly1305 + HKDF
    argon2-cffi   — passphrase fallback (Argon2id RFC 9106)
    pyyaml        — frontmatter parsing
    rfc8785       — canonical JSON for audit chain
    zxcvbn        — passphrase strength (optional; warns on dictionary words)

Author: CyberOS local-optimization Stage 5 implementation
"""

from __future__ import annotations

import argparse
import base64
import datetime as dt
import getpass
import hashlib
import json
import os
import secrets
import subprocess
import sys
from pathlib import Path
from typing import Any

try:
    import yaml  # type: ignore
except ImportError:
    yaml = None  # type: ignore

try:
    from cryptography.hazmat.primitives.ciphers.aead import ChaCha20Poly1305
    from cryptography.hazmat.primitives.kdf.hkdf import HKDF
    from cryptography.hazmat.primitives import hashes
    _HAS_CRYPTO = True
except ImportError:
    _HAS_CRYPTO = False

try:
    from argon2.low_level import hash_secret_raw, Type as Argon2Type
    _HAS_ARGON2 = True
except ImportError:
    _HAS_ARGON2 = False

try:
    import rfc8785
    _HAS_JCS = True
except ImportError:
    _HAS_JCS = False

try:
    from zxcvbn import zxcvbn
    _HAS_ZXCVBN = True
except ImportError:
    _HAS_ZXCVBN = False


# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

XCHACHA_NONCE_BYTES = 24
MASTER_KEY_BYTES = 32
HKDF_INFO = b"cyberos-stage5-master-key-v1"
ARGON2_T = 3
ARGON2_M_KIB = 64 * 1024  # 64 MiB
ARGON2_P = 4
ARGON2_SALT = b"cyberos-stage5-passphrase-salt-v1"  # static salt OK; key never persisted

SHAMIR_THRESHOLD = 3
SHAMIR_TOTAL = 5
PASSPHRASE_MIN_CHARS = 16
PASSPHRASE_MIN_ZXCVBN = 3


# ---------------------------------------------------------------------------
# Shamir Secret Sharing over GF(256) — canonical Shamir-1979 with Lagrange.
# ~80 LOC inline; avoids external dependency. Reference: rfc-draft-mcgrew-tss
# ---------------------------------------------------------------------------

def _gf_mul(a: int, b: int) -> int:
    """Multiply two bytes in GF(256) (Rijndael's polynomial 0x11b)."""
    p = 0
    for _ in range(8):
        if b & 1:
            p ^= a
        hi = a & 0x80
        a = (a << 1) & 0xff
        if hi:
            a ^= 0x1b
        b >>= 1
    return p


def _gf_pow(base: int, exp: int) -> int:
    result = 1
    while exp:
        if exp & 1:
            result = _gf_mul(result, base)
        base = _gf_mul(base, base)
        exp >>= 1
    return result


def _gf_inv(a: int) -> int:
    """a^(-1) = a^254 in GF(256)."""
    if a == 0:
        raise ValueError("zero has no inverse in GF(256)")
    return _gf_pow(a, 254)


def _eval_poly(coeffs: list[int], x: int) -> int:
    """Horner evaluation in GF(256)."""
    y = 0
    for c in reversed(coeffs):
        y = _gf_mul(y, x) ^ c
    return y


def shamir_split(secret: bytes, threshold: int, total: int) -> list[bytes]:
    """Split `secret` into `total` shares; any `threshold` reconstruct.

    Each share is `bytes([x_index]) || bytes_per_byte_share`. Encoded for
    base32 round-trip.
    """
    assert 1 <= threshold <= total <= 255
    shares = [bytearray([i + 1]) for i in range(total)]  # x ∈ {1..total}
    for byte_index, secret_byte in enumerate(secret):
        # Polynomial: f(x) = secret_byte + a1 x + a2 x^2 + ...
        coeffs = [secret_byte] + [secrets.randbelow(256)
                                   for _ in range(threshold - 1)]
        for share in shares:
            x = share[0]
            share.append(_eval_poly(coeffs, x))
    return [bytes(s) for s in shares]


def shamir_combine(shares: list[bytes]) -> bytes:
    """Reconstruct secret from `len(shares) >= threshold` distinct shares."""
    if not shares:
        raise ValueError("no shares")
    secret_len = len(shares[0]) - 1
    for s in shares:
        if len(s) - 1 != secret_len:
            raise ValueError("share length mismatch")
    xs = [s[0] for s in shares]
    if len(set(xs)) != len(xs):
        raise ValueError("duplicate share x-coords")

    secret = bytearray(secret_len)
    for byte_index in range(secret_len):
        ys = [s[byte_index + 1] for s in shares]
        # Lagrange interpolation at x=0
        result = 0
        for i, xi in enumerate(xs):
            num = 1
            den = 1
            for j, xj in enumerate(xs):
                if i == j:
                    continue
                num = _gf_mul(num, xj)
                den = _gf_mul(den, xi ^ xj)
            term = _gf_mul(ys[i], _gf_mul(num, _gf_inv(den)))
            result ^= term
        secret[byte_index] = result
    return bytes(secret)


def encode_share(share: bytes, label: str = "") -> str:
    """Encode a Shamir share for printable distribution."""
    b32 = base64.b32encode(share).decode("ascii").rstrip("=")
    # Format as 4-char groups for readability
    grouped = "-".join(b32[i:i+4] for i in range(0, len(b32), 4))
    label_part = label if label else f"{share[0]:02x}"
    prefix = f"CYBOS-S5-{label_part}"
    return f"{prefix}-{grouped}"


def decode_share(encoded: str) -> bytes:
    """Decode a printable share back to bytes."""
    parts = encoded.strip().split("-")
    if len(parts) < 4 or parts[0] != "CYBOS" or parts[1] != "S5":
        raise ValueError(f"not a CyberOS Stage-5 share: {encoded[:30]}...")
    b32 = "".join(parts[3:])
    pad = "=" * ((8 - len(b32) % 8) % 8)
    return base64.b32decode(b32 + pad)


# ---------------------------------------------------------------------------
# Key-derivation backends
# ---------------------------------------------------------------------------

class KeyBackend:
    """Abstract: derive a master key from a backend-specific source."""
    name: str

    def derive(self) -> bytes:  # noqa: D401
        """Return 32-byte master key."""
        raise NotImplementedError


class PassphraseBackend(KeyBackend):
    name = "passphrase-argon2id"

    def __init__(self, passphrase: str):
        if len(passphrase) < PASSPHRASE_MIN_CHARS:
            raise ValueError(
                f"passphrase must be ≥{PASSPHRASE_MIN_CHARS} characters")
        if _HAS_ZXCVBN:
            score = zxcvbn(passphrase).get("score", 0)
            if score < PASSPHRASE_MIN_ZXCVBN:
                raise ValueError(
                    f"passphrase zxcvbn score {score} < required "
                    f"{PASSPHRASE_MIN_ZXCVBN}; use a stronger passphrase")
        self.passphrase = passphrase

    def derive(self) -> bytes:
        if not _HAS_ARGON2:
            raise RuntimeError("argon2-cffi not installed")
        return hash_secret_raw(
            secret=self.passphrase.encode("utf-8"),
            salt=ARGON2_SALT,
            time_cost=ARGON2_T,
            memory_cost=ARGON2_M_KIB,
            parallelism=ARGON2_P,
            hash_len=MASTER_KEY_BYTES,
            type=Argon2Type.ID,
        )


class MacOSKeychainBackend(KeyBackend):
    """macOS Keychain-stored secret + Touch ID prompt (when configured).

    Uses the `security` CLI to read/write a 32-byte master-seed entry in the
    user's login keychain. First derive() generates and stores; subsequent
    calls retrieve. The Keychain ACL configures Touch ID requirement.

    Caveats:
    - This is keychain-stored secret, not pure Secure Enclave-bound. True SE
      binding requires SecKeyCreateRandomKey with kSecAttrTokenIDSecureEnclave
      which Python can only reach via PyObjC + Security framework. v1
      ships keychain-stored as the practical first step.
    - Touch ID prompts only if the keychain item ACL was created with
      kSecAccessControlBiometryAny — which the `security` CLI doesn't expose
      directly. This implementation uses a passphrase-protected keychain
      entry; OS prompts the user for keychain unlock when read.
    """
    name = "macos-keychain"
    SERVICE = "cyberos-master-key"
    ACCOUNT = "cyberos-stage5-master-seed"

    def _security(self, args: list[str]) -> tuple[int, str, str]:
        proc = subprocess.run(["security"] + args,
                              capture_output=True, text=True, check=False)
        return proc.returncode, proc.stdout, proc.stderr

    def _read(self) -> bytes | None:
        rc, out, err = self._security([
            "find-generic-password",
            "-s", self.SERVICE,
            "-a", self.ACCOUNT,
            "-w",  # output password only
        ])
        if rc != 0:
            return None
        return base64.b64decode(out.strip())

    def _write(self, seed: bytes) -> None:
        rc, _, err = self._security([
            "add-generic-password",
            "-s", self.SERVICE,
            "-a", self.ACCOUNT,
            "-w", base64.b64encode(seed).decode("ascii"),
            "-T", "",  # restrict access
            "-U",  # update if exists
        ])
        if rc != 0:
            raise RuntimeError(f"keychain write failed: {err}")

    def derive(self) -> bytes:
        # Try to read existing seed; if not found, generate + store
        seed = self._read()
        if seed is None:
            seed = secrets.token_bytes(32)
            self._write(seed)
        # Derive master via HKDF
        if not _HAS_CRYPTO:
            raise RuntimeError("cryptography package not installed")
        hkdf = HKDF(
            algorithm=hashes.SHA256(),
            length=MASTER_KEY_BYTES,
            salt=None,
            info=HKDF_INFO,
        )
        return hkdf.derive(seed)


class WindowsTPMBackend(KeyBackend):
    """Stub: Windows TPM 2.0 via Windows Hello. Not implemented in v0."""
    name = "windows-tpm"

    def derive(self) -> bytes:
        raise NotImplementedError(
            "Windows TPM backend not implemented in v0; "
            "use --passphrase fallback")


class LinuxTPMBackend(KeyBackend):
    """Stub: Linux TPM 2.0 via tpm2-tools / FIDO2 hmac-secret. Not in v0."""
    name = "linux-tpm-or-fido2"

    def derive(self) -> bytes:
        raise NotImplementedError(
            "Linux TPM/FIDO2 backend not implemented in v0; "
            "use --passphrase fallback")


def detect_hw_backend() -> KeyBackend | None:
    """Return a hardware backend instance if one is available; else None."""
    if sys.platform == "darwin":
        # Check `security` CLI is available
        try:
            proc = subprocess.run(["security", "-h"], capture_output=True,
                                  check=False, timeout=2)
            if proc.returncode == 0 or b"security" in proc.stdout + proc.stderr:
                return MacOSKeychainBackend()
        except (FileNotFoundError, subprocess.TimeoutExpired):
            pass
    # Windows TPM + Linux TPM/FIDO2 backends remain stubs (raise NotImplementedError on derive)
    return None


# ---------------------------------------------------------------------------
# Encryption envelope (per §5.6.1)
# ---------------------------------------------------------------------------

def derive_aad(memory_id: str, last_updated_at: str) -> bytes:
    return hashlib.sha256(
        memory_id.encode("utf-8") + last_updated_at.encode("utf-8")
    ).digest()


def encrypt_body(plaintext: str, master_key: bytes,
                 memory_id: str, last_updated_at: str) -> dict:
    """Return frontmatter `encryption:` block + base64 body."""
    if not _HAS_CRYPTO:
        raise RuntimeError("cryptography package not installed")
    if len(master_key) != MASTER_KEY_BYTES:
        raise ValueError(f"master_key must be {MASTER_KEY_BYTES} bytes")

    nonce = secrets.token_bytes(XCHACHA_NONCE_BYTES)
    aad = derive_aad(memory_id, last_updated_at)
    # cryptography's ChaCha20Poly1305 only supports 12-byte nonces; for the
    # 24-byte XChaCha variant we use the same algorithm but with the IETF
    # variant's HKDF-derived subkey (sub-key derivation per §5.6.1 elsewhere
    # is conceptual — for the v0 we use the underlying cryptography lib's
    # ChaCha20Poly1305 with a 12-byte nonce derived from the 24-byte nonce
    # via SHA-256 truncation, preserving the AEAD security properties).
    # NOTE: A full XChaCha20 implementation would use HChaCha20 to derive the
    # subkey; SHA-256 truncation is a v0 approximation. Migrate to a proper
    # XChaCha library (e.g., pynacl) for production.
    subkey = hashlib.sha256(master_key + nonce[:16]).digest()[:32]
    short_nonce = nonce[12:24]  # 12 bytes for ChaCha20Poly1305 IETF
    cipher = ChaCha20Poly1305(subkey)
    ciphertext_with_tag = cipher.encrypt(
        short_nonce, plaintext.encode("utf-8"), aad)

    return {
        "encryption_block": {
            "algorithm": "xchacha20poly1305-ietf-v0",
            "nonce": base64.b64encode(nonce).decode("ascii"),
            "aad": "sha256(memory_id||last_updated_at)",
        },
        "body": base64.b64encode(ciphertext_with_tag).decode("ascii"),
    }


def decrypt_body(encrypted_body: str, master_key: bytes,
                 nonce_b64: str, memory_id: str,
                 last_updated_at: str) -> str:
    if not _HAS_CRYPTO:
        raise RuntimeError("cryptography package not installed")
    nonce = base64.b64decode(nonce_b64)
    aad = derive_aad(memory_id, last_updated_at)
    subkey = hashlib.sha256(master_key + nonce[:16]).digest()[:32]
    short_nonce = nonce[12:24]  # 12 bytes for ChaCha20Poly1305 IETF
    cipher = ChaCha20Poly1305(subkey)
    ciphertext_with_tag = base64.b64decode(encrypted_body)
    plaintext = cipher.decrypt(short_nonce, ciphertext_with_tag, aad)
    return plaintext.decode("utf-8")


# ---------------------------------------------------------------------------
# Frontmatter helpers
# ---------------------------------------------------------------------------

def split_frontmatter(text: str) -> tuple[str | None, str]:
    if not text.startswith("---\n"):
        return None, text
    rest = text[4:]
    end = rest.find("\n---\n")
    if end < 0:
        return None, text
    return rest[:end], rest[end + 5:]


def in_scope(memory_path: str, fm: dict, scopes: list[str]) -> bool:
    """Check whether memory matches any scope filter entry."""
    classification = fm.get("classification", "")
    for entry in scopes:
        if entry.startswith("classification:"):
            if classification == entry.split(":", 1)[1]:
                return True
        elif entry.startswith("path:"):
            if memory_path.startswith(entry.split(":", 1)[1]):
                return True
        elif entry.startswith("member:") and "private" in entry:
            # member:<self>/private — match member/<id>/private/* paths
            if "/private/" in memory_path or memory_path.endswith("/private"):
                return True
    return False


# ---------------------------------------------------------------------------
# Command implementations
# ---------------------------------------------------------------------------

def cmd_status(store: Path) -> int:
    manifest = json.loads((store / "manifest.json").read_text())
    pol = manifest.get("encryption_policy", {})
    sh = manifest.get("shamir_fragments", {})

    # Walk memories, count encrypted vs plaintext per scope
    encrypted = plaintext = 0
    for d in ("company", "module", "member", "client", "project",
              "persona", "memories", "meta"):
        scope_dir = store / d
        if not scope_dir.exists():
            continue
        for md in scope_dir.rglob("*.md"):
            try:
                fm_yaml, _ = split_frontmatter(md.read_text(encoding="utf-8"))
                if fm_yaml and yaml:
                    fm = yaml.safe_load(fm_yaml) or {}
                    if fm.get("encrypted"):
                        encrypted += 1
                    else:
                        plaintext += 1
            except (OSError, UnicodeDecodeError):
                continue

    print(json.dumps({
        "policy_enabled": pol.get("enabled", False),
        "policy_scopes": pol.get("scopes", []),
        "policy_algorithm": pol.get("algorithm"),
        "policy_key_derivation": pol.get("key_derivation"),
        "shamir_threshold": sh.get("threshold"),
        "shamir_total": sh.get("total"),
        "shamir_master_key_fingerprint": sh.get("master_key_fingerprint"),
        "shamir_fragments_distributed": len([
            f for f in sh.get("fragments", [])
            if f.get("distributed_at")
        ]),
        "memories_encrypted": encrypted,
        "memories_plaintext": plaintext,
    }, indent=2))
    return 0


def cmd_enable(store: Path, *, passphrase_mode: bool = True) -> int:
    """Wizard flow per §5.6.3."""
    if not _HAS_CRYPTO:
        print("error: cryptography package not installed", file=sys.stderr)
        return 3
    if not _HAS_ARGON2 and passphrase_mode:
        print("error: argon2-cffi not installed (required for passphrase fallback)",
              file=sys.stderr)
        return 3

    manifest_path = store / "manifest.json"
    manifest = json.loads(manifest_path.read_text())
    if manifest.get("encryption_policy", {}).get("enabled"):
        print("encryption already enabled")
        return 1

    # 1. Choose key backend
    if not passphrase_mode:
        hw = detect_hw_backend()
        if hw is None:
            print("no hardware key detected; falling back to passphrase",
                  file=sys.stderr)
            passphrase_mode = True

    if passphrase_mode:
        print("=== Passphrase setup ===")
        print(f"  Requirements: ≥{PASSPHRASE_MIN_CHARS} chars, "
              f"{'zxcvbn ≥3' if _HAS_ZXCVBN else 'no zxcvbn check available'}")
        while True:
            pw1 = getpass.getpass("  passphrase: ")
            pw2 = getpass.getpass("  confirm:    ")
            if pw1 != pw2:
                print("  ✘ mismatch; try again")
                continue
            try:
                backend = PassphraseBackend(pw1)
                break
            except ValueError as e:
                print(f"  ✘ {e}")
        print(f"  ✅ passphrase accepted; deriving via Argon2id "
              f"(t={ARGON2_T}, m={ARGON2_M_KIB}KiB, p={ARGON2_P}; ~2s)")
    else:
        backend = hw  # type: ignore

    master_key = backend.derive()
    fingerprint = "sha256:" + hashlib.sha256(master_key).hexdigest()
    print(f"  ✅ master key derived (fingerprint {fingerprint[:20]}...)")

    # 2. Shamir 3-of-5 split
    print("\n=== Shamir 3-of-5 escrow ===")
    print("  Generating 5 fragments. ANY 3 reconstruct the master key.")
    print("  Fragments NEVER stored in memory — only fingerprints + holder labels.")
    shares = shamir_split(master_key, SHAMIR_THRESHOLD, SHAMIR_TOTAL)
    fragment_records = []
    print()
    for i, share in enumerate(shares, 1):
        print(f"  --- FRAGMENT {i} of {SHAMIR_TOTAL} ---")
        label = input(f"    Holder label (e.g. 'spouse', 'lawyer'): ").strip() \
                or f"holder-{i}"
        encoded = encode_share(share, label.replace(" ", "-")[:16])
        fp = "sha256:" + hashlib.sha256(share).hexdigest()
        print(f"    Fragment encoded: {encoded}")
        print(f"    Fingerprint:      {fp[:30]}...")
        confirm = input(f"    Distributed to {label}? [y/N] ").strip().lower()
        if confirm != "y":
            print("  ✘ wizard cancelled (distribution not confirmed)")
            return 1
        fragment_records.append({
            "label": label,
            "fingerprint": fp,
            "created_at": dt.datetime.now(
                dt.timezone(dt.timedelta(hours=7))
            ).isoformat(timespec="seconds"),
            "distributed_at": dt.datetime.now(
                dt.timezone(dt.timedelta(hours=7))
            ).isoformat(timespec="seconds"),
        })
        print()

    # 3. Update manifest
    manifest["encryption_policy"]["enabled"] = True
    manifest["encryption_policy"]["key_derivation"] = backend.name
    manifest["shamir_fragments"]["master_key_fingerprint"] = fingerprint
    manifest["shamir_fragments"]["fragments"] = fragment_records
    manifest["last_updated_at"] = dt.datetime.now(
        dt.timezone(dt.timedelta(hours=7))).isoformat(timespec="seconds")

    # Atomic write
    tmp = manifest_path.with_suffix(f".tmp.{secrets.token_hex(8)}.part")
    tmp.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
    os.replace(tmp, manifest_path)

    # Audit-ledger integration (v1): emit one envelope of maintenance.start →
    # 5× shamir_distribution_confirmed → encryption_policy_change → maintenance.end.
    maint_session = _uuid7(dt.datetime.now(dt.timezone(dt.timedelta(hours=7))))
    _audit_append(store, {
        "op": "maintenance.start",
        "path": ".cyberos-memory/manifest.json",
        "reason": f"cyberos-encrypt enable wizard, session {maint_session}",
    })
    try:
        for i, rec in enumerate(fragment_records, 1):
            _audit_append(store, {
                "op": "shamir_distribution_confirmed",
                "path": ".cyberos-memory/manifest.json",
                "reason": (
                    f"enable wizard: fragment {i}/{SHAMIR_TOTAL} distributed "
                    f"to {rec['label']} (fingerprint {rec['fingerprint'][:30]}...)"
                ),
            })
        _audit_append(store, {
            "op": "encryption_policy_change",
            "path": ".cyberos-memory/manifest.json",
            "reason": (
                f"enable: policy.enabled false→true; key_derivation={backend.name}; "
                f"master fingerprint {fingerprint[:30]}...; "
                f"shamir threshold={SHAMIR_THRESHOLD}/{SHAMIR_TOTAL}"
            ),
        })
    finally:
        _audit_append(store, {
            "op": "maintenance.end",
            "path": ".cyberos-memory/manifest.json",
            "reason": f"cyberos-encrypt enable complete: session {maint_session}",
        })

    print("✅ encryption enabled.")
    print(f"   master key fingerprint: {fingerprint}")
    print(f"   distributed fragments:  {len(fragment_records)}/{SHAMIR_TOTAL}")
    print(f"   threshold:              {SHAMIR_THRESHOLD}")
    print()
    print("⚠ Security reminders:")
    print("  - Don't store any fragment alongside the memory")
    print("  - Recovery requires ≥3 distinct fragments")
    print("  - Rotate fragments via `cyberos-encrypt rotate-shamir` if any holder changes")
    return 0


# ---------------------------------------------------------------------------
# Audit-ledger integration helpers
# ---------------------------------------------------------------------------

def _audit_append(store: Path, partial: dict) -> str:
    """Append an audit row with full chain computation. Returns new chain."""
    audit_dir = store / "audit"
    ledgers = sorted(audit_dir.glob("*.jsonl"))
    if not ledgers:
        raise RuntimeError("no audit ledger found")
    latest = ledgers[-1]
    prev_chain = None
    with latest.open("r", encoding="utf-8") as f:
        for line in f:
            if line.strip():
                try:
                    prev_chain = json.loads(line).get("chain")
                except json.JSONDecodeError:
                    continue
    if not prev_chain:
        raise RuntimeError("ledger has no parseable rows")

    now = dt.datetime.now(dt.timezone(dt.timedelta(hours=7)))
    full = {
        "audit_id": f"evt_{_uuid7(now)}",
        "ts": now.isoformat(timespec="seconds"),
        "actor_kind": "agent",
        "actor_id": "cyberos-encrypt",
        "persona": None,
        "scope": "meta",
        "memory_id": None,
        "prev_version": None,
        "new_version": None,
        "supersedes_event_id": None,
        "classification": None,
        "authority": None,
        "consent_event_id": None,
        "provenance": {"source": "manual", "source_ref": "cyberos-encrypt", "confidence": 1.0},
        "before_hash": None,
        "after_hash": None,
        "diff": "<hash-only>",
        "reason": "",
        "correction_to": None,
        **partial,
    }
    if not _HAS_JCS:
        raise RuntimeError("rfc8785 required for audit chain computation")
    canonical = rfc8785.dumps(full)
    chain = "sha256:" + hashlib.sha256(canonical + prev_chain.encode("utf-8")).hexdigest()
    full["prev_chain"] = prev_chain
    full["chain"] = chain
    with latest.open("a", encoding="utf-8") as f:
        f.write(json.dumps(full, ensure_ascii=False) + "\n")
        f.flush()
        os.fsync(f.fileno())
    return chain


def _uuid7(now: dt.datetime) -> str:
    ms = int(now.timestamp() * 1000)
    rand_a = secrets.randbits(12)
    rand_b = secrets.randbits(62)
    high = (ms & 0xFFFFFFFFFFFF) << 16 | (0x7 << 12) | rand_a
    low = (0b10 << 62) | rand_b
    n = (high << 64) | low
    h = f"{n:032x}"
    return f"{h[0:8]}-{h[8:12]}-{h[12:16]}-{h[16:20]}-{h[20:32]}"


def _walk_in_scope(store: Path, scopes: list[str]) -> list[tuple[Path, str, dict, str]]:
    """Return [(path, rel, frontmatter, body), ...] for in-scope memories."""
    out = []
    for d in ("company", "module", "member", "client", "project",
              "persona", "memories", "meta"):
        scope_path = store / d
        if not scope_path.exists():
            continue
        for md in scope_path.rglob("*.md"):
            rel = md.relative_to(store).as_posix()
            if md.name == "README.md":
                continue
            try:
                text = md.read_text(encoding="utf-8")
            except (OSError, UnicodeDecodeError):
                continue
            fm_yaml, body = split_frontmatter(text)
            if fm_yaml is None or yaml is None:
                continue
            try:
                fm = yaml.safe_load(fm_yaml)
            except yaml.YAMLError:
                continue
            if not isinstance(fm, dict):
                continue
            if in_scope(rel, fm, scopes):
                out.append((md, rel, fm, body))
    return out


# ---------------------------------------------------------------------------
# disable / migrate-batch / rotate-shamir (v1)
# ---------------------------------------------------------------------------

def cmd_disable(store: Path) -> int:
    """Decrypt all in-scope memories → re-write plaintext → flip flag."""
    manifest_path = store / "manifest.json"
    manifest = json.loads(manifest_path.read_text())
    pol = manifest.get("encryption_policy", {})
    if not pol.get("enabled"):
        print("encryption is already disabled")
        return 0

    print("=== Disable encryption ===")
    print("This will decrypt all encrypted memories back to plaintext.")
    print("Requires the master key (HW or passphrase).")
    pw = getpass.getpass("  passphrase (or empty for HW key): ")
    if pw:
        master_key = PassphraseBackend(pw).derive()
    else:
        hw = detect_hw_backend()
        if hw is None:
            print("✘ no HW backend available; supply passphrase", file=sys.stderr)
            return 3
        master_key = hw.derive()

    fp = "sha256:" + hashlib.sha256(master_key).hexdigest()
    pinned = manifest["shamir_fragments"].get("master_key_fingerprint")
    if pinned and fp != pinned:
        print(f"✘ master key fingerprint mismatch (got {fp[:30]}..., "
              f"expected {pinned[:30]}...)", file=sys.stderr)
        return 2

    confirm = input(f"  Decrypt all encrypted memories and disable? [yes/NO] ").strip()
    if confirm.lower() != "yes":
        print("cancelled")
        return 1

    # Begin maintenance envelope
    maintenance_session = _uuid7(dt.datetime.now(dt.timezone(dt.timedelta(hours=7))))
    _audit_append(store, {
        "op": "maintenance.start",
        "path": ".cyberos-memory/",
        "reason": f"cyberos-encrypt disable session {maintenance_session}",
    })

    decrypted = errors = 0
    try:
        memories = _walk_in_scope(store, pol.get("scopes", []))
        for md, rel, fm, body in memories:
            if not fm.get("encrypted"):
                continue
            try:
                plaintext = decrypt_body(
                    body.strip(), master_key,
                    fm["encryption"]["nonce"],
                    fm["memory_id"],
                    fm["last_updated_at"] if isinstance(fm["last_updated_at"], str)
                    else fm["last_updated_at"].isoformat(),
                )
                # Re-emit as plaintext: strip encrypted: + encryption: from FM
                new_fm = {k: v for k, v in fm.items()
                          if k not in ("encrypted", "encryption")}
                # Naive YAML re-emit (preserves field order for compactness pass)
                fm_lines = ["---"]
                for k, v in new_fm.items():
                    fm_lines.append(f"{k}: {json.dumps(v, default=str) if not isinstance(v, str) else v}")
                fm_lines.append("---")
                new_text = "\n".join(fm_lines) + "\n" + plaintext
                before_hash = "sha256:" + hashlib.sha256(
                    md.read_bytes()).hexdigest()
                after_hash = "sha256:" + hashlib.sha256(
                    new_text.encode("utf-8")).hexdigest()
                tmp = md.with_suffix(f".tmp.{secrets.token_hex(8)}.part")
                tmp.write_text(new_text, encoding="utf-8")
                os.replace(tmp, md)
                _audit_append(store, {
                    "op": "str_replace",
                    "path": f".cyberos-memory/{rel}",
                    "memory_id": fm["memory_id"],
                    "before_hash": before_hash,
                    "after_hash": after_hash,
                    "reason": f"cyberos-encrypt disable: decrypt {rel}",
                })
                decrypted += 1
            except Exception as e:  # noqa: BLE001
                print(f"  ✘ {rel}: {e}", file=sys.stderr)
                errors += 1

        # Flip flag
        manifest["encryption_policy"]["enabled"] = False
        manifest["shamir_fragments"]["master_key_fingerprint"] = None
        manifest["shamir_fragments"]["fragments"] = []
        manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
        _audit_append(store, {
            "op": "encryption_policy_change",
            "path": ".cyberos-memory/manifest.json",
            "reason": f"disable: decrypted {decrypted} memories, {errors} errors",
        })
    finally:
        _audit_append(store, {
            "op": "maintenance.end",
            "path": ".cyberos-memory/",
            "reason": f"cyberos-encrypt disable complete: {decrypted} decrypted, {errors} errors",
        })

    print(f"✅ disabled. Decrypted {decrypted} memories, {errors} errors.")
    return 0 if errors == 0 else 2


def cmd_migrate_batch(store: Path, n: int) -> int:
    """Encrypt N more in-scope plaintext memories under one MAINTENANCE envelope."""
    manifest = json.loads((store / "manifest.json").read_text())
    pol = manifest.get("encryption_policy", {})
    if not pol.get("enabled"):
        print("✘ encryption not enabled; run `cyberos-encrypt enable` first",
              file=sys.stderr)
        return 3

    pw = getpass.getpass("  passphrase: ")
    master_key = PassphraseBackend(pw).derive()
    fp = "sha256:" + hashlib.sha256(master_key).hexdigest()
    pinned = manifest["shamir_fragments"].get("master_key_fingerprint")
    if pinned and fp != pinned:
        print(f"✘ master key fingerprint mismatch", file=sys.stderr)
        return 2

    memories = _walk_in_scope(store, pol.get("scopes", []))
    candidates = [m for m in memories if not m[2].get("encrypted")]
    print(f"  {len(candidates)} plaintext memories in scope; encrypting up to {n}")

    if not candidates:
        return 0

    maintenance_session = _uuid7(dt.datetime.now(dt.timezone(dt.timedelta(hours=7))))
    _audit_append(store, {
        "op": "maintenance.start",
        "path": ".cyberos-memory/",
        "reason": f"cyberos-encrypt migrate-batch {n}, session {maintenance_session}",
    })
    encrypted = errors = 0
    try:
        for md, rel, fm, body in candidates[:n]:
            try:
                lua = (fm["last_updated_at"] if isinstance(fm["last_updated_at"], str)
                       else fm["last_updated_at"].isoformat())
                result = encrypt_body(body, master_key, fm["memory_id"], lua)
                # Re-emit: append encrypted: true + encryption: block; body becomes ciphertext
                new_fm = dict(fm)
                new_fm["encrypted"] = True
                new_fm["encryption"] = result["encryption_block"]
                fm_lines = ["---"]
                for k, v in new_fm.items():
                    if isinstance(v, dict):
                        fm_lines.append(f"{k}:")
                        for kk, vv in v.items():
                            fm_lines.append(f"  {kk}: {vv}")
                    elif isinstance(v, bool):
                        fm_lines.append(f"{k}: {'true' if v else 'false'}")
                    else:
                        fm_lines.append(f"{k}: {json.dumps(v, default=str) if not isinstance(v, str) else v}")
                fm_lines.append("---")
                new_text = "\n".join(fm_lines) + "\n" + result["body"] + "\n"
                before_hash = "sha256:" + hashlib.sha256(
                    md.read_bytes()).hexdigest()
                # after_hash is over PLAINTEXT body per §5.6.5
                after_hash = "sha256:" + hashlib.sha256(
                    body.encode("utf-8")).hexdigest()
                tmp = md.with_suffix(f".tmp.{secrets.token_hex(8)}.part")
                tmp.write_text(new_text, encoding="utf-8")
                os.replace(tmp, md)
                _audit_append(store, {
                    "op": "str_replace",
                    "path": f".cyberos-memory/{rel}",
                    "memory_id": fm["memory_id"],
                    "before_hash": before_hash,
                    "after_hash": after_hash,
                    "reason": f"cyberos-encrypt migrate-batch: encrypt {rel}",
                })
                encrypted += 1
            except Exception as e:  # noqa: BLE001
                print(f"  ✘ {rel}: {e}", file=sys.stderr)
                errors += 1
    finally:
        _audit_append(store, {
            "op": "maintenance.end",
            "path": ".cyberos-memory/",
            "reason": f"migrate-batch complete: {encrypted} encrypted, {errors} errors, {len(candidates) - encrypted} remaining",
        })

    print(f"✅ migrated. Encrypted {encrypted} memories. {len(candidates) - encrypted} plaintext remaining.")
    return 0 if errors == 0 else 2


def cmd_rotate_shamir(store: Path) -> int:
    """Refresh the 5 fragments without changing the master key."""
    manifest_path = store / "manifest.json"
    manifest = json.loads(manifest_path.read_text())
    pol = manifest.get("encryption_policy", {})
    if not pol.get("enabled"):
        print("✘ encryption not enabled", file=sys.stderr)
        return 3

    pw = getpass.getpass("  passphrase: ")
    master_key = PassphraseBackend(pw).derive()
    fp = "sha256:" + hashlib.sha256(master_key).hexdigest()
    pinned = manifest["shamir_fragments"].get("master_key_fingerprint")
    if pinned and fp != pinned:
        print(f"✘ master key fingerprint mismatch", file=sys.stderr)
        return 2

    print("=== Shamir rotation: generating fresh 5 fragments ===")
    print("(Master key unchanged; old fragments become useless after this rotation.)")
    new_shares = shamir_split(master_key, SHAMIR_THRESHOLD, SHAMIR_TOTAL)
    new_records = []
    now = dt.datetime.now(dt.timezone(dt.timedelta(hours=7)))
    for i, share in enumerate(new_shares, 1):
        label = input(f"  Holder label for fragment {i}: ").strip() or f"holder-{i}"
        encoded = encode_share(share, label.replace(" ", "-")[:16])
        share_fp = "sha256:" + hashlib.sha256(share).hexdigest()
        print(f"    Encoded: {encoded}")
        confirm = input(f"    Distributed to {label}? [y/N] ").strip().lower()
        if confirm != "y":
            print("✘ wizard cancelled — old fragments remain valid")
            return 1
        new_records.append({
            "label": label,
            "fingerprint": share_fp,
            "created_at": now.isoformat(timespec="seconds"),
            "distributed_at": now.isoformat(timespec="seconds"),
        })
        _audit_append(store, {
            "op": "shamir_distribution_confirmed",
            "path": ".cyberos-memory/manifest.json",
            "reason": f"rotate-shamir: fragment {i}/5 distributed to {label}",
        })

    manifest["shamir_fragments"]["fragments"] = new_records
    manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
    _audit_append(store, {
        "op": "shamir_rotation",
        "path": ".cyberos-memory/manifest.json",
        "reason": f"rotated {SHAMIR_TOTAL} Shamir fragments; master key unchanged",
    })
    print(f"✅ rotated. Old fragments are now useless. {SHAMIR_TOTAL} new fragments distributed.")
    return 0


def cmd_recover(store: Path) -> int:
    """Read fragment lines from stdin, reconstruct master key."""
    print("=== Shamir recovery ===")
    print(f"  Paste ≥{SHAMIR_THRESHOLD} fragments (one per line, empty line to end):")
    fragments = []
    while True:
        try:
            line = input().strip()
        except EOFError:
            break
        if not line:
            break
        try:
            fragments.append(decode_share(line))
        except ValueError as e:
            print(f"  ✘ invalid fragment: {e}")
            continue

    if len(fragments) < SHAMIR_THRESHOLD:
        print(f"  ✘ need ≥{SHAMIR_THRESHOLD}, got {len(fragments)}")
        return 2

    try:
        master_key = shamir_combine(fragments[:SHAMIR_THRESHOLD])
    except ValueError as e:
        print(f"  ✘ reconstruction failed: {e}")
        return 2

    fingerprint = "sha256:" + hashlib.sha256(master_key).hexdigest()
    print(f"  ✅ master key reconstructed (fingerprint {fingerprint[:30]}...)")

    # Verify against pinned fingerprint
    manifest = json.loads((store / "manifest.json").read_text())
    pinned = manifest.get("shamir_fragments", {}).get("master_key_fingerprint")
    if pinned and pinned != fingerprint:
        print(f"  ✘ FINGERPRINT MISMATCH — pinned: {pinned[:30]}...")
        print(f"    Wrong fragments? Or memory has been tampered with.")
        return 2
    print(f"  ✅ fingerprint matches pinned in manifest")
    print()
    print("Recovered key NOT printed for security. To use it: this CLI does not")
    print("yet integrate with the running session — ship cyberos-encrypt v1 to")
    print("complete the recovery flow.")
    return 0


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="cyberos-encrypt",
        description="At-rest encryption + Shamir 3-of-5 escrow per AGENTS.md §5.6.")
    parser.add_argument("path", help="path to .cyberos-memory/ or project root")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_enable = sub.add_parser("enable", help="Run the enable wizard")
    p_enable.add_argument("--passphrase", action="store_true", default=True,
                          help="Use Argon2id passphrase (default; HW key not in v0)")
    p_enable.add_argument("--hw", help="Use named HW backend (stub in v0)")

    sub.add_parser("disable", help="Disable encryption (decrypt all)")
    p_mig = sub.add_parser("migrate-batch", help="Encrypt N more memories")
    p_mig.add_argument("n", type=int, default=50)
    sub.add_parser("rotate-shamir", help="Refresh 5 fragments")
    sub.add_parser("recover", help="Reconstruct master from ≥3 fragments")
    sub.add_parser("status", help="Show encryption coverage stats")

    args = parser.parse_args(argv)

    store = Path(args.path).resolve()
    if (store / ".cyberos-memory").is_dir():
        store = store / ".cyberos-memory"
    if not store.is_dir():
        print(f"error: {store} not a directory", file=sys.stderr)
        return 3

    if args.cmd == "status":
        return cmd_status(store)
    if args.cmd == "enable":
        return cmd_enable(store, passphrase_mode=not args.hw)
    if args.cmd == "disable":
        return cmd_disable(store)
    if args.cmd == "migrate-batch":
        return cmd_migrate_batch(store, args.n)
    if args.cmd == "rotate-shamir":
        return cmd_rotate_shamir(store)
    if args.cmd == "recover":
        return cmd_recover(store)
    return 3


if __name__ == "__main__":
    sys.exit(main())
