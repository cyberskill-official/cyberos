#!/usr/bin/env tsx
/**
 * gen-module.ts — stamp `apps/{module}/` for every entry in modules.yaml.
 *
 * Reads the canonical apps/_template/ tree and substitutes the per-module
 * variables ({{MODULE}}, {{module}}, {{PORT}}, etc.). Idempotent: a file
 * with a `<!-- module: {CODE} -->` marker (or generator-known content) is
 * preserved on re-run unless `--force`.
 *
 * CLI:
 *   pnpm gen:module                          # stamp every module
 *   pnpm gen:module -- --module AUTH         # only AUTH
 *   pnpm gen:module -- --phase P0            # only P0 modules
 *   pnpm gen:module -- --dry-run             # preview without writing
 *   pnpm gen:module -- --force               # overwrite even hand-edited files
 */

import { mkdirSync, readFileSync, writeFileSync, existsSync, readdirSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { loadModules, REPO_ROOT, type ModuleEntry } from "./lib/modules.ts";
import type { Phase } from "./lib/types.ts";

const TEMPLATE_DIR = join(REPO_ROOT, "apps/_template");

interface Options {
  module?: string;
  phase?: Phase;
  dryRun: boolean;
  force: boolean;
}

const PHASES: readonly Phase[] = ["P0", "P1", "P2", "P3", "P4"];

function parseArgs(argv: string[]): Options {
  const opts: Options = { dryRun: false, force: false };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    switch (a) {
      case "--dry-run":
        opts.dryRun = true;
        break;
      case "--force":
        opts.force = true;
        break;
      case "--module":
        opts.module = argv[++i];
        break;
      case "--phase": {
        const next = argv[++i];
        if (!PHASES.includes(next as Phase)) die(`--phase must be P0..P4`);
        opts.phase = next as Phase;
        break;
      }
      case "-h":
      case "--help":
        printHelp();
        process.exit(0);
      default:
        if (a?.startsWith("--")) die(`unknown flag ${a}`);
    }
  }
  return opts;
}

function printHelp(): void {
  console.log(`gen-module — stamp apps/{module}/ from modules.yaml

USAGE
  pnpm gen:module [-- options]

OPTIONS
  --module <CODE>     Stamp only this module (e.g. AUTH).
  --phase  <P0..P4>   Stamp only modules in this phase.
  --dry-run           Preview without writing.
  --force             Overwrite even hand-edited files.
`);
}

function die(msg: string): never {
  console.error(`error: ${msg}`);
  process.exit(1);
}

/** Build the substitution map for a module. */
function vars(m: ModuleEntry): Record<string, string> {
  const dependsList =
    m.depends_on && m.depends_on.length > 0
      ? m.depends_on
          .map(
            (d) =>
              `- \`${d}\` — see [docs/feature-requests/](../../docs/feature-requests/) for the dependency contract`,
          )
          .join("\n")
      : "_None — this module is foundational._";
  return {
    MODULE: m.code,
    module: m.code.toLowerCase(),
    NAME: m.name,
    PHASE: m.phase,
    PORT: String(m.port),
    PACKAGE: m.package,
    NAMESPACE: m.graphql_namespace,
    SCHEMA: m.prisma_schema,
    TYPE: titleCase(m.code), // "AUTH" -> "Auth", "PROJ" -> "Proj"
    DEPENDS_ON: dependsList,
  };
}

function titleCase(code: string): string {
  return code.charAt(0) + code.slice(1).toLowerCase();
}

function substitute(content: string, vars: Record<string, string>): string {
  return content.replace(/\{\{(\w+)\}\}/g, (_, key: string) => {
    if (!(key in vars)) {
      throw new Error(`unknown template var {{${key}}}`);
    }
    return vars[key]!;
  });
}

/** Walk a directory yielding [absSourcePath, relSourcePath] pairs. */
function* walk(dir: string, base = dir): Generator<[string, string]> {
  for (const name of readdirSync(dir)) {
    const abs = join(dir, name);
    const s = statSync(abs);
    if (s.isDirectory()) yield* walk(abs, base);
    else yield [abs, relative(base, abs)];
  }
}

/**
 * Detect hand-edits — if a file lacks the `<!-- module: {CODE} -->` marker
 * (or has the wrong code) it's been hand-edited; preserve unless --force.
 *
 * Files without a marker channel (binary or comment-incompatible) are
 * always rewritten — there's no safe way to detect drift on them.
 */
function isHandEdited(filePath: string, expectedCode: string): boolean {
  if (!existsSync(filePath)) return false;
  // Only check files that should carry our HTML/JS/MD comment marker.
  const ext = filePath.slice(filePath.lastIndexOf("."));
  if (![".md", ".ts", ".tsx", ".js", ".prisma"].includes(ext)) return false;
  const text = readFileSync(filePath, "utf8");
  const m = text.match(/<!--\s*module:\s*([A-Z]+)\s*-->|\/\/\s*module:\s*([A-Z]+)/);
  if (!m) return false; // no marker — never had one (or doesn't need one)
  const tag = m[1] ?? m[2];
  return tag !== expectedCode;
}

function shouldEmitModule(m: ModuleEntry, opts: Options): boolean {
  if (opts.module && m.code !== opts.module) return false;
  if (opts.phase && m.phase !== opts.phase) return false;
  return true;
}

function main(): void {
  const opts = parseArgs(process.argv.slice(2));
  const { modules } = loadModules();
  let written = 0;
  let unchanged = 0;
  let preserved = 0;
  let skipped = 0;

  for (const m of modules) {
    if (!shouldEmitModule(m, opts)) {
      skipped++;
      continue;
    }
    const v = vars(m);
    const moduleRoot = join(REPO_ROOT, "apps", m.code.toLowerCase());

    for (const [absSource, rel] of walk(TEMPLATE_DIR)) {
      const absDest = join(moduleRoot, rel);
      const isText = !/\.(png|jpg|jpeg|gif|webp|ico|woff2?|ttf)$/i.test(rel);

      if (!isText) {
        // Binary asset — copy verbatim.
        if (!opts.dryRun && !existsSync(absDest)) {
          mkdirSync(dirname(absDest), { recursive: true });
          writeFileSync(absDest, readFileSync(absSource));
          written++;
        }
        continue;
      }

      const raw = readFileSync(absSource, "utf8");
      let rendered: string;
      try {
        rendered = substitute(raw, v);
      } catch (e) {
        die(`${m.code}: ${rel}: ${(e as Error).message}`);
      }

      if (opts.dryRun) {
        console.log(`  ✓ ${m.code} → ${relative(REPO_ROOT, absDest)}`);
        continue;
      }

      if (!opts.force && isHandEdited(absDest, m.code)) {
        preserved++;
        continue;
      }

      mkdirSync(dirname(absDest), { recursive: true });
      if (existsSync(absDest) && readFileSync(absDest, "utf8") === rendered) {
        unchanged++;
        continue;
      }
      writeFileSync(absDest, rendered);
      written++;
    }
  }

  console.log(`\n=== gen-module summary ===`);
  console.log(`  modules:   ${modules.length}`);
  console.log(`  written:   ${written}`);
  console.log(`  unchanged: ${unchanged}`);
  console.log(`  preserved: ${preserved}`);
  console.log(`  skipped:   ${skipped} (filtered out)`);
}

main();
