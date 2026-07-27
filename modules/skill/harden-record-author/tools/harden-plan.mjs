#!/usr/bin/env node
// harden-plan.mjs - read an inspection-report@1 and emit the ordered work plan
// that /harden must present at its HRD-HITL-1 plan gate.
//
// Does three things a human should not do by hand:
//   1. orders findings per HRD-SEQ-1 (next action first, then timeline class,
//      then severity, then effort ascending)
//   2. classifies each finding as agent / operator / split per HRD-ACT-1
//   3. surfaces stated dependencies and same-change instructions per HRD-SEQ-2
//
// Read-only. Emits a plan; changes nothing and works nothing.
//
// Usage: node harden-plan.mjs <report.md> [--json] [--include-deferred]

import fs from 'node:fs';

const VERSION = '1.0.1';

const TIMELINE_ORDER = [
  'Immediate', 'Before-production', 'Short', 'Medium', 'Long',
];
const NEVER_AUTO = new Set([
  'Experimental', 'Deferred', 'Not-recommended', 'Requires-research',
  'Requires-human-decision', 'Requires-specialist-review',
]);
const SEVERITY_ORDER = ['Critical', 'High', 'Medium', 'Low', 'Opportunity'];
const EFFORT_ORDER = ['Trivial', 'Small', 'Medium', 'Large', 'Strategic', 'Unknown'];

// Phrases that mean the remediation leaves the repository. Kept explicit rather
// than clever: a wrong classification here is a safety failure, so the list is
// auditable and errs toward operator.
const OPERATOR_SIGNALS = [
  [/\brotate\b/i, 'requires rotating a credential at its provider'],
  [/\bpurge\b.*\bhistory\b|\bhistory\b.*\bpurge\b|\brewrit\w*\b.*\bhistory\b/i, 'requires rewriting git history'],
  [/force[- ]push/i, 'requires a force push'],
  [/branch protection|repository settings/i, 'requires a repository settings change'],
  [/\bprovider\b.*\bconsole\b|\bdashboard\b/i, 'requires a change in a provider console'],
  [/record(ed)? (an )?(explicit )?accepted-risk/i, 'requires a recorded operator risk decision'],
  [/\blicence\b|\blicense\b/i, 'requires a licensing decision'],
  [/coordinat\w+ with/i, 'requires coordination outside the repository'],
];
// Phrases that mean part of the work is still a file change.
const AGENT_SIGNALS = [
  /\badd\b/i, /\bremove\b/i, /\breplace\b/i, /\bpin\b/i, /\bchange\b/i,
  /\bdelete\b/i, /\bcorrect\b/i, /\binvert\b/i, /\bwiden\b/i, /\bscope\b/i,
  /\bwrap\b/i, /\bconfigure\b/i, /\bgenerate\b/i, /\bcommit\b/i, /\bmove\b/i,
  /\bpoint\b/i, /\bset\b/i, /\bupdate\b/i, /\benable\b/i, /\bdisable\b/i,
  /\bdrop\b/i, /\braise\b/i, /\bwrite\b/i, /\bextract\b/i, /\brename\b/i,
  /\bfold\b/i, /\bcollapse\b/i, /\bexport\b/i, /\bconvert\b/i, /\brefuse\b/i,
];

function stripComment(line) {
  let inS = false, inD = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    if (c === "'" && !inD) inS = !inS;
    else if (c === '"' && !inS) inD = !inD;
    else if (c === '#' && !inS && !inD && (i === 0 || line[i - 1] === ' ')) return line.slice(0, i);
  }
  return line;
}
function unquote(v) {
  const t = v.trim();
  if (t.length >= 2 && ((t[0] === '"' && t.at(-1) === '"') || (t[0] === "'" && t.at(-1) === "'"))) return t.slice(1, -1);
  return t;
}
function miniYaml(block) {
  const data = {};
  let key = null;
  for (const raw of block.split('\n')) {
    const line = stripComment(raw).replace(/\s+$/, '');
    if (!line.trim()) continue;
    const indent = line.match(/^ */)[0].length;
    const body = line.trim();
    if (indent === 0) {
      const m = body.match(/^([A-Za-z_][A-Za-z0-9_-]*):(?:\s*(.*))?$/);
      if (!m) continue;
      key = m[1];
      const rest = (m[2] ?? '').trim();
      if (rest === '') data[key] = null;
      else if (rest.startsWith('[') && rest.endsWith(']')) {
        const inner = rest.slice(1, -1).trim();
        data[key] = inner === '' ? [] : inner.split(',').map((s) => unquote(s));
      } else data[key] = unquote(rest);
      continue;
    }
    if (!key) continue;
    const li = body.match(/^-\s+(.*)$/);
    if (li) {
      if (data[key] === null) data[key] = [];
      if (Array.isArray(data[key])) {
        const item = li[1].trim();
        const kv = item.match(/^([A-Za-z_][A-Za-z0-9_-]*):\s+(.*)$/);
        data[key].push(kv ? { [kv[1]]: unquote(kv[2]) } : unquote(item));
      }
    }
  }
  return data;
}

function parseFindings(text) {
  const out = [];
  const re = /```(?:yaml|yml)\r?\n([\s\S]*?)```/g;
  let m;
  while ((m = re.exec(text)) !== null) {
    if (!/^id:\s*INS-F-/m.test(m[1])) continue;
    const d = miniYaml(m[1]);
    if (d.strength === 'true' || d.strength === true) continue;
    out.push(d);
  }
  return out;
}

function hasOperatorPrerequisites(f) {
  // INS-FIND-4: a non-empty operator_prerequisites other than an explicit "none"
  // means work outside the repository (or outside agent reach). Must not classify
  // as pure agent — shopass INS-F-0002 is the regression fixture.
  const raw = f.operator_prerequisites;
  if (raw == null) return false;
  const s = String(Array.isArray(raw) ? raw.join(' ') : raw).trim();
  if (!s) return false;
  return !/^(none|n\/a|na|-|nil|null)$/i.test(s);
}

function classify(f) {
  const text = [f.remediation, f.rollback, f.acceptance_criteria, f.operator_prerequisites]
    .filter(Boolean).join(' ');
  const operatorHit = OPERATOR_SIGNALS.find(([re]) => re.test(text));
  const approval = String(f.approval_required ?? 'no').toLowerCase() === 'yes';
  const review = f.review_required && String(f.review_required).toLowerCase() !== 'none';
  const agentHit = AGENT_SIGNALS.some((re) => re.test(String(f.remediation ?? '')));
  const opPrereq = hasOperatorPrerequisites(f);

  let actor = 'agent';
  let why = 'remediation is a change to tracked files';
  if (operatorHit) {
    if (agentHit) { actor = 'split'; why = `${operatorHit[1]}, and also changes files`; }
    else { actor = 'operator'; why = operatorHit[1]; }
  }
  // Honour explicit operator_prerequisites even when remediation prose looks
  // like a file change (HRD-ACT / INS-FIND-4).
  if (opPrereq) {
    const prereqWhy = `operator_prerequisites: ${String(Array.isArray(f.operator_prerequisites) ? f.operator_prerequisites.join('; ') : f.operator_prerequisites).trim()}`;
    if (agentHit || actor === 'agent') {
      actor = 'split';
      why = prereqWhy;
    } else if (actor === 'operator') {
      why = prereqWhy;
    } else {
      // already split from OPERATOR_SIGNALS; prefer the explicit field as reason
      why = prereqWhy;
    }
  }
  // An irreversible rollback forces at least operator involvement (HRD-SAFE-1).
  // "not advisable" in a rollback field means "do not undo this", which is the
  // opposite of irreversible. Only these four actually mean the change cannot
  // be taken back without coordination.
  const irreversible = /none applicable|one-way|requires coordination|only with the deployment offline/i.test(String(f.rollback ?? ''));
  if (irreversible && actor === 'agent') { actor = 'split'; why = 'rollback field describes the change as irreversible'; }

  const gates = [];
  if (approval) gates.push('approval_required');
  if (review) gates.push(`review_required: ${f.review_required}`);
  if (irreversible) gates.push('irreversible');
  if (opPrereq) gates.push('operator_prerequisites');
  return { actor, why, gates };
}

function sortKey(f) {
  const t = String(f.timeline_class ?? 'Long');
  const ti = TIMELINE_ORDER.indexOf(t);
  const si = SEVERITY_ORDER.indexOf(String(f.severity ?? 'Low'));
  const ei = EFFORT_ORDER.indexOf(String(f.effort ?? 'Unknown'));
  return [ti === -1 ? 99 : ti, si === -1 ? 99 : si, ei === -1 ? 99 : ei, String(f.id)];
}

function statedLinks(f, ids) {
  const text = [f.priority, f.remediation, f.related_contract].filter(Boolean).join(' ');
  const found = new Set();
  for (const other of ids) {
    if (other === f.id) continue;
    if (text.includes(other)) found.add(other);
  }
  const sameChange = /same change|in the same change|fold into|take .*in the same/i.test(text);
  return { links: [...found], sameChange };
}

const args = process.argv.slice(2);
const json = args.includes('--json');
const includeDeferred = args.includes('--include-deferred');
const file = args.find((a) => !a.startsWith('--'));
if (!file) {
  console.error('usage: node harden-plan.mjs <report.md> [--json] [--include-deferred]');
  process.exit(2);
}

const text = fs.readFileSync(file, 'utf8');
const findings = parseFindings(text);
if (findings.length === 0) { console.error(`no INS-FIND-1 records found in ${file}`); process.exit(2); }

const na = text.match(/^NEXT-ACTION:\s*(INS-F-\d{4})\s+(\S+)\s*$/m);
if (!na) { console.error('no NEXT-ACTION line; the report is not /harden-ready'); process.exit(2); }
const [, naId, naFp] = na;
const naFinding = findings.find((f) => f.id === naId);
if (!naFinding) { console.error(`NEXT-ACTION names ${naId}, which is not in the register`); process.exit(2); }
if (naFinding.fingerprint !== naFp) { console.error(`NEXT-ACTION fingerprint does not match ${naId}`); process.exit(2); }

const ids = findings.map((f) => f.id);
const rows = findings.map((f) => {
  const c = classify(f);
  const l = statedLinks(f, ids);
  return {
    id: f.id, fingerprint: f.fingerprint, title: f.title,
    severity: f.severity, effort: f.effort, timeline: f.timeline_class,
    discipline: f.primary_discipline,
    actor: c.actor, actor_reason: c.why, gates: c.gates,
    links: l.links, same_change: l.sameChange,
    deferred: NEVER_AUTO.has(String(f.timeline_class)),
  };
});

const workable = rows.filter((r) => includeDeferred || !r.deferred);
const rest = workable.filter((r) => r.id !== naId).sort((a, b) => {
  const ka = sortKey(findings.find((f) => f.id === a.id));
  const kb = sortKey(findings.find((f) => f.id === b.id));
  for (let i = 0; i < ka.length; i++) { if (ka[i] < kb[i]) return -1; if (ka[i] > kb[i]) return 1; }
  return 0;
});
const ordered = [rows.find((r) => r.id === naId), ...rest];
const held = rows.filter((r) => r.deferred && !includeDeferred);

const counts = ordered.reduce((a, r) => { a[r.actor] = (a[r.actor] ?? 0) + 1; return a; }, {});
const blockedFirst = ordered[0].actor !== 'agent';

if (json) {
  console.log(JSON.stringify({ file, version: VERSION, next_action: { id: naId, fingerprint: naFp }, ordered, held, counts, agent_blocked_at_entry: blockedFirst }, null, 2));
} else {
  console.log(`harden-plan ${VERSION} - ${file}`);
  console.log(`${ordered.length} workable, ${held.length} held, actors: ${Object.entries(counts).map(([k, v]) => `${k} ${v}`).join(', ')}\n`);
  ordered.forEach((r, i) => {
    const tag = i === 0 ? 'NEXT' : String(i).padStart(4);
    console.log(`${tag}  ${r.id}  ${String(r.actor).padEnd(8)} ${String(r.severity).padEnd(8)} ${String(r.effort).padEnd(8)} ${String(r.timeline).padEnd(18)} ${r.discipline}`);
    console.log(`      ${r.title}`);
    if (r.actor !== 'agent') console.log(`      actor: ${r.actor_reason}`);
    if (r.gates.length) console.log(`      gates: ${r.gates.join('; ')}`);
    if (r.links.length) console.log(`      ${r.same_change ? 'same change as' : 'references'}: ${r.links.join(', ')}`);
  });
  if (held.length) {
    console.log(`\nHeld (never worked without an explicit instruction naming the finding):`);
    for (const r of held) console.log(`      ${r.id}  ${r.timeline}  ${r.title}`);
  }
  if (blockedFirst) {
    console.log(`\nEntry point is ${ordered[0].actor}, not agent. /harden cannot start on its own; the operator acts first.`);
  }
}
