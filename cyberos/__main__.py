"""
cyberos — single entry point, no per-tool Python cold-start tax.

All heavy imports (msgspec, sqlite3, mmap, the writer) are loaded only
inside the subcommand handler that needs them. Cold ``cyberos --help`` is
argparse + stdlib only — target <30ms on a 2024 MacBook M2 vs ~110ms for
the legacy shell-out-to-tool.py pattern.

PEP 690 documents 50–70% startup reductions from this lazy-import pattern
(peps.python.org/pep-0690). Hugo van Kemenade's pypistats benchmark shows
104ms → 46ms → 35ms on a comparable seven-direct-dep CLI.

The 33 umbrella subcommands of the legacy writer collapse here into 12::

    cyberos view <path>
    cyberos create <path> <body_file>
    cyberos str-replace <path> <old_file> <new_file>
    cyberos insert <path> <line> <text_file>
    cyberos delete <path>
    cyberos rename <src> <dst>
    cyberos audit dump [--month YYYY-MM]
    cyberos audit head
    cyberos verify [--bit-perfect]
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
    return Path(args.store).resolve()


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


def _cmd_create(args: argparse.Namespace) -> int:
    from cyberos.core.ops import create
    from cyberos.core.writer import Writer

    body = Path(args.body_file).read_bytes()
    with Writer(_store(args)) as writer:
        seq = create(
            writer,
            args.path,
            body,
            actor=_actor(args),
            kind=args.kind or "unknown",
        )
    print(f"seq={seq}")
    return 0


def _cmd_put(args: argparse.Namespace) -> int:
    """Canonical v2 op — create or replace a memory file."""
    from cyberos.core.ops import put
    from cyberos.core.writer import Writer

    body = Path(args.body_file).read_bytes()
    with Writer(_store(args)) as writer:
        seq = put(
            writer,
            args.path,
            body,
            actor=_actor(args),
            kind=args.kind or "unknown",
        )
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


def _cmd_str_replace(args: argparse.Namespace) -> int:
    from cyberos.core.ops import str_replace
    from cyberos.core.writer import Writer

    old = Path(args.old_file).read_bytes()
    new = Path(args.new_file).read_bytes()
    with Writer(_store(args)) as writer:
        seq = str_replace(writer, args.path, old, new, actor=_actor(args))
    print(f"seq={seq}")
    return 0


def _cmd_insert(args: argparse.Namespace) -> int:
    from cyberos.core.ops import insert
    from cyberos.core.writer import Writer

    text = Path(args.text_file).read_bytes()
    with Writer(_store(args)) as writer:
        seq = insert(writer, args.path, args.line, text, actor=_actor(args))
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


def _cmd_rename(args: argparse.Namespace) -> int:
    from cyberos.core.ops import rename
    from cyberos.core.writer import Writer

    with Writer(_store(args)) as writer:
        seq = rename(writer, args.src, args.dst, actor=_actor(args))
    print(f"seq={seq}")
    return 0


def _cmd_verify(args: argparse.Namespace) -> int:
    from cyberos.core.walker import verify_segments  # noqa: WPS433
    from cyberos.core.writer import resolve_initial_chain_from_manifest  # noqa: WPS433

    store = _store(args)
    segments = sorted(
        p for p in (store / "audit").glob("*.binlog") if p.name != "current.binlog"
    )
    current = store / "audit" / "current.binlog"
    if current.exists():
        segments.append(current)
    # Honour the legacy chain bridge: if the manifest carries
    # migration.legacy_last_chain, the binlog's first record's
    # prev_chain MUST equal that value, not GENESIS.
    start_prev = resolve_initial_chain_from_manifest(store)
    n = verify_segments(segments, start_prev=start_prev)
    bridge = "" if start_prev == "0" * 64 else f" (bridge: prev={start_prev[:16]}…)"
    print(f"verified {n} records across {len(segments)} segment(s); chain intact{bridge}")
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
    from cyberos.core.index import open_index, search_memories  # noqa: WPS433

    store = _store(args)
    fingerprint = hashlib.sha256(str(store).encode("utf-8")).hexdigest()[:16]
    conn = open_index(fingerprint)
    for rel_path, snippet in search_memories(conn, args.query, limit=args.limit):
        print(f"{rel_path}\t{snippet}")
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
    repo's ``docs/memory/memory.schema.json``. If jsonschema isn't
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
    """Pull memories from another BRAIN into this one (PROPOSAL.md P6)."""
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
            "ledger-bridge-continuity",
            "ledger-mmr-cross-check",
            "manifest-schema-version",
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
    """Run Walk → Compact → Sign → Publish (AGENTS.md v2 §7)."""
    from cyberos.core.consolidate import format_report, run  # noqa: WPS433

    report = run(
        _store(args),
        dry_run=args.dry_run,
        compact_horizon_days=args.compact_horizon_days,
    )
    print(format_report(report, json_mode=args.json))
    return 0 if report.ok else 1


def _cmd_doctor(args: argparse.Namespace) -> int:
    """Run the self-audit walker.

    Iterates every invariant in ``docs/memory/memory.invariants.yaml``,
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


# --- argparse wiring ------------------------------------------------------


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="cyberos", description="CyberOS Layer-1 CLI")
    p.add_argument("--store", default=".cyberos-memory", help="store root (default: ./.cyberos-memory)")
    p.add_argument("--actor", default=None, help="principal identifier for audit rows")
    sub = p.add_subparsers(dest="cmd", required=True)

    sp = sub.add_parser("view", help="read a memory file")
    sp.add_argument("path")
    sp.set_defaults(fn=_cmd_view)

    sp = sub.add_parser("create", help="(v1 alias) create a new memory file")
    sp.add_argument("path")
    sp.add_argument("body_file")
    sp.add_argument("--kind", default=None)
    sp.set_defaults(fn=_cmd_create)

    sp = sub.add_parser("put", help="canonical v2 op: create-or-replace a memory file")
    sp.add_argument("path")
    sp.add_argument("body_file")
    sp.add_argument("--kind", default=None)
    sp.set_defaults(fn=_cmd_put)

    sp = sub.add_parser("move", help="canonical v2 op: rename within the store")
    sp.add_argument("src")
    sp.add_argument("dst")
    sp.set_defaults(fn=_cmd_move)

    sp = sub.add_parser("str-replace", help="replace a substring in a memory file")
    sp.add_argument("path")
    sp.add_argument("old_file")
    sp.add_argument("new_file")
    sp.set_defaults(fn=_cmd_str_replace)

    sp = sub.add_parser("insert", help="insert text at a line in a memory file")
    sp.add_argument("path")
    sp.add_argument("line", type=int)
    sp.add_argument("text_file")
    sp.set_defaults(fn=_cmd_insert)

    sp = sub.add_parser("delete", help="tombstone or purge a memory file")
    sp.add_argument("path")
    sp.add_argument("--mode", choices=["tombstone", "purge"], default="tombstone",
                    help="tombstone (default; reversible) or purge (GDPR Art. 17)")
    sp.add_argument("--reason", default=None,
                    help="required for --mode=purge; free-form text")
    sp.add_argument("--approval-phrase", default=None,
                    help="purge gate; must equal the AGENTS.md §16.2 magic phrase")
    sp.set_defaults(fn=_cmd_delete)

    sp = sub.add_parser("rename", help="rename a memory file")
    sp.add_argument("src")
    sp.add_argument("dst")
    sp.set_defaults(fn=_cmd_rename)

    sp = sub.add_parser("verify", help="verify the Merkle LINK chain")
    sp.set_defaults(fn=_cmd_verify)

    sp = sub.add_parser("export", help="produce a deterministic zip export")
    sp.add_argument("out")
    sp.set_defaults(fn=_cmd_export)

    sp = sub.add_parser("audit", help="audit-ledger inspection")
    sp.add_argument("action", choices=["dump", "head"])
    sp.add_argument("--month", default=None)
    sp.set_defaults(fn=_cmd_audit)

    sp = sub.add_parser("search", help="FTS5 search over memory bodies")
    sp.add_argument("query")
    sp.add_argument("--limit", type=int, default=50)
    sp.set_defaults(fn=_cmd_search)

    sp = sub.add_parser("checkpoint", help="force a power-loss-safe checkpoint flush")
    sp.set_defaults(fn=_cmd_checkpoint)

    sp = sub.add_parser("import",
                        help="import memories from another BRAIN (PROPOSAL.md P6)")
    sp.add_argument("source",
                    help="path to another .cyberos-memory/ or a cyberos export zip")
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

    sp = sub.add_parser("consolidate",
                        help="run the 4-phase Walk → Compact → Sign → Publish consolidation")
    sp.add_argument("--dry-run", action="store_true",
                    help="Walk only; do not compact/sign/publish")
    sp.add_argument("--json", action="store_true",
                    help="emit JSON instead of human-readable")
    sp.add_argument("--compact-horizon-days", type=int, default=90,
                    help="archive sealed segments older than this (default 90)")
    sp.set_defaults(fn=_cmd_consolidate)

    sp = sub.add_parser("doctor", help="run the self-audit walker over the store")
    sp.add_argument("--json", action="store_true", help="emit JSON instead of human-readable")
    sp.add_argument("--only", default=None,
                    help="comma-separated list of invariant ids to run (default: all)")
    sp.add_argument("--repair", action="store_true",
                    help="attempt safe auto-repair for recoverable failures "
                         "(NEVER for chain corruption / unparseable manifest)")
    sp.set_defaults(fn=_cmd_doctor)

    sp = sub.add_parser("validate", help="validate memory files against the frontmatter schema")
    sp.add_argument("paths", nargs="+",
                    help="memory file paths relative to the store root")
    sp.set_defaults(fn=_cmd_validate)

    return p


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return int(args.fn(args) or 0)


if __name__ == "__main__":
    sys.exit(main())
