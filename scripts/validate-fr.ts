#!/usr/bin/env tsx
/**
 * validate-fr.ts — schema validator for feature_request@1 markdown files.
 *
 * Usage:
 *   pnpm validate:fr                     # validates docs/feature-requests/**\/*.md
 *   pnpm validate:fr -- path/to/dir      # validates a specific subtree
 *   pnpm validate:fr -- --yaml-only      # validates docs/roadmap/tasks.yaml only
 *
 * Exit codes (per template contract):
 *   0  pass
 *   1  errors
 *   2  warnings only
 */

import { readFileSync, readdirSync, statSync, existsSync } from "node:fs";
import { join, resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { parse as parseYaml } from "yaml";

import { parseDoc } from "./lib/frontmatter.ts";
import { ENUMS, validateDoc } from "./lib/schema.ts";
import type { Finding, TasksYaml } from "./lib/types.ts";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const TASKS_YAML = join(ROOT, "docs/roadmap/tasks.yaml");
const DEFAULT_DIR = join(ROOT, "docs/feature-requests");

function main(): void {
  const args = process.argv.slice(2);
  const yamlOnly = args.includes("--yaml-only");
  const target = args.find((a) => !a.startsWith("--"));
  const findings: Finding[] = [];

  if (yamlOnly) {
    findings.push(...validateYaml());
  } else {
    findings.push(...validateYaml());
    const root = target ? resolve(target) : DEFAULT_DIR;
    if (!existsSync(root)) {
      console.warn(`note: target ${root} does not exist (nothing to validate)`);
    } else {
      const stat = statSync(root);
      const files: string[] = [];
      if (stat.isDirectory()) {
        walk(root, (p) => p.endsWith(".md") && files.push(p));
      } else if (root.endsWith(".md")) {
        files.push(root);
      }
      console.log(`validate-fr: scanning ${files.length} file(s) under ${root.replace(ROOT + "/", "")}`);
      for (const f of files) findings.push(...validateOne(f));
    }
  }

  // Report
  const errors = findings.filter((f) => f.level === "error");
  const warnings = findings.filter((f) => f.level === "warning");
  if (errors.length === 0 && warnings.length === 0) {
    console.log(`✓ all checks passed`);
    process.exit(0);
  }
  for (const f of findings) {
    const tag = f.level === "error" ? "ERROR" : "WARN";
    const rel = f.file.replace(ROOT + "/", "");
    console.log(`  ${tag}  ${rel}: ${f.message}`);
  }
  console.log(`\n  errors:   ${errors.length}`);
  console.log(`  warnings: ${warnings.length}`);
  if (errors.length) process.exit(1);
  process.exit(2);
}

function validateOne(filePath: string): Finding[] {
  try {
    const text = readFileSync(filePath, "utf8");
    const { frontmatter, body } = parseDoc(text);
    return validateDoc(filePath, frontmatter, body);
  } catch (e) {
    return [
      {
        level: "error",
        file: filePath,
        message: `parse error: ${(e as Error).message}`,
      },
    ];
  }
}

function validateYaml(): Finding[] {
  if (!existsSync(TASKS_YAML)) {
    return [
      {
        level: "error",
        file: TASKS_YAML,
        message: "tasks.yaml not found",
      },
    ];
  }
  const findings: Finding[] = [];
  const data = parseYaml(readFileSync(TASKS_YAML, "utf8")) as TasksYaml;
  if (!Array.isArray(data.tasks)) {
    findings.push({ level: "error", file: TASKS_YAML, message: "missing tasks: []" });
    return findings;
  }
  const seen = new Set<string>();
  for (const t of data.tasks) {
    if (!t.id) {
      findings.push({ level: "error", file: TASKS_YAML, message: "task missing `id`" });
      continue;
    }
    if (seen.has(t.id)) {
      findings.push({ level: "error", file: TASKS_YAML, message: `duplicate id ${t.id}` });
    }
    seen.add(t.id);
    if (!/^FR-[A-Z]+-\d{3,}$/.test(t.id)) {
      findings.push({
        level: "error",
        file: TASKS_YAML,
        message: `${t.id} does not match FR-{MOD}-{NNN}`,
      });
    }
    if (!ENUMS.phase.includes(t.phase as never)) {
      findings.push({
        level: "error",
        file: TASKS_YAML,
        message: `${t.id} phase ${t.phase} invalid`,
      });
    }
    if (!ENUMS.eu_ai_act_risk_class.includes(t.eu_ai_act_risk_class as never)) {
      findings.push({
        level: "error",
        file: TASKS_YAML,
        message: `${t.id} eu_ai_act_risk_class ${t.eu_ai_act_risk_class} invalid`,
      });
    }
    if (!ENUMS.priority.includes(t.priority as never)) {
      findings.push({
        level: "error",
        file: TASKS_YAML,
        message: `${t.id} priority ${t.priority} invalid`,
      });
    }
    if (!t.title) {
      findings.push({
        level: "warning",
        file: TASKS_YAML,
        message: `${t.id} has empty title`,
      });
    }
    if (typeof t.title === "string" && t.title.length > 72) {
      findings.push({
        level: "error",
        file: TASKS_YAML,
        message: `${t.id} title exceeds 72 chars (${t.title.length})`,
      });
    }
  }
  // depends_on cross-check
  for (const t of data.tasks) {
    for (const dep of t.depends_on ?? []) {
      if (!seen.has(dep)) {
        findings.push({
          level: "error",
          file: TASKS_YAML,
          message: `${t.id} depends_on unknown id ${dep}`,
        });
      }
    }
  }
  return findings;
}

function walk(dir: string, fn: (p: string) => void): void {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    const s = statSync(p);
    if (s.isDirectory()) walk(p, fn);
    else fn(p);
  }
}

main();
