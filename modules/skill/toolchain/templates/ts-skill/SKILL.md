---
name: ts-hello
description: >-
  Reference TypeScript skill — say hello with a structured greeting. Use as a starting template for new CyberOS skills. Do NOT use in production — this is a hello-world scaffold demonstrating the SKILL.md + WASM component pattern. Use when user asks to "reference ts hello" or "look up ts hello".
license: Apache-2.0
compatibility: Fully offline. Compiled to wasm32-wasi via the Bun toolchain.
metadata:
  author: cyberskill
  version: "0.1.0"
  template: "true"
allowed-tools: ""
---

# TS Hello (reference skill)

## When to use

- Never in production — this is a build template.
- As the starting point for `cyberos-skill new --lang ts <name>`.

## What it does

Given an input `{ "name": "string" }`, returns `{ "greeting": "Hello, <name>!" }`.

## Build

```bash
cd toolchain
bun run build templates/ts-skill
```

Output: `dist/skill.wasm` (Phase 1 stub) and `dist/skill.js` (bundled source).

## Layout

| File | Purpose |
|---|---|
| `SKILL.md` | This file — manifest + procedural instructions |
| `src/index.ts` | TypeScript entry point — exports `run(input)` |
| `dist/skill.wasm` | Compiled wasm32-wasi component (build output) |
