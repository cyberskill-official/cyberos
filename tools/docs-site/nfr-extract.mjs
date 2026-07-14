#!/usr/bin/env node
// tools/docs-site/nfr-extract.mjs
// Extract data from NFR markdown frontmatter to JSON.
// Walks docs/non-functional-requirements/**/NFR-*.md (excluding .audit.md),
// parses YAML frontmatter, emits tools/docs-site/data/nfrs.json sorted deterministically.

import { readFileSync, writeFileSync, readdirSync, mkdirSync, statSync } from 'node:fs';
import { join, resolve, dirname, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(__dirname, '..', '..');
const NFR_DIR   = join(REPO_ROOT, 'docs', 'non-functional-requirements');
const OUT_DIR   = join(__dirname, 'data');
const OUT_FILE  = join(OUT_DIR, 'nfrs.json');

// ─────────────────────────────────────────────────────────────────────────────
// Minimal YAML-flow parser (lifted from data-extract.mjs)
// ─────────────────────────────────────────────────────────────────────────────
function parseScalar(raw) {
  let s = raw.trim();
  if (!s.startsWith('"') && !s.startsWith("'") && !s.startsWith('[')) {
    const hashIdx = s.indexOf('#');
    if (hashIdx >= 0) s = s.slice(0, hashIdx).trim();
  }
  if (s === '' || s === 'null') return null;
  if (s === 'true') return true;
  if (s === 'false') return false;
  if (/^-?\d+$/.test(s)) return parseInt(s, 10);
  if (/^-?\d*\.\d+$/.test(s)) return parseFloat(s);
  if (s.startsWith('[') && s.endsWith(']')) {
    const inner = s.slice(1, -1).trim();
    if (inner === '') return [];
    return inner.split(',').map(x => parseScalar(x));
  }
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
    if (!raw.trim() || raw.trim().startsWith('#')) { i++; continue; }
    const m = raw.match(/^([A-Za-z_][A-Za-z0-9_]*):\s*(.*)$/);
    if (!m) { i++; continue; }
    const key = m[1];
    const after = m[2];
    if (after === '') {
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

// ─────────────────────────────────────────────────────────────────────────────
// Category name → ISO/IEC 25010 key mapping (frontmatter uses full names)
// ─────────────────────────────────────────────────────────────────────────────
const CATEGORY_MAP = {
  performance:            'perf',
  reliability:            'rel',
  security:               'sec',
  usability:              'usab',
  maintainability:        'maint',
  compatibility:          'compat',
  transferability:        'tran',
  functional_suitability: 'func',
  compliance:             'comp',
  observability:          'obs',
  privacy:                'priv',
  scalability:            'scal',
};

function walkNFR(dir) {
  const out = [];
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    const st = statSync(p);
    if (st.isDirectory()) {
      out.push(...walkNFR(p));
    } else if (/^NFR-.*\.md$/.test(name) && !name.endsWith('.audit.md')) {
      out.push(p);
    }
  }
  return out;
}

const files = walkNFR(NFR_DIR).sort();
const records = [];
for (const file of files) {
  const md = readFileSync(file, 'utf8');
  const fm = parseFrontmatter(md);
  if (!fm || !fm.id) {
    console.warn(`skip (no frontmatter): ${relative(REPO_ROOT, file)}`);
    continue;
  }

  // Map category full name to short key
  const rawCategory = (fm.category || '').toLowerCase();
  const categoryKey = CATEGORY_MAP[rawCategory] || rawCategory;

  // Extract description from §1 body (first numbered list item after "## §1")
  let description = '';
  const bodyAfterFrontmatter = md.slice(md.indexOf('\n---', 4) + 4);
  const section1Match = bodyAfterFrontmatter.match(/## §1[\s\S]*?\n\n([\s\S]*?)(?=\n## |\n---|\n\*End)/);
  if (section1Match) {
    const section1 = section1Match[1];
    // First numbered list item
    const firstItem = section1.match(/^\d+\.\s+(.+)/m);
    if (firstItem) {
      description = firstItem[1].replace(/\*\*/g, '').trim();
    }
  }

  // Extract measurement from §3 — either a bullet point or the first paragraph
  let measurement = '';
  const section3Match = bodyAfterFrontmatter.match(/## §3[\s\S]*?\n\n([\s\S]*?)(?=\n## |\n---|\n\*End)/);
  if (section3Match) {
    const section3 = section3Match[1];
    const firstBullet = section3.match(/^-\s+(.+)/m);
    if (firstBullet) {
      measurement = firstBullet[1].replace(/\*\*/g, '').trim();
    } else {
      // Prose paragraph — take first sentence
      const firstPara = section3.match(/^(.+?)(?:\.\s|\.$)/m);
      if (firstPara) {
        measurement = firstPara[1].replace(/\*\*/g, '').trim();
        if (!measurement.endsWith('.')) measurement += '.';
      }
    }
  }

  records.push({
    id:          fm.id,
    category:    categoryKey,
    phase:       fm.phase || '',
    title:       fm.title || '',
    description: description,
    target:      fm.slo || '',
    measurement: measurement,
    modules:     fm.module || '',
    verify:      fm.verification || '',
    priority:    fm.priority || '',
    owner:       fm.owner || '',
    created:     fm.created || '',
    related_tasks: Array.isArray(fm.related_tasks) ? fm.related_tasks : [],
    path:        relative(REPO_ROOT, file),
  });
}

// Deterministic sort by module then id
records.sort((a, b) => {
  if (a.modules !== b.modules) return a.modules.localeCompare(b.modules);
  return a.id.localeCompare(b.id, 'en', { numeric: true });
});

mkdirSync(OUT_DIR, { recursive: true });
const payload = {
  schema_version: 'v1',
  generated_at_marker: 'deterministic',
  count: records.length,
  nfrs: records,
};
writeFileSync(OUT_FILE, JSON.stringify(payload, null, 2) + '\n', 'utf8');
console.log(`✓ wrote ${records.length} NFRs → ${relative(REPO_ROOT, OUT_FILE)}`);
