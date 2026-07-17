#!/usr/bin/env node
// tools/docs-site/render-status-hub.mjs - TASK-DOCS-006 / TASK-DOCS-007 / TASK-IMP-074.
// ONE page answers "where is the project". Roadmap, Backlog and Changelog stopped being
// three tabs: they are three lenses (board / table / releases) over one filtered corpus,
// with a task detail drawer that carries the full spec.
// Inputs (exactly three, unchanged): task frontmatter, CHANGELOG.md version sections, VERSION.
// Node stdlib only; deterministic stamp (corpus fingerprint - no git, no wall clock); honest failures.
// Emits: reference/status.html, reference/data/task/<ID>.js (per-task spec chunks, lazy-loaded),
//        reference/assets/{status.css,status.js,favicon.svg} when CYBEROS_PAGE_ASSETS=1,
//        reference/roadmap.html (redirect stub - bookmarks stay alive).
// Usage: node render-status-hub.mjs [repoRoot] [outDir]
// Env:   CYBEROS_PROJECT     page title (default: basename of repoRoot)
//        CYBEROS_PAGE_ASSETS 1 = emit assets/ and link them instead of inlining
//        CYBEROS_HUB_LENIENT 1 = warn instead of failing on bad frontmatter / missing inputs
//        CYBEROS_STATUS_SPECS 0 = skip the per-task spec chunks (drawer links out instead)
import { readFileSync, writeFileSync, readdirSync, mkdirSync, existsSync, rmSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { join, resolve, dirname, basename } from 'node:path';
import { fileURLToPath } from 'node:url';
import { renderMarkdown } from './md.mjs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(process.argv[2] || resolve(__dirname, '..', '..'));
const OUT = resolve(process.argv[3] || join(ROOT, 'dist', 'website'));
let NAME = process.env.CYBEROS_PROJECT || basename(ROOT);
if (NAME.toLowerCase() === 'cyberos') NAME = 'CyberOS';
const ASSETS = process.env.CYBEROS_PAGE_ASSETS === '1';
const LENIENT = process.env.CYBEROS_HUB_LENIENT === '1';
const SPECS = process.env.CYBEROS_STATUS_SPECS !== '0';

const STATUSES = ['draft', 'ready_to_implement', 'implementing', 'ready_to_review', 'reviewing',
  'ready_to_test', 'testing', 'done', 'on_hold', 'closed'];
const ACTIVE = ['ready_to_implement', 'implementing', 'ready_to_review', 'reviewing', 'ready_to_test', 'testing'];
const esc = s => String(s ?? '').replace(/&/g, '&amp;').replace(/</g, '&lt;')
  .replace(/>/g, '&gt;').replace(/"/g, '&quot;');
const die = msg => { throw new Error(`status-hub: ${msg}`); };
const bucketOf = st => (st === 'done' || st === 'closed') ? 'done'
  : ACTIVE.includes(st) ? 'active' : st === 'on_hold' ? 'hold' : 'todo';

// ---- frontmatter: scalars, inline lists, block lists, block scalars -------------------
function scalar(v) {
  const s = String(v).trim();
  const q = s.match(/^(["'])([\s\S]*)\1\s*(?:#.*)?$/);
  if (q) return q[2].trim();
  return s.replace(/\s+#.*$/, '').trim();
}
function frontmatter(text, file) {
  if (!text.startsWith('---')) die(`unparseable frontmatter (no opening fence) in ${file}`);
  const end = text.indexOf('\n---', 3);
  if (end === -1) die(`unparseable frontmatter (no closing fence) in ${file}`);
  const meta = {};
  const lines = text.slice(3, end).split('\n');
  let key = null;      // the key a block list / block scalar is accumulating into
  let fold = null;     // '>' folded or '|' literal, when a block scalar is open
  for (const raw of lines) {
    if (fold && /^\s+\S/.test(raw)) {
      meta[key] = (meta[key] ? meta[key] + (fold === '|' ? '\n' : ' ') : '') + raw.trim();
      continue;
    }
    fold = null;
    const item = raw.match(/^\s*-\s+(.*)$/);
    if (item && key !== null && Array.isArray(meta[key])) { meta[key].push(scalar(item[1])); continue; }
    const kv = raw.match(/^([A-Za-z_][\w-]*):\s*(.*)$/);
    if (!kv) continue;
    key = kv[1];
    const v = kv[2].trim();
    if (v === '') { meta[key] = []; continue; }                     // block list (or empty scalar)
    if (/^[|>][-+]?$/.test(v)) { fold = v[0]; meta[key] = ''; continue; }   // block scalar
    if (/^\[.*\]$/.test(v)) {                                       // inline list
      meta[key] = v.slice(1, -1).split(',').map(scalar).filter(Boolean);
      key = null;
      continue;
    }
    meta[key] = scalar(v);
    key = null;
  }
  const body = text.slice(end + 4).replace(/^[ \t]*\r?\n/, '');
  return { meta, body };
}
const str = v => (Array.isArray(v) ? '' : String(v ?? '')).trim();
const list = v => Array.isArray(v) ? v : (str(v) && str(v) !== 'null' ? [str(v)] : []);
// Matches TASK-*. The constant is TASKID in the 2026-07-15
// rename; this body was not, so it matched an id shape that no longer exists anywhere.
// Result: `cited` came back [] for every release and the changelog -> task binding was
// dead on the live status page. A renamed name over an unrenamed body reads as correct.
const TASKID = /\bTASK-[A-Z][A-Z0-9]*-\d+\b/g;
const ids = s => (String(s ?? '').match(TASKID) || []);

// A one-paragraph summary: the first prose paragraph of the body (the §1 lead, in practice).
function summarize(body) {
  const lines = body.split('\n');
  let start = lines.findIndex(l => /^##\s*(§\s*)?1\b/.test(l));
  const out = [];
  for (let i = start + 1; i < lines.length; i++) {
    const l = lines[i].trim();
    if (!l || /^#{1,6}\s/.test(l) || /^[-*+]\s/.test(l) || /^\|/.test(l) || /^```/.test(l) || /^>/.test(l)) {
      if (out.length) break;
      continue;
    }
    out.push(l);
    if (out.join(' ').length > 200) break;
  }
  let s = out.join(' ')
    .replace(/!\[[^\]]*\]\([^)]*\)/g, '')
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
    .replace(/[*_`]/g, '')
    .trim();
  if (s.length > 220) s = s.slice(0, 217).replace(/\s+\S*$/, '') + '…';
  return s;
}

// ---- corpus: one object drives the deck, every lens, and the drawer -------------------
const TASK_ROOT = join(ROOT, 'docs', 'tasks');
if (!existsSync(TASK_ROOT)) die(`no docs/tasks under ${ROOT}`);

const tasks = [];
const invalid = [];
const specFiles = [];                  // { rel, abs } - every discovered spec.md; the stamp hashes these
const specs = new Map();               // id -> rendered spec HTML (chunked out below)
const safeId = id => /^[A-Za-z0-9._-]+$/.test(id);

for (const mod of readdirSync(TASK_ROOT, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
  if (!mod.isDirectory() || mod.name.startsWith('_') || mod.name.startsWith('.')) continue;
  for (const d of readdirSync(join(TASK_ROOT, mod.name), { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
    if (!d.isDirectory() || !d.name.startsWith('TASK-')) continue;
    const p = join(TASK_ROOT, mod.name, d.name, 'spec.md');
    if (!existsSync(p)) continue;
    specFiles.push({ rel: `docs/tasks/${mod.name}/${d.name}/spec.md`, abs: p });
    let parsed;
    try { parsed = frontmatter(readFileSync(p, 'utf-8'), p); }
    catch (e) {
      if (!LENIENT) throw e;                    // strict mode keeps the honest-failure contract
      console.error(`status-hub: WARN ${e.message} (lenient - spec skipped; add frontmatter to include it)`);
      continue;
    }
    const m = parsed.meta;
    const id = str(m.id) || str(m.task_id) || d.name;
    const task = {
      i: id, k: d.name, dm: mod.name,
      t: str(m.title) || '(untitled)',
      m: (str(m.module) || mod.name).toLowerCase(),
      // `type` is the schema field since the 2026-07-14 migration; `class` was folded
      // into it. Read type first, keep class as a transition fallback, and only then
      // fall back to the module-name guess.
      //
      // This read was `m.class` alone. The migration moved 509 specs to `type:` and left
      // zero carrying `class:`, so every task silently fell through to the guess below —
      // module `improvement` -> "improvement", everything else -> "product". The whole
      // feature/bug/chore distinction was invisible on the status page, and the class
      // facet was really a module facet wearing its name.
      c: str(m.type) || str(m.class) || (mod.name === 'improvement' ? 'improvement' : 'product'),
      p: str(m.priority), s: str(m.status) || '(none)',
      ph: str(m.phase), ms: str(m.milestone), sl: str(m.slice),
      o: str(m.owner), cr: str(m.created), sh: str(m.shipped),
      e: str(m.effort_hours), v: str(m.verify), r: str(m.risk_if_skipped),
      // TASK-IMP-108 §1.7: WHICH KIND of draft / ready_to_implement. Absent renders as 'unknown',
      // which is the truth for every task this run did not author - the page must not invent a
      // reason it was not told. `rb` lets the reader see thrash (routed_back_count > 0) without
      // opening the frontmatter, which is the whole complaint the field answers.
      dr: str(m.draft_reason), ev: str(m.entered_via), rb: Number(str(m.routed_back_count)) || 0,
      st: list(m.subtasks),
      // relations are resolved AFTER the corpus is known: a repo's ids are not always
      // TASK-shaped (strategem carries COV-001, API-READY...), so an id-regex alone would
      // silently drop those edges. Keep the raw values; resolve against the real corpus below.
      _d: list(m.depends_on), _b: list(m.blocks), _rl: list(m.related_tasks),
      d: [], b: [], rl: [],
      sm: summarize(parsed.body),
      pg: null, sp: 0,
    };
    if (task.sh === 'null') task.sh = '';
    if (!STATUSES.includes(task.s)) {
      invalid.push(task);
      console.error(`status-hub: WARN invalid status '${task.s}' on ${task.i}`);
    }
    if (SPECS && safeId(task.i) && parsed.body.trim()) {
      if (specs.has(task.i)) console.error(`status-hub: WARN duplicate task id ${task.i} - the later spec wins`);
      specs.set(task.i, renderMarkdown(parsed.body));
      task.sp = 1;
    }
    tasks.push(task);
  }
}
tasks.sort((a, b) => a.i.localeCompare(b.i));

// ---- draft staleness (TASK-IMP-108 §1.7) ----------------------------------------------
// 336 drafts sit indefinitely and the page reports a percentage against a denominator nobody
// believes. This groups them by REASON and age. It is a REPORT: it changes no task's status,
// closes nothing, and ages nothing automatically - the operator is the fix, this is the finding.
// Age is derived from `created` (a committed field), so a re-render of an unchanged corpus is
// byte-identical and TASK-IMP-082's fp- fingerprint still holds. `today` is NOT read from the
// clock for the same reason: a page that changes because time passed is a page that churns.
const draftStaleness = (() => {
  const drafts = tasks.filter(t => t.s === 'draft');
  const byReason = {};
  for (const t of drafts) {
    const r = t.dr || 'unknown';
    (byReason[r] ||= []).push({ id: t.i, created: t.cr || '' });
  }
  const out = Object.keys(byReason).sort().map(r => ({
    reason: r,
    count: byReason[r].length,
    oldest: byReason[r].map(x => x.created).filter(Boolean).sort()[0] || '',
  }));
  return { total: drafts.length, by_reason: out };
})();

if (!tasks.length && !LENIENT) die('zero task specs found under docs/tasks');

// per-task pages (rendered by render-task-pages.mjs) sit next to the hub in the website build
for (const f of tasks) {
  const rel = join('tasks', f.dm, f.k, 'index.html');
  if (existsSync(join(OUT, rel))) f.pg = `../${rel}`;
}
const byId = new Map(tasks.map(f => [f.i, f]));

// resolve relations now that every id is known: an entry counts as an edge when it names a real
// task in this corpus, or when it carries an TASK-shaped id (a forward reference to one not written
// yet). Prose entries ("none", "n/a", a sentence) resolve to nothing and are dropped - honestly.
const resolveRefs = raw => [...new Set(raw.flatMap(v => {   // NOT `resolve` - node:path owns that name
  const t = String(v).trim();
  if (byId.has(t)) return [t];   // an id of this corpus, whatever shape it has
  return ids(t);                 // else: TASK-shaped tokens inside the value (forward refs included)
}))];
for (const f of tasks) {
  f.d = resolveRefs(f._d); f.b = resolveRefs(f._b); f.rl = resolveRefs(f._rl);
  delete f._d; delete f._b; delete f._rl;
}

const modules = [...new Set(tasks.map(f => f.m))].sort();
const phases = [...new Set(tasks.map(f => f.ph).filter(Boolean))].sort();
const priorities = [...new Set(tasks.map(f => f.p).filter(Boolean))].sort();
const classes = [...new Set(tasks.map(f => f.c).filter(Boolean))].sort();
// the facet lists every status the corpus actually carries - including the invalid ones,
// so the data-quality problem is reachable, not hidden
const statusesSeen = [
  ...STATUSES.filter(s => tasks.some(f => f.s === s)),
  ...[...new Set(invalid.map(f => f.s))].sort(),
];

// ---- CHANGELOG -> releases, bound to the tasks they shipped ----------------------------
// The changelog stops being a wall of text: every task id it names becomes a chip that opens
// the task, and tasks whose `shipped:` date matches a release date are folded in as well
// (dashed chips - date-matched, not cited). Prose is kept, but collapsed behind the chips.
const clPath = join(ROOT, 'CHANGELOG.md');
if (!existsSync(clPath) && !LENIENT) die('CHANGELOG.md missing');
const clText = existsSync(clPath) ? readFileSync(clPath, 'utf-8') : '';

// section shapes seen in the wild: "## [X.Y.Z] - date", en/em dash, "## vX.Y.Z (date)",
// "## X.Y.Z", and date-sectioned logs "## [YYYY-MM-DD]".
const clRe = /^## \[?v?(\d+\.\d+\.\d+|\d{4}-\d{2}-\d{2})\]?(?:\s*[-–—(]\s*(.*?)\)?\s*)?$/gm;
const SEC_RE = /^(Added|Changed|Fixed|Removed|Security|Deprecated|Notes?)\s*:?\s*$/i;
const inlineHtml = md => renderMarkdown(md).replace(/^<p>([\s\S]*)<\/p>\s*$/, '$1').trim();
// linkify task ids inside already-rendered HTML without touching tags
const linkify = html => html.split(/(<[^>]+>)/).map(part => part.startsWith('<') ? part
  : part.replace(TASKID, id => byId.has(id)
    ? `<button class="chip ${bucketOf(byId.get(id).s)}" data-task="${esc(id)}">${esc(id)}</button>`
    : id)).join('');

const marks = [];
let mm;
while ((mm = clRe.exec(clText)) !== null) {
  marks.push({ version: mm[1], date: (mm[2] || '').trim(), at: mm.index, end: clRe.lastIndex });
}
const releases = marks.map((mk, i) => {
  const raw = clText.slice(mk.end, i + 1 < marks.length ? marks[i + 1].at : clText.length);
  const intro = [];
  const sec = [];
  let cur = null;
  let para = [];
  let item = null;
  const flushPara = () => { if (para.length) { intro.push(inlineHtml(para.join(' '))); para = []; } };
  const flushItem = () => {
    if (item === null) return;
    const html = linkify(inlineHtml(item));
    if (!cur) { cur = { h: 'Notes', items: [] }; sec.push(cur); }
    cur.items.push(html);
    item = null;
  };
  for (const line of raw.split('\n')) {
    const t = line.trim();
    if (!t) { flushItem(); flushPara(); continue; }
    if (SEC_RE.test(t)) { flushItem(); flushPara(); cur = { h: t.replace(/:$/, ''), items: [] }; sec.push(cur); continue; }
    const bullet = t.match(/^[-*+]\s+(.*)$/);
    if (bullet) { flushItem(); flushPara(); item = bullet[1]; continue; }
    if (item !== null) { item += ' ' + t; continue; }   // wrapped bullet continuation
    para.push(t);
  }
  flushItem();
  flushPara();
  const cited = [...new Set(ids(raw))].sort();
  // "TASK-IMP-071/072" - the slash run is a real habit in this changelog; expand it
  for (const run of raw.match(/\bTASK-[A-Z][A-Z0-9]*-\d+(?:\/\d+)+/g) || []) {
    const [head, ...rest] = run.split('/');
    const prefix = head.replace(/\d+$/, '');
    for (const n of rest) if (!cited.includes(prefix + n)) cited.push(prefix + n);
  }
  cited.sort();
  const vlabel = /^\d{4}-\d{2}-\d{2}$/.test(mk.version) ? mk.version : 'v' + mk.version;
  return { v: mk.version, vl: vlabel, d: mk.date, intro, sec: sec.filter(s => s.items.length), cited, dated: [] };
});
if (!releases.length) {
  if (!LENIENT) { console.error('status-hub: ERROR zero version sections parsed from CHANGELOG.md'); process.exit(1); }
  console.error('status-hub: WARN no CHANGELOG version sections (lenient - the releases lens is empty)');
}

// date-matched tasks: bind to the newest release carrying that date, and never twice
const citedAll = new Set(releases.flatMap(r => r.cited));
const firstByDate = new Map();
releases.forEach((r, i) => { if (r.d && !firstByDate.has(r.d)) firstByDate.set(r.d, i); });
for (const f of tasks) {
  if (!f.sh || citedAll.has(f.i) || !firstByDate.has(f.sh)) continue;
  releases[firstByDate.get(f.sh)].dated.push(f.i);
}
releases.forEach(r => r.dated.sort());
// every task a release accounts for - so the "unreleased" card can name the ones no release does
const bound = [...new Set(releases.flatMap(r => [...r.cited, ...r.dated]))].sort();

// ---- deck, facets, no-JS fallback (server-rendered: the page is true before JS runs) --
const VERSION = existsSync(join(ROOT, 'VERSION'))
  ? readFileSync(join(ROOT, 'VERSION'), 'utf-8').trim()
  : (LENIENT ? 'unversioned' : die('VERSION missing'));
// The default stamp is a fingerprint of the render inputs, not a git sha: 'fp-' + the first
// 12 hex of sha256 over every task spec's raw bytes in bytewise-sorted repo-relative path
// order, then CHANGELOG.md, then VERSION, when present. A HEAD default self-chased: the page
// staged by the pre-commit hook carried the PARENT sha (the new commit did not exist yet),
// so every re-render differed by the stamp alone, and committing THAT armed the next diff.
// The page's own bytes are never an input, so render -> commit -> render is byte-stable,
// git checkout or not. CYBEROS_COMMIT still pins an explicit stamp when set and non-empty.
function corpusFingerprint() {
  const h = createHash('sha256');
  const files = [...specFiles].sort((a, b) => Buffer.compare(Buffer.from(a.rel), Buffer.from(b.rel)));
  for (const f of files) h.update(readFileSync(f.abs));   // per-file update - no concat buffer
  if (existsSync(clPath)) h.update(readFileSync(clPath));
  const vPath = join(ROOT, 'VERSION');
  if (existsSync(vPath)) h.update(readFileSync(vPath));
  return 'fp-' + h.digest('hex').slice(0, 12);
}
const COMMIT = process.env.CYBEROS_COMMIT || corpusFingerprint();

const count = b => tasks.filter(f => bucketOf(f.s) === b).length;
const doneN = count('done'), activeN = count('active'), holdN = count('hold'), todoN = count('todo');
const pct = n => tasks.length ? (100 * n / tasks.length).toFixed(1) : '0.0';
const moving = tasks.filter(f => bucketOf(f.s) === 'active');
const effortOpen = tasks.filter(f => bucketOf(f.s) !== 'done')
  .reduce((a, f) => a + (parseFloat(f.e) || 0), 0);

const kpi = (b, n, label) =>
  `<button class="kpi" type="button" data-bucket="${b}" title="Filter to ${label}"><b>${n}</b>` +
  `<span><i class="dot seg-${b}"></i>${label}</span></button>`;
const deck = `
<div class="panel">
  <h2>Overall progress · ${tasks.length} tasks · ${modules.length} modules</h2>
  <div class="kpis">
    ${kpi('done', doneN, 'done')}
    ${kpi('active', activeN, 'in flight')}
    ${kpi('todo', todoN, 'draft')}
    ${kpi('hold', holdN, 'on hold')}
    <div class="kpi"><b>${effortOpen ? Math.round(effortOpen) : '—'}</b><span>open effort (h)</span></div>
  </div>
  <div class="bar-seg">
    <i class="seg-done" style="width:${pct(doneN)}%" title="done ${doneN}"></i>
    <i class="seg-active" style="width:${pct(activeN)}%" title="in flight ${activeN}"></i>
    <i class="seg-hold" style="width:${pct(holdN)}%" title="on hold ${holdN}"></i>
  </div>
  <div class="legend">
    <span><i class="dot seg-done"></i> done / closed (${pct(doneN)}%)</span>
    <span><i class="dot seg-active"></i> in flight — ready_to_implement → testing</span>
    <span><i class="dot seg-todo"></i> draft</span>
    <span><i class="dot seg-hold"></i> on hold</span>
    <span class="muted">click a number to filter · click any task to open it</span>
  </div>
</div>
<div class="panel">
  <h2>Releases · ${releases.length}${releases.length ? ` · latest ${esc(releases[0].vl)}` : ''}</h2>
  ${releases.length ? `<div class="spark">${releases.slice(0, 12).reverse().map(r => {
    const n = r.cited.length + r.dated.length;
    const max = Math.max(1, ...releases.map(x => x.cited.length + x.dated.length));
    return `<a href="#timeline" title="${esc(r.vl)}${r.d ? ' · ' + esc(r.d) : ''} — ${n} tasks">` +
      `<b>${n}</b><i style="height:${Math.max(4, Math.round(46 * n / max))}px"></i>` +
      `<span>${esc(r.vl.replace(/^v/, ''))}</span></a>`;
  }).join('')}</div>
  <p class="relnote">tasks bound to each release — cited in the entry, or matched by <code>shipped:</code> date.</p>`
    : '<p class="relnote">No release sections parsed from CHANGELOG.md.</p>'}
</div>`;

const nowHtml = moving.length ? `
<section class="now">
  <h2>Now shipping (${moving.length})</h2>
  <div class="tasks">${moving.map(f =>
    `<button class="chip active" data-task="${esc(f.i)}" title="${esc(`${f.i} — ${f.t} [${f.s}]`)}">${esc(f.i.replace(/^TASK-/, ''))}</button>`
  ).join('')}</div>
</section>` : '';

const sel = (id, label, values) =>
  `<label class="facet">${esc(label)}<select id="f-${id}"><option value="">all</option>` +
  values.map(v => `<option value="${esc(v)}">${esc(v)}</option>`).join('') + '</select></label>';
const facets = [
  sel('m', 'module', modules),
  sel('s', 'status', statusesSeen),
  sel('p', 'priority', priorities),
  sel('c', 'class', classes),
  sel('ph', 'phase', phases),
  `<label class="facet">group by<select id="f-g">` +
  [['m', 'module'], ['ph', 'phase'], ['s', 'status'], ['p', 'priority'], ['c', 'class'], ['o', 'owner']]
    .map(([v, l]) => `<option value="${v}">${l}</option>`).join('') + '</select></label>',
].join('\n  ');

// No-JS: the same truth, statically. Every task in one table, every release in one list.
const nojs = `
<p class="relnote">JavaScript is off, so the lenses, filters and the task drawer are unavailable.
Everything below is the same corpus, rendered statically.</p>
<div class="tbl-wrap"><table><thead><tr>
<th>id</th><th>title</th><th>module</th><th>class</th><th>priority</th><th>phase</th><th>status</th>
</tr></thead><tbody>
${tasks.map(f => `<tr><td class="code">${f.pg ? `<a href="${esc(f.pg)}">${esc(f.i)}</a>` : esc(f.i)}</td>` +
  `<td>${esc(f.t)}</td><td>${esc(f.m)}</td><td>${esc(f.c)}</td><td>${esc(f.p)}</td>` +
  `<td>${esc(f.ph)}</td><td>${esc(f.s)}</td></tr>`).join('\n')}
</tbody></table></div>
<h3 class="groupq">Releases</h3>
${releases.map(r => `<article class="rel"><span class="tick">✓</span><div>
  <div class="rel-h"><b>${esc(r.vl)}</b>${r.d ? `<span class="muted">${esc(r.d)}</span>` : ''}</div>
  ${[...r.cited, ...r.dated].length ? `<p class="relnote">tasks: ${[...r.cited, ...r.dated].map(esc).join(', ')}</p>` : ''}
  ${r.sec.map(s => `<div class="rel-sec"><h4>${esc(s.h)}</h4><ul>${s.items.map(x => `<li>${x}</li>`).join('')}</ul></div>`).join('')}
</div></article>`).join('\n')}`;

// ---- the corpus the client runs on ----------------------------------------------------
const data = {
  project: NAME, version: VERSION, commit: COMMIT,
  statuses: STATUSES,
  // The client reads specDir out of the emitted JSON (status-app.js:312), so this
  // one string is the whole producer/consumer contract for the chunk path.
  specDir: 'data/task',
  // where spec.md lives *relative to this page* - empty when the markdown is not shipped
  // next to the output (the website build links the rendered task page instead).
  frBase: process.env.CYBEROS_TASK_BASE || '',
  tasks, releases, bound,
};
// empty scalars carry no information and cost ~25% of the payload - drop them.
// (d / b / rl / st stay: the client reads .length on them.)
const KEEP = new Set(['i', 'k', 'dm', 't', 'm', 's', 'd', 'b', 'rl', 'st']);
const compact = f => Object.fromEntries(Object.entries(f)
  .filter(([k, v]) => KEEP.has(k) || (Array.isArray(v) ? v.length : v !== '' && v !== null && v !== 0)));
const dataJson = JSON.stringify({ ...data, draft_staleness: draftStaleness, tasks: tasks.map(compact) }).replace(/</g, '\\u003c');

// ---- templates ------------------------------------------------------------------------
const tpl = (sub, name) => {
  const env = process.env.CYBEROS_TEMPLATES;
  for (const c of [
    env && join(env, name),
    join(ROOT, 'modules', 'templates', sub, name),
    join(__dirname, 'templates', name),
  ]) if (c && existsSync(c)) return readFileSync(c, 'utf-8');
  return die(`template not found: ${name} (set CYBEROS_TEMPLATES)`);
};
const SHELL = tpl('html', 'status-hub.html');
const CSS = tpl('cds', 'tokens.css') + '\n' + tpl('cds', 'status.css');
const APP = tpl('html', 'status-app.js');

// ---- write ----------------------------------------------------------------------------
const REF = join(OUT, 'reference');
mkdirSync(REF, { recursive: true });

let specBytes = 0;
const dataDir = join(REF, 'data', 'task');
rmSync(join(REF, 'data'), { recursive: true, force: true });   // stale chunks must not linger
if (specs.size) {
  mkdirSync(dataDir, { recursive: true });
  for (const [id, html] of [...specs].sort((a, b) => a[0].localeCompare(b[0]))) {
    const js = `window.CS_SPEC=window.CS_SPEC||{};window.CS_SPEC[${JSON.stringify(id)}]=${JSON.stringify(html)};\n`;
    specBytes += Buffer.byteLength(js);
    writeFileSync(join(dataDir, `${id}.js`), js);
  }
}

const initial = esc((NAME[0] || 'C').toUpperCase());
let page = SHELL;
for (const [k, v] of Object.entries({
  'title': `${esc(NAME)} status`,
  'initial': initial,
  'subtitle': `Where ${esc(NAME)} is and what is coming — generated from task frontmatter, CHANGELOG and VERSION`,
  'search_placeholder': `Search ${tasks.length} tasks — id, title, module, owner, phase…`,
  'meta:html': `VERSION <span class="code">${esc(VERSION)}</span> · built from <span class="code">${esc(COMMIT)}</span> · ${tasks.length} tasks · ${releases.length} releases`,
  'deck:html': deck,
  'now:html': nowHtml,
  'facets:html': facets,
  'nojs:html': nojs,
  'data:json': dataJson,
  'styles:html': ASSETS ? '<link rel="stylesheet" href="assets/status.css">' : `<style>\n${CSS}\n</style>`,
  'head:html': ASSETS ? '<link rel="icon" type="image/svg+xml" href="assets/favicon.svg">' : '',
  'script:html': ASSETS ? '<script defer src="assets/status.js"></script>' : `<script>\n${APP}\n</script>`,
  'footer': esc(`${NAME} — generated at ${VERSION} (${COMMIT}). Markdown is the record of truth; this page only renders it.`),
})) page = page.split(`{{slot:${k}}}`).join(v);
page = page.replace(/\{\{slot:[a-z_]+(:html|:json)?\}\}/g, '');
writeFileSync(join(REF, 'status.html'), page);

if (ASSETS) {
  const adir = join(REF, 'assets');
  mkdirSync(adir, { recursive: true });
  writeFileSync(join(adir, 'status.css'), CSS);
  writeFileSync(join(adir, 'status.js'), APP);
  const tok = n => (CSS.match(new RegExp(`--cs-color-${n}:\\s*([^;]+);`)) || [])[1]?.trim();
  const umber = tok('brand-umber') || '#45210E';
  const ochre = tok('brand-ochre') || '#F4BA17';
  writeFileSync(join(adir, 'favicon.svg'),
`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
<rect width="64" height="64" rx="14" fill="${umber}"/>
<rect y="50" width="64" height="14" fill="${ochre}"/>
<text x="32" y="42" font-family="Georgia, 'Times New Roman', serif" font-size="34" font-weight="700" fill="#ffffff" text-anchor="middle">${initial}</text>
</svg>
`);
}

// bookmarks stay alive: the three old tabs are the three lenses now (#roadmap -> board,
// #backlog -> table, #changelog -> releases; status-app.js maps the legacy hashes).
writeFileSync(join(REF, 'roadmap.html'), `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta http-equiv="refresh" content="0; url=status.html#roadmap">
<title>Roadmap moved</title></head>
<body><p>The roadmap is a lens of the <a href="status.html#roadmap">status hub</a> now (TASK-DOCS-006).</p></body></html>
`);

const summary = STATUSES.filter(s => tasks.some(f => f.s === s))
  .map(s => `${tasks.filter(f => f.s === s).length} ${s}`).join(', ');
console.log(`status-hub: ${tasks.length} tasks (${summary}), ${modules.length} modules, ${releases.length} releases, ` +
  `VERSION ${VERSION} - one page, three lenses` +
  (specs.size ? `, ${specs.size} spec chunks (${(specBytes / 1048576).toFixed(1)} MB)` : ', no spec chunks (CYBEROS_STATUS_SPECS=0)') +
  (invalid.length ? `, ${invalid.length} invalid status` : ''));
