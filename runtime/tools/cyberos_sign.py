#!/usr/bin/env python3
"""
cyberos_sign.py — Ed25519 signing of protocol-history snapshots.

Batch 14 (Tier D) of post-catalog improvements.

Every §0.5 protocol upgrade archives the prior protocol text under
`.cyberos-memory/meta/protocol-history/AGENTS-sha256-<sha>.md`. This tool
signs each archive with an Ed25519 keypair and writes a detached
signature next to it. Validator verifies the signature on every verify.

Key management:
  - Private key: `~/.cyberos/keys/protocol-signing.ed25519` (operator-owned)
  - Public key:  `.cyberos-memory/meta/protocol-signing-pubkey.ed25519`
    (committed; everyone can verify)

Subcommands:
  cyberos sign keygen              # one-time: generate keypair
  cyberos sign sign <snapshot>     # sign a single protocol-history file
  cyberos sign verify <snapshot>   # verify signature
  cyberos sign verify-all          # verify every snapshot in protocol-history/

Requires: `pip install cryptography`. Degrades to a stub if missing.
"""
from __future__ import annotations
import argparse
import sys
from pathlib import Path


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def have_crypto():
    try:
        from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey, Ed25519PublicKey  # noqa
        return True
    except ImportError:
        return False


def priv_key_path() -> Path:
    return Path.home() / ".cyberos" / "keys" / "protocol-signing.ed25519"


def pub_key_path(brain_root: Path) -> Path:
    return brain_root / ".cyberos-memory" / "meta" / "protocol-signing-pubkey.ed25519"


def cmd_keygen(_args):
    if not have_crypto():
        print("  ✗ cryptography library not installed: `pip install cryptography`", file=sys.stderr)
        return 3
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    from cryptography.hazmat.primitives import serialization

    brain_root = find_brain()
    priv = priv_key_path()
    if priv.exists():
        print(f"  ✗ private key already exists at {priv}; refuse to overwrite", file=sys.stderr)
        return 2
    priv.parent.mkdir(parents=True, exist_ok=True)
    sk = Ed25519PrivateKey.generate()
    priv.write_bytes(sk.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.PKCS8,
        encryption_algorithm=serialization.NoEncryption(),
    ))
    priv.chmod(0o600)
    pub = pub_key_path(brain_root)
    pub.parent.mkdir(parents=True, exist_ok=True)
    pub.write_bytes(sk.public_key().public_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PublicFormat.SubjectPublicKeyInfo,
    ))
    print(f"  ✓ keypair generated:")
    print(f"    private: {priv} (mode 600)")
    print(f"    public:  {pub} (commit this to the BRAIN)")
    return 0


def _load_priv():
    from cryptography.hazmat.primitives import serialization
    return serialization.load_pem_private_key(priv_key_path().read_bytes(), password=None)


def _load_pub(brain_root: Path):
    from cryptography.hazmat.primitives import serialization
    return serialization.load_pem_public_key(pub_key_path(brain_root).read_bytes())


def cmd_sign(args):
    if not have_crypto():
        print("  ✗ cryptography library not installed: `pip install cryptography`", file=sys.stderr)
        return 3
    if not priv_key_path().exists():
        print(f"  ✗ no private key at {priv_key_path()}; run `cyberos sign keygen` first", file=sys.stderr)
        return 2
    snap = Path(args.snapshot)
    if not snap.exists():
        print(f"  ✗ no such snapshot: {snap}", file=sys.stderr)
        return 2
    sk = _load_priv()
    sig = sk.sign(snap.read_bytes())
    sig_path = snap.with_suffix(snap.suffix + ".sig")
    sig_path.write_bytes(sig)
    print(f"  ✓ signature: {sig_path} ({len(sig)} bytes)")
    return 0


def cmd_verify(args):
    if not have_crypto():
        print("  ✗ cryptography library not installed", file=sys.stderr)
        return 3
    brain_root = find_brain()
    if not pub_key_path(brain_root).exists():
        print(f"  ✗ no public key at {pub_key_path(brain_root)}", file=sys.stderr)
        return 2
    pk = _load_pub(brain_root)
    snap = Path(args.snapshot)
    sig_path = snap.with_suffix(snap.suffix + ".sig")
    if not sig_path.exists():
        print(f"  ✗ no signature alongside snapshot: {sig_path}", file=sys.stderr)
        return 2
    from cryptography.exceptions import InvalidSignature
    try:
        pk.verify(sig_path.read_bytes(), snap.read_bytes())
        print(f"  ✓ signature valid: {snap.name}")
        return 0
    except InvalidSignature:
        print(f"  ✗ signature INVALID: {snap.name}")
        return 1


def cmd_verify_all(args):
    brain_root = find_brain()
    hist = brain_root / ".cyberos-memory" / "meta" / "protocol-history"
    if not hist.exists():
        print("  no protocol-history/ dir")
        return 0
    snaps = sorted(hist.glob("AGENTS-sha256-*.md"))
    if not snaps:
        print("  no snapshots to verify")
        return 0
    failed = 0
    for snap in snaps:
        args.snapshot = str(snap)
        rc = cmd_verify(args)
        if rc != 0:
            failed += 1
    print(f"\n  {len(snaps) - failed}/{len(snaps)} verified")
    return 1 if failed else 0


def main():
    p = argparse.ArgumentParser(description="Ed25519 signing for protocol-history snapshots (Batch 14 / Tier D)")
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("keygen").set_defaults(func=cmd_keygen)
    ps = sub.add_parser("sign"); ps.add_argument("snapshot"); ps.set_defaults(func=cmd_sign)
    pv = sub.add_parser("verify"); pv.add_argument("snapshot"); pv.set_defaults(func=cmd_verify)
    pa = sub.add_parser("verify-all"); pa.set_defaults(func=cmd_verify_all)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
