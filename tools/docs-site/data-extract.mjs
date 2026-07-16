#!/usr/bin/env node
// tools/docs-site/data-extract.mjs
// TASK-DOCS-001 §1 #1 — extract data from task markdown frontmatter to JSON
// Walks docs/tasks/**/TASK-*.md (excluding .audit.md), parses YAML
// frontmatter, emits tools/docs-site/data/tasks.json sorted deterministically by task id.

import { readFileSync, writeFileSync, readdirSync, mkdirSync, statSync } from 'node:fs';
import { join, resolve, dirname, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(__dirname, '..', '..');
const TASK_DIR    = join(REPO_ROOT, 'docs', 'tasks');
const OUT_DIR   = join(__dirname, 'data');
const OUT_FILE  = join(OUT_DIR, 'tasks.json');

// ─────────────────────────────────────────────────────────────────────────────
// Minimal YAML-flow parser for the frontmatter shapes we actually use.
// Handles: scalars, quoted strings, flow lists `[a, b, c]`, inline comments,
// `null`, ISO dates as strings. No anchors, no block scalars beyond what the
// task template uses.
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

// Underscore-prefixed dirs under docs/tasks/ are META, not the live backlog: _archive/
// (superseded specs) and _audits/ (audit evidence). They are NOT catalog input.
//
// Nothing excluded them before. _archive/ was skipped purely because its files were named
// Ids that /^TASK-.*\.md$/ did not match — an accident of naming, not a rule. The
// 2026-07-15 rename moved them to TASK-*, they started matching, and the catalog jumped
// 509 -> 529 with TASK-APP-001 listed TWICE: once live, once from its archived copy.
// _audits/ survived only on the .audit.md suffix check — one coincidence deep.
//
// The comment on the flat-file branch read "none post-migration", which was true when it
// was written. An exclusion that depends on how a file happens to be NAMED is not an
// exclusion; it is a coincidence with a shelf life.
const META_DIR = /^_/;

function walkTasks(dir) {
  const out = [];
  for (const name of readdirSync(dir)) {
    if (META_DIR.test(name)) continue;              // _archive/, _audits/ — never catalog input
    const p = join(dir, name);
    const st = statSync(p);
    if (st.isDirectory()) {
      if (/^TASK-/.test(name)) {                     // TASK-DOCS-004 folder-per-task
        const spec = join(p, 'spec.md');
        try { if (statSync(spec).isFile()) { out.push(spec); continue; } } catch {}
      }
      out.push(...walkTasks(p));
    } else if (/^TASK-.*\.md$/.test(name) && !name.endsWith('.audit.md')) {
      out.push(p);                                  // legacy flat file (none post-migration)
    }
  }
  return out;
}

const files = walkTasks(TASK_DIR).sort();
const records = [];
for (const file of files) {
  const md = readFileSync(file, 'utf8');
  const fm = parseFrontmatter(md);
  if (!fm || !fm.id) {
    console.warn(`skip (no frontmatter): ${relative(REPO_ROOT, file)}`);
    continue;
  }
  const relParts = file.split('/').slice(-3);   // <module>/<STEM>/spec.md (TASK-DOCS-005 links)
  records.push({
    dir_module:   relParts[0],
    stem:         relParts[1],
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
    related_tasks:  Array.isArray(fm.related_tasks) ? fm.related_tasks : [],
    path:         relative(REPO_ROOT, file),
  });
}

// Deterministic sort by task id (TASK-AI-001 < TASK-AI-002 < TASK-AUTH-001 ...).
// Use module then numeric id for stable ordering even if id strings differ.
records.sort((a, b) => {
  if (a.module !== b.module) return a.module.localeCompare(b.module);
  return a.id.localeCompare(b.id, 'en', { numeric: true });
});

mkdirSync(OUT_DIR, { recursive: true });
const payload = {
  schema_version: 'v1',
  generated_at_marker: 'deterministic',  // TASK-DOCS-001 §1 #3 — no Date.now()
  count: records.length,
  tasks: records,
};
writeFileSync(OUT_FILE, JSON.stringify(payload, null, 2) + '\n', 'utf8');
console.log(`✓ wrote ${records.length} tasks → ${relative(REPO_ROOT, OUT_FILE)}`);
