#!/usr/bin/env node
// skill-log.mjs — record per-audit-verdict pass/fail and render a trust table (TASK-IMP-113).
//
// WHY: all skills have identical standing forever. `task-lint` (deterministic, provable) and
// `code-review-author` (a model's opinion) are trusted equally, and no pass-rate exists for any of
// them because nothing records per-skill outcomes. This tool logs one row per audit verdict and
// renders the aggregate, so "which of our skills actually works?" stops being answered by vibes.
//
// ── THE CORE CONSTRAINT (spec §1.4): TIER LABELS ARE INFORMATIONAL ──────────────────────────────
// The tier column (`auto` / `trial` / `watch` / `no data`) is a REPORT FOR THE OPERATOR, never a
// licence for the machine. NO workflow, gate, or queue may read a tier to decide anything. A skill
// at 60% is a FINDING for a human, not a signal to the machine. The article this borrows from uses
// the tier to authorise unattended shipping; CyberOS's two HITL gates forbid unattended shipping by
// design, so only the MEASUREMENT transfers — the gate does not. This helper therefore RENDERS the
// tier and nothing in the repo READS it back: the sole reader of "tier" is this file's own renderer.
// If you are tempted to branch on a tier anywhere else, that is the exact thing this task forbids.
//
// ── APPEND-ONLY (spec §1.2) ─────────────────────────────────────────────────────────────────────
// `append` writes ONE line and never reads-modifies-writes the ledger: a row, once written, is
// never rewritten or deleted. Concurrent appends from a swarm batch are single short lines and the
// OS orders them; the helper never loads the file to append. `--render` aggregates in ONE streaming
// pass (a running {runs,passes} per skill), never a structure per row, so a 10k-row ledger renders
// in constant memory.
//
// ── UNMEASURED IS NOT FAILING (spec §1.5) ───────────────────────────────────────────────────────
// A skill with ZERO runs renders `no data`, never `0%`. Zero runs (never measured) and zero passes
// over >=1 run (measured, genuinely failing → a real 0%) are different facts and render differently.
//
// GUARD (spec, docs-tools convention): the ledger path is resolved under the repo root and refused
// if it escapes (relUnderRoot — the same guard cone-audit / task-reconcile / verify-goals carry).
// DETERMINISM: `--render` sorts skills bytewise and reads no clock, so identical ledger → identical
// output. (`append` stamps a real ISO-8601 time — that is the event it records, not render output.)
//
// Usage:
//   node skill-log.mjs append --skill <name> --verdict <pass|fail> --task <task-id> \
//        [--at <iso8601>] [--repo <root>] [--ledger <rel-path>]
//   node skill-log.mjs --render [--skills <a,b,c>] [--repo <root>] [--ledger <rel-path>] [--json]
//
// Exit:  0  ran (row appended, or table rendered)
//        2  usage error / guard refusal (bad verdict, missing field, ledger escapes root, ...)
//
// Node stdlib only (docs-tools convention — see cone-audit.mjs, task-lint.mjs). Read-only except the
// single append to the ledger; git is invoked only via rev-parse to locate the root (no writes).

import { readFileSync, existsSync, appendFileSync, mkdirSync } from "node:fs";
import { resolve, relative, isAbsolute, dirname } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const LEDGER_DEFAULT = "docs/tasks/.workflow/skill-trust.tsv";
const VERDICTS = new Set(["pass", "fail"]);
// ISO-8601 with a REQUIRED timezone (Z or ±HH:MM) — the row is a durable audit fact, so an
// ambiguous local time is refused rather than recorded.
const ISO_RE = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$/;
// A field that must never carry the TSV delimiters — a tab or newline would split one logical row
// into two, silently corrupting every later aggregate. Refused, never escaped-and-written.
const hasDelims = (s) => /[\t\n\r]/.test(s);

// ── tier thresholds (spec: mirror the article — auto >=20 runs & >=95%; watch <10 runs OR <90%) ──
// INFORMATIONAL ONLY (§1.4). A total function over (runs, rate): every skill lands in exactly one
// bucket, and the middle band (measured enough to show, not yet `auto`) is `trial`.
export function tierOf(runs, rate) {
  if (runs === 0) return "no data";           // §1.5 — unmeasured, NOT 0%
  if (runs >= 20 && rate >= 0.95) return "auto";
  if (runs < 10 || rate < 0.9) return "watch";
  return "trial";                             // 10<=runs, rate>=0.90, but short of auto
}

// ── guard predicate (shared with cone-audit / task-reconcile / verify-goals / coverage-scope) ────
const relUnderRoot = (root, p) => {
  const rel = relative(root, resolve(root, p));
  return rel === "" || rel.startsWith("..") || isAbsolute(rel) ? null : rel;
};

function gitToplevel(from) {
  const r = spawnSync("git", ["-C", from, "rev-parse", "--show-toplevel"], { encoding: "utf8" });
  return !r.error && r.status === 0 ? r.stdout.trim() : null;
}

class Usage extends Error {}

// ── aggregation: ONE streaming pass, {runs,passes} per skill (spec §1.2 / edge: 10k rows) ────────
export function aggregate(text) {
  const acc = new Map(); // skill -> { runs, passes }
  let malformed = 0;
  for (const line of text.split("\n")) {
    if (line === "") continue;
    const f = line.split("\t");
    if (f.length < 4 || !VERDICTS.has(f[1])) { malformed++; continue; }
    const skill = f[0];
    const cur = acc.get(skill) || { runs: 0, passes: 0 };
    cur.runs += 1;
    if (f[1] === "pass") cur.passes += 1;
    acc.set(skill, cur);
  }
  return { acc, malformed };
}

// rows for a roster ∪ ledger skills, sorted bytewise (determinism). rate is null for zero runs so a
// caller can never mistake unmeasured for a real 0% (§1.5).
export function renderRows(acc, roster) {
  const names = new Set(acc.keys());
  for (const s of roster) if (s) names.add(s);
  return [...names].sort().map((skill) => {
    const { runs, passes } = acc.get(skill) || { runs: 0, passes: 0 };
    const rate = runs === 0 ? null : passes / runs;
    return { skill, runs, passes, rate, tier: tierOf(runs, rate === null ? 0 : rate) };
  });
}

function resolveLedger(argv) {
  const root = resolve(argv.repo || gitToplevel(process.cwd()) || process.cwd());
  const rel = relUnderRoot(root, argv.ledger || LEDGER_DEFAULT);
  if (rel === null) {
    throw new Usage(`ledger path '${argv.ledger || LEDGER_DEFAULT}' escapes the repo root ${root} — refused`);
  }
  return { root, rel, abs: resolve(root, rel) };
}

function doAppend(argv) {
  const { rel, abs } = resolveLedger(argv);
  const skill = argv.skill;
  const verdict = argv.verdict;
  const task = argv.task;
  if (!skill) throw new Usage("append needs --skill <name>");
  if (!verdict || !VERDICTS.has(verdict)) throw new Usage(`append needs --verdict <pass|fail> (got '${verdict ?? ""}')`);
  if (!task) throw new Usage("append needs --task <task-id>");
  for (const [k, v] of [["--skill", skill], ["--task", task]]) {
    if (hasDelims(v)) throw new Usage(`${k} value must not contain a tab or newline — refused`);
  }
  let at = argv.at;
  if (at === undefined) {
    at = new Date().toISOString(); // the real event time — this is the fact the row records
  } else if (!ISO_RE.test(at)) {
    throw new Usage(`--at '${at}' is not an ISO-8601 timestamp with a timezone (e.g. 2026-07-19T10:00:00Z) — refused`);
  }
  const row = `${skill}\t${verdict}\t${task}\t${at}\n`;
  mkdirSync(dirname(abs), { recursive: true }); // guarded under root already
  appendFileSync(abs, row);                     // APPEND-ONLY — never read-modify-write (§1.2)
  process.stderr.write(`skill-log: appended ${skill} ${verdict} ${task} ${at} -> ${rel}\n`);
  return 0;
}

function pad(s, w) { return s.length >= w ? s : s + " ".repeat(w - s.length); }
function padL(s, w) { return s.length >= w ? s : " ".repeat(w - s.length) + s; }

function doRender(argv) {
  const { rel, abs } = resolveLedger(argv);
  const text = existsSync(abs) ? readFileSync(abs, "utf8") : "";
  const { acc, malformed } = aggregate(text);
  const roster = (argv.skills || "").split(",").map((s) => s.trim()).filter(Boolean);
  const rows = renderRows(acc, roster);

  if (argv.json) {
    const out = {
      tool: "skill-trust@1",
      ledger: rel,
      note: "tier labels are informational; no workflow, gate, or queue reads them (TASK-IMP-113 §1.4)",
      malformed_rows: malformed,
      skills: rows.map((r) => ({ skill: r.skill, runs: r.runs, passes: r.passes, rate: r.rate, tier: r.tier })),
    };
    process.stdout.write(JSON.stringify(out, null, 2) + "\n");
    return 0;
  }

  process.stdout.write(`skill-trust@1  ledger=${rel}  (tiers are INFORMATIONAL — nothing reads them; §1.4)\n`);
  if (rows.length === 0) {
    process.stdout.write("  (no skills recorded yet)\n");
    if (malformed) process.stdout.write(`  (${malformed} malformed row(s) skipped)\n`);
    return 0;
  }
  const rateStr = (r) => (r.rate === null ? "no data" : `${(r.rate * 100).toFixed(1)}%`);
  const wSkill = Math.max(5, ...rows.map((r) => r.skill.length));
  const wRuns = Math.max(4, ...rows.map((r) => String(r.runs).length));
  const wPass = Math.max(6, ...rows.map((r) => String(r.passes).length));
  const wRate = Math.max(7, ...rows.map((r) => rateStr(r).length));
  process.stdout.write(
    `  ${pad("SKILL", wSkill)}  ${padL("RUNS", wRuns)}  ${padL("PASSES", wPass)}  ${pad("RATE", wRate)}  TIER\n`,
  );
  for (const r of rows) {
    process.stdout.write(
      `  ${pad(r.skill, wSkill)}  ${padL(String(r.runs), wRuns)}  ${padL(String(r.passes), wPass)}  ${pad(rateStr(r), wRate)}  ${r.tier}\n`,
    );
  }
  if (malformed) process.stdout.write(`  (${malformed} malformed row(s) skipped)\n`);
  return 0;
}

const HELP = `skill-log.mjs — log per-audit-verdict pass/fail; render a trust table (TASK-IMP-113)

usage:
  node skill-log.mjs append --skill <name> --verdict <pass|fail> --task <task-id> \\
       [--at <iso8601>] [--repo <root>] [--ledger <rel-path>]
  node skill-log.mjs --render [--skills <a,b,c>] [--repo <root>] [--ledger <rel-path>] [--json]

append   append ONE row (skill, pass|fail, task-id, ISO-8601 time) to the ledger. APPEND-ONLY:
         a written row is never rewritten or deleted; --at defaults to now.
--render aggregate the ledger and print skill, runs, passes, rate, and a tier label. --skills lists
         extra skills to include so an UNMEASURED skill shows 'no data' (never '0%'). Deterministic.

Tier labels (auto >=20 runs & >=95%; watch <10 runs or <90%; trial between; no data at 0 runs) are
INFORMATIONAL ONLY. No workflow, gate, or queue reads a tier to decide anything — the only reader of
"tier" is this renderer. A low tier is a finding for the operator, never a signal to the machine.

ledger defaults to ${LEDGER_DEFAULT}, resolved under the repo root and refused if it escapes.

exit  0  ran (appended, or rendered)
      2  usage error / guard refusal
`;

export function main(argv) {
  const a = { json: false };
  const pos = [];
  for (let i = 0; i < argv.length; i++) {
    const t = argv[i];
    if (t === "-h" || t === "--help") { process.stdout.write(HELP); return 0; }
    else if (t === "--render") a.render = true;
    else if (t === "--json") a.json = true;
    else if (t === "--skill") { a.skill = argv[++i]; if (a.skill === undefined) return usageErr("--skill needs a value"); }
    else if (t === "--verdict") { a.verdict = argv[++i]; if (a.verdict === undefined) return usageErr("--verdict needs a value"); }
    else if (t === "--task") { a.task = argv[++i]; if (a.task === undefined) return usageErr("--task needs a value"); }
    else if (t === "--at") { a.at = argv[++i]; if (a.at === undefined) return usageErr("--at needs a value"); }
    else if (t === "--repo") { a.repo = argv[++i]; if (a.repo === undefined) return usageErr("--repo needs a value"); }
    else if (t === "--ledger") { a.ledger = argv[++i]; if (a.ledger === undefined) return usageErr("--ledger needs a value"); }
    else if (t === "--skills") { a.skills = argv[++i]; if (a.skills === undefined) return usageErr("--skills needs a value"); }
    else if (t.startsWith("--")) return usageErr(`unknown flag '${t}'`);
    else pos.push(t);
  }
  const mode = a.render || pos[0] === "render" ? "render" : pos[0] === "append" ? "append" : null;
  try {
    if (mode === "append") {
      if (a.render) throw new Usage("append and --render are mutually exclusive");
      return doAppend(a);
    }
    if (mode === "render") return doRender(a);
    throw new Usage(pos.length ? `unknown mode '${pos[0]}' (expected 'append' or --render)` : "no mode given (expected 'append' or --render)");
  } catch (err) {
    if (err instanceof Usage) return usageErr(err.message);
    throw err;
  }
}

function usageErr(msg) {
  process.stderr.write(`skill-log: ${msg}\n`);
  return 2;
}

// Run only when invoked directly; importing the module (the test's unit arms) must not execute main.
const invokedDirectly = (() => {
  try { return process.argv[1] && resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url)); }
  catch { return false; }
})();
if (invokedDirectly) process.exitCode = main(process.argv.slice(2));
