#!/usr/bin/env node
// backlog-mutate.mjs — byte-discipline executor for backlog-state-update@2 writes (TASK-IMP-085).
//
// Executes the two sanctioned BACKLOG.md mutations from
// modules/skill/backlog-state-update-author/SKILL.md §2-§3 — flip one status cell,
// insert one row — and NOTHING else: this tool never moves, reorders, or deletes rows,
// and never edits a line outside the declared mutation (whole-file discipline: one row,
// at most one section header, at most one Totals line - TASK-IMP-116). Grammar authority stays with
// regen_backlog() in scripts/migrate_task_layout.py / migrate_improvement_to_task.py;
// this tool encodes it, never redefines it.
//
// Usage:  node backlog-mutate.mjs [--json] [--root <repo-root>] <command> ...
//
//   flip <task-id> <from> <to> [--backlog <path>] [--old-line <text>]
//        [--verdict-by <actor> --verdict-evidence <path>]
//       Locate the row by task STEM (the `<task-id>-<slug>` token), verify the status
//       cell equals <from> AND — when --old-line carries the recorded pre-image — the
//       full old line byte-for-byte (line terminator excluded), then rewrite EXACTLY
//       that one cell; every other byte of the line (title, tags, comments, CR) is
//       preserved. BEFORE writing, reads the task's spec.md frontmatter `status` and
//       refuses (exit 6) unless it ALREADY equals <to> — the frontmatter is the record
//       of truth and the index may only ever catch up to it, never lead (TASK-IMP-120;
//       an unfindable, unreadable, or ambiguous truth also refuses). Refuses with exit 6
//       on a missing row, on 2+ matching rows (corrupted backlog — both lines are named,
//       never a guess), or on any drift. When the
//       containing `## section` header carries `(N status, ...)` counts, the header is
//       rewritten from a FULL RETALLY of the section's rows after the flip (zero-count
//       statuses omitted, statuses in lifecycle order) — an inherited wrong count is
//       corrected, never propagated (TASK-IMP-092); a header without parseable counts
//       is left untouched.
//       HUMAN-ACCEPTANCE GATE (TASK-CUO-303): the two transitions doctrine reserves for
//       a recorded human verdict — `reviewing -> ready_to_test` and `testing -> done`
//       (STATUS-REFERENCE §1.4) — additionally require --verdict-by (non-empty actor)
//       and --verdict-evidence (an existing, non-empty regular file), refusing with
//       exit 8 AFTER every exit-6 refusal above; every other transition ignores the
//       flags. On a gated flip, ONE status_overridden row is appended via the sibling
//       memory-append.mjs BEFORE the index moves whenever a BRAIN store resolves
//       (CYBEROS_STORE override, else <root>/.cyberos/memory/store); a present store
//       that cannot take the row fails the flip (exit 9, audit-before-action), and no
//       store at all means the evidence file is the record (noted on stderr).
//   insert <task-id> <stem> <title> <status> [--backlog <path>] [--section <name>] [--class product|improvement]
//       Uniqueness gate first: NO row for <task-id> (or <stem>) may pre-exist anywhere
//       in the file — violation is exit 7 naming the line. The row is rendered in the
//       regenerator-identical grammar `- [<status>] <stem> - <title>` (+ ` (improvement)`
//       for --class improvement) and placed in stem-ascending order inside the target
//       section's contiguous row block (bytewise on the stem token only — titles never
//       affect placement). A `- (nothing remaining)` placeholder is replaced by the new
//       first row. A counted header is rewritten from a full retally of the section's
//       rows after the insert (same rule as flip - TASK-IMP-092).
//       --section names the target `## <name>` section; without it the section is
//       auto-detected as the unique section already holding rows with the same
//       `TASK-<MODULE>-` prefix (zero or many candidates = exit 2, pass --section).
//       This tool never creates sections; regenerate the backlog for that.
//   next-id <module>
//       Print the module's next task stem and exit 0 (TASK-IMP-105). Scans the
//       TASK-<PREFIX>-<NNN> task FOLDERS in docs/tasks/<module>/ and returns the
//       HIGHEST plus one — gaps ignored (001,002,004 -> 005), never the lowest free
//       number. An empty or not-yet-existent module returns its first stem (...-001);
//       the prefix comes from the existing folders (improvement -> IMP) so no map is
//       maintained. Reads the same folder corpus the insert gate's uniqueness check
//       reads, so an id it returns cannot be rejected by that gate for non-uniqueness
//       (spec §1.3). Strictly read-only: it never writes, moves, or flips anything.
//
// Exit codes:
//   0  ok
//   2  usage error, unreadable backlog, section not found / ambiguous, no row block
//   6  flip refusal: missing row, duplicate rows, or drifted pre-image (status cell or
//      --old-line bytes) — the optimistic-concurrency check from SKILL.md §3
//   7  insert refusal: a row for the id already exists (uniqueness pre-image violated)
//   8  flip refusal: the transition is a human-acceptance gate (reviewing->ready_to_test,
//      testing->done; STATUS-REFERENCE §1.4) and no recorded verdict accompanied it —
//      --verdict-by missing/empty, or --verdict-evidence missing/empty/not a regular
//      file (TASK-CUO-303). Nothing written.
//   9  flip failure: the status_overridden verdict row could not be appended to a
//      PRESENT BRAIN store (audit-before-action: the index does not move without its
//      audit row). Nothing written.
//
// Byte discipline: the file is split on '\n' and rejoined on '\n' only — CRLF endings,
// a missing final newline, unicode titles, everything outside the mutated line(s)
// round-trips byte-identically (t07 proves it with a whole-file diff). Inserted rows
// take the line ending of their section. Writes are two-phase atomic (`.tmp.<nonce>`
// then rename). No clock, no randomness in output: identical input + identical args =
// byte-identical result file and stdout — for verdict-gated flips "input" includes the
// resolved BRAIN store's state, and the appended row inherits memory-append.mjs's
// CYBEROS_NOW clock convention (pin it for deterministic runs). Node stdlib only
// (docs-tools convention).

import {
  readFileSync, writeFileSync, renameSync, existsSync, mkdirSync,
  openSync, fsyncSync, closeSync, readdirSync, statSync,
} from "node:fs";
import { randomBytes } from "node:crypto";
import { join, resolve, dirname, relative, isAbsolute } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

// STATUS-REFERENCE.md §1 enum; the first ten are regen_backlog()'s STATUS_ORDER —
// header counts render in this order.
const STATUS_ORDER = [
  "draft", "ready_to_implement", "implementing", "ready_to_review", "reviewing",
  "ready_to_test", "testing", "done", "on_hold", "closed", "cannot_reproduce", "duplicate",
];
const PLACEHOLDER = "- (nothing remaining)";
const ID_RE = /^[A-Za-z0-9._-]+$/;

class UsageError extends Error {}
// `fields` (optional) rides into the --json refusal envelope — the verdict gate uses it
// so ship-manifest consumers can record the verdict fields on refusals too (TASK-CUO-303
// edge case: refusal AND success both carry them).
class Refusal extends Error { constructor(code, msg, fields) { super(msg); this.code = code; if (fields) this.fields = fields; } }

// ── deterministic serialization + atomic write (shared idiom) ────────────────
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

function atomicWrite(path, text) {
  mkdirSync(dirname(path), { recursive: true });
  const tmp = `${path}.tmp.${randomBytes(6).toString("hex")}`;
  writeFileSync(tmp, text);
  const fd = openSync(tmp, "r");
  try { fsyncSync(fd); } finally { closeSync(fd); }
  renameSync(tmp, path);
}

// ── line model (CR-preserving) ───────────────────────────────────────────────
const stripCR = (l) => (l.endsWith("\r") ? l.slice(0, -1) : l);
const crOf = (l) => (l.endsWith("\r") ? "\r" : "");

// Row grammar: `- [<status>] <stem> - <title>` (regen_backlog() byte-authority).
function parseRow(raw) {
  const m = /^- \[([a-z_]+)\] (\S+) - (.*)$/.exec(stripCR(raw));
  return m ? { status: m[1], stem: m[2], rest: m[3] } : null;
}
const stemMatchesId = (stem, id) => stem === id || stem.startsWith(id + "-");

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

// CONFINE (TASK-IMP-105): the same relUnderRoot rule every repo-reading docs-tool carries
// (task-reconcile, coverage-scope, verify-goals) — a value that resolves to the root itself,
// escapes it with `..`, or is absolute is refused, never followed out of the corpus. Returns
// the confined POSIX-relative path, or null when the argument walks out.
const relUnderRoot = (root, p) => {
  const abs = isAbsolute(p) ? p : resolve(root, p);
  const rel = relative(root, abs).split("\\").join("/");
  return (rel === "" || rel.startsWith("..") || isAbsolute(rel)) ? null : rel;
};

function readBacklog(root, opts) {
  const path = opts.backlog ? resolve(root, opts.backlog) : join(root, "docs", "tasks", "BACKLOG.md");
  let text;
  try { text = readFileSync(path, "utf8"); }
  catch { throw new UsageError(`backlog unreadable: ${opts.backlog || "docs/tasks/BACKLOG.md"}`); }
  return { path, given: opts.backlog || "docs/tasks/BACKLOG.md", lines: text.split("\n") };
}

// ── header counts ────────────────────────────────────────────────────────────
// A counts header is `## <name>  (<N status>(, <N status>)*)` exactly (regen grammar).
// Anything else — bare `## name`, prose parens — is not a counts header: no edit.
function parseCountsHeader(raw) {
  const s = stripCR(raw);
  const m = /^(## .*?)  \(([^()]+)\)$/.exec(s);
  if (!m) return null;
  const counts = new Map();
  for (const part of m[2].split(", ")) {
    const pm = /^(\d+) ([a-z_]+)$/.exec(part);
    if (!pm || !STATUS_ORDER.includes(pm[2]) || counts.has(pm[2])) return null;
    counts.set(pm[2], Number(pm[1]));
  }
  return { prefix: m[1], counts };
}

// Rewrites the header at lines[h] from a FULL RETALLY of its section's rows — every
// `- [<status>] <stem> - <title>` row between the header and the next `## ` header (in
// the regenerated layout that is the section's contiguous row block; the placeholder
// parses as no row). Replaces the incremental +1/-1 adjust (TASK-IMP-092): incremental
// adjustment faithfully preserves an inherited lie forever (the 086 incident's 34 vs
// true 20), while the retally makes every mutation emit the section's truth. Returns
// the rewritten line, or null when the header carries no parseable counts (bare
// headers stay untouched) or the section has no rows to tally (never `()`, never a
// zero entry). Rendering keeps the file's own convention: two spaces before the paren,
// zero-count statuses omitted, statuses in lifecycle (STATUS_ORDER) order; rows whose
// status token is outside the enum are not counted, matching regen_backlog().
// The file-top `Totals:` line, retallied from EVERY parseable row (TASK-IMP-116). TASK-IMP-092
// replaced incremental header adjustment with a retally because "incremental adjustment faithfully
// preserves an inherited lie forever (the 086 incident's 34 vs true 20)" - and stopped at section
// headers, leaving the file's most-read number free to grow the same lie one line higher. It did:
// a 2026-07-17 review found `Totals: 336 draft, 4 ready_to_implement, 176 done` over a file whose
// improvement section alone read 67/9/39. The line had no maintainer - regen_backlog owns it and
// cannot run (it reads docs/improvement/memory/backlog.yaml, retired 2026-07-08).
//
// The declared mutation is now THREE lines: the row, its section header, and this. Capped there
// (§1.6): a footprint that grows on convenience is not a footprint. Returns { line, text } or
// { line: -1 } when the file has no Totals line (legal - it is never given one) or no countable
// rows. Counting rule is retallyHeader's, exactly: out-of-enum statuses do not count.
function retallyTotals(lines) {
  const t = lines.findIndex(l => /^Totals:\s/.test(stripCR(l)));
  if (t < 0) return { line: -1, text: null };
  const tally = new Map();
  for (const l of lines) {
    const row = parseRow(l);
    if (row && STATUS_ORDER.includes(row.status)) tally.set(row.status, (tally.get(row.status) || 0) + 1);
  }
  if (tally.size === 0) return { line: -1, text: null };
  const rendered = STATUS_ORDER.filter(st => tally.get(st)).map(st => `${tally.get(st)} ${st}`).join(", ");
  return { line: t, text: `Totals: ${rendered}` + crOf(lines[t]) };
}

function retallyHeader(lines, h) {
  const parsed = parseCountsHeader(lines[h]);
  if (!parsed) return null;
  const tally = new Map();
  for (let i = h + 1; i < lines.length; i++) {
    if (stripCR(lines[i]).startsWith("## ")) break;
    const row = parseRow(lines[i]);
    if (row && STATUS_ORDER.includes(row.status)) tally.set(row.status, (tally.get(row.status) || 0) + 1);
  }
  if (tally.size === 0) return null;
  const rendered = STATUS_ORDER.filter((s) => (tally.get(s) || 0) > 0)
    .map((s) => `${tally.get(s)} ${s}`).join(", ");
  return `${parsed.prefix}  (${rendered})` + crOf(lines[h]);
}

const nearestHeaderAbove = (lines, idx) => {
  for (let i = idx - 1; i >= 0; i--) if (stripCR(lines[i]).startsWith("## ")) return i;
  return -1;
};

// ── flip truth guard (TASK-IMP-120): the index catches up to the truth, never leads ──────────
// STATUS-REFERENCE.md §1: the task's spec.md frontmatter `status` IS the record of truth and
// BACKLOG.md is only its index. So `flip` reads the truth BEFORE it writes the index and refuses
// unless the frontmatter already carries the target - a caller who moves the index first (the
// TASK-IMP-116 divergence, where two index flips ran while the frontmatter still said `reviewing`)
// gets a refusal, not a silent disagreement. The spec is resolved the SAME way the rest of the
// toolchain resolves a task-id (task-reconcile.mjs findTask): scan docs/tasks/<module>/<dir>/spec.md
// for dir === id or dir startsWith `${id}-`. One resolver, reused - not a second one invented here.
// This whole path is FLIP-only: insert never calls it (spec §1.4).
function resolveSpecPaths(root, id) {
  const base = join(root, "docs", "tasks");
  const hits = [];
  if (!existsSync(base)) return { base, hits };
  let mods;
  try { mods = readdirSync(base); } catch { return { base, hits }; }
  for (const mod of mods) {
    const md = join(base, mod);
    let st; try { st = statSync(md); } catch { continue; }
    if (!st.isDirectory()) continue;
    let entries; try { entries = readdirSync(md); } catch { continue; }
    for (const d of entries) {
      if (d === id || d.startsWith(id + "-")) {
        const spec = join(md, d, "spec.md");
        if (existsSync(spec)) hits.push(spec);
      }
    }
  }
  return { base, hits };
}

// Reads the SINGLE frontmatter `status:` value from a spec. Returns { status } or { error }: an
// absent status, an unterminated fence, or two `status:` lines is an error - an unreadable or
// ambiguous truth is not a matching truth (spec §1.3, edge rows 1/6). A trailing `# comment`
// (FM-001, live across the corpus) is stripped, and surrounding whitespace/quotes trimmed
// (edge rows 5/7), before the value is returned.
function frontmatterStatus(text) {
  const lines = text.split("\n");
  if (stripCR(lines[0] ?? "").trim() !== "---") return { error: "spec has no frontmatter fence" };
  let end = -1;
  for (let i = 1; i < lines.length; i++) { if (stripCR(lines[i]).trim() === "---") { end = i; break; } }
  if (end < 0) return { error: "spec frontmatter fence is unterminated" };
  const found = [];
  for (let i = 1; i < end; i++) {
    const m = /^status:[ \t]*(.*)$/.exec(stripCR(lines[i]));
    if (m) found.push(m[1]);
  }
  if (found.length === 0) return { error: "spec frontmatter carries no `status:` field" };
  if (found.length > 1) return { error: `spec frontmatter carries ${found.length} \`status:\` lines (ambiguous truth)` };
  let v = found[0];
  const h = v.indexOf("#");
  if (h >= 0) v = v.slice(0, h);
  v = v.trim().replace(/^["']|["']$/g, "").trim();
  if (v === "") return { error: "spec frontmatter `status:` is empty" };
  return { status: v };
}

// ── verdict gate (TASK-CUO-303): the two human-acceptance transitions ─────────
// STATUS-REFERENCE §1.4 reserves `reviewing -> ready_to_test` (review acceptance) and
// `testing -> done` (final acceptance) for a RECORDED human verdict. This gate makes the
// doctrine mechanical on the single documented backlog write path (TASK-CUO-205): those
// two flips refuse with exit 8 unless --verdict-by (non-empty actor) and
// --verdict-evidence (an existing, non-empty regular file the human produced — resolved
// against --root when relative, same as --backlog) are both supplied. EXACTLY the two
// forward gate transitions are locked: operator superset overrides (done ->
// ready_to_review re-audit, ready_to_review -> ready_to_test skip-review, route-backs,
// off-ramps) stay flag-free, and every other transition ignores the flags entirely.
// The gate evaluates AFTER every existing refusal (missing/duplicate row, pre-image
// drift, truth-precedes-index — all exit 6), so code 8 means exactly one thing: the
// transition was otherwise legal but no verdict was recorded (spec §1.2). Known
// residual, accepted in the spec: an agent editing spec.md frontmatter directly and
// regenerating the backlog bypasses any tool gate; the 1.5.0 state engine closes that.
const GATE_TRANSITIONS = new Set(["reviewing->ready_to_test", "testing->done"]);
const isGateTransition = (from, to) => GATE_TRANSITIONS.has(`${from}->${to}`);

// BRAIN store resolution for the verdict audit row (spec §1.4), mirroring the memory
// protocol's order (memory AGENTS.md §0.4): the explicit CYBEROS_STORE override first,
// else the repo-anchored .cyberos/memory/store under the SAME root whose backlog is
// being flipped. An explicit override counts as "present" by fiat (the appender
// bootstraps it); without one, only an existing directory counts — no store, no row,
// the evidence file is the record.
function resolveBrainStore(root) {
  const env = process.env.CYBEROS_STORE;
  if (env !== undefined && env.trim() !== "") return resolve(env);
  const dflt = join(root, ".cyberos", "memory", "store");
  try { if (statSync(dflt).isDirectory()) return dflt; } catch { /* absent/unreadable = no store */ }
  return null;
}

// Appends the ONE status_overridden row via the sibling memory-append.mjs — the row
// rides the appender's existing §4.2 lease + §4.1 two-phase write discipline. This is a
// direct child-process invocation of the node binary with a fixed argv (spawnSync
// without shell), NOT a shell-out: no string is interpreted, and the evidence path is
// never executed or parsed — it travels only as payload data. Any failure on a PRESENT
// store fails the flip with exit 9 (audit-before-action: the index does not move
// without its audit row). Clock: the row inherits the appender's CYBEROS_NOW
// convention, so pinned-clock runs stay deterministic.
function appendVerdictRow(root, store, payload, jsonFields) {
  const appender = join(dirname(fileURLToPath(import.meta.url)), "memory-append.mjs");
  const failMsg = (detail) =>
    `flip ${payload.task_id}: the status_overridden verdict row could not be appended to the BRAIN store at ${store} - ${detail}. ` +
    `Audit-before-action (STATUS-REFERENCE §1.4, TASK-CUO-303): the index does not move without its audit row. Refusing; nothing written.`;
  if (!existsSync(appender)) throw new Refusal(9, failMsg(`memory-append.mjs not found next to backlog-mutate.mjs (${appender})`), jsonFields);
  const res = spawnSync(process.execPath, [appender, "--json", "append", store, "status_overridden", "-"], {
    input: JSON.stringify(payload), encoding: "utf8",
  });
  if (res.error) throw new Refusal(9, failMsg(`appender could not be spawned (${res.error.message})`), jsonFields);
  if (res.stderr) process.stderr.write(res.stderr); // pass the appender's notes through, already prefixed
  if (res.status !== 0) {
    let detail = "";
    try { detail = JSON.parse(res.stdout).error || ""; } catch { /* non-JSON child output — fall back below */ }
    if (!detail) {
      // A crashed child prints a stack, not a --json envelope: surface its Error line
      // (e.g. "Error: EACCES: permission denied"), not an arbitrary stack frame.
      const lines = (res.stderr || "").split("\n").map((s) => s.trim()).filter(Boolean);
      detail = lines.find((l) => /error/i.test(l)) || lines[lines.length - 1] || `appender exit ${res.status}`;
    }
    throw new Refusal(9, failMsg(detail), jsonFields);
  }
  let out = {};
  try { out = JSON.parse(res.stdout); } catch { /* tolerated: row landed (exit 0); seq/chain just stay null */ }
  return { seq: out.seq ?? null, chain: out.chain ?? null, store };
}

// ── flip ─────────────────────────────────────────────────────────────────────
function cmdFlip(root, positionals, opts) {
  const [id, from, to] = positionals;
  if (!id || !from || !to) throw new UsageError("flip requires <task-id> <from> <to>");
  if (!ID_RE.test(id)) throw new UsageError(`task-id must match ${ID_RE}`);
  for (const s of [from, to]) if (!STATUS_ORDER.includes(s)) throw new UsageError(`status must be one of ${STATUS_ORDER.join("|")} (got '${s}')`);
  const { path, given, lines } = readBacklog(root, opts);

  const hits = [];
  for (let i = 0; i < lines.length; i++) {
    const row = parseRow(lines[i]);
    if (row && stemMatchesId(row.stem, id)) hits.push({ i, row });
  }
  if (hits.length === 0) throw new Refusal(6, `flip ${id}: no row for '${id}' in ${given} - refusing (missing row)`);
  if (hits.length > 1) {
    throw new Refusal(6, `flip ${id}: ${hits.length} rows match '${id}' (lines ${hits.map((h) => h.i + 1).join(" and ")}) - corrupted backlog, refusing to guess`);
  }
  const { i, row } = hits[0];
  if (row.status !== from) {
    throw new Refusal(6, `flip ${id}: status cell at line ${i + 1} is '[${row.status}]', expected '[${from}]' - pre-image drifted, refusing (re-read the backlog and re-enter the queue)`);
  }
  if (opts["old-line"] !== undefined && opts["old-line"] !== stripCR(lines[i])) {
    throw new Refusal(6, `flip ${id}: full old line at ${i + 1} differs byte-for-byte from the recorded pre-image - refusing (optimistic concurrency, SKILL.md §3)`);
  }

  // ── TASK-IMP-120: the frontmatter (truth) MUST already carry <to> before the index moves ──
  // Read AFTER the pre-image checks (missing/duplicate/drift keep their own refusals) but BEFORE
  // the write: the guard gates exactly the flip that would otherwise succeed - the one that would
  // move the index. Every refusal here is exit 6, the pre-image refusal code, and NEVER writes the
  // file. When the truth already equals <to>, the guard is a no-op and the flip proceeds exactly as
  // before - same footprint, same retally (spec §1.2).
  const spec = resolveSpecPaths(root, id);
  if (spec.hits.length === 0) {
    throw new Refusal(6, `flip ${id}: no spec.md found for '${id}' under ${relative(root, spec.base) || "docs/tasks"}/*/ - an unreadable truth is not agreement, refusing (write the frontmatter first)`);
  }
  if (spec.hits.length > 1) {
    throw new Refusal(6, `flip ${id}: ${spec.hits.length} specs match '${id}' (${spec.hits.map((p) => relative(root, p)).join(", ")}) - ambiguous truth, refusing to guess`);
  }
  let specText;
  try { specText = readFileSync(spec.hits[0], "utf8"); }
  catch { throw new Refusal(6, `flip ${id}: spec ${relative(root, spec.hits[0])} cannot be read - an unreadable truth is not agreement, refusing`); }
  const fm = frontmatterStatus(specText);
  if (fm.error) {
    throw new Refusal(6, `flip ${id}: ${relative(root, spec.hits[0])} - ${fm.error} - refusing (the truth must carry '${to}' before the index catches up)`);
  }
  if (fm.status !== to) {
    throw new Refusal(6, `flip ${id}: spec frontmatter status is '${fm.status}', but the flip targets '${to}' - truth precedes index (TASK-IMP-120): write '${to}' into ${relative(root, spec.hits[0])} first, then re-run. Refusing.`);
  }

  // ── TASK-CUO-303: the two human-acceptance transitions require a recorded verdict ──
  // Evaluated LAST among the refusals (spec §1.2): everything above kept its exit 6, so
  // exit 8 means exactly "otherwise legal, but no verdict was recorded". Non-gate
  // transitions skip this whole block — flags ignored, behavior byte-identical to today.
  let verdictInfo = null;
  if (isGateTransition(from, to)) {
    const jsonFields = { verdict_by: opts["verdict-by"] ?? null, verdict_evidence: opts["verdict-evidence"] ?? null };
    const by = opts["verdict-by"];
    const ev = opts["verdict-evidence"];
    const problems = [];
    if (by === undefined) problems.push("--verdict-by is missing");
    else if (by.trim() === "") problems.push("--verdict-by is empty (a verdict needs an identifiable actor)");
    if (ev === undefined) problems.push("--verdict-evidence is missing");
    else {
      // Exists + regular file + non-empty, nothing else (spec §1.1): a directory or an
      // unreadable path is treated as "does not exist" (edge case); content quality is
      // the reviewer's judgment, not the tool's. The path is stat'ed only — never
      // opened, executed, or parsed.
      let st = null;
      try { st = statSync(resolve(root, ev)); } catch { /* missing/unreadable = does not exist */ }
      if (st === null) problems.push(`--verdict-evidence '${ev}' does not exist (an unreadable path counts as missing)`);
      else if (!st.isFile()) problems.push(`--verdict-evidence '${ev}' is not a regular file (a directory counts as missing)`);
      else if (st.size === 0) problems.push(`--verdict-evidence '${ev}' is empty - the evidence must preexist the flip with content`);
    }
    if (problems.length > 0) {
      throw new Refusal(8,
        `flip ${id}: '${from} -> ${to}' is a human-acceptance gate (STATUS-REFERENCE §1.4): a recorded human verdict must accompany it - ${problems.join("; ")}. ` +
        `Pass --verdict-by <actor> and --verdict-evidence <path to the review/acceptance note the human produced>. Refusing; nothing written.`,
        jsonFields);
    }
    // Audit-before-action (spec §1.4): the row lands BEFORE the index moves. A present
    // store that cannot take the row fails the flip (exit 9); no store at all is legal —
    // the evidence file is the record, said on stderr.
    const store = resolveBrainStore(root);
    if (store === null) {
      process.stderr.write(`backlog-mutate: note: no BRAIN store resolvable (no CYBEROS_STORE override, no .cyberos/memory/store under the root) - the verdict evidence file is the record; no status_overridden row appended (TASK-CUO-303)\n`);
      verdictInfo = { verdict_by: by, verdict_evidence: ev, audit_row: null };
    } else {
      const row = appendVerdictRow(root, store, { actor: by, task_id: id, prior_status: from, new_status: to, reason: ev }, jsonFields);
      verdictInfo = { verdict_by: by, verdict_evidence: ev, audit_row: { seq: row.seq, chain: row.chain, store: row.store } };
    }
  }

  const cellPrefix = `- [${from}]`;
  const oldLine = lines[i];
  const newLine = `- [${to}]` + oldLine.slice(cellPrefix.length);
  lines[i] = newLine;

  let headerInfo = { header_line: null, old_header: null, new_header: null };
  const h = nearestHeaderAbove(lines, i);
  if (h >= 0) {
    const rewritten = retallyHeader(lines, h);
    if (rewritten !== null && rewritten !== lines[h]) {
      headerInfo = { header_line: h + 1, old_header: stripCR(lines[h]), new_header: stripCR(rewritten) };
      lines[h] = rewritten;
    }
  }
  const tot = retallyTotals(lines);            // §1.2
  let totalsInfo = {};
  if (tot.line >= 0 && tot.text !== lines[tot.line]) { totalsInfo = { totals_line: tot.line + 1 }; lines[tot.line] = tot.text; }
  atomicWrite(path, lines.join("\n"));
  // Gate flips carry the verdict fields (and the appended row's coordinates) in both the
  // prose message and the --json envelope; non-gate flips emit exactly what they always
  // did — verdictInfo is null there, so nothing spreads and nothing is appended.
  const verdictNote = verdictInfo === null ? ""
    : verdictInfo.audit_row ? `; status_overridden row seq ${verdictInfo.audit_row.seq} appended (verdict by ${verdictInfo.verdict_by})`
    : `; no BRAIN store - the verdict evidence file is the record`;
  return {
    code: 0, backlog: given, line: i + 1, old_line: stripCR(oldLine), new_line: stripCR(newLine), ...headerInfo, ...totalsInfo,
    ...(verdictInfo ?? {}),
    message: `flip ${id}: [${from}] -> [${to}] at line ${i + 1}${headerInfo.header_line ? `; header retallied at line ${headerInfo.header_line}` : ""}${totalsInfo.totals_line ? `; Totals retallied at line ${totalsInfo.totals_line}` : ""}${verdictNote}`,
  };
}

// ── insert ───────────────────────────────────────────────────────────────────
function sectionBlocks(lines) {
  const sections = [];
  for (let i = 0; i < lines.length; i++) {
    const s = stripCR(lines[i]);
    if (!s.startsWith("## ")) continue;
    const name = (/^## (.*?)(?:  \(.*\))?$/.exec(s))[1];
    sections.push({ header: i, name, end: lines.length });
    if (sections.length > 1) sections[sections.length - 2].end = i;
  }
  return sections;
}

function blockOf(lines, section) {
  // The contiguous `- ` run inside the section; the blank after the header stays outside.
  let start = -1;
  for (let i = section.header + 1; i < section.end; i++) {
    if (stripCR(lines[i]).startsWith("- ")) { start = i; break; }
  }
  if (start === -1) return null;
  let end = start;
  while (end + 1 < section.end && stripCR(lines[end + 1]).startsWith("- ")) end++;
  return { start, end };
}

function cmdInsert(root, positionals, opts) {
  const [id, stem, title, status] = positionals;
  if (!id || !stem || !title || !status) throw new UsageError("insert requires <task-id> <stem> <title> <status>");
  if (!ID_RE.test(id)) throw new UsageError(`task-id must match ${ID_RE}`);
  if (!STATUS_ORDER.includes(status)) throw new UsageError(`status must be one of ${STATUS_ORDER.join("|")} (got '${status}')`);
  if (!stemMatchesId(stem, id)) throw new UsageError(`stem '${stem}' does not extend task-id '${id}' (expected '${id}' or '${id}-<slug>')`);
  if (!/^\S+$/.test(stem)) throw new UsageError("stem must be a single whitespace-free token");
  if (/[\r\n]/.test(title)) throw new UsageError("title must not contain newline bytes (row-injection guard: one mutation is one row)");
  const cls = opts.class || "product";
  if (cls !== "product" && cls !== "improvement") throw new UsageError(`--class must be product|improvement (got '${cls}')`);
  const { path, given, lines } = readBacklog(root, opts);

  // Uniqueness pre-image: no row for the id (or the exact stem) anywhere in the file.
  for (let i = 0; i < lines.length; i++) {
    const row = parseRow(lines[i]);
    if (row && (stemMatchesId(row.stem, id) || row.stem === stem)) {
      throw new Refusal(7, `insert ${id}: row already present at line ${i + 1} ('${row.stem}') - uniqueness pre-image violated, refusing`);
    }
  }

  // Target section: --section by exact name, else the unique section already holding
  // rows with the same TASK-<MODULE>- prefix.
  const sections = sectionBlocks(lines);
  let target = null;
  if (opts.section !== undefined) {
    target = sections.find((s) => s.name === opts.section) || null;
    if (!target) throw new UsageError(`section '## ${opts.section}' not found in ${given} - this tool never creates sections (regenerate the backlog for that)`);
  } else {
    const pm = /^([A-Za-z]+-[A-Za-z0-9]+-)/.exec(id + "-");
    const prefix = pm ? pm[1] : null;
    const candidates = prefix === null ? [] : sections.filter((s) => {
      const b = blockOf(lines, s);
      if (!b) return false;
      for (let i = b.start; i <= b.end; i++) {
        const row = parseRow(lines[i]);
        if (row && row.stem.startsWith(prefix)) return true;
      }
      return false;
    });
    if (candidates.length !== 1) {
      throw new UsageError(`cannot auto-detect the target section for '${id}' (${candidates.length} candidate sections) - pass --section <name>`);
    }
    target = candidates[0];
  }

  const block = blockOf(lines, target);
  if (!block) throw new UsageError(`section '## ${target.name}' has no row block (not a regenerated layout) - refusing to guess placement`);

  const eol = crOf(lines[block.start]);
  const newRow = `- [${status}] ${stem} - ${title}${cls === "improvement" ? " (improvement)" : ""}` + eol;

  let insertedAt;
  const single = block.start === block.end && stripCR(lines[block.start]) === PLACEHOLDER;
  if (single) {
    lines[block.start] = newRow; // the empty section's placeholder becomes the first row
    insertedAt = block.start;
  } else {
    // Stem-ascending placement, bytewise on the stem token only.
    insertedAt = block.end + 1;
    for (let i = block.start; i <= block.end; i++) {
      const row = parseRow(lines[i]);
      if (row && row.stem > stem) { insertedAt = i; break; }
    }
    lines.splice(insertedAt, 0, newRow);
  }

  let headerInfo = { header_line: null, old_header: null, new_header: null };
  const rewritten = retallyHeader(lines, target.header);
  if (rewritten !== null && rewritten !== lines[target.header]) {
    headerInfo = { header_line: target.header + 1, old_header: stripCR(lines[target.header]), new_header: stripCR(rewritten) };
    lines[target.header] = rewritten;
  }
  const tot = retallyTotals(lines);            // §1.3 - a stale total is stale whichever mutation caused it
  let totalsInfo = {};
  if (tot.line >= 0 && tot.text !== lines[tot.line]) { totalsInfo = { totals_line: tot.line + 1 }; lines[tot.line] = tot.text; }
  atomicWrite(path, lines.join("\n"));
  return {
    code: 0, backlog: given, line: insertedAt + 1, row: stripCR(newRow), section: target.name,
    replaced_placeholder: single, ...headerInfo, ...totalsInfo,
    message: `insert ${id}: row landed at line ${insertedAt + 1} in '## ${target.name}'${single ? " (placeholder replaced)" : ""}${headerInfo.header_line ? `; header retallied at line ${headerInfo.header_line}` : ""}${totalsInfo.totals_line ? `; Totals retallied at line ${totalsInfo.totals_line}` : ""}`,
  };
}

// ── next-id (TASK-IMP-105): allocate a module's next task stem ────────────────
// The next id for a module is the HIGHEST existing stem in docs/tasks/<module>/ PLUS ONE
// (spec §1.2, §1.5). The corpus scanned is the set of TASK-<PREFIX>-<NNN> task FOLDERS on
// disk, not BACKLOG rows: the folder is the task, the row is only its index, so a half-landed
// task that has a folder but no row is still counted — skipping it would hand out the colliding
// id again (spec edge row 1). Every backlog row has a folder but not every folder has a row, so
// the folder set is a superset of the row set and highest-folder+1 is free in BOTH: an id this
// returns can never be rejected by the insert uniqueness gate for non-uniqueness in the same
// instant (spec §1.3). GAPS ARE IGNORED — 001,002,004 -> 005, never the free 003 (§1.5) —
// because reusing a retired id makes two different tasks share a name in the history.
//
// A populated module derives its PREFIX from the folders themselves (docs/tasks/improvement/
// holds TASK-IMP-*, not TASK-IMPROVEMENT-*), so the live corpus's abbreviated modules
// (improvement -> IMP, templates -> TPL) resolve with no hand-maintained map. An empty or
// not-yet-existent module is NOT an error: it returns that module's first stem,
// TASK-<MODULE-UPPERCASED>-001 (spec §1.4, edge row 4).
//
// Read-only and deterministic: reads directory names, prints one stem, writes nothing and
// flips nothing (node stdlib only). The <module> argument is confined under docs/tasks/ by the
// shared relUnderRoot rule (spec security-class edge row) so a crafted `../` cannot walk out.
// A single TASK-prefixed folder that does not parse is skipped with a note on stderr — one bad
// folder must not stop allocation (spec edge row 2).
const STEM_RE = /^TASK-([A-Za-z0-9]+)-(\d+)(?:-.*)?$/;

function cmdNextId(root, positionals) {
  const [module] = positionals;
  if (!module) throw new UsageError("next-id requires <module>");
  if (!ID_RE.test(module)) throw new UsageError(`module must match ${ID_RE} (a single docs/tasks/<module> directory name)`);
  const base = join(root, "docs", "tasks");
  const rel = relUnderRoot(base, module);
  if (rel === null || rel.includes("/")) {
    throw new UsageError(`module '${module}' must be a single directory name directly under docs/tasks/`);
  }
  const moduleDir = join(base, module);

  // Scan the module directory. A non-existent (or unreadable) directory is an EMPTY module,
  // not an error (spec §1.4, edge row 4).
  let names = [];
  if (existsSync(moduleDir)) {
    try { names = readdirSync(moduleDir); } catch { names = []; }
  }

  let best = null;                          // { prefix, num, width } of the highest parsed stem
  for (const name of names) {
    let st; try { st = statSync(join(moduleDir, name)); } catch { continue; }
    if (!st.isDirectory()) continue;        // a task is a FOLDER; a stray MIGRATION-MAP.md etc. is not one
    const m = STEM_RE.exec(name);
    if (!m) {
      // A folder that LOOKS like a task (TASK-*) but does not parse is malformed: note it and
      // move on. A folder that is not TASK-* at all is simply not a task and is ignored silently.
      if (/^TASK-/.test(name)) process.stderr.write(`next-id: skipping malformed task folder '${name}' under docs/tasks/${module}/\n`);
      continue;
    }
    const num = parseInt(m[2], 10);
    if (best === null || num > best.num) best = { prefix: m[1], num, width: m[2].length };
  }

  let stem;
  if (best === null) {
    // First stem for an empty/absent module. The prefix defaults to the uppercased module name;
    // the corpus's populated modules never reach this branch, so their abbreviations are moot.
    stem = `TASK-${module.toUpperCase()}-001`;
  } else {
    const next = String(best.num + 1).padStart(Math.max(3, best.width), "0");
    stem = `TASK-${best.prefix}-${next}`;
  }
  return { code: 0, module, stem, message: stem };
}

// ── CLI shell ────────────────────────────────────────────────────────────────
const HELP = `backlog-mutate.mjs - byte-discipline executor for backlog-state-update@2 writes (TASK-IMP-085)

usage: node backlog-mutate.mjs [--json] [--root <repo-root>] <command> ...

commands
  flip <task-id> <from> <to> [--backlog <path>] [--old-line <text>]
       [--verdict-by <actor> --verdict-evidence <path>]
      rewrite ONE status cell: the row is located by stem, the cell must equal <from>,
      and --old-line (the recorded pre-image) must match the full line byte-for-byte
      when given; every other byte of the line is preserved. The task's spec.md
      frontmatter 'status' MUST already equal <to> or the flip refuses (exit 6): the
      frontmatter is the record of truth, the index only catches up to it (TASK-IMP-120).
      A counted section header ('(N status, ...)') is rewritten from a full retally of
      the section's rows after the flip; bare headers stay untouched.
      The two human-acceptance gates - reviewing -> ready_to_test and testing -> done
      (STATUS-REFERENCE §1.4) - REQUIRE a recorded human verdict: --verdict-by (a
      non-empty actor) plus --verdict-evidence (an existing, non-empty regular file,
      resolved against --root when relative); a bare gate flip refuses with exit 8 and
      writes nothing (TASK-CUO-303). Every other transition ignores the flags. When a
      BRAIN store resolves (CYBEROS_STORE, else <root>/.cyberos/memory/store), the
      gated flip first appends ONE status_overridden row via memory-append.mjs
      (payload {actor, task_id, prior_status, new_status, reason: evidence-path});
      an append failure on a present store fails the flip (exit 9) BEFORE the index
      moves. With no store, the flip succeeds and the evidence file is the record.
  insert <task-id> <stem> <title> <status> [--backlog <path>] [--section <name>] [--class product|improvement]
      insert ONE row in the regenerator-identical grammar
      '- [<status>] <stem> - <title>' (+ ' (improvement)'), stem-ascending inside the
      target section's contiguous block; a '- (nothing remaining)' placeholder becomes
      the first row. Uniqueness is enforced across the WHOLE file. This tool never
      creates sections.
  next-id <module>
      print the module's next task stem to stdout and exit 0: the HIGHEST existing
      TASK-<PREFIX>-<NNN> folder in docs/tasks/<module>/ PLUS ONE (gaps ignored -
      001,002,004 -> 005). An empty or absent module returns its first stem
      (...-001). Read-only: reads folder names, writes and flips nothing. The insert
      uniqueness gate (exit 7) stays the authority on admission (TASK-IMP-105).

exit codes
  0  ok
  2  usage error, unreadable backlog, section not found / ambiguous, no row block
  6  flip refusal: missing row, duplicate rows, drifted pre-image (status cell or
     --old-line bytes) - optimistic concurrency per backlog-state-update-author SKILL.md §3
     - OR the spec frontmatter does not already carry <to> (truth precedes index,
     TASK-IMP-120: unfindable / unreadable / ambiguous / disagreeing truth all refuse)
  7  insert refusal: a row for the id already exists (uniqueness pre-image violated)
  8  flip refusal: a human-acceptance gate transition (reviewing -> ready_to_test,
     testing -> done) with no recorded verdict - STATUS-REFERENCE §1.4 requires
     --verdict-by + --verdict-evidence (existing, non-empty regular file); evaluated
     AFTER every exit-6 refusal, so 8 means "otherwise legal, verdict missing"
  9  flip failure: the status_overridden verdict row could not be appended to a
     PRESENT BRAIN store - audit-before-action, the index never moves without its row

discipline
    a mutation is exactly one row, at most one section header, and at most one Totals
    line - a 3-line ceiling (TASK-IMP-116). the header and Totals, when present and
    counted, are FULL retallies after the mutation: an inherited wrong count is corrected,
    never propagated (TASK-IMP-092). a file with no Totals line is never given one. this
    tool never moves, reorders, or deletes rows and never normalizes line endings (CRLF
    round-trips). deterministic: identical input + args = byte-identical result
    file and stdout (no clock, no randomness in output). writes are two-phase atomic
    (.tmp.<nonce> then rename). node stdlib only.
`;

function main(argv) {
  const flags = new Set(["json", "help"]);
  const valued = new Set(["root", "backlog", "old-line", "section", "class", "verdict-by", "verdict-evidence"]);
  const opts = {};
  const positionals = [];
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "-h" || a === "--help") { opts.help = true; continue; }
    if (a.startsWith("--")) {
      const name = a.slice(2);
      if (flags.has(name)) { opts[name] = true; continue; }
      if (valued.has(name)) {
        if (i + 1 >= argv.length) { process.stderr.write(`backlog-mutate: --${name} needs a value\n`); return 2; }
        opts[name] = argv[++i]; continue;
      }
      process.stderr.write(`backlog-mutate: unknown flag '${a}'\n${HELP}`);
      return 2;
    }
    positionals.push(a);
  }
  if (opts.help) { process.stdout.write(HELP); return 0; }
  const [command, ...rest] = positionals;
  const emit = (r) => {
    if (opts.json) {
      const env = { command, ok: r.code === 0, exit_code: r.code, task_id: rest[0] ?? null, ...r };
      delete env.code;
      process.stdout.write(stableStringify(env) + "\n");
    } else if (r.message) {
      process.stdout.write(r.message + "\n");
    }
    return r.code;
  };
  try {
    if (command === "flip") return emit(cmdFlip(findRoot(opts.root), rest, opts));
    if (command === "insert") return emit(cmdInsert(findRoot(opts.root), rest, opts));
    if (command === "next-id") return emit(cmdNextId(findRoot(opts.root), rest));
    throw new UsageError(command ? `unknown command '${command}'` : "no command given");
  } catch (e) {
    if (e instanceof Refusal || e instanceof UsageError) {
      const code = e instanceof Refusal ? e.code : 2;
      if (opts.json) {
        process.stdout.write(stableStringify({ command: command ?? null, task_id: rest[0] ?? null, ok: false, exit_code: code, error: e.message, ...(e.fields ?? {}) }) + "\n");
      } else {
        process.stderr.write(`backlog-mutate: ${e.message}\n`);
        if (code === 2) process.stderr.write("usage: node backlog-mutate.mjs [--json] [--root <dir>] <flip|insert> ... (--help for details)\n");
      }
      return code;
    }
    throw e;
  }
}

process.exitCode = main(process.argv.slice(2));
