// measure-phase.mjs - emit the REAL corpus numbers for TASK-IMP-121/122/123.
// Written because three specs shipped four false counts derived from greps of a SUBSET
// reported as corpus facts. Output is the evidence; specs are written from it, not from recall.
import { readdirSync, statSync, readFileSync, existsSync } from "node:fs";
import { join } from "node:path";
const root = process.argv[2];
const specs = [];
const walk = d => { for (const e of readdirSync(d)) { const p = join(d, e);
  if (statSync(p).isDirectory()) walk(p); else if (e === "spec.md") specs.push(p); } };
walk(join(root, "docs/tasks"));

const fm = s => { const m = /^---\n([\s\S]*?)\n---/.exec(s); return m ? m[1] : null; };
const one = (f, k) => { const m = new RegExp(`^${k}:[ \\t]*(.*)$`, "m").exec(f);
  if (!m) return undefined; return m[1].trim().replace(/^["']|["']$/g, "").replace(/\s+#.*$/, ""); };

const rows = [];
for (const p of specs) { const f = fm(readFileSync(p, "utf8")); if (!f) continue;
  rows.push({ p, id: one(f,"id"), status: one(f,"status"), phase: one(f,"phase"),
    module: p.replace(root+"/","").split("/")[2] }); }

const withPhase = rows.filter(r => r.phase !== undefined && r.phase !== "");
const noPhase   = rows.filter(r => r.phase === undefined || r.phase === "");
const done      = rows.filter(r => r.status === "done");
const doneWith  = done.filter(r => r.phase !== undefined && r.phase !== "");

// vocabulary classifier - declared explicitly so the buckets are auditable, not vibes
const vocab = v => /^P[0-9]$/.test(v) ? "P-number"
  : /^pre-1\.0\.0|^post-1\.0\.0/.test(v) ? "release-gate"
  : /^Wave [0-9]/.test(v) ? "Wave-numeric"
  : /^Wave [A-Z]/.test(v) ? "Wave-lettered"
  : /^Phase [0-9]/.test(v) ? "Phase-prose" : "OTHER";

const byVocab = {}, byValue = {};
for (const r of withPhase) { const k = vocab(r.phase);
  (byVocab[k] ??= new Set()).add(r.phase); byValue[r.phase] = (byValue[r.phase]||0)+1; }

console.log("=== TOTALS ===");
console.log("specs_total             =", rows.length);
console.log("specs_with_phase        =", withPhase.length);
console.log("specs_missing_phase     =", noPhase.length);
console.log("done_total              =", done.length);
console.log("done_carrying_phase     =", doneWith.length);
console.log("done_missing_phase      =", done.length - doneWith.length);
console.log("distinct_phase_values   =", Object.keys(byValue).length);
console.log("distinct_vocabularies   =", Object.keys(byVocab).length);
console.log("\n=== VOCABULARIES ===");
for (const [k, set] of Object.entries(byVocab)) {
  const n = [...set].reduce((a,v)=>a+byValue[v],0);
  console.log(`${k.padEnd(14)} values=${String(set.size).padStart(2)}  specs=${String(n).padStart(3)}  ${[...set].sort().slice(0,4).join(" | ")}${set.size>4?" | ...":""}`);
}
console.log("\n=== SPECS MISSING phase (the full list) ===");
for (const r of noPhase.sort((a,b)=>String(a.id).localeCompare(String(b.id))))
  console.log(`  ${String(r.id).padEnd(18)} ${r.status.padEnd(20)} ${r.p.replace(root+"/","")}`);
console.log("\n=== improvement module ONLY (the subset I mistook for the corpus) ===");
const imp = rows.filter(r => r.module === "improvement");
const impV = {}; for (const r of imp.filter(r=>r.phase)) impV[vocab(r.phase)] = (impV[vocab(r.phase)]||0)+1;
console.log("  improvement specs =", imp.length, " with_phase =", imp.filter(r=>r.phase).length);
console.log("  its vocabularies  =", JSON.stringify(impV));
