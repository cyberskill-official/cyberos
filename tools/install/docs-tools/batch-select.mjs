#!/usr/bin/env node
// batch-select.mjs - a greedy cone-independent batch, computed (TASK-IMP-104 evidence, v2.8.0).
//
// GREEDY, not maximal. Members are admitted in Queue-selection priority order and any task that
// clashes with an already-admitted member is excluded and never reconsidered. A lower-priority
// task dropped by an early admission could sometimes have permitted a strictly larger independent
// set, so this is not a maximum independent set and does not try to be - that problem is NP-hard,
// and a batch that is larger but ignores priority is the wrong batch anyway.
//
// The header said "maximal" until 2026-07-17. It overstated what the loop computes. The value
// here is that a batch is COMPUTED at all rather than chosen by mood; overstating the guarantee
// invited a consumer to depend on maximality that was never delivered. (Devin review.)
//
// ship-tasks.md has called BATCH the default since v2.5.0 while its outer loop asked for
// next_eligible() - one task. Nothing computed a batch, so nothing could notice when one was
// skipped: on 2026-07-17 the workflow shipped TASK-IMP-104 alone while TASK-IMP-106 sat eligible
// and cone-independent beside it. A default no step computes is a comment. This is the step.
//
// Deterministic by construction: reads frontmatter, prints a batch. No model, no judgment - the
// machine floor decides membership (TASK-IMP-084), the human decides the verdicts.
//
// usage: node batch-select.mjs [--repo <root>] [--json]
// exits: 0 selection computed (even when the batch is 1)   2 usage   3 corpus unreadable
import { readFileSync, readdirSync, existsSync, statSync } from "node:fs";
import { join, resolve } from "node:path";

const argv = process.argv.slice(2);
if (argv.includes("--help")) { console.log("usage: node batch-select.mjs [--repo <root>] [--json]"); process.exit(2); }
const asJson = argv.includes("--json");
// indexOf returns -1 when a flag is absent, and -1 + 1 = 0 - which reads argv[0] as the value.
// `--repo` absent therefore made `--json` the repo root, so the DOCUMENTED invocation
// (ship-tasks §11a: `node .cyberos/docs-tools/batch-select.mjs --json`) always exited 3 with
// "no docs/tasks under <cwd>/--json". The mandatory batch step could never run as documented.
// The sibling verify-goals.mjs already fixed this exact defect and it was not back-ported;
// batch-select had no suite, so nothing caught it. Read flags by presence, never by offset.
const flag = (name, dflt) => { const i = argv.indexOf(name); return (i >= 0 && argv[i + 1] !== undefined && !argv[i + 1].startsWith("--")) ? argv[i + 1] : dflt; };
const root = resolve(flag("--repo", "."));
const tasksDir = join(root, "docs", "tasks");
if (!existsSync(tasksDir)) { console.error(`batch-select: no docs/tasks under ${root}`); process.exit(3); }

const fm = (t) => {
  if (!t.startsWith("---")) return null;
  const e = t.indexOf("\n---", 3); if (e < 0) return null;
  return t.slice(4, e);
};
const one = (f, k) => (f.match(new RegExp(`^${k}:\\s*(.*)$`, "m"))?.[1] ?? "").trim();
const list = (f, k) => {
  const inline = one(f, k);
  if (inline.startsWith("[")) return inline.slice(1, -1).split(",").map(s => s.trim()).filter(Boolean);
  const blk = f.match(new RegExp(`^${k}:\\s*\\n((?:  - .*\\n?)+)`, "m"));
  return blk ? blk[1].split("\n").map(l => l.replace(/^\s*-\s*/, "").trim()).filter(x => x && x !== "(none)") : [];
};

const tasks = [];
const walk = (d) => { for (const e of readdirSync(d)) {
  const p = join(d, e);
  if (!statSync(p).isDirectory()) continue;
  const sp = join(p, "spec.md");
  if (existsSync(sp)) { const f = fm(readFileSync(sp, "utf8")); if (f) tasks.push({ dir: p, f }); }
  else walk(p);
} };
walk(tasksDir);

const byId = new Map();
for (const { f } of tasks) {
  const id = one(f, "id"); if (!id) continue;
  byId.set(id, {
    id, status: one(f, "status"), priority: one(f, "priority") || "p9",
    service: one(f, "service"), deps: list(f, "depends_on"),
    // The cone is files UNION service. `service` is not decoration: a declared file list is what
    // the AUTHOR expected to touch. TASK-IMP-104 declared install.sh and edited version.sh +
    // lib/update-check.sh - both inside its service, neither declared. Files-only would have let
    // a sibling tools/install task race it on files nobody wrote down.
    cone: new Set([...list(f, "new_files"), ...list(f, "modified_files"), ...(one(f, "service") ? [one(f, "service")] : [])]),
  });
}
const done = (id) => byId.get(id)?.status === "done";
const eligible = [...byId.values()]
  .filter(t => t.status === "ready_to_implement" && t.deps.every(done))
  .sort((a, b) => (a.priority.localeCompare(b.priority)) || a.id.localeCompare(b.id));

const clash = (a, b) => {
  for (const x of a.cone) for (const y of b.cone) {
    if (x === y) return x;                                   // same file, or same service
    if (x.startsWith(y + "/") || y.startsWith(x + "/")) return `${x} ⊂ ${y}`;  // file inside a service
  }
  if (a.deps.includes(b.id) || b.deps.includes(a.id)) return "depends_on edge";
  return null;
};

const batch = [], excluded = [];
for (const t of eligible) {
  const hit = batch.map(m => [m, clash(m, t)]).find(([, c]) => c);
  if (hit) excluded.push({ id: t.id, blocked_by: hit[0].id, conflict: hit[1] });
  else batch.push(t);
}
const out = {
  // No `generated` date. The file header claims "deterministic by construction"; a wall-clock
  // field made that false - the same corpus produced a different artefact tomorrow, so any
  // consumer diffing batch-selection@1 outputs for equality would see a spurious change.
  // The selection is a function of the corpus and nothing else. If a consumer needs "when",
  // the commit that carries the artefact has it, and that timestamp is not invented here.
  // (External review 2026-07-17. The claim had no test, which is why it drifted from the code
  // without anything noticing - the exact gap TASK-IMP-118 exists to close. t11 now pins it.)
  artefact: "batch-selection@1",
  eligible: eligible.map(t => t.id), batch: batch.map(t => t.id),
  swarm_required: batch.length > 1, excluded,
};
if (asJson) { console.log(JSON.stringify(out, null, 2)); process.exit(0); }
console.log(`batch-selection@1  (eligible ${eligible.length}, batch ${batch.length}, swarm_required=${out.swarm_required})`);
for (const t of batch) console.log(`  BATCH    ${t.id}  [${t.priority}]  service=${t.service}`);
for (const e of excluded) console.log(`  excluded ${e.id}  <- conflicts with ${e.blocked_by} on: ${e.conflict}`);
process.exit(0);
