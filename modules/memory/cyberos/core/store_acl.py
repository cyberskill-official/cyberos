"""
cyberos.core.store_acl — per-subtree write ACL (TASK-MEMORY-117, AGENTS.md §14.4).

The ACL layer sits between the canonical Writer and the filesystem. Every
``put`` / ``move`` / ``delete`` consults the nearest ``STORE.yaml`` walking
UP from the target path; the first match wins.

Three modes (per AGENTS.md §14.4 — closed enum):

* ``read-write`` — the actor may write to this subtree.
* ``read`` — writes are refused; the body remains readable via OS FS perms.
* ``deny`` — explicit block; takes precedence over later allow entries.

The ``read`` vs ``deny`` distinction is mostly diagnostic: both refuse
writes, but ``deny`` signals operator intent ("this actor MUST NEVER
write here") versus ``read`` ("this subtree is read-only for everyone
matching this entry").

WARN-ONLY mode (per §14.4.4): when the AGENTS.md §14.4 anchor is absent
in the project, the resolver still emits ``memory.acl_denied`` rows but
returns ``allowed=True`` so writes proceed. This is the anti-footgun
transition state — operators who pull this code before APPROVE'ing the
amendment shouldn't have writes silently blocked overnight.
"""

from __future__ import annotations

import fnmatch
from dataclasses import dataclass
from pathlib import Path
from typing import Literal, Optional

Mode = Literal["read", "read-write", "deny"]


@dataclass(frozen=True)
class StoreAcl:
    """Parsed STORE.yaml content."""

    store_id: str
    default_mode: Mode
    acl: tuple[tuple[str, Mode], ...]  # ordered (pattern, mode)

    @classmethod
    def from_yaml(cls, path: Path) -> "StoreAcl":
        """Parse STORE.yaml at the given absolute path.

        Raises ValueError on malformed shape (caller decides whether to
        propagate or fall back to permissive default).
        """
        # Lazy import — many operators don't have PyYAML installed.
        import yaml

        try:
            raw = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
        except yaml.YAMLError as e:
            raise ValueError(f"{path}: invalid YAML: {e}") from e
        if not isinstance(raw, dict):
            raise ValueError(f"{path}: must be a YAML object, got {type(raw).__name__}")

        store_id = raw.get("store_id")
        if not isinstance(store_id, str) or not store_id:
            raise ValueError(f"{path}: missing or empty `store_id`")

        default_mode = raw.get("default_mode", "read-write")
        if default_mode not in ("read", "read-write", "deny"):
            raise ValueError(
                f"{path}: default_mode={default_mode!r} not in closed enum"
            )

        entries = raw.get("acl", [])
        if not isinstance(entries, list):
            raise ValueError(f"{path}: `acl` must be a list")
        parsed_entries: list[tuple[str, Mode]] = []
        for i, e in enumerate(entries):
            if not isinstance(e, dict):
                raise ValueError(f"{path}: acl[{i}] must be an object")
            actor = e.get("actor")
            mode = e.get("mode")
            if not isinstance(actor, str) or not actor:
                raise ValueError(f"{path}: acl[{i}] missing or empty `actor`")
            if mode not in ("read", "read-write", "deny"):
                raise ValueError(
                    f"{path}: acl[{i}].mode={mode!r} not in closed enum"
                )
            parsed_entries.append((actor, mode))

        return cls(
            store_id=store_id,
            default_mode=default_mode,
            acl=tuple(parsed_entries),
        )

    def resolve_mode(self, actor: str) -> Mode:
        """First-match-wins glob resolution; default_mode on no match."""
        for pattern, mode in self.acl:
            if pattern == "*" or pattern == actor or fnmatch.fnmatchcase(actor, pattern):
                return mode
        return self.default_mode


@dataclass(frozen=True)
class AclResult:
    """Outcome of a write-side ACL check."""

    allowed: bool
    mode: Mode
    store_id: Optional[str]
    yaml_path: Optional[str]            # relative to memory_root
    matched_entry: Optional[str]
    reason: Optional[str]               # populated on rejection or warn-only

    def to_aux_payload(
        self, actor: str, target_path: str, attempt_kind: str
    ) -> dict:
        """Render the TASK-MEMORY-117 §1 #7 / AGENTS.md §14.4.4 aux row shape."""
        return {
            "actor": actor,
            "target_path": target_path,
            "store_id": self.store_id,
            "yaml_path": self.yaml_path,
            "mode": self.mode,
            "matched_entry": self.matched_entry,
            "attempt_kind": attempt_kind,
            "warn_only": self.reason is not None and self.reason.startswith("warn_only:"),
        }


def find_governing_store_yaml(
    memory_root: Path, target_rel_path: str
) -> Optional[Path]:
    """Walk UP from target's parent dir until we find STORE.yaml or hit memory_root.

    The first STORE.yaml encountered governs. Further-up STORE.yaml files
    are ignored — innermost wins (per AGENTS.md §14.4.2).
    """
    memory_root = memory_root.resolve()
    abs_target = (memory_root / target_rel_path).resolve()
    # Start from the target's parent dir
    current = abs_target.parent
    # Walk up until we leave memory_root
    while True:
        try:
            current.relative_to(memory_root)
        except ValueError:
            break
        candidate = current / "STORE.yaml"
        if candidate.exists():
            return candidate
        if current == memory_root:
            break
        current = current.parent
    return None


def _has_section_14_4(memory_root: Path) -> bool:
    """Anchor check for the WARN-ONLY transition state."""
    candidates: list[Path] = [memory_root / "AGENTS.md"]
    for parent in [memory_root, *memory_root.parents][:6]:
        candidates.append(parent / "modules" / "memory" / "AGENTS.md")
    for c in candidates:
        if c.exists():
            try:
                body = c.read_text(encoding="utf-8", errors="ignore")
            except Exception:
                continue
            if "§14.4" in body and "Store-level ACL" in body:
                return True
    return False


def check_write(
    memory_root: Path,
    target_rel_path: str,
    actor: str,
) -> AclResult:
    """Resolve the effective ACL for ``actor`` writing to ``target_rel_path``.

    Returns an :class:`AclResult`. In WARN-ONLY mode (when §14.4 isn't
    anchored in AGENTS.md), the result's ``allowed=True`` even when the
    resolved mode is ``read`` / ``deny`` — but ``reason`` is set to
    ``warn_only:mode=<mode>`` so callers know to emit a
    ``memory.acl_denied`` aux row.
    """
    yml = find_governing_store_yaml(memory_root, target_rel_path)
    warn_only = not _has_section_14_4(memory_root)

    if yml is None:
        # No STORE.yaml found anywhere up the tree → permissive default
        return AclResult(
            allowed=True,
            mode="read-write",
            store_id=None,
            yaml_path=None,
            matched_entry=None,
            reason=None,
        )

    try:
        acl = StoreAcl.from_yaml(yml)
    except (ValueError, ImportError) as e:
        # Malformed STORE.yaml — fail safe (refuse writes, surface error)
        # In WARN-ONLY mode, still allow but with reason=warn_only.
        rel_yml = str(yml.relative_to(memory_root))
        rel_yml_reason = f"malformed_store_yaml:{rel_yml}:{e}"
        return AclResult(
            allowed=warn_only,
            mode="deny",
            store_id=None,
            yaml_path=rel_yml,
            matched_entry=None,
            reason=(f"warn_only:{rel_yml_reason}" if warn_only else rel_yml_reason),
        )

    mode = acl.resolve_mode(actor)
    rel_yml = str(yml.relative_to(memory_root))

    if mode == "read-write":
        return AclResult(
            allowed=True,
            mode=mode,
            store_id=acl.store_id,
            yaml_path=rel_yml,
            matched_entry=f"actor={actor!r} → mode={mode!r}",
            reason=None,
        )

    # mode is "read" or "deny"
    reason = f"acl_denied:mode={mode}"
    if warn_only:
        reason = f"warn_only:mode={mode}"
    return AclResult(
        allowed=warn_only,
        mode=mode,
        store_id=acl.store_id,
        yaml_path=rel_yml,
        matched_entry=f"actor={actor!r} → mode={mode!r}",
        reason=reason,
    )


def explain(
    memory_root: Path, target_rel_path: str, actor: str
) -> dict:
    """Operator-readable diagnostic — used by `cyberos acl explain <path>`."""
    res = check_write(memory_root, target_rel_path, actor)
    return {
        "path": target_rel_path,
        "actor": actor,
        "yaml_path": res.yaml_path,
        "store_id": res.store_id,
        "effective_mode": res.mode,
        "matched_entry": res.matched_entry,
        "allowed_write": res.allowed,
        "warn_only_active": res.reason is not None and res.reason.startswith("warn_only:"),
        "reason": res.reason,
    }
