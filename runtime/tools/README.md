# `runtime/tools/` — Local-optimization CLIs for `.cyberos-memory/`

Single-file Python tools that operate against any `.cyberos-memory/` store. No CyberOS infrastructure required — they work today on your local BRAIN.

## What's here

| Tool | Purpose | Stage |
|------|---------|-------|
| `cyberos_validate.py` | BRAIN health check — 11 check categories now (chain, schema, supersedes graph, caps, stale-checkpoint, encryption envelope, Shamir consistency) | Stage 2 + 5 ✅ |
| `cyberos_doctor.py` | Recovery CLI for CORRUPT-state diagnosis + structured repairs under MAINTENANCE mode (§8.8) | Stage 2 ✅ |
| `cyberos_index.py` | SQLite-backed local search index (tags, relationships, source-SHA, audit-by-path, tombstones) | Stage 3 ✅ |
| `cyberos_export.py` | Deterministic export bundles + daemon mode for periodic backup | Stage 4 ✅ |
| `cyberos_encrypt.py` | At-rest encryption + Shamir 3-of-5 escrow per §5.6 (enable wizard, status, recover, disable, migrate-batch, rotate-shamir) | Stage 5 ✅ |
| `canonical_sha.py` | Compute the §0.5 canonical SHA of an AGENTS.md (for protocol upgrade approval) | Stage 1 ✅ |
| `extract_agents_core.py` | Generate `AGENTS-CORE.md` (10K-token normative subset with auto-guide directives); `--check` for CI; `--aggressive` for compact output | Stage 1 / Bundle M ✅ |
| `benchmark.py` | Measure validator + export performance against any store | Cross-stage |
| `tests/generate_vectors.py` | Regenerate the test-vector corpus | Stage 2 ✅ |
| `tests/vectors/` | 16 fixtures covering CRITICAL findings the validator should catch | Stage 2 ✅ |

## Quick start

```bash
# Install the one Python dependency
pip3 install pyyaml --break-system-packages   # or via your venv

# Run the validator against your project
python3 runtime/tools/cyberos_validate.py .

# Run a deterministic export
python3 runtime/tools/cyberos_export.py . -o ~/Backups/cyberos

# Benchmark current performance
python3 runtime/tools/benchmark.py . --runs 5

# Verify the AGENTS.md SHA matches the manifest pin
python3 runtime/tools/canonical_sha.py docs/CyberOS-AGENTS.md
python3 -c "import json; print(json.load(open('.cyberos-memory/manifest.json'))['protocol']['sha256'])"
# These two outputs MUST match — if not, §13.0 trips INCOMPATIBLE:protocol-sha256-mismatch.
```

## Self-test

```bash
python3 runtime/tools/cyberos_validate.py --self-test
```

Expected: 15 fixtures pass.

## What's NOT here yet (post-Stage-5/6 landed)

All six stages of `docs/CyberOS-AGENTS.LOCAL-OPTIMIZATION.md` are landed at the protocol level. Remaining work:

- **Stage 5/6 implementation extensions to existing tools** — non-§0.5; just code:
  - `cyberos_validate.py` — `_check_merkle_checkpoints()` for Stage 6 (recompute Merkle roots on every `op:"consolidation_run"` row)
  - `cyberos_doctor.py` — `R5-rebuild-merkle-checkpoint` + `R6-rotate-master-key` repairs; `decompact-ledger --month <YYYY-MM>` CLI for §7.7 reverse path
  - `cyberos_index.py` — `merkle_checkpoints` table + `query merkle-proof <chain>` subcommand
  - `cyberos_encrypt.py` v1 — `disable`, `migrate-batch`, `rotate-shamir` are stubs in v0; need full implementations + audit-ledger integration
- **HW-key backends for `cyberos_encrypt.py`** — v0 ships passphrase-only; production wants Apple Secure Enclave (macOS), Windows TPM 2.0 (Hello), Linux TPM 2.0 / FIDO2 hmac-secret
- **RFC 8785 JCS chain-hash recomputation** — currently only LINK invariant verified (INFO-severity finding per §7.2 cross-writer-version compatibility)
- **Cookbooks** — `docs/cookbook/encryption-and-recovery.md`, `docs/cookbook/ledger-compaction.md`
- **Local-only embedding index** — DEFERRED until Layer 2 ships with bge-m3 + reranker

Beyond local optimization (when CyberOS-the-product builds): the long-term vision is in `docs/CyberOS-AGENTS.EVOLUTION.md` — BRAIN module P1, Layer 2 vector+graph, MCP Gateway, multi-tenancy, GraphRAG, IETF standards work. None of that is unblocked yet because the rest of CyberOS isn't built.

## How this fits the broader plan

`docs/CyberOS-AGENTS.LOCAL-OPTIMIZATION.md` is the strategic plan; these tools are the implementation. Specifically:

- **Stage 1** (session-start speed) ships as a §0.5 protocol upgrade — the proposal text is in `docs/proposals/STAGE-1-PROTOCOL-UPGRADE.md`. The agent landing it uses `canonical_sha.py` to compute the post-edit SHA for the approval phrase.
- **Stage 2** (validator) ships as `cyberos_validate.py` + the test-vector corpus, BOTH ALREADY IN PLACE. After Stage 1 lands, the validator gains a `stale-checkpoint` check.
- **Stage 4** (backup) ships as `cyberos_export.py`, ALREADY IN PLACE. Daemon mode covers the "Mac dies → BRAIN dies" gap. The git-as-backup pattern is documented in `docs/cookbook/filesystem-sync.md`.
- **Stages 3, 5, 6** are not yet shipped. Their proposals will follow once Stage 1 lands.

## Performance baseline (taken 2026-05-09)

Measured against the live `cyberos/` store (290 audit rows, 102 memory files, 1.3 MB total):

```
cyberos-validate p95: 180ms       ← target Stage 2: <500ms ✅
cyberos-export   p95: 88ms        ← bundle 486KB, deterministic
```

Re-run `python3 runtime/tools/benchmark.py . --runs 5` to track regression as the store grows.

## Contributing

These tools are read-only against `.cyberos-memory/` (validate, benchmark) or strictly additive (export — writes to a separate destination). They never mutate the store. Any future tool that does mutate the store MUST go through:

1. The §4.4 two-phase atomic write
2. An `op:"…"` audit row appended to the chain
3. A SHA-pinned protocol upgrade if it touches §4 / §5.1 / §6 / §7 semantics

Don't shortcut these. The chain LINK invariant (§7.2) is what makes any of this trustworthy.

## License

Same as `CyberOS-AGENTS.md` — internal CyberSkill IP, all rights reserved. May be open-sourced when the broader CyberOS open-core strategy is decided.
