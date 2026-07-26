#!/usr/bin/env node
// docs/status-v3-preview/build.mjs — PREVIEW-ONLY data snapshot builder.
//
// This script is NOT wired into any hook, workflow, or generator. It exists so the
// preview renders against real repo data instead of invented numbers. It reads:
//   1. docs/tasks/<module>/TASK-*/spec.md   (frontmatter: status, deps, dates, ...)
//   2. CHANGELOG.md                         (release sections + cited task ids)
//   3. git log                              (commit -> task traceability, release buckets)
//   4. VERSION, modules/manifest.yaml
// and emits index.html with one embedded JSON corpus (id="sv3-data").
//
// If approved, this logic would fold into tools/docs-site/render-status-hub.mjs as
// status-feed@1; the page itself (app.css/app.js) is generator-agnostic.
//
// Usage: node docs/status-v3-preview/build.mjs [repoRoot] [outDir]

import { readFileSync, writeFileSync, readdirSync, existsSync } from 'node:fs';
import { execSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(process.argv[2] || resolve(__dirname, '..', '..'));
const OUT = resolve(process.argv[3] || __dirname);

const TASKID = /\bTASK-[A-Z][A-Z0-9]*-\d+\b/g;
const ISODATE = /^\d{4}-\d{2}-\d{2}$/;
const STATUSES = ['draft', 'ready_to_implement', 'implementing', 'ready_to_review', 'reviewing',
  'ready_to_test', 'testing', 'done', 'on_hold', 'closed', 'cannot_reproduce', 'duplicate'];
const ACTIVE = ['ready_to_implement', 'implementing', 'ready_to_review', 'reviewing', 'ready_to_test', 'testing'];
const TERMINAL = ['done', 'closed', 'cannot_reproduce', 'duplicate'];

const bucketOf = s => TERMINAL.includes(s) ? 'done'
  : ACTIVE.includes(s) ? 'active'
  : s === 'on_hold' ? 'hold' : 'draft';

const warn = m => console.warn('  ! ' + m);
const day = s => { const m = String(s ?? '').match(/\d{4}-\d{2}-\d{2}/); return m ? m[0] : null; };

/* ---- minimal frontmatter parser (scalars, quoted, inline + block lists, block scalars) */
function scalar(v) {
  const s = String(v).trim();
  const q = s.match(/^(["'])([\s\S]*)\1\s*(?:#.*)?$/);
  if (q) return q[2].trim();
  return s.replace(/\s+#.*$/, '').trim();
}
function frontmatter(text, file) {
  if (!text.startsWith('---')) { warn(`no frontmatter in ${file}`); return null; }
  const end = text.indexOf('\n---', 3);
  if (end === -1) { warn(`unterminated frontmatter in ${file}`); return null; }
  const meta = {};
  let key = null, fold = null;
  for (const raw of text.slice(3, end).split('\n')) {
    if (fold && /^\s+\S/.test(raw)) {
      meta[key] = (meta[key] ? meta[key] + (fold === '|' ? '\n' : ' ') : '') + raw.trim();
      continue;
    }
    fold = null;
    const item = raw.match(/^\s*-\s+(.*)$/);
    if (item && key !== null && Array.isArray(meta[key])) { meta[key].push(scalar(item[1])); continue; }
    const kv = raw.match(/^([A-Za-z_][\w-]*):\s*(.*)$/);
    if (!kv) continue;
    key = kv[1];
    const v = kv[2].trim();
    if (v === '') { meta[key] = []; continue; }
    if (/^[|>][-+]?$/.test(v)) { fold = v[0]; meta[key] = ''; continue; }
    if (/^\[.*\]$/.test(v)) {
      meta[key] = v.slice(1, -1).split(',').map(scalar).filter(x => x && x !== 'null');
      key = null; continue;
    }
    meta[key] = scalar(v);
    key = null;
  }
  return { meta, body: text.slice(end + 4) };
}
const str = v => (Array.isArray(v) ? '' : String(v ?? '')).trim();
const list = v => Array.isArray(v) ? v.map(x => String(x).trim()).filter(Boolean)
  : (str(v) && str(v) !== 'null' ? [str(v)] : []);

/* ---- first prose paragraph of a spec body, markdown flattened ------------------------ */
function summarize(body) {
  const lines = body.split('\n');
  const out = [];
  let inFence = false;
  for (const raw of lines) {
    const l = raw.trim();
    if (/^```/.test(l)) { inFence = !inFence; continue; }
    if (inFence) continue;
    if (!l) { if (out.length) break; continue; }
    if (/^([#>|]|[-*]\s|\d+\.\s|\||<!--)/.test(l)) { if (out.length) break; continue; }
    out.push(l);
    if (out.join(' ').length > 400) break;
  }
  let s = out.join(' ')
    .replace(/`([^`]*)`/g, '$1')
    .replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
    .replace(/\*\*?|__/g, '')
    .replace(/\s+/g, ' ').trim();
  if (s.length > 250) s = s.slice(0, 247).replace(/\s+\S*$/, '') + '…';
  return s;
}

/* ---- 1. tasks ------------------------------------------------------------------------ */
console.log('reading tasks…');
const tasksDir = join(ROOT, 'docs', 'tasks');
const tasks = [];
for (const mod of readdirSync(tasksDir, { withFileTypes: true })) {
  if (!mod.isDirectory() || mod.name.startsWith('_') || mod.name.startsWith('.')) continue;
  for (const t of readdirSync(join(tasksDir, mod.name), { withFileTypes: true })) {
    if (!t.isDirectory() || !/^TASK-/.test(t.name)) continue;
    const p = join(tasksDir, mod.name, t.name, 'spec.md');
    if (!existsSync(p)) continue;
    const parsed = frontmatter(readFileSync(p, 'utf8'), p);
    if (!parsed) continue;
    const { meta, body } = parsed;
    const id = str(meta.id) || t.name.match(/^TASK-[A-Z0-9]+-\d+/)?.[0];
    if (!id) { warn(`no id in ${p}`); continue; }
    const status = str(meta.status);
    if (!STATUSES.includes(status)) warn(`unknown status "${status}" in ${id}`);
    const dep = [...new Set(list(meta.depends_on).flatMap(x => x.match(TASKID) || []))];
    const blk = [...new Set(list(meta.blocks).flatMap(x => x.match(TASKID) || []))];
    const rel = [...new Set(list(meta.related_tasks).flatMap(x => x.match(TASKID) || []))];
    tasks.push({
      i: id, k: `${mod.name}/${t.name}`, t: str(meta.title) || id,
      m: mod.name, c: str(meta.type) || 'feature', p: str(meta.priority) || '',
      s: status, b: bucketOf(status),
      ph: str(meta.phase), o: str(meta.owner) || str(meta.author).replace(/^@/, ''),
      cr: day(meta.created) || day(meta.created_at), sh: day(meta.shipped),
      e: parseFloat(str(meta.effort_hours)) || null,
      d: dep, bl: blk, rl: rel, sm: summarize(body),
    });
  }
}
tasks.sort((a, b) => a.i < b.i ? -1 : 1);
const BY = Object.fromEntries(tasks.map(t => [t.i, t]));
console.log(`  ${tasks.length} tasks`);

/* Task references appear in three forms in commits and notes:
     canonical  TASK-IMP-122
     shorthand  IMP-122, OBS-004, docs(IMP-122):
     lists      TEN-002/004, IMP-127 / 128 / 129
   Shorthand only counts when it resolves to a task that actually exists. */
const PREFIXES = new Set(tasks.map(t => t.i.split('-')[1]));
const SHORTID = /\b([A-Z]{2,10})-(\d{1,4})((?:\s*\/\s*\d{1,4})*)/g;
function idsInText(txt) {
  const out = new Set();
  const s = String(txt || '');
  for (const m of s.match(TASKID) || []) if (BY[m]) out.add(m); else out.add(m); // keep ghosts visible
  for (const m of s.matchAll(SHORTID)) {
    const pre = m[1];
    if (pre === 'TASK' || !PREFIXES.has(pre)) continue;
    const nums = [m[2]].concat(m[3] ? m[3].split('/').map(x => x.trim()).filter(Boolean) : []);
    for (const n of nums) {
      const id3 = `TASK-${pre}-${String(n).padStart(3, '0')}`;
      const idRaw = `TASK-${pre}-${n}`;
      if (BY[id3]) out.add(id3);
      else if (BY[idRaw]) out.add(idRaw);
    }
  }
  return [...out];
}

/* dangling dep references (archived or renamed ids) are kept but flagged */
let dangling = 0;
for (const t of tasks) for (const d of t.d) if (!BY[d]) dangling++;
if (dangling) console.log(`  ${dangling} dependency refs point outside the live corpus (ghosts)`);

/* phase grouping: P0..P5 are the program lanes; module waves/phases fold into TRACK */
const phaseGroup = ph => !ph ? null
  : /^P(\d)\b/.test(ph) ? 'P' + ph.match(/^P(\d)\b/)[1]
  : /^pre-1\.0\.0/i.test(ph) ? 'PRE'
  : /^(post-|\d+\.\d+)/i.test(ph) ? 'POST'
  : 'TRACK';
for (const t of tasks) t.pg = phaseGroup(t.ph);

/* ---- 2. modules + cross-module dependency edges -------------------------------------- */
console.log('aggregating modules…');
let kinds = {};
try {
  const man = readFileSync(join(ROOT, 'modules', 'manifest.yaml'), 'utf8');
  let cur = null;
  for (const l of man.split('\n')) {
    const mId = l.match(/^\s*-\s*id:\s*(\S+)/); if (mId) { cur = mId[1]; continue; }
    const mk = l.match(/^\s*kind:\s*(\S+)/); if (mk && cur) kinds[cur] = mk[1];
  }
} catch { warn('modules/manifest.yaml not readable'); }

const modules = {};
for (const t of tasks) {
  const m = modules[t.m] ||= { id: t.m, kind: kinds[t.m] || '', total: 0, done: 0, active: 0, hold: 0, draft: 0 };
  m.total++; m[t.b] = (m[t.b] || 0) + 1;
}
const edgeMap = {};
for (const t of tasks) for (const d of t.d) {
  const dm = BY[d]?.m; if (!dm || dm === t.m) continue;
  const k = dm + '' + t.m;
  const e = edgeMap[k] ||= { from: dm, to: t.m, w: 0, pairs: [] };
  e.w++; if (e.pairs.length < 8) e.pairs.push(d + ' → ' + t.i);
}
const medges = Object.values(edgeMap).sort((a, b) => b.w - a.w);
console.log(`  ${Object.keys(modules).length} modules, ${medges.length} cross-module edges`);

/* deterministic constellation layout: size-sorted golden-angle spiral + relaxation */
const mlist = Object.values(modules).sort((a, b) => b.total - a.total);
const CX = 490, CY = 315;
mlist.forEach((m, idx) => {
  m.r = Math.min(56, 15 + Math.sqrt(m.total) * 3.4);
  const a = idx * 2.39996; // golden angle
  const rad = idx === 0 ? 0 : 34 * Math.sqrt(idx + 2.5);
  m.x = CX + rad * Math.cos(a); m.y = CY + rad * Math.sin(a) * 0.72;
});
for (let it = 0; it < 420; it++) {
  for (let a = 0; a < mlist.length; a++) for (let b = a + 1; b < mlist.length; b++) {
    const A = mlist[a], B = mlist[b];
    const dx = B.x - A.x, dy = B.y - A.y;
    const dist = Math.max(1, Math.hypot(dx, dy)), min = A.r + B.r + 26;
    if (dist < min) {
      const push = (min - dist) / 2, ux = dx / dist, uy = dy / dist;
      A.x -= ux * push; A.y -= uy * push; B.x += ux * push; B.y += uy * push;
    }
  }
  for (const m of mlist) {
    m.x = Math.min(980 - m.r - 8, Math.max(m.r + 8, m.x));
    m.y = Math.min(622 - m.r - 8, Math.max(m.r + 14, m.y));
  }
}
for (const m of mlist) { m.x = Math.round(m.x); m.y = Math.round(m.y); m.r = Math.round(m.r); }

/* ---- 3. phases (program lanes with date spans) ---------------------------------------- */
const PH_LABEL = {
  P0: 'P0', P1: 'P1', P2: 'P2', P3: 'P3', P4: 'P4', P5: 'P5',
  PRE: 'pre-1.0.0', TRACK: 'module tracks', POST: 'post-release',
};
const PH_ORDER = ['P0', 'P1', 'P2', 'P3', 'P4', 'P5', 'PRE', 'TRACK', 'POST'];
const phases = [];
for (const pg of PH_ORDER) {
  const members = tasks.filter(t => t.pg === pg);
  if (!members.length) continue;
  const done = members.filter(t => t.b === 'done').length;
  const dates = members.map(t => t.cr).filter(Boolean).sort();
  const ships = members.map(t => t.sh).filter(Boolean).sort();
  phases.push({
    id: pg, label: PH_LABEL[pg], total: members.length, done,
    active: members.filter(t => t.b === 'active').length,
    start: dates[0] || null, lastShip: ships[ships.length - 1] || null,
  });
}

/* ---- 4. changelog releases ------------------------------------------------------------ */
console.log('parsing CHANGELOG.md…');
const CATMAP = { Added: 'features', Fixed: 'fixes' };  // everything else → improvements
const relByV = {};
const relOrder = [];
let unreleasedNotes = { features: [], fixes: [], improvements: [] };
{
  const lines = readFileSync(join(ROOT, 'CHANGELOG.md'), 'utf8').split('\n');
  let cur = null, cat = 'improvements';
  for (const raw of lines) {
    const l = raw.trim();
    const rh = l.match(/^##\s+\[?(\d+\.\d+\.\d+)\]?(?:\s*-\s*(\d{4}-\d{2}-\d{2}))?/);
    if (rh) {
      cur = relByV[rh[1]] = { v: rh[1], d: rh[2] || null, notes: { features: [], fixes: [], improvements: [] } };
      relOrder.push(rh[1]); cat = 'improvements'; continue;
    }
    if (/^##\s+Unreleased/i.test(l)) { cur = { notes: unreleasedNotes }; cat = 'improvements'; continue; }
    const ch = l.match(/^#{0,3}\s*(Added|Changed|Fixed|Deprecated|Removed|Security|Docs)\s*$/);
    if (ch) { cat = CATMAP[ch[1]] || 'improvements'; continue; }
    if (/^-\s+/.test(l) && cur) {
      const text = l.replace(/^-\s+/, '');
      cur.notes[cat].push({ x: text, ids: idsInText(text) });
    }
  }
}
console.log(`  ${relOrder.length} releases in changelog`);

/* ---- 5. git commits → release buckets + traceability ---------------------------------- */
console.log('reading git log…');
const EXEMPT = [
  /^chore\(release\):/, /^chore\(web\): rebuild/, /\[skip ci\]/i,
  /^Merge (branch|pull request|remote)/,
];

/* reviewed backfill ledger: hash -> task ids, for commits whose link genuinely never
   made it into the message. History is never rewritten; the ledger is the fix. */
const ledger = {};
try {
  const lf = readFileSync(join(ROOT, 'docs', 'tasks', '_state', 'commit-links.yaml'), 'utf8');
  for (const line of lf.split('\n')) {
    const m = line.match(/^\s*([0-9a-f]{7,40})\s*:\s*\[([^\]]*)\]/);
    if (!m) continue;
    const ids = (m[2].match(TASKID) || []).filter(id => BY[id] ||
      (warn(`commit-links.yaml cites unknown ${id} — dropped`), false));
    if (ids.length) ledger[m[1].slice(0, 8)] = ids;
  }
  if (Object.keys(ledger).length) console.log(`  backfill ledger: ${Object.keys(ledger).length} commits`);
} catch { /* no ledger yet — fine */ }

let commits = [];
try {
  const raw = execSync('git log --date=short --format=%h%x01%ad%x01%s%x01%b%x02', {
    cwd: ROOT, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024,
  });
  commits = raw.split('\x02').map(c => c.trim()).filter(Boolean).map(c => {
    const [h, d, s, b] = c.split('\x01');
    const canon = ((s || '') + ' ' + (b || '')).match(TASKID) || [];
    let ids = idsInText((s || '') + ' ' + (b || ''));
    let via = canon.length ? 'id' : ids.length ? 'shorthand' : null;
    const led = ledger[(h || '').slice(0, 8)];
    if (led) { ids = [...new Set(ids.concat(led))]; via = via || 'ledger'; }
    const k = ids.length ? 'linked' : EXEMPT.some(re => re.test(s || '')) ? 'exempt' : 'unlinked';
    return { h, d, s: (s || '').slice(0, 130), ids, k, ...(via && via !== 'id' ? { via } : {}) };
  });
} catch { warn('git log unavailable — commit coverage will be empty'); }

/* The repo rolled its version back to 0.1.0 on 2026-07-12, so version strings like
   1.9.0 occur in TWO epochs. Bucket by marker occurrence walking newest → oldest;
   everything older than the rollback commit folds into one "first epoch" entry. */
const covOf = () => ({ commits: [], linked: 0, exempt: 0, unlinked: 0 });
const unreleasedCov = covOf();
const emptyNotes = () => ({ features: [], fixes: [], improvements: [] });
const seq = [];            // current-epoch releases in true chronological order (newest first)
const seqByV = {};
const legacyRel = { v: 'first-epoch', label: 'first epoch', d: null, lg: 1, notes: emptyNotes(), cov: covOf() };
let bucket = unreleasedCov;
let legacy = false;
for (const c of commits) {
  const rm = c.s.match(/^chore\(release\): (\d+\.\d+\.\d+)/);
  if (!legacy && rm) {
    const v = rm[1];
    if (seqByV[v]) bucket = seqByV[v].cov;      // same-version retry markers merge
    else {
      const r = { v, d: relByV[v]?.d || c.d, rc: c.h, notes: relByV[v]?.notes || emptyNotes(), cov: covOf() };
      seqByV[v] = r; seq.push(r); bucket = r.cov;
    }
  }
  bucket.commits.push(c);
  bucket[c.k]++;
  if (!legacy && /^chore\(release\): roll back/.test(c.s)) {
    legacy = true;                              // every OLDER commit belongs to the first epoch
    legacyRel.d = c.d;
    bucket = legacyRel.cov;
  }
}
/* changelog sections that never had a marker commit still get a card, in version order */
relOrder.forEach(v => {
  if (!seqByV[v]) { const r = { v, d: relByV[v].d, notes: relByV[v].notes, cov: covOf() }; seqByV[v] = r; seq.push(r); }
});
const releases = seq.concat(legacyRel.cov.commits.length ? [legacyRel] : []);
const totCommits = commits.length;
const totLinked = commits.filter(c => c.k === 'linked').length;
const totUnlinked = commits.filter(c => c.k === 'unlinked').length;
console.log(`  ${totCommits} commits: ${totLinked} linked, ${totUnlinked} unlinked, rest exempt`);

/* ---- 6. burn-up (cumulative shipped tasks by day) ------------------------------------- */
const shipDays = {};
for (const t of tasks) if (t.sh && ISODATE.test(t.sh)) shipDays[t.sh] = (shipDays[t.sh] || 0) + 1;
let cum = 0;
const burnup = Object.keys(shipDays).sort().map(d => ({ d, n: (cum += shipDays[d]) }));

/* ---- 7. assemble ----------------------------------------------------------------------- */
const VERSION = (() => { try { return readFileSync(join(ROOT, 'VERSION'), 'utf8').trim(); } catch { return '?'; } })();
let snapshot = new Date().toISOString().slice(0, 10), head = '', branch = '', repoUrl = '', tags = [];
try {
  head = execSync('git log -1 --format=%h', { cwd: ROOT, encoding: 'utf8' }).trim();
  snapshot = execSync('git log -1 --date=short --format=%ad', { cwd: ROOT, encoding: 'utf8' }).trim();
  branch = execSync('git branch --show-current', { cwd: ROOT, encoding: 'utf8' }).trim();
  tags = execSync('git tag', { cwd: ROOT, encoding: 'utf8' }).split('\n').map(t => t.trim()).filter(Boolean);
  const remote = execSync('git remote get-url origin', { cwd: ROOT, encoding: 'utf8' }).trim();
  const gm = remote.match(/(?:git@|https:\/\/)([^:/]+)[:/](.+?)(?:\.git)?$/);
  if (gm) repoUrl = `https://${gm[1]}/${gm[2]}`;
} catch { /* keep wall clock, no links */ }

const data = {
  project: 'CyberOS', version: VERSION, snapshot, head, branch, repoUrl, tags,
  statuses: STATUSES,
  specBase: '../tasks/',
  rule: 'Every change introduced by a commit must be linked to one or more tasks and reflected on the Status page and in the Release Notes. No exceptions.',
  enforcement: 'Advisory today (commit-msg hook nudges Conventional Commits only). Proposed: commit-msg gate + CI range check + release tag block when unlinked > 0.',
  tasks, modules: mlist, medges, phases, releases,
  unreleased: { notes: unreleasedNotes, cov: unreleasedCov },
  burnup,
};
const json = JSON.stringify(data).replace(/</g, '\\u003c');
const fp = 'fp-' + createHash('sha256').update(json).digest('hex').slice(0, 12);
data.fp = fp;
const json2 = JSON.stringify(data).replace(/</g, '\\u003c');

const html = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>CyberOS · Status — v3 preview</title>
<meta name="description" content="Standalone preview of the redesigned CyberOS status page. Not integrated; data is a build-time snapshot.">
<link rel="icon" href="../status/assets/favicon.svg">
<link rel="stylesheet" href="app.css">
</head>
<body data-theme="paper">
<div id="app">
<noscript><p style="padding:2rem;font-family:sans-serif">This preview needs JavaScript.
The live status page (with a no-JS fallback) remains at <a href="../status/index.html">docs/status/</a>.</p></noscript>
</div>
<script type="application/json" id="sv3-data">${json2}</script>
<script src="app.js"></script>
</body>
</html>
`;
writeFileSync(join(OUT, 'index.html'), html);
console.log(`wrote index.html (${(html.length / 1024).toFixed(0)} KB, ${fp}, snapshot ${snapshot} @ ${head})`);
