#!/usr/bin/env node
// workflow-improve.mjs — the outer loop's machine floor (TASK-IMP-110).
//
// Every ingredient of a learning loop already exists and nothing consumes them: human
// verdicts at two gates, route-back reasons, reconcile reports. This tool READS a bounded window
// of that exhaust and emits `skill-amendment@1` proposals. It PROPOSES; it never edits.
//
// NOT read: `memory.status_overridden` rows. TASK-IMP-110 §1.1 named them as a fourth stream and
// this header advertised them, but they live ONLY in `.cyberos/memory/store` - per-install, and
// gitignored (0 tracked files). This tool refuses any path not tracked at HEAD, on purpose: its
// evidence must be the corpus every reader can see, not one machine's local state. Reading them
// would mean weakening that guard AND making the output depend on which host ran it.
//
// So the spec asked for a stream the guard is right to refuse. The claim is corrected here rather
// than satisfied. If operator overrides are evidence the loop must learn from, they belong
// somewhere tracked - a design change, not a patch. (Greptile, PR #53, 2026-07-17.)
//
// THE HARD RULE (spec §1.4): this tool MUST NOT write to modules/**, any SKILL.md, any
// rubric, or any workflow file. There is exactly ONE write path in this file — the `--out`
// document — and it is refused before it opens if it resolves outside the repo root or
// into any of those protected trees. A skill edit is a doctrine change; doctrine changes
// go through a human, as a draft task, like everything else.
//
// DRAFT ONLY (spec §1.5). Every emitted proposal carries `status: draft`. This tool never
// audits, never flips a status, and never lands `ready_to_implement`. `ready_to_implement`
// is what a passing task-audit means, and a tool that awards itself that word is the
// machine grading its own homework.
//
// NO PADDING (spec §1.6). A window with no qualifying pattern reports "no amendment
// proposed" and emits NOTHING — no --out file, no placeholder proposal. Three proposals is
// a CAP, never a quota. An improver that always finds three has stopped measuring.
//
// UNTRUSTED INPUT (spec §3). Gate logs are prose written by a model. Quoted evidence is
// reproduced VERBATIM with its id and is never interpolated into any command — this tool
// spawns exactly one program (`git ls-tree`, argv form, never a shell string) and it never
// passes evidence text to it. Nothing read is ever executed.
//
// DETERMINISTIC. The corpus is the only input. No `new Date()`, no wall-clock, no network:
// two runs over the same tree produce byte-identical output. Evidence ids are content-
// addressed (sha256 of `<relpath>:<line>`), so a row keeps its id across windows.
//
// usage: node workflow-improve.mjs [--repo <root>] [--window <N>] [--json] [--out <path>]
// exits: 0 ran (proposals emitted, or "no amendment proposed")
//        2 usage · unreadable repo · --out refused (outside root, or into a protected tree)
//        3 one or more evidence paths REFUSED — present but unconfined or untracked at HEAD.
//          The absence is the finding: an unreadable evidence file is never a silent skip.
import { readFileSync, readdirSync, existsSync, statSync, writeFileSync, mkdirSync } from "node:fs";
import { join, resolve, relative, isAbsolute, dirname, basename } from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";

const argv = process.argv.slice(2);
const USAGE = "usage: node workflow-improve.mjs [--repo <root>] [--window <N>] [--json] [--out <path>]\n" +
  "  exit: 0 ran · 2 usage/--out refused · 3 evidence refused (unconfined or untracked at HEAD)";
if (argv.includes("--help")) { console.log(USAGE); process.exit(2); }
const asJson = argv.includes("--json");

// Read flags by presence, never by blind offset, and guard the NEXT token too — `--repo --json`
// must not swallow the flag as a path. Identical to verify-goals.mjs / batch-select.mjs; these
// siblings have drifted apart once already (the argv-offset bug was fixed in one and not the
// others). Keep them identical.
const flag = (name, dflt) => { const i = argv.indexOf(name); return (i >= 0 && argv[i + 1] !== undefined && !argv[i + 1].startsWith("--")) ? argv[i + 1] : dflt; };
const root = resolve(flag("--repo", "."));
const WINDOW_RAW = flag("--window", "20");
const WINDOW = Number(WINDOW_RAW);
if (!Number.isInteger(WINDOW) || WINDOW <= 0) { console.error(`workflow-improve: --window must be a positive integer (got '${WINDOW_RAW}')`); process.exit(2); }

// Same predicate as verify-goals.mjs / task-reconcile.mjs / coverage-scope.mjs. One rule, four tools.
const relUnderRoot = (root, p) => {
  const rel = relative(root, resolve(root, p));
  return (rel === "" || rel.startsWith("..") || isAbsolute(rel)) ? null : rel;
};
const sh = (cmd, args, cwd) => spawnSync(cmd, args, { cwd, encoding: "utf8", timeout: 10000 });
const sha8 = (s) => createHash("sha256").update(s).digest("hex").slice(0, 8);

if (!existsSync(join(root, "docs", "tasks"))) {
  console.error(`workflow-improve: no docs/tasks under ${root} — there is no run exhaust to read`);
  process.exit(2);
}

// ── §1.4 THE WRITE GUARD ────────────────────────────────────────────────────
// Refused BEFORE anything is read, so a refused --out cannot even be reached by a run that
// found proposals. `modules/**` is named explicitly because that is where every SKILL.md and
// rubric lives; the basename/segment rules catch the same file classes wherever they are
// vendored (a payload's cuo/skills/, a consumer's .cyberos/). Fail closed: an --out this tool
// cannot prove is harmless is refused, not sanitised.
const PROTECTED = [
  { test: (rel) => rel === "modules" || rel.startsWith("modules/"), why: "modules/** is the skill tree — this tool proposes amendments, it never applies them (§1.4)" },
  { test: (rel) => basename(rel) === "SKILL.md", why: "a SKILL.md is doctrine — a skill edit goes through a human as a draft task (§1.4)" },
  { test: (rel) => /(^|\/)rubrics?(\/|$)/.test(rel) || /(^|\/)[a-z0-9_-]*rubric[a-z0-9_-]*\.md$/i.test(rel), why: "a rubric is doctrine (§1.4)" },
  { test: (rel) => /(^|\/)workflows?(\/|$)/.test(rel), why: "a workflow file is doctrine (§1.4)" },
];
let outRel = null;
const outRaw = flag("--out", null);
if (outRaw !== null) {
  outRel = relUnderRoot(root, outRaw);
  if (outRel === null) { console.error(`workflow-improve: --out '${outRaw}' resolves outside the repo root — REFUSED, nothing written`); process.exit(2); }
  for (const p of PROTECTED) {
    if (p.test(outRel)) { console.error(`workflow-improve: --out '${outRel}' — REFUSED, nothing written: ${p.why}`); process.exit(2); }
  }
}

const fmOf = (t) => { if (!t.startsWith("---")) return null; const e = t.indexOf("\n---", 3); return e < 0 ? null : t.slice(4, e); };
const one = (f, k) => (f.match(new RegExp(`^${k}:\\s*(.*)$`, "m"))?.[1] ?? "").trim().replace(/^["']|["']$/g, "");
const isDir = (p) => { try { return statSync(p).isDirectory(); } catch { return false; } };
const lsDir = (p) => { try { return readdirSync(p); } catch { return []; } };

// ── the known-skill set: an EXISTENCE check against the repo, never a guess ──
// A row is attributed to a skill only when it names one that actually exists. Longest name
// first so `coverage-gate-author` is never mistaken for a prefix of something shorter.
const KNOWN_SKILLS = lsDir(join(root, "modules", "skill"))
  .filter((n) => !n.startsWith(".") && isDir(join(root, "modules", "skill", n)) && existsSync(join(root, "modules", "skill", n, "SKILL.md")))
  .sort((a, b) => b.length - a.length || a.localeCompare(b));

// ── the window: the last N COMPLETED tasks (§1.1) ───────────────────────────
// "Completed" is `status: done` — the one terminal success in STATUS-REFERENCE §1.1.
//
// "Last" is ordered by `shipped` (the recorded completion date), falling back to `created_at`
// when a done task carries no `shipped` — 9 of the 181 done tasks in this corpus do not.
// Both come from tracked bytes, so the order is deterministic; a git commit date would order
// by when the file was last TOUCHED, which is not when the task shipped.
//
// This was ordered by task NUMBER first. That is wrong and quietly so: module numbering is
// independent, so TASK-CUO-301 sorted above every TASK-IMP-1xx and the "last 20 completed"
// window contained none of the tasks this run actually shipped — and therefore none of the
// 17 gate logs in the corpus. It read 20 tasks and 0 evidence rows, and looked like a clean
// window rather than a mis-aimed one.
const notes = [];
const DATE = /^\d{4}-\d{2}-\d{2}/;
// `shipped: null` and `shipped: now()` both occur in this corpus. Neither is a date; an
// unparseable stamp falls back rather than sorting as the string "now()".
const dateOf = (v) => (DATE.test(v) ? v.slice(0, 10) : null);
const tasks = [];
const tasksRoot = join(root, "docs", "tasks");
for (const mod of lsDir(tasksRoot).sort()) {
  const modDir = join(tasksRoot, mod);
  if (mod.startsWith(".") || !isDir(modDir)) continue;
  for (const stem of lsDir(modDir).sort()) {
    const spec = join(modDir, stem, "spec.md");
    if (!existsSync(spec)) continue;
    let f = null;
    try { f = fmOf(readFileSync(spec, "utf8")); } catch { continue; }
    if (!f) continue;
    if (one(f, "status") !== "done") continue;
    const created = one(f, "created_at");
    const shipped = dateOf(one(f, "shipped"));
    tasks.push({
      id: one(f, "id"), dir: join(modDir, stem), rel: `docs/tasks/${mod}/${stem}`,
      completed_at: shipped || dateOf(created) || "0000-00-00",
      order_source: shipped ? "shipped" : (dateOf(created) ? "created_at (no shipped recorded)" : "none"),
      created,
    });
  }
}
tasks.sort((a, b) => b.completed_at.localeCompare(a.completed_at) || b.created.localeCompare(a.created) || b.id.localeCompare(a.id));
const window = tasks.slice(0, WINDOW);
const undated = window.filter((t) => t.order_source === "none").map((t) => t.id);
if (undated.length) notes.push(`ordered last in the window (no shipped and no parseable created_at): ${undated.join(", ")}`);

// ── the readers: gate logs · route-back reasons · reconcile reports ──────────────────
// Three line matchers, a closed documented set, applied to the three sources harvest() is
// actually pointed at (see the harvest calls below). Each yields ONE reasonText — the recorded
// text, kept verbatim for quoting.
//
// `status_overridden` stays in the set: it fires if such a line is ever RECORDED in a gate log,
// which is where a doc-driven run would write it. It does NOT mean the memory store is read -
// see the header. A matcher whose name implies a source nobody harvests is what made this look
// satisfied for a whole review round.
const MATCHERS = [
  { name: "routed_back_comment", re: /<!--\s*routed back:\s*(.+?)\s*-->/ },
  { name: "recorded_reason", re: /^\s*[-*|>]?\s*(?:reason|route_back_reason)\s*[:=]\s*["']?(.+?)["']?\s*\|?\s*$/i },
  { name: "status_overridden", re: /status_overridden\b.*?\breason\s*[:=]\s*["']?(.+?)["']?\s*$/ },
];

const refusals = [];
// CONFINE -> EXISTS -> TRACKED AT HEAD -> only then read. Every refusal names its reason and
// is counted; none is ever a silent skip. An untracked file on disk cannot be a repo's evidence.
const readable = (rel) => {
  const conf = relUnderRoot(root, rel);
  if (conf === null || !(conf === "docs/tasks" || conf.startsWith("docs/tasks/"))) {
    refusals.push({ path: rel, why: "escapes the evidence window (the window is confined under docs/tasks/**) — REFUSED, not read" });
    return false;
  }
  if (!existsSync(join(root, conf))) { refusals.push({ path: conf, why: "resolves nowhere — REFUSED, not read" }); return false; }
  const t = sh("git", ["ls-tree", "HEAD", "--", conf], root);
  if (!(t.status === 0 && t.stdout.trim() !== "")) {
    refusals.push({ path: conf, why: "not tracked at HEAD — REFUSED, not read (an untracked file on disk cannot be a run's evidence)" });
    return false;
  }
  return true;
};

const rows = [];
const harvest = (rel, kind, taskId) => {
  if (!readable(rel)) return;
  let text = "";
  try { text = readFileSync(join(root, rel), "utf8"); } catch (e) { refusals.push({ path: rel, why: `unreadable: ${e.code || "error"} — REFUSED, not read` }); return; }
  const lines = text.split("\n");
  for (let i = 0; i < lines.length; i++) {
    for (const m of MATCHERS) {
      const hit = lines[i].match(m.re);
      if (!hit) continue;
      const reason = hit[1].trim();
      if (!reason) continue;
      const named = KNOWN_SKILLS.filter((s) => new RegExp(`(^|[^a-z0-9-])${s}([^a-z0-9-]|$)`).test(reason));
      rows.push({
        id: `EV-${sha8(`${rel}:${i + 1}`)}`,
        kind, task: taskId, source: rel, line: i + 1,
        matcher: m.name,
        signal: codeOf(reason),
        // Exactly one existing skill named -> attributed. Zero or many -> unattributable, and
        // it stays that way. Guessing which skill a prose reason blames is precisely the
        // model's-opinion-of-itself that the two-gate design exists not to trust.
        skill: named.length === 1 ? named[0] : null,
        ambiguous: named.length > 1 ? named.slice() : null,
        quote: reason,   // VERBATIM. Never interpolated into a command; never executed.
      });
      break;   // one row per line: the first matcher that fits wins
    }
  }
};

// signal = the recorded reason's leading CODE — the taxonomy ship-tasks already writes
// ("trace-004: ...", "awh-gate: ...", "circuit_breaker_5_consecutive_test_failures").
// Normalised so two runs spell the same failure the same way. Parentheticals are dropped:
// they carry the instance, not the class.
function codeOf(reason) {
  const i = reason.indexOf(":");
  let code = (i > 0 && i <= 40) ? reason.slice(0, i) : reason;
  code = code.replace(/\([^)]*\)/g, " ").toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "");
  return code.slice(0, 60);
}

if (!KNOWN_SKILLS.length) notes.push("no modules/skill/*/SKILL.md found — every row is unattributable, so no proposal can name a target skill");
const backlog = "docs/tasks/BACKLOG.md";
if (existsSync(join(root, backlog))) harvest(backlog, "route_back", null);
else notes.push("no docs/tasks/BACKLOG.md — route-back comment cells are not among this window's sources");
for (const t of window) {
  for (const n of lsDir(t.dir).sort()) {
    if (/^gate-log.*\.md$/.test(n)) harvest(`${t.rel}/${n}`, "gate_log", t.id);
    else if (/^reconcile.*\.md$/.test(n)) harvest(`${t.rel}/${n}`, "reconcile_report", t.id);
  }
}

// ── clustering (§1.3): >= 2 INDEPENDENT rows, or it is an anecdote ──────────
// Independence is counted in distinct SOURCE FILES, not raw lines: two quotes from one gate
// log are one observation of one run written twice. A pattern that never left a single file
// has not recurred — it has been restated.
const clusters = new Map();
for (const r of rows) {
  if (!r.skill || !r.signal) continue;   // unattributable rows are reported, never clustered
  const key = `${r.skill}::${r.signal}`;
  if (!clusters.has(key)) clusters.set(key, { key, skill: r.skill, signal: r.signal, rows: [], sources: new Set() });
  const c = clusters.get(key);
  c.rows.push(r); c.sources.add(r.source);
}
const qualifying = [...clusters.values()]
  .filter((c) => c.sources.size >= 2)
  // highest-evidence first (§2 AC 3): independent sources, then total rows, then the key —
  // the last is a determinism tiebreak, so ties never depend on filesystem order.
  .sort((a, b) => b.sources.size - a.sources.size || b.rows.length - a.rows.length || a.key.localeCompare(b.key));

const CAP = 3;
const proposals = qualifying.slice(0, CAP).map((c) => ({
  artefact: "skill-amendment@1",
  id: `SA-${sha8(c.key)}`,
  status: "draft",              // §1.5 — the only status this tool ever writes
  target_skill: c.skill,
  target_passage: "<model-drafted: name the passage of the target skill this evidence indicts>",
  signal: c.signal,
  occurrences: c.rows.length,
  independent_sources: c.sources.size,
  evidence: c.rows.map((r) => ({ id: r.id, kind: r.kind, task: r.task, source: r.source, line: r.line, quote: r.quote })),
  proposed_change: "<model-drafted: what the passage should say instead, and why this evidence requires it>",
}));

const unattributable = rows.filter((r) => !r.skill);
const report = {
  artefact: "improvement-window@1",
  window: { requested: WINDOW, completed_tasks_read: window.length, tasks: window.map((t) => t.id) },
  evidence: { rows: rows.length, attributed: rows.length - unattributable.length, unattributable: unattributable.length },
  clusters: clusters.size,
  qualifying: qualifying.length,
  cap: CAP,
  proposals,
  refusals,
  notes,
};

// ── §1.6: a clean window emits NOTHING ──────────────────────────────────────
// Not an empty proposal, not a placeholder, not a padded third. The --out file is not
// created at all — "it wrote a file that says it found nothing" and "it found nothing" are
// different claims, and only the second one is true.
if (proposals.length === 0) {
  report.verdict = "no amendment proposed";
  if (asJson) console.log(JSON.stringify(report, null, 2));
  else {
    console.log(render(report));
    console.log("\nworkflow-improve: no amendment proposed — no pattern in this window cleared the two-independent-rows floor (§1.3). Nothing written.");
  }
  process.exit(refusals.length ? 3 : 0);
}

report.verdict = `${proposals.length} proposal(s)`;
const doc = render(report);
if (asJson) console.log(JSON.stringify(report, null, 2));
else console.log(doc);
if (outRel !== null) {
  mkdirSync(dirname(join(root, outRel)), { recursive: true });   // outRel was confined above
  writeFileSync(join(root, outRel), doc.endsWith("\n") ? doc : doc + "\n");
  console.error(`workflow-improve: proposals written to ${outRel} (status: draft — hand them to create-tasks; this tool never lands a task)`);
}
process.exit(refusals.length ? 3 : 0);

function render(r) {
  const L = [];
  L.push(`# improvement-window@1 — ${r.window.completed_tasks_read} completed task(s) read (window ${r.window.requested})`);
  L.push("");
  L.push(`evidence rows: ${r.evidence.rows} (attributed ${r.evidence.attributed}, unattributable ${r.evidence.unattributable})`);
  L.push(`clusters: ${r.clusters} · qualifying (>= 2 independent sources): ${r.qualifying} · cap: ${r.cap}`);
  for (const n of r.notes) L.push(`note: ${n}`);
  for (const f of r.refusals) L.push(`REFUSED ${f.path}: ${f.why}`);
  L.push("");
  if (!r.proposals.length) { L.push("## no amendment proposed"); return L.join("\n"); }
  L.push(`## ${r.proposals.length} skill-amendment@1 proposal(s), highest-evidence first`);
  for (const p of r.proposals) {
    L.push("");
    L.push(`### ${p.id}  ·  status: ${p.status}`);
    L.push(`- target_skill: ${p.target_skill}`);
    L.push(`- target_passage: ${p.target_passage}`);
    L.push(`- signal: ${p.signal}`);
    L.push(`- occurrences: ${p.occurrences} row(s) across ${p.independent_sources} independent source(s)`);
    L.push(`- evidence:`);
    for (const e of p.evidence) L.push(`  - [${e.id}] ${e.source}:${e.line} (${e.kind}${e.task ? `, ${e.task}` : ""})\n    > ${e.quote}`);
    L.push(`- proposed_change: ${p.proposed_change}`);
  }
  return L.join("\n");
}
