#!/usr/bin/env node
// inspect-lint.mjs - mechanical output lint for /inspect reports.
// Validates a finished report against the INS contract: coverage ledger
// (INS-DISC-9), non-duplication (INS-DISC-10), finding schema (INS-FIND-1),
// verbatim quotes on VERIFIED claims (INS-GATE-VQ), the recorded
// claim-to-evidence ratio (INS-GATE-CR), the single NEXT-ACTION handoff
// (INS-RPT-9), and the shipped self-audit gate lines (INS-AUDIT-2).
// Zero dependencies. Node >= 18. Exit codes: 0 pass, 1 lint errors, 2 usage.
//
// Usage:
//   node inspect-lint.mjs <report.md> [--json] [--max-cer N] [--taxonomy file]
//   node inspect-lint.mjs --selftest
//
// Known limits (deliberate): the embedded YAML reader covers the INS-FIND-1
// shape only (flat keys, inline arrays, one level of block lists and maps,
// "- key: value" items). Anchors, block scalars (| and >), and deeper nesting
// are parse errors by design; the schema does not use them.

import fs from 'node:fs';
import process from 'node:process';

const VERSION = '1.2.0';

// ---------------------------------------------------------------------------
// Canonical taxonomy (INS-DISC-8): 75 disciplines across 12 clusters at spec ≥1.2
// (TAXONOMY_12). Spec 1.0/1.1 reports are linted against TAXONOMY_10 (69 rows).
// ---------------------------------------------------------------------------
const TAXONOMY_12 = {
  'CORE-01': 'Requirements engineering',
  'CORE-02': 'Domain engineering',
  'CORE-03': 'Systems engineering',
  'CORE-04': 'Architecture engineering',
  'CORE-05': 'Software engineering',
  'CORE-06': 'Repository engineering',
  'CORE-07': 'Configuration engineering',
  'CORE-08': 'Concurrency engineering',
  'CORE-09': 'Distributed systems engineering',
  'PRODUCT-01': 'Product engineering',
  'PRODUCT-02': 'Documentation engineering',
  'PRODUCT-03': 'Content engineering',
  'PRODUCT-04': 'Internationalization and localization engineering',
  'DATA-01': 'Data engineering',
  'DATA-02': 'Database engineering',
  'DATA-03': 'Migration engineering',
  'IFACE-01': 'API engineering',
  'IFACE-02': 'Integration engineering',
  'IFACE-03': 'Event and messaging engineering',
  'SEC-01': 'Security engineering',
  'SEC-02': 'Privacy engineering',
  'SEC-03': 'Identity and access engineering',
  'SEC-04': 'Supply-chain engineering',
  'SEC-05': 'Functional safety engineering',
  'SEC-06': 'Threat modeling engineering',
  'SEC-07': 'Business-logic security engineering',
  'REL-01': 'Reliability engineering',
  'REL-02': 'Resilience engineering',
  'REL-03': 'Performance engineering',
  'REL-04': 'Capacity engineering',
  'REL-05': 'Site reliability engineering',
  'REL-06': 'Observability engineering',
  'REL-07': 'Incident and problem management engineering',
  'DELIVERY-01': 'Platform engineering',
  'DELIVERY-02': 'Infrastructure engineering',
  'DELIVERY-03': 'Cloud engineering',
  'DELIVERY-04': 'Build engineering',
  'DELIVERY-05': 'Release engineering',
  'DELIVERY-06': 'CI/CD engineering',
  'DELIVERY-07': 'Embedded and firmware engineering',
  'DELIVERY-08': 'Repository and build integrity engineering',
  'QUAL-01': 'Test engineering',
  'QUAL-02': 'Quality engineering',
  'QUAL-03': 'Verification and validation engineering',
  'QUAL-04': 'Security testing engineering',
  'EXP-01': 'User experience engineering',
  'EXP-02': 'Accessibility engineering',
  'EXP-03': 'Design system engineering',
  'EXP-04': 'Frontend engineering',
  'EXP-05': 'Backend engineering',
  'EXP-06': 'Client and application engineering',
  'EXP-07': 'Developer experience engineering',
  'EXP-08': 'Package and library engineering',
  'AGENT-01': 'Prompt engineering',
  'AGENT-02': 'Context engineering',
  'AGENT-03': 'Memory engineering',
  'AGENT-04': 'Retrieval engineering',
  'AGENT-05': 'Harness engineering',
  'AGENT-06': 'Evaluation engineering',
  'AGENT-07': 'Agentic engineering',
  'AGENT-08': 'Orchestration engineering',
  'AGENT-09': 'Tool engineering',
  'AGENT-10': 'Workflow engineering',
  'AGENT-11': 'Human-in-the-loop engineering',
  'AIML-01': 'Model and ML engineering',
  'GOV-01': 'Decision engineering',
  'GOV-02': 'Governance engineering',
  'GOV-03': 'Risk engineering',
  'GOV-04': 'Compliance engineering',
  'GOV-05': 'Legal, licensing, and intellectual-property engineering',
  'GOV-06': 'Ethics and responsible-technology engineering',
  'GOV-07': 'Cost, FinOps, and economic engineering',
  'GOV-08': 'Evolution, sustainability, maintenance, and disposal engineering',
  'GOV-09': 'Vulnerability disclosure and patch lifecycle engineering',
  'GOV-10': 'AI governance and impact assessment engineering',
};
// Rows added in specification 1.2. A report declaring 1.0 or 1.1 predates them and
// is checked against the taxonomy that existed when it was written, so an amendment
// cannot retroactively invalidate evidence gathered under the earlier contract.
const SPEC12_ROWS = ['SEC-06', 'SEC-07', 'QUAL-04', 'DELIVERY-08', 'GOV-09', 'GOV-10'];
const TAXONOMY_10 = Object.fromEntries(
  Object.entries(TAXONOMY_12).filter(([id]) => !SPEC12_ROWS.includes(id)),
);
const TAXONOMY = TAXONOMY_12;
const CANON_COUNT = 75;

function taxonomyForSpec(spec) {
  return specAtLeast(spec, '1.2') ? TAXONOMY_12 : TAXONOMY_10;
}

const EVIDENCE_STATES = new Set([
  'VERIFIED', 'STRONG_EVIDENCE', 'SUSPECTED', 'BLOCKED',
  'VERIFIED_ABSENT', 'NOT_FOUND', 'NOT_APPLICABLE',
]);
const SEVERITIES = new Set(['Critical', 'High', 'Medium', 'Low', 'Opportunity']);
const CONFIDENCES = new Set(['High', 'Medium', 'Low']);
const EFFORTS = new Set(['Trivial', 'Small', 'Medium', 'Large', 'Strategic', 'Unknown']);
const TIMELINES = new Set([
  'Immediate', 'Before-production', 'Short', 'Medium', 'Long', 'Experimental',
  'Deferred', 'Not-recommended', 'Requires-research', 'Requires-human-decision',
  'Requires-specialist-review',
]);
const RUN_STATUSES = new Set([
  'new', 'unchanged', 'regressed', 'resolved', 'reopened', 'accepted-risk',
  'false-positive', 'blocked', 'superseded',
]);

const REQUIRED_KEYS = [
  'id', 'fingerprint', 'title', 'primary_discipline', 'category', 'severity',
  'confidence', 'evidence_state', 'evidence', 'affected_scope', 'root_cause',
  'impact_now', 'remediation', 'effort', 'priority', 'timeline_class',
  'acceptance_criteria', 'validation_method', 'regression_gate',
  'owner_discipline', 'run_status',
];
const STRENGTH_REQUIRED_KEYS = [
  'id', 'fingerprint', 'title', 'primary_discipline', 'evidence_state', 'evidence',
];
const SPEC11_REQUIRED_KEYS = ['operator_prerequisites', 'likely_template_origin'];
const SPEC12_REQUIRED_KEYS = ['confidence_band', 'refutation'];
const SPEC12_ABSENCE_KEYS = ['search_space', 'detection_sensitivity'];
// INS-EVD-10 bands, keyed by evidence state.
const CONFIDENCE_BANDS = {
  VERIFIED: [0.95, 1.0],
  STRONG_EVIDENCE: [0.80, 0.95],
  SUSPECTED: [0.40, 0.70],
  VERIFIED_ABSENT: [0.80, 1.0],
};
const TEMPLATE_ORIGIN = new Set(['yes', 'no', 'unknown']);

const RECOMMENDED_KEYS = [
  'risk_future', 'blast_radius', 'likelihood', 'related_contract', 'rollback',
  'review_required', 'approval_required', 'open_questions', 'related_disciplines',
];
const KNOWN_KEYS = new Set([
  ...REQUIRED_KEYS, ...RECOMMENDED_KEYS, ...SPEC11_REQUIRED_KEYS, ...SPEC12_REQUIRED_KEYS, ...SPEC12_ABSENCE_KEYS,
  'alternatives', 'cross_impact', 'strength',
]);
const PLACEHOLDER_FIELDS = [
  'title', 'category', 'affected_scope', 'root_cause', 'impact_now',
  'remediation', 'priority', 'acceptance_criteria', 'validation_method',
  'regression_gate',
];

// ---------------------------------------------------------------------------
// Issue collection.
// ---------------------------------------------------------------------------
function makeCollector() {
  const errors = [];
  const warnings = [];
  return {
    errors,
    warnings,
    err(rule, message, line) { errors.push({ rule, message, line: line ?? null }); },
    warn(rule, message, line) { warnings.push({ rule, message, line: line ?? null }); },
  };
}

// ---------------------------------------------------------------------------
// Minimal YAML reader for INS-FIND-1 records.
// ---------------------------------------------------------------------------
function stripComment(line) {
  let inS = false, inD = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    if (c === "'" && !inD) inS = !inS;
    else if (c === '"' && !inS) inD = !inD;
    else if (c === '#' && !inS && !inD && (i === 0 || line[i - 1] === ' ')) {
      return line.slice(0, i);
    }
  }
  return line;
}
function unquote(v) {
  const t = v.trim();
  if (t.length >= 2 && ((t[0] === '"' && t.at(-1) === '"') || (t[0] === "'" && t.at(-1) === "'"))) {
    return t.slice(1, -1);
  }
  return t;
}
function coerce(v) {
  const t = unquote(v);
  if (t === 'true') return true;
  if (t === 'false') return false;
  return t;
}
function parseInlineArray(v) {
  const inner = v.trim().slice(1, -1).trim();
  if (inner === '') return [];
  return inner.split(',').map((s) => coerce(s));
}
function miniYaml(block, baseLine) {
  const data = {};
  const errors = [];
  let currentKey = null;
  const lines = block.split('\n');
  for (let i = 0; i < lines.length; i++) {
    const lineNo = baseLine + i;
    const line = stripComment(lines[i]).replace(/\s+$/, '');
    if (line.trim() === '') continue;
    const indent = line.match(/^ */)[0].length;
    const body = line.trim();

    if (indent === 0) {
      const m = body.match(/^([A-Za-z_][A-Za-z0-9_-]*):(?:\s*(.*))?$/);
      if (!m) { errors.push({ line: lineNo, message: `unparseable line: ${body}` }); continue; }
      const [, key, restRaw] = m;
      const rest = (restRaw ?? '').trim();
      currentKey = key;
      if (rest === '') data[key] = null;
      else if (rest.startsWith('[') && rest.endsWith(']')) data[key] = parseInlineArray(rest);
      else data[key] = coerce(rest);
      continue;
    }

    if (!currentKey) { errors.push({ line: lineNo, message: 'indented line with no parent key' }); continue; }
    const parent = data[currentKey];

    const li = body.match(/^-\s+(.*)$/);
    if (li) {
      if (parent === null) data[currentKey] = [];
      if (!Array.isArray(data[currentKey])) {
        errors.push({ line: lineNo, message: `list item under non-list key "${currentKey}"` });
        continue;
      }
      const item = li[1].trim();
      const kv = item.match(/^([A-Za-z_][A-Za-z0-9_-]*):\s+(.*)$/);
      data[currentKey].push(kv ? { [kv[1]]: coerce(kv[2]) } : coerce(item));
      continue;
    }

    const kv = body.match(/^([A-Za-z_][A-Za-z0-9_-]*):(?:\s*(.*))?$/);
    if (kv) {
      if (parent === null) data[currentKey] = {};
      if (Array.isArray(data[currentKey]) || typeof data[currentKey] !== 'object') {
        errors.push({ line: lineNo, message: `nested key under non-map key "${currentKey}"` });
        continue;
      }
      data[currentKey][kv[1]] = kv[2] !== undefined && kv[2].trim() !== '' ? coerce(kv[2]) : null;
      continue;
    }
    errors.push({ line: lineNo, message: `unparseable line: ${body}` });
  }
  return { data, errors };
}

// ---------------------------------------------------------------------------
// Extractors.
// ---------------------------------------------------------------------------
function lineOfIndex(text, index) {
  return text.slice(0, index).split('\n').length;
}
function extractFindingBlocks(text) {
  const blocks = [];
  const re = /```(?:yaml|yml)\r?\n([\s\S]*?)```/g;
  let m;
  while ((m = re.exec(text)) !== null) {
    if (/^id:\s*INS-F-/m.test(m[1])) {
      blocks.push({ raw: m[1], line: lineOfIndex(text, m.index) + 1 });
    }
  }
  return blocks;
}
function normalizeState(s) {
  return String(s ?? '').trim().toUpperCase().replace(/[\s-]+/g, '_');
}
function extractLedger(text) {
  const lines = text.split('\n');
  for (let i = 0; i < lines.length - 1; i++) {
    const l = lines[i];
    if (!l.includes('|')) continue;
    const low = l.toLowerCase();
    if (low.includes('discipline') && low.includes('applicability')) {
      if (!/^\s*\|?[\s:|-]+\|?\s*$/.test(lines[i + 1] || '')) continue;
      const rows = [];
      for (let j = i + 2; j < lines.length; j++) {
        const rl = lines[j];
        if (!rl.includes('|') || rl.trim() === '') break;
        let cells = rl.split('|').map((c) => c.trim());
        if (cells[0] === '') cells = cells.slice(1);
        if (cells.at(-1) === '') cells = cells.slice(0, -1);
        rows.push({ cells, line: j + 1 });
      }
      return { headerLine: i + 1, rows };
    }
  }
  return null;
}

// ---------------------------------------------------------------------------
// Check groups.
// ---------------------------------------------------------------------------
// Reports declare their specification version on a machine line (INS-RPT-11).
// Absent, the report is treated as 1.0 and the 1.1 rules are not enforced, so
// reports written against the earlier contract stay valid.
function detectSpec(text) {
  const m = text.match(/^INSPECT-SPEC:\s*([0-9]+\.[0-9]+)\s*$/m);
  return m ? m[1] : '1.0';
}

function specAtLeast(spec, target) {
  const [a, b] = spec.split('.').map(Number);
  const [c, d] = target.split('.').map(Number);
  return a > c || (a === c && b >= d);
}

function checkStructure(text, col, opts) {
  if (!/side-effect disclosure/i.test(text)) {
    col.err('INSL-STR-002', 'no side-effect disclosure section (INS-RPT-1 section 1 is always required, "none" allowed)');
  }
  for (const marker of ['executive summary', 'coverage ledger', 'findings register', 'readiness']) {
    if (!text.toLowerCase().includes(marker)) {
      col.warn('INSL-STR-001', `required section marker not found: "${marker}"`);
    }
  }
  const missing = [];
  let failed = [];
  for (let g = 1; g <= 9; g++) {
    const m = text.match(new RegExp(`^G${g}:\\s*(pass|fail)\\b`, 'mi'));
    if (!m) missing.push(`G${g}`);
    else if (m[1].toLowerCase() === 'fail') failed.push(`G${g}`);
  }
  if (missing.length) col.err('INSL-STR-003', `self-audit gate lines missing: ${missing.join(', ')} (INS-AUDIT-2)`);
  if (failed.length) col.err('INSL-STR-003', `report shipped with failed gate(s): ${failed.join(', ')} (INS-AUDIT-2 forbids shipping on a failed gate)`);
  const cer = text.match(/^CLAIM-EVIDENCE-RATIO:\s*([0-9]+(?:\.[0-9]+)?)\s*$/m);
  if (!cer) {
    col.err('INSL-STR-004', 'CLAIM-EVIDENCE-RATIO line missing or non-numeric (INS-GATE-CR)');
    return { cer: null };
  }
  const ratio = Number(cer[1]);
  if (ratio > opts.maxCer) {
    col.warn('INSL-STR-004', `claim-to-evidence ratio ${ratio} exceeds --max-cer ${opts.maxCer}; claims may ride on too few sources (INS-GATE-CR)`);
  }
  return { cer: ratio };
}

function checkLedger(text, col, taxonomy) {
  const canonIds = Object.keys(taxonomy);
  const ledger = extractLedger(text);
  if (!ledger) {
    col.err('INSL-LED-001', 'coverage ledger table not found (INS-DISC-9)');
    return { rows: [], byId: new Map() };
  }
  const byId = new Map();
  const seenOrder = [];
  for (const row of ledger.rows) {
    if (row.cells.length < 7) {
      col.err('INSL-LED-013', `ledger row has ${row.cells.length} cells, expected 7 (INS-DISC-9)`, row.line);
      continue;
    }
    const [c0, cCluster, cApp, cState, cCount, cPointer, cRelated] = row.cells;
    const idM = c0.match(/\b([A-Z]+-\d{2})\b/);
    if (!idM) { col.err('INSL-LED-004', `no discipline id in ledger cell: "${c0}"`, row.line); continue; }
    const id = idM[1];
    if (!taxonomy[id]) { col.err('INSL-LED-004', `unknown discipline id in ledger: ${id}`, row.line); continue; }
    if (byId.has(id)) { col.err('INSL-LED-003', `duplicate ledger row for ${id}`, row.line); continue; }
    seenOrder.push(id);

    const appU = cApp.toUpperCase();
    let applicability = null;
    if (appU.startsWith('NOT APPLICABLE')) applicability = 'NOT_APPLICABLE';
    else if (appU.startsWith('APPLICABLE')) applicability = 'APPLICABLE';
    else col.err('INSL-LED-006', `${id}: applicability must start with APPLICABLE or NOT APPLICABLE, got "${cApp}"`, row.line);

    const state = normalizeState(cState);
    if (!EVIDENCE_STATES.has(state)) {
      col.err('INSL-LED-008', `${id}: invalid evidence state "${cState}" (INS-EVD-2)`, row.line);
    }
    let count = null;
    if (/^\d+$/.test(cCount)) count = Number(cCount);
    else col.err('INSL-LED-009', `${id}: finding count must be a non-negative integer, got "${cCount}"`, row.line);

    const pointerNone = /^none$/i.test(cPointer.trim());
    if (applicability === 'NOT_APPLICABLE') {
      if (count !== 0 || !pointerNone) {
        col.err('INSL-LED-007', `${id}: NOT APPLICABLE row must have finding count 0 and evidence pointer NONE (INS-DISC-9)`, row.line);
      }
    }
    if ((state === 'VERIFIED' || state === 'STRONG_EVIDENCE') && pointerNone) {
      col.err('INSL-LED-012', `${id}: evidence state ${state} with evidence pointer NONE`, row.line);
    }
    const cluster = id.split('-')[0];
    if (cCluster && cCluster.toUpperCase() !== cluster) {
      col.warn('INSL-LED-010', `${id}: cluster cell "${cCluster}" does not match id prefix ${cluster}`, row.line);
    }
    const name = c0.replace(idM[0], '').replace(/^[\s:.-]+/, '').trim();
    if (name && name.toLowerCase() !== taxonomy[id].toLowerCase()) {
      col.warn('INSL-LED-010', `${id}: name "${name}" differs from canonical "${taxonomy[id]}"`, row.line);
    }
    let related = [];
    const relTrim = (cRelated ?? '').trim();
    if (relTrim && !/^(-|none)$/i.test(relTrim)) {
      related = relTrim.split(',').map((s) => s.trim()).filter(Boolean);
      for (const r of related) {
        if (!taxonomy[r]) col.err('INSL-LED-011', `${id}: related_disciplines contains unknown id "${r}"`, row.line);
      }
    }
    byId.set(id, { id, applicability, state, count, pointer: cPointer, related, line: row.line });
  }

  if (byId.size !== canonIds.length) {
    col.err('INSL-LED-002', `ledger has ${byId.size} valid rows, taxonomy requires ${canonIds.length} (INS-DISC-9)`);
  }
  const missing = canonIds.filter((id) => !byId.has(id));
  if (missing.length) col.err('INSL-LED-003', `ledger missing rows for: ${missing.join(', ')}`);
  const canonOrder = canonIds.filter((id) => byId.has(id));
  for (let i = 0; i < seenOrder.length; i++) {
    if (seenOrder[i] !== canonOrder[i]) {
      col.err('INSL-LED-005', `ledger rows out of stable id order starting at ${seenOrder[i]} (expected ${canonOrder[i]}) (INS-DISC-9)`);
      break;
    }
  }
  return { rows: [...byId.values()], byId };
}

// A template placeholder looks like <finding id> or <number>: lowercase words
// only. Verbatim HTML such as <link rel="icon" /> is not a placeholder, so the
// pattern rejects quotes, slashes, and equals signs.
const PLACEHOLDER_RE = /^<[a-z][a-z0-9 _-]{0,40}>$/;

function hasQuoteEntry(evidence) {
  if (!Array.isArray(evidence)) return false;
  return evidence.some((e) => {
    if (e && typeof e === 'object' && 'quote' in e) {
      const q = String(e.quote ?? '').trim();
      return q.length > 0 && !PLACEHOLDER_RE.test(q);
    }
    return false;
  });
}

function checkFindings(text, col, taxonomy, spec) {
  const blocks = extractFindingBlocks(text);
  const findings = [];
  const idsSeen = new Map();
  const fpsSeen = new Map();

  for (const block of blocks) {
    const { data, errors: perrs } = miniYaml(block.raw, block.line);
    for (const e of perrs) col.err('INSL-FND-001', `yaml: ${e.message}`, e.line);
    const fid = typeof data.id === 'string' ? data.id : null;
    const label = fid ?? `block@${block.line}`;
    const isStrength = data.strength === true;

    const required = isStrength ? STRENGTH_REQUIRED_KEYS : REQUIRED_KEYS;
    const missing = required.filter((k) => !(k in data) || data[k] === null || data[k] === '');
    if (missing.length) col.err('INSL-FND-002', `${label}: missing required key(s): ${missing.join(', ')} (INS-FIND-1)`, block.line);
    if (!isStrength) {
      const rec = RECOMMENDED_KEYS.filter((k) => !(k in data));
      if (rec.length) col.warn('INSL-FND-003', `${label}: recommended key(s) absent: ${rec.join(', ')}`, block.line);
    }
    for (const k of Object.keys(data)) {
      if (!KNOWN_KEYS.has(k)) col.warn('INSL-FND-011', `${label}: unknown key "${k}" (typo?)`, block.line);
    }

    // INSL-FND-012 / 013: spec 1.1 fields. Not enforced against 1.0 reports.
    if (spec !== '1.0' && !isStrength) {
      const miss11 = SPEC11_REQUIRED_KEYS.filter((k) => !(k in data) || data[k] === null || data[k] === '');
      if (miss11.length) col.err('INSL-FND-012', `${label}: spec ${spec} requires ${miss11.join(', ')} (INS-FIND-4, INS-FIND-5)`, block.line);
      const to = data.likely_template_origin;
      if (to != null && !TEMPLATE_ORIGIN.has(String(to))) {
        col.err('INSL-FND-013', `${label}: likely_template_origin must be yes, no, or unknown; got "${to}" (INS-FIND-5)`, block.line);
      }
    }

    // INSL-FND-015/016/017: spec 1.2 verification, confidence banding, absence rigour.
    if (specAtLeast(spec, '1.2') && !isStrength) {
      const miss12 = SPEC12_REQUIRED_KEYS.filter((k) => !(k in data) || data[k] === null || String(data[k]).trim() === '');
      if (miss12.length) col.err('INSL-FND-015', `${label}: spec ${spec} requires ${miss12.join(', ')} (INS-VER-2, INS-EVD-10)`, block.line);

      const st = normalizeState(data.evidence_state);
      const band = CONFIDENCE_BANDS[st];
      const raw = data.confidence_band == null ? '' : String(data.confidence_band).trim();
      if (band && raw) {
        const m = raw.match(/^([0-9]*\.?[0-9]+)\s*-\s*([0-9]*\.?[0-9]+)$/);
        if (!m) col.err('INSL-FND-016', `${label}: confidence_band must read "low-high", got "${raw}" (INS-EVD-10)`, block.line);
        else {
          const lo = Number(m[1]), hi = Number(m[2]);
          if (lo < band[0] - 1e-9 || hi > band[1] + 1e-9) {
            col.err('INSL-FND-016', `${label}: confidence_band ${raw} is outside the ${st} band ${band[0]}-${band[1]} (INS-EVD-10)`, block.line);
          }
        }
      } else if (!band && raw && raw.toLowerCase() !== 'none') {
        col.warn('INSL-FND-016', `${label}: evidence_state ${st} takes no confidence band; got "${raw}" (INS-EVD-10)`, block.line);
      }

      if (st === 'VERIFIED_ABSENT') {
        const missA = SPEC12_ABSENCE_KEYS.filter((k) => !(k in data) || data[k] === null || String(data[k]).trim() === '');
        if (missA.length) col.err('INSL-FND-017', `${label}: VERIFIED_ABSENT requires ${missA.join(', ')} (INS-EVD-9)`, block.line);
      }
    }

    // INSL-FND-014: an absence claim needs two differently shaped searches.
    if (spec !== '1.0' && normalizeState(data.evidence_state) === 'VERIFIED_ABSENT') {
      const n = Array.isArray(data.evidence) ? data.evidence.length : 0;
      if (n < 2) col.err('INSL-FND-014', `${label}: VERIFIED_ABSENT with ${n} evidence entr${n === 1 ? 'y' : 'ies'}; two differently shaped searches are required (INS-EVD-7)`, block.line);
    }

    if (fid) {
      if (!/^INS-F-\d{4}$/.test(fid)) col.err('INSL-FND-004', `${label}: id must match INS-F-NNNN`, block.line);
      if (idsSeen.has(fid)) col.err('INSL-FND-004', `duplicate finding id ${fid} (also at line ${idsSeen.get(fid)})`, block.line);
      idsSeen.set(fid, block.line);
    }
    const fp = typeof data.fingerprint === 'string' ? data.fingerprint.trim() : '';
    if (fp) {
      if (fpsSeen.has(fp)) col.err('INSL-FND-009', `${label}: duplicate fingerprint "${fp}" (also at line ${fpsSeen.get(fp)}); consolidate per INS-DISC-10`, block.line);
      fpsSeen.set(fp, block.line);
    }

    for (const [field, val] of [['primary_discipline', data.primary_discipline], ['owner_discipline', data.owner_discipline]]) {
      if (typeof val === 'string' && val && !taxonomy[val]) {
        col.err('INSL-FND-005', `${label}: ${field} "${val}" is not a canonical discipline id (INS-DISC-8)`, block.line);
      }
    }
    if (Array.isArray(data.related_disciplines)) {
      for (const r of data.related_disciplines) {
        if (!taxonomy[r]) col.err('INSL-FND-005', `${label}: related_disciplines contains unknown id "${r}"`, block.line);
      }
      if (data.related_disciplines.includes(data.primary_discipline)) {
        col.warn('INSL-FND-005', `${label}: primary_discipline repeated in related_disciplines`, block.line);
      }
    }

    const state = normalizeState(data.evidence_state);
    if (data.evidence_state != null && !EVIDENCE_STATES.has(state)) {
      col.err('INSL-FND-006', `${label}: invalid evidence_state "${data.evidence_state}"`, block.line);
    }
    if (!isStrength) {
      if (data.severity != null && !SEVERITIES.has(data.severity)) col.err('INSL-FND-006', `${label}: invalid severity "${data.severity}"`, block.line);
      if (data.confidence != null && !CONFIDENCES.has(data.confidence)) col.err('INSL-FND-006', `${label}: invalid confidence "${data.confidence}"`, block.line);
      if (data.effort != null && !EFFORTS.has(data.effort)) col.err('INSL-FND-006', `${label}: invalid effort "${data.effort}"`, block.line);
      if (data.timeline_class != null && !TIMELINES.has(data.timeline_class)) col.err('INSL-FND-006', `${label}: invalid timeline_class "${data.timeline_class}"`, block.line);
      if (data.run_status != null && !RUN_STATUSES.has(data.run_status)) col.err('INSL-FND-006', `${label}: invalid run_status "${data.run_status}"`, block.line);
    }

    const ev = data.evidence;
    if (state === 'VERIFIED' && !hasQuoteEntry(ev)) {
      col.err('INSL-FND-007', `${label}: VERIFIED without a non-empty verbatim quote entry in evidence (INS-GATE-VQ)`, block.line);
    }
    if ((state === 'VERIFIED' || state === 'STRONG_EVIDENCE') && (!Array.isArray(ev) || ev.length === 0)) {
      col.err('INSL-FND-008', `${label}: evidence_state ${state} with empty evidence (INS-EVD-1)`, block.line);
    } else if ((!Array.isArray(ev) || ev.length === 0) && !isStrength) {
      col.warn('INSL-FND-008', `${label}: evidence list is empty`, block.line);
    }

    for (const f of PLACEHOLDER_FIELDS) {
      const v = data[f];
      if (typeof v !== 'string') continue;
      if (PLACEHOLDER_RE.test(v.trim())) col.err('INSL-FND-010', `${label}: field "${f}" still holds a template placeholder`, block.line);
      else if (/<[a-z][a-z _-]{2,40}>/.test(v) && !/[<>]\s*[a-z]+\s*=/.test(v)) col.warn('INSL-FND-010', `${label}: field "${f}" looks like it contains a placeholder fragment`, block.line);
    }

    findings.push({ data, line: block.line, isStrength, state, fingerprint: fp });
  }
  return findings;
}

function checkNextAction(text, col, findings) {
  const matches = [...text.matchAll(/^NEXT-ACTION:\s*(INS-F-\d{4})\s+(\S+)\s*$/gm)];
  if (matches.length !== 1) {
    col.err('INSL-NXT-001', `expected exactly one NEXT-ACTION line, found ${matches.length} (INS-RPT-9)`);
    return;
  }
  const [, fid, fp] = matches[0];
  const f = findings.find((x) => x.data.id === fid);
  if (!f) { col.err('INSL-NXT-002', `NEXT-ACTION references ${fid}, which is not in the findings register`); return; }
  if (f.fingerprint !== fp) {
    col.err('INSL-NXT-003', `NEXT-ACTION fingerprint "${fp}" does not match ${fid}'s fingerprint "${f.fingerprint}"`);
  }
}

function checkCross(col, ledger, findings) {
  const counts = new Map();
  const countsWithStrengths = new Map();
  for (const f of findings) {
    const pd = f.data.primary_discipline;
    if (typeof pd !== 'string' || !pd) continue;
    countsWithStrengths.set(pd, (countsWithStrengths.get(pd) ?? 0) + 1);
    if (!f.isStrength) counts.set(pd, (counts.get(pd) ?? 0) + 1);
    const row = ledger.byId.get(pd);
    if (row && row.applicability === 'NOT_APPLICABLE') {
      col.err('INSL-CRS-001', `${f.data.id ?? 'finding'}@${f.line}: primary_discipline ${pd} is NOT APPLICABLE in the ledger`, f.line);
    }
  }
  for (const row of ledger.rows) {
    if (row.count === null) continue;
    const actual = counts.get(row.id) ?? 0;
    if (row.count !== actual) {
      const withS = countsWithStrengths.get(row.id) ?? 0;
      if (row.count === withS) {
        col.warn('INSL-CRS-002', `${row.id}: ledger count ${row.count} matches only when strength records are counted as findings; state the convention`, row.line);
      } else {
        col.err('INSL-CRS-002', `${row.id}: ledger finding count ${row.count} does not reconcile with ${actual} register finding(s) whose primary_discipline is ${row.id} (INS-DISC-10)`, row.line);
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Entry points.
// ---------------------------------------------------------------------------
function lintText(text, opts = {}) {
  const col = makeCollector();
  const spec = detectSpec(text);
  const taxonomy = opts.taxonomy ?? taxonomyForSpec(spec);
  const { cer } = checkStructure(text, col, { maxCer: opts.maxCer ?? 3 });

  if (spec !== '1.0') {
    // INSL-STR-005: the reversal ledger (INS-EVD-8).
    if (!/\breversal\b|\breversed\b|\bfalsified\b/i.test(text)) {
      col.err('INSL-STR-005', 'spec 1.1 requires the reversal ledger in the limitations section, or an explicit statement that no hypothesis was reversed (INS-EVD-8)');
    }
    // INSL-STR-007: quality header (INS-RPT-12).
    if (specAtLeast(spec, '1.2')) {
      if (!/^QUALITY-HEADER\s*$/m.test(text)) {
        col.err('INSL-STR-007', 'spec 1.2 requires a QUALITY-HEADER block (INS-RPT-12)');
      } else {
        for (const k of ['coverage', 'evidence', 'verification', 'stability', 'calibration']) {
          if (!new RegExp(`^\\s*${k}:`, 'm').test(text)) {
            col.err('INSL-STR-007', `QUALITY-HEADER is missing the ${k} line (INS-RPT-12)`);
          }
        }
      }
    }

    // INSL-STR-006: banned words in generated prose (INS-RPT-10).
    const banned = opts.banned ?? [];
    // INS-RPT-10 exempts fixed identifiers. The ledger table is discipline names
    // and nothing else, so a term appearing only there is a row name, not prose.
    const prose = text
      .split('\n')
      .filter((l) => !/^\s*\|/.test(l))
      .join('\n');
    for (const w of banned) {
      const re = new RegExp(`\\b${w}\\b`, 'gi');
      const hits = [...prose.matchAll(re)];
      if (hits.length) col.err('INSL-STR-006', `banned term "${w}" appears ${hits.length} time(s) in the report (INS-RPT-10)`);
    }
  }
  const ledger = checkLedger(text, col, taxonomy);
  const findings = checkFindings(text, col, taxonomy, spec);
  checkNextAction(text, col, findings);
  checkCross(col, ledger, findings);
  return {
    spec,
    status: col.errors.length === 0 ? 'pass' : 'fail',
    errors: col.errors,
    warnings: col.warnings,
    stats: {
      taxonomyRows: Object.keys(taxonomy).length,
      ledgerRows: ledger.rows.length,
      applicableRows: ledger.rows.filter((r) => r.applicability === 'APPLICABLE').length,
      findings: findings.filter((f) => !f.isStrength).length,
      strengths: findings.filter((f) => f.isStrength).length,
      claimEvidenceRatio: cer,
    },
  };
}

function loadTaxonomyFile(path) {
  const tax = {};
  for (const raw of fs.readFileSync(path, 'utf8').split('\n')) {
    const line = raw.trim();
    if (!line || line.startsWith('#')) continue;
    const m = line.match(/^([A-Z]+-\d{2})\s*(.*)$/);
    if (!m) throw new Error(`taxonomy file line not "ID Name": ${line}`);
    tax[m[1]] = m[2].trim() || m[1];
  }
  if (Object.keys(tax).length === 0) throw new Error('taxonomy file is empty');
  return tax;
}

function printHuman(result, file) {
  const { errors, warnings, stats } = result;
  console.log(`inspect-lint ${VERSION} - ${file} (spec ${result.spec})`);
  console.log(`taxonomy rows ${stats.taxonomyRows}, ledger rows ${stats.ledgerRows} (${stats.applicableRows} applicable), findings ${stats.findings}, strengths ${stats.strengths}, claim-evidence ratio ${stats.claimEvidenceRatio ?? 'n/a'}`);
  for (const e of errors) console.log(`  ERROR ${e.rule}${e.line ? ` (line ${e.line})` : ''}: ${e.message}`);
  for (const w of warnings) console.log(`  warn  ${w.rule}${w.line ? ` (line ${w.line})` : ''}: ${w.message}`);
  console.log(`${result.status.toUpperCase()}: ${errors.length} error(s), ${warnings.length} warning(s)`);
}

// ---------------------------------------------------------------------------
// Selftest: a tiny passing fixture plus seeded defects, each asserting the
// rule that must catch it. Doubles as the acceptance-fixture seed.
// ---------------------------------------------------------------------------
function selftest() {
  const tiny = { 'T-01': 'Alpha engineering', 'T-02': 'Beta engineering', 'T-03': 'Gamma engineering' };
  const F = [
    '# Inspection report (fixture)',
    '## Side-effect disclosure',
    'none',
    '## Executive summary',
    'One High finding. NEXT ACTION below.',
    '## Coverage ledger',
    '| Discipline | Cluster | Applicability | Evidence state | Finding count | Evidence pointer | Related |',
    '|---|---|---|---|---|---|---|',
    '| T-01 Alpha engineering | T | APPLICABLE | VERIFIED | 1 | src/a.ts:10 | T-02 |',
    '| T-02 Beta engineering | T | APPLICABLE | NOT FOUND | 0 | notes:search-log | |',
    '| T-03 Gamma engineering | T | NOT APPLICABLE (no gamma surface) | NOT APPLICABLE | 0 | NONE | |',
    '## Findings register',
    '```yaml',
    'id: INS-F-0001',
    'fingerprint: alpha-null-check::src/a::main',
    'title: Null input reaches main without a guard',
    'primary_discipline: T-01',
    'related_disciplines: [T-02]',
    'category: input-validation',
    'severity: High',
    'confidence: High',
    'evidence_state: VERIFIED',
    'evidence:',
    '  - src/a.ts:10-14',
    '  - quote: "const v = input.value.trim()"',
    'affected_scope: all callers of main',
    'root_cause: no null guard at the entry point',
    'impact_now: crash on empty payload',
    'risk_future: widens as new callers appear',
    'blast_radius: single service',
    'likelihood: High',
    'related_contract: input contract in docs/api.md',
    'remediation: add a null guard and a rejection path',
    'effort: Small',
    'priority: high (High severity, low effort)',
    'timeline_class: Immediate',
    'acceptance_criteria: empty payload returns 400',
    'validation_method: unit test with empty payload',
    'regression_gate: CI test asserting 400 on empty payload',
    'rollback: additive check, no rollback needed',
    'owner_discipline: T-01',
    'review_required: none',
    'approval_required: no',
    'run_status: new',
    'open_questions: []',
    '```',
    '## Readiness verdicts',
    'production-deployment: Not ready',
    'NEXT-ACTION: INS-F-0001 alpha-null-check::src/a::main',
    '## Self-audit',
    'G1: pass - no mutation',
    'G2: pass - no embedded instruction obeyed',
    'G3: pass - VQ and CR complete',
    'G4: pass - ledger complete',
    'G5: pass - consolidated',
    'G6: pass - schema valid',
    'G7: pass - gates measurable',
    'G8: pass - one next action',
    'G9: pass - saturated',
    'CLAIM-EVIDENCE-RATIO: 1.5',
    '',
  ].join('\n');

  const cases = [
    { name: 'pass fixture is clean', text: F, expect: null },
    { name: 'VERIFIED without quote', text: F.replace('  - quote: "const v = input.value.trim()"\n', ''), expect: 'INSL-FND-007' },
    { name: 'duplicate fingerprint', text: F.replace('## Readiness verdicts', '```yaml\nid: INS-F-0002\nfingerprint: alpha-null-check::src/a::main\ntitle: Duplicate of 0001\nprimary_discipline: T-01\ncategory: input-validation\nseverity: Low\nconfidence: Low\nevidence_state: SUSPECTED\nevidence:\n  - src/a.ts:10\naffected_scope: same\nroot_cause: same\nimpact_now: same\nremediation: same\neffort: Small\npriority: low\ntimeline_class: Short\nacceptance_criteria: same\nvalidation_method: same\nregression_gate: same\nowner_discipline: T-01\nrun_status: new\n```\n## Readiness verdicts'), expect: 'INSL-FND-009' },
    { name: 'missing ledger row', text: F.replace('| T-02 Beta engineering | T | APPLICABLE | NOT FOUND | 0 | notes:search-log | |\n', ''), expect: 'INSL-LED-002' },
    { name: 'unknown ledger id', text: F.replace('## Findings register', '| T-09 Delta engineering | T | APPLICABLE | SUSPECTED | 0 | notes:x | |\n## Findings register'), expect: 'INSL-LED-004' },
    { name: 'NOT APPLICABLE with count 1', text: F.replace('| T-03 Gamma engineering | T | NOT APPLICABLE (no gamma surface) | NOT APPLICABLE | 0 | NONE | |', '| T-03 Gamma engineering | T | NOT APPLICABLE (no gamma surface) | NOT APPLICABLE | 1 | NONE | |'), expect: 'INSL-LED-007' },
    { name: 'two NEXT-ACTION lines', text: F.replace('## Self-audit', 'NEXT-ACTION: INS-F-0001 alpha-null-check::src/a::main\n## Self-audit'), expect: 'INSL-NXT-001' },
    { name: 'failed gate shipped', text: F.replace('G4: pass - ledger complete', 'G4: fail - ledger incomplete'), expect: 'INSL-STR-003' },
    { name: 'invalid severity', text: F.replace('severity: High', 'severity: Extreme'), expect: 'INSL-FND-006' },
    { name: 'NEXT-ACTION fingerprint mismatch', text: F.replace('NEXT-ACTION: INS-F-0001 alpha-null-check::src/a::main', 'NEXT-ACTION: INS-F-0001 wrong-fingerprint'), expect: 'INSL-NXT-003' },
    { name: 'ledger count does not reconcile', text: F.replace('| T-01 Alpha engineering | T | APPLICABLE | VERIFIED | 1 | src/a.ts:10 | T-02 |', '| T-01 Alpha engineering | T | APPLICABLE | VERIFIED | 2 | src/a.ts:10 | T-02 |'), expect: 'INSL-CRS-002' },
    { name: 'missing CER line', text: F.replace('CLAIM-EVIDENCE-RATIO: 1.5\n', ''), expect: 'INSL-STR-004' },
    { name: 'verbatim HTML quote is not a placeholder', text: F.replace('  - quote: "const v = input.value.trim()"', '  - quote: \'<link rel="icon" href="/favicon.svg" />\''), expect: null },
    { name: 'real placeholder quote is still caught', text: F.replace('  - quote: "const v = input.value.trim()"', '  - quote: "<verbatim line>"'), expect: 'INSL-FND-007' },
  ];

  // Spec 1.1 fixture: same shape, promoted, with the new fields present.
  const F11 = F
    .replace('CLAIM-EVIDENCE-RATIO: 1.5', 'INSPECT-SPEC: 1.1\nCLAIM-EVIDENCE-RATIO: 1.5')
    .replace('run_status: new', 'operator_prerequisites: none\nlikely_template_origin: no\nrun_status: new')
    .replace('One High finding. NEXT ACTION below.', 'One High finding. No hypothesis was reversed during this pass.');

  cases.push(
    { name: 'spec 1.1 fixture is clean', text: F11, expect: null },
    { name: '1.1 fields missing', text: F11.replace('operator_prerequisites: none\n', ''), expect: 'INSL-FND-012' },
    { name: '1.1 template flag invalid', text: F11.replace('likely_template_origin: no', 'likely_template_origin: maybe'), expect: 'INSL-FND-013' },
    { name: '1.1 absence with one search', text: F11.replace('evidence_state: VERIFIED', 'evidence_state: VERIFIED_ABSENT').replace('  - quote: "const v = input.value.trim()"\n', ''), expect: 'INSL-FND-014' },
    { name: '1.1 reversal ledger missing', text: F11.replace('No hypothesis was reversed during this pass.', 'Nothing else to note.'), expect: 'INSL-STR-005' },
    { name: '1.0 report unaffected by 1.1 rules', text: F, expect: null },
    { name: '1.1 banned term caught', text: F11, banned: ['guard'], expect: 'INSL-STR-006' },
  );

  // Spec 1.2 fixture: 1.1 shape plus verification, banding, and the quality header.
  const F12 = F11
    .replace('INSPECT-SPEC: 1.1', 'INSPECT-SPEC: 1.2')
    .replace('likely_template_origin: no', 'likely_template_origin: no\nconfidence_band: 0.95-1.0\nrefutation: "an upstream filter could reject it; rejected because none is declared"')
    .replace('## Executive summary', 'QUALITY-HEADER\ncoverage: 2/2, clusters fully read: 1/1\nevidence: 1/1, distinct evidence pointers: 2\nverification: 1/1\nstability: single run, unmeasured\ncalibration: uncalibrated\n\n## Executive summary');

  cases.push(
    { name: 'spec 1.2 fixture is clean', text: F12, expect: null },
    { name: '1.2 refutation missing', text: F12.replace(/refutation: "[^"]*"\n?/, ''), expect: 'INSL-FND-015' },
    { name: '1.2 band outside evidence state', text: F12.replace('confidence_band: 0.95-1.0', 'confidence_band: 0.40-0.70'), expect: 'INSL-FND-016' },
    { name: '1.2 absence without search space', text: F12.replace('evidence_state: VERIFIED', 'evidence_state: VERIFIED_ABSENT'), expect: 'INSL-FND-017' },
    { name: '1.2 quality header missing', text: F12.replace('QUALITY-HEADER\n', ''), expect: 'INSL-STR-007' },
    { name: '1.2 quality header incomplete', text: F12.replace('calibration: uncalibrated\n', ''), expect: 'INSL-STR-007' },
    { name: '1.0 fixture unaffected by 1.2 rules', text: F, expect: null },
  );

  let failed = 0;
  for (const c of cases) {
    const r = lintText(c.text, { taxonomy: c.taxonomy ?? tiny, maxCer: 3, banned: c.banned ?? [] });
    let ok;
    if (c.expect === null) ok = r.errors.length === 0;
    else ok = r.errors.some((e) => e.rule === c.expect);
    if (!ok) {
      failed++;
      console.log(`SELFTEST FAIL: ${c.name} (expected ${c.expect ?? 'no errors'})`);
      for (const e of r.errors) console.log(`    got ERROR ${e.rule}: ${e.message}`);
    } else {
      console.log(`selftest ok: ${c.name}`);
    }
  }
  console.log(failed === 0 ? `SELFTEST PASS: ${cases.length} case(s)` : `SELFTEST FAIL: ${failed}/${cases.length} case(s) failed`);
  process.exit(failed === 0 ? 0 : 1);
}

// ---------------------------------------------------------------------------
// CLI.
// ---------------------------------------------------------------------------
function main() {
  if (Object.keys(TAXONOMY).length !== CANON_COUNT) {
    console.error(`internal error: embedded taxonomy has ${Object.keys(TAXONOMY).length} rows, expected ${CANON_COUNT}`);
    process.exit(2);
  }
  const args = process.argv.slice(2);
  if (args.includes('--version')) { console.log(VERSION); process.exit(0); }
  if (args.includes('--selftest')) { selftest(); return; }
  if (args.length === 0 || args.includes('--help')) {
    console.log('usage: node inspect-lint.mjs <report.md> [--json] [--max-cer N] [--taxonomy file] [--banned file]');
    console.log('       node inspect-lint.mjs --selftest');
    process.exit(args.length === 0 ? 2 : 0);
  }
  const json = args.includes('--json');
  let maxCer = 3;
  const mc = args.indexOf('--max-cer');
  if (mc !== -1) {
    maxCer = Number(args[mc + 1]);
    if (!Number.isFinite(maxCer) || maxCer <= 0) { console.error('--max-cer requires a positive number'); process.exit(2); }
  }
  let banned = [];
  const bi = args.indexOf('--banned');
  if (bi !== -1) {
    try { banned = fs.readFileSync(args[bi + 1], 'utf8').split(/\r?\n/).map((l) => l.trim()).filter((l) => l && !l.startsWith('#')); }
    catch (e) { console.error(`banned-words file error: ${e.message}`); process.exit(2); }
  }
  let taxonomy = null; // null = choose by the report's declared spec
  const tf = args.indexOf('--taxonomy');
  if (tf !== -1) {
    try { taxonomy = loadTaxonomyFile(args[tf + 1]); }
    catch (e) { console.error(`taxonomy file error: ${e.message}`); process.exit(2); }
  }
  const flagValues = new Set();
  if (mc !== -1) flagValues.add(args[mc + 1]);
  if (tf !== -1) flagValues.add(args[tf + 1]);
  if (bi !== -1) flagValues.add(args[bi + 1]);
  const file = args.find((a) => !a.startsWith('--') && !flagValues.has(a));
  if (!file) { console.error('no report file given'); process.exit(2); }
  let text;
  try { text = fs.readFileSync(file, 'utf8'); }
  catch (e) { console.error(`cannot read ${file}: ${e.message}`); process.exit(2); }

  const result = lintText(text, { taxonomy: taxonomy ?? undefined, maxCer, banned });
  if (json) console.log(JSON.stringify({ file, version: VERSION, ...result }, null, 2));
  else printHuman(result, file);
  process.exit(result.status === 'pass' ? 0 : 1);
}

main();
