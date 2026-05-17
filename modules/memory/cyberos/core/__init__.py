"""
cyberos.core — Layer-1 core. Six file ops, group-commit writer, mmap reader.

Module map (audit report §4):

    fsync.py         platform-correct durability barrier
                     (F_BARRIERFSYNC on Darwin; fdatasync on Linux)
    frontmatter.py   msgspec parser replacing PyYAML
    lock.py          leased single-lock (LOCK_EX/LOCK_SH + monotonic TTL)
    writer.py        the ONLY ledger writer; group commit
    walker.py        mmap'd binlog walker; chain verification
    iouring.py       optional Linux fast path (lazy import)
    reader.py        lock-free read via HEAD seqlock
    index.py         WAL-mode SQLite (lives outside the store)
    ops.py           six file ops dispatch
    export.py        deterministic zip export

Hard import contract: NOTHING in this package may import sqlite3, msgspec,
mmap, hashlib, or rfc8785 at module-import time of cyberos.__init__ or
cyberos.__main__. Lazy imports preserve the cold-CLI startup budget.
"""
