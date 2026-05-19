"""
cyberos — the Layer-1 implementation of the CyberOS memory protocol.

Public surface is intentionally tiny:

    from cyberos.core.writer import Writer, AuditRecord, WriterConfig
    from cyberos.core.reader import Reader
    from cyberos.core import ops               # canonical file ops live here
    from cyberos.core.frontmatter import parse, serialize, Frontmatter
    from cyberos.core.walker import MmapWalker

Everything heavier (sqlite, mmap, msgspec, hashlib) is loaded lazily from
subcommand handlers so cold `python -m cyberos --help` stays under 30ms.

The protocol is unversioned: manifests carry no required schema_version
field. Historical binlog rows from earlier protocol generations remain
readable; the writer emits only the canonical op names.

Invariants protected:

  1. Single writer per store (LOCK_EX on .lock with monotonic lease).
  2. Append-only ledger; no record mutated after the next record is written.
  3. Merkle LINK invariant: chain = SHA-256(canonical_json(rec) || prev_chain).
  4. Atomic record visibility via HEAD seqlock; readers wait-free.
  5. Deterministic export; byte-identical zips across runs and platforms.
  6. Canonical ops: view, put, move, delete. Historical names are
     read-only legacy data.
  7. `.cyberos-memory/` remains a self-contained, zippable artefact.
"""

__version__ = "2.0.0"
