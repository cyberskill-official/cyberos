#!/usr/bin/env node
// tools/docs-site/render-nfr-catalog.mjs
// Render dist/website/reference/nfr-catalog.html from tools/docs-site/data/nfrs.json.
// Pure-Node, no dependencies. Generates a self-contained dark-themed page with
// Alpine.js filtering, matching the site's Liquid Glass design system.

import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { SITE_BASE_URL } from './site-config.mjs';

const __dirname  = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT  = resolve(__dirname, '..', '..');
const DATA_FILE  = join(__dirname, 'data', 'nfrs.json');
const OUT_DIR    = join(REPO_ROOT, 'dist', 'website', 'reference');
const OUT_FILE   = join(OUT_DIR, 'nfr-catalog.html');
const REPORT_FILE = join(__dirname, 'last-nfr-build-report.json');

const esc = s => String(s ?? '')
  .replace(/&/g, '&amp;')
  .replace(/</g, '&lt;')
  .replace(/>/g, '&gt;')
  .replace(/"/g, '&quot;')
  .replace(/'/g, '&#39;');

// Category definitions: key → { code, name, meta description }
const CATEGORIES = {
  perf:   { code: 'PERF',   name: 'Performance Efficiency',      meta: 'Latency budgets, throughput ceilings, cost limits' },
  rel:    { code: 'REL',    name: 'Reliability',                  meta: 'Availability targets, DR, backup RPO/RTO' },
  sec:    { code: 'SEC',    name: 'Security',                     meta: 'Zero-trust, crypto, compliance gates' },
  usab:   { code: 'USAB',   name: 'Usability & Accessibility',   meta: 'WCAG, keyboard, i18n, readability' },
  maint:  { code: 'MAINT',  name: 'Maintainability',             meta: 'CI speed, coverage, schema deprecation' },
  compat: { code: 'COMPAT', name: 'Compatibility',               meta: 'Browser support, RFC compliance, portability' },
  tran:   { code: 'TRAN',   name: 'Transferability',             meta: 'Export, import, portability' },
  func:   { code: 'FUNC',   name: 'Functional Suitability',      meta: 'Functional correctness, determinism' },
  comp:   { code: 'COMP',   name: 'Compliance',                  meta: 'Regulatory, legal, audit obligations' },
  obs:    { code: 'OBS',    name: 'Observability',               meta: 'Metrics, logging, tracing, alerting' },
  priv:   { code: 'PRIV',   name: 'Privacy',                     meta: 'PII handling, data residency, GDPR' },
  scal:   { code: 'SCAL',   name: 'Scalability',                 meta: 'Growth ceilings, shard strategy' },
};

// Category → CSS class suffix for badges
const CAT_CSS = {
  perf: 'perf', rel: 'rel', sec: 'sec', usab: 'usab',
  maint: 'maint', compat: 'compat', tran: 'tran', func: 'func',
  comp: 'comp', obs: 'obs', priv: 'priv', scal: 'scal',
};

function renderCard(nfr) {
  const cat = CATEGORIES[nfr.category] || { code: nfr.category.toUpperCase(), name: nfr.category };
  return `      <article class="nfr-card"
               id="${esc(nfr.id)}"
               data-category="${esc(nfr.category)}"
               data-phase="${esc(nfr.phase)}"
               data-verify="${esc(nfr.verify)}"
               data-modules="${esc(nfr.modules)}">
        <header>
          <span class="nfr-id nfr-${esc(CAT_CSS[nfr.category] || nfr.category)}">${esc(nfr.id)}</span>
          <span class="target-pill">${esc(nfr.target)}</span>
          <span class="phase-chip phase-${esc(nfr.phase.toLowerCase())}">${esc(nfr.phase)}</span>
          <span class="verify-chip verify-t">Verify: ${esc(nfr.verify)}</span>
        </header>
        <h3>${esc(nfr.title)}</h3>
        <p class="description">${esc(nfr.description)}</p>
        <dl class="grid-info">
          <dt>Measurement</dt><dd>${esc(nfr.measurement)}</dd>
          <dt>Modules</dt><dd>${esc(nfr.modules)}</dd>
          <dt>Priority</dt><dd>${esc(nfr.priority)}</dd>
          <dt>Owner</dt><dd>${esc(nfr.owner)}</dd>
        </dl>
      </article>`;
}

function renderPage(data) {
  // Group by category
  const byCategory = {};
  for (const nfr of data.nfrs) {
    (byCategory[nfr.category] = byCategory[nfr.category] || []).push(nfr);
  }

  // Build category list with counts (only categories that have NFRs)
  const activeCategories = Object.keys(CATEGORIES).filter(k => byCategory[k]?.length);
  const catListJs = activeCategories.map(k => {
    const c = CATEGORIES[k];
    return `{ key:'${k}', code:'${c.code}', name:'${c.name}', meta:' · ${byCategory[k].length} NFRs' }`;
  }).join(',\n    ');

  // NFR data array for Alpine.js
  const nfrDataJs = data.nfrs.map(n => {
    return `{id:'${esc(n.id)}',category:'${esc(n.category)}',phase:'${esc(n.phase.toLowerCase())}',title:${JSON.stringify(n.title)},description:${JSON.stringify(n.description)},target:${JSON.stringify(n.target)},measurement:${JSON.stringify(n.measurement)},modules:'${esc(n.modules)}',verify:'${esc(n.verify)}',priority:'${esc(n.priority)}',owner:'${esc(n.owner)}'}`;
  }).join(',\n  ');

  // Category sections HTML
  const sectionsHtml = activeCategories.map(k => {
    const c = CATEGORIES[k];
    const nfrs = byCategory[k];
    return `
    <section id="${k}" class="cat-section">
      <header class="section-head">
        <span class="nfr-id nfr-${k}">${c.code}</span>
        <h2>${c.name}</h2>
        <span class="meta">${c.meta} · ${nfrs.length} NFRs</span>
      </header>
      ${nfrs.map(renderCard).join('\n      ')}
    </section>`;
  }).join('\n');

  // TOC links
  const tocHtml = activeCategories.map(k => {
    const c = CATEGORIES[k];
    return `      <a class="toc-link" href="#${k}">
        <span>${c.code} · ${c.name}</span>
        <span class="count">${byCategory[k].length}</span>
      </a>`;
  }).join('\n');

  // Stats
  const modules = [...new Set(data.nfrs.map(n => n.modules))].sort();
  const mustCount = data.nfrs.filter(n => n.priority === 'MUST').length;
  const shouldCount = data.nfrs.filter(n => n.priority === 'SHOULD').length;

  return `<!DOCTYPE html>
<html lang="en">
<head>
 <meta charset="UTF-8">
 <meta name="viewport" content="width=device-width, initial-scale=1.0">
 <title>CyberOS — NFR Catalog</title>
 <meta name="description" content="Non-Functional Requirements catalog: ${activeCategories.map(k => CATEGORIES[k].name).join(', ')}. ${data.count} specifications across ${modules.length} modules.">
 <link rel="canonical" href="${SITE_BASE_URL}/reference/nfr-catalog.html">
 <link rel="stylesheet" href="../assets/tokens.css">
 <link rel="stylesheet" href="../assets/styles.css">
 <link rel="stylesheet" href="../assets/tailwind.min.css">
 <script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.13.5/dist/cdn.min.js"></script>
 <link rel="preconnect" href="https://fonts.googleapis.com">
 <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
 <script type="module" src="../assets/scripts.js"></script>
 <style>
   .nfr-card { margin-bottom: 0.625rem; padding: 1rem 1.25rem; }
   .nfr-card header { display:flex; align-items:center; gap:0.5rem; flex-wrap:wrap; margin-bottom:0.375rem; }
   .nfr-id { font-family: var(--font-mono); font-size: 0.75rem; font-weight: 700; padding: 0.125rem 0.5rem; border-radius: 0.25rem; }
   .nfr-perf { background: var(--memory-tint); color: var(--memory-dark); }
   .nfr-rel { background: var(--skill-tint); color: var(--skill-dark); }
   .nfr-sec { background: var(--compliance-tint); color: var(--compliance-dark); }
   .nfr-usab { background: var(--cuo-tint); color: var(--cuo-dark); }
   .nfr-maint { background: var(--infra-tint); color: var(--infra-dark); }
   .nfr-compat { background: #e0e7ff; color: #3730a3; }
   .nfr-tran { background: #fce7f3; color: #9d174d; }
   .nfr-func { background: #ccfbf1; color: #115e59; }
   .nfr-comp { background: #fef3c7; color: #92400e; }
   .nfr-obs { background: #e0f2fe; color: #075985; }
   .nfr-priv { background: #ede9fe; color: #5b21b6; }
   .nfr-scal { background: #f0fdf4; color: #166534; }
   .target-pill { font-family: var(--font-mono); font-size: 0.75rem; font-weight: 700; background: #0f172a; color: #f1f5f9; padding: 0.125rem 0.5rem; border-radius: 0.375rem; }
   .nfr-card h3 { font-size: 1rem; font-weight: 700; margin: 0 0 0.375rem 0; color: var(--text); }
   .nfr-card .description { font-size: 0.875rem; color: var(--text-soft); line-height: 1.5; margin: 0 0 0.5rem 0; }
   .nfr-card .grid-info { display:grid; grid-template-columns: max-content 1fr; gap: 0.25rem 0.875rem; font-size: 0.8125rem; }
   .nfr-card .grid-info dt { font-weight:700; color: var(--text-mute); font-size: 0.6875rem; text-transform: uppercase; letter-spacing: 0.05em; }
   .nfr-card .grid-info dd { margin: 0; color: var(--text-soft); }
   .filter-row { display:flex; flex-wrap:wrap; gap:0.5rem; align-items:center; margin-bottom: 0.75rem; }
   .filter-row label { font-size:0.6875rem; font-weight:700; text-transform:uppercase; color:var(--text-mute); letter-spacing:0.06em; min-width:5rem; }
   .filter-row select,.filter-row input[type="search"] { padding:0.375rem 0.625rem; border:1px solid var(--border-strong); border-radius:0.375rem; font-size:0.8125rem; background:var(--bg-card); color:var(--text); }
   .filter-row input[type="search"] { min-width: 18rem; }
   .cat-section { scroll-margin-top: 80px; margin-bottom: 2.5rem; }
   .cat-section .section-head { display:flex; align-items:center; gap:0.75rem; margin-bottom: 1rem; padding-bottom: 0.5rem; border-bottom: 2px solid var(--border); }
   .cat-section .section-head h2 { font-size: 1.5rem; font-weight: 800; margin:0; }
   .cat-section .section-head .meta { font-size: 0.75rem; color: var(--text-mute); margin-left:auto; }
   .toc-link { display:block; padding:0.375rem 0.625rem; font-size:0.8125rem; color:var(--text-soft); text-decoration:none; border-radius:0.375rem; transition: all 0.12s; }
   .toc-link:hover { background: var(--future-tint); color: var(--text); }
   .toc-link .count { float:right; font-family: var(--font-mono); font-size: 0.6875rem; color: var(--text-mute); }
   .toc-sticky { position: sticky; top: 80px; max-height: calc(100vh - 100px); overflow-y: auto; padding: 1rem; }
 </style>
</head>
<body data-pagefind-filter="category:Reference">

<div id="shared-nav"></div>

<div x-data="nfrCatalog" x-init="$nextTick(() => init())" class="container py-8">

 <nav class="breadcrumbs">
   <a href="../index.html">Home</a><span class="sep">›</span>
   <a href="../index.html#navigate">Reference</a><span class="sep">›</span>
   <span class="current">NFR Catalog</span>
 </nav>

 <header class="mb-8">
   <div class="flex items-center gap-3 mb-3">
     <span class="section-badge" style="background:var(--infra-tint);color:var(--infra-dark)">Reference · Catalog</span>
   </div>
   <h1 class="h-1 mb-3">Non-Functional Requirements Catalog</h1>
   <p class="text-lg text-slate-600 max-w-3xl leading-relaxed mb-4">
     Every quantitative budget, threshold, and quality attribute CyberOS commits to.
     ${activeCategories.length} ISO/IEC 25010 categories — ${activeCategories.map(k => CATEGORIES[k].name).join(', ')} —
     each carrying a measurable target and a verification method (T = Test, I = Inspection, D = Demo, A = Analysis).
   </p>

   <div class="grid grid-cols-2 lg:grid-cols-4 gap-3 mt-6">
     <div class="bbg-card p-4"><div class="stat-num text-blue-600" x-text="nfrs.length"></div><div class="text-xs text-slate-500 font-medium mt-1">NFRs total</div></div>
     <div class="bbg-card p-4"><div class="stat-num text-rose-600" x-text="countByCategory('sec')"></div><div class="text-xs text-slate-500 font-medium mt-1">SEC (Security)</div></div>
     <div class="bbg-card p-4"><div class="stat-num text-emerald-600" x-text="countByCategory('perf')"></div><div class="text-xs text-slate-500 font-medium mt-1">PERF (Performance)</div></div>
     <div class="bbg-card p-4"><div class="stat-num text-amber-600">${activeCategories.length}</div><div class="text-xs text-slate-500 font-medium mt-1">Categories</div></div>
   </div>
 </header>

 <!-- FILTER UI -->
 <section class="bbg-card p-5 mb-8 no-print">
   <div class="filter-row">
     <label for="filter-cat">Category</label>
     <select id="filter-cat" x-model="filterCategory">
       <option value="all">All categories</option>
       <template x-for="c in categories" :key="c.key">
         <option :value="c.key" x-text="\`\${c.code} — \${c.name} (\${countByCategory(c.key)})\`"></option>
       </template>
     </select>
     <label for="filter-search" class="ml-4">Search</label>
     <input id="filter-search" type="search" x-model="searchQuery" placeholder="e.g. 'p95', 'WCAG', 'ISO 27001'…">
   </div>
   <div class="filter-row">
     <label>Phase</label>
     <template x-for="p in ['all','p0','p1','p2','p3','p4']" :key="p">
       <button class="chip" :class="{ 'active': filterPhase === p, [\`chip-\${p}\`]: filterPhase === p }" @click="filterPhase = p" x-text="p.toUpperCase()"></button>
     </template>
   </div>
   <div class="filter-row">
     <label>Verify</label>
     <template x-for="v in ['all','T','I','D','A']" :key="v">
       <button class="chip" :class="{ 'active': filterVerify === v }" @click="filterVerify = v" x-text="v"></button>
     </template>
   </div>
   <div class="flex items-center justify-between mt-2 pt-3 border-t border-slate-100">
     <div class="text-xs text-slate-500"><span x-text="visibleCount"></span> of <span x-text="nfrs.length"></span> NFRs match current filters.</div>
     <button class="btn btn-secondary text-xs" @click="reset()">Reset filters</button>
   </div>
 </section>

 <div class="grid lg:grid-cols-4 gap-6">
   <aside class="lg:col-span-1 no-print">
     <div class="toc-sticky bbg-card">
       <h3 class="h-eyebrow mb-3">Categories</h3>
${tocHtml}
     </div>
   </aside>

   <main class="lg:col-span-3">
${sectionsHtml}

     <div x-show="visibleCount === 0" class="bbg-card p-10 text-center">
       <p class="text-slate-500">No NFRs match the current filters. <button class="text-blue-600 font-bold underline" @click="reset()">Reset</button> to see the full catalog.</p>
     </div>
   </main>
 </div>

 <div class="mt-16 pt-8 border-t border-slate-200 text-center text-sm text-slate-500">
   <p>Generated by <code>tools/docs-site/render-nfr-catalog.mjs</code> from <code>tools/docs-site/data/nfrs.json</code>.
   Build is deterministic — same input → byte-identical output.</p>
 </div>
</div>

<script>
function nfrCatalog() {
  return {
    filterCategory: 'all',
    filterPhase: 'all',
    filterVerify: 'all',
    searchQuery: '',
    categories: [
    ${catListJs}
    ],
    nfrs: [],
    init() { this.nfrs = NFR_DATA; },
    matches(n) {
      if (this.filterCategory !== 'all' && n.category !== this.filterCategory) return false;
      if (this.filterPhase !== 'all' && n.phase !== this.filterPhase) return false;
      if (this.filterVerify !== 'all' && !n.verify.includes(this.filterVerify)) return false;
      if (this.searchQuery) {
        const q = this.searchQuery.toLowerCase();
        return n.id.toLowerCase().includes(q) || n.title.toLowerCase().includes(q) ||
          n.description.toLowerCase().includes(q) || n.target.toLowerCase().includes(q);
      }
      return true;
    },
    get visibleCount() { return this.nfrs.filter(n => this.matches(n)).length; },
    countByCategory(c) { return this.nfrs.filter(n => n.category === c).length; },
    reset() { this.filterCategory='all'; this.filterPhase='all'; this.filterVerify='all'; this.searchQuery=''; },
  };
}

const NFR_DATA = [
  ${nfrDataJs}
];
</script>

</body>
</html>`;
}

const data = JSON.parse(readFileSync(DATA_FILE, 'utf8'));
const html = renderPage(data);
mkdirSync(OUT_DIR, { recursive: true });
writeFileSync(OUT_FILE, html, 'utf8');

const modules = [...new Set(data.nfrs.map(n => n.modules))].sort();
const activeCats = [...new Set(data.nfrs.map(n => n.category))].sort();

const report = {
  schema_version: 'v1',
  page:          'nfr-catalog.html',
  nfr_count:     data.count,
  categories:    activeCats,
  modules:       modules,
  output_bytes:  Buffer.byteLength(html, 'utf8'),
  build_marker:  'deterministic',
};
writeFileSync(REPORT_FILE, JSON.stringify(report, null, 2) + '\n', 'utf8');

console.log(`✓ rendered ${data.count} NFRs → dist/website/reference/nfr-catalog.html (${(report.output_bytes / 1024).toFixed(1)} KB)`);
console.log(`✓ build report   → tools/docs-site/last-nfr-build-report.json`);
