# Known issues

Pre-go-live issues found during local live testing. Each entry has the symptom, the root cause with
evidence, the impact, and the fix options.

## 1. AI gateway memory-audit bridge is broken (FR-AI-003 contract drift) - blocker

Symptom. The cost-hold expiry tick never transitions expired holds. The integration tests
`ai-gateway::cost_hold_expiry::tick_skips_non_expired_holds` and `tick_skips_reconciled_holds` fail:
expired holds stay in state `held` after a tick, with no error surfaced.

Root cause. `services/ai-gateway/src/memory_writer.rs` spawns the memory Writer as
`python3 -m cyberos.writer put`, pipes a canonical-JSON payload on stdin, and expects a `{seq, chain,
ts_ns}` triple on stdout. That is exactly what FR-AI-003 specifies. But the memory package never shipped
a `cyberos.writer` module. Its real surface is the `cyberos` package CLI (`modules/memory/cyberos/__main__.py`),
invoked as `cyberos put <path> <body_file>` - a positional path plus a body file, with no stdin-JSON
protocol and no `{seq, chain, ts_ns}` stdout contract. The writer code lives at
`modules/memory/cyberos/core/writer.py`, which has no runnable `-m` entry point. So
`python3 -m cyberos.writer` fails with `No module named cyberos.writer` in every environment.

Evidence.
- `python3 -m cyberos.writer put` returns `No module named cyberos.writer` even with
  `PYTHONPATH=modules/memory`.
- Tracing inside `run_tick` shows the candidate query returns the correct expired-hold IDs and
  `process_one_hold` reaches the transition step; the per-hold transaction then rolls back because
  `memory_writer::emit` returns an error (the subprocess fails), and the rollback is logged via
  `tracing::warn!` which is silent with no subscriber in the test.
- FR-AI-003 §1 #2 and the build envelope literally specify `WRITER_ARGS = ["-m", "cyberos.writer", "put"]`
  with a stdin payload; the memory CLI in `cyberos/__main__.py` reads `Path(args.body_file).read_bytes()`.

Why it was not caught. `tick_processes_expired_holds` is written defensively
(`assert!(report.holds_processed == 3 || report.holds_failed > 0)` and only checks DB state when
`holds_succeeded == 3`), so it passes whether or not the Writer works. The `memory_writer` unit tests
cover the pure helpers (path validation, canonicalisation), not a live subprocess round-trip.

Impact. The design fails safe - audit-before-action means no expiry is written without its audit row, so
there is no data corruption and no unsafe state. But the feature is non-functional: cost holds never
expire, so tenant `spent_usd` stays inflated and tenants drift toward their cap and eventually get
blocked. The same broken bridge means any gateway-to-memory audit write via `memory_writer::emit` fails,
so FR-AI-003's audit trail is not actually being written.

Fix options (a design decision, not a one-liner).
1. Add the FR-AI-003 contract to the memory package: a `cyberos.writer` module (or a
   `cyberos write-audit` subcommand) that reads the `{path, body, meta}` JSON on stdin and prints
   `{seq, chain, ts_ns}`. Smallest change to satisfy the existing Rust bridge unchanged.
2. Rewrite `memory_writer.rs` to drive the actual CLI: write the body to a temp file, call
   `python3 -m cyberos put <path> <body_file>`, and parse that CLI's real output. Couples the gateway to
   the current memory CLI shape.
3. Replace the subprocess bridge with an HTTP/IPC call to the memory service (it already runs as an HTTP
   server), removing the Python-subprocess dependency from the gateway hot path entirely.

Either way, add one integration test that actually round-trips through the real Writer so the contract
cannot drift again, and (per FR-AI-003 §1 #10) wire the documented startup health check
(`python3 -m cyberos.writer --version`) so a broken bridge fails at deploy time, not silently at runtime.

## 2. AUTH p95 latency assertion is environment-sensitive - minor

`auth::create_subject_p95_latency_under_200ms` asserts a p95 under 200 ms. Docker Desktop on macOS is
slower than CI and trips it locally. Not a logic failure. Relax or gate the threshold for local runs, or
skip it (`--skip create_subject_p95`).

## 3. Production VPS compose and Caddyfile are not in the repo - blocker for reproducible deploy

`deploy/vps/` holds only `.env.local` and `data/`. The `docker-compose.yml` and `Caddyfile` that consume
`.env.local` live only on the VPS, so the deploy is not reproducible from the repo. Commit them (with
secrets kept in the untracked `.env.local`). See `docs/deploy/cyberos-core-deploy.md` gaps.
