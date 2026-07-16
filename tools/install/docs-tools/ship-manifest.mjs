#!/usr/bin/env node
// ship-manifest.mjs — ship-manifest@1 executor for chief-technology-officer/ship-tasks (TASK-IMP-085).
//
// Implements the workflow's "Resume semantics (ship-manifest@1)" section verbatim
// (modules/cuo/chief-technology-officer/workflows/ship-tasks.md; contract:
// modules/skill/contracts/task/SHIP-MANIFEST.md; python peer: modules/cuo/cuo/ship_manifest.py).
// The manifest is a CACHE of proven work, never an authority — task frontmatter and
// BACKLOG.md remain the record of truth, human gates always re-ask, and deleting a
// manifest costs at most redone work.
//
// Usage:  node ship-manifest.mjs [--json] [--root <repo-root>] <command> ...
//
//   init <task-id> --task-file <spec.md> --workflow-version <v> [--force]
//       Create docs/tasks/.workflow/<task-id>.ship.json, pinning task_sha256 (hash of
//       the spec at init) and workflow_version. Refuses to clobber an existing manifest
//       without --force. Scaffolds .workflow/.gitignore (manifests are session state).
//   record <task-id> <step> <skill> <status> [--artefact <path>] [--verdict <v>] [--routed-back]
//       Record one step outcome {index, skill, status, artefact_path, artefact_sha256,
//       verdict, completed_at}; the artefact is hashed AT RECORD TIME (a missing
//       artefact is a loud exit 2, never a null hash for a claimed file). status is one
//       of pending|done|failed|skipped-conditional; step is 1..31. --routed-back bumps
//       routed_back_count (route-back keeps the manifest per the workflow's terminal rules).
//   verify <task-id> [--workflow-version <v>] [--workflow-doc <path>] [--task-file <spec.md>]
//       Walk the workflow's staleness order and exit with the matching code (below).
//       Current workflow version comes from --workflow-version, else the frontmatter of
//       --workflow-doc, else auto-discovery (modules/cuo/chief-technology-officer/
//       workflows/ship-tasks.md or .cyberos/cuo/ship-tasks.md under the root); when none
//       is available the version check is SKIPPED with a loud note, never silently passed.
//   resume-line <task-id> [--workflow-version <v>] [--workflow-doc <path>]
//       verify, then echo EXACTLY the workflow's mandated line:
//       resume <task-ID>: steps 1-N verified (K artefacts, hashes OK), continuing at step M/31 (<skill>). routed_back_count=R
//       On staleness it exits 3/4/5 like verify — it never claims "hashes OK" it did not prove.
//       <skill> = the recorded step M's skill, else the skill_chain entry parsed from the
//       workflow doc, else "unknown".
//   delete <task-id>
//       Remove the manifest (terminal handling on done). Idempotent: already-absent is exit 0.
//
// Exit codes:
//   0  ok / manifest intact
//   2  usage error, unreadable input, missing manifest, missing artefact at record time
//   3  workflow_version mismatch (needs_human — never a silent mixed-version run)
//   4  task_sha256 mismatch (task spec edited since run start; every step stale, history
//      and routed_back_count retained — verify only reports, it never rewrites)
//   5  artefact hash mismatch (the EARLIEST stale step is named; redo from there)
//
// Writes are two-phase atomic (`.tmp.<nonce>` then rename, fsynced); readers open the
// manifest path exactly and IGNORE tmp files, so a killed write can never corrupt state.
// Clock: timestamps (started_at/updated_at/completed_at) come from CYBEROS_NOW (env) or
// --now <ISO-8601> when set — the injectable clock that keeps test runs deterministic —
// else the wall clock. Everything else is clock-free: JSON is stable-stringified (sorted
// keys), so identical inputs + identical clock = byte-identical output.
// Node stdlib only (docs-tools convention — see task-lint.mjs, md.mjs).

import {
  readFileSync, writeFileSync, renameSync, existsSync, mkdirSync, unlinkSync,
  openSync, fsyncSync, closeSync,
} from "node:fs";
import { createHash, randomBytes } from "node:crypto";
import { join, resolve, dirname, relative, isAbsolute } from "node:path";

const MANIFEST_VERSION = "ship-manifest@1";
const STEP_STATUSES = ["pending", "done", "failed", "skipped-conditional"];
const TOTAL_STEPS = 31;
const ID_RE = /^[A-Za-z0-9._-]+$/; // manifest filename component — never a path

// ── deterministic serialization ──────────────────────────────────────────────
function stableStringify(v, indent = 0) {
  const pad = "  ".repeat(indent), pad2 = "  ".repeat(indent + 1);
  if (v === null || typeof v !== "object") return JSON.stringify(v);
  if (Array.isArray(v)) {
    if (v.length === 0) return "[]";
    return "[\n" + v.map((x) => pad2 + stableStringify(x, indent + 1)).join(",\n") + "\n" + pad + "]";
  }
  const keys = Object.keys(v).sort();
  if (keys.length === 0) return "{}";
  return "{\n" + keys.map((k) => pad2 + JSON.stringify(k) + ": " + stableStringify(v[k], indent + 1)).join(",\n") + "\n" + pad + "}";
}

// ── two-phase atomic write (memory-protocol discipline) ──────────────────────
function atomicWrite(path, text) {
  mkdirSync(dirname(path), { recursive: true });
  const tmp = `${path}.tmp.${randomBytes(6).toString("hex")}`;
  writeFileSync(tmp, text);
  const fd = openSync(tmp, "r");
  try { fsyncSync(fd); } finally { closeSync(fd); }
  renameSync(tmp, path);
}

// ── helpers ──────────────────────────────────────────────────────────────────
function sha256File(path) {
  try { return createHash("sha256").update(readFileSync(path)).digest("hex"); }
  catch { return null; }
}

function findRoot(explicit) {
  if (explicit) return resolve(explicit);
  let d = process.cwd();
  for (;;) {
    if (existsSync(join(d, "docs", "tasks")) || existsSync(join(d, ".git"))) return d;
    const parent = dirname(d);
    if (parent === d) return process.cwd();
    d = parent;
  }
}

const manifestPathFor = (root, id) => join(root, "docs", "tasks", ".workflow", `${id}.ship.json`);
const relUnderRoot = (root, p) => {
  const abs = isAbsolute(p) ? p : resolve(root, p);
  const rel = relative(root, abs);
  return rel.startsWith("..") ? abs : rel.split("\\").join("/");
};

function loadManifest(root, id) {
  const path = manifestPathFor(root, id);
  if (!existsSync(path)) return { err: `no manifest for ${id} at ${relUnderRoot(root, path)} - run init first` };
  let m;
  try { m = JSON.parse(readFileSync(path, "utf8")); }
  catch (e) { return { err: `manifest for ${id} is unreadable/corrupt (${e.message}) - delete and re-init` }; }
  if (m.manifest_version !== MANIFEST_VERSION) {
    return { err: `manifest for ${id} carries manifest_version '${m.manifest_version}' (expected ${MANIFEST_VERSION})` };
  }
  return { m, path };
}

function writeManifest(path, m) { atomicWrite(path, stableStringify(m) + "\n"); }

function parseWorkflowVersionFromDoc(docPath) {
  try {
    const m = /^workflow_version:\s*([^\s#]+)/m.exec(readFileSync(docPath, "utf8"));
    return m ? m[1] : null;
  } catch { return null; }
}

// The workflow doc's frontmatter skill_chain IS the step->skill authority; parse it
// rather than duplicating the chain here (doc-driven: the doc changes, the tool follows).
function chainFromDoc(docPath) {
  const map = new Map();
  let text;
  try { text = readFileSync(docPath, "utf8"); } catch { return map; }
  const re = /\{\s*step:\s*(\d+)\s*,\s*skill:\s*([A-Za-z0-9._-]+)/g;
  let m;
  while ((m = re.exec(text)) !== null) map.set(Number(m[1]), m[2]);
  return map;
}

function discoverWorkflowDoc(root) {
  const candidates = [
    join(root, "modules", "cuo", "chief-technology-officer", "workflows", "ship-tasks.md"),
    join(root, ".cyberos", "cuo", "ship-tasks.md"),
  ];
  for (const c of candidates) if (existsSync(c)) return c;
  return null;
}

// Injectable clock: --now wins, then CYBEROS_NOW, then the wall clock.
function nowISO(opts) {
  const v = opts.now || process.env.CYBEROS_NOW;
  if (v) {
    if (Number.isNaN(Date.parse(v))) throw new UsageError(`--now/CYBEROS_NOW is not ISO-8601: '${v}'`);
    return v;
  }
  return new Date().toISOString();
}

class UsageError extends Error {}

// ── the staleness walk (verify + resume-line share it) ───────────────────────
// Returns {code, message, fields...}; code follows the workflow's order:
// 3 version, 4 task hash, 5 earliest stale artefact, 0 intact.
function walk(root, m, opts) {
  const notes = [];
  // 1. workflow_version — never a silent mixed-version run.
  let current = opts["workflow-version"] || null;
  let source = current ? "--workflow-version" : null;
  const docPath = opts["workflow-doc"] ? resolve(root, opts["workflow-doc"]) : discoverWorkflowDoc(root);
  if (!current && docPath) {
    current = parseWorkflowVersionFromDoc(docPath);
    source = current ? relUnderRoot(root, docPath) : null;
  }
  if (current === null) {
    notes.push("workflow_version check skipped: pass --workflow-version or --workflow-doc (no workflow doc discovered under the root)");
  } else if (current !== m.workflow_version) {
    return {
      code: 3, workflow_version_current: current, workflow_version_source: source,
      message: `workflow_version mismatch (manifest ${m.workflow_version} vs current ${current}) - needs_human, never a silent mixed-version run`,
    };
  }
  // 2. task_sha256 — spec edited since run start = every step stale.
  const taskFile = opts["task-file"] || m.task_file;
  if (!taskFile) return { code: 2, message: "manifest carries no task_file and no --task-file was given - cannot re-hash the task spec" };
  const th = sha256File(resolve(root, taskFile));
  if (th !== m.task_sha256) {
    return {
      code: 4, task_file: taskFile,
      message: `task_sha256 mismatch - task spec changed since run start; every step stale, restart at step 1 (history and routed_back_count retained)`,
    };
  }
  // 3. re-hash EVERY recorded artefact, ascending; earliest mismatch wins.
  const steps = [...m.steps].sort((a, b) => a.index - b.index);
  let artefactsOK = 0;
  for (const s of steps) {
    if (s.status !== "done" || !s.artefact_sha256) continue;
    const h = sha256File(resolve(root, s.artefact_path));
    if (h !== s.artefact_sha256) {
      return {
        code: 5, stale_step: s.index, stale_skill: s.skill, stale_artefact: s.artefact_path,
        message: `stale artefact at step ${s.index} (${s.skill}): ${s.artefact_path} hash mismatch - that step and all later steps stale, redo from step ${s.index}`,
      };
    }
    artefactsOK += 1;
  }
  // 4. intact — continue at the first non-done step (skipped-conditional counts as resolved).
  const resolved = new Set(steps.filter((s) => s.status === "done" || s.status === "skipped-conditional").map((s) => s.index));
  let first = TOTAL_STEPS;
  for (let i = 1; i <= TOTAL_STEPS; i++) if (!resolved.has(i)) { first = i; break; }
  const verifiedThrough = first - 1;
  const artefactsVerified = steps.filter((s) => s.index <= verifiedThrough && s.status === "done" && s.artefact_sha256).length;
  const recorded = steps.find((s) => s.index === first);
  let skill = recorded ? recorded.skill : null;
  if (!skill && docPath) skill = chainFromDoc(docPath).get(first) || null;
  if (!skill) skill = "unknown";
  return {
    code: 0, first_non_done_step: first, verified_through: verifiedThrough,
    artefacts_ok: artefactsVerified, skill, notes,
    message: `intact - steps 1-${verifiedThrough} verified (${artefactsVerified} artefacts, hashes OK); first non-done step ${first}/${TOTAL_STEPS} (${skill})`,
  };
}

const resumeLine = (id, w, m) =>
  `resume ${id}: steps 1-${w.verified_through} verified (${w.artefacts_ok} artefacts, hashes OK), ` +
  `continuing at step ${w.first_non_done_step}/${TOTAL_STEPS} (${w.skill}). routed_back_count=${m.routed_back_count}`;

// ── commands ─────────────────────────────────────────────────────────────────
function cmdInit(root, id, opts) {
  const taskFile = opts["task-file"];
  const wv = opts["workflow-version"];
  if (!taskFile || !wv) throw new UsageError("init requires --task-file <spec.md> and --workflow-version <v>");
  const path = manifestPathFor(root, id);
  if (existsSync(path) && !opts.force) {
    throw new UsageError(`manifest already exists at ${relUnderRoot(root, path)} - pass --force to re-init (pins are reset)`);
  }
  const taskAbs = resolve(root, taskFile);
  const th = sha256File(taskAbs);
  if (th === null) throw new UsageError(`task file unreadable: ${taskFile}`);
  const t = nowISO(opts);
  const m = {
    manifest_version: MANIFEST_VERSION,
    task_id: id,
    task_file: relUnderRoot(root, taskAbs),
    task_sha256: th,
    workflow_version: wv,
    started_at: t,
    updated_at: t,
    current_step: 1,
    routed_back_count: 0,
    steps: [],
    hitl: { gate: null, requested_at: null },
  };
  const dir = dirname(path);
  if (!existsSync(join(dir, ".gitignore"))) {
    mkdirSync(dir, { recursive: true });
    writeFileSync(join(dir, ".gitignore"), "*\n"); // manifests are gitignored session state
  }
  writeManifest(path, m);
  return { code: 0, manifest: relUnderRoot(root, path), task_sha256: th, workflow_version: wv, message: `initialized ${relUnderRoot(root, path)} (task_sha256 ${th.slice(0, 12)}..., workflow_version ${wv})` };
}

function cmdRecord(root, id, positionals, opts) {
  const [stepRaw, skill, status] = positionals;
  if (!stepRaw || !skill || !status) throw new UsageError("record requires <step> <skill> <status>");
  const index = Number(stepRaw);
  if (!Number.isInteger(index) || index < 1 || index > TOTAL_STEPS) throw new UsageError(`step must be an integer 1..${TOTAL_STEPS} (got '${stepRaw}')`);
  if (!STEP_STATUSES.includes(status)) throw new UsageError(`status must be one of ${STEP_STATUSES.join("|")} (got '${status}')`);
  const { m, path, err } = loadManifest(root, id);
  if (err) throw new UsageError(err);
  let artefactPath = null, artefactSha = null;
  if (opts.artefact) {
    const abs = resolve(root, opts.artefact);
    artefactSha = sha256File(abs);
    if (artefactSha === null) throw new UsageError(`artefact unreadable at record time: ${opts.artefact} (record hashes artefacts when they are written, never later)`);
    artefactPath = relUnderRoot(root, abs);
  }
  const entry = {
    index, skill, status,
    artefact_path: artefactPath,
    artefact_sha256: artefactSha,
    verdict: opts.verdict ?? null,
    completed_at: status === "pending" ? null : nowISO(opts),
  };
  m.steps = m.steps.filter((s) => s.index !== index).concat([entry]).sort((a, b) => a.index - b.index);
  const resolved = new Set(m.steps.filter((s) => s.status === "done" || s.status === "skipped-conditional").map((s) => s.index));
  m.current_step = TOTAL_STEPS;
  for (let i = 1; i <= TOTAL_STEPS; i++) if (!resolved.has(i)) { m.current_step = i; break; }
  if (opts["routed-back"]) m.routed_back_count += 1;
  m.updated_at = nowISO(opts);
  writeManifest(path, m);
  return {
    code: 0, step: index, status, artefact_sha256: artefactSha, routed_back_count: m.routed_back_count,
    message: `recorded step ${index} (${skill}) ${status}${artefactPath ? ` artefact ${artefactPath}` : ""}${opts["routed-back"] ? ` routed_back_count=${m.routed_back_count}` : ""}`,
  };
}

function cmdVerify(root, id, opts) {
  const { m, err } = loadManifest(root, id);
  if (err) throw new UsageError(err);
  const w = walk(root, m, opts);
  if (w.code === 2) throw new UsageError(w.message);
  return { ...w, message: `verify ${id}: ${w.message}` };
}

function cmdResumeLine(root, id, opts) {
  const { m, err } = loadManifest(root, id);
  if (err) throw new UsageError(err);
  const w = walk(root, m, opts);
  if (w.code === 2) throw new UsageError(w.message);
  if (w.code !== 0) return { ...w, message: `resume-line ${id}: ${w.message}` };
  return { ...w, line: resumeLine(id, w, m), message: null };
}

function cmdDelete(root, id) {
  const path = manifestPathFor(root, id);
  if (!existsSync(path)) return { code: 0, deleted: false, message: `no manifest for ${id} - already absent (deletion is always safe)` };
  unlinkSync(path);
  return { code: 0, deleted: true, message: `deleted ${relUnderRoot(root, path)}` };
}

// ── CLI shell ────────────────────────────────────────────────────────────────
const HELP = `ship-manifest.mjs - ship-manifest@1 executor for chief-technology-officer/ship-tasks (TASK-IMP-085)

usage: node ship-manifest.mjs [--json] [--root <repo-root>] <command> ...

commands
  init <task-id> --task-file <spec.md> --workflow-version <v> [--force]
      create docs/tasks/.workflow/<task-id>.ship.json; pins task_sha256 + workflow_version at init
  record <task-id> <step 1..31> <skill> <status> [--artefact <path>] [--verdict <v>] [--routed-back]
      record {index, skill, status, artefact_path, artefact_sha256, verdict, completed_at};
      status: pending|done|failed|skipped-conditional; the artefact is hashed AT RECORD TIME
  verify <task-id> [--workflow-version <v>] [--workflow-doc <path>] [--task-file <spec.md>]
      walk the workflow's staleness order (version -> task hash -> earliest artefact -> intact)
  resume-line <task-id> [--workflow-version <v>] [--workflow-doc <path>]
      verify, then echo exactly:
      resume <task-ID>: steps 1-N verified (K artefacts, hashes OK), continuing at step M/31 (<skill>). routed_back_count=R
  delete <task-id>
      terminal handling on done; idempotent

exit codes
  0  ok / manifest intact (verify names the first non-done step)
  2  usage error, unreadable input, missing manifest, missing artefact at record time
  3  workflow_version mismatch -> needs_human (never a silent mixed-version run)
  4  task_sha256 mismatch -> every step stale; history and routed_back_count retained
  5  artefact hash mismatch -> the earliest stale step is named; redo from there

clock  CYBEROS_NOW (env) or --now <ISO-8601> pins started_at/updated_at/completed_at for
       deterministic runs; unset = wall clock. No other output depends on time.
writes two-phase atomic (.tmp.<nonce> then rename); readers ignore tmp files.
--json prints a stable-stringified result envelope (sorted keys) instead of prose.
`;

function main(argv) {
  const flags = new Set(["json", "force", "routed-back", "help"]);
  const valued = new Set(["root", "task-file", "workflow-version", "workflow-doc", "artefact", "verdict", "now"]);
  const opts = {};
  const positionals = [];
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "-h" || a === "--help") { opts.help = true; continue; }
    if (a.startsWith("--")) {
      const name = a.slice(2);
      if (flags.has(name)) { opts[name] = true; continue; }
      if (valued.has(name)) {
        if (i + 1 >= argv.length) { process.stderr.write(`ship-manifest: --${name} needs a value\n`); return 2; }
        opts[name] = argv[++i]; continue;
      }
      process.stderr.write(`ship-manifest: unknown flag '${a}'\n${HELP}`);
      return 2;
    }
    positionals.push(a);
  }
  if (opts.help) { process.stdout.write(HELP); return 0; }
  const [command, id, ...rest] = positionals;
  const emit = (r) => {
    if (opts.json) {
      const env = { command, task_id: id ?? null, ok: r.code === 0, exit_code: r.code, ...r };
      delete env.code;
      process.stdout.write(stableStringify(env) + "\n");
    } else {
      if (r.line) process.stdout.write(r.line + "\n");
      if (r.message) process.stdout.write(r.message + "\n");
      for (const n of r.notes || []) process.stdout.write(`note: ${n}\n`);
    }
    return r.code;
  };
  try {
    if (!command) throw new UsageError("no command given");
    if (!["init", "record", "verify", "resume-line", "delete"].includes(command)) {
      throw new UsageError(`unknown command '${command}'`);
    }
    if (!id || !ID_RE.test(id)) throw new UsageError(`task-id must match ${ID_RE} (a filename component, never a path)`);
    const root = findRoot(opts.root);
    if (command === "init") return emit(cmdInit(root, id, opts));
    if (command === "record") return emit(cmdRecord(root, id, rest, opts));
    if (command === "verify") return emit(cmdVerify(root, id, opts));
    if (command === "resume-line") return emit(cmdResumeLine(root, id, opts));
    return emit(cmdDelete(root, id));
  } catch (e) {
    if (e instanceof UsageError) {
      if (opts.json) {
        process.stdout.write(stableStringify({ command: command ?? null, task_id: id ?? null, ok: false, exit_code: 2, error: e.message }) + "\n");
      } else {
        process.stderr.write(`ship-manifest: ${e.message}\nusage: node ship-manifest.mjs [--json] [--root <dir>] <init|record|verify|resume-line|delete> <task-id> ... (--help for details)\n`);
      }
      return 2;
    }
    throw e;
  }
}

process.exitCode = main(process.argv.slice(2));
