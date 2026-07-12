#!/usr/bin/env node
// tools/docs-site/render-status-hub.mjs - FR-DOCS-006 (supersedes render-roadmap.mjs / FR-DOCS-003).
// One page answers "where is the project": command deck + Roadmap | Backlog | Changelog tabs.
// Inputs (exactly three): FR frontmatter, CHANGELOG.md version sections, VERSION.
// Node stdlib only; deterministic stamp (VERSION + commit, no wall clock); honest failures.
// Also emits the roadmap.html redirect stub (bookmarks stay alive).
// Usage: node render-status-hub.mjs [repoRoot] [outDir]
import { readFileSync, writeFileSync, readdirSync, mkdirSync, existsSync } from 'node:fs';
import { join, resolve, dirname, basename } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(process.argv[2] || resolve(__dirname, '..', '..'));
const OUT = resolve(process.argv[3] || join(ROOT, 'dist', 'website'));
// The page belongs to the repo it renders - title it after the project, never "CyberOS"
// (except CyberOS itself). Override with CYBEROS_PROJECT.
let NAME = process.env.CYBEROS_PROJECT || basename(ROOT);
if (NAME.toLowerCase() === 'cyberos') NAME = 'CyberOS';
// CYBEROS_PAGE_ASSETS=1: emit reference/assets/{status.css,favicon.svg} and link them from the
// page instead of inlining styles - the deployed form is a folder (index.html + assets/).
const ASSETS = process.env.CYBEROS_PAGE_ASSETS === '1';
const STATUSES = ['draft','ready_to_implement','implementing','ready_to_review','reviewing',
                  'ready_to_test','testing','done','on_hold','closed'];
const esc = s => String(s ?? '').replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');

function gitCommit(root) {
  try {
    const head = readFileSync(join(root, '.git', 'HEAD'), 'utf-8').trim();
    if (!head.startsWith('ref:')) return head.slice(0, 12);
    const ref = head.slice(4).trim();
    const refFile = join(root, '.git', ref);
    if (existsSync(refFile)) return readFileSync(refFile, 'utf-8').trim().slice(0, 12);
    const packed = readFileSync(join(root, '.git', 'packed-refs'), 'utf-8');
    for (const l of packed.split('\n')) if (l.endsWith(' ' + ref)) return l.slice(0, 12);
  } catch {}
  return 'unknown';
}
function frontmatter(text, file) {
  if (!text.startsWith('---')) throw new Error(`status-hub: unparseable frontmatter (no opening fence) in ${file}`);
  const end = text.indexOf('\n---', 3);
  if (end === -1) throw new Error(`status-hub: unparseable frontmatter (no closing fence) in ${file}`);
  const meta = {};
  for (const line of text.slice(3, end).split('\n')) {
    const m = line.match(/^([a-z_]+):\s*(.*)$/);
    if (m) {
      let v = m[2];
      const q = v.match(/^(["'])(.*)\1\s*(#.*)?$/);
      v = q ? q[2] : v.replace(/\s+#.*$/, '');
      meta[m[1]] = v.trim();
    }
  }
  return meta;
}

// ---- one corpus object drives deck + every tab (FR-DOCS-006 §10 #2) ------------------
const FR_ROOT = join(ROOT, 'docs', 'feature-requests');
const LENIENT = process.env.CYBEROS_HUB_LENIENT === '1';
const frs = []; const invalid = [];
for (const mod of readdirSync(FR_ROOT, { withFileTypes: true }).sort((a,b)=>a.name.localeCompare(b.name))) {
  if (!mod.isDirectory() || mod.name.startsWith('_') || mod.name.startsWith('.')) continue;
  for (const d of readdirSync(join(FR_ROOT, mod.name), { withFileTypes: true }).sort((a,b)=>a.name.localeCompare(b.name))) {
    if (!d.isDirectory() || !d.name.startsWith('FR-')) continue;
    const p = join(FR_ROOT, mod.name, d.name, 'spec.md');
    if (!existsSync(p)) continue;
    let meta;
    try { meta = frontmatter(readFileSync(p, 'utf-8'), p); }
    catch (e) {
      if (!LENIENT) throw e;                       // strict mode keeps the honest-failure contract
      console.error(`status-hub: WARN ${e.message} (lenient - spec skipped; add frontmatter to include it)`);
      continue;
    }
    const fr = { id: meta.id || meta.fr_id || d.name, stem: d.name, title: meta.title || '(untitled)',
                 module: (meta.module || mod.name).toLowerCase(), dirModule: mod.name,
                 cls: meta.class || 'product', priority: meta.priority || '', status: meta.status || '(none)' };
    if (!STATUSES.includes(fr.status)) { invalid.push(fr); console.error(`status-hub: WARN invalid status '${fr.status}' on ${fr.id}`); }
    frs.push(fr);
  }
}
frs.sort((a, b) => a.id.localeCompare(b.id));

const clText = existsSync(join(ROOT, 'CHANGELOG.md')) ? readFileSync(join(ROOT, 'CHANGELOG.md'), 'utf-8')
  : (LENIENT ? '' : (() => { throw new Error('status-hub: CHANGELOG.md missing'); })());
// accepted section shapes: "## [X.Y.Z] - date", "## [X.Y.Z] — date" (en/em dash),
// "## vX.Y.Z (date)", "## X.Y.Z", and date-sectioned logs "## [YYYY-MM-DD]" -
// real-world changelogs vary on all three axes.
const releases = []; const clRe = /^## \[?v?(\d+\.\d+\.\d+|\d{4}-\d{2}-\d{2})\]?(?:\s*[-–—(]\s*(.*?)\)?\s*)?$/gm;
let m; const marks = [];
while ((m = clRe.exec(clText)) !== null) marks.push({ version: m[1], date: (m[2]||'').trim(), at: m.index, end: clRe.lastIndex });
marks.forEach((mk, i) => {
  const body = clText.slice(mk.end, i + 1 < marks.length ? marks[i+1].at : clText.length);
  releases.push({ version: mk.version, date: mk.date,
                  lines: body.split('\n').map(s => s.trim()).filter(s => s && !s.startsWith('## ')) });
});
if (releases.length === 0 && !LENIENT) { console.error('status-hub: ERROR zero version sections parsed from CHANGELOG.md'); process.exit(1); }
if (releases.length === 0 && LENIENT) console.error('status-hub: WARN no CHANGELOG version sections (lenient mode - timeline empty)');

const VERSION = existsSync(join(ROOT, 'VERSION')) ? readFileSync(join(ROOT, 'VERSION'), 'utf-8').trim() : (LENIENT ? 'unversioned' : (() => { throw new Error('status-hub: VERSION missing'); })());
const COMMIT = gitCommit(ROOT);
const modules = [...new Set(frs.map(f => f.module))].sort();
const vlabel = v => /^\d{4}-\d{2}-\d{2}$/.test(v) ? esc(v) : 'v' + esc(v);   // date sections carry no 'v'
const byStatus = Object.fromEntries(STATUSES.map(s => [s, frs.filter(f => f.status === s)]));

// ---- fragments (v2 UI - FR-DOCS-007; reference: operator-supplied roadmap.html) ----------
const pageFor = f => {
  const rel = join('frs', f.dirModule, f.stem, 'index.html');
  return existsSync(join(OUT, rel)) ? `../${rel}` : null;
};
const ACTIVE = ['ready_to_implement','implementing','ready_to_review','reviewing','ready_to_test','testing'];
const chipClass = st => st === 'done' || st === 'closed' ? 'done' : ACTIVE.includes(st) ? 'active' : st === 'on_hold' ? 'hold' : 'todo';
const chip = f => {
  const h = pageFor(f);
  const short = esc(f.id.replace(/^FR-/, ''));
  const body = `<span class="chip ${chipClass(f.status)}" data-status="${esc(f.status)}" title="${esc(f.id)} — ${esc(f.title)} [${esc(f.status)}]">${short}</span>`;
  return h ? `<a class="chip-link" href="${h}">${body}</a>` : body;
};

// deck: segmented overall bar + count chips
const doneN = frs.filter(f => f.status === 'done' || f.status === 'closed').length;
const activeN = frs.filter(f => ACTIVE.includes(f.status)).length;
const holdN = byStatus['on_hold'].length;
const todoN = frs.length - doneN - activeN - holdN;
const pct = n => frs.length ? (100 * n / frs.length).toFixed(1) : 0;
const deck = `
<h2>Overall progress (${frs.length} feature requests · ${modules.length} modules)</h2>
<div class="bar">
  <i class="seg-done" style="width:${pct(doneN)}%" title="done ${doneN}"></i>
  <i class="seg-active" style="width:${pct(activeN)}%" title="in flight ${activeN}"></i>
  <i class="seg-hold" style="width:${pct(holdN)}%" title="on hold ${holdN}"></i>
</div>
<div class="counts">
  <span><b>${doneN}</b> done <span class="muted">(${pct(doneN)}%)</span></span>
  <span><b>${activeN}</b> in flight</span>
  <span><b>${todoN}</b> draft</span>
  <span><b>${holdN}</b> on hold</span>
  <span><b>${releases.length}</b> releases</span>
  <span class="muted">latest ${releases.length ? vlabel(releases[0].version) + (releases[0].date ? ' · ' + esc(releases[0].date) : '') : '-'}</span>
</div>`;

// callout: what is moving right now (generated, never hand-written)
const moving = frs.filter(f => ['implementing','reviewing','ready_to_review','ready_to_test','testing'].includes(f.status));
const nowHtml = moving.length ? `
<section class="callout">
  <h3>Now shipping (${moving.length})</h3>
  <div class="frs">${moving.slice(0, 40).map(chip).join(' ')}${moving.length > 40 ? ` <span class="muted">+${moving.length - 40} more</span>` : ''}</div>
</section>` : '';

const legendHtml = `
<div class="legend">
  <span><i class="dot seg-done"></i> done / closed</span>
  <span><i class="dot seg-active"></i> in flight (ready_to_implement → testing)</span>
  <span><i class="dot" style="background:var(--cs-color-border-default)"></i> draft</span>
  <span><i class="dot seg-hold"></i> on hold</span>
  <span class="muted">chips link to each FR's page · hover for title + status</span>
</div>`;

// roadmap tab: one card per module (minibar + chips), then the rollup table
const moduleCards = modules.map(mod => {
  const rows = frs.filter(f => f.module === mod);
  const d = rows.filter(f => f.status === 'done' || f.status === 'closed').length;
  const a = rows.filter(f => ACTIVE.includes(f.status)).length;
  const h = rows.filter(f => f.status === 'on_hold').length;
  const p = n => rows.length ? (100 * n / rows.length).toFixed(1) : 0;
  const activeCls = a ? ' active' : '';
  return `<section class="epic${activeCls}" data-module="${esc(mod)}">
  <span class="pct">${p(d)}% done</span>
  <h3><span class="id">${esc(mod)}</span> <span class="muted">· ${rows.length} FRs</span></h3>
  <div class="minibar"><i class="seg-done" style="width:${p(d)}%"></i><i class="seg-active" style="width:${p(a)}%"></i><i class="seg-hold" style="width:${p(h)}%"></i></div>
  <div class="frs">${rows.map(chip).join(' ')}</div>
</section>`;
}).join('\n');
const rollup = modules.map(mod => {
  const rows = frs.filter(f => f.module === mod);
  return `<tr><td class="code">${esc(mod)}</td><td>${rows.length}</td>${STATUSES.map(s => `<td>${rows.filter(r => r.status === s).length || ''}</td>`).join('')}</tr>`;
}).join('\n');
const roadmapTab = `
<div class="grid">
${moduleCards}
${invalid.length ? `<section class="epic"><h3><span class="id">invalid status</span></h3><div class="frs">${invalid.map(chip).join(' ')}</div><p class="hub-note">These FRs carry a status outside the 10-value enum - the corpus data-quality monitor.</p></section>` : ''}
</div>
<h2 class="section-h">Module rollups</h2>
<table><thead><tr><th>module</th><th>total</th>${STATUSES.map(s => `<th>${esc(s)}</th>`).join('')}</tr></thead><tbody>
${rollup}
</tbody></table>`;

const opt = v => `<option value="${esc(v)}">${esc(v)}</option>`;
const priorities = [...new Set(frs.map(f => f.priority).filter(Boolean))].sort();
const backlogTab = `
<p class="hub-note">Generated view - FR frontmatter is the record of truth; agents write through BACKLOG.md.</p>
<div class="bk-facets">
  <select id="bk-module"><option value="">all modules</option>${modules.map(opt).join('')}</select>
  <select id="bk-class"><option value="">all classes</option>${opt('product')}${opt('improvement')}</select>
  <select id="bk-priority"><option value="">all priorities</option>${priorities.map(opt).join('')}</select>
  <select id="bk-status"><option value="">all statuses</option>${STATUSES.map(opt).join('')}</select>
</div>
<table id="bk-table"><thead><tr><th>id</th><th>title</th><th>module</th><th>class</th><th>priority</th><th>status</th></tr></thead><tbody>
${frs.map(f => `<tr data-module="${esc(f.module)}" data-class="${esc(f.cls)}" data-priority="${esc(f.priority)}" data-status="${esc(f.status)}"><td class="code">${pageFor(f) ? `<a href="${pageFor(f)}">${esc(f.id)}</a>` : esc(f.id)}</td><td>${esc(f.title)}</td><td>${esc(f.module)}</td><td>${esc(f.cls)}</td><td>${esc(f.priority)}</td><td><span class="chip ${chipClass(f.status)}" data-status="${esc(f.status)}">${esc(f.status)}</span></td></tr>`).join('\n')}
</tbody></table>
<script>
(function () {
  var sel = ['bk-module','bk-class','bk-priority','bk-status'].map(function (i) { return document.getElementById(i); });
  var keys = ['module','class','priority','status'];
  function apply() {
    document.querySelectorAll('#bk-table tbody tr').forEach(function (tr) {
      tr.hidden = !sel.every(function (s, i) { return !s.value || tr.dataset[keys[i]] === s.value; });
    });
  }
  sel.forEach(function (s) { s.addEventListener('change', apply); });
})();
</script>`;

// changelog tab: tick-circle release list (newest = "now")
const changelogTab = `<div class="phases">` + releases.map((r, i) => `
<article class="phase ${i === 0 ? 'now' : 'done'}">
  <span class="tick">${i === 0 ? '★' : '✓'}</span>
  <div><b>${vlabel(r.version)}${r.date ? ` <span class="muted">· ${esc(r.date)}</span>` : ''}</b>
  <span>${r.lines.map(l => esc(l)).join('<br>')}</span></div>
</article>`).join('\n') + `</div>`;

// ---- assemble through status-hub@1 ------------------------------------------------------
const TPL_DIR = process.env.CYBEROS_TEMPLATES
  || (existsSync(join(ROOT, 'modules', 'templates', 'html', 'status-hub.html')) ? join(ROOT, 'modules', 'templates', 'html') : join(__dirname, 'templates'));
const TOK_DIR = process.env.CYBEROS_TEMPLATES
  || (existsSync(join(ROOT, 'modules', 'templates', 'cds', 'tokens.css')) ? join(ROOT, 'modules', 'templates', 'cds') : join(__dirname, 'templates'));
const SHELL = readFileSync(join(TPL_DIR, 'status-hub.html'), 'utf-8');
const TOKENS = readFileSync(join(TOK_DIR, 'tokens.css'), 'utf-8');
const extra = `
.code { font-family:var(--cs-font-family-mono); }
.code a, .hub-note a, .chip-link { color:var(--cs-color-text-accent); text-decoration:none; }
.hub-note { color:var(--cs-color-text-muted); font-size:13px; }
.section-h { font-size:14px; text-transform:uppercase; letter-spacing:.08em; color:var(--cs-color-text-muted); margin:var(--cs-space-8) 0 var(--cs-space-3); }
.grid { display:grid; grid-template-columns:repeat(auto-fill, minmax(340px, 1fr)); gap:var(--cs-space-4); }
.epic { background:var(--cs-color-surface-panel); border:1px solid var(--cs-color-border-default); border-radius:12px; padding:var(--cs-space-4) var(--cs-space-4); }
.epic.active { border-color:var(--cs-color-brand-ochre); }
.epic h3 { margin:0; font-size:15px; line-height:var(--cs-font-lineHeight-heading); }
.epic .id { color:var(--cs-color-brand-umber); font-weight:700; }
.epic .pct { float:right; font-size:13px; color:var(--cs-color-text-muted); }
.minibar { height:6px; border-radius:var(--cs-radius-full); background:var(--cs-color-surface-raised); overflow:hidden; display:flex; margin:var(--cs-space-2) 0 var(--cs-space-3); }
.minibar i { display:block; height:100%; }
.frs { display:flex; flex-wrap:wrap; gap:5px; }
.chip { font-size:11.5px; font-weight:600; padding:3px 7px; border-radius:5px; background:var(--cs-color-border-default); color:var(--cs-color-text-accent); display:inline-block; }
.chip.done { background:var(--cs-color-semantic-success); color:var(--cs-color-text-inverse); }
.chip.active { background:var(--cs-color-brand-ochre); color:var(--cs-color-brand-umber); }
.chip.hold { background:var(--cs-color-text-muted); color:var(--cs-color-text-inverse); }
.muted { color:var(--cs-color-text-muted); font-weight:normal; font-size:13px; }
table { border-collapse:collapse; width:100%; font-size:13px; }
th, td { border:1px solid var(--cs-color-border-default); padding:var(--cs-space-1) var(--cs-space-2); text-align:left; }
th { background:var(--cs-color-surface-raised); }
.bk-facets { display:flex; gap:var(--cs-space-2); margin:var(--cs-space-3) 0; flex-wrap:wrap; }
.bk-facets select { background:var(--cs-color-surface-panel); color:var(--cs-color-text-primary); border:1px solid var(--cs-color-border-default); border-radius:var(--cs-radius-sm); padding:var(--cs-space-1) var(--cs-space-2); font-family:inherit; }
.phases { display:grid; gap:var(--cs-space-2); }
.phase { background:var(--cs-color-surface-panel); border:1px solid var(--cs-color-border-default); border-radius:10px; padding:var(--cs-space-3) var(--cs-space-4); display:flex; gap:var(--cs-space-3); align-items:flex-start; }
.phase .tick { width:22px; height:22px; border-radius:50%; flex:none; margin-top:2px; display:grid; place-items:center; font-size:12px; font-weight:700; color:var(--cs-color-text-inverse); background:var(--cs-color-semantic-success); }
.phase.now .tick { background:var(--cs-color-brand-ochre); color:var(--cs-color-brand-umber); }
.phase b { display:block; font-size:14px; }
.phase span { font-size:13px; color:var(--cs-color-text-muted); }
[hidden] { display:none !important; }`;

let page = SHELL.replace('/*{{slot:styles:html}}*/', ASSETS ? '@import url("assets/status.css");' : TOKENS + extra);
for (const [k, v] of Object.entries({
  'meta:html': `VERSION <span class="code">${esc(VERSION)}</span> · built from <span class="code">${esc(COMMIT)}</span> · ${frs.length} FRs · ${releases.length} releases`,
  'deck:html': deck, 'now:html': nowHtml, 'legend:html': legendHtml,
  'tab_roadmap:html': roadmapTab, 'tab_backlog:html': backlogTab, 'tab_changelog:html': changelogTab,
})) page = page.split(`{{slot:${k}}}`).join(v);
page = page.split('{{slot:title}}').join(`${esc(NAME)} status`);
page = page.split('{{slot:subtitle}}').join(`Where ${esc(NAME)} is and what is coming — generated from FR frontmatter, CHANGELOG, and VERSION`);
page = page.split('{{slot:footer}}').join(esc(`${NAME} — generated at ${VERSION} (${COMMIT}). Markdown is the record of truth; this page only renders it.`));
page = page.replace(/\{\{slot:[a-z_]+(:html)?\}\}/g, '');

mkdirSync(join(OUT, 'reference'), { recursive: true });
if (ASSETS) {
  page = page.replace('</head>', '<link rel="icon" type="image/svg+xml" href="assets/favicon.svg">\n</head>');
  const adir = join(OUT, 'reference', 'assets');
  mkdirSync(adir, { recursive: true });
  writeFileSync(join(adir, 'status.css'), TOKENS + extra);
  const tok = n => (TOKENS.match(new RegExp(`--cs-color-${n}:\\s*([^;]+);`)) || [])[1]?.trim();
  const umber = tok('brand-umber') || '#4a3222', ochre = tok('brand-ochre') || '#e0a458';
  const initial = esc((NAME[0] || 'C').toUpperCase());
  writeFileSync(join(adir, 'favicon.svg'),
`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
<rect width="64" height="64" rx="14" fill="${umber}"/>
<rect y="50" width="64" height="14" fill="${ochre}"/>
<text x="32" y="42" font-family="Georgia, 'Times New Roman', serif" font-size="34" font-weight="700" fill="#ffffff" text-anchor="middle">${initial}</text>
</svg>
`);
}
writeFileSync(join(OUT, 'reference', 'status.html'), page);
writeFileSync(join(OUT, 'reference', 'roadmap.html'), `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta http-equiv="refresh" content="0; url=status.html#roadmap">
<title>Roadmap moved</title></head>
<body><p>The roadmap lives in the <a href="status.html#roadmap">status hub</a> now (FR-DOCS-006).</p></body></html>
`);
const statusSummary = STATUSES.filter(s => byStatus[s].length).map(s => `${byStatus[s].length} ${s}`).join(', ');
console.log(`status-hub: ${frs.length} FRs (${statusSummary}), ${releases.length} releases, VERSION ${VERSION} (deck+3 tabs)`);
