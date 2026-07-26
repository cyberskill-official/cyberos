#!/usr/bin/env node
// tools/docs-site/status-feed.mjs - status-feed@1 builder (TASK-DOCS-010 / TASK-DOCS-011).
// Folded from the former docs/status-v3-preview/build.mjs into the generator pipeline.
// Pure helpers are exported for unit tests; buildStatusFeed assembles the payload.
import { readFileSync, existsSync, readdirSync, realpathSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { join, resolve } from 'node:path';

export const FEED_VERSION = 1;
export const TASKID = /\bTASK-[A-Z][A-Z0-9]*-\d+\b/g;
export const SHORTID = /\b([A-Z]{2,10})-(\d{1,4})((?:\s*\/\s*\d{1,4})*)/g;
export const ISODATE = /^\d{4}-\d{2}-\d{2}$/;

export const STATUSES = ['draft', 'ready_to_implement', 'implementing', 'ready_to_review', 'reviewing',
  'ready_to_test', 'testing', 'done', 'on_hold', 'closed', 'cannot_reproduce', 'duplicate'];
const ACTIVE = ['ready_to_implement', 'implementing', 'ready_to_review', 'reviewing', 'ready_to_test', 'testing'];
const TERMINAL = ['done', 'closed', 'cannot_reproduce', 'duplicate'];

export const EXEMPT = [
  /^chore\(release\):/,
  /^chore\(web\): rebuild/,
  /\[skip ci\]/i,
  /^Merge (branch|pull request|remote)/,
  /^Revert /,
  /^fixup!/,
  /^squash!/,
  /^amend!/,
];

export const bucketOf = s => TERMINAL.includes(s) ? 'done'
  : ACTIVE.includes(s) ? 'active'
  : s === 'on_hold' ? 'hold' : 'draft';

export const phaseGroup = ph => !ph ? null
  : /^P(\d)\b/.test(ph) ? 'P' + ph.match(/^P(\d)\b/)[1]
  : /^pre-1\.0\.0/i.test(ph) ? 'PRE'
  : /^(post-|\d+\.\d+)/i.test(ph) ? 'POST'
  : 'TRACK';

const day = s => { const m = String(s ?? '').match(/\d{4}-\d{2}-\d{2}/); return m ? m[0] : null; };
const emptyNotes = () => ({ features: [], fixes: [], improvements: [] });
const covOf = () => ({ commits: [], linked: 0, exempt: 0, unlinked: 0 });

/** Derive https://host/owner/repo from ssh or https origin URLs; '' when unparseable. */
export function parseRemoteUrl(remote) {
  const s = String(remote || '').trim();
  if (!s) return '';
  const gm = s.match(/(?:git@|https:\/\/|ssh:\/\/(?:git@)?)([^:/]+)[:/](.+?)(?:\.git)?$/);
  if (!gm) return '';
  return `https://${gm[1]}/${gm[2].replace(/^\/+/, '')}`;
}

/** Classify one commit subject against the exempt ladder (before link resolution). */
export function isExemptSubject(subject) {
  return EXEMPT.some(re => re.test(subject || ''));
}

/**
 * Resolve task ids in free text: canonical TASK-… always kept; shorthand PREFIX-NNN
 * (and slash lists PREFIX-002/004) only when the task exists in `byId`.
 */
export function idsInText(txt, byId, prefixes) {
  const out = new Set();
  const s = String(txt || '');
  for (const m of s.match(TASKID) || []) out.add(m);
  const PREFIXES = prefixes || new Set([...byId.keys()].map(id => id.split('-')[1]).filter(Boolean));
  for (const m of s.matchAll(SHORTID)) {
    const pre = m[1];
    if (pre === 'TASK' || !PREFIXES.has(pre)) continue;
    const nums = [m[2]].concat(m[3] ? m[3].split('/').map(x => x.trim()).filter(Boolean) : []);
    for (const n of nums) {
      const id3 = `TASK-${pre}-${String(n).padStart(3, '0')}`;
      const idRaw = `TASK-${pre}-${n}`;
      if (byId.has(id3)) out.add(id3);
      else if (byId.has(idRaw)) out.add(idRaw);
    }
  }
  return [...out];
}

/**
 * Parse docs/tasks/_state/commit-links.yaml.
 * Returns { ledger: { short8: [taskIds] }, unknownTasks: [], unknownHashes: [] }.
 * unknownTasks fail the build (caller); unknownHashes are warn-only.
 */
export function loadLedger(text, byId, knownHashes) {
  const ledger = {};
  const unknownTasks = [];
  const unknownHashes = [];
  const hashSet = knownHashes instanceof Set ? knownHashes : null;
  for (const line of String(text || '').split('\n')) {
    const m = line.match(/^\s*([0-9a-f]{7,40})\s*:\s*\[([^\]]*)\]/);
    if (!m) continue;
    const full = m[1];
    const short = full.slice(0, 8);
    if (hashSet && ![...hashSet].some(h => h.startsWith(short) || short.startsWith(h.slice(0, short.length)))) {
      unknownHashes.push(full);
    }
    const ids = [];
    for (const id of (m[2].match(TASKID) || [])) {
      if (byId.has(id)) ids.push(id);
      else unknownTasks.push({ hash: full, id });
    }
    if (ids.length) ledger[short] = [...new Set(ids)];
  }
  return { ledger, unknownTasks, unknownHashes };
}

/**
 * Classification ladder: canonical id → shorthand → ledger → exempt → unlinked.
 * `via` is set for non-canonical links (shorthand | ledger).
 */
export function classifyCommit({ h, d, s, b }, byId, prefixes, ledger) {
  const text = (s || '') + ' ' + (b || '');
  const canon = text.match(TASKID) || [];
  let ids = idsInText(text, byId, prefixes);
  let via = canon.length ? 'id' : ids.length ? 'shorthand' : null;
  const led = ledger[(h || '').slice(0, 8)];
  if (led) {
    ids = [...new Set(ids.concat(led))];
    via = via || 'ledger';
  }
  const k = ids.length ? 'linked' : isExemptSubject(s) ? 'exempt' : 'unlinked';
  const row = { h, d, s: (s || '').slice(0, 130), ids, k };
  if (via && via !== 'id') row.via = via;
  return row;
}

/**
 * Bucket commits into release coverage by marker occurrence (newest → oldest).
 * Duplicate version markers merge; "chore(release): roll back" folds older into first-epoch.
 */
export function bucketByEpoch(commits, relByV) {
  const unreleasedCov = covOf();
  const seq = [];
  const seqByV = {};
  const legacyRel = { v: 'first-epoch', label: 'first epoch', d: null, lg: 1, notes: emptyNotes(), cov: covOf() };
  let bucket = unreleasedCov;
  let legacy = false;
  for (const c of commits) {
    const rm = c.s.match(/^chore\(release\): (\d+\.\d+\.\d+)/);
    if (!legacy && rm) {
      const v = rm[1];
      if (seqByV[v]) bucket = seqByV[v].cov;
      else {
        const r = { v, d: relByV[v]?.d || c.d, rc: c.h, notes: relByV[v]?.notes || emptyNotes(), cov: covOf() };
        seqByV[v] = r; seq.push(r); bucket = r.cov;
      }
    }
    bucket.commits.push(c);
    bucket[c.k]++;
    if (!legacy && /^chore\(release\): roll back/.test(c.s)) {
      legacy = true;
      legacyRel.d = c.d;
      bucket = legacyRel.cov;
    }
  }
  for (const v of Object.keys(relByV)) {
    if (!seqByV[v]) {
      const r = { v, d: relByV[v].d, notes: relByV[v].notes, cov: covOf() };
      seqByV[v] = r; seq.push(r);
    }
  }
  const releases = seq.concat(legacyRel.cov.commits.length ? [legacyRel] : []);
  return { releases, unreleasedCov };
}

export function parseChangelogNotes(clText, byId, prefixes) {
  const CATMAP = { Added: 'features', Fixed: 'fixes' };
  const relByV = {};
  const relOrder = [];
  const unreleasedNotes = emptyNotes();
  const lines = String(clText || '').split('\n');
  let cur = null, cat = 'improvements';
  for (const raw of lines) {
    const l = raw.trim();
    const rh = l.match(/^##\s+\[?(\d+\.\d+\.\d+)\]?(?:\s*-\s*(\d{4}-\d{2}-\d{2}))?/);
    if (rh) {
      cur = relByV[rh[1]] = { v: rh[1], d: rh[2] || null, notes: emptyNotes() };
      relOrder.push(rh[1]); cat = 'improvements'; continue;
    }
    if (/^##\s+Unreleased/i.test(l)) { cur = { notes: unreleasedNotes }; cat = 'improvements'; continue; }
    const ch = l.match(/^#{0,3}\s*(Added|Changed|Fixed|Deprecated|Removed|Security|Docs)\s*$/);
    if (ch) { cat = CATMAP[ch[1]] || 'improvements'; continue; }
    if (/^-\s+/.test(l) && cur) {
      const text = l.replace(/^-\s+/, '');
      cur.notes[cat].push({ x: text, ids: idsInText(text, byId, prefixes) });
    }
  }
  return { relByV, relOrder, unreleasedNotes };
}

function git(root, args, maxBuffer = 64 * 1024 * 1024) {
  return execFileSync('git', args, { cwd: root, encoding: 'utf8', maxBuffer, stdio: ['ignore', 'pipe', 'pipe'] });
}

function sameRepoRoot(root, top) {
  try {
    const a = realpathSync(resolve(root));
    const b = realpathSync(resolve(top));
    return a === b;
  } catch {
    return resolve(root) === resolve(top);
  }
}

export function readGitCommits(root) {
  try {
    const top = git(root, ['rev-parse', '--show-toplevel'], 1024 * 1024).trim();
    if (!sameRepoRoot(root, top)) return null; // cwd is not its own repo (parent .git)
    const raw = git(root, ['log', '--date=short', '--format=%h%x01%ad%x01%s%x01%b%x02']);
    return raw.split('\x02').map(c => c.trim()).filter(Boolean).map(c => {
      const [h, d, s, b] = c.split('\x01');
      return { h, d, s: s || '', b: b || '' };
    });
  } catch {
    return null; // no git / not a repo
  }
}

export function readGitIdentity(root) {
  const out = { head: '', branch: '', repoUrl: '', tags: [], snapshot: '' };
  try {
    const top = git(root, ['rev-parse', '--show-toplevel'], 1024 * 1024).trim();
    if (!sameRepoRoot(root, top)) return out;
    out.head = git(root, ['log', '-1', '--format=%h'], 1024 * 1024).trim();
    out.snapshot = git(root, ['log', '-1', '--date=short', '--format=%ad'], 1024 * 1024).trim();
    out.branch = git(root, ['branch', '--show-current'], 1024 * 1024).trim();
    out.tags = git(root, ['tag'], 1024 * 1024).split('\n').map(t => t.trim()).filter(Boolean).sort();
    try {
      out.repoUrl = parseRemoteUrl(git(root, ['remote', 'get-url', 'origin'], 1024 * 1024).trim());
    } catch { /* no origin */ }
  } catch { /* degrade */ }
  return out;
}

/** Commit-set digest for tooling; empty string when git unavailable or root is not toplevel. */
export function gitCommitSetHash(root) {
  const commits = readGitCommits(root);
  if (!commits) return '';
  const h = createHash('sha256');
  for (const c of commits) h.update(`${c.h}\n`);
  return h.digest('hex');
}

function readModuleKinds(root) {
  const kinds = {};
  const man = join(root, 'modules', 'manifest.yaml');
  if (!existsSync(man)) return kinds;
  try {
    let cur = null;
    for (const l of readFileSync(man, 'utf8').split('\n')) {
      const mId = l.match(/^\s*-\s*id:\s*(\S+)/); if (mId) { cur = mId[1]; continue; }
      const mk = l.match(/^\s*kind:\s*(\S+)/); if (mk && cur) kinds[cur] = mk[1];
    }
  } catch { /* ignore */ }
  return kinds;
}

function layoutModules(modules) {
  const mlist = Object.values(modules).sort((a, b) => b.total - a.total || a.id.localeCompare(b.id));
  const CX = 490, CY = 315;
  mlist.forEach((m, idx) => {
    m.r = Math.min(56, 15 + Math.sqrt(m.total) * 3.4);
    const a = idx * 2.39996;
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
  return mlist;
}

/**
 * Build status-feed@1 from an already-parsed hub corpus.
 * @param {object} opts
 * @param {string} opts.root
 * @param {string} opts.project
 * @param {string} opts.version
 * @param {object[]} opts.tasks  hub task objects (i,k,dm,t,m,c,p,s,ph,o,cr,sh,e,d,b,rl,sm)
 * @param {boolean} opts.lenient
 * @param {string} [opts.snapshotFallback] deterministic snapshot when git absent
 * @param {(msg:string)=>void} [opts.warn]
 * @param {(msg:string)=>void} [opts.fail]  throw/die helper
 */
export function buildStatusFeed(opts) {
  const {
    root, project, version, tasks: hubTasks, lenient = false,
    snapshotFallback = '', warn = console.warn, fail = (m) => { throw new Error(m); },
  } = opts;

  const byId = new Map(hubTasks.map(t => [t.i, t]));
  const prefixes = new Set(hubTasks.map(t => t.i.split('-')[1]).filter(Boolean));

  // Map hub tasks → feed shape (b = bucket; bl = blocks). Keep ghosts in d/bl/rl.
  const tasks = hubTasks.map(t => {
    const status = t.s;
    // Match hub posture: unknown status is a WARN (surfaced in the corpus), not a hard fail,
    // so a single bad frontmatter cannot blank the whole status page. Strict fail is available
    // via CYBEROS_FEED_STRICT_STATUS=1.
    if (!STATUSES.includes(status)) {
      warn(`status-feed: WARN unknown status "${status}" on ${t.i}`);
      if (!lenient && process.env.CYBEROS_FEED_STRICT_STATUS === '1') {
        fail(`status-feed: unknown status "${status}" on ${t.i}`);
      }
    }
    const dep = [...new Set((t.d || []).flatMap(x => String(x).match(TASKID) || [x]).filter(Boolean))];
    const blk = [...new Set((t.b || []).flatMap(x => String(x).match(TASKID) || [x]).filter(Boolean))];
    const rel = [...new Set((t.rl || []).flatMap(x => String(x).match(TASKID) || [x]).filter(Boolean))];
    return {
      i: t.i,
      k: t.dm && t.k ? `${t.dm}/${t.k}` : (t.k || t.i),
      t: t.t || t.i,
      m: t.m,
      c: t.c || 'feature',
      p: t.p || '',
      s: status,
      b: bucketOf(status),
      ph: t.ph || '',
      o: String(t.o || '').replace(/^@/, ''),
      cr: day(t.cr) || day(t.ca),
      sh: day(t.sh),
      e: parseFloat(t.e) || null,
      d: dep, bl: blk, rl: rel,
      sm: t.sm || '',
      pg: phaseGroup(t.ph),
    };
  }).sort((a, b) => a.i < b.i ? -1 : 1);

  const feedById = Object.fromEntries(tasks.map(t => [t.i, t]));

  let dangling = 0;
  const ghosts = [];
  for (const t of tasks) for (const d of t.d) {
    if (!feedById[d]) { dangling++; ghosts.push(`${t.i}→${d}`); }
  }
  if (dangling) warn(`status-feed: ${dangling} dependency refs point outside the live corpus (ghosts)`);
  const ghostMax = process.env.CYBEROS_FEED_GHOST_MAX;
  if (ghostMax !== undefined && ghostMax !== '' && dangling > Number(ghostMax)) {
    fail(`status-feed: dangling depends_on ${dangling} exceeds CYBEROS_FEED_GHOST_MAX=${ghostMax}`);
  }

  const kinds = readModuleKinds(root);
  const modules = {};
  for (const t of tasks) {
    const m = modules[t.m] ||= { id: t.m, kind: kinds[t.m] || '', total: 0, done: 0, active: 0, hold: 0, draft: 0 };
    m.total++; m[t.b] = (m[t.b] || 0) + 1;
  }
  const edgeMap = {};
  for (const t of tasks) for (const d of t.d) {
    const dm = feedById[d]?.m; if (!dm || dm === t.m) continue;
    const k = dm + '\x01' + t.m;
    const e = edgeMap[k] ||= { from: dm, to: t.m, w: 0, pairs: [] };
    e.w++; if (e.pairs.length < 8) e.pairs.push(d + ' → ' + t.i);
  }
  const medges = Object.values(edgeMap).sort((a, b) => b.w - a.w || a.from.localeCompare(b.from));
  const mlist = layoutModules(modules);

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

  const clPath = join(root, 'CHANGELOG.md');
  const clText = existsSync(clPath) ? readFileSync(clPath, 'utf8') : '';
  const { relByV, relOrder, unreleasedNotes } = parseChangelogNotes(clText, byId, prefixes);
  // Preserve changelog order for cards that never had a marker
  const orderedRelByV = {};
  for (const v of relOrder) orderedRelByV[v] = relByV[v];

  const rawCommits = readGitCommits(root);
  const noGit = rawCommits === null;
  if (noGit) warn('status-feed: git log unavailable — commit coverage will be empty (no git history available)');

  const knownHashes = new Set((rawCommits || []).map(c => c.h));
  const ledgerPath = join(root, 'docs', 'tasks', '_state', 'commit-links.yaml');
  let ledger = {};
  if (existsSync(ledgerPath)) {
    const loaded = loadLedger(readFileSync(ledgerPath, 'utf8'), byId, knownHashes.size ? knownHashes : null);
    for (const u of loaded.unknownTasks) {
      const msg = `commit-links.yaml cites unknown ${u.id} for ${u.hash}`;
      if (lenient) warn(`status-feed: WARN ${msg}`);
      else fail(`status-feed: ${msg}`);
    }
    for (const h of loaded.unknownHashes) warn(`status-feed: WARN commit-links.yaml hash prefix not in git log: ${h}`);
    ledger = loaded.ledger;
    if (Object.keys(ledger).length) warn(`status-feed: backfill ledger: ${Object.keys(ledger).length} commits`);
  }

  const commits = (rawCommits || []).map(c => classifyCommit(c, byId, prefixes, ledger));
  const { releases, unreleasedCov } = bucketByEpoch(commits, orderedRelByV);

  const shipDays = {};
  for (const t of tasks) if (t.sh && ISODATE.test(t.sh)) shipDays[t.sh] = (shipDays[t.sh] || 0) + 1;
  let cum = 0;
  const burnup = Object.keys(shipDays).sort().map(d => ({ d, n: (cum += shipDays[d]) }));

  const ident = noGit
    ? { head: '', branch: '', repoUrl: '', tags: [], snapshot: snapshotFallback || '' }
    : readGitIdentity(root);

  const data = {
    project,
    version,
    feed: FEED_VERSION,
    snapshot: ident.snapshot || snapshotFallback || '',
    head: ident.head,
    branch: ident.branch,
    repoUrl: ident.repoUrl,
    tags: ident.tags,
    statuses: STATUSES,
    specBase: '../tasks/',
    rule: 'Every change introduced by a commit must be linked to one or more tasks and reflected on the Status page and in the Release Notes. No exceptions.',
    enforcement: 'Advisory today (commit-msg hook + CI range check). Ledger at docs/tasks/_state/commit-links.yaml for retroactive fixes.',
    noGit: noGit || undefined,
    tasks,
    modules: mlist,
    medges,
    phases,
    releases,
    unreleased: { notes: unreleasedNotes, cov: unreleasedCov },
    burnup,
    ghosts: ghosts.length ? ghosts.slice(0, 50) : undefined,
  };
  if (!data.noGit) delete data.noGit;
  if (!data.ghosts) delete data.ghosts;

  const json = JSON.stringify(data);
  data.fp = 'fp-' + createHash('sha256').update(json).digest('hex').slice(0, 12);
  return data;
}

/** List live task ids under docs/tasks (for task-lint / external validators). */
export function listLiveTaskIds(root) {
  const ids = new Set();
  const base = join(root, 'docs', 'tasks');
  if (!existsSync(base)) return ids;
  for (const mod of readdirSync(base, { withFileTypes: true })) {
    if (!mod.isDirectory() || mod.name.startsWith('_') || mod.name.startsWith('.')) continue;
    for (const t of readdirSync(join(base, mod.name), { withFileTypes: true })) {
      if (!t.isDirectory() || !/^TASK-/.test(t.name)) continue;
      const id = t.name.match(/^TASK-[A-Z0-9]+-\d+/)?.[0];
      if (id) ids.add(id);
    }
  }
  return ids;
}
