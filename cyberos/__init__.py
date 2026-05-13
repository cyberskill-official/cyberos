"""
cyberos — the optimised Layer-1 implementation of the CyberOS BRAIN protocol.

Implements the audit report dated 2026-05 ("CyberOS Layer-1 Optimization Audit —
Senior Architect Report").

Public surface is intentionally tiny:

    from cyberos.core.writer import Writer, AuditRecord, WriterConfig
    from cyberos.core.reader import Reader
    from cyberos.core import ops               # six file ops live here
    from cyberos.core.frontmatter import parse, serialize, Frontmatter
    from cyberos.core.walker import MmapWalker

Everything heavier (sqlite, mmap, msgspec, hashlib) is loaded lazily from
subcommand handlers so cold `python -m cyberos --help` stays under 30ms.

This package coexists with the legacy `runtime/lib/brain_writer.py` writer.
Activation is gated on `<store>/index/manifest.json:schema_version == 2`
(set by `runtime/tools/cyberos_migrate_v2.py`); when set, host orchestration
uses this package as the active writer.

Invariants protected (audit report §3.C):

  1. Single writer per store (LOCK_EX on .lock with monotonic lease).
  2. Append-only ledger; no record mutated after the next record is written.
  3. Merkle LINK invariant preserved across the JSONL → binlog encoding
     change (msgspec canonical JSON ≡ RFC 8785 JCS for this closed schema;
     CI fuzz check enforces equivalence).
  4. Atomic record visibility via HEAD seqlock; readers wait-free.
  5. Deterministic export; byte-identical zips across runs and platforms.
  6. Six file ops only (view, create, str_replace, insert, delete, rename).
  7. `.cyberos-memory/` remains a self-contained, zippable artefact.
"""

__version__ = "2.0.0"

# Schema version recorded in <store>/index/manifest.json after migration.
# Activation switch: legacy brain_writer is active iff this value is absent
# or == 1; this package's writer is active iff this value is >= 2.
SCHEMA_VERSION = 2
