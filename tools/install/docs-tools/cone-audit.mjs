#!/usr/bin/env node
// cone-audit.mjs — audit a task's ACTUAL writes against its DECLARED cone (TASK-IMP-119).
//
// WHY: batch-select proves two tasks are independent by intersecting their cones
// (new_files ∪ modified_files ∪ service). That proof is only as true as the cones, and NOTHING
// compares a cone to what a task actually wrote. On the 2026-07-17 batch (TASK-IMP-110 +
// TASK-IMP-114) three files escaped BOTH declared cones and nobody noticed — the batch was safe
// by luck, not by proof. This tool closes that loop: it REPORTS every written path outside the
// declared cone at the `implementing -> ready_to_review` flip, where the diff exists and the spec
// is still open for amendment.
//
// It REPORTS, never refuses or flips (spec §1.5). A write discovered mid-implementation is often a
// real finding — the remedy is to amend the cone, which is a spec edit and therefore a human's
// call. An escape is a FINDING for a human, not a workflow failure.
//
// CONTAINMENT + CONE PARSING ARE batch-select's, verbatim (spec §1.2, §1.4). `fm`/`one`/`list`/
// `declares` are copied character-for-character from tools/install/docs-tools/batch-select.mjs so
// the two tools cannot disagree about what a cone contains or how `(none)` is filtered — "two
// tools disagreeing about what a cone contains is worse than neither existing" (§1.2). The
// containment predicate `within` is batch-select's own clash primitive (batch-select.mjs:93),
// used directionally: a path is INSIDE the cone iff it EQUALS a declared entry or is NESTED under
// one. batch-select's clash is the symmetric OR of `within` in both directions; cone-audit takes
// only the "path is at-or-under a declared entry" half — so a write that is a declared file's
// PARENT dir is an escape ("narrower does not cover wider", §3 row 6). test_cone_audit.sh t02
// pins both to a shared table so they provably agree, and greps batch-select's source for the
// `startsWith(y + "/")` operator so a change there turns this red.
//
// GUARD (spec §1.7): the same relUnderRoot guard every repo-reading docs-tool carries
// (task-reconcile / verify-goals / coverage-scope / fm001-migrate): the task's spec is resolved
// by walking docs/tasks under the repo root; a spec whose real path escapes the root (symlink out
// of the corpus), an unreadable spec, an unknown task-id, a non-repo, or an unresolvable base is
// REFUSED and NAMED — never silently skipped, never reported as zero escapes. "Unreadable is not
// clean; empty is not contains-everything."
//
// DETERMINISM (spec §1.6): no wall clock, no randomness, no unordered set iteration in the
// artefact — the cone, the writes, and the escapes are all sorted bytewise before emission, so
// identical repo state + identical args = byte-identical output.
//
// Usage:  node cone-audit.mjs <task-id> [--base <ref|range>] [--repo <root>] [--json]
//   <task-id>   the task whose cone + writes are audited (matched against spec frontmatter `id:`)
//   --base      the diff base. A single ref R diffs `R..HEAD` (the writes since the task entered
//               implementing). A git range (contains "..") is used verbatim — this lets a PAST
//               batch be audited without checking anything out, e.g. `--base <parent>..<tip>`.
//               Omitted: the base is the EARLIEST commit whose subject names <task-id> AND
//               "implementing" (the corpus entry-flip convention, shared with coverage-scope);
//               no such commit and no --base is a refusal, never a guess.
//   --repo      repo root (default: the git toplevel above cwd, else cwd).
//   --json      stable JSON summary instead of the human report.
// Exit:  0  ran — the report was produced (escapes present or not; an escape is NOT a failure).
//        2  usage error / refusal: not a git repo, unknown task-id, spec escapes root or is
//           unreadable, base unresolvable. Never emits "zero escapes" for any of these.
//
// Node stdlib only (docs-tools convention — see batch-select.mjs, coverage-scope.mjs). Read-only:
// git is invoked only via rev-parse / log / diff (no writes); the tool writes nothing at all.

import { readFileSync, readdirSync, existsSync, statSync, realpathSync } from "node:fs";
import { join, resolve, relative, isAbsolute, dirname } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const TASK_ID_RE = /^[A-Za-z0-9._-]+$/; // an id, never a path — rejects "/" and ".." traversal

// ── cone parsing: batch-select.mjs verbatim (spec §1.2, §1.4 — the two must not drift) ──────────
// fm / one / list / declares are copied character-for-character from batch-select.mjs. `list`
// carries batch-select's `(none)` filter (batch-select.mjs `.filter(x => x && x !== "(none)")`,
// the historical line 51/57): a literal placeholder is not a path (§1.4).
const fm = (t) => {
  if (!t.startsWith("---")) return null;
  const e = t.indexOf("\n---", 3); if (e < 0) return null;
  return t.slice(4, e);
};
const one = (f, k) => (f.match(new RegExp(`^${k}:\\s*(.*)$`, "m"))?.[1] ?? "").trim();
const declares = (f, k) => new RegExp(`^${k}:`, "m").test(f);
const list = (f, k) => {
  const inline = one(f, k);
  if (inline.startsWith("[")) return inline.slice(1, -1).split(",").map(s => s.trim()).filter(Boolean);
  const blk = f.match(new RegExp(`^${k}:\\s*\\n((?:  - .*\\n?)+)`, "m"));
  return blk ? blk[1].split("\n").map(l => l.replace(/^\s*-\s*/, "").trim()).filter(x => x && x !== "(none)") : [];
};

// The cone is files UNION service — batch-select.mjs line 80, verbatim. `service` folds in because
// a declared file list is only what the AUTHOR expected to touch; TASK-IMP-104 declared install.sh
// and edited two more files inside its service, neither declared.
export function parseCone(frontmatter) {
  const f = frontmatter;
  return new Set([
    ...list(f, "new_files"),
    ...list(f, "modified_files"),
    ...(one(f, "service") ? [one(f, "service")] : []),
  ]);
}
// Whether the AUTHOR said anything about the cone (batch-select.mjs line 82) — distinct from
// whether the cone is empty. Not needed for the escape math (an empty cone contains nothing, so
// every write escapes — §1.3), but exported so a caller can tell "absent" from "(none)".
export function coneDeclared(frontmatter) {
  const f = frontmatter;
  return declares(f, "new_files") || declares(f, "modified_files") || declares(f, "service");
}

// ── containment: batch-select.mjs:93's clash primitive, used directionally (spec §1.2) ──────────
// batch-select clashes cones a,b when `x === y || x.startsWith(y + "/") || y.startsWith(x + "/")`
// for some x∈a, y∈b. That is `within(x,y) || within(y,x)`. A path is INSIDE a cone iff it is
// at-or-under some declared entry — the "path side" half only. So a write EQUAL to an entry is
// inside (§3 row 4), a write NESTED under an entry is inside (§3 row 5), but a write that is an
// entry's PARENT directory is an ESCAPE — narrower does not cover wider (§3 row 6). The `+ "/"`
// boundary is load-bearing: "ab/c" is NOT under "a" though it shares the prefix "a".
export function within(path, entry) {
  return path === entry || path.startsWith(entry + "/");
}
export function insideCone(path, cone) {
  for (const entry of cone) if (within(path, entry)) return true; // empty cone → nothing inside (§1.3)
  return false;
}

// ── guard predicate (one rule, shared with task-reconcile / verify-goals / coverage-scope) ──────
const relUnderRoot = (root, p) => {
  const rel = relative(root, resolve(root, p));
  return (rel === "" || rel.startsWith("..") || isAbsolute(rel)) ? null : rel;
};

// ── git plumbing (read-only) ────────────────────────────────────────────────────────────────────
function git(root, args) {
  return spawnSync("git", ["-C", root, ...args], { encoding: "utf8" });
}
function isRepo(root) {
  const r = git(root, ["rev-parse", "--is-inside-work-tree"]);
  return !r.error && r.status === 0 && r.stdout.trim() === "true";
}
function gitToplevel(from) {
  const r = git(from, ["rev-parse", "--show-toplevel"]);
  return !r.error && r.status === 0 ? r.stdout.trim() : null;
}

// ── spec resolution: find the task's spec by frontmatter id, under docs/tasks, guarded ──────────
// Returns { frontmatter, rel } or throws Refusal. The walk is sorted (determinism). The guard is
// the relUnderRoot rule every repo-reading docs-tool carries: a spec.md whose REAL path escapes the
// root (symlink out of the corpus, §3 rows 11/12) is never read as authority, and an unreadable
// spec (§3 row 14) is never silently skipped. Escaping / unreadable spec.md files are COLLECTED but
// not read for content; if the target id is not found among the in-corpus, readable specs, and any
// such refused candidate exists, the run is REFUSED and NAMES them — the target may BE one of them,
// and "unreadable/escaping is not clean". Content outside the root is never read.
class Refusal extends Error {}

function findSpec(root, taskId) {
  const tasksDir = join(root, "docs", "tasks");
  if (!existsSync(tasksDir)) throw new Refusal(`no docs/tasks under ${root} — nothing to audit against`);
  const rootReal = realpathSync(root);
  let found = null;
  const refused = []; // {rel, why} for escaping / unreadable spec.md — named on a miss
  const walk = (d) => {
    if (found) return;
    let entries;
    try { entries = readdirSync(d).sort(); } catch { return; }
    for (const e of entries) {
      if (found) return;
      const p = join(d, e);
      let st;
      try { st = statSync(p); } catch { continue; }
      if (!st.isDirectory()) continue;
      const sp = join(p, "spec.md");
      if (existsSync(sp)) {
        const rel = relative(root, sp);
        let real;
        try { real = realpathSync(sp); }
        catch (err) { refused.push({ rel, why: `unresolvable (${err.code || "read error"})` }); continue; }
        if (!(real === rootReal || real.startsWith(rootReal + "/"))) {
          refused.push({ rel, why: `resolves OUTSIDE the repo root via symlink (${real})` }); // never read (§3 row 12)
          continue;
        }
        let text;
        try { text = readFileSync(sp, "utf8"); }
        catch (err) { refused.push({ rel, why: `unreadable (${err.code || "read error"})` }); continue; } // §3 row 14
        const f = fm(text);
        if (f && one(f, "id") === taskId) { found = { frontmatter: f, rel }; return; }
      } else {
        walk(p);
      }
    }
  };
  walk(tasksDir);
  if (found) return found;
  if (refused.length > 0) {
    const named = refused.sort((a, b) => a.rel.localeCompare(b.rel)).map((r) => `${r.rel} (${r.why})`).join("; ");
    throw new Refusal(`no in-corpus readable spec has id '${taskId}', and ${refused.length} spec(s) were REFUSED by the guard, never skipped: ${named}`);
  }
  throw new Refusal(`no task spec with id '${taskId}' found under docs/tasks — refused (an unknown task is not "clean")`);
}

// ── base / diff-range resolution (spec §1.1) ────────────────────────────────────────────────────
// --base: a range (contains "..") is used verbatim; a single ref R becomes `R..HEAD`. No --base:
// the EARLIEST commit whose subject names <task-id> AND "implementing" (the entry-flip convention,
// shared with coverage-scope.mjs) becomes the base R and the range is `R..HEAD`. No base and no
// such commit is a REFUSAL (§3 row 13: base missing → refuse, never zero escapes).
function resolveRange(root, taskId, base) {
  if (base !== undefined) {
    if (base.includes("..")) {
      // explicit range — let the diff itself validate the endpoints (bad range → refusal below)
      return { range: base, provenance: `--base range '${base}'` };
    }
    const r = git(root, ["rev-parse", "--verify", "--quiet", `${base}^{commit}`]);
    if (r.error || r.status !== 0) throw new Refusal(`--base '${base}' does not resolve to a commit — refused, not audited`);
    return { range: `${base}..HEAD`, provenance: `--base '${base}' (${base}..HEAD)` };
  }
  const r = git(root, ["log", "--format=%H %s"]);
  if (r.error || r.status !== 0) throw new Refusal(`cannot read git history to resolve a base — refused (pass --base <ref>)`);
  const matches = [];
  for (const line of r.stdout.split("\n")) {
    if (!line) continue;
    const sp = line.indexOf(" ");
    const sha = sp < 0 ? line : line.slice(0, sp);
    const subject = sp < 0 ? "" : line.slice(sp + 1);
    if (subject.includes(taskId) && /implementing/i.test(subject)) matches.push(sha);
  }
  if (matches.length === 0) {
    throw new Refusal(`no --base given and no commit subject names '${taskId}' + "implementing" (the entry-flip convention) — pass --base <ref>; this tool never guesses a range and never reports zero escapes on a missing base`);
  }
  const earliest = matches[matches.length - 1]; // git log is newest-first
  return { range: `${earliest}..HEAD`, provenance: `subject-scan (earliest entry-flip commit ${earliest.slice(0, 8)})` };
}

// git diff --name-only --no-renames <range>: --no-renames so a rename appears as its OLD (deleted)
// AND NEW (added) paths, both counted as writes and both checked against the cone (§3 row 9).
function actualWrites(root, range) {
  const r = git(root, ["diff", "--name-only", "--no-renames", range]);
  if (r.error) throw new Refusal(`git diff failed to launch — refused`);
  if (r.status !== 0) throw new Refusal(`git diff over '${range}' failed: ${(r.stderr || "").trim() || `exit ${r.status}`} — refused, not audited`);
  return [...new Set(r.stdout.split("\n").filter(Boolean))].sort();
}

// ── audit (pure over resolved inputs) ───────────────────────────────────────────────────────────
export function auditWrites(writes, cone) {
  return writes.filter((w) => !insideCone(w, cone)).sort();
}

// ── CLI ─────────────────────────────────────────────────────────────────────────────────────────
const HELP = `cone-audit.mjs — report a task's writes that escape its declared cone (TASK-IMP-119)

usage: node cone-audit.mjs <task-id> [--base <ref|range>] [--repo <root>] [--json]

  <task-id>   audited task; matched against spec frontmatter id: under docs/tasks
  --base      single ref R -> diff R..HEAD; a range (contains "..") is used verbatim
              (audit a past batch without a checkout); omitted -> earliest commit whose
              subject names <task-id> + "implementing" (entry-flip convention), else refuse
  --repo      repo root (default: git toplevel above cwd, else cwd)
  --json      stable JSON instead of the human report

REPORTS, never refuses or flips: an escape is a finding for a human (amend the cone, a spec edit).
containment + (none) filter are batch-select's, verbatim; deterministic (no clock); read-only.

exit  0  ran — report produced (escapes or not; an escape is not a failure)
      2  usage / refusal: not a repo, unknown task-id, spec escapes root or unreadable, base
         unresolvable (never emits "zero escapes" for any of these)
`;

function main(argv) {
  let base, repo, json = false;
  const positionals = [];
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "-h" || a === "--help") { process.stdout.write(HELP); return 0; }
    else if (a === "--json") json = true;
    else if (a === "--base") { base = argv[++i]; if (base === undefined) { process.stderr.write("cone-audit: --base needs a value\n"); return 2; } }
    else if (a === "--repo") { repo = argv[++i]; if (repo === undefined) { process.stderr.write("cone-audit: --repo needs a value\n"); return 2; } }
    else if (a.startsWith("--")) { process.stderr.write(`cone-audit: unknown flag '${a}'\n${HELP}`); return 2; }
    else positionals.push(a);
  }
  const [taskId, ...extra] = positionals;
  try {
    if (!taskId) throw new Refusal("no <task-id> given");
    if (extra.length > 0) throw new Refusal(`unexpected positional argument '${extra[0]}'`);
    if (!TASK_ID_RE.test(taskId)) throw new Refusal(`task-id '${taskId}' is not an id (must match ${TASK_ID_RE}) — refused, not resolved to a path`);

    const root = resolve(repo || gitToplevel(process.cwd()) || process.cwd());
    if (!isRepo(root)) throw new Refusal(`${root} is not a git repository — refused (the diff needs git; a missing repo is never "zero escapes")`);

    const { frontmatter, rel } = findSpec(root, taskId);
    const cone = parseCone(frontmatter);
    const { range, provenance } = resolveRange(root, taskId, base);
    const writes = actualWrites(root, range);
    const escapes = auditWrites(writes, cone);
    const coneSorted = [...cone].sort();

    if (json) {
      // stable shape, sorted arrays, no clock → byte-identical for identical inputs (§1.6)
      const out = {
        tool: "cone-audit@1",
        task: taskId,
        spec: rel,
        base: range,
        base_provenance: provenance,
        cone: coneSorted,
        cone_declared: coneDeclared(frontmatter),
        writes,
        escapes,
        escape_count: escapes.length,
      };
      process.stdout.write(JSON.stringify(out, null, 2) + "\n");
    } else {
      process.stdout.write(`cone-audit@1  ${taskId}  base=${range} (${provenance})\n`);
      process.stdout.write(`  cone: ${coneSorted.length} entr${coneSorted.length === 1 ? "y" : "ies"}${coneDeclared(frontmatter) ? "" : " (UNDECLARED — every write escapes)"}\n`);
      process.stdout.write(`  writes: ${writes.length}   escapes: ${escapes.length}\n`);
      for (const e of escapes) process.stdout.write(`  ESCAPE  ${e}\n`);
      if (escapes.length === 0) process.stdout.write(`  (no escapes — every write is inside the declared cone)\n`);
    }
    return 0; // ran; an escape is a finding, not a failure (§1.5)
  } catch (err) {
    if (err instanceof Refusal) {
      process.stderr.write(`cone-audit: ${err.message}\n`);
      return 2;
    }
    throw err;
  }
}

// Run only when invoked directly; importing the module (test_cone_audit.sh t02) must not execute
// main — the shared-table agreement proof imports `within`/`insideCone` and needs a clean module.
const invokedDirectly = (() => {
  try { return process.argv[1] && realpathSync(process.argv[1]) === realpathSync(fileURLToPath(import.meta.url)); }
  catch { return false; }
})();
if (invokedDirectly) process.exitCode = main(process.argv.slice(2));
