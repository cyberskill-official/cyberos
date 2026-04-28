#!/usr/bin/env tsx
/**
 * validate-modules.ts — sanity-check `modules.yaml` and the apps/ tree.
 *
 * Catches:
 *  - duplicate codes / packages / ports
 *  - unknown depends_on references
 *  - apps/{module}/ folders that aren't in modules.yaml (orphans)
 *  - modules.yaml entries that don't have an apps/{module}/ folder yet
 *  - port collisions with reserved ranges (4000 = router, 3000 = shell)
 */

import { existsSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { loadModules, REPO_ROOT, type ModuleEntry } from "./lib/modules.ts";

interface Finding {
  level: "error" | "warning";
  message: string;
}

const RESERVED_PORTS = new Set<number>([3000, 4000]); // shell + router
const APPS_DIR = join(REPO_ROOT, "apps");

function main(): void {
  const findings: Finding[] = [];
  const { modules } = loadModules();

  const codes = new Set<string>();
  const packages = new Set<string>();
  const ports = new Set<number>();

  for (const m of modules) {
    if (codes.has(m.code)) findings.push(err(`duplicate code ${m.code}`));
    codes.add(m.code);

    if (packages.has(m.package)) findings.push(err(`duplicate package ${m.package}`));
    packages.add(m.package);

    if (RESERVED_PORTS.has(m.port)) {
      findings.push(err(`${m.code}: port ${m.port} is reserved (router/shell)`));
    }
    if (ports.has(m.port)) findings.push(err(`${m.code}: duplicate port ${m.port}`));
    ports.add(m.port);

    if (!/^[A-Z]+$/.test(m.code)) {
      findings.push(err(`${m.code}: code must be UPPERCASE_ASCII`));
    }
    if (m.package !== `@cyberos/${m.code.toLowerCase()}`) {
      findings.push(
        err(`${m.code}: package must be @cyberos/${m.code.toLowerCase()}, got ${m.package}`),
      );
    }
    if (m.graphql_namespace !== m.code.toLowerCase()) {
      findings.push(err(`${m.code}: graphql_namespace must be ${m.code.toLowerCase()}`));
    }
    if (m.prisma_schema !== m.code.toLowerCase()) {
      findings.push(err(`${m.code}: prisma_schema must be ${m.code.toLowerCase()}`));
    }
  }

  // Cross-check depends_on
  for (const m of modules) {
    for (const dep of m.depends_on ?? []) {
      if (!codes.has(dep)) {
        findings.push(err(`${m.code}: depends_on unknown module ${dep}`));
      }
    }
  }

  // Orphan check: apps/{x}/ that isn't in modules.yaml
  if (existsSync(APPS_DIR)) {
    const knownDirs = new Set(modules.map((m) => m.code.toLowerCase()).concat(["_template"]));
    for (const name of readdirSync(APPS_DIR)) {
      if (name.startsWith(".")) continue;
      const abs = join(APPS_DIR, name);
      if (!statSync(abs).isDirectory()) continue;
      if (!knownDirs.has(name)) {
        findings.push(warn(`apps/${name}/ is not in modules.yaml (orphan?)`));
      }
    }
  }

  // Missing-app check: modules.yaml entry without a folder yet
  for (const m of modules) {
    const dir = join(APPS_DIR, m.code.toLowerCase());
    if (!existsSync(dir)) {
      findings.push(warn(`${m.code}: apps/${m.code.toLowerCase()}/ does not exist (run gen:module)`));
    }
  }

  if (findings.length === 0) {
    console.log(`✓ modules.yaml: ${modules.length} modules, no issues`);
    process.exit(0);
  }
  for (const f of findings) {
    const tag = f.level === "error" ? "ERROR" : "WARN";
    console.log(`  ${tag}  ${f.message}`);
  }
  const errs = findings.filter((f) => f.level === "error").length;
  console.log(
    `\n  errors: ${errs}  warnings: ${findings.length - errs}`,
  );
  process.exit(errs > 0 ? 1 : 2);
}

function err(message: string): Finding {
  return { level: "error", message };
}
function warn(message: string): Finding {
  return { level: "warning", message };
}

main();
