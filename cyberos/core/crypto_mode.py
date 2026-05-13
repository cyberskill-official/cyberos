"""
cyberos.core.crypto_mode — P2 Stage 3 feature flag.

A store's manifest declares one of:

* ``"chained"`` (default) — every audit row carries ``prev_chain`` and
  ``chain`` SHA-256 fields; ``cyberos verify`` re-hashes every row;
  the per-row chain is the integrity primitive. P2 Stage 1 added an
  additive MMR that runs alongside the chain but does not replace it.

* ``"sth_only"`` (opt-in) — the per-row chain still gets COMPUTED (so
  the on-disk binlog format is identical), but verification now treats
  the MMR + Signed Tree Heads as canonical. The doctor ``ledger-link``
  / ``ledger-hash`` invariants become advisory only — green by default,
  surface a divergence as a *warning* not an error. The canonical
  integrity check is ``ledger-mmr-cross-check`` plus the latest STH's
  Ed25519 signature.

This module is the registry for that flag. Three functions:

* :func:`current_mode` — peek manifest, return ``"chained"`` (default) or ``"sth_only"``.
* :func:`upgrade_to_sth_only` — flip manifest atomically. Requires the
  approval phrase (per AGENTS.md §0.2): ``APPROVE protocol change P2 §6
  Stage 3 (chain primitive swap to MMR + STH)``. Also requires the store
  to have at least one persisted STH (proving the MMR has been signed
  at least once) and the doctor's MMR cross-check to be currently green.
* :func:`downgrade_to_chained` — back-rollback. Only safe if the chain
  was never broken in sth_only mode (since chain rows kept being written,
  this is mechanically straightforward).
"""

from __future__ import annotations

import json
import time
from pathlib import Path

# AGENTS.md §0.2 magic phrase for Stage 3 promotion.
APPROVAL_PHRASE: str = (
    "APPROVE protocol change P2 §6 Stage 3 (chain primitive swap to MMR + STH)"
)


class CryptoModeError(RuntimeError):
    """Raised when a mode transition is refused for safety reasons."""


# ---------------------------------------------------------------------------
# read / write the manifest field
# ---------------------------------------------------------------------------


def _manifest_path(store: Path) -> Path:
    return store / "manifest.json"


def current_mode(store: Path) -> str:
    """Return ``"chained"`` or ``"sth_only"``. Missing field → ``"chained"``."""
    path = _manifest_path(store)
    if not path.is_file():
        return "chained"
    try:
        manifest = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return "chained"
    return manifest.get("crypto_mode", "chained")


def _write_manifest_atomic(path: Path, manifest: dict) -> None:
    """Atomic write: tmp + rename. Preserves byte-equivalence on re-runs."""
    tmp = path.with_suffix(".tmp")
    payload = json.dumps(manifest, indent=2, sort_keys=True).encode("utf-8")
    tmp.write_bytes(payload)
    tmp.replace(path)


# ---------------------------------------------------------------------------
# upgrade
# ---------------------------------------------------------------------------


def _has_persisted_sth(store: Path) -> tuple[bool, str]:
    """True iff at least one STH exists under audit/sth/."""
    sth_dir = store / "audit" / "sth"
    if not sth_dir.is_dir():
        return False, "no audit/sth/ directory — STH never signed"
    sths = sorted(sth_dir.glob("*.json"))
    if not sths:
        return False, "audit/sth/ is empty — no signed tree heads on disk"
    return True, f"{len(sths)} STH(s) on disk"


def _mmr_cross_check_green(store: Path) -> tuple[bool, str]:
    """Re-run the MMR cross-check invariant and return (passed, details)."""
    from cyberos.core.invariants import check_ledger_mmr_cross_check
    return check_ledger_mmr_cross_check(store)


def upgrade_to_sth_only(
    store: Path,
    *,
    approval_phrase: str,
    skip_safety_checks: bool = False,
) -> dict:
    """Flip ``crypto_mode`` from ``"chained"`` to ``"sth_only"``.

    Refuses unless:

    1. ``approval_phrase`` matches :data:`APPROVAL_PHRASE` exactly;
    2. at least one STH has been persisted (proving signing works on this host);
    3. the MMR cross-check invariant is currently passing.

    The second and third checks can be bypassed via ``skip_safety_checks``
    for migration scripts that have already validated them externally —
    but this is NEVER recommended for ad-hoc operator use.

    Returns a summary dict (also embedded in the manifest for audit).
    """
    if approval_phrase != APPROVAL_PHRASE:
        raise CryptoModeError(
            "approval_phrase does not match. To upgrade, cite verbatim:\n"
            f"  {APPROVAL_PHRASE}"
        )

    if not skip_safety_checks:
        ok, msg = _has_persisted_sth(store)
        if not ok:
            raise CryptoModeError(
                f"safety check 1 failed: {msg}. "
                "Run `cyberos consolidate` first to produce a signed tree head."
            )
        ok, msg = _mmr_cross_check_green(store)
        if not ok:
            raise CryptoModeError(
                f"safety check 2 failed: MMR cross-check is currently red — "
                f"{msg}. Resolve the divergence before promoting Stage 3."
            )

    path = _manifest_path(store)
    if not path.is_file():
        raise CryptoModeError(
            f"no manifest.json at {path} — this is not a cyberos store"
        )
    manifest = json.loads(path.read_text(encoding="utf-8"))
    previous_mode = manifest.get("crypto_mode", "chained")
    if previous_mode == "sth_only":
        return {
            "status": "already-upgraded",
            "previous_mode": previous_mode,
            "current_mode": "sth_only",
        }

    now_ns = time.time_ns()
    manifest["crypto_mode"] = "sth_only"
    history = manifest.setdefault("crypto_mode_history", [])
    history.append({
        "from": previous_mode,
        "to": "sth_only",
        "at_ns": now_ns,
        "approval_phrase": approval_phrase,
    })
    _write_manifest_atomic(path, manifest)

    return {
        "status": "upgraded",
        "previous_mode": previous_mode,
        "current_mode": "sth_only",
        "at_ns": now_ns,
    }


def downgrade_to_chained(
    store: Path,
    *,
    approval_phrase: str,
) -> dict:
    """Roll back ``crypto_mode`` to ``"chained"``.

    Safe because Stage 3 keeps writing per-row chain hashes; flipping
    back simply restores authoritative verification to the chain. Still
    requires the same approval phrase so it's deliberate.
    """
    if approval_phrase != APPROVAL_PHRASE:
        raise CryptoModeError(
            "approval_phrase does not match (same phrase as upgrade). "
            f"Cite verbatim: {APPROVAL_PHRASE}"
        )

    path = _manifest_path(store)
    if not path.is_file():
        raise CryptoModeError(
            f"no manifest.json at {path} — this is not a cyberos store"
        )
    manifest = json.loads(path.read_text(encoding="utf-8"))
    previous_mode = manifest.get("crypto_mode", "chained")
    if previous_mode == "chained":
        return {
            "status": "already-chained",
            "previous_mode": previous_mode,
            "current_mode": "chained",
        }

    now_ns = time.time_ns()
    manifest["crypto_mode"] = "chained"
    history = manifest.setdefault("crypto_mode_history", [])
    history.append({
        "from": previous_mode,
        "to": "chained",
        "at_ns": now_ns,
        "approval_phrase": approval_phrase,
    })
    _write_manifest_atomic(path, manifest)

    return {
        "status": "downgraded",
        "previous_mode": previous_mode,
        "current_mode": "chained",
        "at_ns": now_ns,
    }


# ---------------------------------------------------------------------------
# invariant adjustment helper
# ---------------------------------------------------------------------------


def is_sth_only(store: Path) -> bool:
    """Convenience predicate used by walker/doctor branches."""
    return current_mode(store) == "sth_only"


__all__ = [
    "APPROVAL_PHRASE",
    "CryptoModeError",
    "current_mode",
    "is_sth_only",
    "upgrade_to_sth_only",
    "downgrade_to_chained",
]
