#!/usr/bin/env node
// website/build/render-module-changelog.mjs
// Render per-module changelog pages from modules/<slug>/CHANGELOG.md.
// Pure-Node, no dependencies. Generates website/docs/modules/<slug>/changelog.html.

import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'node:fs';
import { dirname, join, resolve, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname  = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT  = resolve(__dirname, '..', '..');
const MODULES_DIR = join(REPO_ROOT, 'modules');
const OUT_BASE   = join(REPO_ROOT, 'website', 'docs', 'modules');

// Module metadata for page headers
const MODULE_META = {
  ai:     { name: 'AI Gateway', icon: '🤖', phase: 'P0', color: 'ai' },
  auth:   { name: 'Authentication', icon: '🔐', phase: 'P0', color: 'auth' },
  chat:   { name: 'Chat', icon: '💬', phase: 'P0', color: 'chat' },
  crm:    { name: 'Customer Relationships', icon: '🤝', phase: 'P1', color: 'crm' },
  cuo:    { name: 'Persona Orchestration', icon: '🎭', phase: 'P0', color: 'cuo' },
  doc:    { name: 'Documents & Signatures', icon: '📄', phase: 'P4', color: 'doc' },
  email:  { name: 'Email', icon: '📧', phase: 'P1', color: 'email' },
  esop:   { name: 'Stock Options', icon: '📈', phase: 'P2', color: 'esop' },
  hr:     { name: 'People & HR', icon: '👥', phase: 'P1', color: 'hr' },
  inv:    { name: 'Invoicing', icon: '🧾', phase: 'P2', color: 'inv' },
  kb:     { name: 'Knowledge Base', icon: '📚', phase: 'P1', color: 'kb' },
  learn:  { name: 'Learning', icon: '🎓', phase: 'P1', color: 'learn' },
  mcp:    { name: 'MCP Gateway', icon: '🔌', phase: 'P0', color: 'mcp' },
  memory: { name: 'Memory', icon: '🧠', phase: 'P0', color: 'memory' },
  obs:    { name: 'Observability', icon: '📊', phase: 'P0', color: 'obs' },
  okr:    { name: 'Objectives & KRs', icon: '🎯', phase: 'P3', color: 'okr' },
  // plugin: no CHANGELOG.md — hand-authored changelog.html stays as-is
  portal: { name: 'Client Portal', icon: '🌐', phase: 'P4', color: 'portal' },
  proj:   { name: 'Project Tracking', icon: '📋', phase: 'P1', color: 'proj' },
  res:    { name: 'Resourcing', icon: '👷', phase: 'P3', color: 'res' },
  rew:    { name: 'Compensation', icon: '💰', phase: 'P1', color: 'rew' },
  skill:  { name: 'Skill Catalog', icon: '🛠️', phase: 'P0', color: 'skill' },
  ten:    { name: 'Tenants', icon: '🏢', phase: 'P4', color: 'ten' },
  time:   { name: 'Time Tracking', icon: '⏱️', phase: 'P1', color: 'time' },
  website:{ name: 'Website & Infrastructure', icon: '🌐', phase: '—', color: 'website' },
};

const esc = s => String(s ?? '')
  .replace(/&/g, '&amp;')
  .replace(/</g, '&lt;')
  .replace(/>/g, '&gt;');

// ─────────────────────────────────────────────────────────────────────────────
// Minimal markdown → HTML converter (handles the patterns in CHANGELOG.md)
// ─────────────────────────────────────────────────────────────────────────────
function mdToHtml(md) {
  const lines = md.split('\n');
  const out = [];
  let inCodeBlock = false;
  let codeLang = '';
  let codeLines = [];
  let inTable = false;
  let tableRows = [];

  function flushTable() {
    if (!inTable || tableRows.length === 0) return;
    const header = tableRows[0];
    const body = tableRows.slice(2); // skip separator row
    out.push('<div class="overflow-x-auto mb-4"><table class="text-sm">');
    out.push('<thead><tr>');
    for (const cell of header) {
      out.push(`<th>${inlineMd(cell.trim())}</th>`);
    }
    out.push('</tr></thead>');
    if (body.length) {
      out.push('<tbody>');
      for (const row of body) {
        out.push('<tr>');
        for (const cell of row) {
          out.push(`<td>${inlineMd(cell.trim())}</td>`);
        }
        out.push('</tr>');
      }
      out.push('</tbody>');
    }
    out.push('</table></div>');
    inTable = false;
    tableRows = [];
  }

  function flushCode() {
    if (!inCodeBlock) return;
    out.push(`<pre class="mb-4 p-4 rounded-lg text-sm overflow-x-auto" style="background:var(--bg-code,#1e293b);color:#e2e8f0"><code>${esc(codeLines.join('\n'))}</code></pre>`);
    inCodeBlock = false;
    codeLines = [];
  }

  for (const line of lines) {
    // Fenced code blocks
    if (line.startsWith('```')) {
      if (inCodeBlock) {
        flushCode();
      } else {
        flushTable();
        inCodeBlock = true;
        codeLang = line.slice(3).trim();
        codeLines = [];
      }
      continue;
    }
    if (inCodeBlock) {
      codeLines.push(line);
      continue;
    }

    // Table rows
    if (line.includes('|') && line.trim().startsWith('|')) {
      const cells = line.split('|').slice(1, -1);
      if (cells.every(c => /^\s*[-:]+\s*$/.test(c))) {
        // separator row
        tableRows.push(cells);
        inTable = true;
        continue;
      }
      tableRows.push(cells);
      inTable = true;
      continue;
    } else {
      flushTable();
    }

    // Blank line
    if (!line.trim()) {
      out.push('');
      continue;
    }

    // Horizontal rule
    if (/^---+\s*$/.test(line)) {
      out.push('<hr class="my-6 border-slate-200">');
      continue;
    }

    // Headers
    const hMatch = line.match(/^(#{1,4})\s+(.+)/);
    if (hMatch) {
      const level = hMatch[1].length;
      const text = hMatch[2];
      const tag = `h${Math.min(level + 1, 6)}`; // shift down one level (H1→H2, H2→H3)
      const cls = level <= 2 ? 'class="text-xl font-bold mt-8 mb-3 text-slate-900"' :
                  level === 3 ? 'class="text-lg font-semibold mt-6 mb-2 text-slate-800"' :
                  'class="text-base font-semibold mt-4 mb-2 text-slate-700"';
      const id = text.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
      out.push(`<${tag} id="${id}" ${cls}>${inlineMd(text)}</${tag}>`);
      continue;
    }

    // List items
    const listMatch = line.match(/^(\s*)[-*]\s+(.+)/);
    if (listMatch) {
      const indent = listMatch[1].length;
      const text = listMatch[2];
      const pl = indent >= 2 ? 'ml-6' : '';
      out.push(`<li class="mb-1 ${pl}">${inlineMd(text)}</li>`);
      continue;
    }

    // Numbered list
    const numMatch = line.match(/^(\s*)\d+\.\s+(.+)/);
    if (numMatch) {
      const text = numMatch[2];
      out.push(`<li class="mb-1 list-decimal ml-6">${inlineMd(text)}</li>`);
      continue;
    }

    // Paragraph
    out.push(`<p class="mb-3 leading-relaxed">${inlineMd(line)}</p>`);
  }

  flushCode();
  flushTable();

  return out.join('\n');
}

function inlineMd(text) {
  return text
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    .replace(/`([^`]+)`/g, '<code class="px-1.5 py-0.5 rounded text-sm" style="background:var(--bg-code,#e2e8f0);color:#334155">$1</code>')
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" class="text-blue-600 hover:underline">$1</a>');
}

// ─────────────────────────────────────────────────────────────────────────────
// Extract TOC entries from H2 headers
// ─────────────────────────────────────────────────────────────────────────────
function extractToc(md) {
  const entries = [];
  for (const line of md.split('\n')) {
    const m = line.match(/^## (.+)/);
    if (m) {
      const text = m[1];
      const id = text.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
      entries.push({ text, id });
    }
  }
  return entries;
}

// ─────────────────────────────────────────────────────────────────────────────
// Render a single module's changelog page
// ─────────────────────────────────────────────────────────────────────────────
function renderModuleChangelog(slug) {
  const mdPath = join(MODULES_DIR, slug, 'CHANGELOG.md');
  if (!existsSync(mdPath)) {
    console.log(`  ⚠ skipping ${slug} — no CHANGELOG.md`);
    return false;
  }

  const md = readFileSync(mdPath, 'utf8');
  const meta = MODULE_META[slug] || { name: slug.toUpperCase(), icon: '📦', phase: '—', color: slug };

  // Strip the H1 title line
  const bodyMd = md.replace(/^# .+\n+/, '');
  const htmlContent = mdToHtml(bodyMd);
  const toc = extractToc(md);

  const tocHtml = toc.map(e =>
    `      <a class="block py-1 px-2 text-sm rounded hover:bg-slate-100 text-slate-600 hover:text-slate-900" href="#${e.id}">${esc(e.text)}</a>`
  ).join('\n');

  const page = `<!DOCTYPE html>
<html lang="en">
<head>
 <meta charset="UTF-8">
 <meta name="viewport" content="width=device-width, initial-scale=1.0">
 <title>${esc(meta.name)} — Changelog — CyberOS</title>
 <meta name="description" content="${esc(meta.name)} module changelog — all significant changes.">
 <link rel="canonical" href="https://cyberos-wiki.cyberskill.world/modules/${slug}/changelog.html">
 <link rel="stylesheet" href="../../assets/tokens.css">
 <link rel="stylesheet" href="../../assets/styles.css">
 <link rel="stylesheet" href="../../assets/tailwind.min.css">
 <script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.13.5/dist/cdn.min.js"></script>
 <link rel="preconnect" href="https://fonts.googleapis.com">
 <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
 <script type="module" src="../../assets/scripts.js"></script>
</head>
<body data-pagefind-filter="category:Module">
<span hidden data-pagefind-filter="phase:${esc(meta.phase)}"></span>

<div id="shared-nav"></div>

<header class="hero-gradient border-b border-slate-200">
 <div class="container py-10 lg:py-14">
 <nav class="breadcrumbs">
 <a href="../../index.html">Home</a>
 <span class="sep">›</span>
 <a href="../../index.html#catalog">Modules</a>
 <span class="sep">›</span>
 <a href="index.html">${esc(meta.name)}</a>
 <span class="sep">›</span>
 <span class="current">Changelog</span>
 </nav>
 <div class="flex items-center gap-3 mb-4 flex-wrap">
 <span class="text-5xl">${meta.icon}</span>
 <h1 class="h-display">${esc(meta.name)} — Changelog</h1>
 </div>
 <p class="text-xl text-slate-700 font-medium leading-snug max-w-3xl mb-3">
 All significant changes to the ${esc(meta.name)} module.
 </p>
 <div class="flex flex-wrap gap-3 mt-6">
 <a href="index.html" class="btn btn-primary">← Back to ${esc(meta.name)} overview</a>
 </div>
 </div>
</header>

<div class="container py-10 lg:py-14 grid lg:grid-cols-12 gap-10">

<aside class="lg:col-span-3 order-2 lg:order-1">
 <div class="lg:sticky lg:top-20">
 <div class="bbg-card p-5">
 <div class="h-eyebrow mb-3">On this page</div>
 <ul class="space-y-0.5 text-sm">
${tocHtml}
 </ul>
 </div>
 </div>
</aside>

<main class="lg:col-span-9 prose prose-slate max-w-none">
 <div class="bg-white rounded-xl p-6 lg:p-8 shadow-sm border border-slate-200">
${htmlContent}
 </div>
</main>

</div>

</body>
</html>`;

  // Write output
  const outDir = join(OUT_BASE, slug);
  mkdirSync(outDir, { recursive: true });
  const outPath = join(outDir, 'changelog.html');
  writeFileSync(outPath, page, 'utf8');

  const entryCount = toc.length;
  const bytes = Buffer.byteLength(page, 'utf8');
  console.log(`  ✓ ${slug}: ${entryCount} entries → modules/${slug}/changelog.html (${(bytes / 1024).toFixed(1)} KB)`);
  return true;
}

// ─────────────────────────────────────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────────────────────────────────────
const args = process.argv.slice(2);
const targetSlug = args[0]; // optional: single module slug

if (targetSlug) {
  // Render single module
  console.log(`→ Rendering changelog for ${targetSlug}`);
  renderModuleChangelog(targetSlug);
} else {
  // Render all modules
  console.log('→ Rendering per-module changelogs');
  let count = 0;
  for (const slug of Object.keys(MODULE_META)) {
    if (renderModuleChangelog(slug)) count++;
  }
  console.log(`✓ rendered ${count} module changelogs`);
}
