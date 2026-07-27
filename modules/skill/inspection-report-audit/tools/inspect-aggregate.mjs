#!/usr/bin/env node
// inspect-aggregate.mjs - cluster findings across a set of inspection reports.
//
// A single inspection cannot see a template. A set of them can. This reads the
// finding records out of several reports and groups them so that a defect written
// up N times, because it lives in whatever the repositories were scaffolded from,
// is visible as one item with N instances rather than N unrelated items.
//
// Grouping is by fingerprint suffix rather than by full fingerprint, because the
// same defect in two repositories has the same shape and different paths. The
// suffix after the first :: is the artefact-and-locus pair, which is what recurs.

import fs from 'node:fs';

const VERSION = '1.0.0';

function parseReport(path) {
  const text = fs.readFileSync(path, 'utf8');
  const name = path.replace(/.*inspect-report-/, '').replace(/\.md$/, '');
  const out = [];
  // Finding records are fenced yaml blocks; strengths carry `strength: true`.
  for (const m of text.matchAll(/```yaml\n([\s\S]*?)```/g)) {
    const body = m[1];
    const get = (k) => {
      const r = body.match(new RegExp(`^${k}:\\s*(.+)$`, 'm'));
      return r ? r[1].trim().replace(/^["']|["']$/g, '') : null;
    };
    if (/^strength:\s*true/m.test(body)) continue;
    const id = get('id');
    if (!id) continue;
    out.push({
      repo: name,
      id,
      fingerprint: get('fingerprint'),
      title: get('title'),
      discipline: get('primary_discipline'),
      category: get('category'),
      severity: get('severity'),
      template: get('likely_template_origin'),
      effort: get('effort'),
    });
  }
  return out;
}

const files = process.argv.slice(2).filter((a) => !a.startsWith('--'));
if (!files.length) {
  console.log(`inspect-aggregate ${VERSION}`);
  console.log('usage: node inspect-aggregate.mjs <report.md>... [--json]');
  process.exit(2);
}

const all = files.flatMap(parseReport);
const repos = [...new Set(all.map((f) => f.repo))];

// Group by the locus half of the fingerprint, which is stable across repositories.
const byShape = new Map();
for (const f of all) {
  if (!f.fingerprint) continue;
  const shape = f.fingerprint.split('::')[0];
  if (!byShape.has(shape)) byShape.set(shape, []);
  byShape.get(shape).push(f);
}

const recurring = [...byShape.entries()]
  .map(([shape, fs_]) => ({ shape, instances: fs_, repos: [...new Set(fs_.map((x) => x.repo))] }))
  .filter((g) => g.repos.length > 1)
  .sort((a, b) => b.repos.length - a.repos.length);

const byCategory = new Map();
for (const f of all) {
  const k = f.category || 'uncategorised';
  if (!byCategory.has(k)) byCategory.set(k, new Set());
  byCategory.get(k).add(f.repo);
}

const templated = all.filter((f) => f.template === 'yes');
const templShapes = new Map();
for (const f of templated) {
  const shape = (f.fingerprint || '').split('::')[0];
  if (!templShapes.has(shape)) templShapes.set(shape, new Set());
  templShapes.get(shape).add(f.repo);
}

if (process.argv.includes('--json')) {
  console.log(JSON.stringify({ repos, total: all.length, recurring, templated: templated.length }, null, 1));
  process.exit(0);
}

console.log(`inspect-aggregate ${VERSION} - ${repos.length} report(s), ${all.length} finding(s)\n`);

console.log(`Recurring shapes, present in more than one repository (${recurring.length}):`);
for (const g of recurring) {
  const sevs = [...new Set(g.instances.map((i) => i.severity))].join('/');
  const tmpl = g.instances.some((i) => i.template === 'yes') ? '  [template]' : '';
  console.log(`  ${String(g.repos.length).padStart(2)}x  ${g.shape.slice(0, 46).padEnd(46)} ${sevs.padEnd(16)}${tmpl}`);
  console.log(`       ${g.repos.join(', ')}`);
}

console.log(`\nCategories spanning the most repositories:`);
[...byCategory.entries()]
  .map(([k, v]) => [k, v.size])
  .sort((a, b) => b[1] - a[1])
  .slice(0, 10)
  .forEach(([k, n]) => console.log(`  ${String(n).padStart(2)}/${repos.length}  ${k}`));

if (templShapes.size) {
  console.log(`\nFlagged template-derived, fixable once at the scaffold:`);
  [...templShapes.entries()]
    .sort((a, b) => b[1].size - a[1].size)
    .forEach(([shape, r]) => console.log(`  ${String(r.size).padStart(2)}x  ${shape.slice(0, 48).padEnd(48)} ${[...r].join(', ').slice(0, 60)}`));
  const saved = [...templShapes.values()].reduce((a, s) => a + Math.max(0, s.size - 1), 0);
  const unknown = all.filter((f) => f.template !== 'yes' && f.template !== 'no').length;
  console.log(`\n  ${templated.length} finding(s) flagged template-derived across ${templShapes.size} distinct shape(s).`);
  console.log(`  Fixing each at its scaffold closes ${saved} duplicate write-up(s).`);
  if (unknown) {
    const pct = ((unknown / all.length) * 100).toFixed(0);
    console.log(`\n  Coverage caveat: ${unknown} of ${all.length} findings (${pct}%) carry no template judgement.`);
    console.log(`  Those are records predating the flag. The counts above are a floor, not a total,`);
    console.log(`  and the recurring-shapes table is the more complete view until the flag is backfilled.`);
  }
}
