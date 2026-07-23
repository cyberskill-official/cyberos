#!/usr/bin/env node
// task-lint.mjs — deterministic machine floor under audit_rubric@2.0 (TASK-IMP-084).
//
// Checks the MECHANICAL rule families of modules/skill/task-audit/RUBRIC.md on
// `template: task@1` specs, tagging every finding with its exact rule_id:
//   FM-001..004, FM-101..117   frontmatter structure + per-field rules (§1-§2)
//   SEC-001..009               required sections + hierarchy (§3)
//   COND-001..004              conditionally-required sections (§4)
//   TRACE-001..003             structural traceability halves (§9)
// Judgment families (QA semantics, SAFE content scanning, TRACE semantic
// sufficiency, XCHAIN/STALE) stay with the model audit — this tool FLOORS the
// audit; it never replaces the 10/10 verdict.
//
// Usage:  node task-lint.mjs [--json] <spec.md|dir ...>
//         directories recurse to */spec.md
// Output: one finding per line, `SEVERITY rule_id file:line message`, bytewise
//         sorted; --json emits the same findings (same order) as a JSON array.
// Exit:   0 iff zero error-severity findings, else 2.
// Determinism: byte-identical across runs on identical input — no clock, no
//         env-derived text, no randomness, sorted traversal.
//
// Frontmatter is parsed with a STRICT minimal YAML reader — exactly the subset
// task@1 uses: flat `key: value`, single/double-quoted strings (with escapes),
// inline `[a, b]` lists, block `- item` lists, own-line comments, blank lines.
// Anything beyond that (anchors, aliases, block scalars, nested maps, flow
// maps, trailing comments) is itself an FM-001 finding naming the line:
// failing loudly beats silently accepting what the rule families never defined.
// Node stdlib only (docs-tools convention — see md.mjs, render-status-hub.mjs).

import { readFileSync, readdirSync, existsSync, statSync } from "node:fs";
import { join, dirname, resolve } from "node:path";

// ── rubric enums (RUBRIC.md §2; FM-104 per STATUS-REFERENCE.md §1) ───────────
const DEPARTMENTS = ["engineering", "design", "product", "sales", "operations", "hr", "client_success"];
// TASK-IMP-108: two words were doing two jobs each. `draft` meant four different things (336 rows)
// and `ready_to_implement` meant both "audited, never built" and "built, failed, going round
// again". The fix is to record the REASON, never to mint a status - the enum stays at 12 values
// (operator decision 2026-07-17: `implement` keeps its name). Both fields are OPTIONAL: absent
// means unknown, which is honest for the 336 drafts this run did not author. Inventing a reason
// for them would be the `# UNREVIEWED` mistake with better manners.
const DRAFT_REASONS = ["authoring", "migrated_stub", "needs_spec", "parked_idea"];
const ENTERED_VIA   = ["audit", "rework", "spec_rejected"];
const STATUSES = ["draft", "ready_to_implement", "implementing", "ready_to_review", "reviewing",
  "ready_to_test", "testing", "done", "on_hold", "closed", "cannot_reproduce", "duplicate"];
const PRIORITIES = ["p0", "p1", "p2", "p3"];
const AI_AUTHORSHIP = ["none", "assisted", "co_authored", "generated_then_reviewed"];
const TYPES = ["feature", "bug", "improvement", "chore"];
const RISK_CLASSES = ["not_ai", "minimal", "limited", "high"];
const REQUIRED_SECTIONS = [ // SEC-001..007, in rule order
  ["SEC-001", "Summary"], ["SEC-002", "Problem"], ["SEC-003", "Proposed Solution"],
  ["SEC-004", "Alternatives Considered"], ["SEC-005", "Success Metrics"],
  ["SEC-006", "Scope"], ["SEC-007", "Dependencies"],
];

// ── findings ─────────────────────────────────────────────────────────────────
function finding(findings, severity, rule, file, line, message) {
  findings.push({ severity, rule_id: rule, file, line, message });
}

// ── strict minimal YAML scalar ───────────────────────────────────────────────
// Returns {v, quoted} or {err}. `s` arrives trimmed.
function parseScalar(s) {
  if (s === "") return { v: null, quoted: false };
  const c0 = s[0];
  if (c0 === '"' || c0 === "'") {
    let i = 1, out = "";
    if (c0 === '"') {
      for (; i < s.length; i++) {
        if (s[i] === "\\" && i + 1 < s.length) { out += s[i + 1] === '"' || s[i + 1] === "\\" ? s[i + 1] : "\\" + s[i + 1]; i++; continue; }
        if (s[i] === '"') break;
        out += s[i];
      }
    } else {
      for (; i < s.length; i++) {
        if (s[i] === "'" && s[i + 1] === "'") { out += "'"; i++; continue; }
        if (s[i] === "'") break;
        out += s[i];
      }
    }
    if (i >= s.length) return { err: "unterminated quoted string" };
    if (i !== s.length - 1) return { err: "content after closing quote" };
    return { v: out, quoted: true };
  }
  if ("&*|>{}%?".includes(c0)) return { err: `unsupported YAML construct '${c0}...' (strict task@1 subset)` };
  if (s.includes(" #")) return { err: "trailing comment after value (comments must be own-line)" };
  return { v: s, quoted: false };
}

// Split an inline [a, b] interior on top-level commas, respecting quotes.
function splitInline(inner) {
  const parts = []; let cur = "", q = null;
  for (let i = 0; i < inner.length; i++) {
    const ch = inner[i];
    if (q) { cur += ch; if (ch === "\\" && q === '"') { cur += inner[i + 1] ?? ""; i++; } else if (ch === q) q = null; continue; }
    if (ch === '"' || ch === "'") { q = ch; cur += ch; continue; }
    if (ch === ",") { parts.push(cur.trim()); cur = ""; continue; }
    cur += ch;
  }
  parts.push(cur.trim());
  return parts;
}

// ── frontmatter reader (FM-001/002/003 + raw FM-112) ────────────────────────
// Returns { entries: Map(key -> {v, quoted, list, line}), close } or null when
// the fences themselves are broken (caller stops the file as template_ambiguous).
function readFrontmatter(lines, file, findings) {
  if (lines[0] !== "---") {
    finding(findings, "error", "FM-001", file, 1, "frontmatter must open with '---' on line 1");
    return null;
  }
  let close = -1;
  for (let i = 1; i < lines.length; i++) if (lines[i] === "---") { close = i; break; }
  if (close === -1) {
    finding(findings, "error", "FM-001", file, 1, "closing '---' frontmatter fence not found");
    return null;
  }
  const entries = new Map();
  let i = 1;
  while (i < close) {
    const raw = lines[i];
    const ln = i + 1;
    // FM-112 raw scan: the marker must not survive draft, however it is spelled.
    if (raw.includes("# UNREVIEWED")) {
      finding(findings, "error", "FM-112", file, ln, "'# UNREVIEWED' marker must not survive draft — a human confirms ai_authorship and eu_ai_act_risk_class");
    }
    if (raw.trim() === "" || /^\s*#/.test(raw)) { i++; continue; }
    if (/^\s/.test(raw)) {
      finding(findings, "error", "FM-001", file, ln, "indented line outside a block list (strict task@1 subset)");
      i++; continue;
    }
    const m = /^([^:\s]+):(.*)$/.exec(raw);
    if (!m) {
      finding(findings, "error", "FM-001", file, ln, "expected 'key: value' (strict task@1 subset)");
      i++; continue;
    }
    const key = m[1];
    const rest = m[2].trim();
    if (!/^[a-z_][a-z0-9_]*$/.test(key)) {
      finding(findings, "error", "FM-002", file, ln, `frontmatter key '${key}' is not snake_case`);
    }
    const dup = entries.has(key);
    if (dup) finding(findings, "error", "FM-003", file, ln, `duplicate frontmatter key '${key}'`);

    let entry = null;
    if (rest === "") {
      // possible block list on the following indented lines
      const items = []; let j = i + 1;
      while (j < close) {
        const l = lines[j];
        const im = /^\s+-\s?(.*)$/.exec(l);
        if (im) {
          const sc = parseScalar(im[1].trim());
          if (sc.err) finding(findings, "error", "FM-001", file, j + 1, sc.err);
          else items.push(sc.v ?? "");
          j++; continue;
        }
        if (l.trim() === "" || /^\s*#/.test(l)) {
          // blanks/comments end the list unless another item follows
          let k = j;
          while (k < close && (lines[k].trim() === "" || /^\s*#/.test(lines[k]))) k++;
          if (k < close && /^\s+-\s?/.test(lines[k])) { j = k; continue; }
        }
        break;
      }
      entry = items.length > 0 ? { list: items, line: ln } : { v: null, quoted: false, line: ln };
      if (items.length > 0) i = j - 1;
    } else if (rest[0] === "[") {
      if (!rest.endsWith("]")) {
        finding(findings, "error", "FM-001", file, ln, "unterminated inline list");
        entry = { v: null, quoted: false, line: ln };
      } else {
        const inner = rest.slice(1, -1).trim();
        if (inner.includes("[") || inner.includes("{")) {
          finding(findings, "error", "FM-001", file, ln, "nested collection in inline list (strict task@1 subset)");
          entry = { v: null, quoted: false, line: ln };
        } else {
          const items = [];
          if (inner !== "") for (const part of splitInline(inner)) {
            const sc = parseScalar(part);
            if (sc.err) { finding(findings, "error", "FM-001", file, ln, sc.err); continue; }
            items.push(sc.v ?? "");
          }
          entry = { list: items, line: ln };
        }
      }
    } else {
      const sc = parseScalar(rest);
      if (sc.err) { finding(findings, "error", "FM-001", file, ln, sc.err); entry = { v: null, quoted: false, line: ln }; }
      else entry = { v: sc.v, quoted: sc.quoted, line: ln };
    }
    if (!dup) entries.set(key, entry);
    i++;
  }
  return { entries, close };
}

// ── repo root + FM-113 task resolution + TRACE-003 existence ────────────────
function repoRootFor(file) {
  let d = dirname(resolve(file));
  const start = d;
  for (;;) {
    if (existsSync(join(d, ".git")) || existsSync(join(d, "docs", "tasks"))) return d;
    const parent = dirname(d);
    if (parent === d) return start;
    d = parent;
  }
}

function taskIdResolves(root, id) {
  const base = join(root, "docs", "tasks");
  if (!existsSync(base)) return false;
  const stack = [base];
  while (stack.length > 0) {
    const dir = stack.pop();
    let ents;
    try { ents = readdirSync(dir, { withFileTypes: true }); } catch { continue; }
    for (const e of ents) {
      if (e.name.startsWith(".")) continue;
      if (e.name === id || e.name === id + ".md" || e.name.startsWith(id + "-")) return true;
      if (e.isDirectory()) stack.push(join(dir, e.name));
    }
  }
  return false;
}

// ── per-field FM checks (FM-101..114) ────────────────────────────────────────
function scalarOf(entries, key) {
  const e = entries.get(key);
  if (!e) return { present: false };
  if (e.list) return { present: true, isList: true, line: e.line };
  return { present: true, v: e.v, quoted: e.quoted, line: e.line };
}

// Absent is LEGAL here (unlike checkEnumField, where absence is an error). A field that means
// "unknown" must be allowed to be unknown, or every existing task reds on a rule about tasks
// nobody has triaged yet.
function checkOptionalEnumField(findings, file, entries, key, rule, allowed) {
  const f = scalarOf(entries, key);
  if (!f.present || (!f.isList && (f.v === null || f.v === ""))) return undefined;   // unknown: fine
  if (f.isList) { finding(findings, "error", rule, file, f.line, `${key} must be a string scalar`); return undefined; }
  if (!allowed.includes(f.v)) {
    finding(findings, "error", rule, file, f.line, `${key} must be one of ${allowed.join("|")} (got '${f.v}')`);
    return undefined;
  }
  return f.v;
}

function checkEnumField(findings, file, entries, key, rule, allowed, label) {
  const f = scalarOf(entries, key);
  if (!f.present || (!f.isList && (f.v === null || f.v === ""))) {
    finding(findings, "error", rule, file, f.present ? f.line : 1, `${key} is required`);
    return undefined;
  }
  if (f.isList) { finding(findings, "error", rule, file, f.line, `${key} must be a string scalar`); return undefined; }
  if (!allowed.includes(f.v)) {
    finding(findings, "error", rule, file, f.line, `${key} must be one of ${label} (got '${f.v}')`);
    return undefined;
  }
  return f.v;
}

function checkFrontmatterFields(findings, file, entries, root) {
  // FM-101 title: 1-72 code points after trim (a display bound, not a byte bound)
  const title = scalarOf(entries, "title");
  if (!title.present || (!title.isList && (title.v === null || title.v === ""))) {
    finding(findings, "error", "FM-101", file, title.present ? title.line : 1, "title is required");
  } else if (title.isList) {
    finding(findings, "error", "FM-101", file, title.line, "title must be a string scalar");
  } else {
    const n = [...title.v.trim()].length;
    if (n < 1 || n > 72) finding(findings, "error", "FM-101", file, title.line, `title must be 1-72 code points after trim (got ${n})`);
  }
  // FM-102 author
  const author = scalarOf(entries, "author");
  if (!author.present || (!author.isList && (author.v === null || author.v === ""))) {
    finding(findings, "error", "FM-102", file, author.present ? author.line : 1, "author is required");
  } else if (author.isList || !/^@[A-Za-z0-9_.-]{1,38}$/.test(author.v)) {
    finding(findings, "error", "FM-102", file, author.line, `author must match ^@[A-Za-z0-9_.-]{1,38}$ (got '${author.isList ? "[list]" : author.v}')`);
  }
  // FM-103/104/105/107/108 closed enums
  checkEnumField(findings, file, entries, "department", "FM-103", DEPARTMENTS, DEPARTMENTS.join("|"));
  const status = checkEnumField(findings, file, entries, "status", "FM-104", STATUSES, STATUSES.join("|"));
  checkEnumField(findings, file, entries, "priority", "FM-105", PRIORITIES, PRIORITIES.join("|"));
  const aiAuth = checkEnumField(findings, file, entries, "ai_authorship", "FM-107", AI_AUTHORSHIP, AI_AUTHORSHIP.join("|"));
  const type = checkEnumField(findings, file, entries, "type", "FM-108", TYPES, TYPES.join("|"));
  // FM-115 draft_reason - which KIND of draft this is (TASK-IMP-108 §1.1/§1.2)
  checkOptionalEnumField(findings, file, entries, "draft_reason", "FM-115", DRAFT_REASONS);
  // FM-116 entered_via - which KIND of ready_to_implement this is (TASK-IMP-108 §1.3)
  checkOptionalEnumField(findings, file, entries, "entered_via", "FM-116", ENTERED_VIA);
  // FM-106 created_at: ISO 8601 with timezone (Z or +-HH:MM)
  const created = scalarOf(entries, "created_at");
  if (!created.present || (!created.isList && (created.v === null || created.v === ""))) {
    finding(findings, "error", "FM-106", file, created.present ? created.line : 1, "created_at is required");
  } else if (created.isList
    || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}(:\d{2}(\.\d+)?)?(Z|[+-]\d{2}:\d{2})$/.test(created.v)
    || Number.isNaN(Date.parse(created.v))) {
    finding(findings, "error", "FM-106", file, created.line, `created_at must be ISO 8601 with a timezone (Z or +-HH:MM) (got '${created.isList ? "[list]" : created.v}')`);
  }
  // FM-109 eu_ai_act_risk_class ('unacceptable' rejected per Article 5)
  const risk = scalarOf(entries, "eu_ai_act_risk_class");
  let riskV;
  if (!risk.present || (!risk.isList && (risk.v === null || risk.v === ""))) {
    finding(findings, "error", "FM-109", file, risk.present ? risk.line : 1, "eu_ai_act_risk_class is required");
  } else if (!risk.isList && risk.v === "unacceptable") {
    finding(findings, "error", "FM-109", file, risk.line, "eu_ai_act_risk_class 'unacceptable' is rejected (EU AI Act Article 5)");
  } else if (risk.isList || !RISK_CLASSES.includes(risk.v)) {
    finding(findings, "error", "FM-109", file, risk.line, `eu_ai_act_risk_class must be one of ${RISK_CLASSES.join("|")} (got '${risk.isList ? "[list]" : risk.v}')`);
  } else riskV = risk.v;
  // FM-110 target_release: optional; SemVer or YYYY-Q[1-4] when present
  const tr = scalarOf(entries, "target_release");
  if (tr.present && !tr.isList && tr.v !== null && tr.v !== "") {
    if (!/^\d+\.\d+\.\d+(-[A-Za-z0-9.-]+)?$/.test(tr.v) && !/^\d{4}-Q[1-4]$/.test(tr.v)) {
      finding(findings, "error", "FM-110", file, tr.line, `target_release must be SemVer or YYYY-Q[1-4] (got '${tr.v}')`);
    }
  } else if (tr.present && tr.isList) {
    finding(findings, "error", "FM-110", file, tr.line, "target_release must be a string scalar");
  }
  // FM-111 client_visible: literal YAML boolean (not a quoted string, not yes/no)
  const cv = scalarOf(entries, "client_visible");
  let cvV;
  if (!cv.present || (!cv.isList && cv.v === null)) {
    finding(findings, "error", "FM-111", file, cv.present ? cv.line : 1, "client_visible is required");
  } else if (cv.isList || cv.quoted || (cv.v !== "true" && cv.v !== "false")) {
    const got = cv.isList ? "[list]" : cv.quoted ? `'"${cv.v}"' (quoted string)` : `'${cv.v}'`;
    finding(findings, "error", "FM-111", file, cv.line, `client_visible must be the literal YAML boolean true or false (got ${got})`);
  } else cvV = cv.v;
  // FM-113 duplicate_of: required iff status duplicate; must resolve to an existing task
  const dupOf = scalarOf(entries, "duplicate_of");
  if (status === "duplicate") {
    if (!dupOf.present || (!dupOf.isList && (dupOf.v === null || dupOf.v === ""))) {
      finding(findings, "error", "FM-113", file, dupOf.present ? dupOf.line : 1, "duplicate_of is required when status is duplicate — the link is the entire value of the status");
    } else if (dupOf.isList || !/^TASK-[A-Z0-9]+-\d+$/.test(dupOf.v)) {
      finding(findings, "error", "FM-113", file, dupOf.line, `duplicate_of must match TASK-<MODULE>-<NNN> (got '${dupOf.isList ? "[list]" : dupOf.v}')`);
    } else if (!taskIdResolves(root, dupOf.v)) {
      finding(findings, "error", "FM-113", file, dupOf.line, `duplicate_of '${dupOf.v}' does not resolve to an existing task under docs/tasks/`);
    }
  } else if (dupOf.present) {
    finding(findings, "error", "FM-113", file, dupOf.line, "duplicate_of is forbidden unless status is duplicate");
  }
  // FM-114 severity: required iff type bug (BUG-010); forbidden otherwise
  const sev = scalarOf(entries, "severity");
  if (type === "bug") {
    if (!sev.present || (!sev.isList && (sev.v === null || sev.v === ""))) {
      finding(findings, "error", "FM-114", file, sev.present ? sev.line : 1, "severity is required when type is bug");
    }
  } else if (sev.present && type !== undefined) {
    finding(findings, "error", "FM-114", file, sev.line, "severity is forbidden unless type is bug (severity is how bad if left alone; priority is when we get to it)");
  }
  // FM-117 module: when present, the value must be lowercase AND equal the containing
  // docs/tasks/<module>/ folder name (TASK-IMP-139; its audit's ISS-006 pinned BOTH
  // halves — `module: auth` inside improvement/ is lowercase and still wrong). The two
  // halves fire independently so each failure names itself. Outside the
  // docs/tasks/<module>/<task>/spec.md shape, and under the historical `_`/`.` trees,
  // only the case half is judged — there is no module folder to equal. Absent stays
  // legal: the rule governs values (the regenerator groups by folder either way).
  const modF = scalarOf(entries, "module");
  if (modF.present && modF.isList) {
    finding(findings, "error", "FM-117", file, modF.line, "module must be a string scalar");
  } else if (modF.present && modF.v !== null && modF.v !== "") {
    if (modF.v !== modF.v.toLowerCase()) {
      finding(findings, "error", "FM-117", file, modF.line, `module must be lowercase (got '${modF.v}')`);
    }
    const parts = resolve(file).split(/[\\/]/);
    let tasksIdx = -1;
    for (let k = parts.length - 1; k >= 1; k--) {
      if (parts[k - 1] === "docs" && parts[k] === "tasks") { tasksIdx = k; break; }
    }
    if (tasksIdx !== -1 && parts.length - tasksIdx >= 4) { // docs/tasks/<module>/<task…>/spec.md
      const folder = parts[tasksIdx + 1];
      if (!folder.startsWith("_") && !folder.startsWith(".") && modF.v.toLowerCase() !== folder) {
        finding(findings, "error", "FM-117", file, modF.line, `module must equal the containing docs/tasks/<module>/ folder name '${folder}' (got '${modF.v}')`);
      }
    }
  }
  return { clientVisible: cvV === "true", risk: riskV, aiAuthorship: aiAuth };
}

// ── body model ───────────────────────────────────────────────────────────────
// Structural lines exclude fenced code and <untrusted_content> interiors:
// quoted foreign bytes are data, not document structure (RUBRIC §6 discipline).
function buildBody(lines, from) {
  const rows = []; // {text, line, shadowed}
  let inFence = false, inUntrusted = false;
  for (let i = from; i < lines.length; i++) {
    const text = lines[i];
    const t = text.trim();
    let shadowed = inFence || inUntrusted;
    if (t.startsWith("```")) { shadowed = true; inFence = !inFence; }
    else if (!inFence && !inUntrusted && t.startsWith("<untrusted_content")) {
      shadowed = true;
      if (!t.includes("</untrusted_content>")) inUntrusted = true;
    } else if (!inFence && inUntrusted && t.includes("</untrusted_content>")) {
      shadowed = true; inUntrusted = false;
    }
    rows.push({ text, line: i + 1, shadowed });
  }
  const headings = [];
  for (const r of rows) {
    if (r.shadowed) continue;
    const m = /^(#{1,6})\s+(.*)$/.exec(r.text);
    if (m) headings.push({ level: m[1].length, text: m[2].trim(), line: r.line });
  }
  return { rows, headings };
}

// Rows strictly between heading h and the next heading (any level when
// anyLevel, else the next heading with level <= h.level).
function sectionRows(body, h, anyLevel) {
  const out = [];
  let next = Infinity;
  for (const g of body.headings) {
    if (g.line > h.line && (anyLevel || g.level <= h.level)) { next = g.line; break; }
  }
  for (const r of body.rows) if (r.line > h.line && r.line < next) out.push(r);
  return out;
}

const findH2 = (body, name) => body.headings.find((h) => h.level === 2 && h.text === name);

// ── SEC family ───────────────────────────────────────────────────────────────
function checkSections(findings, file, body) {
  for (const [rule, name] of REQUIRED_SECTIONS) {
    const h = findH2(body, name);
    if (!h) { finding(findings, "error", rule, file, 1, `required section '## ${name}' is missing`); continue; }
    const content = sectionRows(body, h, true);
    if (!content.some((r) => r.text.trim() !== "")) {
      finding(findings, "error", "SEC-008", file, h.line, `section '## ${name}' is empty (no non-blank line before the next heading)`);
    }
  }
  // SEC-009 (warning): one or zero H1s; no skipped heading levels
  let h1Count = 0, prevLevel = null;
  for (const h of body.headings) {
    if (h.level === 1) {
      h1Count++;
      if (h1Count === 2) finding(findings, "warning", "SEC-009", file, h.line, "more than one H1 heading");
    }
    if (prevLevel !== null && h.level > prevLevel + 1) {
      finding(findings, "warning", "SEC-009", file, h.line, `heading level jumps from H${prevLevel} to H${h.level}`);
    }
    prevLevel = h.level;
  }
}

// ── COND family ──────────────────────────────────────────────────────────────
function checkConditionalSections(findings, file, body, fm) {
  if (fm.clientVisible) {
    const cq = findH2(body, "Customer Quotes");
    if (!cq) finding(findings, "error", "COND-001", file, 1, "client_visible: true requires '## Customer Quotes'");
    else if (!sectionRows(body, cq, false).some((r) => r.text.includes("<untrusted_content"))) {
      finding(findings, "error", "COND-001", file, cq.line, "'## Customer Quotes' must carry at least one quote inside an <untrusted_content> block");
    }
    if (!findH2(body, "Sales/CS Summary")) {
      finding(findings, "error", "COND-002", file, 1, "client_visible: true requires '## Sales/CS Summary'");
    }
  }
  if (fm.risk === "limited" || fm.risk === "high") {
    const ra = findH2(body, "AI Risk Assessment");
    if (!ra) finding(findings, "error", "COND-003", file, 1, `eu_ai_act_risk_class '${fm.risk}' requires '## AI Risk Assessment'`);
    else {
      const span = sectionRows(body, ra, false);
      const lo = span.length > 0 ? span[0].line : ra.line;
      const hi = span.length > 0 ? span[span.length - 1].line : ra.line;
      const h3s = body.headings.filter((h) => h.level === 3 && h.line >= lo && h.line <= hi).map((h) => h.text);
      const want = ["Data Sources", "Human Oversight", "Failure Modes"];
      const idx = want.map((w) => h3s.indexOf(w));
      if (idx.some((n) => n === -1)) {
        finding(findings, "error", "COND-003", file, ra.line, `'## AI Risk Assessment' must carry H3s '### Data Sources', '### Human Oversight', '### Failure Modes' (missing: ${want.filter((_, k) => idx[k] === -1).join(", ")})`);
      } else if (!(idx[0] < idx[1] && idx[1] < idx[2])) {
        finding(findings, "error", "COND-003", file, ra.line, "'## AI Risk Assessment' H3s must appear in order: Data Sources, Human Oversight, Failure Modes");
      }
    }
  }
  if (fm.aiAuthorship !== undefined && fm.aiAuthorship !== "none") {
    const ad = findH2(body, "AI Authorship Disclosure");
    if (!ad) finding(findings, "error", "COND-004", file, 1, `ai_authorship '${fm.aiAuthorship}' requires '## AI Authorship Disclosure'`);
    else {
      const bullets = sectionRows(body, ad, false).filter((r) => !r.shadowed && /^\s*[-*]\s/.test(r.text));
      const missing = ["Tools used:", "Scope:", "Human review:"].filter((l) => !bullets.some((b) => b.text.includes(l)));
      if (missing.length > 0) {
        finding(findings, "error", "COND-004", file, ad.line, `'## AI Authorship Disclosure' must carry bullets labeled 'Tools used:', 'Scope:', 'Human review:' (missing: ${missing.join(", ")})`);
      }
    }
  }
}

// ── TRACE structural halves ──────────────────────────────────────────────────
function checkTraceability(findings, file, body, entries, root) {
  const descH = body.headings.find((h) => h.level === 2 && /^1\.\s*Description\b/.test(h.text));
  const clauses = []; // {id: '1.N', sub, line, text}
  if (descH) {
    for (const r of sectionRows(body, descH, false)) {
      if (r.shadowed) continue;
      const m = /^-\s+1\.(\d+)\s+(.*)$/.exec(r.text);
      if (m) clauses.push({ sub: m[1], line: r.line, text: m[2] });
    }
  }
  const acs = body.rows.filter((r) => !r.shadowed && /^- \[[ x]\] AC /.test(r.text));

  // TRACE-001: every BCP-14 clause is cited by at least one AC.
  if (clauses.length === 0) {
    finding(findings, "info", "TRACE-001", file, 1, "no numbered '- 1.N' clauses under '## 1. Description' — clause traceability left to the model audit");
  } else {
    for (const c of clauses) {
      if (!/\b(MUST|SHOULD|MAY)\b/.test(c.text)) continue; // covers MUST NOT / SHOULD NOT too
      if (c.text.includes("(deferred to slice")) continue;
      const reDot = new RegExp(`#1\\.${c.sub}(?![0-9])`);
      const reHash = new RegExp(`§1 #${c.sub}(?![0-9])`);
      const cited = acs.some((a) => {
        if (reDot.test(a.text) || reHash.test(a.text)) return true;
        const t = /traces_to:\s*([^)]*)/.exec(a.text);
        if (!t) return false;
        return t[1].split(/[\s,]+/).some((tok) => {
          const clean = tok.replace(/^[#§]*/, "").replace(/[),.;]+$/, "");
          return clean === `1.${c.sub}`;
        });
      });
      if (!cited) {
        finding(findings, "error", "TRACE-001", file, c.line, `clause 1.${c.sub} carries a BCP-14 keyword but no AC cites it (via '#1.${c.sub}', '§1 #${c.sub}', or a traces_to list)`);
      }
    }
  }
  // TRACE-002: every AC line carries a test: or verify: entry.
  for (const a of acs) {
    if (!/\btest:|\bverify:/.test(a.text)) {
      finding(findings, "error", "TRACE-002", file, a.line, "AC carries neither a 'test:' nor a 'verify:' entry");
    }
  }
  // TRACE-003: every backticked `path::name` in a test: entry is in new_files or on disk.
  const nf = entries.get("new_files");
  const newFiles = nf && nf.list ? nf.list : [];
  for (const a of acs) {
    const re = /test:\s*`([^`]+)`/g;
    let m;
    while ((m = re.exec(a.text)) !== null) {
      if (!m[1].includes("::")) continue;
      const p = m[1].split("::")[0];
      if (!newFiles.includes(p) && !existsSync(resolve(root, p))) {
        finding(findings, "error", "TRACE-003", file, a.line, `test path '${p}' is neither listed in frontmatter new_files nor an existing file relative to the repo root`);
      }
    }
  }
}

// ── per-file driver ──────────────────────────────────────────────────────────
function lintFile(file, findings) {
  let text;
  try {
    text = readFileSync(file, "utf8");
  } catch (e) {
    finding(findings, "error", "FM-004", file, 0, `template_ambiguous: unreadable input (${e.code || "read error"})`);
    return;
  }
  if (text.charCodeAt(0) === 0xfeff) text = text.slice(1); // BOM
  const lines = text.split("\n").map((l) => (l.endsWith("\r") ? l.slice(0, -1) : l)); // CRLF-normalised

  const fmRead = readFrontmatter(lines, file, findings);
  if (fmRead === null) {
    finding(findings, "error", "FM-004", file, 1, "template_ambiguous: frontmatter fences unparseable — cannot detect template");
    return;
  }
  const { entries, close } = fmRead;
  const body = buildBody(lines, close + 1);

  // FM-004 template detection (RUBRIC §10): this lint handles task@1 only.
  const tpl = entries.get("template");
  const tplV = tpl && !tpl.list ? tpl.v : undefined;
  const engGrammar = body.headings.some((h) => /^§\d+/.test(h.text));
  if (tplV !== "task@1") {
    const detail = tpl === undefined
      ? "template key absent (expected 'task@1')"
      : `template is '${tpl.list ? "[list]" : String(tplV)}' (expected 'task@1')`;
    finding(findings, "error", "FM-004", file, tpl ? tpl.line : 1,
      `template_ambiguous: ${detail}${engGrammar ? "; body carries engineering-spec '## §N' grammar" : ""} — this lint handles task@1 only, stopping this file`);
    return;
  }
  if (engGrammar) {
    finding(findings, "error", "FM-004", file, tpl.line,
      "template_ambiguous: task@1 frontmatter but the body carries engineering-spec '## §N' grammar — both markers present, stopping this file");
    return;
  }

  const root = repoRootFor(file);
  const fm = checkFrontmatterFields(findings, file, entries, root);
  checkSections(findings, file, body);
  checkConditionalSections(findings, file, body, fm);
  checkTraceability(findings, file, body, entries, root);
}

// ── input expansion (dirs recurse to */spec.md, sorted traversal) ────────────
function collectSpecs(path, out, findings) {
  let st;
  try { st = statSync(path); } catch {
    finding(findings, "error", "FM-004", path, 0, "template_ambiguous: unreadable input (ENOENT)");
    return;
  }
  if (st.isFile()) { out.push(path); return; }
  if (!st.isDirectory()) return;
  let ents;
  try { ents = readdirSync(path, { withFileTypes: true }); } catch { return; }
  ents.sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0));
  for (const e of ents) {
    if (e.name.startsWith(".")) continue;
    const p = join(path, e.name);
    if (e.isDirectory()) collectSpecs(p, out, findings);
    else if (e.isFile() && e.name === "spec.md") out.push(p);
  }
}

// ── main ─────────────────────────────────────────────────────────────────────
function main(argv) {
  let json = false;
  const paths = [];
  for (const a of argv) {
    if (a === "--json") { json = true; continue; }
    if (a.startsWith("--")) {
      process.stderr.write(`task-lint: unknown flag '${a}'\nusage: node task-lint.mjs [--json] <spec.md|dir ...>\n`);
      return 2;
    }
    paths.push(a);
  }
  if (paths.length === 0) {
    process.stderr.write("usage: node task-lint.mjs [--json] <spec.md|dir ...>\n");
    return 2;
  }
  const findings = [];
  const files = [];
  for (const p of paths) collectSpecs(p, files, findings);
  for (const f of files) lintFile(f, findings);

  const lineOf = (f) => `${f.severity} ${f.rule_id} ${f.file}:${f.line} ${f.message}`;
  findings.sort((a, b) => { const x = lineOf(a), y = lineOf(b); return x < y ? -1 : x > y ? 1 : 0; });
  if (json) {
    process.stdout.write(JSON.stringify(findings, null, 2) + "\n");
  } else {
    for (const f of findings) process.stdout.write(lineOf(f) + "\n");
  }
  return findings.some((f) => f.severity === "error") ? 2 : 0;
}

process.exitCode = main(process.argv.slice(2));
