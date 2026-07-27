#!/usr/bin/env node
// inspect-taxonomy-stats.mjs - aggregate coverage ledgers across N reports.
//
// Answers the step-5 feedback-loop questions with data instead of opinion:
//   - which rows never collected a finding in any inspection
//   - which rows were never applicable in any inspection
//   - which rows carry findings often enough to deserve splitting
//   - which related_disciplines pairs recur, suggesting a boundary problem
//
// Emits a human table, optional --json, and optional --emit-taxonomy to write
// a pruned "ID Name" override file for inspect-lint.mjs --taxonomy.
//
// Deliberately does NOT change anything. Demotion and promotion are decisions
// for a human with enough samples; this tool only reports the counts and the
// sample size they rest on.
//
// Usage: node inspect-taxonomy-stats.mjs report1.md report2.md ... [--json]
//                                        [--min-samples N] [--emit-taxonomy out.txt]

import fs from 'node:fs';

const MIN_SAMPLES_DEFAULT = 10; // the audit's benchmark before acting on a trend

function parseLedger(text) {
  const lines = text.split('\n');
  for (let i = 0; i < lines.length - 1; i++) {
    const low = lines[i].toLowerCase();
    if (!lines[i].includes('|')) continue;
    if (!(low.includes('discipline') && low.includes('applicability'))) continue;
    if (!/^\s*\|?[\s:|-]+\|?\s*$/.test(lines[i + 1] || '')) continue;
    const rows = [];
    for (let j = i + 2; j < lines.length; j++) {
      const rl = lines[j];
      if (!rl.includes('|') || rl.trim() === '') break;
      let cells = rl.split('|').map((c) => c.trim());
      if (cells[0] === '') cells = cells.slice(1);
      if (cells.at(-1) === '') cells = cells.slice(0, -1);
      if (cells.length < 7) continue;
      const idM = cells[0].match(/\b([A-Z]+-\d{2})\b/);
      if (!idM) continue;
      rows.push({
        id: idM[1],
        name: cells[0].replace(idM[0], '').replace(/^[\s:.-]+/, '').trim(),
        applicable: !cells[2].toUpperCase().startsWith('NOT APPLICABLE'),
        state: cells[3].trim().toUpperCase().replace(/[\s-]+/g, '_'),
        count: /^\d+$/.test(cells[4]) ? Number(cells[4]) : 0,
        related: (cells[6] || '').split(',').map((s) => s.trim()).filter((s) => /^[A-Z]+-\d{2}$/.test(s)),
      });
    }
    return rows;
  }
  return null;
}

const args = process.argv.slice(2);
const json = args.includes('--json');
let minSamples = MIN_SAMPLES_DEFAULT;
const ms = args.indexOf('--min-samples');
if (ms !== -1) minSamples = Number(args[ms + 1]);
const et = args.indexOf('--emit-taxonomy');
const emitPath = et !== -1 ? args[et + 1] : null;
const flagValues = new Set();
if (ms !== -1) flagValues.add(args[ms + 1]);
if (et !== -1) flagValues.add(args[et + 1]);
const files = args.filter((a) => !a.startsWith('--') && !flagValues.has(a));

if (files.length === 0) {
  console.error('usage: node inspect-taxonomy-stats.mjs <report.md>... [--json] [--min-samples N] [--emit-taxonomy out.txt]');
  process.exit(2);
}

const stat = new Map(); // id -> aggregate
const pairs = new Map(); // "A|B" -> count
const perReport = [];

for (const f of files) {
  const rows = parseLedger(fs.readFileSync(f, 'utf8'));
  if (!rows) { console.error(`no ledger found in ${f}`); process.exit(2); }
  const pairsThisReport = new Set(); // an unordered pair counts once per report
  perReport.push({ file: f, rows: rows.length, applicable: rows.filter((r) => r.applicable).length });
  for (const r of rows) {
    if (!stat.has(r.id)) {
      stat.set(r.id, { id: r.id, name: r.name, seen: 0, applicable: 0, findings: 0, reportsWithFinding: 0, states: new Map() });
    }
    const s = stat.get(r.id);
    s.seen++;
    if (r.name && !s.name) s.name = r.name;
    if (r.applicable) s.applicable++;
    s.findings += r.count;
    if (r.count > 0) s.reportsWithFinding++;
    s.states.set(r.state, (s.states.get(r.state) ?? 0) + 1);
    for (const other of r.related) {
      const key = [r.id, other].sort().join('|');
      if (pairsThisReport.has(key)) continue;
      pairsThisReport.add(key);
      pairs.set(key, (pairs.get(key) ?? 0) + 1);
    }
  }
}

const all = [...stat.values()];
const n = files.length;
const neverApplicable = all.filter((s) => s.applicable === 0);
const applicableNeverFired = all.filter((s) => s.applicable > 0 && s.findings === 0);
const alwaysApplicable = all.filter((s) => s.applicable === s.seen);
const hot = all.filter((s) => s.findings > 0).sort((a, b) => b.findings - a.findings || b.reportsWithFinding - a.reportsWithFinding);
const recurringPairs = [...pairs.entries()].filter(([, c]) => c >= Math.ceil(n * 0.6)).sort((a, b) => b[1] - a[1]);

const enough = n >= minSamples;

if (json) {
  console.log(JSON.stringify({
    sampleSize: n, minSamples, actionable: enough,
    perReport,
    neverApplicable: neverApplicable.map((s) => s.id),
    applicableNeverFired: applicableNeverFired.map((s) => s.id),
    alwaysApplicable: alwaysApplicable.map((s) => s.id),
    hot: hot.map((s) => ({ id: s.id, findings: s.findings, reports: s.reportsWithFinding })),
    recurringPairs: recurringPairs.map(([k, c]) => ({ pair: k.split('|'), count: c })),
  }, null, 2));
} else {
  console.log(`inspect-taxonomy-stats - ${n} report(s), ${all.length} taxonomy rows\n`);
  for (const p of perReport) console.log(`  ${p.file.replace(/.*report-/, '').replace('.md', '').padEnd(18)} ${p.applicable}/${p.rows} applicable`);

  console.log(`\nRows that collected findings (${hot.length} of ${all.length}):`);
  for (const s of hot) {
    console.log(`  ${s.id.padEnd(12)} ${String(s.findings).padStart(2)} finding(s) across ${s.reportsWithFinding}/${n} report(s)  ${s.name}`);
  }

  console.log(`\nApplicable but never fired (${applicableNeverFired.length}): candidates to watch, not to demote`);
  console.log(`  ${applicableNeverFired.map((s) => s.id).join(', ') || '(none)'}`);

  console.log(`\nNever applicable in any report (${neverApplicable.length}): candidates for conditional status`);
  console.log(`  ${neverApplicable.map((s) => s.id).join(', ') || '(none)'}`);

  console.log(`\nAlways applicable (${alwaysApplicable.length}): the irreducible core of the taxonomy`);
  console.log(`  ${alwaysApplicable.map((s) => s.id).join(', ') || '(none)'}`);

  console.log(`\nRelated-discipline pairs recurring in >=60% of reports (${recurringPairs.length}): possible boundary ambiguity`);
  for (const [k, c] of recurringPairs.slice(0, 12)) console.log(`  ${k.replace('|', ' <-> ').padEnd(28)} ${c}/${n}`);

  console.log(`\nVerdict: sample size ${n}, threshold ${minSamples}. ${enough ? 'Actionable.' : 'NOT actionable - report only, change nothing.'}`);
}

if (emitPath) {
  if (!enough) { console.error(`\nrefusing to emit a pruned taxonomy at n=${n} (threshold ${minSamples})`); process.exit(1); }
  const keep = all.filter((s) => s.applicable > 0);
  fs.writeFileSync(emitPath, keep.map((s) => `${s.id} ${s.name}`).join('\n') + '\n');
  console.log(`\nwrote ${emitPath}: ${keep.length} rows`);
}
