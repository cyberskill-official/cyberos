#!/usr/bin/env node
// coverage-scope.mjs — task diff -> per-file coverage, as a coverage-gate@1 skeleton (TASK-IMP-098).
//
// The coverage gate's unit of judgment is "files this task touched", but run-gates
// emits whole-workspace numbers and every gate so far mapped diff to coverage by hand
// (IMPROVEMENT_HANDOFF.md IMP-14). This tool owns the deterministic half: resolve the
// task's diff base, list the touched files, join them to a coverage report, and emit a
// coverage-gate@1 skeleton ready for coverage-gate-author to complete. The judgment
// fields (tests_failed, ecm_rows_uncovered, raw_terminal) stay with the author skill —
// they are emitted as TODO markers, never guessed.
//
// Usage:  node coverage-scope.mjs <task-id> [--base <ref>] [--coverage <file>]
//                                 [--repo <root>] [--out <file>]
//
// Base resolution (spec §1 #1.1 — never guess a range):
//   1. --base <ref> wins (verified against the repo).
//   2. Else scan `git log --format=%H %s` for commits whose SUBJECT names <task-id>
//      AND the word "implementing" — the corpus entry-flip convention (every batch
//      commit that flips a task into implementing says so in its subject, e.g.
//      "TASK-X-001: enter implementing"). The EARLIEST such commit is the base:
//      "touched since implementing was set" means everything after the flip landed.
//      Two or more matches resolve to the earliest AND the ambiguity is noted in the
//      skeleton's range note.
//   3. Else fail loudly (exit 3) demanding --base.
//
// Touched files (spec §1 #1.2): `git diff --name-only <base>...HEAD` (three-dot:
// merge-base to HEAD — only the task's own side), filtered to files existing at HEAD.
// Deletions are excluded from the coverage table but NAMED in the skeleton's notes.
//
// Coverage ingestion (spec §1 #1.3) — exactly the two shapes the detected stacks emit,
// recognized BY NAME; anything else is refused by name with exit 4:
//   coverage-summary.json   c8/istanbul summary: {"total": {...}, "<path>": {lines:
//                           {total, covered, skipped, pct}, ...}} — lines.pct is used.
//                           Absolute path keys are normalized repo-relative.
//   lcov.info               LF (lines found) / LH (lines hit) per SF record ->
//                           pct = LH/LF*100 (2-decimal rounding; LF:0 counts as 100,
//                           matching istanbul's treatment of empty files).
//
// Output (spec §1 #1.4): a coverage-gate@1 skeleton to stdout (or --out):
//   frontmatter  artefact: coverage-gate@1, task, phase: testing, tests_failed: TODO,
//                files_below_90pct: computed (STRICT < 90 — exactly 90 is not below,
//                matching the gate's wording), ecm_rows_uncovered: TODO
//   body         the base...HEAD range line with resolution provenance (+ ambiguity
//                note when the subject-scan matched more than once), the report line,
//                a per-file table (bytewise path order) where every touched file shows
//                its lines.pct or `no-coverage-data` — visible, never silently
//                dropped — a deletions note, and TODO markers for the author skill.
//
// Determinism: no clock, no randomness — identical repo state + report + args =
// byte-identical output (the fixture suite compares expected bytes). Read-only over
// git (log/diff/ls-tree/rev-parse only); the only write is --out. Security class:
// no network; --coverage and --out MUST resolve inside the repo root (spec §3).
//
// Exit codes:
//   0  ok — skeleton emitted
//   2  usage error, not a git repo, unreadable/invalid report content, path outside
//      the repo root, --base does not resolve
//   3  base unresolvable: no --base and no entry-flip commit names the task id —
//      the caller must pass --base (never guess a range)
//   4  unsupported coverage report, refused BY NAME (only coverage-summary.json and
//      lcov.info are ingested)
//
// Node stdlib only (docs-tools convention); git is invoked read-only via child_process.

import { readFileSync, writeFileSync, existsSync, mkdirSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve, relative, isAbsolute, basename, dirname } from "node:path";

class UsageError extends Error {}
class Refusal extends Error { constructor(code, msg) { super(msg); this.code = code; } }

const TASK_ID_RE = /^[A-Za-z0-9._-]+$/;

// ── git plumbing (read-only) ─────────────────────────────────────────────────
function git(root, args) {
  const r = spawnSync("git", ["-C", root, ...args], { encoding: "utf8" });
  if (r.error) throw new UsageError(`git failed to launch: ${r.error.message}`);
  return r;
}
function gitOK(root, args, what) {
  const r = git(root, args);
  if (r.status !== 0) throw new UsageError(`${what}: git ${args.join(" ")} -> ${(r.stderr || "").trim() || `exit ${r.status}`}`);
  return r.stdout;
}

function findRoot(explicit) {
  if (explicit) return resolve(explicit);
  let d = process.cwd();
  for (;;) {
    if (existsSync(join(d, ".git"))) return d;
    const parent = dirname(d);
    if (parent === d) return process.cwd();
    d = parent;
  }
}

const relUnderRoot = (root, p) => {
  const abs = isAbsolute(p) ? p : resolve(root, p);
  const rel = relative(root, abs).split("\\").join("/");
  return rel.startsWith("..") ? null : rel;
};

// ── base resolution (spec §1 #1.1) ───────────────────────────────────────────
function resolveBase(root, taskId, opts) {
  if (opts.base) {
    const r = git(root, ["rev-parse", "--verify", `${opts.base}^{commit}`]);
    if (r.status !== 0) throw new UsageError(`--base '${opts.base}' does not resolve to a commit: ${(r.stderr || "").trim()}`);
    return { sha: r.stdout.trim(), provenance: `--base '${opts.base}'`, ambiguity: null };
  }
  // The corpus entry-flip convention: subject names the task id AND "implementing".
  const log = gitOK(root, ["log", "--format=%H %s"], "cannot read history for the subject scan");
  const matches = [];
  for (const line of log.split("\n")) {
    if (!line) continue;
    const sp = line.indexOf(" ");
    const sha = sp < 0 ? line : line.slice(0, sp);
    const subject = sp < 0 ? "" : line.slice(sp + 1);
    if (subject.includes(taskId) && /implementing/i.test(subject)) matches.push({ sha, subject });
  }
  if (matches.length === 0) {
    throw new Refusal(3, `no --base given and no commit subject names '${taskId}' + "implementing" (the entry-flip convention) - pass --base <ref>; this tool never guesses a range`);
  }
  const earliest = matches[matches.length - 1]; // git log is newest-first
  const ambiguity = matches.length > 1
    ? `Note: ${matches.length} commit subjects name ${taskId} + "implementing"; the EARLIEST was used as base - pass --base to override.`
    : null;
  return { sha: earliest.sha, provenance: `subject-scan (entry-flip commit: "${earliest.subject}")`, ambiguity };
}

// ── touched files (spec §1 #1.2) ─────────────────────────────────────────────
function touchedFiles(root, baseSha) {
  const diff = gitOK(root, ["diff", "--name-only", `${baseSha}...HEAD`], "diff failed");
  const names = diff.split("\n").filter((l) => l !== "");
  const atHead = new Set(gitOK(root, ["ls-tree", "-r", "--name-only", "HEAD"], "ls-tree failed").split("\n").filter((l) => l !== ""));
  const touched = [], deleted = [];
  for (const n of names) (atHead.has(n) ? touched : deleted).push(n);
  touched.sort(); deleted.sort();
  return { touched, deleted };
}

// ── coverage ingestion (spec §1 #1.3 — two shapes, refused by name) ──────────
const round2 = (x) => Math.round(x * 100) / 100;

function ingestIstanbul(root, path) {
  let data;
  try { data = JSON.parse(readFileSync(path, "utf8")); }
  catch (e) { throw new UsageError(`coverage-summary.json unreadable or not JSON: ${e.message}`); }
  if (data === null || typeof data !== "object" || Array.isArray(data) || !("total" in data)) {
    throw new UsageError("coverage-summary.json carries no 'total' key - not a c8/istanbul summary");
  }
  const pct = new Map();
  for (const [key, v] of Object.entries(data)) {
    if (key === "total") continue;
    const p = v && v.lines ? v.lines.pct : undefined;
    if (typeof p !== "number") continue; // istanbul's "Unknown" etc -> no data for that file
    const rel = relUnderRoot(root, key);
    if (rel !== null) pct.set(rel, round2(p));
  }
  return pct;
}

function ingestLcov(root, path) {
  let text;
  try { text = readFileSync(path, "utf8"); }
  catch (e) { throw new UsageError(`lcov.info unreadable: ${e.message}`); }
  const pct = new Map();
  let sf = null, lf = null, lh = null, records = 0;
  const flush = () => {
    if (sf === null) return;
    records += 1;
    const rel = relUnderRoot(root, sf);
    if (rel !== null) pct.set(rel, lf === 0 ? 100 : round2(((lh ?? 0) / (lf ?? 1)) * 100));
    sf = null; lf = null; lh = null;
  };
  for (const raw of text.split("\n")) {
    const line = raw.trimEnd();
    if (line.startsWith("SF:")) { flush(); sf = line.slice(3); lf = 0; lh = 0; }
    else if (line.startsWith("LF:")) lf = Number(line.slice(3));
    else if (line.startsWith("LH:")) lh = Number(line.slice(3));
    else if (line === "end_of_record") flush();
  }
  flush();
  if (records === 0) throw new UsageError("lcov.info carries no SF record - not an lcov tracefile");
  return pct;
}

function ingestCoverage(root, opts) {
  let given = opts.coverage;
  if (!given) {
    const candidates = ["coverage/coverage-summary.json", "coverage/lcov.info"];
    given = candidates.find((c) => existsSync(join(root, c)));
    if (!given) throw new UsageError(`no coverage report found (looked for ${candidates.join(" and ")} under the repo root) - pass --coverage <file>`);
  }
  const rel = relUnderRoot(root, given);
  if (rel === null) throw new UsageError(`--coverage '${given}' resolves outside the repo root - refused (read-only over the repo, spec §3)`);
  const abs = resolve(root, rel);
  if (!existsSync(abs)) throw new UsageError(`coverage report not found: ${rel}`);
  const name = basename(abs);
  if (name === "coverage-summary.json") {
    return { pct: ingestIstanbul(root, abs), rel, shapeNote: "coverage-summary.json shape, lines.pct" };
  }
  if (name === "lcov.info") {
    return { pct: ingestLcov(root, abs), rel, shapeNote: "lcov.info shape, LF/LH per SF record" };
  }
  throw new Refusal(4, `unsupported coverage report '${name}' - this tool ingests exactly coverage-summary.json (c8/istanbul) and lcov.info; refused by name, nothing guessed`);
}

// ── skeleton emitter (spec §1 #1.4) ──────────────────────────────────────────
const fmtPct = (p) => String(round2(p));

function buildSkeleton(taskId, base, headSha, report, touched, deleted) {
  const rows = touched.map((f) => {
    const p = report.pct.get(f);
    if (p === undefined) return { f, pct: "no-coverage-data", status: "no-coverage-data", below: false, nodata: true };
    return { f, pct: fmtPct(p), status: p < 90 ? "below-90" : "ok", below: p < 90, nodata: false };
  });
  const below = rows.filter((r) => r.below).map((r) => r.f);
  const nodata = rows.filter((r) => r.nodata).length;

  const lines = [];
  lines.push("---");
  lines.push("artefact: coverage-gate@1");
  lines.push(`task: ${taskId}`);
  lines.push("phase: testing");
  lines.push("tests_failed: TODO");
  lines.push(`files_below_90pct: [${below.join(", ")}]`);
  lines.push("ecm_rows_uncovered: TODO");
  lines.push("---");
  lines.push(`# Coverage scope skeleton - ${taskId}`);
  lines.push("");
  lines.push(`Range: \`${base.sha}...${headSha}\` (base via ${base.provenance})`);
  if (base.ambiguity) lines.push(base.ambiguity);
  lines.push(`Report: ${report.rel} (${report.shapeNote})`);
  lines.push("");
  lines.push("| file | lines.pct | status |");
  lines.push("|---|---|---|");
  if (rows.length === 0) lines.push("| (no files touched in the range) | - | - |");
  for (const r of rows) lines.push(`| ${r.f} | ${r.pct} | ${r.status} |`);
  lines.push("");
  lines.push("Notes:");
  if (deleted.length > 0) lines.push(`- deleted in range (excluded from the table per #1.2): ${deleted.join(", ")}`);
  if (nodata > 0) lines.push(`- ${nodata} touched file(s) carry no coverage data - visible above, never silently dropped.`);
  lines.push("- TODO (coverage-gate-author): tests_failed from the suite run; ecm_rows_uncovered from the edge-case-matrix cross-walk; raw_terminal attached at authoring.");
  lines.push("");
  return lines.join("\n");
}

// ── CLI shell ────────────────────────────────────────────────────────────────
const HELP = `coverage-scope.mjs - task diff -> per-file coverage, as a coverage-gate@1 skeleton (TASK-IMP-098)

usage: node coverage-scope.mjs <task-id> [--base <ref>] [--coverage <file>] [--repo <root>] [--out <file>]

base resolution (never guesses a range)
  --base <ref> wins; else the EARLIEST commit whose subject names <task-id> AND
  "implementing" (the corpus entry-flip convention) is the base - 2+ matches use the
  earliest and note the ambiguity in the skeleton; no match fails demanding --base.

touched files
  git diff --name-only <base>...HEAD, filtered to files existing at HEAD; deletions
  are excluded from the table but named in the notes.

coverage reports (recognized BY NAME; anything else refused)
  coverage-summary.json   c8/istanbul summary - lines.pct per file
  lcov.info               LF/LH per SF record -> pct (2-decimal; LF:0 counts as 100)
  default when --coverage is omitted: coverage/coverage-summary.json, then
  coverage/lcov.info, under the repo root.

output
  coverage-gate@1 skeleton to stdout (or --out, inside the repo root): computed
  files_below_90pct (STRICT < 90 - exactly 90 is not below), per-file table with
  no-coverage-data rows for touched files absent from the report, deletions note,
  TODO markers for the judgment fields (tests_failed, ecm_rows_uncovered).

exit codes
  0  ok
  2  usage error, not a git repo, unreadable/invalid report content, path outside
     the repo root, --base does not resolve
  3  base unresolvable (no --base, no entry-flip commit) - pass --base
  4  unsupported coverage report, refused by name

deterministic: no clock, no randomness - identical repo + report + args emit
byte-identical bytes. read-only over git; the only write is --out.
`;

function main(argv) {
  const valued = new Set(["base", "coverage", "repo", "out"]);
  const opts = {};
  const positionals = [];
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "-h" || a === "--help") { opts.help = true; continue; }
    if (a.startsWith("--")) {
      const name = a.slice(2);
      if (valued.has(name)) {
        if (i + 1 >= argv.length) { process.stderr.write(`coverage-scope: --${name} needs a value\n`); return 2; }
        opts[name] = argv[++i]; continue;
      }
      process.stderr.write(`coverage-scope: unknown flag '${a}'\n${HELP}`);
      return 2;
    }
    positionals.push(a);
  }
  if (opts.help) { process.stdout.write(HELP); return 0; }
  const [taskId, ...extra] = positionals;
  try {
    if (!taskId) throw new UsageError("no <task-id> given");
    if (extra.length > 0) throw new UsageError(`unexpected positional argument '${extra[0]}'`);
    if (!TASK_ID_RE.test(taskId)) throw new UsageError(`task-id must match ${TASK_ID_RE}`);
    const root = findRoot(opts.repo);
    if (git(root, ["rev-parse", "--is-inside-work-tree"]).stdout.trim() !== "true") {
      throw new UsageError(`${root} is not a git repository (pass --repo <root>)`);
    }
    const base = resolveBase(root, taskId, opts);
    const headSha = gitOK(root, ["rev-parse", "HEAD"], "cannot resolve HEAD").trim();
    const { touched, deleted } = touchedFiles(root, base.sha);
    const report = ingestCoverage(root, opts);
    const skeleton = buildSkeleton(taskId, base, headSha, report, touched, deleted);
    if (opts.out) {
      const outRel = relUnderRoot(root, opts.out);
      if (outRel === null) throw new UsageError(`--out '${opts.out}' resolves outside the repo root - refused (spec §3)`);
      // PR-review fix (Devin, 2026-07-17): a missing parent dir surfaced as an uncaught
      // ENOENT stack trace instead of the tool's clean exit-coded behavior. The dir is
      // created recursively - it stays inside the root because outRel is already validated.
      mkdirSync(dirname(resolve(root, outRel)), { recursive: true });
      writeFileSync(resolve(root, outRel), skeleton);
      process.stderr.write(`coverage-scope: wrote ${outRel}\n`);
    } else {
      process.stdout.write(skeleton);
    }
    return 0;
  } catch (e) {
    if (e instanceof Refusal || e instanceof UsageError) {
      const code = e instanceof Refusal ? e.code : 2;
      process.stderr.write(`coverage-scope: ${e.message}\n`);
      if (code === 2) process.stderr.write("usage: node coverage-scope.mjs <task-id> [--base <ref>] [--coverage <file>] [--repo <root>] [--out <file>] (--help for details)\n");
      return code;
    }
    throw e;
  }
}

process.exitCode = main(process.argv.slice(2));
