#!/usr/bin/env node
// verify-goals.mjs - re-verify what `done` claimed, forever (TASK-IMP-109).
//
// WHY: `done` is terminal and nothing re-checks it. TRACE-004 proves every clause had a passing
// test ON THE DAY IT SHIPPED, and then nobody looks again. A task shipped in batch 1 could be
// broken today and the corpus would still show it green. A goal you verify once is an assumption
// with a timestamp. task-reconcile does NOT close this: it measures drift when a task RE-ENTERS
// the workflow - a turnstile, not a sentinel. A done task that never comes back is never seen again.
//
// SECURITY-CLASS: HIGH, and it is the whole task. This tool EXECUTES COMMANDS READ FROM FILES.
// That is precisely the rung-5 defect the 2026-07-17 review caught in task-reconcile: a crafted
// spec could name a command. The guard here is not an afterthought, it is the spine:
//   CONFINE (relUnderRoot) -> EXISTS -> TRACKED AT HEAD (git ls-tree) -> only then execute.
// Every refusal is a VIOLATION with its reason named, never a silent skip and never a run.
// A predicate is a repo-tracked suite file, never a constructed shell string.
//
// DETECTION ONLY (spec §1.7). A violated goal changes no status, writes no code, re-opens no
// task. The sentinel detects; the pipeline fixes. An auto-fix on a violated acceptance is the
// machine grading its own homework at the exact moment nobody is watching.
//
// usage: node verify-goals.mjs [--repo <root>] [--json] [--timeout <secs>]
// exits: 0 all goals checked and passing · 1 one or more VIOLATED · 2 usage · 3 goals present
//        that CANNOT be checked (no re-runnable predicate). A missing or empty docs/goals is
//        NOT 3 - there is nothing to verify, so it is 0 (see the early return below).
//        This line read '3 corpus unreadable' until 2026-07-17: a code documented and never
//        implemented, which then collided with the real meaning 3 acquired. The authoritative
//        contract is at the foot of this file; this header must agree with it. (Devin review.)
import { readFileSync, writeFileSync, readdirSync, existsSync, appendFileSync, mkdirSync } from "node:fs";
import { join, resolve, relative, isAbsolute, dirname } from "node:path";
import { spawnSync } from "node:child_process";

const argv = process.argv.slice(2);
if (argv.includes("--help")) { console.log("usage: node verify-goals.mjs [--repo <root>] [--json] [--timeout <secs>]\n  exit: 0 all checked and passing · 1 a goal violated · 2 usage · 3 goals that cannot be checked"); process.exit(2); }
const asJson = argv.includes("--json");
// indexOf returns -1 when a flag is absent, and -1 + 1 = 0 - which reads the FIRST argv element
// as the value. `--timeout` absent therefore yielded Number("--repo") = NaN and spawnSync threw
// on the DEFAULT invocation. Read flags by presence, never by blind offset. (Caught by t03.)
// Guard the NEXT token too, matching batch-select.mjs. Without it `--repo --json` swallows the
// flag as a path. These two are siblings and have drifted apart once already: the argv-offset
// bug was found and fixed HERE and never back-ported THERE. Keep them identical. (Review 2026-07-17.)
const flag = (name, dflt) => { const i = argv.indexOf(name); return (i >= 0 && argv[i + 1] !== undefined && !argv[i + 1].startsWith("--")) ? argv[i + 1] : dflt; };
const root = resolve(flag("--repo", "."));
const TIMEOUT_S = Number(flag("--timeout", "60"));
if (!Number.isFinite(TIMEOUT_S) || TIMEOUT_S <= 0) { console.error(`verify-goals: --timeout must be a positive number of seconds (got '${flag("--timeout", "60")}')`); process.exit(2); }
const TIMEOUT = TIMEOUT_S * 1000;

// Same predicate as task-reconcile's and coverage-scope's. One rule, three tools.
const relUnderRoot = (root, p) => {
  const rel = relative(root, resolve(root, p));
  return (rel === "" || rel.startsWith("..") || isAbsolute(rel)) ? null : rel;
};
const sh = (cmd, args, cwd, timeout) => spawnSync(cmd, args, { cwd, encoding: "utf8", timeout });

const goalsDir = join(root, "docs", "goals");
if (!existsSync(goalsDir)) {
  const msg = "no docs/goals yet - nothing has graduated. Goals are written at the done flip.";
  if (asJson) console.log(JSON.stringify({ artefact: "goal-ledger@1", goals: [], violations: 0, note: msg }));
  else console.log(`verify-goals: ${msg}`);
  process.exit(0);
}

const fm = (t) => { if (!t.startsWith("---")) return null; const e = t.indexOf("\n---", 3); return e < 0 ? null : t.slice(4, e); };
const one = (f, k) => (f.match(new RegExp(`^${k}:\\s*(.*)$`, "m"))?.[1] ?? "").trim();
const preds = (f) => {
  const b = f.match(/^predicates:\s*\n((?:  - .*\n?)+)/m);
  return b ? b[1].split("\n").map(l => l.replace(/^\s*-\s*/, "").trim()).filter(Boolean) : [];
};

const results = [], ledgerRows = [];
const retiredTasks = new Set();   // have a goal, quarantined - NOT the same as having none
let violations = 0;
let unverifiable = 0;   // goals that CANNOT be checked - distinct from goals that FAILED

for (const e of readdirSync(goalsDir)) {
  if (!e.endsWith(".md") || e.startsWith(".")) continue;
  const p = join(goalsDir, e);
  const f = fm(readFileSync(p, "utf8"));
  if (!f) { console.error(`verify-goals: WARN unparseable goal ${e} - skipped, not counted`); continue; }
  const status = one(f, "status");
  if (status === "retired") { retiredTasks.add(one(f, "source") || e.replace(/\.md$/, "")); continue; }   // quarantined; never deleted (§3)
  const id = one(f, "source") || e.replace(/\.md$/, "");
  const list = preds(f);

  // §1.4: a task with no mechanically re-runnable predicate STILL gets a goal, marked
  // `predicate: none`. The absence IS the finding - it must not read as a pass.
  if (!list.length) {
    results.push({ goal: e, task: id, verdict: "no_predicate", notes: [one(f, "predicate_none_reason") || "no mechanically re-runnable predicate (verify:-only ACs are not predicates - §1.3)"] });
    unverifiable++;   // §1.4 - see the exit contract at the foot of this file
    ledgerRows.push([new Date().toISOString(), id, "NO_PREDICATE", "0"].join("\t"));
    continue;
  }

  const notes = []; let failed = false;
  for (const raw of list) {
    // ---- THE GUARD. Order matters and every step refuses loudly. ----
    const rel = relUnderRoot(root, raw);
    if (rel === null) { failed = true; notes.push(`predicate escapes the repo root: ${raw} - REFUSED, not executed`); continue; }
    if (!existsSync(join(root, rel))) { failed = true; notes.push(`predicate resolves nowhere: ${rel} - the acceptance cites a test that no longer exists, which IS the finding`); continue; }
    const tracked = sh("git", ["ls-tree", "HEAD", "--", rel], root, 10000);
    if (!(tracked.status === 0 && tracked.stdout.trim() !== "")) {
      failed = true; notes.push(`predicate is not tracked at HEAD: ${rel} - REFUSED, not executed (an untracked file on disk cannot be a repo's acceptance)`); continue;
    }
    // Tracked, confined, present. Run it WITHOUT a shell: argv, never a string.
    const started = Date.now();
    const r = sh("bash", [rel], root, TIMEOUT);
    const ms = Date.now() - started;
    // §1.8: a timeout is a VIOLATION named as one. An unrunnable predicate is not a passing one.
    if (r.error && r.error.code === "ETIMEDOUT") { failed = true; notes.push(`predicate TIMED OUT after ${TIMEOUT / 1000}s: ${rel} - a predicate that cannot finish is not a predicate that passes (§1.8; cheapen it)`); continue; }
    if (r.status !== 0) { failed = true; notes.push(`predicate FAILED (exit ${r.status}) in ${ms}ms: ${rel}`); continue; }
    notes.push(`ok ${rel} (${ms}ms)`);
  }

  const verdict = failed ? "VIOLATED" : "satisfied";
  if (failed) violations++;
  results.push({ goal: e, task: id, verdict, notes });
  ledgerRows.push([new Date().toISOString(), id, failed ? "VIOLATED" : "PASS", String(list.length)].join("\t"));

  // §1.5: flip the goal file's own status + refresh last_pass on success. This writes ONLY the
  // goal file - never a task, never a status cell, never code (§1.7).
  let t = readFileSync(p, "utf8");
  t = t.replace(/^status:.*$/m, `status: ${failed ? "violated" : "satisfied"}`);
  if (!failed) t = t.replace(/^last_pass:.*$/m, `last_pass: ${new Date().toISOString().slice(0, 10)}`);
  writeFileSync(p, t);
}

// the ledger: append-only, one row per goal per run
const ledger = join(goalsDir, ".ledger.tsv");
if (ledgerRows.length) { mkdirSync(dirname(ledger), { recursive: true }); appendFileSync(ledger, ledgerRows.join("\n") + "\n"); }

// §3 (operator, HITL gate 1, 2026-07-17): the report MUST state how many `done` tasks have NO
// goal. Without it the runner says "0 violated" over a corpus it barely covers, and a green
// summary that hides its own denominator is the TASK-IMP-086 class - a claim nobody can check.
// Enrolment starts at the done flip and is NOT backfilled (explicit Non-Goal), so this number is
// large by design and shrinks only as tasks ship. It is a coverage statement, not a failure.
const coverage = (() => {
  // A task whose goal is quarantined HAS a goal - it is just not actively verifying. Counting it
  // under "have NO goal" overstates the unverified set and makes the report inaccurate about the
  // exact case §3 asks us to handle gently. (External review, 2026-07-17.)
  const enrolled = new Set([...results.map(r => r.task), ...retiredTasks]);
  const doneTasks = [];
  const walk = (d) => { if (!existsSync(d)) return;
    for (const e of readdirSync(d, { withFileTypes: true })) {
      if (!e.isDirectory()) continue;
      const sp = join(d, e.name, "spec.md");
      if (existsSync(sp)) {
        const f = fm(readFileSync(sp, "utf8"));
        if (f && one(f, "status") === "done") doneTasks.push(one(f, "id") || e.name);
      } else walk(join(d, e.name));
    } };
  walk(join(root, "docs", "tasks"));
  const missing = doneTasks.filter(t => !enrolled.has(t));
  return { done_tasks: doneTasks.length, enrolled: doneTasks.length - missing.length, without_goal: missing.length, retired: [...retiredTasks].filter(t => doneTasks.includes(t)).length };
})();

// No `generated` date: the artefact is a function of the corpus, and a wall-clock field made one
// corpus emit a different artefact every day. Same defect external review found in batch-select;
// fixed here in the same pass rather than left for the next reviewer to find twice. The LEDGER
// timestamps stay - a log of runs is exactly where a clock belongs.
const out = { artefact: "goal-ledger@1", coverage, goals: results, violations, unverifiable };
if (asJson) console.log(JSON.stringify(out, null, 2));
else {
  console.log(`verify-goals: ${results.length} goal(s), ${violations} violated, ${unverifiable} unverifiable`);
  console.log(`  coverage: ${coverage.enrolled}/${coverage.done_tasks} done tasks enrolled - ${coverage.without_goal} have NO goal and are unverified since the day they shipped${coverage.retired ? `; ${coverage.retired} have a QUARANTINED goal (has one, not verifying)` : ""}.`);
  for (const r of results) {
    console.log(`  ${r.verdict.padEnd(12)} ${r.task}`);
    for (const n of r.notes) console.log(`      ${n}`);
  }
  if (violations) console.log("\n  A violated goal is a FINDING, not a fix. Remedy: a type: bug task through create-tasks -> ship-tasks.");
  if (unverifiable && !violations) console.log(`\n  ${unverifiable} goal(s) CANNOT be checked (exit 3). Nothing is broken; nothing is proven either.`);
}

// Exit contract (§1.4 + §1.6):
//   0  every goal was checked and passed
//   1  at least one goal VIOLATED - something that passed now fails (§1.6)
//   2  usage error
//   3  no violations, but goals exist that cannot be checked at all (§1.4)
//
// 3 exists because §1.4 says the absent predicate IS the finding and "must not read as a pass" -
// and this branch used to `continue` without touching any counter, so the tool printed the
// finding and exited 0. CI and operators read the exit code, not the prose: a green tick for a
// task nobody ever checked. I wrote the rule and broke it in the same block. (Greptile P1,
// 2026-07-17.)
//
// Not folded into 1, deliberately: "cannot be checked" and "was passing, now broken" are
// different facts. §1.6's word is VIOLATED, and calling an honest gap a violation is a different
// lie - it would also paint CI red for a state §1.4 explicitly blesses, which teaches people to
// ignore red.
process.exit(violations ? 1 : unverifiable ? 3 : 0);
