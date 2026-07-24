#!/usr/bin/env node
// task-state.mjs — TASK-IMP-144: transition-locked state engine entrypoint.
// Writes frontmatter status, then invokes backlog-mutate flip (HITL + IMP-143 artifacts).
// Regenerators refuse invented edges without flip receipts (migrate_improvement_to_task.py).
import {
  readFileSync, writeFileSync, existsSync, renameSync, openSync, fsyncSync, closeSync, readdirSync,
} from "node:fs";
import { join, resolve, dirname } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { randomBytes } from "node:crypto";

const __dirname = dirname(fileURLToPath(import.meta.url));
const BM = join(__dirname, "backlog-mutate.mjs");

function findRoot(explicit) {
  if (explicit) return resolve(explicit);
  let d = process.cwd();
  for (;;) {
    if (existsSync(join(d, "docs", "tasks"))) return d;
    const parent = dirname(d);
    if (parent === d) return process.cwd();
    d = parent;
  }
}

function resolveSpec(root, id) {
  const base = join(root, "docs", "tasks");
  const hits = [];
  let mods = [];
  try { mods = readdirSync(base); } catch { return hits; }
  for (const mod of mods) {
    if (mod.startsWith("_") || mod.startsWith(".")) continue;
    let names = [];
    try { names = readdirSync(join(base, mod)); } catch { continue; }
    for (const name of names) {
      const m = /^(TASK-[A-Z0-9]+-\d+)/.exec(name);
      if (!m || m[1] !== id) continue;
      const spec = join(base, mod, name, "spec.md");
      if (existsSync(spec)) hits.push(spec);
    }
  }
  return hits;
}

function frontmatterStatus(text) {
  if (!text.startsWith("---")) return { error: "no opening fence" };
  const end = text.indexOf("\n---", 3);
  if (end === -1) return { error: "no closing fence" };
  const head = text.slice(3, end);
  const m = /^status:\s*(.+)$/m.exec(head);
  return {
    status: m ? m[1].trim().replace(/^["']|["']$/g, "") : null,
    head,
    body: text.slice(end + 4),
    error: m ? null : "no status field",
  };
}

function writeFrontmatterStatus(specPath, to) {
  const raw = readFileSync(specPath, "utf8");
  const fm = frontmatterStatus(raw);
  if (fm.error && fm.error !== "no status field") throw new Error(fm.error);
  let head = fm.head;
  if (/^status:\s*/m.test(head)) head = head.replace(/^status:\s*.*$/m, `status: ${to}`);
  else head = `${head.replace(/\s*$/, "")}\nstatus: ${to}\n`;
  const body = fm.body.startsWith("\n") ? fm.body : `\n${fm.body}`;
  const next = `---${head}\n---${body}`;
  const tmp = `${specPath}.tmp.${randomBytes(4).toString("hex")}`;
  writeFileSync(tmp, next);
  const fd = openSync(tmp, "r");
  try { fsyncSync(fd); } finally { closeSync(fd); }
  renameSync(tmp, specPath);
  return fm.status;
}

function usage() {
  process.stderr.write(
    "usage: node task-state.mjs [--root <repo>] [--json] transition <task-id> <from> <to> " +
    "[--verdict-by a --verdict-evidence p] [--verdict-artifact p]\n",
  );
  process.exit(2);
}

function main(argv) {
  const opts = {};
  const pos = [];
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--json") { opts.json = true; continue; }
    if (a.startsWith("--")) {
      const k = a.slice(2);
      if (i + 1 >= argv.length) usage();
      opts[k] = argv[++i];
      continue;
    }
    pos.push(a);
  }
  const [cmd, id, from, to] = pos;
  if (cmd !== "transition" || !id || !from || !to) usage();
  const root = findRoot(opts.root);
  const hits = resolveSpec(root, id);
  if (hits.length !== 1) {
    process.stderr.write(`task-state: expected exactly one spec for ${id}, found ${hits.length}\n`);
    return 6;
  }
  const specPath = hits[0];
  let prior;
  try { prior = writeFrontmatterStatus(specPath, to); }
  catch (e) {
    process.stderr.write(`task-state: cannot write frontmatter: ${e.message}\n`);
    return 2;
  }
  const flipArgs = [];
  if (opts.json) flipArgs.push("--json");
  flipArgs.push("flip", id, from, to, "--root", root);
  if (opts["verdict-by"]) flipArgs.push("--verdict-by", opts["verdict-by"]);
  if (opts["verdict-evidence"]) flipArgs.push("--verdict-evidence", opts["verdict-evidence"]);
  if (opts["verdict-artifact"]) flipArgs.push("--verdict-artifact", opts["verdict-artifact"]);
  const res = spawnSync(process.execPath, [BM, ...flipArgs], { encoding: "utf8" });
  if (res.stderr) process.stderr.write(res.stderr);
  if (res.status !== 0) {
    if (prior) {
      try { writeFrontmatterStatus(specPath, prior); }
      catch (e) { process.stderr.write(`task-state: restore failed: ${e.message}\n`); }
    }
    process.stderr.write(`task-state: flip failed (exit ${res.status}); frontmatter restored to '${prior}'\n`);
    if (res.stdout) process.stdout.write(res.stdout);
    return res.status || 1;
  }
  if (res.stdout) process.stdout.write(res.stdout);
  else process.stdout.write(`task-state: ${id} ${from} -> ${to} (fm+index+receipt)\n`);
  return 0;
}

process.exit(main(process.argv.slice(2)));
