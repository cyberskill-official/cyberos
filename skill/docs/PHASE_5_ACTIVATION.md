# Phase 5 — Activating the WASM execution path

The Rust host's WASM execution path is feature-gated. The entire WASM machinery (wasmtime engine, WASI capability translator, AOT cache, jco componentize pipeline) compiles, tests, and is ready — it just doesn't get built unless you opt in.

This doc walks the one-shot activation on a Mac (or Linux). After this, `cyberos-skill run --executor wasm` works end-to-end against any componentized skill.

## Prerequisites

```bash
# 1. Add the Rust wasm32-wasi target
rustup target add wasm32-wasi

# 2. Install wasmtime CLI (for local component smoke-testing)
curl https://wasmtime.dev/install.sh -sSf | bash
# Then re-source your shell rc, or restart the terminal:
source ~/.zshrc   # zsh
source ~/.bashrc  # bash

# 3. Install jco for TypeScript componentization
cd skill/toolchain
bun add @bytecodealliance/jco @bytecodealliance/preview2-shim
```

## Build the host with `wasm` feature

```bash
cd skill
cargo build --features wasm
```

First run downloads wasmtime 27 plus ~50 transitive deps (~30s on a modern Mac). Subsequent builds are cached.

## Componentize a TypeScript skill

```bash
cd skill/toolchain
bun run build.ts templates/ts-skill
```

If `jco` is installed correctly, this produces a real `dist/skill.wasm` Component-Model artifact. Otherwise it falls back to the 8-byte stub with a clear install hint.

## Verify end-to-end

```bash
cd skill
echo '{"name":"world"}' | cargo run --features wasm -p cyberos-skill-cli -- \
    run ts-hello --executor wasm
```

Expected: a JSON greeting from the TS hello-world skill, executed inside the Wasmtime sandbox with the declared capabilities.

## Soak phase

Once Phase 5 builds clean and a few skills run end-to-end through the WASM path, begin the 30-day soak:

1. Run the parity harness against both executors:

   ```bash
   python skill/tests/parity/run_parity.py
   ```

2. Add a `--executor wasm` variant to the parity harness once componentized skills exist.

3. Monitor `~/.cyberos/cache/wasm/` size + AOT cache hit rate.

4. Track any capability mis-grants via the audit trail (`cyberos-skill cap audit`).

5. After 30 days of zero P0 incidents, follow `docs/PHASE_7_RETIREMENT.md` to retire the Python script tier.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `rustup target add wasm32-wasi` fails | rustup not installed | https://rustup.rs |
| `wasmtime: command not found` after install | shell rc not re-sourced | `source ~/.zshrc` or restart terminal |
| `bunx jco componentize` fails | preview2-shim version mismatch | `bun update @bytecodealliance/preview2-shim` |
| `cargo build --features wasm` very slow | first-time wasmtime fetch | normal; ~30s on Apple Silicon |
| `cargo build --features wasm` fails on Linux | system libssl mismatch | install libssl-dev or use rustls feature |
| `wasm executor not compiled in` | host built without feature | rebuild with `--features wasm` |

## What's NOT in Phase 5

- WASI 0.2 sockets domain allowlist for `fetch_url(<pattern>)` — pending in wasmtime upstream. Until then, `fetch_url` capability is logged but not effective; the skill sees no network. Track wasmtime-rs#7894.
- Real cosign signature verification on `.skill.tar.gz` bundles — that's Phase 6 work, wired in `crates/resolver/`.
- Python guest-language components — componentize-py works but is preview-only. Stick to TypeScript via `jco componentize` for production.

## Why this is feature-gated

The audit's risk register flags MMR-class silent failures as Medium-likelihood / High-impact for any new crypto-adjacent primitive. WASM execution is in the same category: a bug in the WASI capability translation could silently widen a skill's authority. Gating it behind a feature flag means:

1. The workspace builds clean on machines that don't have the Rust wasm32-wasi target installed (most contributors).
2. The capability broker (Phase 6) gets to soak in isolation before WASM execution depends on it in production.
3. Rollback is a one-flag flip (`cargo build` without `--features wasm`), not a code revert.
