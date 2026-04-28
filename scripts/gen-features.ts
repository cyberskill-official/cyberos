#!/usr/bin/env tsx
/**
 * gen-features.ts — bulk generator for feature_request@1 markdown files.
 *
 * Reads:   docs/roadmap/tasks.yaml          (canonical FR inventory)
 *          docs/templates/feature_request.md (template@1 stub)
 *
 * Writes:  docs/feature-requests/{phase}/{module}/FR-{MOD}-{NNN}.md
 *
 * Idempotent — re-running with no YAML changes produces no diffs.
 *
 * CLI:
 *   pnpm gen:features                          # emit all FR files
 *   pnpm gen:features -- --module AUTH         # filter by module
 *   pnpm gen:features -- --phase P0            # filter by phase
 *   pnpm gen:features -- --dry-run             # preview without writing
 *   pnpm gen:features -- --validate-only       # parse YAML, no emit
 *   pnpm gen:features -- --force               # overwrite hand-edited files
 */

import { readFileSync, writeFileSync, mkdirSync, existsSync, readdirSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parse as parseYaml } from "yaml";

import { serializeFrontmatter } from "./lib/frontmatter.ts";
import {
  ENUMS,
  FRONTMATTER_KEY_ORDER,
  TEMPLATE_ID,
  validateDoc,
} from "./lib/schema.ts";
import type {
  GenOptions,
  Phase,
  TaskEntry,
  TasksYaml,
  Finding,
} from "./lib/types.ts";
import { parseDoc } from "./lib/frontmatter.ts";

// ─── paths ──────────────────────────────────────────────────────────────────
const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const TASKS_YAML = join(ROOT, "docs/roadmap/tasks.yaml");
const TEMPLATE_FILE = join(ROOT, "docs/templates/feature-request/FEATURE_REQUEST.md");
const OUT_ROOT = join(ROOT, "docs/feature-requests");

// ─── CLI parsing ───────────────────────────────────────────────────────────
function parseArgs(argv: string[]): GenOptions {
  const opts: GenOptions = {
    dryRun: false,
    validateOnly: false,
    force: false,
  };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    switch (a) {
      case "--dry-run":
        opts.dryRun = true;
        break;
      case "--validate-only":
        opts.validateOnly = true;
        break;
      case "--force":
        opts.force = true;
        break;
      case "--module":
        opts.module = argv[++i];
        break;
      case "--phase": {
        const next = argv[++i];
        if (!ENUMS.phase.includes(next as never)) {
          die(`--phase must be one of ${ENUMS.phase.join(", ")}`);
        }
        opts.phase = next as Phase;
        break;
      }
      case "-h":
      case "--help":
        printHelp();
        process.exit(0);
      default:
        if (a?.startsWith("--")) die(`unknown flag: ${a}`);
    }
  }
  return opts;
}

function printHelp(): void {
  console.log(`gen-features — emit feature_request@1 files from tasks.yaml

USAGE
  pnpm gen:features [-- options]

OPTIONS
  --module <CODE>     Only emit FRs for this module code (e.g. AUTH).
  --phase  <P0..P4>   Only emit FRs in this phase.
  --dry-run           Show planned writes without touching disk.
  --validate-only     Parse YAML and template; emit nothing.
  --force             Overwrite files even if hand-edits would be lost.
  -h, --help          Show this help.
`);
}

function die(msg: string): never {
  console.error(`error: ${msg}`);
  process.exit(1);
}

// ─── Loaders ────────────────────────────────────────────────────────────────
function loadTasks(): TasksYaml {
  const raw = readFileSync(TASKS_YAML, "utf8");
  const data = parseYaml(raw) as TasksYaml;
  if (data.template !== TEMPLATE_ID) {
    die(`tasks.yaml.template must be ${TEMPLATE_ID}, got ${data.template}`);
  }
  if (!Array.isArray(data.tasks)) die("tasks.yaml is missing `tasks: [...]`");
  return data;
}

function loadTemplateBody(): string {
  const raw = readFileSync(TEMPLATE_FILE, "utf8");
  const { body } = parseDoc(raw);
  return body;
}

// ─── Emission ───────────────────────────────────────────────────────────────
/**
 * Canonical frontmatter only — matches `@cyberskill/templates` v1.0.0
 * `feature-request.schema.json` which has `additionalProperties: false`.
 *
 * The bookkeeping fields (id, module, phase, moscow, depends_on, tags) live
 * in `tasks.yaml` and the on-disk path encodes phase + module + id, so we
 * never lose them — they're just not in the artifact's frontmatter.
 */
function buildFrontmatter(t: TaskEntry, today: string): Record<string, unknown> {
  const fm: Record<string, unknown> = {
    title: t.title,
    author: t.author ?? "@cyberos-bot",
    department: t.department,
    status: t.status ?? "draft",
    priority: t.priority,
    created_at: today,
    ai_authorship: t.ai_authorship ?? "none",
    feature_type: t.feature_type,
    eu_ai_act_risk_class: t.eu_ai_act_risk_class,
    client_visible: t.client_visible,
    template: TEMPLATE_ID,
  };
  // target_release is optional in the schema; only emit when set
  if (t.target_release) fm.target_release = t.target_release;
  return fm;
}

/**
 * Build the body — start from the canonical English template, splice the
 * spec-extracted summary into the Summary section, and add a discreet
 * source-of-truth back-reference comment so readers can trace back to SRS.
 *
 * The body stays English-only; Vietnamese glosses live in README_VI files,
 * never inline (per @cyberskill/templates v1.0.0 design).
 */
function buildBody(template: string, t: TaskEntry): string {
  const sourceRef = `<!-- source: SRS ${t.id} · module ${t.module} · phase ${t.phase} · moscow ${t.moscow} -->`;
  // Replace the placeholder Summary paragraph with the spec-extracted line.
  const withSummary = template.replace(
    /(## Summary\n\n)A single-paragraph summary\.[^\n]*/,
    `$1${t.summary.trim()}\n\n${sourceRef}`,
  );
  return withSummary;
}

function targetPath(t: TaskEntry): string {
  return join(OUT_ROOT, t.phase, t.module, `${t.id}.md`);
}

/** Detect hand-edits by reading the `<!-- source: SRS … -->` back-reference
 *  comment that the generator always emits. If the comment is missing or its
 *  FR id doesn't match, the file has been hand-edited; preserve it unless
 *  --force is passed. */
function isHandEdited(filePath: string, expectedFrId: string): boolean {
  if (!existsSync(filePath)) return false;
  const text = readFileSync(filePath, "utf8");
  const m = text.match(/<!--\s*source:\s*SRS\s+(FR-[A-Z]+-\d+)\b/);
  if (!m) return true;
  return m[1] !== expectedFrId;
}

function shouldEmit(t: TaskEntry, opts: GenOptions): boolean {
  if (opts.module && t.module !== opts.module) return false;
  if (opts.phase && t.phase !== opts.phase) return false;
  return true;
}

// ─── Main ──────────────────────────────────────────────────────────────────
function main(): void {
  const opts = parseArgs(process.argv.slice(2));
  const yaml = loadTasks();
  const template = loadTemplateBody();
  const today = new Date().toISOString().slice(0, 10);

  const findings: Finding[] = [];
  let written = 0;
  let skipped = 0;
  let preserved = 0;
  let unchanged = 0;

  console.log(`gen-features: ${yaml.tasks.length} tasks loaded from ${TASKS_YAML}`);
  if (opts.module) console.log(`  filter: module=${opts.module}`);
  if (opts.phase) console.log(`  filter: phase=${opts.phase}`);
  if (opts.dryRun) console.log(`  dry-run: no files written`);
  if (opts.validateOnly) console.log(`  validate-only: parsing only, no emit`);

  for (const t of yaml.tasks) {
    if (!shouldEmit(t, opts)) {
      skipped++;
      continue;
    }
    const fm = buildFrontmatter(t, today);
    const body = buildBody(template, t);
    const path = targetPath(t);

    // Validate before writing
    const docFindings = validateDoc(path, fm, body);
    findings.push(...docFindings);
    if (docFindings.some((f) => f.level === "error")) {
      console.error(`✗ ${t.id} validation errors — file not written:`);
      for (const f of docFindings.filter((x) => x.level === "error")) {
        console.error(`  · ${f.message}`);
      }
      continue;
    }

    if (opts.validateOnly || opts.dryRun) {
      console.log(`  ✓ ${t.id} → ${path.replace(ROOT + "/", "")}`);
      continue;
    }

    if (!opts.force && isHandEdited(path, t.id)) {
      preserved++;
      console.log(`  ↷ ${t.id} preserved (hand-edited; --force to overwrite)`);
      continue;
    }

    mkdirSync(dirname(path), { recursive: true });
    const text = serializeFrontmatter(fm, FRONTMATTER_KEY_ORDER) + body;
    // True idempotency: skip the write when on-disk content is identical.
    if (existsSync(path) && readFileSync(path, "utf8") === text) {
      unchanged++;
      continue;
    }
    writeFileSync(path, text);
    written++;
  }

  // Optionally check for orphans — files that exist but have no YAML entry
  if (!opts.module && !opts.phase && existsSync(OUT_ROOT)) {
    const allYamlIds = new Set(yaml.tasks.map((t) => t.id));
    const orphans: string[] = [];
    walk(OUT_ROOT, (p) => {
      if (!p.endsWith(".md")) return;
      const id = p.split("/").pop()!.replace(/\.md$/, "");
      if (!allYamlIds.has(id)) orphans.push(p);
    });
    if (orphans.length) {
      console.warn(`\n  ! ${orphans.length} orphan file(s) found (no YAML entry):`);
      for (const o of orphans) console.warn(`    ${o.replace(ROOT + "/", "")}`);
    }
  }

  // Summary
  console.log(`\n=== gen-features summary ===`);
  console.log(`  written:   ${written}`);
  console.log(`  unchanged: ${unchanged}`);
  console.log(`  preserved: ${preserved}`);
  console.log(`  skipped:   ${skipped}`);
  console.log(`  errors:    ${findings.filter((f) => f.level === "error").length}`);
  console.log(`  warnings:  ${findings.filter((f) => f.level === "warning").length}`);

  const errorCount = findings.filter((f) => f.level === "error").length;
  if (errorCount > 0) process.exit(1);
  process.exit(0);
}

function walk(dir: string, fn: (path: string) => void): void {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    const s = statSync(p);
    if (s.isDirectory()) walk(p, fn);
    else fn(p);
  }
}

main();
