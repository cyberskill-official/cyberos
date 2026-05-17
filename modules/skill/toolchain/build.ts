#!/usr/bin/env bun
/**
 * build.ts — Compile a CyberOS Skill from TypeScript source to a single
 * bundled .js, then (when jco is available) componentize to a real
 * wasm32-wasi component for the Wasmtime host (Phase 5).
 *
 * Usage:
 *   bun run build.ts <skill-dir>
 *
 * Args:
 *   <skill-dir>  Path to a skill directory containing SKILL.md and src/index.ts
 *
 * Output:
 *   <skill-dir>/dist/skill.js   — bundled JS (esbuild output)
 *   <skill-dir>/dist/skill.wasm — wasm32-wasi component:
 *                                   * real component if @bytecodealliance/jco
 *                                     is on PATH (bunx jco componentize)
 *                                   * 8-byte stub fallback otherwise (with a
 *                                     pointer to install instructions)
 *
 * The Rust host (crates/host/src/wasm.rs) consumes the .wasm via
 * Component::from_file → AOT-caches under ~/.cyberos/cache/wasm/.
 */

import { build } from "esbuild";
import { mkdirSync, writeFileSync, existsSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "node:path";

const skillDir = process.argv[2];
if (!skillDir) {
  console.error("usage: bun run build.ts <skill-dir>");
  process.exit(2);
}

const absSkill = resolve(skillDir);
const entry = join(absSkill, "src/index.ts");
const skillMd = join(absSkill, "SKILL.md");
const dist = join(absSkill, "dist");
const witPath = resolve(__dirname, "../crates/host/wit/cyberos-skill.wit");

if (!existsSync(entry)) {
  console.error(`error: missing entry point ${entry}`);
  process.exit(2);
}
if (!existsSync(skillMd)) {
  console.error(`error: missing SKILL.md at ${skillMd}`);
  process.exit(2);
}
mkdirSync(dist, { recursive: true });

console.log(`[build] esbuild ${entry} -> ${dist}/skill.js`);
await build({
  entryPoints: [entry],
  bundle: true,
  format: "esm",
  target: ["es2022"],
  platform: "neutral",
  outfile: join(dist, "skill.js"),
  minify: false,
  sourcemap: "inline",
});
console.log(`[build] skill.js OK`);

// Phase 5: try a real componentize via @bytecodealliance/jco. If jco isn't
// installed, fall back to the 8-byte stub so the executor=auto detection
// flow stays meaningful for tests.
function tryComponentize(): boolean {
  const probe = spawnSync("bunx", ["--bun", "jco", "--version"], {
    stdio: ["ignore", "pipe", "pipe"],
    encoding: "utf8",
  });
  if (probe.status !== 0) {
    return false;
  }
  if (!existsSync(witPath)) {
    console.log(`[build] componentize: WIT not found at ${witPath}, skipping`);
    return false;
  }
  console.log(`[build] componentize: jco ${probe.stdout.trim()} -> ${dist}/skill.wasm`);
  const r = spawnSync(
    "bunx",
    [
      "--bun",
      "jco",
      "componentize",
      join(dist, "skill.js"),
      "--wit",
      witPath,
      "--world",
      "skill",
      "--out",
      join(dist, "skill.wasm"),
    ],
    { stdio: "inherit" },
  );
  if (r.status !== 0) {
    console.warn(`[build] componentize failed (status=${r.status}) — falling back to stub`);
    return false;
  }
  console.log(`[build] skill.wasm OK (real component)`);
  return true;
}

if (!tryComponentize()) {
  const stubWasm = new Uint8Array([
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
  ]); // bare WebAssembly magic + version (empty module)
  writeFileSync(join(dist, "skill.wasm"), stubWasm);
  console.log(`[build] skill.wasm (stub, ${stubWasm.length} bytes)`);
  console.log(
    `[build] To produce a real component, install jco:\n` +
      `        cd toolchain && bun add @bytecodealliance/jco @bytecodealliance/preview2-shim\n` +
      `        and rerun this build.`,
  );
}

console.log(`[build] done.`);
