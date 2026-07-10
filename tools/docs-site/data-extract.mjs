#!/usr/bin/env node
// tools/docs-site/data-extract.mjs
// FR-DOCS-001 §1 #1 — extract data from FR markdown frontmatter to JSON
// Walks docs/feature-requests/**/FR-*.md (excluding .audit.md), parses YAML
// frontmatter, emits tools/docs-site/data/frs.json sorted deterministically by FR id.

import { readFileSync, writeFileSync, readdirSync, mkdirSync, statSync } from 'node:fs';
import { join, resolve, dirname, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(__dirname, '..', '..');
const FR_DIR    = join(REPO_ROOT, 'docs', 'feature-requests');
const OUT_DIR   = join(__dirname, 'data');
const OUT_FILE  = join(OUT_DIR, 'frs.json');

// ─────────────────────────────────────────────────────────────────────────────
// Minimal YAML-flow parser for the frontmatter shapes we actually use.
// Handles: scalars, quoted strings, flow lists `[a, b, c]`, inline comments,
// `null`, ISO dates as strings. No anchors, no block scalars beyond what the
// FR template uses.
// ─────────────────────────────────────────────────────────────────────────────
function parseScalar(raw) {
  let s = raw.trim();
  // strip trailing inline comment (unquoted only)
  if (!s.startsWith('"') && !s.startsWith("'") && !s.startsWith('[')) {
    const hashIdx = s.indexOf('#');
    if (hashIdx >= 0) s = s.slice(0, hashIdx).trim();
  }
  if (s === '' || s === 'null') return null;
  if (s === 'true') return true;
  if (s === 'false') return false;
  if (/^-?\d+$/.test(s)) return parseInt(s, 10);
  if (/^-?\d*\.\d+$/.test(s)) return parseFloat(s);
  // flow list
  if (s.startsWith('[') && s.endsWith(']')) {
    const inner = s.slice(1, -1).trim();
    if (inner === '') return [];
    return inner.split(',').map(x => parseScalar(x));
  }
  // quoted
  if ((s.startsWith('"') && s.endsWith('"')) || (s.startsWith("'") && s.endsWith("'"))) {
    return s.slice(1, -1);
  }
  return s;
}

function parseFrontmatter(md) {
  if (!md.startsWith('---')) return null;
  const end = md.indexOf('\n---', 4);
  if (end < 0) return null;
  const block = md.slice(4, end);
  const result = {};
  const lines = block.split('\n');
  let i = 0;
  while (i < lines.length) {
    const raw = lines[i];
    // skip blank + pure-comment lines
    if (!raw.trim() || raw.trim().startsWith('#')) { i++; continue; }
    const m = raw.match(/^([A-Za-z_][A-Za-z0-9_]*):\s*(.*)$/);
    if (!m) { i++; continue; }
    const key = m[1];
    const after = m[2];
    if (after === '') {
      // block list or block scalar — collect indented lines that start with '- '
      const items = [];
      let j = i + 1;
      while (j < lines.length) {
        const l = lines[j];
        if (/^\s+- /.test(l)) {
          items.push(parseScalar(l.replace(/^\s+- /, '')));
          j++;
        } else if (l.trim() === '' || l.match(/^[A-Za-z_]/)) {
          break;
        } else {
          j++;
        }
      }
      result[key] = items;
      i = j;
    } else {
      result[key] = parseScalar(after);
      i++;
    }
  }
  return result;
}

function walkFR(dir) {
  const out = [];
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    const st = statSync(p);
    if (st.isDirectory()) {
      out.push(...walkFR(p));
    } else if (/^FR-.*\.md$/.test(name) && !name.endsWith('.audit.md')) {
      out.push(p);
    }
  }
  return out;
}

const files = walkFR(FR_DIR).sort();
const records = [];
for (const file of files) {
  const md = readFileSync(file, 'utf8');
  const fm = parseFrontmatter(md);
  if (!fm || !fm.id) {
    console.warn(`skip (no frontmatter): ${relative(REPO_ROOT, file)}`);
    continue;
  }
  records.push({
    id:           fm.id,
    title:        fm.title || '',
    module:       fm.module || '',
    priority:     fm.priority || '',
    status:       fm.status || '',
    verify:       fm.verify || '',
    phase:        fm.phase || '',
    milestone:    fm.milestone || '',
    slice:        fm.slice ?? null,
    owner:        fm.owner || '',
    created:      fm.created || '',
    shipped:      fm.shipped ?? null,
    effort_hours: fm.effort_hours ?? null,
    depends_on:   Array.isArray(fm.depends_on)  ? fm.depends_on  : [],
    blocks:       Array.isArray(fm.blocks)      ? fm.blocks      : [],
    related_frs:  Array.isArray(fm.related_frs) ? fm.related_frs : [],
    path:         relative(REPO_ROOT, file),
  });
}

// Deterministic sort by FR id (FR-AI-001 < FR-AI-002 < FR-AUTH-001 ...).
// Use module then numeric id for stable ordering even if id strings differ.
records.sort((a, b) => {
  if (a.module !== b.module) return a.module.localeCompare(b.module);
  return a.id.localeCompare(b.id, 'en', { numeric: true });
});

mkdirSync(OUT_DIR, { recursive: true });
const payload = {
  schema_version: 'v1',
  generated_at_marker: 'deterministic',  // FR-DOCS-001 §1 #3 — no Date.now()
  count: records.length,
  frs: records,
};
writeFileSync(OUT_FILE, JSON.stringify(payload, null, 2) + '\n', 'utf8');
console.log(`✓ wrote ${records.length} FRs → ${relative(REPO_ROOT, OUT_FILE)}`);
