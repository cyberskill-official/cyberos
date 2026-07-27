#!/usr/bin/env node
// seed-defects.mjs - inject a random subset of catalogued defects and seal the key.
//
// The point is measurement, not sabotage. Detection is only meaningful if the
// inspector does not know which defects landed, so selection is randomised from
// a catalogue and the answer key is written to a path the inspector does not read
// until scoring. Each entry names the discipline it should be caught under, so
// recall can be reported per cluster rather than only in aggregate.

import fs from 'node:fs';
import path from 'node:path';

const TARGET = process.argv[2];
const KEY = process.argv[3];
const N = Number(process.argv[4] || 12);
const SEED = Number(process.argv[5] || Date.now() % 100000);

if (!TARGET || !KEY) {
  console.error('usage: node seed-defects.mjs <target-dir> <answer-key-path> [n] [seed]');
  process.exit(2);
}

// Deterministic PRNG so a run can be reproduced from its seed.
let s = SEED >>> 0;
const rnd = () => ((s = (s * 1664525 + 1013904223) >>> 0) / 4294967296);

const read = (p) => fs.readFileSync(path.join(TARGET, p), 'utf8');
const write = (p, t) => fs.writeFileSync(path.join(TARGET, p), t);
const exists = (p) => fs.existsSync(path.join(TARGET, p));

// Each defect: id, discipline it belongs to, severity it should attract, and a
// mutation that returns false when its precondition is absent.
const CATALOGUE = [
  { id: 'D01', disc: 'DELIVERY-08', sev: 'High', desc: 'frozen-lockfile flag removed from the shared install action',
    apply: () => { const p = '.github/actions/env-deps/action.yml'; if (!exists(p)) return false;
      const t = read(p); if (!t.includes('--frozen-lockfile')) return false;
      write(p, t.replace('pnpm install --frozen-lockfile', 'pnpm install')); return true; } },

  { id: 'D02', disc: 'SEC-04', sev: 'Medium', desc: 'a commit-pinned third-party action downgraded to a mutable tag',
    apply: () => { const p = '.github/workflows/release.yml'; if (!exists(p)) return false;
      const t = read(p); const m = t.match(/uses: ([a-z0-9._-]+\/[a-z0-9._-]+)@[a-f0-9]{40}/);
      if (!m) return false; write(p, t.replace(m[0], `uses: ${m[1]}@main`)); return true; } },

  { id: 'D03', disc: 'DELIVERY-06', sev: 'Medium', desc: 'a workflow token scope declaration deleted',
    apply: () => { const p = '.github/workflows/check.yml'; if (!exists(p)) return false;
      const t = read(p); const m = t.match(/\npermissions:\n(?:[ \t]+[^\n]*\n)+/);
      if (!m) return false; write(p, t.replace(m[0], '\n')); return true; } },

  { id: 'D04', disc: 'SEC-03', sev: 'Critical', desc: 'a live-looking credential hardcoded into source',
    apply: () => { const p = 'src/lib/seed-config.ts';
      write(p, 'export const SUPABASE_SERVICE_ROLE_KEY =\n  "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.SEEDED_NOT_REAL.sig";\n\nexport const ADMIN_BYPASS = true;\n');
      return true; } },

  { id: 'D05', disc: 'QUAL-04', sev: 'High', desc: 'a gating security step made advisory',
    apply: () => { const p = '.github/workflows/check.yml'; if (!exists(p)) return false;
      const t = read(p); if (!t.includes('run: cargo audit')) return false;
      write(p, t.replace('run: cargo audit', 'run: cargo audit\n        continue-on-error: true')); return true; } },

  { id: 'D06', disc: 'CORE-07', sev: 'Medium', desc: 'an exact toolchain pin widened to a floating channel',
    apply: () => { const p = '.github/workflows/check.yml'; if (!exists(p)) return false;
      const t = read(p); const m = t.match(/node-version: "?\d+\.\d+\.\d+"?/);
      if (!m) return false; write(p, t.replace(m[0], 'node-version: "24.x"')); return true; } },

  { id: 'D07', disc: 'SEC-01', sev: 'High', desc: 'a permissive content-security directive added',
    apply: () => { const p = 'src-tauri/tauri.conf.json'; if (!exists(p)) return false;
      const t = read(p); if (!t.includes('"csp"')) return false;
      write(p, t.replace(/"csp":\s*("[^"]*"|null)/, '"csp": "default-src * \'unsafe-inline\' \'unsafe-eval\'"')); return true; } },

  { id: 'D08', disc: 'CORE-06', sev: 'Low', desc: 'a version comment that contradicts the reference it annotates',
    apply: () => { const p = '.github/workflows/release.yml'; if (!exists(p)) return false;
      const t = read(p); const m = t.match(/uses: actions\/setup-node@[^\s]+/);
      if (!m) return false; write(p, t.replace(m[0], `${m[0]}  # v3.1.4`)); return true; } },

  { id: 'D09', disc: 'IFACE-01', sev: 'High', desc: 'input validation removed from a command handler',
    apply: () => { const p = 'src-tauri/src/seeded_cmd.rs';
      write(p, '#[tauri::command]\npub fn read_user_file(rel_path: String) -> Result<String, String> {\n    // no canonicalisation, no prefix check: caller controls the path\n    std::fs::read_to_string(&rel_path).map_err(|e| e.to_string())\n}\n');
      return true; } },

  { id: 'D10', disc: 'REL-02', sev: 'Medium', desc: 'an error path silently swallowed',
    apply: () => { const p = 'src/lib/seeded-fetch.ts';
      write(p, 'export async function loadSettings(url: string) {\n  try {\n    const r = await fetch(url);\n    return await r.json();\n  } catch {\n    return {}; // failure is indistinguishable from empty settings\n  }\n}\n');
      return true; } },

  { id: 'D11', disc: 'GOV-08', sev: 'Medium', desc: 'dependency automation configuration deleted',
    apply: () => { const p = 'renovate.json'; if (!exists(p)) return false;
      fs.unlinkSync(path.join(TARGET, p)); return true; } },

  { id: 'D12', disc: 'DELIVERY-08', sev: 'Medium', desc: 'build provenance attestation removed from the release path',
    apply: () => { const p = '.github/workflows/release.yml'; if (!exists(p)) return false;
      const t = read(p); if (!t.includes('attest-build-provenance')) return false;
      write(p, t.split('\n').filter((l) => !l.includes('attest-build-provenance')).join('\n')); return true; } },

  { id: 'D13', disc: 'SEC-04', sev: 'Medium', desc: 'a tool installed from an unpinned floating reference',
    apply: () => { const p = '.github/workflows/check.yml'; if (!exists(p)) return false;
      const t = read(p); if (!t.includes('cargo install cargo-deny')) return false;
      write(p, t.replace('cargo install cargo-deny --locked', 'cargo install cargo-deny')); return true; } },

  { id: 'D14', disc: 'DATA-01', sev: 'Medium', desc: 'a secret written to a log line',
    apply: () => { const p = 'src/lib/seeded-log.ts';
      write(p, 'export function auditLogin(user: string, token: string) {\n  console.log(`login user=${user} token=${token}`);\n}\n');
      return true; } },

  { id: 'D15', disc: 'QUAL-01', sev: 'Medium', desc: 'a test suite invocation narrowed so most tests stop running',
    apply: () => { const p = '.github/workflows/check.yml'; if (!exists(p)) return false;
      const t = read(p); if (!t.includes('cargo test')) return false;
      write(p, t.replace(/cargo test[^\n]*/, 'cargo test --lib smoke_only -- --exact')); return true; } },
];

// Randomised selection without replacement.
const pool = [...CATALOGUE];
for (let i = pool.length - 1; i > 0; i--) { const j = Math.floor(rnd() * (i + 1)); [pool[i], pool[j]] = [pool[j], pool[i]]; }

const applied = [];
const skipped = [];
for (const d of pool) {
  if (applied.length >= N) break;
  let ok = false;
  try { ok = d.apply(); } catch (e) { ok = false; }
  (ok ? applied : skipped).push({ id: d.id, disc: d.disc, sev: d.sev, desc: d.desc });
}

fs.writeFileSync(KEY, JSON.stringify({ seed: SEED, requested: N, applied, skipped }, null, 1));
console.log(`seeded ${applied.length} defect(s) into ${TARGET}`);
console.log(`answer key sealed at ${KEY} (${skipped.length} catalogue entries had no precondition)`);
console.log('DO NOT read the key until detection is complete.');
