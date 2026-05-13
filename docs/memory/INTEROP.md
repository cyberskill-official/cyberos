# CyberOS Memory — Interop Subset (≤ 6 000 chars, Cursor-compatible)

This file is the **minimum profile** another agent or editor must obey to safely
share a `.cyberos-memory/` store with the canonical CyberOS Layer-1 writer.
Cursor rule files, Codex CLI, Aider conventions, Copilot AGENTS.md — drop this
as `INTEROP.md` or symlink your `AGENTS.md` to it.

The full protocol (audit ledger, Merkle chain, consolidation, encryption envelope,
sync classes, GDPR purge) lives in `AGENTS.md` and `memory.schema.json`. This
subset documents only what a *consumer* must guarantee. A consumer that obeys
INTEROP.md is safe to coexist with the canonical writer; it will not corrupt the
audit chain or break determinism.

The key words **MUST**, **MUST NOT**, **SHOULD**, **MAY** are RFC 2119 / BCP 14.

---

## §1  Precedence

A USER instruction in the active chat session takes precedence over this file.
This file takes precedence over the consumer's defaults and over any other
generic agent-rules file in the project root.

## §2  Filesystem layout

The store root is `<project-root>/.cyberos-memory/`. Consumers MUST NOT operate
on any other path as the memory root. The canonical top-level layout:

```
.cyberos-memory/
├── manifest.json
├── HEAD                 (binary; do not write)
├── .lock                (coordination; do not bypass)
├── audit/               (the ledger — read-only for consumers)
│   ├── *.jsonl          legacy (schema v1)
│   ├── *.binlog         current (schema v2)
│   └── current.binlog
├── memories/<kind>/[<hex>/<hex>/]<file>.md
├── meta/  company/  module/  member/  client/  project/  persona/
├── conflicts/
└── exports/
```

`<kind>` ∈ `decisions | facts | people | projects | preferences | drift | refinements`.

## §3  File operations (the six)

Consumers MUST express every memory-state mutation as exactly one of:

| op | semantic |
|---|---|
| `view`        | read a memory file (still emits an audit row) |
| `create`      | create a new memory file; rejects if the file exists |
| `str_replace` | replace one unique occurrence of `old` with `new` in an existing file |
| `insert`      | splice text at line N of an existing file |
| `delete`      | soft-tombstone (the file body is preserved; tombstone is in the ledger) |
| `rename`      | move a file to a new path within the store |

There is no `update`, no `overwrite`, no `append`. If you need to replace whole
file contents, decompose: `delete` then `create` (or call the canonical CLI's
`overwrite` helper, which emits the right row pair).

Consumers MUST NOT touch `audit/`, `HEAD`, or `.lock` directly. To append to
the audit, MUST go through the canonical writer (legacy: `runtime/lib/brain_writer.py`;
v2: `python -m cyberos`).

## §4  Path validation

Every path argument:

* MUST be relative (no leading `/`, no drive letter on Windows);
* MUST resolve to a path strictly inside `.cyberos-memory/`;
* MUST NOT contain `..` segments after normalisation;
* MUST match `[A-Za-z0-9_][A-Za-z0-9_./\-]*$`.

Violations MUST be rejected at the call site, not at write time.

## §5  Atomic write

Every write to a memory file MUST be performed as a two-phase write:

1. write to `<path>.tmp.<nonce>` and fsync (on macOS: `fcntl(F_BARRIERFSYNC)`);
2. `rename(2)` to the final path;
3. fsync the parent directory.

Plain `fsync()` on macOS does NOT flush the device write cache. Use
`F_BARRIERFSYNC` for per-write durability and `F_FULLFSYNC` only for
checkpoint flushes. The canonical implementation is `cyberos/core/fsync.py`.

## §6  Frontmatter

Each memory file is Markdown with a frontmatter block:

```
---
{"id":"DEC-104","kind":"decision","ts_ns":1715126400000000000,"actor":"stephen","tags":[],"extra":{}}
---
# Body
```

Schema-v2 stores: JSON frontmatter (sorted keys; UTF-8; deterministic).
Schema-v1 stores: YAML frontmatter; consumers SHOULD use a tolerant parser
during the migration window. New writes SHOULD emit JSON.

Required fields: `id`, `kind`, `ts_ns`, `actor`. Optional: `tags` (list of
strings), `extra` (free-form object). Consumers MUST NOT introduce new
top-level keys outside `extra`.

## §7  Locking

Before any write, acquire the exclusive lock:

```
flock(<store>/.lock, LOCK_EX)
```

For long scans (read consistency across multiple files), acquire LOCK_SH.
For single-file reads, the canonical reader uses a HEAD seqlock and does
not take the flock — consumers that don't have a seqlock implementation
MAY use LOCK_SH instead.

Stale locks (writer died with SIGKILL) are reaped after 10 s via a
monotonic-clock lease stored inside `.lock` itself.

## §8  Determinism

Two `python -m cyberos export <path>` calls on the same store MUST produce
byte-identical zip output. Consumers MUST NOT introduce non-determinism
(unstable iteration order, hostname-dependent fields, etc.) in any file
under the store.

## §9  End-of-response block

Agents using this store SHOULD report, at the end of any session that
touched the store:

* file ops performed (count + path summary);
* memories read (count);
* any rejections (path traversal, content gate, validation);
* whether the agent observed `schema_version` 1 or 2.

This is informational, not normative.

## §10  What this subset does NOT include

The full protocol (load `AGENTS.md` for these) adds:

* the audit chain itself (consumers see it only via canonical-writer outputs);
* consolidation phases (Walk / Compact / Sign-STH / Publish);
* signed tree heads / Merkle Mountain Range;
* the `delete(path, "purge")` GDPR mode;
* classification + retention rules;
* encryption envelope;
* the `sync_class` privacy model;
* multi-agent interop semantics beyond the file-level invariants above.

Consumers that adopt only INTEROP.md are safe — they will not corrupt the
chain — but they are NOT auditing the chain, and they MUST defer all
chain-touching operations to the canonical writer.
