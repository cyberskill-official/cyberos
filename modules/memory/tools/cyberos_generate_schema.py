#!/usr/bin/env python3
"""
memory/tools/cyberos_generate_schema.py — emit ``memory.schema.json``.

Derived from the msgspec types in :mod:`cyberos.core`. Single source of truth:
edit the Struct, regenerate the schema. Never hand-edit ``memory.schema.json``.

Why this exists (Deep Optimization Audit §4.2, §4.3):

* The audit recommends an RFC-style normative split — AGENTS.md as prose,
  ``memory.schema.json`` as the machine-validatable contract.
* ``additionalProperties: false`` on every closed shape so unknown fields
  surface immediately instead of silently propagating.
* Versioning is via ``$id`` and the schema's own ``schema_version``
  field, both pinned to ``cyberos.SCHEMA_VERSION``.

Run::

    cd memory && python tools/cyberos_generate_schema.py > docs/memory.schema.json

CI runs this, diffs against the committed file, fails on drift.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

# memory/ onto path so this tool can be invoked as a script and import
# the cyberos package that lives at ``memory/cyberos/``.
_MEMORY_ROOT = Path(__file__).resolve().parent.parent
if str(_MEMORY_ROOT) not in sys.path:
    sys.path.insert(0, str(_MEMORY_ROOT))

import msgspec  # noqa: E402

from cyberos.core.frontmatter import Frontmatter  # noqa: E402
from cyberos.core.writer import AuditRecord  # noqa: E402


# Canonical op names emitted by the current writer. The on-disk op field
# is free-form text (validated as a non-empty string only) — historical
# binlog rows may carry op names from earlier protocol generations and
# remain readable.
_CANONICAL_OPS = [
    "view", "put", "move", "delete",
    # Session boundary rows.
    "session.start", "session.end",
]

_KIND_ENUM = [
    "decision", "fact", "person", "project", "preference",
    "drift", "refinement",
    "unknown",  # forward-compat sentinel — write path uses this when no
                # frontmatter kind is supplied
]

_ACTOR_KIND_ENUM = ["human", "agent", "automation"]

_CLASSIFICATION_ENUM = ["public", "internal", "confidential", "restricted"]

_AUTHORITY_ENUM = ["human-edited", "agent-edited", "imported"]

_SYNC_CLASS_ENUM_V1 = ["local-only", "publishable", "shared", "client-visible"]
_SYNC_CLASS_ENUM_V2 = ["private", "shareable"]

_STATE_ENUM = ["READY", "FROZEN_RECOVERABLE", "FROZEN_HUMAN"]


def _inline_ref(schema: dict, components: dict) -> dict:
    """Replace a top-level ``{"$ref": "#/$defs/X"}`` with the referenced schema.

    msgspec returns the per-struct schema as a ``$ref`` into ``components``;
    we need the actual property dict so we can attach enum constraints.
    """
    if list(schema.keys()) == ["$ref"]:
        ref = schema["$ref"]
        # "#/$defs/Frontmatter" → "Frontmatter"
        name = ref.rsplit("/", 1)[-1]
        if name in components:
            inlined = dict(components[name])
            # Don't double-store the ref target.
            return inlined
    return dict(schema)


def _msgspec_schema(struct_cls) -> tuple[dict, dict]:
    """Return ``(schema, components)`` for one msgspec Struct.

    ``components`` MUST be merged into the top-level ``$defs`` of the
    composed schema so any ``$ref: #/$defs/X`` inside ``schema`` resolves.
    Without this merge, jsonschema's resolver raises
    ``RefResolutionError`` and validate() can't run.
    """
    schemas, components = msgspec.json.schema_components([struct_cls])
    return schemas[0], components


def build_schema() -> dict:
    """Compose the full memory.schema.json document."""
    # Start from msgspec-derived schemas for the closed Structs. msgspec
    # may emit `$ref: #/$defs/X` for nested types — components carries
    # those targets and we mirror them into both `$defs` and `definitions`
    # below so resolvers built against either convention work.
    audit_schema, audit_components = _msgspec_schema(AuditRecord)
    fm_schema, fm_components = _msgspec_schema(Frontmatter)

    # Tighten: enumerate the string fields whose value sets are closed.
    # When msgspec returned a bare `$ref`, we need to inline the
    # referenced schema first so the property dict actually exists.
    audit_schema = _inline_ref(audit_schema, audit_components)
    fm_schema = _inline_ref(fm_schema, fm_components)

    audit_schema.setdefault("properties", {})
    audit_schema["properties"]["op"] = {
        "type": "string",
        "minLength": 1,
        "description": (
            "Audit row op. Free-form text; historical binlog rows from "
            "earlier protocol generations may carry op names not in the "
            "current canonical set. Current writer emits: "
            + ", ".join(_CANONICAL_OPS) + "."
        ),
    }
    audit_schema["additionalProperties"] = False

    fm_schema.setdefault("properties", {})
    fm_schema["properties"].setdefault("kind", {})["enum"] = _KIND_ENUM
    fm_schema["additionalProperties"] = False

    # MemoryPath: a closed regex pattern matching the §4.1 traversal guard
    # and §3 canonical layout.
    memory_path_pattern = (
        r"^(meta|memories|company|module|member|client|project|persona|"
        r"audit|conflicts|exports|index|drift|refinements)"
        r"(/[A-Za-z0-9_][A-Za-z0-9_.\-]*)+\.(md|json)$"
    )

    schema = {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://cyberos.world/schemas/memory.json",
        "title": "CyberOS Layer-1 Memory Protocol",
        "description": (
            "Machine-validatable schema for the CyberOS memory. Generated "
            "from cyberos.core msgspec Structs by "
            "memory/tools/cyberos_generate_schema.py. Do not hand-edit. "
            "The protocol document (AGENTS.md) is the prose "
            "companion; this file is the contract. The protocol is "
            "unversioned: there is no schema_version field."
        ),
        "definitions": {
            "MemoryPath": {
                "type": "string",
                "pattern": memory_path_pattern,
                "description": (
                    "POSIX-relative path from <memory-root>/. MUST NOT "
                    "contain '..' segments or absolute prefixes. Pattern "
                    "is enforced by cyberos.core.ops._check_rel_path."
                ),
            },
            "Sha256Hex": {
                "type": "string",
                "pattern": "^[0-9a-f]{64}$",
                "description": "Lowercase 64-hex SHA-256 digest. No `sha256:` prefix.",
            },
            "Sha256Prefixed": {
                "type": "string",
                "pattern": "^sha256:[0-9a-f]{64}$",
                "description": (
                    "Legacy chain hash format used by earlier writers. "
                    "Current writer emits the bare 64-hex form."
                ),
            },
            "AuditRecord": audit_schema,
            "Frontmatter": fm_schema,
            "Manifest": _manifest_schema(),
            "Envelope": _envelope_schema(),
        },
        "type": "object",
        "properties": {},
        "additionalProperties": True,
    }
    return schema


def _manifest_schema() -> dict:
    """Schema for ``manifest.json`` at the store root.

    The protocol is unversioned: manifest is a small JSON file with no
    required fields. The ``schema_version`` property is preserved as an
    optional descriptive integer so historical manifests still validate;
    new manifests omit it.
    """
    return {
        "type": "object",
        "properties": {
            "schema_version": {
                "type": "integer",
                "minimum": 1,
                "description": (
                    "Optional, descriptive only. The protocol is now "
                    "unversioned; this property is preserved for "
                    "back-compat so historical manifests still validate."
                ),
            },
            "audit_chain_head": {"$ref": "#/definitions/Sha256Prefixed"},
            "last_updated_at": {"type": "string", "format": "date-time"},
            "timezone": {"type": "string"},
            "project": {
                "type": "object",
                "properties": {
                    "root_path": {"type": "string"},
                },
                "additionalProperties": True,
            },
            "scope_contract": {
                "type": "object",
                "additionalProperties": True,
            },
            "crypto_mode": {
                "type": "string",
                "enum": ["chained", "sth_only"],
                "description": (
                    "P2 Stage 3 feature flag. 'chained' (default): per-row "
                    "chain is canonical. 'sth_only': MMR + Signed Tree Heads "
                    "are canonical; the chain remains computed but advisory. "
                    "See cyberos.core.crypto_mode."
                ),
            },
            "crypto_mode_history": {
                "type": "array",
                "description": (
                    "Append-only audit trail of crypto_mode transitions."
                ),
                "items": {
                    "type": "object",
                    "properties": {
                        "from": {"type": "string"},
                        "to": {"type": "string"},
                        "at_ns": {"type": "integer"},
                        "approval_phrase": {"type": "string"},
                    },
                    "additionalProperties": True,
                },
            },
        },
        "additionalProperties": True,
    }


def _envelope_schema() -> dict:
    """Encryption envelope for cipher-protected memory bodies (§5.6)."""
    return {
        "type": "object",
        "description": (
            "Encryption envelope per AGENTS.md §5.6. Specifies algorithm "
            "and key wrap; the body file then contains ciphertext."
        ),
        "properties": {
            "cipher": {
                "type": "string",
                "enum": ["age-x25519", "aes-256-gcm", "chacha20-poly1305"],
            },
            "key_id": {"type": "string"},
            "nonce": {"type": "string"},
            "ciphertext_hash": {"$ref": "#/definitions/Sha256Hex"},
        },
        "required": ["cipher", "key_id"],
        "additionalProperties": False,
    }


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(prog="cyberos_generate_schema")
    p.add_argument(
        "--out", default="-",
        help="Output path (default: stdout)",
    )
    p.add_argument(
        "--check", action="store_true",
        help="Generate to stdout; exit 1 if --out path differs (CI gate)",
    )
    args = p.parse_args(argv)

    schema = build_schema()
    body = json.dumps(schema, indent=2, sort_keys=False) + "\n"

    if args.out == "-":
        sys.stdout.write(body)
        return 0

    out_path = Path(args.out)
    if args.check:
        current = out_path.read_text(encoding="utf-8") if out_path.exists() else ""
        if current == body:
            return 0
        sys.stderr.write(
            f"[cyberos_generate_schema] {out_path} is out of date; "
            f"re-run without --check to regenerate.\n"
        )
        return 1

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(body, encoding="utf-8")
    sys.stderr.write(f"[cyberos_generate_schema] wrote {out_path}\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
