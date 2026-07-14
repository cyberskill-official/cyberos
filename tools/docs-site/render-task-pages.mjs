#!/usr/bin/env node
// tools/docs-site/render-fr-pages.mjs - TASK-DOCS-005.
// Renders every docs/tasks/<module>/<STEM>/spec.md (+ audit.md, + assets/) into a
// self-contained CDS page via modules/templates/html/deliverable.html (template@1).
// Node stdlib only. Deterministic. Fails loud on unreadable spec or missing referenced asset.
// Usage: node render-fr-pages.mjs [repoRoot] [outDir]
import { readFileSync, writeFileSync, readdirSync, mkdirSync, existsSync, statSync, copyFileSync } from 'node:fs';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { renderMarkdown, frontmatter } from './md.mjs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(process.argv[2] || resolve(__dirname, '..', '..'));
const OUT = resolve(process.argv[3] || join(ROOT, 'dist', 'website'));
const FR_ROOT = join(ROOT, 'docs', 'tasks');
const SHELL = readFileSync(join(ROOT, 'modules', 'templates', 'html', 'deliverable.html'), 'utf-8');
const TOKENS = readFileSync(join(ROOT, 'modules', 'templates', 'cds', 'tokens.css'), 'utf-8');

const esc = s => String(s ?? '').replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
const fill = (tpl, slots) => {
  let out = tpl.replace('/*{{slot:styles:html}}*/', TOKENS);
  for (const [k, v] of Object.entries(slots)) {
    out = out.split(`{{slot:${k}}}`).join(k.endsWith(':html') ? (v ?? '') : esc(v ?? ''));
  }
  return out.replace(/\{\{slot:[a-z_]+(:html)?\}\}/g, '');
};

function listArr(fmText, key) {
  const m = fmText.match(new RegExp(`^${key}:\\s*\\[([^\\]]*)\\]`, 'm'));
  return m ? m[1].split(',').map(s => s.trim().replace(/^["']|["']$/g, '')).filter(Boolean) : [];
}

// index all FR folders (for cross-links)
const index = new Map(); // FR-ID -> {module, stem}
const folders = [];
for (const mod of readdirSync(FR_ROOT, { withFileTypes: true }).sort((a,b)=>a.name.localeCompare(b.name))) {
  if (!mod.isDirectory() || mod.name.startsWith('_') || mod.name.startsWith('.')) continue;
  for (const d of readdirSync(join(FR_ROOT, mod.name), { withFileTypes: true }).sort((a,b)=>a.name.localeCompare(b.name))) {
    if (!d.isDirectory() || !d.name.startsWith('FR-')) continue;
    const spec = join(FR_ROOT, mod.name, d.name, 'spec.md');
    if (!existsSync(spec)) continue;
    folders.push({ module: mod.name, stem: d.name, spec });
    const id = (d.name.match(/^(FR-[A-Z0-9]+-\d+)/) || [null, d.name])[1];
    index.set(id, { module: mod.name, stem: d.name });
  }
}

const frLink = id => {
  const hit = index.get(id);
  return hit ? `<a href="../../${hit.module}/${hit.stem}/index.html">${esc(id)}</a>` : esc(id);
};

let pages = 0, assets = 0, withAudit = 0;
for (const { module, stem, spec } of folders) {
  let raw;
  try { raw = readFileSync(spec, 'utf-8'); }
  catch (e) { console.error(`fr-pages: ERROR unreadable spec ${spec}: ${e.message}`); process.exit(1); }
  const { meta, body } = frontmatter(raw);
  const fmText = (raw.match(/\A?---\n([\s\S]*?)\n---\n/) || ['',''])[1];
  let html = renderMarkdown(body);
  // video support: image-syntax with a video extension becomes <video controls>
  html = html.replace(/<img src="([^"]+\.(mp4|webm|mov))"([^>]*)>/g, '<video controls src="$1"$3></video>');

  // audit block
  const auditPath = join(dirname(spec), 'audit.md');
  let auditHtml = '';
  if (existsSync(auditPath)) {
    withAudit++;
    const a = frontmatter(readFileSync(auditPath, 'utf-8'));
    auditHtml = `<section class="dlv-audit"><h2>Audit</h2>${renderMarkdown(a.body)}</section>`;
  }

  // meta strip
  const dep = listArr(fmText, 'depends_on').map(frLink).join(', ') || '<em>none</em>';
  const blk = listArr(fmText, 'blocks').map(frLink).join(', ') || '<em>none</em>';
  const metaHtml = `<strong>module</strong> ${esc(meta.module ?? module)} · <strong>class</strong> ${esc(meta.class ?? 'product')} · <strong>priority</strong> ${esc(meta.priority ?? '')} · <strong>created</strong> ${esc(meta.created ?? '')} · <strong>shipped</strong> ${esc(meta.shipped ?? 'null')}<br><strong>depends on</strong> ${dep} · <strong>blocks</strong> ${blk}`;

  const page = fill(SHELL, {
    title: meta.title ?? stem, kind: 'Task — engineering-spec@1',
    id: meta.id ?? stem, status: meta.status ?? '(none)',
    'badges:html': '', 'meta:html': metaHtml, 'body:html': html + auditHtml,
    footer: `Generated from docs/tasks/${module}/${stem}/spec.md — markdown is the source of truth (TASK-DOCS-002/005).`,
  });

  const outDir = join(OUT, 'frs', module, stem);
  mkdirSync(outDir, { recursive: true });
  writeFileSync(join(outDir, 'index.html'), page);
  pages++;

  // assets: copy + verify referenced ones exist
  const assetDir = join(dirname(spec), 'assets');
  if (existsSync(assetDir)) {
    const walk = (src, dst) => {
      mkdirSync(dst, { recursive: true });
      for (const f of readdirSync(src, { withFileTypes: true }).sort((a,b)=>a.name.localeCompare(b.name))) {
        if (f.isDirectory()) walk(join(src, f.name), join(dst, f.name));
        else { copyFileSync(join(src, f.name), join(dst, f.name)); assets++; }
      }
    };
    walk(assetDir, join(outDir, 'assets'));
  }
  for (const m of page.matchAll(/(?:src|href)="(assets\/[^"]+)"/g)) {
    if (!existsSync(join(dirname(spec), m[1]))) {
      console.error(`fr-pages: ERROR missing referenced asset ${m[1]} in ${spec}`); process.exit(1);
    }
  }
}
console.log(`fr-pages: ${pages} pages, ${assets} assets copied, ${withAudit} with audits`);
