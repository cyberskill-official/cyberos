"""
cyberos — single entry point, no per-tool Python cold-start tax.

All heavy imports (msgspec, sqlite3, mmap, the writer) are loaded only
inside the subcommand handler that needs them. Cold ``cyberos --help`` is
argparse + stdlib only — target <30ms on a 2024 MacBook M2 vs ~110ms for
the legacy shell-out-to-tool.py pattern.

PEP 690 documents 50–70% startup reductions from this lazy-import pattern
(peps.python.org/pep-0690). Hugo van Kemenade's pypistats benchmark shows
104ms → 46ms → 35ms on a comparable seven-direct-dep CLI.

The CLI exposes the canonical ops plus housekeeping commands::

    cyberos view <path>
    cyberos put <path> <body_file>
    cyberos move <src> <dst>
    cyberos delete <path>
    cyberos audit dump [--month YYYY-MM]
    cyberos audit head
    cyberos verify
    cyberos export <out.zip>
    cyberos search <query>
    cyberos checkpoint
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


# --- subcommand handlers — each lazy-imports what it needs ----------------


def _store(args: argparse.Namespace) -> Path:
    """Resolve the memory root.

    Resolution order (§0.4):

    1. If ``--store`` was explicitly provided (or the env var
       ``CYBEROS_STORE`` is set), use that.
    2. Otherwise auto-discover by walking up from CWD for a
       ``.cyberos/memory/store/`` directory. This lets you run
       ``cyberos doctor`` from any subdir of the project — `memory/`,
       `memory/cyberos/`, repo root — and the CLI finds the memory.
    3. Fall back to ``./.cyberos/memory/store`` (the canonical default;
       a fresh store is created there) if neither of the above pans out.
    """
    import os

    # 1. explicit --store or env var
    explicit = args.store
    if explicit and explicit != ".cyberos/memory/store":
        return Path(explicit).resolve()
    env_store = os.environ.get("CYBEROS_STORE")
    if env_store:
        return Path(env_store).resolve()

    # 2. walk up from CWD for .cyberos/memory/store (§0.4)
    cwd = Path.cwd().resolve()
    for parent in (cwd, *cwd.parents):
        candidate = parent / ".cyberos" / "memory" / "store"
        if candidate.is_dir():
            return candidate

    # 3. fallback — the canonical default (a fresh store is created here,
    #    otherwise the invariant walker surfaces a meaningful error).
    return (cwd / ".cyberos" / "memory" / "store").resolve()


def _actor(args: argparse.Namespace) -> str:
    return args.actor or "cyberos-cli"


def _cmd_view(args: argparse.Namespace) -> int:
    from cyberos.core.reader import Reader  # noqa: WPS433

    fm, body = Reader(_store(args)).view(args.path)
    sys.stdout.buffer.write(b"---\n")
    import msgspec  # noqa: WPS433
    sys.stdout.buffer.write(msgspec.json.encode(fm, order="sorted"))
    sys.stdout.buffer.write(b"\n---\n")
    sys.stdout.buffer.write(body)
    return 0


def _cmd_put(args: argparse.Namespace) -> int:
    """Canonical v2 op — create or replace a memory file.

    Per TASK-MEMORY-114 §1 #1 / #2, the ``--score-importance`` flag opts in
    to write-time LLM scoring; ``--importance <float>`` is the explicit
    operator override (wins over scoring). With neither flag the default
    `put` path is unchanged.
    """
    from cyberos.core.ops import put
    from cyberos.core.writer import Writer

    body = Path(args.body_file).read_bytes()

    # Resolve importance per TASK-MEMORY-114 §1 #1 / #2 priority chain:
    #   1. --importance <float>  → operator-pinned, no LLM call
    #   2. --score-importance     → invoke LLM (with cache); record outcome
    #   3. neither                → no importance metadata written
    importance_value: float | None = None
    importance_aux_rows: list[tuple[str, dict]] = []  # collected, emitted post-write

    if getattr(args, "importance", None) is not None:
        if not (0.0 <= args.importance <= 1.0):
            sys.stderr.write(
                f"error: --importance must be in [0.0, 1.0]; got {args.importance}\n"
            )
            return 2
        importance_value = float(args.importance)
    elif getattr(args, "score_importance", False):
        # Synchronous scoring path — the CLI is not async.
        from cyberos.core.importance import (
            ImportanceCache,
            score_sync,
            select_invoker,
        )

        invoker_name = getattr(args, "invoker", None)
        try:
            invoker = select_invoker(invoker_name)
        except (ValueError, RuntimeError) as exc:
            sys.stderr.write(f"error: {exc}\n")
            return 2

        cache_path = _store(args) / "index" / "importance_cache.db"
        cache = ImportanceCache(cache_path)

        def _capture_aux(kind: str, payload: dict) -> None:
            importance_aux_rows.append((kind, payload))

        try:
            result = score_sync(
                # Score on the body bytes the operator is about to write
                body.decode("utf-8", errors="replace"),
                invoker,
                cache,
                aux_emitter=_capture_aux,
                path=args.path,
            )
        finally:
            cache.close()
        importance_value = float(result.score)
        if args.dry_run:
            print(
                f"DRY-RUN: would write {args.path} with importance={importance_value:.3f} "
                f"(model={result.model} outcome={result.outcome} latency_ms={result.latency_ms})"
            )
            return 0

    extra: dict[str, object] = {}
    if importance_value is not None:
        extra["importance"] = importance_value

    if getattr(args, "dry_run", False):
        # No-write dry-run path (operator inspecting --score-importance output)
        print(f"DRY-RUN: would write {args.path}; extra={extra}")
        return 0

    with Writer(_store(args)) as writer:
        seq = put(
            writer,
            args.path,
            body,
            actor=_actor(args),
            kind=args.kind or "unknown",
            extra=extra or None,
        )
        # Emit captured aux rows AFTER the put (so they trail the put row in seq)
        for kind, payload in importance_aux_rows:
            from cyberos.core.writer import AuditRecord
            writer.submit(AuditRecord(
                op=kind,
                path=args.path,
                actor=_actor(args),
                extra=payload,
            ))
    print(f"seq={seq}")
    return 0


def _cmd_move(args: argparse.Namespace) -> int:
    """Canonical v2 op — rename within the store."""
    from cyberos.core.ops import move
    from cyberos.core.writer import Writer

    with Writer(_store(args)) as writer:
        seq = move(writer, args.src, args.dst, actor=_actor(args))
    print(f"seq={seq}")
    return 0


def _cmd_delete(args: argparse.Namespace) -> int:
    from cyberos.core.ops import delete, PurgeRefused
    from cyberos.core.writer import Writer

    try:
        with Writer(_store(args)) as writer:
            seq = delete(
                writer, args.path, actor=_actor(args),
                mode=args.mode,
                reason=args.reason,
                approval_phrase=args.approval_phrase,
            )
    except PurgeRefused as exc:
        sys.stderr.write(f"purge refused: {exc}\n")
        return 2
    print(f"seq={seq}")
    return 0


def _cmd_verify(args: argparse.Namespace) -> int:
    from cyberos.core.walker import verify_segments  # noqa: WPS433
    from cyberos.core.writer import _GENESIS_CHAIN  # noqa: WPS433

    store = _store(args)
    segments = sorted(
        p for p in (store / "audit").glob("*.binlog") if p.name != "current.binlog"
    )
    current = store / "audit" / "current.binlog"
    if current.exists():
        segments.append(current)
    n = verify_segments(segments, start_prev=_GENESIS_CHAIN)
    print(f"verified {n} records across {len(segments)} segment(s); chain intact")
    return 0


def _cmd_export(args: argparse.Namespace) -> int:
    from cyberos.core.export import export_zip  # noqa: WPS433

    digest = export_zip(_store(args), Path(args.out))
    print(f"sha256={digest}")
    return 0


def _cmd_audit_dump(args: argparse.Namespace) -> int:
    from cyberos.core.walker import MmapWalker  # noqa: WPS433
    import msgspec  # noqa: WPS433

    store = _store(args)
    audit_dir = store / "audit"
    if args.month:
        targets = [audit_dir / f"{args.month}.binlog"]
    else:
        targets = sorted(audit_dir.glob("*.binlog"))
    enc = msgspec.json.Encoder(order="sorted")
    for path in targets:
        if not path.exists():
            continue
        with MmapWalker(path) as walker:
            for _offset, rec in walker.iter_records():
                sys.stdout.buffer.write(enc.encode(rec))
                sys.stdout.buffer.write(b"\n")
    return 0


def _cmd_audit_head(args: argparse.Namespace) -> int:
    import struct  # noqa: WPS433
    head_path = _store(args) / "HEAD"
    if not head_path.is_file():
        print("0")
        return 0
    with open(head_path, "rb") as fh:
        buf = fh.read(8)
    if len(buf) != 8:
        print("0")
        return 0
    print(struct.unpack("<Q", buf)[0])
    return 0


def _cmd_audit(args: argparse.Namespace) -> int:
    if args.action == "dump":
        return _cmd_audit_dump(args)
    if args.action == "head":
        return _cmd_audit_head(args)
    print(f"unknown audit action: {args.action}", file=sys.stderr)
    return 2


def _cmd_search(args: argparse.Namespace) -> int:
    import hashlib  # noqa: WPS433

    store = _store(args)

    if args.semantic:
        from cyberos.core.semantic import available, search as semantic_search  # noqa: WPS433
        if not available():
            sys.stderr.write(
                "semantic search dependencies missing; install with:\n"
                "  pip install sentence-transformers --break-system-packages\n"
                "Falling back to FTS5.\n"
            )
        else:
            hits = semantic_search(store, args.query, limit=args.limit)
            if not hits:
                sys.stderr.write(
                    "no embedded memories yet; run "
                    "`cyberos semantic-sync` first.\n"
                )
                return 0
            for h in hits:
                print(f"{h.score:.3f}\t{h.rel_path}\t{h.snippet}")
            return 0

    from cyberos.core.index import (  # noqa: WPS433
        open_index, replay_from_binlog, search_memories,
    )
    fingerprint = hashlib.sha256(str(store).encode("utf-8")).hexdigest()[:16]
    conn = open_index(fingerprint)
    # Lazy-sync the index from the audit binlog before querying. Replay is
    # idempotent (uses last_applied_seq) so this is a no-op when up-to-date.
    replay_from_binlog(conn, store)
    for rel_path, snippet in search_memories(conn, args.query, limit=args.limit):
        print(f"{rel_path}\t{snippet}")
    return 0


def _cmd_semantic_sync(args: argparse.Namespace) -> int:
    """Re-embed memories whose body_sha256 changed since the last sync."""
    from cyberos.core.semantic import available, sync  # noqa: WPS433
    if not available():
        sys.stderr.write(
            "semantic search dependencies missing; install with:\n"
            "  pip install sentence-transformers --break-system-packages\n"
        )
        return 2
    report = sync(_store(args), batch_size=args.batch_size)
    print(f"  indexed   : {report.indexed}")
    print(f"  unchanged : {report.skipped_unchanged}")
    print(f"  removed   : {report.removed}")
    print(f"  total     : {report.total_in_index}")
    return 0


def _cmd_checkpoint(args: argparse.Namespace) -> int:
    from cyberos.core.writer import Writer  # noqa: WPS433

    with Writer(_store(args)) as writer:
        writer.checkpoint()
    print("checkpoint flushed (F_FULLFSYNC barrier on Darwin)")
    return 0


def _validate_against_schema(store: "Path", fm) -> str | None:
    """Return an error message string, or None if frontmatter validates.

    Schema lives at ``<store>/memory.schema.json`` (if present) or in the
    repo's ``memory/docs/memory.schema.json``. If jsonschema isn't
    installed, skip — we don't want validate to silently miss
    constraints; print a one-time stderr note and return None.
    """
    from cyberos.core.invariants import _find_memory_schema  # noqa: WPS433
    schema_path = _find_memory_schema(store)
    if schema_path is None:
        return None
    try:
        import jsonschema  # type: ignore[import-not-found]
    except ImportError:
        # Hint once per process so the user knows validation is partial.
        global _VALIDATE_HINT_PRINTED
        if not _VALIDATE_HINT_PRINTED:
            sys.stderr.write(
                "[validate] jsonschema not installed; enum constraints not enforced. "
                "Run: pip install jsonschema --break-system-packages\n"
            )
            _VALIDATE_HINT_PRINTED = True
        return None
    try:
        import json  # noqa: WPS433
        import msgspec  # noqa: WPS433
        full = json.loads(schema_path.read_text(encoding="utf-8"))
        validator_cls = (
            getattr(jsonschema, "Draft202012Validator", None)
            or getattr(jsonschema, "Draft201909Validator", None)
            or jsonschema.Draft7Validator
        )
        resolver = jsonschema.RefResolver.from_schema(full)
        validator = validator_cls(
            full["definitions"]["Frontmatter"], resolver=resolver,
        )
        fm_dict = msgspec.to_builtins(fm)
        errors = list(validator.iter_errors(fm_dict))
    except Exception as exc:  # noqa: BLE001 — surface as a validate failure
        return f"validate harness error: {exc!r}"
    if errors:
        first = errors[0]
        return f"schema violation: {first.message} (at {list(first.path) or 'root'})"
    return None


_VALIDATE_HINT_PRINTED = False


def _cmd_validate(args: argparse.Namespace) -> int:
    """Validate one or more memory files against the frontmatter schema.

    Three classes of error surface here:

    * **Path violations** (caught by :func:`cyberos.core.ops._check_rel_path`)
      — leading ``/``, ``..`` segments, NUL bytes, invalid characters.
    * **Frontmatter shape** — missing delimiters, unparseable JSON,
      schema violations like an unknown ``kind`` enum value.
    * **Hash drift** — if ``meta.body_hash`` exists (a Deep-Audit P3
      sidecar-spec property), check it matches the body bytes.

    Exits 0 if every file passes, 1 if any failure surfaces.
    """
    from cyberos.core.frontmatter import (  # noqa: WPS433 — lazy heavy import
        looks_like_yaml, parse, parse_legacy_yaml,
    )
    from cyberos.core.ops import _check_rel_path, PathTraversal  # noqa: WPS433

    store = _store(args)
    failures: list[tuple[str, str]] = []
    n_checked = 0

    for raw_path in args.paths:
        n_checked += 1
        abs_path = (store / raw_path).resolve()
        # Path sanity (catches "../foo", "/abs", etc.)
        try:
            _check_rel_path(raw_path)
        except PathTraversal as exc:
            failures.append((raw_path, f"path traversal: {exc}"))
            continue

        if not abs_path.is_file():
            failures.append((raw_path, "not a file"))
            continue
        try:
            data = abs_path.read_bytes()
        except OSError as exc:
            failures.append((raw_path, f"read failed: {exc}"))
            continue
        try:
            if looks_like_yaml(data):
                fm, body = parse_legacy_yaml(data)
            else:
                fm, body = parse(data)
        except Exception as exc:  # noqa: BLE001 — surface every parse failure
            failures.append((raw_path, f"frontmatter parse: {type(exc).__name__}: {exc}"))
            continue

        # JSON-Schema validation — catches enum violations on string
        # fields (kind, classification, etc.) that msgspec doesn't gate
        # because those Struct fields are plain `str`.
        schema_err = _validate_against_schema(store, fm)
        if schema_err is not None:
            failures.append((raw_path, schema_err))
            continue
        # If the frontmatter carries a body_hash in extra, verify it.
        body_hash = fm.extra.get("body_hash") if hasattr(fm, "extra") else None
        if isinstance(body_hash, str) and body_hash:
            import hashlib  # noqa: WPS433
            actual = hashlib.sha256(body).hexdigest()
            if body_hash.startswith("sha256:"):
                body_hash = body_hash[len("sha256:"):]
            if actual != body_hash:
                failures.append((
                    raw_path,
                    f"body_hash drift: meta={body_hash[:16]}… body={actual[:16]}…",
                ))
                continue

    for path, msg in failures:
        print(f"  FAIL {path}  {msg}")
    print(f"checked {n_checked}, {len(failures)} failure(s)")
    return 0 if not failures else 1


def _cmd_import(args: argparse.Namespace) -> int:
    """Pull memories from another memory into this one (PROPOSAL.md P6)."""
    from cyberos.core.import_ import format_report, run  # noqa: WPS433

    map_actor: dict[str, str] | None = None
    if args.map_actor:
        map_actor = {}
        for spec in args.map_actor:
            if ":" not in spec:
                sys.stderr.write(f"--map-actor expects FROM:TO, got {spec!r}\n")
                return 2
            old, new = spec.split(":", 1)
            map_actor[old] = new

    try:
        report = run(
            _store(args),
            Path(args.source).expanduser().resolve(),
            filters=args.filter,
            on_conflict=args.on_conflict,
            map_actor=map_actor,
            since=args.since,
            dry_run=args.dry_run,
        )
    except ValueError as exc:
        sys.stderr.write(f"error: {exc}\n")
        return 2
    print(format_report(report, dry_run=args.dry_run))
    return 0 if report.ok else 1


def _cmd_backup(args: argparse.Namespace) -> int:
    """Take an incremental snapshot (hard-linked) to ``--target``."""
    from cyberos.core.backup import (  # noqa: WPS433
        backup, format_backup_report, list_snapshots, verify_snapshot,
    )
    target = Path(args.target).expanduser().resolve()
    if args.list:
        snapshots = list_snapshots(target)
        if not snapshots:
            print(f"no snapshots under {target}")
            return 0
        for s in snapshots:
            print(
                f"  {s['name']}  "
                f"linked={s['files_linked']:>5}  "
                f"copied={s['files_copied']:>5}  "
                f"root={s['root_hash'][:16]}…  "
                f"label={s.get('label') or '—'}"
            )
        return 0
    if args.verify:
        if not args.snapshot:
            sys.stderr.write("--verify requires --snapshot <name>\n")
            return 2
        ok, msg = verify_snapshot(target, args.snapshot)
        print(f"{'OK' if ok else 'FAIL'}: {msg}")
        return 0 if ok else 1
    report = backup(_store(args), target, label=args.label)
    print(format_backup_report(report))
    return 0 if report.ok else 1


def _cmd_prune(args: argparse.Namespace) -> int:
    """Sweep archived binlog originals after the soak window."""
    from cyberos.core.prune import (  # noqa: WPS433
        format_prune_report, format_restore_report, prune, restore,
    )
    if args.restore:
        segs = args.segments if args.segments else None
        report = restore(_store(args), segment_names=segs)
        print(format_restore_report(report))
        return 0 if report.ok else 1
    p = prune(_store(args), soak_days=args.soak_days, dry_run=args.dry_run)
    print(format_prune_report(p, dry_run=args.dry_run))
    return 0 if p.ok else 1


def _cmd_prove(args: argparse.Namespace) -> int:
    """Produce a Merkle inclusion proof for an audit row.

    Output JSON shape::

        {
          "leaf_index": <int>,
          "leaf_payload_b64": "<base64 of canonical-JSON bytes>",
          "proof": ["<hex>", ...],
          "root_hex": "<hex>",
          "leaf_count": <int>,
          "sth_path": "<rel path>" or null
        }

    The verifier (``cyberos verify-proof``) consumes this and asserts
    ``MMR.verify_inclusion(leaf_payload, leaf_index, proof, root,
    leaf_count)`` returns True.
    """
    import base64 as _b64
    import json as _json

    from cyberos.core.mmr import MMR  # noqa: WPS433
    from cyberos.core.sth import latest_sth  # noqa: WPS433
    from cyberos.core.walker import MmapWalker  # noqa: WPS433

    store = _store(args)
    target_seq: int = args.seq

    # Collect every payload and find the one whose _seq matches.
    audit = store / "audit"
    segs = sorted(p for p in audit.glob("*.binlog") if p.name != "current.binlog")
    current = audit / "current.binlog"
    if current.exists():
        segs.append(current)

    all_payloads: list[bytes] = []
    target_index: int | None = None
    for seg in segs:
        with MmapWalker(seg) as walker:
            for _o, payload in walker.iter_payloads():
                all_payloads.append(payload)
        # Re-walk for record metadata to find the leaf index of the
        # requested seq. Walker preserves per-record _seq in extra; we
        # mirror that by re-reading the same segment via iter_records.
    # Find leaf index of target_seq.
    leaf_idx = -1
    cur_idx = 0
    for seg in segs:
        with MmapWalker(seg) as walker:
            for _o, rec in walker.iter_records():
                seq = int(rec.extra.get("_seq", 0))
                if seq == target_seq:
                    leaf_idx = cur_idx
                cur_idx += 1
    if leaf_idx == -1:
        sys.stderr.write(f"audit row seq={target_seq} not found\n")
        return 2

    # Build MMR + proof.
    mmr = MMR()
    for p in all_payloads:
        mmr.append_leaf(p)
    proof = mmr.inclusion_proof(leaf_idx, iter(all_payloads))
    root = mmr.root()

    sth_info = latest_sth(store)
    sth_relpath = (
        str(sth_info[0].relative_to(store)) if sth_info else None
    )

    out = {
        "leaf_index": leaf_idx,
        "audit_seq": target_seq,
        "leaf_payload_b64": _b64.b64encode(all_payloads[leaf_idx]).decode("ascii"),
        "proof": [p.hex() for p in proof],
        "root_hex": root.hex(),
        "leaf_count": mmr.leaf_count,
        "sth_path": sth_relpath,
    }
    if args.out and args.out != "-":
        Path(args.out).write_text(_json.dumps(out, indent=2), encoding="utf-8")
        print(f"wrote proof → {args.out}")
    else:
        print(_json.dumps(out, indent=2))
    return 0


def _cmd_verify_proof(args: argparse.Namespace) -> int:
    """Re-verify a proof emitted by ``cyberos prove``.

    Re-runs ``MMR.verify_inclusion`` over the proof's stated leaf bytes,
    index, path, and root. Optionally cross-checks the root against the
    STH the proof references (if present in the store).
    """
    import base64 as _b64
    import json as _json
    from cyberos.core.mmr import MMR  # noqa: WPS433

    proof_path = Path(args.proof)
    if not proof_path.is_file():
        sys.stderr.write(f"proof file not found: {proof_path}\n")
        return 2
    proof = _json.loads(proof_path.read_text(encoding="utf-8"))
    leaf = _b64.b64decode(proof["leaf_payload_b64"])
    proof_digests = [bytes.fromhex(h) for h in proof["proof"]]
    root = bytes.fromhex(proof["root_hex"])

    ok = MMR.verify_inclusion(
        leaf, proof["leaf_index"], proof_digests, root, proof["leaf_count"],
    )
    if not ok:
        print(f"VERIFY FAIL — proof does not verify against root {proof['root_hex'][:16]}…")
        return 1

    # Optional STH cross-check.
    sth_msg = ""
    if proof.get("sth_path"):
        from cyberos.core.sth import (  # noqa: WPS433
            P2NotActive, verify_tree_head,
        )
        store = _store(args)
        sth_full = store / proof["sth_path"]
        if sth_full.is_file():
            sth_rec = _json.loads(sth_full.read_text(encoding="utf-8"))
            if sth_rec.get("root_hash") != proof["root_hex"]:
                print(
                    f"VERIFY FAIL — proof root != STH root "
                    f"({proof['root_hex'][:16]}… vs {sth_rec.get('root_hash', '')[:16]}…)"
                )
                return 1
            try:
                if not verify_tree_head(sth_rec):
                    print("VERIFY FAIL — STH signature did not verify")
                    return 1
                sth_msg = f"; STH {proof['sth_path']} ✓"
            except P2NotActive:
                sth_msg = "; STH check skipped (no signing key on this host)"
    print(f"VERIFY OK — leaf {proof['leaf_index']} in MMR root {proof['root_hex'][:16]}…{sth_msg}")
    return 0


def _cmd_sth_wrap(args: argparse.Namespace) -> int:
    """Migrate the STH signing key from stage-1 raw to stage-2 wrapped form.

    The key on disk goes from a bare 32-byte Ed25519 seed to a
    scrypt+ChaCha20-Poly1305 wrapped form. Public key (and therefore
    all existing STHs' verifiability) is unchanged. Idempotent.
    """
    from cyberos.core.sth import KeyPaths, P2NotActive, wrap_existing_key

    # Passphrase MUST come explicitly — no env-var fallback for this
    # one-shot operation to avoid accidental wrap with an ambient value.
    if not args.passphrase_file and not args.passphrase:
        print(
            "error: provide --passphrase or --passphrase-file. "
            "(env-var fallback is intentionally disabled for the one-shot "
            "rekey to avoid accidental wraps with stale ambient values.)",
            file=sys.stderr,
        )
        return 2
    if args.passphrase_file:
        phrase = Path(args.passphrase_file).read_bytes().rstrip(b"\n")
    else:
        phrase = args.passphrase.encode("utf-8")

    try:
        wrap_existing_key(paths=None, passphrase=phrase)
    except P2NotActive as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    except FileNotFoundError as exc:
        print(f"error: no key file at {exc}; "
              f"run any signing command first to generate one", file=sys.stderr)
        return 2
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    print("STH signing key migrated to passphrase-wrapped form (stage-2).")
    print("Set CYBEROS_STH_PASSPHRASE for non-interactive operations.")
    return 0


def _cmd_state(args: argparse.Namespace) -> int:
    """Print AGENTS.md v2 §12 agent state derived from doctor results.

    READY                — all invariants pass.
    FROZEN_RECOVERABLE   — at least one error-level check failed but
                           the failure mode is recoverable via repair
                           tooling. (Examples: stale shard layout,
                           missing index manifest.)
    FROZEN_HUMAN         — catastrophic divergence (chain corrupt,
                           manifest unparseable, MMR cross-check
                           failed). Requires explicit human steps.
    """
    from cyberos.core.invariants import run_all  # noqa: WPS433
    report = run_all(_store(args))

    if report.ok:
        state = "READY"
        reason = "all invariants pass"
    else:
        catastrophic_ids = {
            "ledger-link-invariant",
            "ledger-hash-invariant",
            "ledger-mmr-cross-check",
            "manifest-validates-against-schema",
        }
        catastrophic_failures = [
            r for r in report.errors if r.id in catastrophic_ids
        ]
        if catastrophic_failures:
            state = "FROZEN_HUMAN"
            reason = "; ".join(
                f"{r.id}: {r.details}" for r in catastrophic_failures
            )
        else:
            state = "FROZEN_RECOVERABLE"
            reason = "; ".join(
                f"{r.id}: {r.details}" for r in report.errors
            )

    if args.json:
        import json as _json
        print(_json.dumps({"state": state, "reason": reason}))
    else:
        print(f"state:  {state}")
        print(f"reason: {reason}")
    return 0 if state == "READY" else 1


def _cmd_consolidate(args: argparse.Namespace) -> int:
    """Run Walk → Compact → Sign → Publish [→ SemanticDedup] (AGENTS.md v2 §7).

    Per TASK-MEMORY-116, ``--semantic-dedup`` adds a fifth phase that reuses
    TASK-MEMORY-115's duplicates detector. ``--semantic-dedup-apply``
    promotes the dry-run dedup pass into actual writes.
    """
    from cyberos.core.consolidate import format_report, run  # noqa: WPS433

    report = run(
        _store(args),
        dry_run=args.dry_run,
        compact_horizon_days=args.compact_horizon_days,
        semantic_dedup=getattr(args, "semantic_dedup", False),
        semantic_dedup_apply=getattr(args, "semantic_dedup_apply", False),
        semantic_dedup_threshold=getattr(args, "semantic_dedup_threshold", 0.92),
        semantic_dedup_scope=getattr(args, "semantic_dedup_scope", "") or "",
    )
    print(format_report(report, json_mode=args.json))
    if getattr(args, "semantic_dedup", False) and not args.json:
        print(f"semantic_dedup_ran:               {report.semantic_dedup_ran}")
        print(f"semantic_dedup_proposals_count:   {report.semantic_dedup_proposals_count}")
        print(f"semantic_dedup_applied_count:     {report.semantic_dedup_applied_count}")
        print(f"semantic_dedup_dream_id:          {report.semantic_dedup_dream_id}")
        if report.semantic_dedup_dry_run:
            print("(dry-run: pass --semantic-dedup-apply to merge proposals)")
    return 0 if report.ok else 1


def _cmd_crypto_mode(args: argparse.Namespace) -> int:
    """Inspect or migrate the store's crypto_mode (P2 Stage 3)."""
    from cyberos.core.crypto_mode import (  # noqa: WPS433
        APPROVAL_PHRASE, CryptoModeError, current_mode,
        downgrade_to_chained, upgrade_to_sth_only,
    )
    import json as _json

    store = _store(args)
    action = args.action

    if action == "show":
        mode = current_mode(store)
        if args.json:
            print(_json.dumps({"crypto_mode": mode}))
        else:
            print(f"crypto_mode: {mode}")
            if mode == "chained":
                print("  (default — per-row chain is the canonical integrity primitive)")
            else:
                print("  (P2 Stage 3 — MMR + STH are canonical; chain is advisory)")
        return 0

    if action == "upgrade":
        if not args.approval_phrase:
            sys.stderr.write(
                "upgrade requires --approval-phrase\n"
                f"Cite verbatim: {APPROVAL_PHRASE}\n"
            )
            return 2
        try:
            summary = upgrade_to_sth_only(
                store,
                approval_phrase=args.approval_phrase,
                skip_safety_checks=args.skip_safety_checks,
            )
        except CryptoModeError as exc:
            sys.stderr.write(f"error: {exc}\n")
            return 2
        if args.json:
            print(_json.dumps(summary, indent=2, sort_keys=True))
        else:
            print(f"crypto_mode: {summary['previous_mode']} → {summary['current_mode']}")
        return 0

    if action == "downgrade":
        if not args.approval_phrase:
            sys.stderr.write(
                "downgrade requires --approval-phrase (same phrase as upgrade)\n"
                f"Cite verbatim: {APPROVAL_PHRASE}\n"
            )
            return 2
        try:
            summary = downgrade_to_chained(
                store, approval_phrase=args.approval_phrase,
            )
        except CryptoModeError as exc:
            sys.stderr.write(f"error: {exc}\n")
            return 2
        if args.json:
            print(_json.dumps(summary, indent=2, sort_keys=True))
        else:
            print(f"crypto_mode: {summary['previous_mode']} → {summary['current_mode']}")
        return 0

    sys.stderr.write(f"unknown crypto-mode action: {action!r}\n")
    return 2


def _cmd_session(args: argparse.Namespace) -> int:
    """Manage multi-agent coordination sessions (PROPOSAL.md P11)."""
    from cyberos.core.session import (  # noqa: WPS433
        end_session, find_scope_conflicts, format_sessions,
        list_sessions, start_session,
    )
    import json as _json

    store = _store(args)
    action = args.action

    if action == "start":
        scope = (
            [s.strip() for s in args.scope.split(",") if s.strip()]
            if args.scope else []
        )
        conflicts = find_scope_conflicts(store, scope)
        if conflicts and not args.force:
            sys.stderr.write(
                "scope overlap with existing active session(s):\n"
            )
            for sess, overlaps in conflicts:
                sys.stderr.write(
                    f"  - {sess.id}  actor={sess.actor}  "
                    f"overlapping={overlaps}\n"
                )
            sys.stderr.write(
                "Pass --force to claim anyway, or pick a narrower --scope.\n"
            )
            return 2
        sess = start_session(
            store,
            actor=_actor(args) if not args.session_actor else args.session_actor,
            scope=scope,
            ttl_ns=args.ttl_hours * 3600 * 1_000_000_000,
            note=args.note or "",
        )
        if args.json:
            from dataclasses import asdict
            print(_json.dumps(asdict(sess), indent=2, sort_keys=True))
        else:
            print(f"session started: {sess.id}")
            print(f"  actor : {sess.actor}")
            print(f"  scope : {sess.scope}")
            print(f"  host  : {sess.host}")
            print(f"  expires at ns: {sess.expires_at_ns}")
        return 0

    if action == "end":
        if not args.id:
            sys.stderr.write("session end requires --id\n")
            return 2
        try:
            summary = end_session(store, args.id, actor=_actor(args))
        except FileNotFoundError as exc:
            sys.stderr.write(f"{exc}\n")
            return 2
        if args.json:
            print(_json.dumps(summary, indent=2, sort_keys=True))
        else:
            print(f"session ended: {summary['id']}  "
                  f"duration={summary['duration_ns'] / 1e9:.1f}s")
        return 0

    if action == "list":
        sessions = list_sessions(store)
        if args.json:
            from dataclasses import asdict
            print(_json.dumps(
                [asdict(s) for s in sessions],
                indent=2, sort_keys=True,
            ))
        else:
            print(format_sessions(sessions))
        return 0 if sessions or not args.exit_code else 1

    sys.stderr.write(f"unknown session action: {action!r}\n")
    return 2


def _cmd_serve(args: argparse.Namespace) -> int:
    """Run the local read-only HTTP API (PROPOSAL.md P10)."""
    from cyberos.core.serve import (  # noqa: WPS433
        ServeConfig, get_or_create_token, reset_token, serve_forever,
    )

    store = _store(args)
    if args.reset_token:
        new = reset_token(store)
        print(f"new bearer token: {new}")
        return 0
    if args.print_token:
        print(get_or_create_token(store))
        return 0

    cfg = ServeConfig(store=store, host=args.host, port=args.port)
    token = get_or_create_token(store)
    print(f"cyberos serve — http://{cfg.host}:{cfg.port}")
    print(f"  Authorization: Bearer {token}")
    print()
    print("  curl -H 'Authorization: Bearer <token>' http://localhost:%d/state" % cfg.port)
    print()
    serve_forever(cfg)
    return 0


def _cmd_publish(args: argparse.Namespace) -> int:
    """Produce a single-file mobile-friendly static site (PROPOSAL.md P12)."""
    from cyberos.core.publish import publish_to_file  # noqa: WPS433
    import json as _json

    kinds = (
        [k.strip() for k in args.kinds.split(",") if k.strip()]
        if args.kinds else None
    )
    exclude_kinds = (
        [k.strip() for k in args.exclude_kinds.split(",") if k.strip()]
        if args.exclude_kinds else None
    )

    summary = publish_to_file(
        _store(args),
        Path(args.out).expanduser().resolve(),
        kinds=kinds,
        exclude_kinds=exclude_kinds,
        max_body_chars=args.max_body_chars,
        deterministic=args.deterministic,
    )
    if args.json:
        print(_json.dumps(summary, indent=2, sort_keys=True))
    else:
        print(f"  wrote   : {summary['out_path']}")
        print(f"  bytes   : {summary['bytes']:,}")
        print(f"  memories: {summary['n_memories']}")
        print(f"  sha256  : {summary['sha256'][:16]}…")
    return 0


def _cmd_digest(args: argparse.Namespace) -> int:
    """Daily summary of audit activity (PROPOSAL.md P8)."""
    from cyberos.core.digest import (  # noqa: WPS433
        build, claude_prose, format_markdown, format_text, parse_human_duration,
    )
    import time as _time

    until_ns = _time.time_ns() if not args.until else int(args.until)
    if args.since:
        # Two interpretations: an int (epoch ns) or a human duration like "24h".
        try:
            since_ns = until_ns - parse_human_duration(args.since)
        except ValueError:
            try:
                since_ns = int(args.since)
            except ValueError as exc:
                sys.stderr.write(f"error: --since {args.since!r}: {exc}\n")
                return 2
    else:
        since_ns = until_ns - parse_human_duration(args.window)

    digest = build(
        _store(args),
        since_ns=since_ns,
        until_ns=until_ns,
        highlight_cap=args.highlight_cap,
    )

    if args.format == "json":
        print(digest.to_json())
    elif args.format == "markdown":
        print(format_markdown(digest))
    else:
        print(format_text(digest))

    if args.via_claude:
        prose = claude_prose(digest, model=args.claude_model)
        print()
        print("── prose summary ──")
        print(prose)

    return 0


def _cmd_resolve_conflict(args: argparse.Namespace) -> int:
    """List, diff, or resolve sync-FS conflict siblings (PROPOSAL.md P9)."""
    from cyberos.core.conflicts import (  # noqa: WPS433
        diff, format_scan, resolve_conflict, scan,
    )

    store = _store(args)

    if args.list or not args.path:
        pairs = scan(store)
        print(format_scan(pairs))
        return 0 if not pairs else 1

    target = (store / args.path).resolve()
    pairs = [p for p in scan(store) if p.canonical == target]
    if not pairs:
        print(f"no sync-FS conflicts found for {args.path}")
        return 0
    pair = pairs[0]

    if args.diff or not args.keep:
        # Default action with a target path: print diffs.
        for i, (sibling, source) in enumerate(sorted(pair.siblings, key=lambda s: s[0].name), 1):
            print(f"# sibling {i}: [{source}] {sibling.name}")
            d = diff(pair.canonical, sibling)
            if not d:
                print("  (byte-identical to canonical — safe to discard)\n")
            else:
                print(d)
        return 0

    # Active resolution requested.
    result = resolve_conflict(
        store, pair.canonical,
        keep=args.keep,
        actor=_actor(args),
        dry_run=args.dry_run,
    )
    import json as _json
    print(_json.dumps(result, indent=2, sort_keys=True))
    if (
        not args.dry_run
        and args.keep.startswith("sibling:")
        and "next_step" in result
    ):
        print()
        print(f"NEXT: {result['next_step']}")
    return 0


def _cmd_episode_log(args: argparse.Namespace) -> int:
    """Append an Episode to the memory (TASK-MEMORY-112 §1 #8).

    Constructs an ``Episode`` from CLI flags, routes through
    ``cyberos.core.episode.log`` → ``cyberos.core.ops.put``. Emits one
    ``op="put"`` audit row whose ``extra.kind="episode"`` so TASK-MEMORY-115
    dream + TASK-MEMORY-120 history can project the rich shape without
    re-parsing bodies.
    """
    from cyberos.core.episode import Episode, log as episode_log
    from cyberos.core.writer import Writer

    try:
        ep = Episode(
            task=args.task,
            approach=args.approach,
            outcome=args.outcome,
            duration_ms=args.duration_ms,
            token_cost=args.token_cost,
            quality_score=args.quality_score,
            notes=args.notes or "",
            error=args.error,
        )
    except ValueError as exc:
        sys.stderr.write(f"error: {exc}\n")
        return 2

    with Writer(_store(args)) as writer:
        seq, rel_path = episode_log(writer, ep, actor=_actor(args))

    if args.json:
        import json
        print(json.dumps({"seq": seq, "path": rel_path}, indent=2))
    else:
        print(f"seq={seq} path={rel_path}")
    return 0


def _cmd_recall_similar(args: argparse.Namespace) -> int:
    """Find episodes similar to ``task`` (TASK-MEMORY-112 §1 #9).

    Filters semantic / FTS5 hits to ``kind="episode"``, ranks by combined
    score per TASK-MEMORY-113. Returns JSON to stdout for scripting, or a
    human table by default.
    """
    from cyberos.core.episode import recall_similar

    backend = "fts5" if args.fts5 else ("semantic" if args.semantic else "auto")
    result = recall_similar(
        _store(args),
        args.task,
        k=args.k,
        min_relevance=args.min_relevance,
        backend=backend,
    )

    if args.json:
        import json
        print(json.dumps(result, indent=2, default=str))
        return 0

    if not result["matches"]:
        print(f"No similar episodes found (reason: {result['reason']}).")
        return 0

    print(f"Backend: {result['backend']}  ·  {len(result['matches'])} match(es)")
    print()
    for m in result["matches"]:
        qs = m.get("quality_score")
        qs_s = f"qs={qs:.2f}" if qs is not None else "qs=—"
        print(
            f"  [{m['combined_score']:.3f}] rel={m['relevance']:.2f} {qs_s} "
            f"outcome={m['outcome']}  {m['path']}"
        )
        print(f"             task: {m['task'][:120]}")
        print(f"             approach: {m['approach'][:120]}")
    return 0


def _cmd_dream(args: argparse.Namespace) -> int:
    """Run one dream pass (TASK-MEMORY-115 §1 #1, §1 #8).

    Always produces a ``dreams/<ts>/diff.json`` artefact. The diff is
    advisory until ``cyberos dream apply <id>`` is invoked.
    """
    from datetime import timedelta
    from cyberos.core.dream.runner import run_sync
    from cyberos.core.writer import Writer

    # Parse --since "24h" / "7d" / "30d" / ISO timestamp
    since_str: str = args.since or "24h"
    if since_str.endswith("h"):
        since = timedelta(hours=int(since_str[:-1]))
    elif since_str.endswith("d"):
        since = timedelta(days=int(since_str[:-1]))
    else:
        from datetime import datetime, timezone
        dt = datetime.fromisoformat(since_str.replace("Z", "+00:00"))
        since = datetime.now(timezone.utc) - dt

    detectors = args.detectors.split(",") if args.detectors else ("duplicates", "stale", "patterns", "verify")

    with Writer(_store(args)) as writer:
        diff = run_sync(
            writer,
            since=since,
            scope=args.scope or "",
            detector_names=tuple(detectors),
            invoker_name=args.invoker,
            dry_run=args.dry_run,
            duplicates_threshold=args.threshold,
        )

    if args.json:
        import json
        print(json.dumps(diff.to_dict(), indent=2, sort_keys=True))
    else:
        kinds = diff.metrics.get("proposals_count_by_kind", {})
        print(f"dream_id: {diff.dream_id}")
        print(f"scope:    {diff.scope}")
        print(f"since:    {diff.since}")
        print(f"snapshot_head: {diff.metrics.get('snapshot_head')}")
        print(f"duration_ms:   {diff.metrics.get('duration_ms')}")
        print(f"proposals:     {len(diff.proposals)} total "
              f"(merge={kinds.get('merge', 0)} stale={kinds.get('stale', 0)} "
              f"new={kinds.get('new', 0)} verify={kinds.get('verify', 0)})")
        if args.dry_run:
            print("(dry-run: no apply rows emitted; diff persisted to disk)")
        else:
            print(f"Apply with: cyberos dream apply {diff.dream_id}")
    return 0


def _cmd_dream_apply(args: argparse.Namespace) -> int:
    """Apply selected proposals from a previous dream run
    (TASK-MEMORY-115 §1 #4, §1 #11, §1 #14)."""
    import json
    from pathlib import Path
    from cyberos.core.dream.proposals import DreamDiff
    from cyberos.core.dream.applier import (
        apply, PreconditionFailed, ProtocolAmendmentMissing,
    )
    from cyberos.core.writer import Writer

    # Locate the diff file by dream_id (search dreams/*/diff.json)
    dreams_root = _store(args) / "dreams"
    target_diff: Path | None = None
    if dreams_root.is_dir():
        for diff_path in dreams_root.glob("*/diff.json"):
            try:
                data = json.loads(diff_path.read_text())
                if data.get("dream_id") == args.dream_id:
                    target_diff = diff_path
                    break
            except Exception:
                continue
    if target_diff is None:
        sys.stderr.write(
            f"error: no diff.json found with dream_id={args.dream_id!r} "
            f"under {dreams_root}\n"
        )
        return 2

    diff = DreamDiff.from_dict(json.loads(target_diff.read_text()))
    proposal_ids = set(args.proposal_ids.split(",")) if args.proposal_ids else None

    try:
        with Writer(_store(args)) as writer:
            summary = apply(
                writer, diff,
                proposal_ids=proposal_ids,
                actor=_actor(args) if args.actor else "dream-applier",
                enforce_section_7_7=not args.no_check_protocol,
            )
    except ProtocolAmendmentMissing as exc:
        sys.stderr.write(f"error: {exc}\n")
        return 3
    except PreconditionFailed as exc:
        sys.stderr.write(f"error: {exc}\n")
        return 4

    if args.json:
        print(json.dumps(summary, indent=2, default=str))
    else:
        print(f"dream_id: {summary['dream_id']}")
        print(f"applied:  {summary['applied_count']}")
        print(f"rejected: {summary['rejected']}")
        if summary["errors"]:
            print("errors:")
            for e in summary["errors"]:
                print(f"  - {e}")
    return 0


def _cmd_history(args: argparse.Namespace) -> int:
    """`cyberos history <path>` — per-path version + attribution view
    (TASK-MEMORY-120). Pure read-only projection over the audit chain.
    """
    from datetime import datetime, timedelta, timezone
    from cyberos.core.history import walk, render_human

    # Parse --since: 24h | 7d | ISO
    since_dt: "datetime | None" = None
    if args.since:
        s = args.since
        if s.endswith("h"):
            since_dt = datetime.now(timezone.utc) - timedelta(hours=int(s[:-1]))
        elif s.endswith("d"):
            since_dt = datetime.now(timezone.utc) - timedelta(days=int(s[:-1]))
        else:
            try:
                since_dt = datetime.fromisoformat(s.replace("Z", "+00:00"))
            except ValueError:
                sys.stderr.write(f"error: --since must be Nh, Nd, or ISO; got {s!r}\n")
                return 2

    entries = walk(
        _store(args),
        args.path,
        follow_moves=not args.no_follow_moves,
        since=since_dt,
        limit=args.limit,
        show_body=args.show_body,
    )
    if args.chronological:
        entries.reverse()

    if args.json:
        import json
        print(json.dumps([e.to_dict() for e in entries], indent=2, default=str))
        return 0

    if not entries:
        print(f"No history for {args.path!r}.")
        return 0
    for e in entries:
        print(render_human(e, show_body=args.show_body))
    return 0


def _cmd_transcript(args: argparse.Namespace) -> int:
    """`cyberos transcript {start|append|end|read|list|purge-expired}` per TASK-MEMORY-119.

    Namespaced under ``transcript`` (not ``session``) because the existing
    P11 ``cyberos session`` subcommand covers multi-agent coordination —
    a different product with the same verb in the spec.
    """
    from cyberos.core.transcript import (
        ProtocolAmendmentMissing, TranscriptError,
        active_session_id, append, end, list_sessions,
        purge_expired, read, start,
    )
    from cyberos.core.writer import Writer
    from datetime import timedelta
    import json

    action = args.transcript_action
    store = _store(args)

    if action == "start":
        try:
            with Writer(store) as w:
                s = start(
                    w,
                    session_id=args.id,
                    classification=args.classification,
                    retention_days=args.retention_days,
                    actor=_actor(args),
                )
        except ProtocolAmendmentMissing as e:
            sys.stderr.write(f"error: {e}\n")
            return 3
        except (ValueError, TranscriptError) as e:
            sys.stderr.write(f"error: {e}\n")
            return 2
        if args.json:
            print(json.dumps({
                "id": s.id,
                "started_at": s.started_at.isoformat(),
                "classification": s.classification,
                "retention_days": s.retention_days,
                "binlog_path": str(s.binlog_path),
            }, indent=2))
        else:
            print(f"session_id:     {s.id}")
            print(f"started_at:     {s.started_at.isoformat()}")
            print(f"classification: {s.classification}")
            print(f"binlog_path:    {s.binlog_path.relative_to(store) if s.binlog_path else '(unknown)'}")
        return 0

    if action == "append":
        try:
            with Writer(store) as w:
                seq = append(
                    w,
                    session_id=args.id,
                    role=args.role,
                    content=args.content,
                    redactions_applied=args.redactions_applied,
                )
        except (ValueError, TranscriptError) as e:
            sys.stderr.write(f"error: {e}\n")
            return 2
        if args.json:
            print(json.dumps({"turn_seq": seq}))
        else:
            print(f"turn_seq: {seq}")
        return 0

    if action == "end":
        try:
            with Writer(store) as w:
                s = end(
                    w,
                    session_id=args.id,
                    reason=args.reason,
                    seal_binlog=not args.no_seal,
                )
        except TranscriptError as e:
            sys.stderr.write(f"error: {e}\n")
            return 2
        if args.json:
            print(json.dumps({
                "id": s.id,
                "ended_at": s.ended_at.isoformat() if s.ended_at else None,
                "ended_reason": s.ended_reason,
                "binlog_path": str(s.binlog_path) if s.binlog_path else None,
            }, indent=2))
        else:
            print(f"session_id: {s.id}")
            print(f"ended_at:   {s.ended_at.isoformat() if s.ended_at else '(unknown)'}")
            print(f"binlog_at:  {s.binlog_path.relative_to(store) if s.binlog_path else '(unknown)'}")
        return 0

    if action == "read":
        turns = read(store, args.id, decrypt=args.decrypt)
        if args.json:
            print(json.dumps(turns, indent=2, default=str))
        else:
            if not turns:
                print(f"(no turns for session {args.id!r})")
                return 0
            for t in turns:
                if t.get("tombstone"):
                    print(f"[TOMBSTONE] purged_at={t.get('purged_at')}")
                    continue
                print(f"[{t.get('turn_seq', '?')}] {t.get('ts','?')} {t.get('role','?')}: "
                      f"{t.get('content','(encrypted)')[:200]}")
        return 0

    if action == "list":
        since = None
        if args.since:
            if args.since.endswith("h"):
                since = timedelta(hours=int(args.since[:-1]))
            elif args.since.endswith("d"):
                since = timedelta(days=int(args.since[:-1]))
        sessions = list_sessions(store, since=since)
        if args.json:
            print(json.dumps(sessions, indent=2, default=str))
        else:
            if not sessions:
                print("(no sessions)")
                return 0
            for s in sessions:
                print(f"  {s['session_id']:<32} {s['started_date']}  {s['state']:<8}  {s['binlog_path']}")
        # Indicate active session
        active = active_session_id(store)
        if active and not args.json:
            print(f"\nActive: {active}")
        return 0

    if action == "purge-expired":
        with Writer(store) as w:
            result = purge_expired(
                w,
                retention_days=args.retention_days,
                dry_run=args.dry_run,
            )
        if args.json:
            print(json.dumps(result, indent=2))
        else:
            print(f"purged_count: {result['purged_count']}")
            if result["dry_run"]:
                print("(dry-run: nothing actually purged)")
            for p in result["purged"]:
                print(f"  - {p['session_id']:<32} {p['date']} (age {p['age_days']}d)")
        return 0

    sys.stderr.write(f"error: unknown transcript action {action!r}\n")
    return 2


def _cmd_put_if(args: argparse.Namespace) -> int:
    """`cyberos put-if <path> <body_file> --precondition <hex|none>`.

    Per TASK-MEMORY-118 §1 #8. Atomic content-conditional write.
    """
    from cyberos.core.ops import put_if
    from cyberos.core.writer import Writer

    body = Path(args.body_file).read_bytes()

    # Resolve the precondition argument
    precondition: str | None
    raw = args.precondition.strip()
    if raw.lower() == "none":
        precondition = None
    elif args.precondition_from_file:
        precondition = Path(args.precondition_from_file).read_text().strip()
    else:
        precondition = raw

    try:
        with Writer(_store(args)) as writer:
            result = put_if(
                writer,
                args.path,
                body,
                actor=_actor(args),
                precondition_body_hash=precondition,
                kind=args.kind or "unknown",
            )
    except ValueError as exc:
        sys.stderr.write(f"error: {exc}\n")
        return 2
    except Exception as exc:
        # ProtocolAmendmentMissing or other infra error
        if "ProtocolAmendmentMissing" in type(exc).__name__:
            sys.stderr.write(f"error: {exc}\n")
            return 3
        raise

    if args.json:
        import json
        print(json.dumps({
            "outcome": result.outcome,
            "reason": result.reason,
            "expected": result.expected,
            "actual": result.actual,
            "committed_seq": result.committed_seq,
        }, indent=2))
    else:
        print(f"outcome: {result.outcome}")
        if result.reason:
            print(f"reason:  {result.reason}")
        if result.expected is not None:
            print(f"expected: {result.expected[:16]}…")
        if result.actual is not None:
            actual_disp = result.actual if result.actual.startswith("<") else f"{result.actual[:16]}…"
            print(f"actual:   {actual_disp}")
        if result.committed_seq is not None:
            print(f"seq:      {result.committed_seq}")

    return 0 if result.outcome == "written" else 1


def _cmd_acl(args: argparse.Namespace) -> int:
    """`cyberos acl {show|validate|explain}` — TASK-MEMORY-117 §1 #12."""
    import json
    from cyberos.core.store_acl import StoreAcl, check_write, explain

    store = _store(args)
    action = args.acl_action

    if action == "show":
        yaml_paths = sorted(store.rglob("STORE.yaml"))
        if not yaml_paths:
            print("(no STORE.yaml files in store)")
            return 0
        for yml in yaml_paths:
            try:
                acl = StoreAcl.from_yaml(yml)
            except ValueError as e:
                print(f"{yml.relative_to(store)}: INVALID ({e})")
                continue
            print(f"{yml.relative_to(store)}:")
            print(f"  store_id:     {acl.store_id}")
            print(f"  default_mode: {acl.default_mode}")
            print(f"  acl:")
            for actor, mode in acl.acl:
                print(f"    - {actor:<24} → {mode}")
        return 0

    if action == "validate":
        yaml_paths = sorted(store.rglob("STORE.yaml"))
        errors = 0
        for yml in yaml_paths:
            try:
                StoreAcl.from_yaml(yml)
                print(f"  OK    {yml.relative_to(store)}")
            except ValueError as e:
                print(f"  FAIL  {yml.relative_to(store)}: {e}")
                errors += 1
        if errors:
            print(f"\n{errors} STORE.yaml file(s) failed validation")
        return 1 if errors else 0

    if action == "explain":
        if not args.path:
            sys.stderr.write("error: `cyberos acl explain <path>` requires a memory path\n")
            return 2
        actor = args.actor or _actor(args)
        result = explain(store, args.path, actor)
        if args.json:
            print(json.dumps(result, indent=2, default=str))
        else:
            print(f"Path:           {result['path']}")
            print(f"Actor:          {result['actor']}")
            print(f"Governing YAML: {result['yaml_path'] or '(none — permissive default)'}")
            print(f"Store ID:       {result['store_id']}")
            print(f"Effective mode: {result['effective_mode']}")
            print(f"Matched entry:  {result['matched_entry']}")
            print(f"Allow write:    {result['allowed_write']}")
            print(f"WARN-ONLY mode: {result['warn_only_active']}")
            if result["reason"]:
                print(f"Reason:         {result['reason']}")
        return 0

    sys.stderr.write(f"error: unknown acl action: {action}\n")
    return 2


def _cmd_doctor(args: argparse.Namespace) -> int:
    """Run the self-audit walker.

    Iterates every invariant in ``memory/docs/memory.invariants.yaml``,
    runs the corresponding check function in :mod:`cyberos.core.invariants`,
    and prints a structured report. Exits non-zero on any ``error``-level
    failure; ``warning``-level failures print but don't fail the exit code.

    With ``--repair``, attempts auto-fix for safe recoverable failures
    (layout shard uniformity, index/manifest regeneration). Catastrophic
    failures (chain corruption, MMR cross-check, unparseable manifest)
    are NEVER auto-repaired — they require human review.
    """
    from cyberos.core.invariants import format_report, repair, run_all  # noqa: WPS433

    only: list[str] | None = None
    if args.only:
        only = [s.strip() for s in args.only.split(",") if s.strip()]

    if args.repair:
        # Run repairs first (they may resolve some invariants), then re-walk.
        actions = repair(_store(args), only=only)
        for a in actions:
            tag = "FIXED" if a.fixed else "SKIP "
            print(f"  [{tag}] {a.invariant_id}: {a.details}")
        if actions:
            print()
        # Fall through to a fresh walk so the report reflects post-repair state.

    report = run_all(_store(args), only=only)
    print(format_report(report, json_mode=args.json))
    return 0 if report.ok else 1


def _cmd_init(args: argparse.Namespace) -> int:
    """Initialize a new BRAIN store and optionally auto-index + auto-digest.

    Creates the ``.cyberos/memory/store/`` directory structure with manifest.json,
    HEAD, audit/, memories/<kind>/, meta/, and exports/. With ``--auto-index``,
    builds the FTS5 index from existing memory files (bypassing binlog).
    With ``--auto-digest``, ingests project knowledge (README.md, docs/, etc.)
    into memory files.

    Tracks the cyberos release version in ``manifest.json:cyberos_version``.
    On ``--force`` re-init, detects version changes and prints migration info.
    """
    import hashlib
    import json
    import os
    import struct
    import time

    store = _store(args)
    source_repo = _find_source_repo()
    current_version = _read_source_version(source_repo) if source_repo else None

    # --- Phase 1: Create directory structure ---
    is_reinit = False
    stored_version = None
    if store.exists() and (store / "manifest.json").is_file():
        existing_manifest = json.loads((store / "manifest.json").read_text())
        stored_version = existing_manifest.get("cyberos_version")
        if not args.force:
            # Check if version changed — auto-migrate without --force
            if stored_version and current_version and stored_version != current_version:
                print(f"  Version change detected: {stored_version} → {current_version}")
                print(f"  Migrating BRAIN at {store}...")
                is_reinit = True
            else:
                sys.stderr.write(
                    f"BRAIN store already exists at {store}\n"
                    "Use --force to reinitialize (preserves existing memories)\n"
                )
                return 1
        else:
            print(f"Reinitializing existing BRAIN at {store}")
            is_reinit = True
    else:
        print(f"Creating new BRAIN at {store}")

    # Create directory structure (superset of all known layouts)
    dirs = [
        store / "audit",
        store / "audit" / "checkpoints",
        store / "memories" / "decisions",
        store / "memories" / "facts",
        store / "memories" / "people",
        store / "memories" / "projects",
        store / "memories" / "preferences",
        store / "memories" / "drift",
        store / "memories" / "refinements",
        store / "meta",
        store / "company",
        store / "module",
        store / "member",
        store / "client",
        store / "project",
        store / "persona",
        store / "conflicts",
        store / "exports",
        store / "index",
    ]
    for d in dirs:
        d.mkdir(parents=True, exist_ok=True)

    # Initialize HEAD (8-byte LE u64 sequence counter)
    head_path = store / "HEAD"
    if not head_path.exists():
        fd = os.open(str(head_path), os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o644)
        try:
            os.write(fd, struct.pack("<Q", 0))  # LE u64 = 0
            if hasattr(os, "fdatasync"):
                os.fdatasync(fd)
            else:
                os.fsync(fd)
        finally:
            os.close(fd)

    # Initialize or update manifest.json
    manifest_path = store / "manifest.json"
    if manifest_path.exists():
        manifest = json.loads(manifest_path.read_text())
    else:
        manifest = {}

    manifest.setdefault("version", 2)
    manifest.setdefault("created_at_ns", time.time_ns())
    manifest.setdefault("fingerprint", hashlib.sha256(
        str(store.resolve()).encode()
    ).hexdigest()[:16])
    manifest.setdefault("actor", args.actor or "cyberos-cli")
    manifest.setdefault("crypto_mode", "chained")
    manifest.setdefault("imports", {})
    if current_version:
        manifest["cyberos_version"] = current_version

    # Copy protocol files into the store (self-contained)
    if source_repo:
        # AGENTS.md may be at top-level or inside cyberos/data/
        agents_src = source_repo / "AGENTS.md"
        if not agents_src.is_file():
            agents_src = source_repo / "cyberos" / "data" / "AGENTS.md"
        if agents_src.is_file():
            agents_dst = store / "AGENTS.md"
            agents_dst.write_bytes(agents_src.read_bytes())
            print(f"  Copied AGENTS.md")
        for proto_file in ("memory.schema.json", "memory.invariants.yaml"):
            src = source_repo / proto_file
            if src.is_file():
                dst = store / proto_file
                dst.write_bytes(src.read_bytes())
                print(f"  Copied {proto_file}")

    # Write manifest atomically
    tmp = manifest_path.with_suffix(".tmp")
    tmp.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    os.replace(str(tmp), str(manifest_path))
    if is_reinit:
        print(f"  Updated manifest.json (cyberos_version={current_version})")
    else:
        print(f"  Wrote manifest.json")

    # --- Phase 2: Auto-index existing memories ---
    if args.auto_index:
        print("\nAuto-indexing existing memories...")
        _auto_index(store)

    # --- Phase 3: Auto-digest project knowledge ---
    if args.auto_digest:
        print("\nAuto-digesting project knowledge...")
        _auto_digest(store, args.actor or "cyberos-cli", limit=args.digest_limit)

    print(f"\nBRAIN initialized at {store}")
    if is_reinit:
        print(f"  Migrated from {stored_version or '(unknown)'} → {current_version}")
    print("The store is now ready. Run `cyberos doctor` to verify invariants.")
    return 0


def _auto_index(store: Path) -> None:
    """Build FTS5 index from existing memory files, bypassing binlog."""
    import hashlib
    import sqlite3

    from cyberos.core.index import open_index, write_index_manifest

    fingerprint = hashlib.sha256(str(store).encode("utf-8")).hexdigest()[:16]
    conn = open_index(fingerprint)
    try:
        # Clear existing FTS data
        conn.execute("DELETE FROM memories_fts")
        conn.execute("DELETE FROM memories")

        count = 0
        memories_dir = store / "memories"
        if not memories_dir.exists():
            print("  No memories/ directory found, skipping index")
            return

        for md_file in memories_dir.rglob("*.md"):
            rel_path = str(md_file.relative_to(store))
            if ".meta.json" in rel_path:
                continue

            # Read file content
            try:
                raw = md_file.read_bytes()
            except OSError:
                continue

            # Parse frontmatter to get kind
            kind = "unknown"
            body_text = raw.decode("utf-8", errors="replace")
            if raw.startswith(b"---\n"):
                end = raw.find(b"\n---\n", 4)
                if end > 0:
                    frontmatter = raw[4:end].decode("utf-8", errors="replace")
                    body_text = raw[end + 5:].decode("utf-8", errors="replace")
                    for line in frontmatter.split("\n"):
                        if line.startswith("kind:"):
                            kind = line.split(":", 1)[1].strip().strip('"').strip("'")
                            break

            # If no frontmatter kind, infer from path
            if kind == "unknown":
                parts = md_file.relative_to(memories_dir).parts
                if parts and parts[0] in (
                    "decisions", "facts", "people", "projects",
                    "preferences", "drift", "refinements",
                ):
                    kind = parts[0]

            # Compute content hash
            content_sha256 = hashlib.sha256(raw).hexdigest()

            # Insert into memories table
            ts_ns = int(md_file.stat().st_mtime * 1e9)
            conn.execute(
                """
                INSERT INTO memories(rel_path, kind, actor, ts_ns, content_sha256, last_seq, tombstoned)
                VALUES(?, ?, ?, ?, ?, 0, 0)
                ON CONFLICT(rel_path) DO UPDATE SET
                    kind = excluded.kind,
                    ts_ns = excluded.ts_ns,
                    content_sha256 = excluded.content_sha256,
                    tombstoned = 0
                """,
                (rel_path, kind, "auto-index", ts_ns, content_sha256),
            )

            # Insert into FTS index
            if body_text:
                conn.execute(
                    "DELETE FROM memories_fts WHERE rel_path = ?",
                    (rel_path,),
                )
                conn.execute(
                    "INSERT INTO memories_fts(rel_path, body) VALUES(?, ?)",
                    (rel_path, body_text),
                )

            count += 1

        conn.commit()
        print(f"  Indexed {count} memory files")
        write_index_manifest(store, 0)
    finally:
        conn.close()


def _auto_digest(store: Path, actor: str, limit: int = 0) -> None:
    """Digest project knowledge files into memory files."""
    import hashlib
    import time

    # Find project root: the store sits at <root>/.cyberos/memory/store
    # (unified) or <root>/.cyberos/memory/store (legacy). Strip the known suffix.
    if store.parts[-3:] == (".cyberos", "memory", "store"):
        project_root = store.parents[2]
    else:
        project_root = store.parent
    if not project_root.exists():
        print("  Cannot find project root, skipping digest")
        return

    # Files to digest — include READMEs, docs, and key project files
    digest_targets = []
    for pattern in [
        "README.md", "README.rst",
        "docs/*.md", "docs/**/*.md",
        "CONTRIBUTING.md", "CHANGELOG.md", "ARCHITECTURE.md",
        "AGENTS.md", "CLAUDE.md",
    ]:
        digest_targets.extend(project_root.glob(pattern))

    # Deduplicate and exclude audit/generated files
    seen = set()
    filtered = []
    exclude_patterns = [
        ".audit.md",  # audit companion files
        "node_modules/",  # dependencies
        ".git/",  # git internals
        "__pycache__/",  # python cache
        ".cyberos/",  # unified module tree (store lives here)
        ".cyberos/memory/store/",  # legacy memory store
    ]
    for target in digest_targets:
        rel = str(target.relative_to(project_root))
        if rel in seen:
            continue
        if any(excl in rel for excl in exclude_patterns):
            continue
        seen.add(rel)
        filtered.append(target)

    if not filtered:
        print("  No project knowledge files found to digest")
        return

    print(f"  Found {len(filtered)} knowledge files to digest")

    # Create digest memories (limit=0 means no limit)
    targets_to_digest = filtered if limit == 0 else filtered[:limit]
    for target in targets_to_digest:
        rel_path = target.relative_to(project_root)
        try:
            content = target.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue

        # Skip empty or very small files
        if len(content.strip()) < 50:
            continue

        # Create memory file
        ts = int(time.time())
        ts_ns = ts * 1_000_000_000
        content_hash = hashlib.sha256(content.encode()).hexdigest()[:8]
        memory_path = (
            store / "memories" / "facts" / content_hash[:2] / content_hash[2:4] /
            f"digest-{rel_path.name.replace('.', '-')}-{ts}.md"
        )
        memory_path.parent.mkdir(parents=True, exist_ok=True)

        # Build memory content with frontmatter
        frontmatter = {
            "kind": "facts",
            "actor": actor,
            "ts_ns": ts_ns,
            "content_sha256": hashlib.sha256(content.encode()).hexdigest(),
            "description": f"Auto-digested from {rel_path}",
            "source": str(rel_path),
        }

        import json as _json
        body = f"---\n{_json.dumps(frontmatter, indent=2, sort_keys=True)}\n---\n\n"
        body += f"# {rel_path.name}\n\n"
        body += content[:5000]  # Limit body size

        memory_path.write_text(body, encoding="utf-8")
        print(f"    Digested: {rel_path}")


# --- self-update -----------------------------------------------------------


def _find_source_repo() -> Path | None:
    """Find the modules/memory/ source repo from the installed package.

    With ``pip install -e .``, ``__file__`` points into the source tree.
    Walk up from ``cyberos/__main__.py`` to ``modules/memory/``.
    """
    pkg_dir = Path(__file__).resolve().parent          # cyberos/
    memory_dir = pkg_dir.parent                        # modules/memory/
    # Check both locations: top-level and inside cyberos/data/
    if (memory_dir / "AGENTS.md").is_file():
        return memory_dir
    if (memory_dir / "cyberos" / "data" / "AGENTS.md").is_file():
        return memory_dir
    # Fallback: check if protocol files exist at the top level
    if (memory_dir / "memory.schema.json").is_file():
        return memory_dir
    return None


def _read_source_version(source: Path) -> str | None:
    """Read the canonical version from the repo-root VERSION file.

    Falls back to parsing ``cyberos/__init__.py`` if VERSION is missing.
    """
    # Primary: repo-root VERSION file
    repo_root = source.parent.parent  # modules/memory/ → modules/ → repo root
    version_file = repo_root / "VERSION"
    if version_file.is_file():
        v = version_file.read_text().strip()
        if v:
            return v
    # Fallback: __init__.py
    init_file = source / "cyberos" / "__init__.py"
    for line in init_file.read_text().splitlines():
        if line.startswith("__version__"):
            return line.split("=")[1].strip().strip('"').strip("'")
    return None


def _read_manifest(store: Path) -> dict:
    """Read manifest.json, returning empty dict on missing/corrupt."""
    import json
    mf = store / "manifest.json"
    if not mf.is_file():
        return {}
    try:
        return json.loads(mf.read_text())
    except Exception:
        return {}


def _write_manifest(store: Path, data: dict) -> None:
    """Write manifest.json atomically."""
    import json
    import os
    mf = store / "manifest.json"
    tmp = mf.with_suffix(".tmp")
    tmp.write_text(json.dumps(data, indent=2) + "\n")
    os.replace(str(tmp), str(mf))


def _cmd_self_update(args: argparse.Namespace) -> int:
    """Check for a newer version in the source repo, update, and re-init.

    1. Locate the source repo (modules/memory/) from the installed package.
    2. Compare installed version with source ``__init__.py`` version.
    3. If newer, ``pip install -e .`` to update.
    4. Re-copy ``AGENTS.md`` into the current ``.cyberos/memory/store/`` store.
    5. Record the current version in ``manifest.json``.
    """
    import importlib
    import subprocess

    source = _find_source_repo()
    if source is None:
        print("error: cannot locate source repo (modules/memory/) from installed package")
        return 1

    # Read source version (from VERSION file or __init__.py)
    source_version = _read_source_version(source)
    if source_version is None:
        print("error: cannot read version from source repo")
        return 1

    # Installed version
    from cyberos import __version__ as installed_version

    store = _store(args)
    manifest = _read_manifest(store)
    store_version = manifest.get("cyberos_version", "0.0.0")

    print(f"  installed : {installed_version}")
    print(f"  source    : {source_version}")
    print(f"  store     : {store_version}")

    needs_update = installed_version != source_version
    needs_reinit = store_version != installed_version

    if not needs_update and not needs_reinit and not args.force:
        print("\n  already up to date.")
        return 0

    # Step 1: re-install package if source is newer
    if needs_update:
        print(f"\n  updating package {installed_version} → {source_version} ...")
        result = subprocess.run(
            [sys.executable, "-m", "pip", "install", "-e", str(source)],
            capture_output=True, text=True,
        )
        if result.returncode != 0:
            print(f"  pip install failed:\n{result.stderr}")
            return 1
        print("  ✓ package updated")
        # Reload the version
        importlib.reload(importlib.import_module("cyberos"))

    # Step 2: re-copy protocol files into the store
    import shutil
    for name in ("AGENTS.md", "memory.schema.json", "memory.invariants.yaml"):
        src = source / name
        dst = store / name
        if src.is_file():
            shutil.copy2(str(src), str(dst))
    print(f"  ✓ protocol files synced into {store}")

    # Step 3: record version in manifest
    from cyberos import __version__ as new_version
    manifest["cyberos_version"] = new_version
    _write_manifest(store, manifest)
    print(f"  ✓ manifest.json cyberos_version → {new_version}")

    print("\n  done. run `cyberos doctor` to verify.")
    return 0


def _maybe_check_version(store: Path) -> None:
    """On first run, print a hint if the store version is stale.

    Non-blocking — prints a warning to stderr and returns immediately.
    """
    try:
        from cyberos import __version__ as installed
        manifest = _read_manifest(store)
        store_ver = manifest.get("cyberos_version")
        if store_ver and store_ver != installed:
            sys.stderr.write(
                f"hint: cyberos store version ({store_ver}) differs from "
                f"installed ({installed}). run `cyberos self-update` to sync.\n"
            )
    except Exception:
        pass


# --- workflow subcommand (delegates to cyberos-cuo) ----------------------


_SUBCOMMANDS = {"list-personas", "list-workflows", "dry-run", "route"}


def _find_skill_root() -> Path | None:
    """Find the skill/ module directory.

    Walks up from CWD looking for modules/skill/MODULE.md.
    Also checks CYBEROS_ROOT env var.
    """
    import os
    env_root = os.environ.get("CYBEROS_ROOT")
    if env_root:
        candidate = Path(env_root) / "modules" / "skill"
        if (candidate / "MODULE.md").is_file():
            return candidate
    probe = Path.cwd()
    while True:
        candidate = probe / "modules" / "skill"
        if (candidate / "MODULE.md").is_file():
            return candidate
        parent = probe.parent
        if parent == probe:
            break
        probe = parent
    return None


def _cmd_skill(args: argparse.Namespace) -> int:
    """Dispatch ``cyberos skill`` to cyberos-skill binary or built-in Python."""
    import shutil
    import subprocess

    skill_args = args.skill_args or []

    if not skill_args or skill_args[0] in ("-h", "--help"):
        print("usage: cyberos skill <sub-command> [args]")
        print("")
        print("sub-commands:")
        print("  list                  list installed skills")
        print("  info <name>           show skill frontmatter + body preview")
        print("  run <name>            invoke a skill (reads JSON from stdin)")
        print("  validate [name ...]   validate SKILL.md files")
        return 0

    subcmd = skill_args[0]
    rest = skill_args[1:]

    # If cyberos-skill binary is on PATH, delegate to it
    binary = shutil.which("cyberos-skill")
    if binary:
        cmd = [binary]
        # Add --root if we can find the skill root
        skill_root = _find_skill_root()
        if skill_root:
            cmd.extend(["--root", str(skill_root)])
        cmd.append(subcmd)
        cmd.extend(rest)
        # For 'run', pass stdin through
        proc = subprocess.run(cmd, capture_output=(subcmd != "run"))
        if subcmd == "run":
            import sys
            proc = subprocess.run(cmd, stdin=sys.stdin)
        return proc.returncode

    # Built-in Python fallback for list/info/validate
    skill_root = _find_skill_root()
    if skill_root is None:
        sys.stderr.write(
            "error: cyberos-skill binary not on PATH and skill/ module not found.\n"
            "       Either:\n"
            "         1. cargo install --path modules/skill/crates/cli\n"
            "         2. Run from within a CyberOS project\n"
        )
        return 1

    if subcmd == "list":
        count = 0
        for d in sorted(skill_root.iterdir()):
            if d.is_dir() and (d / "SKILL.md").is_file():
                # Read first line of description from frontmatter
                desc = ""
                try:
                    txt = (d / "SKILL.md").read_text(encoding="utf-8")
                    for line in txt.splitlines():
                        if line.startswith("description:"):
                            desc = line.split(":", 1)[1].strip().strip('"')
                            break
                except (OSError, UnicodeDecodeError):
                    pass
                print(f"  {d.name:50s} {desc}")
                count += 1
        print(f"\n{count} skill(s) installed")
        return 0

    if subcmd == "info":
        if not rest:
            sys.stderr.write("error: info requires a skill name\n")
            return 2
        name = rest[0]
        skill_md = skill_root / name / "SKILL.md"
        if not skill_md.is_file():
            sys.stderr.write(f"error: skill not found: {skill_md}\n")
            return 1
        print(skill_md.read_text(encoding="utf-8")[:2000])
        return 0

    if subcmd == "validate":
        names = rest if rest else [
            d.name for d in sorted(skill_root.iterdir())
            if d.is_dir() and (d / "SKILL.md").is_file()
        ]
        errors = 0
        for name in names:
            skill_md = skill_root / name / "SKILL.md"
            if not skill_md.is_file():
                print(f"  FAIL {name}: SKILL.md not found")
                errors += 1
                continue
            try:
                txt = skill_md.read_text(encoding="utf-8")
                if not txt.startswith("---"):
                    print(f"  FAIL {name}: no YAML frontmatter")
                    errors += 1
                else:
                    print(f"  OK   {name}")
            except (OSError, UnicodeDecodeError) as e:
                print(f"  FAIL {name}: {e}")
                errors += 1
        return 1 if errors else 0

    if subcmd == "run":
        sys.stderr.write(
            "error: 'cyberos skill run' requires cyberos-skill binary on PATH.\n"
            "       For prompt-only skills, use 'cyberos workflow' which invokes LLM directly.\n"
        )
        return 1

    sys.stderr.write(f"error: unknown skill sub-command '{subcmd}'\n")
    return 2


def _auto_init_if_needed() -> None:
    """Check if memory store exists; auto-init if not."""
    # §0.4: the store is .cyberos/memory/store at the project root.
    store_rel = Path(".cyberos") / "memory" / "store"
    # Walk up from CWD looking for an existing store
    probe = Path.cwd()
    while True:
        if (probe / store_rel).is_dir():
            return  # already initialized
        parent = probe.parent
        if parent == probe:
            break
        probe = parent
    # Not found — auto-init a fresh store at the canonical path in CWD
    sys.stderr.write("cyberos: memory store not found — running auto-init...\n")
    store_root = Path.cwd() / store_rel
    store_root.mkdir(parents=True, exist_ok=True)
    # Create directory structure
    for subdir in [
        "memories/decisions", "memories/facts", "memories/people",
        "memories/projects", "memories/preferences", "memories/drift",
        "memories/refinements",
        "meta/company", "meta/module", "meta/member", "meta/client",
        "meta/project", "meta/persona",
        "conflicts", "exports", "audit/checkpoints",
    ]:
        (store_root / subdir).mkdir(parents=True, exist_ok=True)
    # Write HEAD
    (store_root / "HEAD").write_bytes(b"\x00" * 8)
    # Write manifest
    import json as _json
    manifest = {
        "version": 2,
        "created_at": __import__("datetime").datetime.utcnow().isoformat() + "Z",
        "audit_chain_head": "",
    }
    (store_root / "manifest.json").write_text(
        _json.dumps(manifest, indent=2, sort_keys=True), encoding="utf-8"
    )
    # Copy protocol files from cyberos source repo if available
    src_repo = _find_source_repo()
    if src_repo:
        import shutil
        for fname in ("AGENTS.md", "memory.schema.json", "memory.invariants.yaml"):
            # Check both locations: top-level and cyberos/data/
            src = src_repo / fname
            if not src.is_file():
                src = src_repo / "cyberos" / "data" / fname
            if src.is_file():
                shutil.copy2(src, Path.cwd() / fname)
    # Add the memory store to .gitignore if not already present.
    # ".cyberos/" covers the unified store (.cyberos/memory/store).
    gitignore = Path.cwd() / ".gitignore"
    markers = [".cyberos/"]
    try:
        existing = gitignore.read_text(encoding="utf-8") if gitignore.is_file() else ""
    except (OSError, UnicodeDecodeError):
        existing = ""
    missing = [m for m in markers if m not in existing]
    if missing:
        entry = "\n# CyberOS\n" + "\n".join(f"{m}/" for m in missing) + "\n"
        try:
            with open(gitignore, "a", encoding="utf-8") as f:
                f.write(entry)
            sys.stderr.write(f"cyberos: added {', '.join(m + '/' for m in missing)} to .gitignore\n")
        except OSError:
            pass
    sys.stderr.write(f"cyberos: initialized memory store at {store_root}\n")


def _cmd_workflow(args: argparse.Namespace) -> int:
    """Dispatch ``cyberos workflow`` to the CUO API.

    If the first arg after ``workflow`` is a known sub-command, dispatch to it.
    Otherwise treat it as a workflow path and run the drain loop.
    """
    try:
        from cuo import api as cuo_api
    except ImportError:
        sys.stderr.write(
            "error: cyberos-cuo package not installed.\n"
            "       Install with: pip install -e /path/to/cyberos/modules/cuo\n"
        )
        return 1

    # Auto-init memory store if not found
    _auto_init_if_needed()

    wf_args = args.workflow_args or []

    if not wf_args or wf_args[0] in ("-h", "--help"):
        print("usage: cyberos workflow <persona-workflow> [--rework] [options]")
        print("       cyberos workflow list-personas [--show-extinct]")
        print("       cyberos workflow list-workflows <persona-slug>")
        print("       cyberos workflow dry-run <persona-workflow>")
        print("")
        print("Run 'cyberos workflow <cmd> --help' for more info on a command.")
        return 0

    subcmd = wf_args[0]

    if subcmd == "list-personas":
        show_extinct = "--show-extinct" in wf_args
        cuo_api.list_personas(show_extinct=show_extinct)
        return 0

    if subcmd == "list-workflows":
        if len(wf_args) < 2:
            sys.stderr.write("error: list-workflows requires a persona-slug\n")
            return 2
        cuo_api.list_workflows(wf_args[1])
        return 0

    if subcmd == "dry-run":
        if len(wf_args) < 2:
            sys.stderr.write("error: dry-run requires a persona-workflow path\n")
            return 2
        cuo_api.dry_run(wf_args[1])
        return 0

    if subcmd == "route":
        if len(wf_args) < 2:
            sys.stderr.write("error: route requires a query string\n")
            return 2
        from cuo.core.router import route
        from cuo.core.catalog import discover_personas
        from cuo.cli import _resolve_roots
        cuo_path, _ = _resolve_roots(None, None)
        personas = discover_personas(cuo_path)
        decision = route(" ".join(wf_args[1:]), personas)
        if decision is None:
            print("NO_MATCH")
            return 1
        print(f"{decision.persona_slug}/{decision.workflow_slug} (confidence={decision.confidence:.2f})")
        return 0

    # Default: treat as workflow path → run drain
    persona_workflow = subcmd
    # Parse remaining flags
    rework = "--rework" in wf_args
    output_dir = None
    module = None
    backlog_path = None
    max_frs = 0
    invoker = "auto"
    no_memory_emit = "--no-memory-emit" in wf_args
    actor = "cuo-drain"
    halt_on_repeat_rework = 2

    # Simple flag parsing for the remaining args
    i = 1  # skip the workflow path
    while i < len(wf_args):
        arg = wf_args[i]
        if arg == "--rework":
            rework = True
        elif arg == "--no-rework":
            rework = False
        elif arg == "--output-dir" and i + 1 < len(wf_args):
            i += 1
            output_dir = Path(wf_args[i])
        elif arg.startswith("--output-dir="):
            output_dir = Path(arg.split("=", 1)[1])
        elif arg == "--module" and i + 1 < len(wf_args):
            i += 1
            module = wf_args[i]
        elif arg.startswith("--module="):
            module = arg.split("=", 1)[1]
        elif arg == "--backlog" and i + 1 < len(wf_args):
            i += 1
            backlog_path = Path(wf_args[i])
        elif arg.startswith("--backlog="):
            backlog_path = Path(arg.split("=", 1)[1])
        elif arg == "--max-frs" and i + 1 < len(wf_args):
            i += 1
            max_frs = int(wf_args[i])
        elif arg == "--invoker" and i + 1 < len(wf_args):
            i += 1
            invoker = wf_args[i]
        elif arg == "--no-memory-emit":
            no_memory_emit = True
        elif arg == "--actor" and i + 1 < len(wf_args):
            i += 1
            actor = wf_args[i]
        elif arg == "--halt-on-repeat-rework" and i + 1 < len(wf_args):
            i += 1
            halt_on_repeat_rework = int(wf_args[i])
        elif arg in ("-h", "--help"):
            print("usage: cyberos workflow <persona-workflow> [options]")
            print("")
            print("options:")
            print("  --rework              re-run done FRs from implementing to done")
            print("  --output-dir DIR      directory for per-FR artefacts (default: cwd)")
            print("  --backlog PATH        path to BACKLOG.md (default: auto-discover)")
            print("  --module MODULE       filter FRs by module slug")
            print("  --max-frs N           max FRs to drain (0 = unbounded)")
            print("  --invoker INVOKER     auto|subprocess|llm")
            print("  --no-memory-emit      skip memory audit emission")
            print("  --actor NAME          actor name for memory rows")
            print("  --halt-on-repeat-rework N  halt after N re-routes (default 2)")
            return 0
        else:
            sys.stderr.write(f"error: unknown option '{arg}'\n")
            return 2
        i += 1

    cuo_api.run(
        persona_workflow,
        output_dir=output_dir,
        module=module,
        backlog_path=backlog_path,
        max_frs=max_frs,
        invoker=invoker,
        memory_emit=not no_memory_emit,
        actor=actor,
        halt_on_repeat_rework=halt_on_repeat_rework,
        rework=rework,
    )
    return 0


# --- argparse wiring ------------------------------------------------------


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="cyberos", description="CyberOS Layer-1 CLI")
    p.add_argument(
        "--store", default=".cyberos/memory/store",
        help=(
            "store root. If not given, auto-discovers by walking up from "
            "CWD looking for .cyberos/memory/store/. Env var CYBEROS_STORE also works."
        ),
    )
    p.add_argument("--actor", default=None, help="principal identifier for audit rows")
    sub = p.add_subparsers(dest="cmd", required=True)

    sp = sub.add_parser(
        "init",
        help="initialize a new BRAIN store (auto-index + auto-digest optional)",
    )
    sp.add_argument(
        "--auto-index", action="store_true",
        help="build FTS5 index from existing memory files",
    )
    sp.add_argument(
        "--auto-digest", action="store_true",
        help="ingest project knowledge (README.md, docs/) into memory",
    )
    sp.add_argument(
        "--digest-limit", type=int, default=0,
        help="max files to digest with --auto-digest (0 = no limit, default)",
    )
    sp.add_argument(
        "--force", action="store_true",
        help="reinitialize existing store (preserves memories)",
    )
    sp.set_defaults(fn=_cmd_init)

    sp = sub.add_parser("view", help="read a memory file")
    sp.add_argument("path")
    sp.set_defaults(fn=_cmd_view)

    sp = sub.add_parser("put", help="canonical op: create-or-replace a memory file")
    sp.add_argument("path")
    sp.add_argument("body_file")
    sp.add_argument("--kind", default=None)
    # TASK-MEMORY-114 write-time importance scoring (opt-in)
    sp.add_argument(
        "--score-importance", action="store_true",
        dest="score_importance",
        help="invoke configured LLM to rate importance ∈ [0, 1] (TASK-MEMORY-114; opt-in)",
    )
    sp.add_argument(
        "--importance", type=float, default=None,
        help="explicit operator-pinned importance ∈ [0, 1]; wins over --score-importance",
    )
    sp.add_argument(
        "--invoker", default=None, choices=["mock", "anthropic"],
        help="(with --score-importance) invoker selection: mock | anthropic. "
             "Defaults to env CYBEROS_IMPORTANCE_INVOKER or anthropic-if-API-key-else-mock",
    )
    sp.add_argument(
        "--dry-run", action="store_true",
        dest="dry_run",
        help="(with --score-importance) print the would-be importance + extras; no write",
    )
    sp.set_defaults(fn=_cmd_put)

    sp = sub.add_parser("move", help="canonical op: rename within the store")
    sp.add_argument("src")
    sp.add_argument("dst")
    sp.set_defaults(fn=_cmd_move)

    sp = sub.add_parser("delete", help="tombstone or purge a memory file")
    sp.add_argument("path")
    sp.add_argument("--mode", choices=["tombstone", "purge"], default="tombstone",
                    help="tombstone (default; reversible) or purge (GDPR Art. 17)")
    sp.add_argument("--reason", default=None,
                    help="required for --mode=purge; free-form text")
    sp.add_argument("--approval-phrase", default=None,
                    help="purge gate; must equal the AGENTS.md §16.2 magic phrase")
    sp.set_defaults(fn=_cmd_delete)

    sp = sub.add_parser("verify", help="verify the Merkle LINK chain")
    sp.set_defaults(fn=_cmd_verify)

    sp = sub.add_parser("export", help="produce a deterministic zip export")
    sp.add_argument("out")
    sp.set_defaults(fn=_cmd_export)

    sp = sub.add_parser("audit", help="audit-ledger inspection")
    sp.add_argument("action", choices=["dump", "head"])
    sp.add_argument("--month", default=None)
    sp.set_defaults(fn=_cmd_audit)

    sp = sub.add_parser("search", help="FTS5 or semantic search over memory bodies")
    sp.add_argument("query")
    sp.add_argument("--limit", type=int, default=50)
    sp.add_argument("--semantic", action="store_true",
                    help="use local embeddings + cosine similarity "
                         "(requires `pip install sentence-transformers`)")
    sp.set_defaults(fn=_cmd_search)

    sp = sub.add_parser("semantic-sync",
                        help="(re-)embed memory bodies for `cyberos search --semantic`")
    sp.add_argument("--batch-size", type=int, default=32,
                    help="embedder batch size (default 32)")
    sp.set_defaults(fn=_cmd_semantic_sync)

    sp = sub.add_parser("checkpoint", help="force a power-loss-safe checkpoint flush")
    sp.set_defaults(fn=_cmd_checkpoint)

    sp = sub.add_parser("import",
                        help="import memories from another memory (PROPOSAL.md P6)")
    sp.add_argument("source",
                    help="path to another .cyberos/memory/store/ or a cyberos export zip")
    sp.add_argument("--filter", action="append", default=None,
                    help="key=value predicate (kind=, sync_class=, actor=, classification=); repeatable")
    sp.add_argument("--on-conflict", choices=["skip", "overwrite", "branch"],
                    default="skip",
                    help="how to handle path collisions (default: skip)")
    sp.add_argument("--map-actor", action="append", default=None,
                    help="FROM:TO actor rename; repeatable")
    sp.add_argument("--since", type=int, default=None,
                    help="override the auto-tracked last_imported_seq")
    sp.add_argument("--dry-run", action="store_true",
                    help="report what would import; no writes")
    sp.set_defaults(fn=_cmd_import)

    sp = sub.add_parser("backup",
                        help="incremental hard-link snapshot of the store")
    sp.add_argument("--target", required=True,
                    help="directory under which snapshots are organised")
    sp.add_argument("--label", default=None,
                    help="optional human label for this snapshot")
    sp.add_argument("--list", action="store_true",
                    help="list existing snapshots and exit")
    sp.add_argument("--verify", action="store_true",
                    help="re-verify a snapshot's root hash (needs --snapshot)")
    sp.add_argument("--snapshot", default=None,
                    help="(with --verify) snapshot name to verify")
    sp.set_defaults(fn=_cmd_backup)

    sp = sub.add_parser("prune",
                        help="sweep archived binlog originals after soak window")
    sp.add_argument("--soak-days", type=int, default=30,
                    help="only prune segments older than this (default 30)")
    sp.add_argument("--dry-run", action="store_true",
                    help="report what would be pruned without deleting")
    sp.add_argument("--restore", action="store_true",
                    help="inverse: decompress .zst archives back to .binlog")
    sp.add_argument("--segments", nargs="*", default=None,
                    help="(with --restore) specific segments to restore")
    sp.set_defaults(fn=_cmd_prune)

    sp = sub.add_parser("prove",
                        help="produce an MMR inclusion proof for an audit row")
    sp.add_argument("seq", type=int, help="audit row sequence number")
    sp.add_argument("--out", default="-",
                    help="output path (default stdout)")
    sp.set_defaults(fn=_cmd_prove)

    sp = sub.add_parser("verify-proof",
                        help="re-verify a proof emitted by `cyberos prove`")
    sp.add_argument("proof", help="path to proof JSON")
    sp.set_defaults(fn=_cmd_verify_proof)

    sp = sub.add_parser("sth-wrap",
                        help="passphrase-wrap the STH signing key (PROPOSAL.md P2 Stage 2)")
    sp.add_argument("--passphrase", default=None,
                    help="passphrase (NOT RECOMMENDED — visible in shell history)")
    sp.add_argument("--passphrase-file", default=None,
                    help="read passphrase from file (preferred for scripts)")
    sp.set_defaults(fn=_cmd_sth_wrap)

    sp = sub.add_parser("state",
                        help="print agent state (READY / FROZEN_RECOVERABLE / FROZEN_HUMAN)")
    sp.add_argument("--json", action="store_true",
                    help="emit JSON instead of human-readable")
    sp.set_defaults(fn=_cmd_state)

    sp = sub.add_parser(
        "consolidate",
        help="run the 4-phase Walk → Compact → Sign → Publish consolidation "
             "(optionally followed by SemanticDedup per TASK-MEMORY-116)",
    )
    sp.add_argument("--dry-run", action="store_true",
                    help="Walk only; do not compact/sign/publish")
    sp.add_argument("--json", action="store_true",
                    help="emit JSON instead of human-readable")
    sp.add_argument("--compact-horizon-days", type=int, default=90,
                    help="archive sealed segments older than this (default 90)")
    # TASK-MEMORY-116 SemanticDedup phase
    sp.add_argument(
        "--semantic-dedup", action="store_true", dest="semantic_dedup",
        help="add the SemanticDedup phase after Publish (TASK-MEMORY-116; opt-in)",
    )
    sp.add_argument(
        "--semantic-dedup-apply", action="store_true",
        dest="semantic_dedup_apply",
        help="(with --semantic-dedup) actually merge proposals; default is dry-run",
    )
    sp.add_argument(
        "--semantic-dedup-threshold", type=float, default=0.92,
        dest="semantic_dedup_threshold",
        help="duplicates similarity cutoff for SemanticDedup (default 0.92)",
    )
    sp.add_argument(
        "--semantic-dedup-scope", default="", dest="semantic_dedup_scope",
        help="limit SemanticDedup to a subtree (e.g. memories/facts)",
    )
    sp.set_defaults(fn=_cmd_consolidate)

    sp = sub.add_parser("doctor", help="run the self-audit walker over the store")
    sp.add_argument("--json", action="store_true", help="emit JSON instead of human-readable")
    sp.add_argument("--only", default=None,
                    help="comma-separated list of invariant ids to run (default: all)")
    sp.add_argument("--repair", action="store_true",
                    help="attempt safe auto-repair for recoverable failures "
                         "(NEVER for chain corruption / unparseable manifest)")
    sp.set_defaults(fn=_cmd_doctor)

    sp = sub.add_parser("crypto-mode",
                        help="inspect/migrate crypto_mode (P2 Stage 3 chained ↔ sth_only)")
    sp.add_argument("action", choices=["show", "upgrade", "downgrade"])
    sp.add_argument("--approval-phrase", default=None,
                    help="required for upgrade/downgrade — see crypto_mode.APPROVAL_PHRASE")
    sp.add_argument("--skip-safety-checks", action="store_true",
                    help="(for upgrade) bypass STH-presence and MMR-cross-check gates")
    sp.add_argument("--json", action="store_true",
                    help="emit JSON output")
    sp.set_defaults(fn=_cmd_crypto_mode)

    sp = sub.add_parser("session",
                        help="manage multi-agent coordination sessions (PROPOSAL.md P11)")
    sp.add_argument("action", choices=["start", "end", "list"])
    sp.add_argument("--id", default=None,
                    help="(for end) session id to close")
    sp.add_argument("--scope", default=None,
                    help="(for start) comma-separated POSIX path prefixes")
    sp.add_argument("--ttl-hours", type=int, default=4,
                    help="(for start) lease TTL in hours (default 4)")
    sp.add_argument("--note", default=None,
                    help="(for start) human-readable description")
    sp.add_argument("--session-actor", default=None,
                    help="(for start) override the actor recorded for this session")
    sp.add_argument("--force", action="store_true",
                    help="(for start) ignore scope-overlap warnings")
    sp.add_argument("--json", action="store_true",
                    help="emit JSON instead of human-readable output")
    sp.add_argument("--exit-code", action="store_true",
                    help="(for list) exit 1 if no active sessions")
    sp.set_defaults(fn=_cmd_session)

    sp = sub.add_parser("serve",
                        help="run the local read-only HTTP REST API (PROPOSAL.md P10)")
    sp.add_argument("--host", default="127.0.0.1",
                    help="bind address (default 127.0.0.1; only loopback recommended)")
    sp.add_argument("--port", type=int, default=8765,
                    help="bind port (default 8765)")
    sp.add_argument("--print-token", action="store_true",
                    help="print the current bearer token and exit")
    sp.add_argument("--reset-token", action="store_true",
                    help="rotate the bearer token and exit")
    sp.set_defaults(fn=_cmd_serve)

    sp = sub.add_parser("publish",
                        help="produce a single-file mobile-friendly static site (PROPOSAL.md P12)")
    sp.add_argument("--out", required=True,
                    help="output path for the .html file")
    sp.add_argument("--kinds", default=None,
                    help="comma-separated allowlist of memory kinds")
    sp.add_argument("--exclude-kinds", default=None,
                    help="comma-separated blocklist of memory kinds")
    sp.add_argument("--max-body-chars", type=int, default=200_000,
                    help="cap per-body size (default 200000)")
    sp.add_argument("--deterministic", action="store_true",
                    help="zero out generated_at_ns so output is byte-stable across runs")
    sp.add_argument("--json", action="store_true",
                    help="emit summary as JSON")
    sp.set_defaults(fn=_cmd_publish)

    sp = sub.add_parser("digest",
                        help="daily summary of audit activity (PROPOSAL.md P8)")
    sp.add_argument("--window", default="24h",
                    help="lookback window: e.g. 24h, 7d, 2w (default 24h)")
    sp.add_argument("--since", default=None,
                    help="explicit start: human duration like '6h' OR epoch-ns int")
    sp.add_argument("--until", default=None, type=int,
                    help="explicit end (epoch-ns); default: now")
    sp.add_argument("--format", choices=["text", "markdown", "json"], default="text",
                    help="output format (default text)")
    sp.add_argument("--highlight-cap", type=int, default=50,
                    help="max highlights to surface (default 50)")
    sp.add_argument("--via-claude", action="store_true",
                    help="also pipe the JSON digest to local Claude CLI for prose summary")
    sp.add_argument("--claude-model", default=None,
                    help="(with --via-claude) override the Claude model")
    sp.set_defaults(fn=_cmd_digest)

    sp = sub.add_parser("resolve-conflict",
                        help="list, diff, or merge sync-FS conflict siblings (PROPOSAL.md P9)")
    sp.add_argument("path", nargs="?", default=None,
                    help="path of the canonical memory; omit to list all conflicts")
    sp.add_argument("--list", action="store_true",
                    help="list all conflict pairs and exit")
    sp.add_argument("--diff", action="store_true",
                    help="show unified diffs vs. each sibling (default if no --keep given)")
    sp.add_argument("--keep", default=None,
                    help="canonical (archive siblings) or sibling:<index> (replace canonical)")
    sp.add_argument("--dry-run", action="store_true",
                    help="report actions without moving any files")
    sp.set_defaults(fn=_cmd_resolve_conflict)

    sp = sub.add_parser("validate", help="validate memory files against the frontmatter schema")
    sp.add_argument("paths", nargs="+",
                    help="memory file paths relative to the store root")
    sp.set_defaults(fn=_cmd_validate)

    # --- TASK-MEMORY-120 cyberos history ---
    sp = sub.add_parser(
        "history",
        help="per-path version + attribution from the audit chain "
             "(TASK-MEMORY-120; read-only)",
    )
    sp.add_argument("path", help="memory path under <memory-root>/")
    sp.add_argument("--limit", type=int, default=10,
                    help="cap entries (most-recent first; default 10)")
    sp.add_argument("--chronological", action="store_true",
                    help="oldest-first instead of most-recent-first")
    sp.add_argument("--no-follow-moves", action="store_true",
                    dest="no_follow_moves",
                    help="stop the walk at move boundaries")
    sp.add_argument("--show-body", action="store_true",
                    dest="show_body",
                    help="include body diffs when available")
    sp.add_argument("--since", default=None,
                    help="24h | 7d | ISO timestamp")
    sp.add_argument("--json", action="store_true",
                    help="emit JSON instead of human view")
    sp.set_defaults(fn=_cmd_history)

    # --- TASK-MEMORY-119 session transcript ledger ---
    sp = sub.add_parser(
        "transcript",
        help="session transcript ledger (TASK-MEMORY-119) — "
             "start / append / end / read / list / purge-expired",
    )
    sp.add_argument(
        "transcript_action",
        choices=["start", "append", "end", "read", "list", "purge-expired"],
    )
    sp.add_argument("--id", default=None,
                    help="session id (required for start/append/end/read)")
    sp.add_argument("--classification", default="confidential",
                    choices=["confidential", "restricted"],
                    help="(start) default classification per Stephen 2026-05-19")
    sp.add_argument("--retention-days", type=int, default=30,
                    dest="retention_days",
                    help="(start/purge-expired) retention horizon")
    sp.add_argument("--role", choices=["user", "assistant", "system", "tool"],
                    default=None, help="(append) turn role")
    sp.add_argument("--content", default=None,
                    help="(append) turn content")
    sp.add_argument("--redactions-applied", type=lambda s: s.lower() == "true",
                    default=None, dest="redactions_applied",
                    help="(append) record that TASK-MEMORY-111 PII redaction ran")
    sp.add_argument("--reason", default=None,
                    help="(end) free-form reason")
    sp.add_argument("--no-seal", action="store_true",
                    help="(end) skip zstd compression of binlog")
    sp.add_argument("--decrypt", action="store_true",
                    help="(read) decrypt content_cipher payloads")
    sp.add_argument("--since", default=None,
                    help="(list) restrict to sessions in the last N hours/days")
    sp.add_argument("--dry-run", action="store_true",
                    help="(purge-expired) report what would be purged")
    sp.add_argument("--json", action="store_true",
                    help="emit JSON output")
    sp.set_defaults(fn=_cmd_transcript)

    # --- TASK-MEMORY-118 put_if (optimistic-concurrency) ---
    sp = sub.add_parser(
        "put-if",
        help="content-conditional put: writes only when current body matches "
             "the supplied SHA-256 (TASK-MEMORY-118)",
    )
    sp.add_argument("path", help="target memory path")
    sp.add_argument("body_file", help="path to file containing new body bytes")
    sp.add_argument(
        "--precondition", required=True,
        help='64-char lowercase hex SHA-256 OR "none" for create-only',
    )
    sp.add_argument(
        "--precondition-from-file", default=None,
        dest="precondition_from_file",
        help="alternative: read hex from this file's first line",
    )
    sp.add_argument("--kind", default=None,
                    help="memory kind tag for the audit row (default 'unknown')")
    sp.add_argument("--json", action="store_true",
                    help="emit JSON result")
    sp.set_defaults(fn=_cmd_put_if)

    # --- TASK-MEMORY-117 per-store ACL ---
    sp = sub.add_parser(
        "acl",
        help="manage per-store ACL (TASK-MEMORY-117) — show / validate / explain",
    )
    sp.add_argument("acl_action", choices=["show", "validate", "explain"],
                    help="show: list STORE.yaml content; validate: check shapes; "
                         "explain: resolve effective mode for a path+actor")
    sp.add_argument("path", nargs="?", default=None,
                    help="(for explain) memory path to resolve")
    sp.add_argument("--json", action="store_true",
                    help="(for explain) JSON output")
    sp.set_defaults(fn=_cmd_acl)

    # --- TASK-MEMORY-115 dreaming ---
    sp = sub.add_parser(
        "dream",
        help="run one out-of-band batch reflection pass (TASK-MEMORY-115)",
    )
    sp.add_argument("--since", default="24h",
                    help="time window: 24h | 7d | 30d | ISO timestamp (default 24h)")
    sp.add_argument("--scope", default="",
                    help="limit detectors to memories under this path prefix")
    sp.add_argument("--detectors", default=None,
                    help="comma-separated subset of duplicates,stale,patterns,verify "
                         "(default: all four)")
    sp.add_argument("--invoker", default=None, choices=["mock", "anthropic"],
                    help="LLM invoker selection for detectors that use one")
    sp.add_argument("--threshold", type=float, default=0.92,
                    help="duplicates detector similarity cutoff (default 0.92)")
    sp.add_argument("--dry-run", action="store_true", dest="dry_run",
                    help="produce the diff but tag dream.complete with dry_run=True")
    sp.add_argument("--json", action="store_true",
                    help="emit JSON diff to stdout")
    sp.set_defaults(fn=_cmd_dream)

    sp = sub.add_parser(
        "dream-apply",
        help="apply selected proposals from a prior `cyberos dream` run",
    )
    sp.add_argument("dream_id", help="ULID from a prior dream.start row")
    sp.add_argument("--proposal-ids", default=None, dest="proposal_ids",
                    help="comma-separated subset of proposal_ids; absent ⇒ apply all")
    sp.add_argument("--actor", default=None,
                    help="override the actor recorded on apply rows "
                         "(default: dream-applier)")
    sp.add_argument("--no-check-protocol", action="store_true",
                    dest="no_check_protocol",
                    help="skip AGENTS.md §7.7 anchor check (DEV ONLY)")
    sp.add_argument("--json", action="store_true",
                    help="emit JSON summary")
    sp.set_defaults(fn=_cmd_dream_apply)

    # --- TASK-MEMORY-112 episodic memory ---
    sp = sub.add_parser(
        "episode",
        help="episodic memory (TASK-MEMORY-112) — `cyberos episode log ...`",
    )
    ep_sub = sp.add_subparsers(dest="episode_cmd", required=True)
    ep_log = ep_sub.add_parser("log", help="append an Episode to the memory")
    ep_log.add_argument("--task", required=True,
                        help="what task the agent did (free-form, ≥ 1 char)")
    ep_log.add_argument("--approach", required=True,
                        help="how the agent approached the task (one line)")
    ep_log.add_argument("--outcome", required=True,
                        choices=["success", "partial", "failure"],
                        help="closed enum outcome")
    ep_log.add_argument("--duration-ms", type=int, required=True,
                        dest="duration_ms", help="wall-clock duration in ms (≥ 0)")
    ep_log.add_argument("--token-cost", type=int, default=None,
                        dest="token_cost", help="input+output tokens combined (optional)")
    ep_log.add_argument("--quality-score", type=float, default=None,
                        dest="quality_score",
                        help="0.0–1.0; absent ≡ 0.5 in ranking maths (DEC-181)")
    ep_log.add_argument("--notes", default="",
                        help="free-form observations (included in searchable doc)")
    ep_log.add_argument("--error", default=None,
                        help="required when outcome is partial or failure")
    ep_log.add_argument("--json", action="store_true",
                        help="emit JSON output instead of seq=N path=...")
    ep_log.set_defaults(fn=_cmd_episode_log)

    sp = sub.add_parser(
        "recall-similar",
        help="find episodes similar to a task (TASK-MEMORY-112 §1 #9)",
    )
    sp.add_argument("task", help="task description to find similar episodes for")
    sp.add_argument("--k", type=int, default=3,
                    help="top-K results (default 3)")
    sp.add_argument("--min-relevance", type=float, default=0.65,
                    dest="min_relevance",
                    help="reject hits below this relevance (default 0.65)")
    sp.add_argument("--semantic", action="store_true",
                    help="force semantic backend (requires sentence-transformers)")
    sp.add_argument("--fts5", action="store_true",
                    help="force FTS5 backend (skip semantic even if available)")
    sp.add_argument("--json", action="store_true",
                    help="emit JSON output instead of human table")
    sp.set_defaults(fn=_cmd_recall_similar)

    # --- self-update ---
    sp = sub.add_parser(
        "self-update",
        help="check for newer version in source repo, update package + store",
    )
    sp.add_argument("--force", action="store_true",
                    help="re-copy AGENTS.md even if version matches")
    sp.set_defaults(fn=_cmd_self_update)

    # --- skill (delegates to cyberos-skill binary or built-in) ---
    sp = sub.add_parser(
        "skill",
        help="skill operations (list, info, run, validate)",
    )
    sp.add_argument("skill_args", nargs=argparse.REMAINDER,
                    help="skill sub-command + args (list, info, run, validate)")
    sp.set_defaults(fn=_cmd_skill)

    # --- workflow (delegates to cyberos-cuo) ---
    sp = sub.add_parser(
        "workflow",
        help="run persona workflows (requires cyberos-cuo package)",
    )
    sp.add_argument("workflow_args", nargs=argparse.REMAINDER,
                    help="persona-workflow path + flags, or sub-command (list-personas, list-workflows, dry-run)")
    sp.set_defaults(fn=_cmd_workflow)

    return p


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    # Version freshness hint on every non-self-update command
    if hasattr(args, "fn") and args.fn is not _cmd_self_update:
        try:
            _maybe_check_version(_store(args))
        except Exception:
            pass
    return int(args.fn(args) or 0)


if __name__ == "__main__":
    sys.exit(main())
