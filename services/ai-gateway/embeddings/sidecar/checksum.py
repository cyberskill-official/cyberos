"""Checksum verification for the BGE-M3 sidecar."""

from __future__ import annotations

import hashlib
from pathlib import Path


class ChecksumMismatch(RuntimeError):
    """Raised when the mounted model artefact does not match the pinned hash."""


def read_expected_sha256(path: str | Path) -> str:
    """Read the first non-comment checksum token from a checksum file."""
    for line in Path(path).read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped and not stripped.startswith("#"):
            token = stripped.split()[0].lower()
            if len(token) != 64 or any(c not in "0123456789abcdef" for c in token):
                raise ValueError(f"invalid sha256 in {path}: {token!r}")
            return token
    raise ValueError(f"no sha256 found in {path}")


def sha256_path(path: str | Path) -> str:
    """Hash a file or a directory tree deterministically."""
    target = Path(path)
    h = hashlib.sha256()
    if target.is_file():
        with target.open("rb") as fh:
            for chunk in iter(lambda: fh.read(1024 * 1024), b""):
                h.update(chunk)
        return h.hexdigest()

    if not target.is_dir():
        raise FileNotFoundError(target)

    for child in sorted(p for p in target.rglob("*") if p.is_file()):
        rel = child.relative_to(target).as_posix().encode("utf-8")
        h.update(rel)
        h.update(b"\0")
        with child.open("rb") as fh:
            for chunk in iter(lambda: fh.read(1024 * 1024), b""):
                h.update(chunk)
        h.update(b"\0")
    return h.hexdigest()


def verify_model_checksum(model_path: str | Path, checksum_path: str | Path) -> str:
    """Return the verified full sha256, or raise ChecksumMismatch."""
    expected = read_expected_sha256(checksum_path)
    actual = sha256_path(model_path)
    if actual != expected:
        raise ChecksumMismatch(f"expected={expected} actual={actual}")
    return actual
