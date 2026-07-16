#!/usr/bin/env node
// backlog-mutate.mjs — byte-discipline executor for backlog-state-update@2 writes (TASK-IMP-085).
//
// Executes the two sanctioned BACKLOG.md mutations from
// modules/skill/backlog-state-update-author/SKILL.md §2-§3 — flip one status cell,
// insert one row — and NOTHING else: this tool never moves, reorders, or deletes rows,
// and never edits a line outside the declared mutation (whole-file discipline: one row
// plus at most one section-header line per mutation). Grammar authority stays with
// regen_backlog() in scripts/migrate_task_layout.py / migrate_improvement_to_task.py;
// this tool encodes it, never redefines it.
//
// Usage:  node backlog-mutate.mjs [--json] [--root <repo-root>] <command> ...
//
//   flip <task-id> <from> <to> [--backlog <path>] [--old-line <text>]
//       Locate the row by task STEM (the `<task-id>-<slug>` token), verify the status
//       cell equals <from> AND — when --old-line carries the recorded pre-image — the
//       full old line byte-for-byte (line terminator excluded), then rewrite EXACTLY
//       that one cell; every other byte of the line (title, tags, comments, CR) is
//       preserved. Refuses with exit 6 on a missing row, on 2+ matching rows (corrupted
//       backlog — both lines are named, never a guess), or on any drift. When the
//       containing `## section` header carries `(N status, ...)` counts, the counts are
//       updated (from -1, to +1, zero counts dropped, statuses in lifecycle order);
//       a header without parseable counts is left untouched.
//   insert <task-id> <stem> <title> <status> [--backlog <path>] [--section <name>] [--class product|improvement]
//       Uniqueness gate first: NO row for <task-id> (or <stem>) may pre-exist anywhere
//       in the file — violation is exit 7 naming the line. The row is rendered in the
//       regenerator-identical grammar `- [<status>] <stem> - <title>` (+ ` (improvement)`
//       for --class improvement) and placed in stem-ascending order inside the target
//       section's contiguous row block (bytewise on the stem token only — titles never
//       affect placement). A `- (nothing remaining)` placeholder is replaced by the new
//       first row. Header counts gain +1 <status> when the header carries counts.
//       --section names the target `## <name>` section; without it the section is
//       auto-detected as the unique section already holding rows with the same
//       `TASK-<MODULE>-` prefix (zero or many candidates = exit 2, pass --section).
//       This tool never creates sections; regenerate the backlog for that.
//
// Exit codes:
//   0  ok
//   2  usage error, unreadable backlog, section not found / ambiguous, no row block
//   6  flip refusal: missing row, duplicate rows, or drifted pre-image (status cell or
//      --old-line bytes) — the optimistic-concurrency check from SKILL.md §3
//   7  insert refusal: a row for the id already exists (uniqueness pre-image violated)
//
// Byte discipline: the file is split on '\n' and rejoined on '\n' only — CRLF endings,
// a missing final newline, unicode titles, everything outside the mutated line(s)
// round-trips byte-identically (t07 proves it with a whole-file diff). Inserted rows
// take the line ending of their section. Writes are two-phase atomic (`.tmp.<nonce>`
// then rename). No clock, no randomness in output: identical input + identical args =
// byte-identical result file and stdout. Node stdlib only (docs-tools convention).

import {
  readFileSync, writeFileSync, renameSync, existsSync, mkdirSync,
  openSync, fsyncSync, closeSync,
} from "node:fs";
import { randomBytes } from "node:crypto";
import { join, resolve, dirname } from "node:path";

// STATUS-REFERENCE.md §1 enum; the first ten are regen_backlog()'s STATUS_ORDER —
// header counts render in this order.
const STATUS_ORDER = [
  "draft", "ready_to_implement", "implementing", "ready_to_review", "reviewing",
  "ready_to_test", "testing", "done", "on_hold", "closed", "cannot_reproduce", "duplicate",
];
const PLACEHOLDER = "- (nothing remaining)";
const ID_RE = /^[A-Za-z0-9._-]+$/;

class UsageError extends Error {}
class Refusal extends Error { constructor(code, msg) { super(msg); this.code = code; } }

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

// Returns the rewritten header line, or null when the header carries no counts or the
// counts do not cover the mutation (a header that never counted this row is left alone
// rather than rewritten into a lie).
function updateHeaderCounts(raw, deltas) {
  const parsed = parseCountsHeader(raw);
  if (!parsed) return null;
  const { prefix, counts } = parsed;
  for (const [status, d] of Object.entries(deltas)) {
    const cur = counts.get(status) || 0;
    if (cur + d < 0) return null; // counts don't cover this row; refuse to write a negative
    counts.set(status, cur + d);
  }
  const rendered = STATUS_ORDER.filter((s) => (counts.get(s) || 0) > 0)
    .map((s) => `${counts.get(s)} ${s}`).join(", ");
  return `${prefix}  (${rendered})` + crOf(raw);
}

const nearestHeaderAbove = (lines, idx) => {
  for (let i = idx - 1; i >= 0; i--) if (stripCR(lines[i]).startsWith("## ")) return i;
  return -1;
};

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
  const cellPrefix = `- [${from}]`;
  const oldLine = lines[i];
  const newLine = `- [${to}]` + oldLine.slice(cellPrefix.length);
  lines[i] = newLine;

  let headerInfo = { header_line: null, old_header: null, new_header: null };
  const h = nearestHeaderAbove(lines, i);
  if (h >= 0) {
    const rewritten = updateHeaderCounts(lines[h], { [from]: -1, [to]: +1 });
    if (rewritten !== null && rewritten !== lines[h]) {
      headerInfo = { header_line: h + 1, old_header: stripCR(lines[h]), new_header: stripCR(rewritten) };
      lines[h] = rewritten;
    }
  }
  atomicWrite(path, lines.join("\n"));
  return {
    code: 0, backlog: given, line: i + 1, old_line: stripCR(oldLine), new_line: stripCR(newLine), ...headerInfo,
    message: `flip ${id}: [${from}] -> [${to}] at line ${i + 1}${headerInfo.header_line ? `; header counts updated at line ${headerInfo.header_line}` : ""}`,
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
  const rewritten = updateHeaderCounts(lines[target.header], { [status]: +1 });
  if (rewritten !== null && rewritten !== lines[target.header]) {
    headerInfo = { header_line: target.header + 1, old_header: stripCR(lines[target.header]), new_header: stripCR(rewritten) };
    lines[target.header] = rewritten;
  }
  atomicWrite(path, lines.join("\n"));
  return {
    code: 0, backlog: given, line: insertedAt + 1, row: stripCR(newRow), section: target.name,
    replaced_placeholder: single, ...headerInfo,
    message: `insert ${id}: row landed at line ${insertedAt + 1} in '## ${target.name}'${single ? " (placeholder replaced)" : ""}${headerInfo.header_line ? `; header counts updated at line ${headerInfo.header_line}` : ""}`,
  };
}

// ── CLI shell ────────────────────────────────────────────────────────────────
const HELP = `backlog-mutate.mjs - byte-discipline executor for backlog-state-update@2 writes (TASK-IMP-085)

usage: node backlog-mutate.mjs [--json] [--root <repo-root>] <command> ...

commands
  flip <task-id> <from> <to> [--backlog <path>] [--old-line <text>]
      rewrite ONE status cell: the row is located by stem, the cell must equal <from>,
      and --old-line (the recorded pre-image) must match the full line byte-for-byte
      when given; every other byte of the line is preserved. Section-header counts
      update when the header carries '(N status, ...)' counts.
  insert <task-id> <stem> <title> <status> [--backlog <path>] [--section <name>] [--class product|improvement]
      insert ONE row in the regenerator-identical grammar
      '- [<status>] <stem> - <title>' (+ ' (improvement)'), stem-ascending inside the
      target section's contiguous block; a '- (nothing remaining)' placeholder becomes
      the first row. Uniqueness is enforced across the WHOLE file. This tool never
      creates sections.

exit codes
  0  ok
  2  usage error, unreadable backlog, section not found / ambiguous, no row block
  6  flip refusal: missing row, duplicate rows, or drifted pre-image (status cell or
     --old-line bytes) - optimistic concurrency per backlog-state-update-author SKILL.md §3
  7  insert refusal: a row for the id already exists (uniqueness pre-image violated)

discipline
  a mutation is exactly one row plus at most one header line; this tool never moves,
  reorders, or deletes rows, never normalizes line endings (CRLF round-trips), and never
  touches the Totals line. deterministic: identical input + args = byte-identical result
  file and stdout (no clock, no randomness in output). writes are two-phase atomic
  (.tmp.<nonce> then rename). node stdlib only.
`;

function main(argv) {
  const flags = new Set(["json", "help"]);
  const valued = new Set(["root", "backlog", "old-line", "section", "class"]);
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
    throw new UsageError(command ? `unknown command '${command}'` : "no command given");
  } catch (e) {
    if (e instanceof Refusal || e instanceof UsageError) {
      const code = e instanceof Refusal ? e.code : 2;
      if (opts.json) {
        process.stdout.write(stableStringify({ command: command ?? null, task_id: rest[0] ?? null, ok: false, exit_code: code, error: e.message }) + "\n");
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
