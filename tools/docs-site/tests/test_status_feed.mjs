#!/usr/bin/env node
// tools/docs-site/tests/test_status_feed.mjs — TASK-DOCS-010 / TASK-DOCS-011 unit tests.
// Committed fixtures cover: epoch-duplicate markers, shorthand + slash-list resolution,
// ledger precedence, exempt classes, dangling deps, no-git degrade, ssh/https remotes,
// sparse tags. Schema validation uses a minimal required-keys check (no ajv dep).
import { readFileSync, writeFileSync, mkdirSync, rmSync, existsSync, mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';
import {
  parseRemoteUrl, idsInText, classifyCommit, bucketByEpoch, loadLedger,
  isExemptSubject, buildStatusFeed, phaseGroup,
} from '../status-feed.mjs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const SCHEMA = JSON.parse(readFileSync(join(__dirname, 'status-feed.schema.json'), 'utf8'));
const TMP = mkdtempSync(join(tmpdir(), 'status-feed-'));
let pass = 0, fail = 0;
const ok = (n) => { pass++; console.log(`  ok   ${n}`); };
const bad = (n, m) => { fail++; console.log(`  FAIL ${n}: ${m}`); };

function assertSchema(data, name) {
  for (const k of SCHEMA.required) {
    if (!(k in data)) return bad(name, `missing required key ${k}`);
  }
  if (!/^fp-[0-9a-f]{12}$/.test(data.fp)) return bad(name, `bad fp ${data.fp}`);
  if (!Array.isArray(data.tasks) || !Array.isArray(data.releases)) return bad(name, 'tasks/releases not arrays');
  ok(name);
}

function byIdMap(ids) {
  return new Map(ids.map(i => [i, { i }]));
}

// --- remote parsing ---
{
  const ssh = parseRemoteUrl('git@github.com:CyberSkill/cyberos.git');
  const https = parseRemoteUrl('https://github.com/CyberSkill/cyberos.git');
  const sshAlt = parseRemoteUrl('ssh://git@github.com/CyberSkill/cyberos.git');
  (ssh === 'https://github.com/CyberSkill/cyberos'
    && https === ssh && sshAlt === ssh
    && parseRemoteUrl('') === ''
    && parseRemoteUrl('not-a-url') === '')
    ? ok('remote_ssh_https') : bad('remote_ssh_https', `${ssh} / ${https}`);
}

// --- shorthand + slash lists ---
{
  const by = byIdMap(['TASK-IMP-122', 'TASK-TEN-002', 'TASK-TEN-004', 'TASK-OBS-004']);
  const a = idsInText('feat: IMP-122 done', by);
  const b = idsInText('batch TEN-002/004 + OBS-004', by);
  const c = idsInText('ghost XYZ-999 and TASK-GHOST-001', by);
  (a.includes('TASK-IMP-122')
    && b.includes('TASK-TEN-002') && b.includes('TASK-TEN-004') && b.includes('TASK-OBS-004')
    && c.includes('TASK-GHOST-001') && !c.includes('TASK-XYZ-999'))
    ? ok('shorthand_slash_lists') : bad('shorthand_slash_lists', JSON.stringify({ a, b, c }));
}

// --- ledger precedence + via ---
{
  const by = byIdMap(['TASK-IMP-145', 'TASK-IMP-146']);
  const prefixes = new Set(['IMP']);
  const ledger = { bb161013: ['TASK-IMP-145', 'TASK-IMP-146'] };
  const linked = classifyCommit(
    { h: 'bb161013', d: '2026-07-20', s: 'feat: something without id', b: '' },
    by, prefixes, ledger,
  );
  const short = classifyCommit(
    { h: 'aaaaaaaa', d: '2026-07-20', s: 'fix(IMP-145): x', b: '' },
    by, prefixes, {},
  );
  const canon = classifyCommit(
    { h: 'bbbbbbbb', d: '2026-07-20', s: 'feat: (TASK-IMP-145)', b: '' },
    by, prefixes, {},
  );
  (linked.k === 'linked' && linked.via === 'ledger'
    && short.k === 'linked' && short.via === 'shorthand'
    && canon.k === 'linked' && !canon.via)
    ? ok('ledger_precedence_via') : bad('ledger_precedence_via', JSON.stringify({ linked, short, canon }));
}

// --- exempt classes ---
{
  const cases = [
    'chore(release): 1.2.3',
    'chore(web): rebuild served bundle @ 1.2.3 [skip ci]',
    'Merge branch \'main\' into x',
    'Revert "feat: foo"',
    'fixup! wip',
    'squash! tidy',
    'amend! note',
  ];
  cases.every(isExemptSubject) && !isExemptSubject('feat: real change')
    ? ok('exempt_classes') : bad('exempt_classes', 'exempt ladder mismatch');
}

// --- epoch duplicate markers + rollback fold ---
{
  const relByV = {
    '1.0.0': { v: '1.0.0', d: '2026-07-01', notes: { features: [], fixes: [], improvements: [] } },
  };
  const commits = [
    { h: 'c1', d: '2026-07-20', s: 'feat: after', ids: [], k: 'unlinked' },
    { h: 'c2', d: '2026-07-19', s: 'chore(release): 1.0.0', ids: [], k: 'exempt' },
    { h: 'c3', d: '2026-07-18', s: 'feat: mid', ids: ['TASK-X-001'], k: 'linked' },
    { h: 'c4', d: '2026-07-17', s: 'chore(release): 1.0.0', ids: [], k: 'exempt' }, // duplicate marker
    { h: 'c5', d: '2026-07-16', s: 'feat: older same version', ids: [], k: 'unlinked' },
    { h: 'c6', d: '2026-07-12', s: 'chore(release): roll back to 0.1.0', ids: [], k: 'exempt' },
    { h: 'c7', d: '2026-07-01', s: 'feat: first epoch', ids: [], k: 'unlinked' },
  ];
  const { releases, unreleasedCov } = bucketByEpoch(commits, relByV);
  const cur = releases.find(r => r.v === '1.0.0');
  const leg = releases.find(r => r.lg === 1);
  (unreleasedCov.commits.some(c => c.h === 'c1')
    && cur && cur.cov.commits.map(c => c.h).join(',') === 'c2,c3,c4,c5,c6'
    && leg && leg.cov.commits.map(c => c.h).join(',') === 'c7')
    ? ok('epoch_duplicate_markers') : bad('epoch_duplicate_markers', JSON.stringify(releases.map(r => ({ v: r.v, hs: r.cov.commits.map(c => c.h) }))));
}

// --- loadLedger unknown task / hash ---
{
  const by = byIdMap(['TASK-IMP-145']);
  const text = 'deadbeef: [TASK-IMP-145, TASK-NOPE-001]\nbb161013: [TASK-IMP-145]\n';
  const known = new Set(['bb161013']);
  const loaded = loadLedger(text, by, known);
  (loaded.unknownTasks.some(u => u.id === 'TASK-NOPE-001')
    && loaded.unknownHashes.includes('deadbeef')
    && loaded.ledger.bb161013?.includes('TASK-IMP-145')
    && loaded.ledger.deadbeef?.includes('TASK-IMP-145')
    && !loaded.ledger.deadbeef?.includes('TASK-NOPE-001'))
    ? ok('ledger_unknown_task_hash') : bad('ledger_unknown_task_hash', JSON.stringify(loaded));
}

// --- phase groups ---
{
  (phaseGroup('P3 — ship') === 'P3'
    && phaseGroup('pre-1.0.0 cleanup') === 'PRE'
    && phaseGroup('1.2 track') === 'POST'
    && phaseGroup('module-wave-2') === 'TRACK'
    && phaseGroup('') === null)
    ? ok('phase_groups') : bad('phase_groups', 'pg mapping');
}

// --- integration: dangling deps + no-git + sparse tags via buildStatusFeed ---
rmSync(TMP, { recursive: true, force: true });
mkdirSync(join(TMP, 'docs/tasks/aa/TASK-AA-001-first'), { recursive: true });
mkdirSync(join(TMP, 'docs/tasks/bb/TASK-BB-001-second'), { recursive: true });
mkdirSync(join(TMP, 'docs/tasks/_state'), { recursive: true });
mkdirSync(join(TMP, 'modules'), { recursive: true });
writeFileSync(join(TMP, 'VERSION'), '0.0.1\n');
writeFileSync(join(TMP, 'CHANGELOG.md'), '# CL\n\n## [0.0.1] - 2026-07-01\n\nAdded\n- TASK-AA-001 landed\n');
writeFileSync(join(TMP, 'modules/manifest.yaml'), 'modules:\n  - id: aa\n    kind: product\n  - id: bb\n    kind: product\n');
writeFileSync(join(TMP, 'docs/tasks/_state/commit-links.yaml'), 'abababab: [TASK-AA-001]\n');
const hubTasks = [
  {
    i: 'TASK-AA-001', k: 'TASK-AA-001-first', dm: 'aa', t: 'First', m: 'aa', c: 'feature',
    p: 'MUST', s: 'done', ph: 'P0', o: 'Ada', cr: '2026-06-01', sh: '2026-07-01', e: '2',
    d: ['TASK-GHOST-999'], b: ['TASK-BB-001'], rl: [], sm: 'First summary',
  },
  {
    i: 'TASK-BB-001', k: 'TASK-BB-001-second', dm: 'bb', t: 'Second', m: 'bb', c: 'feature',
    p: 'SHOULD', s: 'draft', ph: 'P1', o: '', cr: '2026-06-02', sh: '', e: '',
    d: ['TASK-AA-001'], b: [], rl: [], sm: 'Second summary',
  },
];

{
  const feed = buildStatusFeed({
    root: TMP, project: 'Scratch', version: '0.0.1', tasks: hubTasks,
    lenient: true, snapshotFallback: '2026-07-01',
    warn: () => {}, fail: (m) => { throw new Error(m); },
  });
  // no .git under TMP → empty coverage, noGit flag
  (feed.noGit === true
    && feed.head === ''
    && feed.tags.length === 0
    && feed.unreleased.cov.commits.length === 0
    && feed.tasks.find(t => t.i === 'TASK-AA-001').d.includes('TASK-GHOST-999')
    && Array.isArray(feed.ghosts) && feed.ghosts.some(g => g.includes('TASK-GHOST-999')))
    ? ok('no_git_dangling_deps') : bad('no_git_dangling_deps', JSON.stringify({
      noGit: feed.noGit, head: feed.head, tags: feed.tags.length,
      ghosts: feed.ghosts, d: feed.tasks[0]?.d,
    }));
  assertSchema(feed, 'schema_no_git');
}

// with git: sparse tags + identity
{
  const G = (args) => execFileSync('git', args, { cwd: TMP, stdio: ['ignore', 'pipe', 'pipe'] });
  G(['init', '-q']);
  G(['config', 'user.email', 't@t']);
  G(['config', 'user.name', 't']);
  G(['config', 'commit.gpgsign', 'false']);
  G(['add', '-A']);
  G(['commit', '-qm', 'feat: seed (TASK-AA-001)']);
  G(['tag', 'v0.0.1']);
  G(['remote', 'add', 'origin', 'git@github.com:Acme/scratch.git']);
  const feed = buildStatusFeed({
    root: TMP, project: 'Scratch', version: '0.0.1', tasks: hubTasks,
    lenient: true, snapshotFallback: '2026-07-01',
    warn: () => {}, fail: (m) => { throw new Error(m); },
  });
  (feed.noGit === undefined
    && feed.tags.includes('v0.0.1')
    && feed.repoUrl === 'https://github.com/Acme/scratch'
    && feed.head.length >= 7
    && feed.unreleased.cov.commits.length + feed.releases.reduce((n, r) => n + r.cov.commits.length, 0) >= 1)
    ? ok('sparse_tags_git_identity') : bad('sparse_tags_git_identity', JSON.stringify({
      tags: feed.tags, repoUrl: feed.repoUrl, head: feed.head, noGit: feed.noGit,
    }));
  assertSchema(feed, 'schema_with_git');

  // double-run byte-identical feed
  const feed2 = buildStatusFeed({
    root: TMP, project: 'Scratch', version: '0.0.1', tasks: hubTasks,
    lenient: true, snapshotFallback: '2026-07-01',
    warn: () => {}, fail: (m) => { throw new Error(m); },
  });
  JSON.stringify(feed) === JSON.stringify(feed2)
    ? ok('feed_double_run') : bad('feed_double_run', 'feed bytes diverged');
}

// unknown ledger task fails (strict)
{
  writeFileSync(join(TMP, 'docs/tasks/_state/commit-links.yaml'), 'cccccccc: [TASK-NOPE-001]\n');
  let threw = false;
  try {
    buildStatusFeed({
      root: TMP, project: 'Scratch', version: '0.0.1', tasks: hubTasks,
      lenient: false, warn: () => {}, fail: (m) => { throw new Error(m); },
    });
  } catch (e) {
    threw = /TASK-NOPE-001/.test(e.message);
  }
  threw ? ok('ledger_unknown_task_fails') : bad('ledger_unknown_task_fails', 'did not fail');
}

rmSync(TMP, { recursive: true, force: true });
console.log('----');
console.log(`pass=${pass} fail=${fail}`);
process.exit(fail === 0 ? 0 : 1);
