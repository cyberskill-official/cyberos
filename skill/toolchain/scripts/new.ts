#!/usr/bin/env bun
/**
 * new.ts — Scaffold a new skill from a template.
 *
 * Usage: bun run scripts/new.ts <name> [--template ts-skill]
 */

import { cpSync, existsSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const args = process.argv.slice(2);
const name = args[0];
if (!name) {
  console.error("usage: bun run scripts/new.ts <name> [--template ts-skill]");
  process.exit(2);
}
const templateIdx = args.indexOf("--template");
const template = templateIdx >= 0 ? args[templateIdx + 1] : "ts-skill";

if (!/^[a-z0-9-]{1,64}$/.test(name)) {
  console.error("name must match [a-z0-9-]{1,64}");
  process.exit(2);
}

const here = import.meta.dir;
const src = join(here, "..", "templates", template);
const dest = resolve(process.cwd(), name);

if (!existsSync(src)) {
  console.error(`template not found: ${src}`);
  process.exit(2);
}
if (existsSync(dest)) {
  console.error(`destination already exists: ${dest}`);
  process.exit(2);
}

cpSync(src, dest, { recursive: true });

// Rewrite the SKILL.md name to match.
const skillMdPath = join(dest, "SKILL.md");
let skillMd = readFileSync(skillMdPath, "utf8");
skillMd = skillMd.replace(/^name:\s+.+$/m, `name: ${name}`);
writeFileSync(skillMdPath, skillMd, "utf8");

console.log(`scaffolded ${name} from template ${template} -> ${dest}`);
console.log(`next: cd toolchain && bun run build ${dest}`);
