#!/usr/bin/env node
// coverage-ratchet.mjs — install-suite test-touch coverage ratchet (TASK-IMP-012).
//
// Measures the fraction of tools/install scripts (.sh/.mjs under top-level, gates/,
// docs-tools/; excluding tests/) whose basename appears in tools/install/tests/.
// Compares against a committed baseline and fails ONLY on regression.
//
// Usage:
//   node coverage-ratchet.mjs [--repo <root>] [--baseline <file>] [--json] [--write-baseline]
//
// Exit codes:
//   0  ok (pct >= baseline, or --write-baseline succeeded)
//   1  regression (pct < baseline.pct)
//   2  usage / missing baseline (unless --write-baseline) / not a repo

import { readFileSync, writeFileSync, readdirSync, statSync, existsSync } from "node:fs";
import { join, resolve, relative, basename, dirname } from "node:path";
import { spawnSync } from "node:child_process";

const HELP = `coverage-ratchet.mjs - install-suite test-touch coverage ratchet (TASK-IMP-012)

usage: node coverage-ratchet.mjs [--repo <root>] [--baseline <file>] [--json] [--write-baseline]

Measures % of tools/install scripts referenced by tools/install/tests/.
Fails (exit 1) only when current pct is strictly below the baseline.
Missing baseline → exit 2 (use --write-baseline to seed).
`;

function findRoot(explicit) {
  if (explicit) return resolve(explicit);
  const r = spawnSync("git", ["rev-parse", "--show-toplevel"], { encoding: "utf8" });
  if (r.status === 0) return r.stdout.trim();
  let d = process.cwd();
  for (;;) {
    if (existsSync(join(d, ".git"))) return d;
    const parent = dirname(d);
    if (parent === d) return process.cwd();
    d = parent;
  }
}

function listScripts(installDir) {
  const out = [];
  const dirs = [installDir, join(installDir, "gates"), join(installDir, "docs-tools")];
  for (const dir of dirs) {
    if (!existsSync(dir)) continue;
    for (const name of readdirSync(dir)) {
      if (!/\.(sh|mjs)$/.test(name)) continue;
      const abs = join(dir, name);
      try {
        if (!statSync(abs).isFile()) continue;
      } catch {
        continue;
      }
      out.push(abs);
    }
  }
  return out.sort();
}

function listTestFiles(testsDir) {
  if (!existsSync(testsDir)) return [];
  return readdirSync(testsDir)
    .filter((n) => n.startsWith("test_") && (n.endsWith(".sh") || n.endsWith(".mjs")))
    .map((n) => join(testsDir, n))
    .sort();
}

function measure(root) {
  const installDir = join(root, "tools", "install");
  const testsDir = join(installDir, "tests");
  const scripts = listScripts(installDir);
  const tests = listTestFiles(testsDir);
  const corpus = tests.map((f) => readFileSync(f, "utf8")).join("\n");
  const covered = [];
  const uncovered = [];
  for (const abs of scripts) {
    const base = basename(abs);
    if (corpus.includes(base)) covered.push(base);
    else uncovered.push(base);
  }
  const total = scripts.length;
  const cov = covered.length;
  const pct = total === 0 ? 100.0 : Math.round((cov / total) * 1000) / 10;
  return {
    schema: "coverage-ratchet@1",
    metric: "install-script-test-touch",
    pct,
    covered: cov,
    total,
    uncovered: uncovered.sort(),
  };
}

function parseArgs(argv) {
  const opts = { json: false, writeBaseline: false, repo: null, baseline: null };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--help" || a === "-h") return { help: true };
    if (a === "--json") { opts.json = true; continue; }
    if (a === "--write-baseline") { opts.writeBaseline = true; continue; }
    if (a === "--repo" || a === "--baseline") {
      if (i + 1 >= argv.length) throw new Error(`${a} needs a value`);
      opts[a.slice(2) === "repo" ? "repo" : "baseline"] = argv[++i];
      continue;
    }
    throw new Error(`unknown flag ${a}`);
  }
  return opts;
}

function main(argv) {
  let opts;
  try {
    opts = parseArgs(argv);
  } catch (e) {
    process.stderr.write(`coverage-ratchet: ${e.message}\n${HELP}`);
    return 2;
  }
  if (opts.help) {
    process.stdout.write(HELP);
    return 0;
  }
  const root = findRoot(opts.repo);
  const baselinePath = opts.baseline
    ? resolve(root, opts.baseline)
    : join(root, "tools", "install", "coverage-baseline.json");
  const m = measure(root);

  if (opts.writeBaseline) {
    writeFileSync(baselinePath, JSON.stringify({
      schema: "coverage-ratchet@1",
      metric: m.metric,
      pct: m.pct,
      covered: m.covered,
      total: m.total,
      written_note: "seeded by coverage-ratchet.mjs --write-baseline",
    }, null, 2) + "\n");
    process.stderr.write(`coverage-ratchet: wrote baseline pct=${m.pct} → ${relative(root, baselinePath) || baselinePath}\n`);
    if (opts.json) process.stdout.write(JSON.stringify(m) + "\n");
    return 0;
  }

  if (!existsSync(baselinePath)) {
    process.stderr.write(`coverage-ratchet: missing baseline ${relative(root, baselinePath) || baselinePath} — run with --write-baseline to seed\n`);
    return 2;
  }
  let baseline;
  try {
    baseline = JSON.parse(readFileSync(baselinePath, "utf8"));
  } catch (e) {
    process.stderr.write(`coverage-ratchet: baseline unreadable: ${e.message}\n`);
    return 2;
  }
  const basePct = Number(baseline.pct);
  if (Number.isNaN(basePct)) {
    process.stderr.write("coverage-ratchet: baseline.pct is not a number\n");
    return 2;
  }

  if (opts.json) process.stdout.write(JSON.stringify({ ...m, baseline_pct: basePct }) + "\n");
  else {
    process.stderr.write(
      `coverage-ratchet: ${m.covered}/${m.total} = ${m.pct}% (baseline ${basePct}%)\n`,
    );
  }

  if (m.pct < basePct) {
    process.stderr.write(
      `coverage-ratchet: REGRESSION — ${m.pct}% < baseline ${basePct}%\n` +
      `  uncovered: ${m.uncovered.join(", ") || "(none)"}\n`,
    );
    return 1;
  }
  process.stderr.write("coverage-ratchet: OK (no regression)\n");
  return 0;
}

process.exit(main(process.argv.slice(2)));
