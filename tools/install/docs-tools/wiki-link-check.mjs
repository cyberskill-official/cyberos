#!/usr/bin/env node
// wiki-link-check.mjs — TASK-IMP-059
// Scan docs/**/*.md for broken relative links and missing TASK- cross-refs.
// Exit 0 when clean; exit 1 with a report when anything fails.
// Node stdlib only.
//
// Checks:
//   1. Relative markdown links [text](path) whose target is a file path must resolve.
//   2. TASK-XXX ids in depends_on / blocks / related_tasks frontmatter must match an
//      existing docs/tasks/**/TASK-XXX* folder (or an allowlisted id).
//   3. Markdown links whose path contains a TASK- stem must resolve (same as #1).
//
// Skips by default: http(s)/mailto/data URLs, pure #anchors, *.html site targets,
// paths under docs/tasks/_archive/, and entries in the allowlist file.
//
// usage:
//   node wiki-link-check.mjs [--root <repo>] [--docs <rel>] [--allowlist <path>] [--json]

import { readFileSync, existsSync, readdirSync, statSync } from "node:fs";
import { dirname, join, normalize, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));

function parseArgs(argv) {
  const out = { root: null, docs: "docs", allowlist: null, json: false };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--json") out.json = true;
    else if (a === "--root") out.root = argv[++i];
    else if (a === "--docs") out.docs = argv[++i];
    else if (a === "--allowlist") out.allowlist = argv[++i];
    else if (a === "--help" || a === "-h") {
      process.stdout.write(
        "usage: node wiki-link-check.mjs [--root <repo>] [--docs <rel>] [--allowlist <path>] [--json]\n",
      );
      process.exit(0);
    } else {
      process.stderr.write(`wiki-link-check: unknown arg ${a}\n`);
      process.exit(2);
    }
  }
  return out;
}

function findRepoRoot(start) {
  let dir = resolve(start || process.cwd());
  for (;;) {
    if (existsSync(join(dir, "docs", "tasks", "BACKLOG.md"))) return dir;
    if (existsSync(join(dir, ".git"))) return dir;
    const parent = dirname(dir);
    if (parent === dir) return resolve(start || process.cwd());
    dir = parent;
  }
}

function walkMd(dir, acc = []) {
  if (!existsSync(dir)) return acc;
  for (const name of readdirSync(dir)) {
    if (name === "node_modules" || name === ".git") continue;
    const p = join(dir, name);
    let st;
    try {
      st = statSync(p);
    } catch {
      continue;
    }
    if (st.isDirectory()) walkMd(p, acc);
    else if (st.isFile() && name.endsWith(".md")) acc.push(p);
  }
  return acc;
}

function loadAllowlist(path) {
  const set = new Set();
  if (!path || !existsSync(path)) return set;
  for (const line of readFileSync(path, "utf8").split(/\r?\n/)) {
    const t = line.trim();
    if (!t || t.startsWith("#")) continue;
    set.add(t);
  }
  return set;
}

/** Build map TASK-XXX -> [folder stems] under docs/tasks (skip _archive, _state, …). */
function indexTasks(tasksRoot) {
  const byId = new Map();
  if (!existsSync(tasksRoot)) return byId;
  for (const mod of readdirSync(tasksRoot)) {
    if (mod.startsWith("_") || mod === "BACKLOG.md") continue;
    const modDir = join(tasksRoot, mod);
    let st;
    try {
      st = statSync(modDir);
    } catch {
      continue;
    }
    if (!st.isDirectory()) continue;
    for (const name of readdirSync(modDir)) {
      const m = name.match(/^(TASK-[A-Z]+-\d{3})/);
      if (!m) continue;
      const full = join(modDir, name);
      try {
        if (!statSync(full).isDirectory()) continue;
      } catch {
        continue;
      }
      const id = m[1];
      if (!byId.has(id)) byId.set(id, []);
      byId.get(id).push(name);
    }
  }
  return byId;
}

const LINK_RE = /\[[^\]]*\]\(([^)]+)\)/g;
const TASK_ID_RE = /TASK-[A-Z]+-\d{3}/g;

function extractFrontmatter(text) {
  if (!text.startsWith("---\n") && !text.startsWith("---\r\n")) return "";
  const end = text.indexOf("\n---", 4);
  if (end < 0) return "";
  return text.slice(4, end);
}

function shouldSkipTarget(raw) {
  if (!raw) return true;
  if (/^(https?:|mailto:|data:)/i.test(raw)) return true;
  if (raw.startsWith("#")) return true;
  // docs-site HTML artefacts (rendered, not source)
  if (/\.html?(#|$)/i.test(raw)) return true;
  // mustache / template placeholders
  if (/[{}]/.test(raw)) return true;
  return false;
}

function underArchive(absPath, repoRoot) {
  const rel = relative(repoRoot, absPath).split(sep).join("/");
  return rel.includes("/_archive/") || rel.startsWith("docs/tasks/_archive/");
}

function main() {
  const opts = parseArgs(process.argv.slice(2));
  const root = findRepoRoot(opts.root || process.cwd());
  const docsDir = resolve(root, opts.docs);
  const allowPath =
    opts.allowlist ||
    join(HERE, "wiki-link-allowlist.txt");
  const allow = loadAllowlist(allowPath);
  const taskIndex = indexTasks(join(root, "docs", "tasks"));

  const files = walkMd(docsDir).filter((f) => !underArchive(f, root));
  const brokenLinks = [];
  const missingTasks = [];

  for (const file of files) {
    const relFile = relative(root, file).split(sep).join("/");
    let text;
    try {
      text = readFileSync(file, "utf8");
    } catch {
      continue;
    }
    const dir = dirname(file);

    // --- relative links ---
    LINK_RE.lastIndex = 0;
    let m;
    while ((m = LINK_RE.exec(text))) {
      let target = m[1].trim().split(/\s+/)[0].replace(/^<|>$/g, "");
      if (shouldSkipTarget(target)) continue;
      const hash = target.indexOf("#");
      const filePart = hash >= 0 ? target.slice(0, hash) : target;
      if (!filePart) continue; // pure anchor after strip — already skipped by shouldSkip when starts with #
      const key = `${relFile} -> ${target}`;
      if (allow.has(key) || allow.has(target) || allow.has(relFile)) continue;
      let decoded;
      try {
        decoded = decodeURIComponent(filePart);
      } catch {
        decoded = filePart;
      }
      const resolved = normalize(join(dir, decoded));
      if (!existsSync(resolved)) {
        brokenLinks.push({ file: relFile, target, resolved: relative(root, resolved).split(sep).join("/") });
      }
    }

    // --- frontmatter TASK refs (spec.md only) ---
    if (!relFile.endsWith("/spec.md") && !relFile.endsWith(`${sep}spec.md`)) {
      // also allow basename check
    }
    if (relFile.endsWith("spec.md")) {
      const fm = extractFrontmatter(text);
      for (const line of fm.split(/\r?\n/)) {
        const fmMatch = line.match(
          /^(depends_on|blocks|related_tasks)\s*:\s*\[([^\]]*)\]\s*$/,
        );
        if (!fmMatch) continue;
        const ids = fmMatch[2].match(TASK_ID_RE) || [];
        for (const id of ids) {
          if (allow.has(id) || allow.has(`${relFile}::${id}`)) continue;
          if (!taskIndex.has(id)) {
            missingTasks.push({ file: relFile, id, field: fmMatch[1] });
          }
        }
      }
    }
  }

  const report = {
    root,
    docs: opts.docs,
    files_scanned: files.length,
    broken_links: brokenLinks.length,
    missing_task_refs: missingTasks.length,
    brokenLinks,
    missingTasks,
  };

  if (opts.json) {
    process.stdout.write(JSON.stringify(report, null, 2) + "\n");
  } else {
    process.stdout.write(
      `wiki-link-check: scanned ${files.length} md files under ${opts.docs}/\n`,
    );
    if (brokenLinks.length) {
      process.stdout.write(`broken relative links (${brokenLinks.length}):\n`);
      for (const b of brokenLinks.slice(0, 50)) {
        process.stdout.write(`  ${b.file} -> ${b.target}\n`);
      }
      if (brokenLinks.length > 50) {
        process.stdout.write(`  … and ${brokenLinks.length - 50} more\n`);
      }
    }
    if (missingTasks.length) {
      process.stdout.write(`missing TASK refs (${missingTasks.length}):\n`);
      for (const t of missingTasks.slice(0, 50)) {
        process.stdout.write(`  ${t.file} ${t.field}: ${t.id}\n`);
      }
      if (missingTasks.length > 50) {
        process.stdout.write(`  … and ${missingTasks.length - 50} more\n`);
      }
    }
    if (!brokenLinks.length && !missingTasks.length) {
      process.stdout.write("ok: no broken relative links or missing TASK refs\n");
    }
  }

  process.exit(brokenLinks.length || missingTasks.length ? 1 : 0);
}

main();
