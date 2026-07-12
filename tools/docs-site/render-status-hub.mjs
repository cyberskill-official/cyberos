#!/usr/bin/env node
// tools/docs-site/render-status-hub.mjs - FR-DOCS-006 (supersedes render-roadmap.mjs / FR-DOCS-003).
// One page answers "where is the project": command deck + Roadmap | Backlog | Changelog tabs.
// Inputs (exactly three): FR frontmatter, CHANGELOG.md version sections, VERSION.
// Node stdlib only; deterministic stamp (VERSION + commit, no wall clock); honest failures.
// Also emits the roadmap.html redirect stub (bookmarks stay alive).
// Usage: node render-status-hub.mjs [repoRoot] [outDir]
import { readFileSync, writeFileSync, readdirSync, mkdirSync, existsSync } from 'node:fs';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(process.argv[2] || resolve(__dirname, '..', '..'));
const OUT = resolve(process.argv[3] || join(ROOT, 'dist', 'website'));
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
const frs = []; const invalid = [];
for (const mod of readdirSync(FR_ROOT, { withFileTypes: true }).sort((a,b)=>a.name.localeCompare(b.name))) {
  if (!mod.isDirectory() || mod.name.startsWith('_') || mod.name.startsWith('.')) continue;
  for (const d of readdirSync(join(FR_ROOT, mod.name), { withFileTypes: true }).sort((a,b)=>a.name.localeCompare(b.name))) {
    if (!d.isDirectory() || !d.name.startsWith('FR-')) continue;
    const p = join(FR_ROOT, mod.name, d.name, 'spec.md');
    if (!existsSync(p)) continue;
    const meta = frontmatter(readFileSync(p, 'utf-8'), p);
    const fr = { id: meta.id || meta.fr_id || d.name, stem: d.name, title: meta.title || '(untitled)',
                 module: (meta.module || mod.name).toLowerCase(), dirModule: mod.name,
                 cls: meta.class || 'product', priority: meta.priority || '', status: meta.status || '(none)' };
    if (!STATUSES.includes(fr.status)) { invalid.push(fr); console.error(`status-hub: WARN invalid status '${fr.status}' on ${fr.id}`); }
    frs.push(fr);
  }
}
frs.sort((a, b) => a.id.localeCompare(b.id));

const clText = readFileSync(join(ROOT, 'CHANGELOG.md'), 'utf-8');
const releases = []; const clRe = /^## \[?(\d+\.\d+\.\d+)\]?(?:\s*-\s*(.*))?$/gm;
let m; const marks = [];
while ((m = clRe.exec(clText)) !== null) marks.push({ version: m[1], date: (m[2]||'').trim(), at: m.index, end: clRe.lastIndex });
marks.forEach((mk, i) => {
  const body = clText.slice(mk.end, i + 1 < marks.length ? marks[i+1].at : clText.length);
  releases.push({ version: mk.version, date: mk.date,
                  lines: body.split('\n').map(s => s.trim()).filter(s => s && !s.startsWith('## ')) });
});
if (releases.length === 0) { console.error('status-hub: ERROR zero version sections parsed from CHANGELOG.md'); process.exit(1); }

const VERSION = readFileSync(join(ROOT, 'VERSION'), 'utf-8').trim();
const COMMIT = gitCommit(ROOT);
const modules = [...new Set(frs.map(f => f.module))].sort();
const byStatus = Object.fromEntries(STATUSES.map(s => [s, frs.filter(f => f.status === s)]));

// ---- fragments ------------------------------------------------------------------------
const pageFor = f => {
  const rel = join('frs', f.dirModule, f.stem, 'index.html');
  return existsSync(join(OUT, rel)) ? `../${rel}` : null;
};
const idCell = f => { const h = pageFor(f); return h ? `<a href="${h}">${esc(f.id)}</a>` : esc(f.id); };

const deck = [
  ['VERSION', esc(VERSION)], ['commit', esc(COMMIT)], ['FRs', frs.length], ['modules', modules.length],
  ['latest release', releases.length ? `v${esc(releases[0].version)}${releases[0].date ? ' · ' + esc(releases[0].date) : ''}` : '-'],
  ...STATUSES.filter(s => byStatus[s].length).map(s => [s.replace(/_/g, ' '), byStatus[s].length]),
].map(([l, v]) => `<div class="deck-stat"><div class="deck-num">${v}</div><div class="deck-label">${l}</div></div>`).join('\n');

const frRow = f => `<div class="fr-row" data-module="${esc(f.module)}" data-class="${esc(f.cls)}" data-status="${esc(f.status)}"><span class="code">${idCell(f)}</span> ${esc(f.title)} <span class="hub-badge">${esc(f.module)}</span>${f.cls === 'improvement' ? ' <span class="hub-badge imp">improvement</span>' : ''}</div>`;
const column = (s, rows) => `<section class="board-col" data-status-col="${esc(s)}"><h3>${esc(s)} <span class="count">${rows.length}</span></h3>\n${rows.map(frRow).join('\n')}</section>`;
const rollup = modules.map(mod => {
  const rows = frs.filter(f => f.module === mod);
  return `<tr><td class="code">${esc(mod)}</td><td>${rows.length}</td>${STATUSES.map(s => `<td>${rows.filter(r => r.status === s).length || ''}</td>`).join('')}</tr>`;
}).join('\n');
const roadmapTab = `
<div class="board">
${STATUSES.map(s => column(s, byStatus[s])).join('\n')}
${invalid.length ? column('invalid', invalid) : ''}
</div>
<h2>Module rollups</h2>
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
${frs.map(f => `<tr data-module="${esc(f.module)}" data-class="${esc(f.cls)}" data-priority="${esc(f.priority)}" data-status="${esc(f.status)}"><td class="code">${idCell(f)}</td><td>${esc(f.title)}</td><td>${esc(f.module)}</td><td>${esc(f.cls)}</td><td>${esc(f.priority)}</td><td>${esc(f.status)}</td></tr>`).join('\n')}
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

const changelogTab = releases.map(r => `<article class="release"><h3>v${esc(r.version)}${r.date ? ` <span class="muted">${esc(r.date)}</span>` : ''}</h3><ul>${r.lines.map(l => `<li>${esc(l)}</li>`).join('')}</ul></article>`).join('\n');

// ---- assemble through status-hub@1 ------------------------------------------------------
const SHELL = readFileSync(join(ROOT, 'modules', 'templates', 'html', 'status-hub.html'), 'utf-8');
const TOKENS = readFileSync(join(ROOT, 'modules', 'templates', 'cds', 'tokens.css'), 'utf-8');
const extra = `
.code { font-family:var(--cs-font-family-mono); }
.code a, .hub-note a { color:var(--cs-color-text-accent); }
.hub-badge { display:inline-block; padding:0 var(--cs-space-2); border:1px solid var(--cs-color-border-default); border-radius:var(--cs-radius-full); font-size:11px; color:var(--cs-color-text-muted); }
.hub-badge.imp { color:var(--cs-color-semantic-success); }
.hub-note { color:var(--cs-color-text-muted); font-size:13px; }
.board { display:grid; grid-template-columns:repeat(auto-fill,minmax(260px,1fr)); gap:var(--cs-space-3); }
.board-col { background:var(--cs-color-surface-panel); border:1px solid var(--cs-color-border-default); border-radius:var(--cs-radius-md); padding:var(--cs-space-3); }
.board-col h3 { margin:0 0 var(--cs-space-2); font-size:12px; text-transform:uppercase; letter-spacing:.05em; color:var(--cs-color-text-muted); }
.board-col .count { color:var(--cs-color-brand-umber); font-weight:600; }
.fr-row { padding:3px 0; border-bottom:1px solid var(--cs-color-border-default); font-size:13px; }
.fr-row:last-child { border-bottom:0; }
.release { background:var(--cs-color-surface-panel); border:1px solid var(--cs-color-border-default); border-radius:var(--cs-radius-md); padding:var(--cs-space-3) var(--cs-space-4); margin-bottom:var(--cs-space-3); }
.muted { color:var(--cs-color-text-muted); font-weight:normal; font-size:13px; }
table { border-collapse:collapse; width:100%; font-size:13px; }
th, td { border:1px solid var(--cs-color-border-default); padding:var(--cs-space-1) var(--cs-space-2); text-align:left; }
th { background:var(--cs-color-surface-raised); }
.bk-facets { display:flex; gap:var(--cs-space-2); margin:var(--cs-space-3) 0; flex-wrap:wrap; }
.bk-facets select { background:var(--cs-color-surface-panel); color:var(--cs-color-text-primary); border:1px solid var(--cs-color-border-default); border-radius:var(--cs-radius-sm); padding:var(--cs-space-1) var(--cs-space-2); font-family:inherit; }
[hidden] { display:none !important; }`;

let page = SHELL.replace('/*{{slot:styles:html}}*/', TOKENS + extra);
for (const [k, v] of Object.entries({
  'deck:html': deck, 'tab_roadmap:html': roadmapTab, 'tab_backlog:html': backlogTab,
  'tab_changelog:html': changelogTab,
})) page = page.split(`{{slot:${k}}}`).join(v);
page = page.split('{{slot:title}}').join('Status');
page = page.split('{{slot:footer}}').join(esc(`Generated from FR frontmatter + CHANGELOG.md + VERSION at ${VERSION} (${COMMIT}) - FR-DOCS-006.`));
page = page.replace(/\{\{slot:[a-z_]+(:html)?\}\}/g, '');

mkdirSync(join(OUT, 'reference'), { recursive: true });
writeFileSync(join(OUT, 'reference', 'status.html'), page);
writeFileSync(join(OUT, 'reference', 'roadmap.html'), `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta http-equiv="refresh" content="0; url=status.html#roadmap">
<title>Roadmap moved</title></head>
<body><p>The roadmap lives in the <a href="status.html#roadmap">status hub</a> now (FR-DOCS-006).</p></body></html>
`);
const statusSummary = STATUSES.filter(s => byStatus[s].length).map(s => `${byStatus[s].length} ${s}`).join(', ');
console.log(`status-hub: ${frs.length} FRs (${statusSummary}), ${releases.length} releases, VERSION ${VERSION} (deck+3 tabs)`);
