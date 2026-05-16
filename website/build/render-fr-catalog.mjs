#!/usr/bin/env node
// website/build/render-fr-catalog.mjs
// FR-DOCS-001 §1 #2 — render <section data-prerendered="true"> blocks
// from website/build/data/frs.json into website/docs/reference/fr-catalog.html.
// Pure-Node, no Handlebars dependency: this is the minimal first-cut renderer
// for the FR catalog; NFR + Risk renderers can lift the same pattern.

import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT  = resolve(__dirname, '..', '..');
const DATA_FILE  = join(__dirname, 'data', 'frs.json');
const OUT_DIR    = join(REPO_ROOT, 'website', 'docs', 'reference');
const OUT_FILE   = join(OUT_DIR, 'fr-catalog.html');
const REPORT_FILE = join(__dirname, 'last-build-report.json');

const esc = s => String(s ?? '')
  .replace(/&/g, '&amp;')
  .replace(/</g, '&lt;')
  .replace(/>/g, '&gt;')
  .replace(/"/g, '&quot;');

const badge = (label, value, cls) =>
  value ? `<span class="badge badge-${cls}">${esc(label)}: ${esc(value)}</span>` : '';

const list = arr =>
  Array.isArray(arr) && arr.length
    ? arr.map(x => {
        // detect "# placeholder" comments stripped during parse — render plain
        const txt = esc(x);
        return `<a href="#${txt}" class="fr-link">${txt}</a>`;
      }).join(', ')
    : '<em class="muted">none</em>';

function renderCard(fr) {
  return `      <article class="fr-card"
               id="${esc(fr.id)}"
               data-module="${esc(fr.module)}"
               data-priority="${esc(fr.priority)}"
               data-status="${esc(fr.status)}"
               data-phase="${esc(fr.phase)}">
        <header class="fr-card-header">
          <a href="#${esc(fr.id)}" class="fr-anchor">#</a>
          <h3 class="fr-id">${esc(fr.id)}</h3>
          <p class="fr-title">${esc(fr.title)}</p>
        </header>
        <div class="fr-badges">
          ${badge('module',   fr.module,   'module')}
          ${badge('priority', fr.priority, 'priority-' + fr.priority.toLowerCase())}
          ${badge('status',   fr.status,   'status-' + fr.status)}
          ${badge('verify',   fr.verify,   'verify')}
          ${badge('phase',    fr.phase,    'phase')}
          ${badge('slice',    fr.slice,    'slice')}
          ${badge('effort',   (fr.effort_hours ? fr.effort_hours + 'h' : ''), 'effort')}
        </div>
        <dl class="fr-meta">
          <dt>Owner</dt>      <dd>${esc(fr.owner) || '<em class="muted">unassigned</em>'}</dd>
          <dt>Milestone</dt>  <dd>${esc(fr.milestone) || '<em class="muted">—</em>'}</dd>
          <dt>Created</dt>    <dd>${esc(fr.created) || '<em class="muted">—</em>'}</dd>
          <dt>Shipped</dt>    <dd>${fr.shipped ? esc(fr.shipped) : '<em class="muted">not yet</em>'}</dd>
          <dt>Depends on</dt> <dd>${list(fr.depends_on)}</dd>
          <dt>Blocks</dt>     <dd>${list(fr.blocks)}</dd>
        </dl>
        <p class="fr-source"><a href="../../${esc(fr.path)}">Open spec ↗</a></p>
      </article>`;
}

function renderPage(data) {
  // Group by module for navigation
  const byModule = {};
  for (const fr of data.frs) {
    (byModule[fr.module] = byModule[fr.module] || []).push(fr);
  }
  const modules = Object.keys(byModule).sort();

  const moduleNav = modules.map(m =>
    `<a href="#module-${esc(m)}" class="module-link">${esc(m)} <span class="count">(${byModule[m].length})</span></a>`
  ).join(' · ');

  const modulesHtml = modules.map(m => `
    <section class="module-section" id="module-${esc(m)}">
      <h2 class="module-heading">${esc(m)} <span class="module-count">${byModule[m].length} FR${byModule[m].length === 1 ? '' : 's'}</span></h2>
      <div class="fr-grid">
${byModule[m].map(renderCard).join('\n')}
      </div>
    </section>
  `).join('\n');

  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>FR Catalog — CyberOS</title>
  <meta name="description" content="Complete catalog of CyberOS Feature Requests (FRs) — ${data.count} specifications across ${modules.length} modules.">
  <link rel="canonical" href="https://docs.cyberos.world/reference/fr-catalog.html">
  <style>
    :root {
      --bg: #0b0d10;
      --panel: #14181d;
      --panel-2: #1a1f25;
      --text: #e6eaef;
      --muted: #8893a0;
      --accent: #6ea8fe;
      --accent-2: #c8a8ff;
      --border: #232a32;
      --green: #5ed09b;
      --yellow: #f0c674;
      --red: #ef6e6e;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font: 16px/1.5 -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
      background: var(--bg);
      color: var(--text);
    }
    .container { max-width: 1280px; margin: 0 auto; padding: 32px 24px 96px; }
    h1 { margin: 0 0 8px; font-size: 32px; letter-spacing: -0.02em; }
    .subtitle { margin: 0 0 28px; color: var(--muted); font-size: 15px; }
    .stats {
      display: flex; gap: 16px; flex-wrap: wrap;
      padding: 16px; background: var(--panel); border: 1px solid var(--border);
      border-radius: 10px; margin-bottom: 24px;
    }
    .stat { display: flex; flex-direction: column; gap: 2px; }
    .stat-num { font-size: 24px; font-weight: 600; color: var(--accent); }
    .stat-label { font-size: 12px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; }
    .module-nav {
      padding: 14px 16px;
      background: var(--panel-2); border: 1px solid var(--border);
      border-radius: 8px; margin-bottom: 28px;
      font-size: 14px;
    }
    .module-link { color: var(--accent); text-decoration: none; }
    .module-link:hover { text-decoration: underline; }
    .module-link .count { color: var(--muted); }
    .module-section { margin: 40px 0; }
    .module-heading {
      font-size: 22px; margin: 0 0 16px;
      padding-bottom: 8px; border-bottom: 1px solid var(--border);
    }
    .module-count { color: var(--muted); font-weight: 400; font-size: 14px; }
    .fr-grid {
      display: grid; gap: 16px;
      grid-template-columns: repeat(auto-fill, minmax(420px, 1fr));
    }
    .fr-card {
      background: var(--panel);
      border: 1px solid var(--border);
      border-radius: 10px;
      padding: 18px 20px;
      scroll-margin-top: 16px;
    }
    .fr-card:target { border-color: var(--accent); box-shadow: 0 0 0 3px rgba(110, 168, 254, 0.18); }
    .fr-card-header { display: flex; align-items: baseline; gap: 8px; margin-bottom: 10px; }
    .fr-anchor { color: var(--muted); text-decoration: none; font-weight: 700; }
    .fr-anchor:hover { color: var(--accent); }
    .fr-id { margin: 0; font-size: 14px; font-weight: 700; color: var(--accent-2); font-family: ui-monospace, SF Mono, Menlo, monospace; }
    .fr-title { margin: 0; font-size: 14px; color: var(--text); flex: 1; }
    .fr-badges { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 12px; }
    .badge {
      display: inline-block; padding: 2px 8px;
      font-size: 11px; border-radius: 4px;
      background: var(--panel-2); color: var(--text);
      border: 1px solid var(--border);
      font-family: ui-monospace, SF Mono, Menlo, monospace;
    }
    .badge-priority-must  { background: rgba(239, 110, 110, 0.15); border-color: var(--red); }
    .badge-priority-should { background: rgba(240, 198, 116, 0.15); border-color: var(--yellow); }
    .badge-status-draft    { background: rgba(240, 198, 116, 0.10); color: var(--yellow); }
    .badge-status-accepted { background: rgba(94, 208, 155, 0.15); color: var(--green); }
    .badge-status-planned  { background: rgba(136, 147, 160, 0.15); color: var(--muted); }
    .badge-status-shipped  { background: rgba(94, 208, 155, 0.25); color: var(--green); }
    .badge-module          { background: rgba(110, 168, 254, 0.15); border-color: var(--accent); }
    .fr-meta {
      display: grid; grid-template-columns: 110px 1fr;
      gap: 4px 12px; margin: 12px 0;
      font-size: 13px;
    }
    .fr-meta dt { color: var(--muted); }
    .fr-meta dd { margin: 0; }
    .fr-link { color: var(--accent); text-decoration: none; }
    .fr-link:hover { text-decoration: underline; }
    .muted { color: var(--muted); font-style: italic; }
    .fr-source { margin: 12px 0 0; font-size: 12px; }
    .fr-source a { color: var(--muted); }
    .fr-source a:hover { color: var(--accent); }
    .footer { margin-top: 64px; padding-top: 24px; border-top: 1px solid var(--border); color: var(--muted); font-size: 13px; }
  </style>
</head>
<body>
  <div class="container">
    <h1>FR Catalog</h1>
    <p class="subtitle">Complete inventory of CyberOS Feature Requests — server-rendered at build time per <a href="../../docs/feature-requests/docs/FR-DOCS-001-server-render-reference-pages.md" class="fr-link">FR-DOCS-001</a>.</p>

    <div class="stats">
      <div class="stat"><span class="stat-num">${data.count}</span><span class="stat-label">total FRs</span></div>
      <div class="stat"><span class="stat-num">${modules.length}</span><span class="stat-label">modules</span></div>
      <div class="stat"><span class="stat-num">${data.frs.filter(f => f.priority === 'MUST').length}</span><span class="stat-label">MUST</span></div>
      <div class="stat"><span class="stat-num">${data.frs.filter(f => f.priority === 'SHOULD').length}</span><span class="stat-label">SHOULD</span></div>
      <div class="stat"><span class="stat-num">${data.frs.filter(f => f.status === 'draft').length}</span><span class="stat-label">draft</span></div>
      <div class="stat"><span class="stat-num">${data.frs.filter(f => f.status === 'accepted').length}</span><span class="stat-label">accepted</span></div>
      <div class="stat"><span class="stat-num">${data.frs.filter(f => f.shipped).length}</span><span class="stat-label">shipped</span></div>
      <div class="stat"><span class="stat-num">${data.frs.reduce((a, f) => a + (f.effort_hours || 0), 0)}h</span><span class="stat-label">total effort</span></div>
    </div>

    <nav class="module-nav">${moduleNav}</nav>

    <section data-prerendered="true">
${modulesHtml}
    </section>

    <p class="footer">
      Generated by <code>website/build/render-fr-catalog.mjs</code> from
      <code>website/build/data/frs.json</code>.
      Build is deterministic (FR-DOCS-001 §1 #3) — same input ⇒ byte-identical output.
      Re-run via <code>node website/build/data-extract.mjs &amp;&amp; node website/build/render-fr-catalog.mjs</code>.
    </p>
  </div>
</body>
</html>
`;
}

const data = JSON.parse(readFileSync(DATA_FILE, 'utf8'));
const html = renderPage(data);
mkdirSync(OUT_DIR, { recursive: true });
writeFileSync(OUT_FILE, html, 'utf8');

const report = {
  schema_version: 'v1',
  page:        'fr-catalog.html',
  fr_count:    data.count,
  modules:     [...new Set(data.frs.map(f => f.module))].sort(),
  output_bytes: Buffer.byteLength(html, 'utf8'),
  build_marker: 'deterministic',
};
writeFileSync(REPORT_FILE, JSON.stringify(report, null, 2) + '\n', 'utf8');

console.log(`✓ rendered ${data.count} FRs → website/docs/reference/fr-catalog.html (${(report.output_bytes / 1024).toFixed(1)} KB)`);
console.log(`✓ build report   → website/build/last-build-report.json`);
