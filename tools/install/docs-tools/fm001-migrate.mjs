#!/usr/bin/env node
// fm001-migrate.mjs — move trailing frontmatter comments to their own line (TASK-IMP-117).
//
// WHY: task-lint's FM-001 (TASK-IMP-084) refuses a trailing comment after a frontmatter value —
// `id: TASK-X-001  # note` parses as the value `TASK-X-001  # note` for any consumer that does not
// strip comments, the exact defect that wrote `priority: MUST  # MUST | SHOULD` into a parsed
// field. The shipped TASK-TEMPLATE.md taught that shape, so ~every spec was BORN violating the
// rule. This tool rewrites those trailing comments to OWN-LINE comments (above the field, indent
// preserved), which is what FM-001 asks for. It ships in the payload (build.sh vendor list) so any
// installed repo can clean its own corpus — the template is vendored, so every consumer inherited
// the disease.
//
// WHAT FM-001 ACTUALLY IS (measured, not assumed): a trailing comment is the sole root cause. The
// linter also emits "indented line outside a block list" and "unterminated inline list" — but BOTH
// are DOWNSTREAM of a trailing comment: when every `- item` in a block list carries a `# note`,
// task-lint's parseScalar errors on all of them, the list parses empty, the cursor is not advanced,
// and the `- item` lines are re-scanned as stray indented lines; an inline `[a, b]  # note` fails
// `endsWith("]")`. Remove the trailing comments and all three clear. (TASK-IMP-117 §Problem.)
//
// SCOPE — clause 1.4: ONLY the frontmatter block between the FIRST TWO `---` lines is rewritten.
// The body is never read for structure and never re-emitted; its bytes pass through untouched.
//
// SAFETY — this tool REWRITES specs, so it is guarded like every docs-tool that acts on repo files
// (task-reconcile, verify-goals, coverage-scope): CONFINE (relUnderRoot) → EXISTS → TRACKED AT HEAD
// (git ls-tree). An untracked or escaping path is REFUSED, never migrated; git-absent / not-a-repo
// refuses the whole run and says so. Idempotent: a second run is a byte-for-byte no-op. A `#` inside
// a quoted value, or one not preceded by whitespace (`a#b`), is NOT a comment and is left alone.
//
// Usage:  node fm001-migrate.mjs [--check] [--json] [--repo <root>] <spec.md ...>
//   --check  report what WOULD change; write nothing (exit 2 if any file still needs migrating).
//   --json   emit a stable JSON summary instead of one line per file.
//   --repo   repo root the guard confines to (default: discovered from the first path / cwd).
// Exit:  0  every named file is FM-001-clean (already clean, or — without --check — cleaned now).
//        2  usage error, a REFUSED file (escapes root / untracked / no frontmatter / unreadable),
//           git-absent / not-a-repo, OR (--check) at least one file still needs migrating.
// Writes are two-phase atomic (`.tmp.<nonce>` then fsync+rename); a killed write cannot truncate a
// spec. Node stdlib only (docs-tools convention — see task-lint.mjs, ship-manifest.mjs).

import { readFileSync, writeFileSync, existsSync, renameSync, openSync, fsyncSync, closeSync, mkdirSync } from "node:fs";
import { resolve, relative, isAbsolute, dirname, join } from "node:path";
import { spawnSync } from "node:child_process";
import { randomBytes } from "node:crypto";

// ── guard predicate (one rule, shared with task-reconcile / verify-goals / coverage-scope) ──
const relUnderRoot = (root, p) => {
  const rel = relative(root, resolve(root, p));
  return (rel === "" || rel.startsWith("..") || isAbsolute(rel)) ? null : rel;
};

// ── two-phase atomic write (memory-protocol discipline, per ship-manifest.mjs) ──
function atomicWrite(path, text) {
  mkdirSync(dirname(path), { recursive: true });
  const tmp = `${path}.tmp.${randomBytes(6).toString("hex")}`;
  writeFileSync(tmp, text);
  const fd = openSync(tmp, "r");
  try { fsyncSync(fd); } finally { closeSync(fd); }
  renameSync(tmp, path);
}

// ── EOL-preserving line scanner ──────────────────────────────────────────────
// Returns [{content, eol}] covering the whole text; joining content+eol reproduces the input
// byte-for-byte (CRLF preserved, trailing-newline-or-not preserved). Splits only on '\n', so a
// '\r\n' line keeps its '\r' as part of the eol and never leaks into content.
function scanLines(text) {
  const out = [];
  let i = 0;
  while (i < text.length) {
    const nl = text.indexOf("\n", i);
    if (nl === -1) { out.push({ content: text.slice(i), eol: "" }); return out; }
    let content = text.slice(i, nl), eol = "\n";
    if (content.endsWith("\r")) { content = content.slice(0, -1); eol = "\r\n"; }
    out.push({ content, eol });
    i = nl + 1;
  }
  return out;
}

// ── trailing-comment detection (clause 1.3) ──────────────────────────────────
// Index of the '#' that starts a trailing comment on this line's content, or -1. A trailing
// comment is a '#' that is: (a) NOT inside a single/double-quoted string, (b) preceded by
// whitespace, and (c) has at least one non-whitespace char before it — so an OWN-LINE comment
// (prefix all whitespace, edges #3/#4) and a bare `a#b` (no leading ws, edge #7) are NOT comments,
// and a `#` inside a quoted value (edges #5/#6) is data, not a comment.
function trailingCommentIndex(c) {
  // An OWN-LINE comment (first non-whitespace char is '#') is never a trailing comment, even if it
  // contains a later ' #' inside its own prose (e.g. a `# ...  # note` example). task-lint skips
  // `/^\s*#/` lines before it parses anything; mirror that, or a second run would split the very
  // comment lines the first run created (idempotence, clause 1.5; edges #3/#4).
  if (c.trimStart().startsWith("#")) return -1;
  let q = null; // null | '"' | "'"
  for (let i = 0; i < c.length; i++) {
    const ch = c[i];
    if (q === '"') { if (ch === "\\") { i++; continue; } if (ch === '"') q = null; continue; }
    if (q === "'") { if (ch === "'" && c[i + 1] === "'") { i++; continue; } if (ch === "'") q = null; continue; }
    if (ch === '"' || ch === "'") { q = ch; continue; }
    if (ch === "#" && i > 0 && (c[i - 1] === " " || c[i - 1] === "\t") && c.slice(0, i).trim() !== "") return i;
  }
  return -1;
}

// Rewrite one frontmatter line, or null if it carries no trailing comment. The comment moves to an
// own line ABOVE, carrying the source line's own indentation (edge #8: list-item indent preserved);
// the value keeps everything left of the comment, minus the separator whitespace.
function splitTrailingComment(content) {
  const p = trailingCommentIndex(content);
  if (p === -1) return null;
  const indent = content.match(/^[ \t]*/)[0];
  return { commentLine: indent + content.slice(p), valueLine: content.slice(0, p).replace(/[ \t]+$/, "") };
}

// ── migrate one file's TEXT (pure; no IO) ────────────────────────────────────
// Returns { text, moved } on success or { error } when there is no frontmatter block (edge #2).
// Only the interior lines between the first two `---` fences are considered; the opening fence,
// the closing fence, and every body line pass through as their original {content, eol}.
function migrateText(text) {
  const lines = scanLines(text);
  if (lines.length === 0 || lines[0].content !== "---") {
    return { error: "no frontmatter block (a spec must open with '---' on line 1)" };
  }
  let close = -1;
  for (let k = 1; k < lines.length; k++) if (lines[k].content === "---") { close = k; break; }
  if (close === -1) return { error: "no frontmatter block (closing '---' fence not found)" };

  const out = [lines[0]];
  let moved = 0;
  for (let k = 1; k < close; k++) {
    const ln = lines[k];
    const split = splitTrailingComment(ln.content);
    if (!split) { out.push(ln); continue; }
    const eol = ln.eol || "\n";
    out.push({ content: split.commentLine, eol }, { content: split.valueLine, eol });
    moved++;
  }
  for (let k = close; k < lines.length; k++) out.push(lines[k]); // closing fence + body, verbatim
  return { text: out.map((l) => l.content + l.eol).join(""), moved };
}

// ── git guard ────────────────────────────────────────────────────────────────
function confirmRepo(root) {
  const r = spawnSync("git", ["-C", root, "rev-parse", "--is-inside-work-tree"], { encoding: "utf8" });
  if (r.error) return { ok: false, why: `git not found on PATH — refusing to migrate unguarded (the tracked-at-HEAD guard needs git)` };
  if (r.status !== 0 || r.stdout.trim() !== "true") return { ok: false, why: `${root} is not a git repository — refusing to migrate unguarded` };
  return { ok: true };
}
function trackedAtHead(root, rel) {
  const r = spawnSync("git", ["-C", root, "ls-tree", "HEAD", "--", rel], { encoding: "utf8" });
  return !r.error && r.status === 0 && r.stdout.trim() !== "";
}

// ── main ───────────────────────────────────────────────────────────────────────
function main(argv) {
  let check = false, json = false, repo = null;
  const paths = [];
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--check") check = true;
    else if (a === "--json") json = true;
    else if (a === "--repo") { repo = argv[++i]; if (repo === undefined) { usage("--repo needs a value"); return 2; } }
    else if (a === "-h" || a === "--help") { process.stdout.write(HELP); return 0; }
    else if (a.startsWith("--")) { usage(`unknown flag '${a}'`); return 2; }
    else paths.push(a);
  }
  if (paths.length === 0) { usage("no spec paths given"); return 2; }

  // Root: explicit --repo, else the git toplevel above the first path, else its dir, else cwd.
  const root = resolve(repo || gitToplevel(dirname(resolve(paths[0]))) || process.cwd());
  const results = [];
  const repoOk = confirmRepo(root);

  for (const p of paths) {
    if (!repoOk.ok) { results.push({ file: p, status: "refused", reason: repoOk.why }); continue; }
    const rel = relUnderRoot(root, p);
    if (rel === null) { results.push({ file: p, status: "refused", reason: "escapes the repo root — REFUSED, not migrated" }); continue; }
    if (!existsSync(join(root, rel))) { results.push({ file: rel, status: "refused", reason: "resolves nowhere — REFUSED, not migrated" }); continue; }
    if (!trackedAtHead(root, rel)) { results.push({ file: rel, status: "refused", reason: "not tracked at HEAD — an untracked spec is not corpus, REFUSED" }); continue; }
    let text;
    try { text = readFileSync(join(root, rel), "utf8"); }
    catch (e) { results.push({ file: rel, status: "refused", reason: `unreadable (${e.code || "read error"}) — migrated nothing` }); continue; }
    const r = migrateText(text);
    if (r.error) { results.push({ file: rel, status: "refused", reason: r.error }); continue; }
    if (r.moved === 0) { results.push({ file: rel, status: "clean", moved: 0 }); continue; }
    if (check) { results.push({ file: rel, status: "would-migrate", moved: r.moved }); continue; }
    atomicWrite(join(root, rel), r.text);
    results.push({ file: rel, status: "migrated", moved: r.moved });
  }

  const refused = results.filter((r) => r.status === "refused").length;
  const would = results.filter((r) => r.status === "would-migrate").length;
  const migrated = results.filter((r) => r.status === "migrated").length;
  const clean = results.filter((r) => r.status === "clean").length;
  const code = refused > 0 || (check && would > 0) ? 2 : 0;

  if (json) {
    process.stdout.write(JSON.stringify({ tool: "fm001-migrate@1", check, root, migrated, clean, would_migrate: would, refused, files: results }, null, 2) + "\n");
  } else {
    for (const r of results) {
      if (r.status === "refused") process.stderr.write(`REFUSED ${r.file}: ${r.reason}\n`);
      else if (r.status === "clean") process.stdout.write(`clean ${r.file}\n`);
      else if (r.status === "would-migrate") process.stdout.write(`would-migrate ${r.file} (${r.moved} comment${r.moved === 1 ? "" : "s"})\n`);
      else process.stdout.write(`migrated ${r.file} (${r.moved} comment${r.moved === 1 ? "" : "s"} moved to own line)\n`);
    }
    process.stderr.write(`fm001-migrate: ${migrated} migrated, ${clean} clean, ${would} would-migrate, ${refused} refused${check ? " (--check: wrote nothing)" : ""}\n`);
  }
  return code;
}

function gitToplevel(from) {
  const r = spawnSync("git", ["-C", from, "rev-parse", "--show-toplevel"], { encoding: "utf8" });
  return !r.error && r.status === 0 ? r.stdout.trim() : null;
}

const HELP = `fm001-migrate.mjs — move trailing frontmatter comments to their own line (TASK-IMP-117)

usage: node fm001-migrate.mjs [--check] [--json] [--repo <root>] <spec.md ...>
  --check  report what WOULD change; write nothing (exit 2 if any file still needs migrating)
  --json   stable JSON summary instead of one line per file
  --repo   repo root the guard confines to (default: git toplevel above the first path)

exit  0  every named file is FM-001-clean (already, or cleaned now)
      2  usage / a refused file (escapes root, untracked, no frontmatter, unreadable) /
         git-absent or not-a-repo / (--check) a file still needs migrating
guard confine (under root) → exists → tracked at HEAD (git ls-tree); untracked/escaping = REFUSED
writes two-phase atomic (.tmp.<nonce> then fsync+rename); a killed write cannot truncate a spec
`;
function usage(msg) { process.stderr.write(`fm001-migrate: ${msg}\n${HELP}`); }

process.exitCode = main(process.argv.slice(2));
