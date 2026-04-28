#!/usr/bin/env tsx
/**
 * gen-roadmap.ts — render docs/ROADMAP.md from docs/roadmap/tasks.yaml.
 *
 * The roadmap is a human-readable view of the same data the FR generator
 * consumes; both must stay in sync.
 */

import { readFileSync, writeFileSync } from "node:fs";
import { join, resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { parse as parseYaml } from "yaml";
import type { TaskEntry, TasksYaml, Phase } from "./lib/types.ts";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const TASKS_YAML = join(ROOT, "docs/roadmap/tasks.yaml");
const OUT = join(ROOT, "docs/ROADMAP.md");

const PHASES: Record<Phase, [string, string]> = {
  P0: [
    "Foundation",
    "AI-native foundation — identity, tenancy, AI gateway, MCP, observability, chat, knowledge layer, mascot. CyberSkill drops Slack/Zalo by P0 exit.",
  ],
  P1: [
    "Run the company",
    "Projects, time, CRM, KB, full HR, full email, payroll core, career path. CyberSkill runs the entire business on CyberOS by P1 exit.",
  ],
  P2: [
    "Operationalize",
    "Invoicing, phantom stock with valuation, project bonus pool with holdback. AI agents perform ≥30% of routine ops via MCP.",
  ],
  P3: [
    "Optimise",
    "Resource allocation and OKR. The platform stops feeling like a v1.",
  ],
  P4: [
    "Externalise",
    "Document signing and client portal — first paying external tenant.",
  ],
};

const MODULE_NAME: Record<string, string> = {
  AUTH: "Authentication & Tenancy", AI: "AI Gateway", MCP: "MCP Server", OBS: "Observability",
  CHAT: "Internal Chat", BRAIN: "Universal Knowledge Layer", GENIE: "Company Mascot AI Assistant",
  PROJ: "Projects & Tasks", TIME: "Time Tracking", CRM: "Customer Relationships",
  KB: "Knowledge Base", HR: "Human Resources", EMAIL: "Email Client", REW: "Total Rewards",
  LEARN: "Career Path & Learning", INV: "Invoicing", ESOP: "Phantom Stock",
  RES: "Resource Allocation", OKR: "Objectives & Key Results", DOC: "Document Signing",
  CP: "Client Portal",
};

const MODULE_ORDER: Record<string, number> = {
  AUTH: 0, AI: 1, MCP: 2, OBS: 3, CHAT: 4, BRAIN: 5, GENIE: 6,
  PROJ: 7, TIME: 8, CRM: 9, KB: 10, HR: 11, EMAIL: 12, REW: 13, LEARN: 14,
  INV: 15, ESOP: 16, RES: 17, OKR: 18, DOC: 19, CP: 20,
};

function emojiFor(m: string): string {
  return ({ MUST: "🔒", SHOULD: "⭐", COULD: "💡", MUST_NOT: "🚫", WONT: "🚫" } as Record<string, string>)[m] ?? "·";
}

function main(): void {
  const yaml = parseYaml(readFileSync(TASKS_YAML, "utf8")) as TasksYaml;
  const byPhase = new Map<Phase, Map<string, TaskEntry[]>>();
  for (const t of yaml.tasks) {
    if (!byPhase.has(t.phase)) byPhase.set(t.phase, new Map());
    const m = byPhase.get(t.phase)!;
    if (!m.has(t.module)) m.set(t.module, []);
    m.get(t.module)!.push(t);
  }
  for (const phaseMap of byPhase.values()) {
    for (const arr of phaseMap.values()) {
      arr.sort((a, b) => parseInt(a.id.split("-").pop()!) - parseInt(b.id.split("-").pop()!));
    }
  }

  const lines: string[] = [];
  lines.push("# CyberOS — Roadmap");
  lines.push("");
  lines.push("> Generated from [`roadmap/tasks.yaml`](./roadmap/tasks.yaml). Do not hand-edit; change the YAML and re-run `pnpm gen:roadmap`.");
  lines.push("> Source of truth: [SRS.md §4](./SRS.md). Phase-gate criteria: [PRD.md §8](./PRD.md), [SRS.md §9.1](./SRS.md).");
  lines.push("");
  lines.push(`**Total: ${yaml.tasks.length} functional requirements across ${new Set(yaml.tasks.map(t => t.module)).size} modules and ${byPhase.size} phases.**`);
  lines.push("");
  lines.push("| Phase | FRs | Modules | Theme |");
  lines.push("|---|---:|---|---|");
  for (const [code, [theme, desc]] of Object.entries(PHASES) as [Phase, [string, string]][]) {
    const phaseMap = byPhase.get(code) ?? new Map();
    const frCount = [...phaseMap.values()].reduce((n, arr) => n + arr.length, 0);
    const mods = [...phaseMap.keys()].sort();
    lines.push(`| **${code}** — ${theme} | ${frCount} | ${mods.join(", ")} | ${desc.split(".")[0]}. |`);
  }
  lines.push("");
  lines.push("Legend: 🔒 MUST · ⭐ SHOULD · 💡 COULD · 🚫 WON'T / MUST NOT");
  lines.push("");
  lines.push("---");
  lines.push("");

  for (const [code, [theme, desc]] of Object.entries(PHASES) as [Phase, [string, string]][]) {
    const phaseMap = byPhase.get(code) ?? new Map();
    const frCount = [...phaseMap.values()].reduce((n, arr) => n + arr.length, 0);
    if (!frCount) continue;
    lines.push(`## ${code} — ${theme}`);
    lines.push("");
    lines.push(desc);
    lines.push("");
    lines.push(`**${frCount} FRs across ${phaseMap.size} modules.**`);
    lines.push("");
    const mods = [...phaseMap.keys()].sort((a, b) => (MODULE_ORDER[a] ?? 99) - (MODULE_ORDER[b] ?? 99));
    for (const mod of mods) {
      const items = phaseMap.get(mod)!;
      const mc: Record<string, number> = {};
      for (const i of items) mc[i.moscow] = (mc[i.moscow] ?? 0) + 1;
      const counts: string[] = [];
      for (const [k, e] of [["MUST", "🔒"], ["SHOULD", "⭐"], ["COULD", "💡"], ["MUST_NOT", "🚫"], ["WONT", "🚫"]] as const) {
        if (mc[k]) counts.push(`${e}${mc[k]}`);
      }
      const risk = items[0]!.eu_ai_act_risk_class;
      const riskChip = risk === "high" ? " · 🟥 EU AI Act high-risk" : risk === "limited" ? " · 🟧 EU AI Act limited" : "";
      lines.push(`### ${code} · ${mod} — ${MODULE_NAME[mod] ?? mod}  <small>${counts.join(" ")}${riskChip}</small>`);
      lines.push("");
      for (const fr of items) {
        const path = `./feature-requests/${code}/${mod}/${fr.id}.md`;
        lines.push(`- [ ] ${emojiFor(fr.moscow)} [\`${fr.id}\`](${path}) — ${fr.title}`);
      }
      lines.push("");
    }
    lines.push("---");
    lines.push("");
  }
  lines.push("## Phase-gate criteria");
  lines.push("");
  lines.push(
    "Entry/exit criteria for each phase live in [PRD §8](./PRD.md) and [SRS §9.1](./SRS.md). Compliance gates (T1/T2/T3) are in [PRD §10](./PRD.md). The verification methods (`[T]` test, `[A]` analysis, `[D]` demonstration, `[I]` inspection) follow IEEE 1233 and are recorded per FR in the SRS.",
  );

  writeFileSync(OUT, lines.join("\n") + "\n");
  console.log(`gen-roadmap: wrote ${OUT.replace(ROOT + "/", "")} (${yaml.tasks.length} FRs)`);
}

main();
