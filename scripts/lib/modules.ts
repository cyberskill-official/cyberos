/**
 * Helpers for reading `modules.yaml` — the canonical 21-module manifest.
 */

import { readFileSync } from "node:fs";
import { join, resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { parse as parseYaml } from "yaml";
import type { Phase } from "./types.ts";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
const MODULES_YAML = join(ROOT, "modules.yaml");

export interface ModuleEntry {
  code: string;
  name: string;
  phase: Phase;
  package: string;
  port: number;
  graphql_namespace: string;
  prisma_schema: string;
  mcp_namespace: string;
  department: string;
  depends_on?: string[];
  eu_ai_act_risk_class?: "not_ai" | "minimal" | "limited" | "high";
}

export interface ModulesYaml {
  version: number;
  generated_apps_root: string;
  modules: ModuleEntry[];
}

export function loadModules(): ModulesYaml {
  const raw = readFileSync(MODULES_YAML, "utf8");
  const data = parseYaml(raw) as ModulesYaml;
  if (!Array.isArray(data.modules)) {
    throw new Error("modules.yaml is missing `modules: [...]`");
  }
  return data;
}

export const MODULES_PATH = MODULES_YAML;
export const REPO_ROOT = ROOT;
