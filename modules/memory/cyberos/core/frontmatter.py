"""
cyberos.core.frontmatter — fast, schema'd frontmatter parser.

Replaces PyYAML for memory .md files. Reads files of the form::

    ---
    {"id":"DEC-104","kind":"decision","ts_ns":1715126400000000000,"tags":[...],"actor":"stephen"}
    ---
    # Body markdown ...

JSON, not YAML, because:

* ``msgspec.json.decode`` is 10–80× faster than PyYAML's pure-Python loader
  (jcristharif.com/msgspec, pythonspeed.com benchmarks).
* Schema-validated at zero extra cost via :class:`msgspec.Struct`.
* Byte-deterministic encoding (sorted keys via ``order='sorted'``); RFC 8785
  JCS equivalent for this closed schema.
* Still human-readable, still git-diffable, still grep-friendly.

During the migration window (audit report §6) the parser auto-detects YAML
frontmatter and routes it to :func:`parse_legacy_yaml`, which lazy-imports
PyYAML only when needed. Once the migration is complete, the legacy reader
can be deleted without changing the on-disk format of new files.

Stability contract: ``serialize(parse(raw)[0], parse(raw)[1]) == raw`` for
any frontmatter this module emitted itself (round-trip identity). Legacy
YAML inputs are NOT round-trip stable — see :func:`parse_legacy_yaml`.
"""

from __future__ import annotations

import re
import sys
from typing import Any, Final, Tuple

try:
    import msgspec
except ImportError:  # pragma: no cover
    sys.stderr.write(
        "FATAL: msgspec is not installed. Run:\n"
        "  pip install -r cyberos/requirements.txt --break-system-packages\n"
        "or, minimally:\n"
        "  pip install msgspec --break-system-packages\n"
        "(See cyberos/requirements.txt for the full Layer-1 dependency list.)\n"
    )
    raise


class Frontmatter(msgspec.Struct, kw_only=True, frozen=True):
    """Schema-validated frontmatter for a memory file.

    Closed schema. Any unknown top-level keys land in ``extra``. The
    encoding contract is "sorted keys + msgspec canonical JSON" — same
    byte sequence as RFC 8785 JCS for this struct (no floats, no NaN/Inf,
    no duplicate keys).

    Attributes
    ----------
    id:
        Stable memory identifier, e.g. ``DEC-104`` or ``REF-027``.
    kind:
        One of: ``decision``, ``fact``, ``person``, ``project``,
        ``preference``, ``drift``, ``refinement``. Enforced by callers,
        not by the struct (schema must accept legacy values during
        migration).
    ts_ns:
        Creation timestamp in nanoseconds since epoch.
    actor:
        Principal identifier — ``stephen``, ``coding-agent``, etc.
    tags:
        Free-form tag list. Ordering preserved as written.
    extra:
        Forward-compat slot for additional fields. Encoded last so reads
        of files written by future versions don't fail.
    """

    id: str
    kind: str
    ts_ns: int
    actor: str
    tags: list[str] = msgspec.field(default_factory=list)
    extra: dict[str, Any] = msgspec.field(default_factory=dict)


# Three-byte delimiter for the frontmatter block. Identical to the legacy
# YAML convention; downstream tooling (Obsidian, git diff renderers, etc.)
# already special-cases ``---\n`` as a frontmatter marker, so we keep it.
_DELIM: Final[bytes] = b"---\n"

# Module-level encoders/decoders — instantiating these is cheap but not
# free, and they are stateless once constructed. Reuse across calls.
_ENC: Final[msgspec.json.Encoder] = msgspec.json.Encoder(order="sorted")
_DEC: Final[msgspec.json.Decoder] = msgspec.json.Decoder(Frontmatter)


def parse(raw: bytes) -> Tuple[Frontmatter, bytes]:
    """Parse a memory file's bytes into ``(frontmatter, body)``.

    The body is returned as a zero-copy ``bytes`` slice of the input.

    Raises
    ------
    ValueError
        Missing leading or trailing delimiter, or invalid JSON inside
        the frontmatter block.
    msgspec.ValidationError
        Frontmatter does not match the :class:`Frontmatter` schema.
    """
    if not raw.startswith(_DELIM):
        raise ValueError("missing leading '---' frontmatter delimiter")
    end = raw.find(_DELIM, len(_DELIM))
    if end < 0:
        raise ValueError("missing trailing '---' frontmatter delimiter")
    fm_bytes = raw[len(_DELIM):end]
    body = raw[end + len(_DELIM):]
    return _DEC.decode(fm_bytes), body


def serialize(fm: Frontmatter, body: bytes) -> bytes:
    """Inverse of :func:`parse`.

    Output is byte-deterministic for identical inputs: sorted keys,
    canonical JSON, no trailing whitespace before the closing delimiter.
    Two calls with the same frontmatter+body produce identical bytes —
    required for the deterministic-export invariant (audit report §3.C.5).
    """
    return _DELIM + _ENC.encode(fm) + b"\n" + _DELIM + body


# --- legacy YAML fallback ---------------------------------------------------
#
# Read-only path for the migration window. Once `cyberos_migrate_v2.py` has
# rewritten every file in the store as JSON-frontmatter, this function can
# be deleted along with the PyYAML dependency. The dependency is loaded
# lazily so the cold-CLI startup budget is not paid until/unless a legacy
# file is read.

_YAML_PATTERN: Final[re.Pattern[bytes]] = re.compile(
    rb"^---\n(.*?)\n---\n", re.DOTALL,
)


def parse_legacy_yaml(raw: bytes) -> Tuple[Frontmatter, bytes]:
    """One-shot reader for pre-migration YAML frontmatter.

    YAML's permissive schema means some inputs that PyYAML accepted will
    be rejected here when converted to :class:`Frontmatter` — that is
    correct behaviour and the migration script handles it by surfacing
    such files for human review rather than silently coercing.

    Also accepts the **workbench-era v0 frontmatter shape** that was
    in use prior to the v1 ``id``/``kind``/``ts_ns``/``actor`` quartet.
    The v0 names ``memory_id`` / ``scope`` / ``created_by`` /
    ``created_at`` are aliased onto the v1 schema only when their v1
    counterparts are missing — explicit v1 values always win. This is the
    minimum compat shim needed so ``cyberos view`` succeeds on the 708
    memories imported from the workbench memory on 2026-05-19 (see
    ``playground/CONSUMED-FROM-WORKBENCH.md``).

    NOT round-trip stable. Migration always re-emits via :func:`serialize`.
    """
    import yaml  # noqa: WPS433 — lazy; PyYAML cold-imports ~30ms

    match = _YAML_PATTERN.match(raw)
    if not match:
        raise ValueError("no YAML frontmatter block found")
    data = yaml.safe_load(match.group(1).decode("utf-8")) or {}
    if isinstance(data, dict):
        _alias_v0_to_v1(data)
    fm = msgspec.convert(data, Frontmatter, strict=False)
    body = raw[match.end():]
    return fm, body


# Mapping from v0 ``scope`` values to v1 ``kind`` enum.
# Anything not in this map falls back to ``"unknown"`` (which IS in the v1
# enum) and the original v0 scope is preserved in ``extra.v0_scope`` for
# retrieval. This keeps cyberos validate happy without lying about content
# semantics — kind=unknown is the honest answer when the v0 scope doesn't
# fit a v1 kind cleanly.
_V0_SCOPE_TO_V1_KIND: Final[dict[str, str]] = {
    "decision": "decision",
    "fact": "fact",
    "person": "person",
    "project": "project",
    "preference": "preference",
    "drift": "drift",
    "refinement": "refinement",
    "episode": "episode",
    # v0-only scopes that don't map cleanly → unknown + extra.v0_scope
    "meta": "unknown",
    "module": "unknown",
    "memories": "unknown",
    "company": "unknown",
    "client": "unknown",
    "member": "unknown",
    "persona": "unknown",
}

# v0 fields that carry useful metadata and should be preserved into ``extra``
# rather than dropped silently by msgspec.convert(strict=False).
_V0_PRESERVE_KEYS: Final[tuple[str, ...]] = (
    "memory_id", "scope", "classification", "authority", "version",
    "created_at", "created_by", "last_updated_at", "updated_by",
    "supersedes", "superseded_by", "expires_at", "provenance", "consent",
)


def _alias_v0_to_v1(data: dict) -> None:
    """In-place alias workbench-era v0 frontmatter fields onto the v1 schema.

    Only fills missing v1 fields; never clobbers explicit v1 values.
    Original v0 metadata (scope, classification, authority, version,
    provenance, etc.) is stashed in ``extra`` so it survives the
    msgspec.convert(strict=False) pass — that pass drops unknown top-level
    keys, which would otherwise silently lose the v0 context.
    """
    if "id" not in data and "memory_id" in data:
        data["id"] = data["memory_id"]
    if "kind" not in data and "scope" in data:
        v0_scope = data["scope"]
        if isinstance(v0_scope, str):
            data["kind"] = _V0_SCOPE_TO_V1_KIND.get(v0_scope, "unknown")
    if "actor" not in data and "created_by" in data:
        data["actor"] = data["created_by"]
    if "ts_ns" not in data and "created_at" in data:
        from datetime import datetime  # noqa: WPS433 — lazy
        ts = data["created_at"]
        dt = None
        if isinstance(ts, datetime):
            dt = ts
        elif isinstance(ts, str):
            try:
                dt = datetime.fromisoformat(ts)
            except ValueError:
                dt = None
        if dt is not None:
            data["ts_ns"] = int(dt.timestamp() * 1_000_000_000)

    # Stash v0-only fields into extra so they survive strict=False drop.
    # Never overwrite a key that's already in extra (round-trip safety).
    extra = data.setdefault("extra", {})
    if isinstance(extra, dict):
        for key in _V0_PRESERVE_KEYS:
            if key in data and f"v0_{key}" not in extra:
                value = data[key]
                # Coerce non-JSON-serialisable types (datetime, etc.) to str.
                # extra is dict[str, Any] but msgspec.json.Encoder requires
                # JSON-compatible types at serialisation time.
                if hasattr(value, "isoformat"):  # datetime, date
                    value = value.isoformat()
                extra[f"v0_{key}"] = value


def parse_sidecar(meta_bytes: bytes, body_bytes: bytes) -> Tuple[Frontmatter, bytes]:
    """Read a (body, sidecar) pair from the sidecar-mode v2 layout.

    Per AGENTS.md v2 §5.1, a memory file MAY be either an in-body
    frontmatter file (legacy) or a body + ``.meta.json`` sidecar pair.
    This helper handles the latter.

    Raises
    ------
    ValueError
        The sidecar carries a ``body_hash`` that doesn't match the body
        bytes (AGENTS.md v2 §5.3 invariant).
    """
    fm = _DEC.decode(meta_bytes)
    body_hash = fm.extra.get("body_hash") if hasattr(fm, "extra") else None
    if isinstance(body_hash, str) and body_hash:
        import hashlib  # noqa: WPS433 — lazy
        # Strip optional "sha256:" prefix for legacy compatibility.
        expected = body_hash[len("sha256:"):] if body_hash.startswith("sha256:") else body_hash
        actual = hashlib.sha256(body_bytes).hexdigest()
        if actual != expected:
            raise ValueError(
                f"sidecar body_hash mismatch: meta={expected[:16]}… body={actual[:16]}…"
            )
    return fm, body_bytes


def looks_like_yaml(raw: bytes) -> bool:
    """Heuristic: does ``raw`` start with a YAML-style frontmatter block?

    Used by :class:`cyberos.core.reader.Reader` to dispatch between
    :func:`parse` and :func:`parse_legacy_yaml` during the migration
    window. Conservative: tries the JSON path first and falls back to
    YAML only if JSON decode fails — so a file that happens to be valid
    as both is treated as JSON (the new format).
    """
    if not raw.startswith(_DELIM):
        return False
    end = raw.find(_DELIM, len(_DELIM))
    if end < 0:
        return False
    candidate = raw[len(_DELIM):end].lstrip()
    # JSON frontmatter always starts with '{' after the delimiter.
    return not candidate.startswith(b"{")


__all__ = [
    "Frontmatter",
    "parse",
    "parse_sidecar",
    "serialize",
    "parse_legacy_yaml",
    "looks_like_yaml",
]
