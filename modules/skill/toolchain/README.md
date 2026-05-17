# `skill/toolchain/` — Developer toolchain for CyberOS Skills

Bun 1.3+ powered. Authors write TypeScript or Python skills; this toolchain builds them to `.wasm` components targeting `wasm32-wasi` Preview 2 (the same format every modern WASI host runs).

## Why Bun for the toolchain?

Anthropic adopted Bun 1.3 to power Claude Code in 2026. We follow suit for skill authoring DX:

- ~5 ms cold start
- Native TypeScript without a separate compile step
- Built-in bundler, test runner, package manager
- Stable on macOS / Linux / Windows

The Bun runtime is for *authoring* only. Skills *execute* in Wasmtime per the audit Phase 5 architecture — see `../docs/SPEC.md`.

## Prerequisites

```bash
# Bun
curl -fsSL https://bun.sh/install | bash

# Rust target for wasm32-wasi (used by the build step)
rustup target add wasm32-wasi

# wasmtime CLI for smoke tests
curl https://wasmtime.dev/install.sh -sSf | bash
```

## Quick start

```bash
cd toolchain
bun install
bun run build templates/ts-skill   # builds the reference hello-world skill
```

The build script (`build.ts`) compiles TypeScript to a single bundled JS file, then wraps it for `wasm32-wasi` Component-Model output. Output lands at `templates/ts-skill/dist/skill.wasm`.

## Templates

| Template | Purpose |
|---|---|
| `templates/ts-skill/` | Reference TypeScript skill — hello-world that demonstrates the SKILL.md + executable component pattern |

More templates land in Phase 2 (Python via componentize-py, additional language scaffolds).

## Status

- Scaffolding shipped.
- Bun + esbuild bundling shipped.
- wasm32-wasi component compilation is scaffolded — by default emits a placeholder `dist/skill.wasm` with a clear install hint. **Real Component-Model output requires the one-shot Phase-5 activation** (install `wasm32-wasi` target + wasmtime CLI + jco). See `../docs/PHASE_5_ACTIVATION.md`.
- `wit-bindgen` glue for skill-to-host calls lands during Phase-5 activation.

More templates pending (Python via componentize-py, additional language scaffolds).
