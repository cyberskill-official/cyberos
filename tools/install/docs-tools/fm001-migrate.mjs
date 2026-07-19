#!/usr/bin/env node
// fm001-migrate.mjs — bring a repo's task@1 frontmatter into the strict subset FM-001 accepts (TASK-IMP-117).
//
// WHY: task-lint's FM-001 (TASK-IMP-084) accepts only the documented task@1 subset — flat
// `key: value`, quoted strings, inline `[a, b]` lists, block `- item` lists, own-line comments,
// blanks. Anything else names the offending line as an FM-001 finding. FM-001 has TWO structural
// classes of violation in the corpus, and this tool clears BOTH:
//
//   (1) TRAILING COMMENTS — `id: TASK-X-001  # note` parses as the value `TASK-X-001  # note`
//       for any consumer that does not strip comments (the defect that wrote `priority: MUST` +
//       `# MUST | SHOULD` into a parsed field). The shipped TASK-TEMPLATE.md taught that shape, so
//       ~every spec was BORN violating the rule. This tool rewrites those to OWN-LINE comments
//       (above the field, indent preserved). Migrated corpus-wide at commit 4c02b556.
//
//   (2) NESTED MAPS — a top-level key whose value is an indented CHILD MAP, e.g.
//       `build_envelope:` followed by `  language: rust 1.81` / `  new_files:` / `    - x`. The
//       strict reader has no model of nested maps (by design — see task-lint.mjs header), so it
//       records the parent as a null scalar and flags every indented child line as FM-001
//       ("indented line outside a block list"). This tool FLATTENS such a key: it hoists the
//       children to top-level keys, dedenting the child block by its own base indent and dropping
//       the parent line, so `  new_files:` / `    - x` become top-level `new_files:` / `  - x` —
//       the flat shape a sibling `done` spec already uses and the shape batch-select reads. This
//       is general to ANY nested-map key, not hard-coded to build_envelope. When a hoisted key
//       already exists at top level it RECONCILES: two block lists become an order-preserving
//       union (exact-duplicate item values deduped, nothing dropped); two scalars of equal value
//       dedupe to one; a genuine scalar/kind conflict HALTS and names the file rather than drop or
//       overwrite. (TASK-IMP-117 §1.8; investigation docs/tasks/_audits/2026-07-18-fm001-nested-map-fork.md.)
//
// It ships in the payload (build.sh vendor list) so any installed repo can clean its own corpus —
// the trailing-comment template shape is vendored, so every consumer corpus inherited that class.
//
// SCOPE — clause 1.4: ONLY the frontmatter block between the FIRST TWO `---` lines is rewritten.
// The body is never read for structure and never re-emitted; its bytes pass through untouched.
// This is what preserves `audited_body_sha256_prefix` on bound specs: the migrator only ever runs
// non-trivially on specs that carry one of the two violation classes, and the audit-bound specs
// carry neither (they are disjoint from the nested-map and trailing-comment sets), so it is a
// byte-for-byte no-op on every bound spec.
//
// SAFETY — this tool REWRITES specs, so it is guarded like every docs-tool that acts on repo files
// (task-reconcile, verify-goals, coverage-scope): CONFINE (relUnderRoot) → EXISTS → TRACKED AT HEAD
// (git ls-tree). An untracked or escaping path is REFUSED, never migrated; git-absent / not-a-repo
// refuses the whole run and says so. Idempotent: a second run is a byte-for-byte no-op. A `#` is a
// comment only when it begins a comment in a PLAIN scalar (space-then-`#`); a quote is a string
// delimiter only when it BEGINS the scalar value, so a `#` inside a quoted value, a `#` mid-token
// with no leading whitespace (`a#b`), and a `#` after a mid-value apostrophe (`broker's ... #4`)
// are handled exactly as the strict reader handles them.
//
// Usage:  node fm001-migrate.mjs [--check] [--json] [--repo <root>] <spec.md ...>
//   --check  report what WOULD change; write nothing (exit 2 if any file still needs migrating).
//   --json   emit a stable JSON summary instead of one line per file.
//   --repo   repo root the guard confines to (default: discovered from the first path / cwd).
// Exit:  0  every named file is FM-001-clean (already clean, or — without --check — cleaned now).
//        2  usage error, a REFUSED file (escapes root / untracked / no frontmatter / unreadable /
//           unreconcilable nested-map collision), git-absent / not-a-repo, OR (--check) at least
//           one file still needs migrating.
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

// ── scalar value start (clause 1.3, quote model) ─────────────────────────────
// Index in `content` of the first non-whitespace char of the scalar VALUE — after `key:` for a
// `key: value` line, or after `-` for a `- item` list line — or -1 when the line carries no scalar
// value (blank, own-line comment, or empty value). Mirrors task-lint's readFrontmatter: the value
// region is what parseScalar sees, and a quote is a string delimiter ONLY when it is the first
// char of that region.
function valueStart(content) {
  const lead = content.match(/^[ \t]*/)[0].length;
  const rest = content.slice(lead);
  if (rest === "" || rest[0] === "#") return -1;                 // blank / own-line comment
  let after;                                                     // index just past the prefix marker
  if (rest[0] === "-" && (rest.length === 1 || rest[1] === " " || rest[1] === "\t")) {
    after = lead + 1;                                            // block-list item: past the dash
  } else {
    const m = /^[^:\s]+:/.exec(rest);                            // key: — past the first colon
    if (!m) return -1;
    after = lead + m[0].length;
  }
  let k = after;
  while (k < content.length && (content[k] === " " || content[k] === "\t")) k++;
  return k < content.length ? k : -1;                            // -1 = empty value
}

// ── trailing-comment detection (clause 1.3) ──────────────────────────────────
// Index of the '#' that starts a trailing comment on this line's content, or -1. The comment is a
// '#' that is: (a) NOT inside a quoted scalar, and (b) preceded by whitespace in the value region.
// An OWN-LINE comment (first non-ws char is '#') is never a trailing comment. A quote is a
// delimiter only when it BEGINS the scalar value (valueStart): so a `#` inside a quoted value
// (edges #5/#6) is data, a bare `a#b` (edge #7) is not a comment, and a `#` after a mid-value
// apostrophe in a PLAIN scalar (`broker's ... #4`) IS a comment — matching task-lint, which reads
// `broker's` as a literal apostrophe and ` #` as a plain-scalar comment.
function trailingCommentIndex(c) {
  if (c.trimStart().startsWith("#")) return -1;                  // own-line comment (edges #3/#4)
  const vs = valueStart(c);
  if (vs === -1) return -1;                                      // no scalar value on this line
  const first = c[vs];
  if (first === '"' || first === "'") {
    // Quoted scalar: '#' inside the string is data; a comment can only follow the closing quote.
    let i = vs + 1;
    if (first === '"') {
      for (; i < c.length; i++) { if (c[i] === "\\") { i++; continue; } if (c[i] === '"') { i++; break; } }
    } else {
      for (; i < c.length; i++) { if (c[i] === "'" && c[i + 1] === "'") { i++; continue; } if (c[i] === "'") { i++; break; } }
    }
    for (; i < c.length; i++) if (c[i] === "#" && (c[i - 1] === " " || c[i - 1] === "\t")) return i;
    return -1;
  }
  // Plain (unquoted) scalar: the first ' #' / '\t#' after the value begins a comment.
  for (let i = vs + 1; i < c.length; i++) {
    if (c[i] === "#" && (c[i - 1] === " " || c[i - 1] === "\t")) return i;
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

// ── nested-map flatten (clause 1.8) ──────────────────────────────────────────
const leadLen = (s) => s.match(/^[ \t]*/)[0].length;
const isBlank = (s) => s.trim() === "";
const isComment = (s) => /^[ \t]*#/.test(s);
const isItem = (s) => /^[ \t]*-(?:[ \t]|$)/.test(s);
const itemValue = (s) => s.replace(/^[ \t]*-[ \t]?/, "").trim();

// Parse the frontmatter interior (the lines strictly between the two `---` fences) into an ordered
// list of top-level CONSTRUCTS. A nested-map construct is EXPANDED in place into its (dedented)
// child constructs — so the returned list is flat (no nested maps remain) but may carry duplicate
// keys, which reconcile() then merges. Construct kinds:
//   { kind:"loose",  key:null,  lines:[l] }              blank / own-line comment / stray line
//   { kind:"scalar", key, value, lines:[l] }             `key: value`, inline list, or empty scalar
//   { kind:"list",   key,        lines:[keyLine, ...] }  `key:` + block `- item` lines (+ interior blanks/comments)
// Returns { constructs, flattened } or { error }.
function parseConstructs(interior) {
  const constructs = [];
  let flattened = 0;
  let i = 0;
  while (i < interior.length) {
    const ln = interior[i];
    const c = ln.content;
    const km = /^([A-Za-z0-9_][^:\s]*):(.*)$/.exec(c);           // col-0 `key:` (no leading ws)
    if (!km) { constructs.push({ kind: "loose", key: null, lines: [ln] }); i++; continue; }
    const key = km[1];
    if (km[2].trim() !== "") {                                   // has a value: scalar or inline list
      constructs.push({ kind: "scalar", key, value: km[2].trim(), lines: [ln] });
      i++; continue;
    }
    // empty value → collect the indented child block (interior blanks kept, trailing blanks dropped)
    let j = i + 1, lastReal = i;
    while (j < interior.length) {
      const cc = interior[j].content;
      if (isBlank(cc)) { j++; continue; }
      if (/^[ \t]/.test(cc)) { lastReal = j; j++; continue; }    // indented content or comment
      break;                                                     // col-0 non-blank → block ends
    }
    const blockEnd = lastReal + 1;
    const block = interior.slice(i + 1, blockEnd);
    const nb = block.filter((l) => !isBlank(l.content));
    if (nb.length === 0) {                                       // `key:` with no children → null scalar
      constructs.push({ kind: "scalar", key, value: "", lines: [ln] });
      i++; continue;
    }
    const base = Math.min(...nb.map((l) => leadLen(l.content)));
    const directNonComment = nb.filter((l) => leadLen(l.content) === base && !isComment(l.content));
    const hasMapChild = directNonComment.some((l) => !isItem(l.content));
    if (!hasMapChild) {                                          // plain block list → valid task@1, leave alone
      constructs.push({ kind: "list", key, lines: [ln, ...block] });
      i = blockEnd; continue;
    }
    // NESTED MAP → flatten: drop the parent line, dedent the block by its base indent, re-parse.
    const dedented = block.map((l) => (isBlank(l.content) ? l : { content: l.content.slice(base), eol: l.eol }));
    const child = parseConstructs(dedented);
    if (child.error) return child;
    for (const ch of child.constructs) constructs.push(ch);      // hoist in place
    flattened += 1 + child.flattened;
    i = blockEnd; continue;
  }
  return { constructs, flattened };
}

// Merge hoisted list `b` into existing list `a`: order-preserving union of item VALUES, exact
// duplicates dropped (nothing unique dropped). `a.lines` gains b's non-duplicate item lines.
function mergeList(a, b) {
  const seen = new Set();
  for (const l of a.lines) if (isItem(l.content)) seen.add(itemValue(l.content));
  for (const l of b.lines.slice(1)) {                            // skip b's key line
    if (!isItem(l.content)) continue;                            // drop blanks/comments from the merged-in block
    const v = itemValue(l.content);
    if (seen.has(v)) continue;                                   // exact-duplicate value → dedupe
    seen.add(v);
    a.lines.push(l);
  }
}

// Reconcile duplicate top-level keys left by flatten. Order-preserving union for lists, dedupe for
// equal scalars; HALT (name the file) on a genuine scalar conflict or a list/scalar kind mismatch.
function reconcile(constructs) {
  const out = [];
  const byKey = new Map();
  for (const con of constructs) {
    if (con.kind === "loose" || con.key === null) { out.push(con); continue; }
    const prev = byKey.get(con.key);
    if (!prev) { out.push(con); byKey.set(con.key, con); continue; }
    if (prev.kind === "list" && con.kind === "list") { mergeList(prev, con); continue; }
    if (prev.kind === "scalar" && con.kind === "scalar") {
      if (prev.value === con.value) continue;                    // identical scalar → dedupe
      return { error: `nested-map flatten collides on scalar key '${con.key}': existing '${prev.value}' vs hoisted '${con.value}' — refusing to overwrite` };
    }
    return { error: `nested-map flatten collides on key '${con.key}' with mismatched kinds (${prev.kind} vs ${con.kind}) — refusing to reconcile` };
  }
  return { out };
}

// ── migrate one file's TEXT (pure; no IO) ────────────────────────────────────
// Returns { text, moved, flattened } on success or { error } when there is no frontmatter block
// (edge #2) or a nested-map collision cannot be reconciled. Only the interior lines between the
// first two `---` fences are considered; the opening fence, the closing fence, and every body line
// pass through as their original {content, eol}. Two passes: (A) flatten nested maps + reconcile,
// (B) move trailing comments to own lines.
function migrateText(text) {
  const lines = scanLines(text);
  if (lines.length === 0 || lines[0].content !== "---") {
    return { error: "no frontmatter block (a spec must open with '---' on line 1)" };
  }
  let close = -1;
  for (let k = 1; k < lines.length; k++) if (lines[k].content === "---") { close = k; break; }
  if (close === -1) return { error: "no frontmatter block (closing '---' fence not found)" };

  const interior = lines.slice(1, close);

  // Pass A — flatten nested maps, then reconcile any duplicate hoisted keys.
  const parsed = parseConstructs(interior);
  if (parsed.error) return { error: parsed.error };
  const rec = reconcile(parsed.constructs);
  if (rec.error) return { error: rec.error };
  const flatInterior = [];
  for (const con of rec.out) for (const l of con.lines) flatInterior.push(l);
  const flattened = parsed.flattened;

  // Pass B — move trailing comments to their own line above the field.
  const out = [lines[0]];
  let moved = 0;
  for (const ln of flatInterior) {
    const split = splitTrailingComment(ln.content);
    if (!split) { out.push(ln); continue; }
    const eol = ln.eol || "\n";
    out.push({ content: split.commentLine, eol }, { content: split.valueLine, eol });
    moved++;
  }
  for (let k = close; k < lines.length; k++) out.push(lines[k]); // closing fence + body, verbatim
  return { text: out.map((l) => l.content + l.eol).join(""), moved, flattened };
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
    if (r.moved === 0 && r.flattened === 0) { results.push({ file: rel, status: "clean", moved: 0, flattened: 0 }); continue; }
    if (check) { results.push({ file: rel, status: "would-migrate", moved: r.moved, flattened: r.flattened }); continue; }
    atomicWrite(join(root, rel), r.text);
    results.push({ file: rel, status: "migrated", moved: r.moved, flattened: r.flattened });
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
      else {
        const parts = [];
        if (r.moved > 0) parts.push(`${r.moved} comment${r.moved === 1 ? "" : "s"}`);
        if (r.flattened > 0) parts.push(`${r.flattened} nested map${r.flattened === 1 ? "" : "s"}`);
        const verb = r.status === "would-migrate" ? "would-migrate" : "migrated";
        process.stdout.write(`${verb} ${r.file} (${parts.join(", ")})\n`);
      }
    }
    process.stderr.write(`fm001-migrate: ${migrated} migrated, ${clean} clean, ${would} would-migrate, ${refused} refused${check ? " (--check: wrote nothing)" : ""}\n`);
  }
  return code;
}

function gitToplevel(from) {
  const r = spawnSync("git", ["-C", from, "rev-parse", "--show-toplevel"], { encoding: "utf8" });
  return !r.error && r.status === 0 ? r.stdout.trim() : null;
}

const HELP = `fm001-migrate.mjs — bring task@1 frontmatter into the FM-001 subset (TASK-IMP-117)

usage: node fm001-migrate.mjs [--check] [--json] [--repo <root>] <spec.md ...>
  --check  report what WOULD change; write nothing (exit 2 if any file still needs migrating)
  --json   stable JSON summary instead of one line per file
  --repo   repo root the guard confines to (default: git toplevel above the first path)

clears both FM-001 structural classes: trailing frontmatter comments (moved to own lines) and
nested-map keys (children hoisted to top-level keys; colliding lists order-preservingly unioned)

exit  0  every named file is FM-001-clean (already, or cleaned now)
      2  usage / a refused file (escapes root, untracked, no frontmatter, unreadable, unreconcilable
         collision) / git-absent or not-a-repo / (--check) a file still needs migrating
guard confine (under root) → exists → tracked at HEAD (git ls-tree); untracked/escaping = REFUSED
writes two-phase atomic (.tmp.<nonce> then fsync+rename); a killed write cannot truncate a spec
`;
function usage(msg) { process.stderr.write(`fm001-migrate: ${msg}\n${HELP}`); }

process.exitCode = main(process.argv.slice(2));
