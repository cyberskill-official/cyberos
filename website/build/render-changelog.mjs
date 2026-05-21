#!/usr/bin/env node
// website/build/render-changelog.mjs
// Render website/docs/reference/changelog.html from root CHANGELOG.md.
// Pure-Node, no dependencies. Converts markdown to HTML matching the site's
// Liquid Glass design system.

import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { dirname, join, resolve, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname  = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT  = resolve(__dirname, '..', '..');
const MD_FILE    = join(REPO_ROOT, 'CHANGELOG.md');
const OUT_DIR    = join(REPO_ROOT, 'website', 'docs', 'reference');
const OUT_FILE   = join(OUT_DIR, 'changelog.html');

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
// Main
// ─────────────────────────────────────────────────────────────────────────────
const md = readFileSync(MD_FILE, 'utf8');

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
 <title>CyberOS — Changelog</title>
 <meta name="description" content="CyberOS changelog — all significant changes across modules, services, and infrastructure.">
 <link rel="canonical" href="https://cyberos-wiki.cyberskill.world/reference/changelog.html">
 <link rel="stylesheet" href="../assets/tokens.css">
 <link rel="stylesheet" href="../assets/styles.css">
 <link rel="stylesheet" href="../assets/tailwind.min.css">
 <script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.13.5/dist/cdn.min.js"></script>
 <link rel="preconnect" href="https://fonts.googleapis.com">
 <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
 <script type="module" src="../assets/scripts.js"></script>
 <link rel="alternate" type="application/rss+xml" title="CyberOS Changelog RSS" href="../feed.xml">
</head>
<body data-pagefind-filter="category:Reference">

<div id="shared-nav"></div>

<div class="container py-8 max-w-5xl">

 <nav class="breadcrumbs mb-6">
   <a href="../index.html">Home</a><span class="sep">›</span>
   <a href="../index.html#navigate">Reference</a><span class="sep">›</span>
   <span class="current">Changelog</span>
 </nav>

 <header class="mb-8">
   <div class="flex items-center gap-3 mb-3">
     <span class="section-badge" style="background:var(--memory-tint);color:var(--memory-dark)">Reference</span>
   </div>
   <h1 class="h-1 mb-3">Changelog</h1>
   <p class="text-lg text-slate-600 max-w-3xl leading-relaxed mb-4">
     All significant changes across CyberOS modules, services, and infrastructure.
     For module-specific changelogs, see the per-module pages in the nav.
   </p>
 </header>

 <div class="grid lg:grid-cols-4 gap-8">

   <!-- TOC sidebar -->
   <aside class="lg:col-span-1 no-print">
     <div class="sticky top-20 max-h-[calc(100vh-100px)] overflow-y-auto p-3 rounded-lg" style="background:var(--bg-card,#f8fafc);border:1px solid var(--border,#e2e8f0)">
       <h3 class="text-xs font-bold uppercase tracking-wider text-slate-500 mb-2 px-2">Entries</h3>
${tocHtml}
     </div>
   </aside>

   <!-- Main content -->
   <main class="lg:col-span-3 prose prose-slate max-w-none">
     <div class="bg-white rounded-xl p-6 lg:p-8 shadow-sm border border-slate-200">
${htmlContent}
     </div>
   </main>

 </div>

 <div class="mt-12 pt-6 border-t border-slate-200 text-center text-sm text-slate-500">
   <p>Generated by <code>website/build/render-changelog.mjs</code> from <code>CHANGELOG.md</code>.</p>
 </div>
</div>

</body>
</html>`;

mkdirSync(OUT_DIR, { recursive: true });
writeFileSync(OUT_FILE, page, 'utf8');

const entryCount = toc.length;
const bytes = Buffer.byteLength(page, 'utf8');
console.log(`✓ rendered ${entryCount} entries → website/docs/reference/changelog.html (${(bytes / 1024).toFixed(1)} KB)`);
