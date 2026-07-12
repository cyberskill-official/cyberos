#!/usr/bin/env node
// tools/docs-site/render-roadmap.mjs - FR-DOCS-003.
// One generated page answering "where is the platform and what is coming", from exactly
// three inputs: FR frontmatter (docs/feature-requests/*/FR-*.md), CHANGELOG.md version
// sections, and VERSION. Node stdlib only. Deterministic: same inputs -> byte-identical
// output (header stamp = VERSION + commit, never wall-clock).
// Usage: node render-roadmap.mjs [repoRoot] [outDir]
import { readFileSync, writeFileSync, readdirSync, mkdirSync, existsSync } from 'node:fs';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(process.argv[2] || resolve(__dirname, '..', '..'));
const OUT_DIR = resolve(process.argv[3] || join(ROOT, 'dist', 'website'));
const STATUSES = ['draft','ready_to_implement','implementing','ready_to_review','reviewing',
                  'ready_to_test','testing','done','on_hold','closed']; // STATUS-REFERENCE §1 order
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
  if (!text.startsWith('---')) throw new Error(`roadmap: unparseable frontmatter (no opening fence) in ${file}`);
  const end = text.indexOf('\n---', 3);
  if (end === -1) throw new Error(`roadmap: unparseable frontmatter (no closing fence) in ${file}`);
  const meta = {};
  for (const line of text.slice(3, end).split('\n')) {
    const m = line.match(/^([a-z_]+):\s*(.*)$/);
    if (m) {
      let v = m[2];
      const q = v.match(/^(["'])(.*)\1\s*(#.*)?$/);          // quoted scalar (comment after close-quote ok)
      v = q ? q[2] : v.replace(/\s+#.*$/, '');                 // unquoted: strip trailing yaml comment
      meta[m[1]] = v.trim();
    }
  }
  return meta;
}

// ---- input 1: FR frontmatter --------------------------------------------------------
const FR_ROOT = join(ROOT, 'docs', 'feature-requests');
const frs = [];
const invalid = [];
for (const dirent of readdirSync(FR_ROOT, { withFileTypes: true }).sort((a,b)=>a.name.localeCompare(b.name))) {
  if (!dirent.isDirectory() || dirent.name.startsWith('_') || dirent.name.startsWith('.')) continue;
  const dir = join(FR_ROOT, dirent.name);
  for (const f of readdirSync(dir, { withFileTypes: true }).sort((a,b)=>a.name.localeCompare(b.name))) {
    if (!f.isDirectory() || !f.name.startsWith('FR-')) continue;   // FR-DOCS-004 folder-per-FR
    const p = join(dir, f.name, 'spec.md');
    if (!existsSync(p)) continue;
    const meta = frontmatter(readFileSync(p, 'utf-8'), p);
    const fr = {
      id: meta.id || meta.fr_id || f.name,
      title: meta.title || '(untitled)',
      module: (meta.module || dirent.name).toLowerCase(),
      cls: meta.class || 'product',
      status: meta.status || '(none)',
    };
    if (!STATUSES.includes(fr.status)) { invalid.push(fr); console.error(`roadmap: WARN invalid status '${fr.status}' on ${fr.id}`); }
    frs.push(fr);
  }
}
frs.sort((a, b) => a.id.localeCompare(b.id));

// ---- input 2: CHANGELOG version sections --------------------------------------------
const clText = readFileSync(join(ROOT, 'CHANGELOG.md'), 'utf-8');
const releases = [];
const clRe = /^## \[?(\d+\.\d+\.\d+)\]?(?:\s*-\s*(.*))?$/gm;
let m, marks = [];
while ((m = clRe.exec(clText)) !== null) marks.push({ version: m[1], date: (m[2]||'').trim(), at: m.index, end: clRe.lastIndex });
marks.forEach((mk, i) => {
  const body = clText.slice(mk.end, i + 1 < marks.length ? marks[i+1].at : clText.length);
  const lines = body.split('\n').map(s => s.trim()).filter(s => s && !s.startsWith('## '));
  releases.push({ version: mk.version, date: mk.date, lines });
});
if (releases.length === 0) { console.error('roadmap: ERROR zero version sections parsed from CHANGELOG.md (structure changed under the parser)'); process.exit(1); }

// ---- input 3: VERSION ----------------------------------------------------------------
const VERSION = readFileSync(join(ROOT, 'VERSION'), 'utf-8').trim();
const COMMIT = gitCommit(ROOT);

// ---- rollups --------------------------------------------------------------------------
const byStatus = Object.fromEntries(STATUSES.map(s => [s, frs.filter(f => f.status === s)]));
const modules = [...new Set(frs.map(f => f.module))].sort();
const rollup = modules.map(mod => {
  const rows = frs.filter(f => f.module === mod);
  const counts = STATUSES.map(s => rows.filter(r => r.status === s).length);
  return { mod, total: rows.length, counts,
           product: rows.filter(r => r.cls === 'product').length,
           improvement: rows.filter(r => r.cls === 'improvement').length };
});

// ---- render ---------------------------------------------------------------------------
const frRow = f => `<div class="fr-row" data-module="${esc(f.module)}" data-class="${esc(f.cls)}" data-status="${esc(f.status)}"><span class="code">${esc(f.id)}</span> ${esc(f.title)} <span class="badge">${esc(f.module)}</span>${f.cls === 'improvement' ? ' <span class="badge badge-imp">improvement</span>' : ''}</div>`;
const column = (s, rows) => `<section class="board-col" data-status-col="${esc(s)}"><h3>${esc(s)} <span class="count" data-count="${rows.length}">${rows.length}</span></h3>\n${rows.map(frRow).join('\n')}\n</section>`;
const timeline = releases.map(r => `<article class="release"><h3>v${esc(r.version)}${r.date ? ` <span class="muted">${esc(r.date)}</span>` : ''}</h3><ul>${r.lines.map(l => `<li>${esc(l)}</li>`).join('')}</ul></article>`).join('\n');
const rollupRows = rollup.map(r => `<tr data-module="${esc(r.mod)}"><td class="code">${esc(r.mod)}</td><td>${r.total}</td><td>${r.product}</td><td>${r.improvement}</td>${r.counts.map(c => `<td>${c || ''}</td>`).join('')}</tr>`).join('\n');
const opt = v => `<option value="${esc(v)}">${esc(v)}</option>`;

const html = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Roadmap — CyberOS</title>
<meta name="description" content="CyberOS roadmap: ${frs.length} FRs across ${modules.length} modules, ${releases.length} releases, VERSION ${esc(VERSION)}.">
<style>
  :root {
    --bg: #0b0d10; --panel: #14181d; --panel-2: #1a1f25; --text: #e6eaef;
    --muted: #8893a0; --border: #232a32; --accent: #4da3ff; --accent-2: #58d0a4;
  }
  body { margin:0; background:var(--bg); color:var(--text); font:15px/1.5 system-ui,-apple-system,sans-serif; padding:24px; }
  h1 { margin:0 0 4px; } h2 { margin:32px 0 12px; border-bottom:1px solid var(--border); padding-bottom:6px; }
  .stamp { color:var(--muted); margin-bottom:20px; }
  .code { font-family:ui-monospace,monospace; color:var(--accent); }
  .muted { color:var(--muted); font-weight:normal; font-size:13px; }
  .badge { display:inline-block; padding:0 6px; border:1px solid var(--border); border-radius:8px; font-size:11px; color:var(--muted); }
  .badge-imp { color:var(--accent-2); }
  .filters { display:flex; gap:12px; margin:12px 0 20px; flex-wrap:wrap; }
  .filters select { background:var(--panel-2); color:var(--text); border:1px solid var(--border); border-radius:6px; padding:4px 8px; }
  .board { display:grid; grid-template-columns:repeat(auto-fill,minmax(260px,1fr)); gap:12px; }
  .board-col { background:var(--panel); border:1px solid var(--border); border-radius:8px; padding:10px 12px; }
  .board-col h3 { margin:0 0 8px; font-size:13px; text-transform:uppercase; letter-spacing:.05em; color:var(--muted); }
  .board-col .count { color:var(--accent); }
  .fr-row { padding:3px 0; border-bottom:1px solid var(--border); font-size:13px; }
  .fr-row:last-child { border-bottom:0; }
  .release { background:var(--panel); border:1px solid var(--border); border-radius:8px; padding:10px 16px; margin-bottom:10px; }
  .release h3 { margin:0 0 6px; }
  table { border-collapse:collapse; width:100%; font-size:13px; }
  th,td { border:1px solid var(--border); padding:4px 8px; text-align:left; }
  th { background:var(--panel-2); color:var(--muted); font-weight:600; }
  [hidden] { display:none !important; }
</style>
</head>
<body>
<h1>CyberOS roadmap</h1>
<p class="stamp">VERSION <span class="code">${esc(VERSION)}</span> · built from <span class="code">${esc(COMMIT)}</span> · ${frs.length} FRs · ${releases.length} releases</p>

<h2>Release timeline</h2>
${timeline}

<h2>Pipeline board</h2>
<div class="filters">
  <select id="f-module"><option value="">all modules</option>${modules.map(opt).join('')}</select>
  <select id="f-class"><option value="">all classes</option>${opt('product')}${opt('improvement')}</select>
  <select id="f-status"><option value="">all statuses</option>${STATUSES.map(opt).join('')}</select>
</div>
<div class="board">
${STATUSES.map(s => column(s, byStatus[s])).join('\n')}
${invalid.length ? column('invalid', invalid) : ''}
</div>

<h2>Module rollups</h2>
<table>
<thead><tr><th>module</th><th>total</th><th>product</th><th>improvement</th>${STATUSES.map(s => `<th>${esc(s)}</th>`).join('')}</tr></thead>
<tbody>
${rollupRows}
</tbody>
</table>

<script>
(function () {
  var fm = document.getElementById('f-module'), fc = document.getElementById('f-class'), fs = document.getElementById('f-status');
  function apply() {
    var m = fm.value, c = fc.value, s = fs.value;
    document.querySelectorAll('.fr-row').forEach(function (r) {
      var show = (!m || r.dataset.module === m) && (!c || r.dataset.class === c) && (!s || r.dataset.status === s);
      r.hidden = !show;
    });
    document.querySelectorAll('.board-col').forEach(function (col) {
      var vis = col.querySelectorAll('.fr-row:not([hidden])').length;
      col.querySelector('.count').textContent = vis;
      col.hidden = (s && col.dataset.statusCol !== s && col.dataset.statusCol !== 'invalid') || (vis === 0 && s !== '' && col.dataset.statusCol !== s);
    });
    document.querySelectorAll('tbody tr[data-module]').forEach(function (tr) { tr.hidden = !!m && tr.dataset.module !== m; });
  }
  [fm, fc, fs].forEach(function (el) { el.addEventListener('change', apply); });
})();
</script>
</body>
</html>
`;
mkdirSync(join(OUT_DIR, 'reference'), { recursive: true });
writeFileSync(join(OUT_DIR, 'reference', 'roadmap.html'), html);
const statusSummary = STATUSES.filter(s => byStatus[s].length).map(s => `${byStatus[s].length} ${s}`).join(', ');
console.log(`roadmap: ${frs.length} FRs (${statusSummary}), ${releases.length} releases, VERSION ${VERSION}`);
