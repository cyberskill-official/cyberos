#!/usr/bin/env node
// task-reconcile.mjs — evidence ladder for drifted task states (TASK-IMP-100).
//
// ship-tasks trusts two things: its own run manifests (hash-verified resume) and its own
// gates (route-back on failure). A task that arrives ALREADY IMPLEMENTED from outside the
// loop — status past ready_to_implement with no manifest, or done with missing gate
// artefacts — is invisible to both. This tool measures that third state.
//
// It is READ-ONLY by contract (spec §1 #1.1): rungs 1-4 never execute repo code, rung 5
// runs only under --run-tests and only suite FILES the spec itself cites. Nothing outside
// --out is ever written. It emits reconcile-report@1 with exactly one recommendation and
// NEVER acts on it — the verdict is HITL's (see modules/skill/task-reconcile/SKILL.md).
//
// Rungs
//   R1 spec integrity      task-lint verdict; audit.md present, overall_status pass, and the
//                          audit's byte-binding still describing the spec's NORMATIVE half.
//                          (Dogfood finding, 2026-07-17: audits bind a whole-FILE sha, but
//                          ship-tasks mutates status/shipped IN THAT FILE on every phase —
//                          so a naive re-hash reds every correctly-shipped task. The binding
//                          is therefore checked against the spec AS OF THE AUDIT COMMIT, and
//                          drift is judged on the normative half only: the body plus the
//                          frontmatter minus the lifecycle-mutable fields. Recorded as a
//                          follow-up: audits should bind the normative half directly.)
//   R2 artefact set        the phase set implied by the claimed status, accepted in EITHER
//                          home: <task folder>/ or docs/tasks/.workflow/<task-id>/ (the
//                          historical corpus bundles artefacts there — a bundle file whose
//                          name or body names a phase artefact satisfies that artefact).
//   R3 manifest            ship-manifest.mjs verify; ABSENT is a finding, not a failure
//                          (out-of-band work legitimately has none).
//   R4 committed object    every frontmatter new_files/modified_files path present at HEAD
//                          (git ls-tree). Present on disk but missing at HEAD = RED: the
//                          TASK-IMP-086 class (a claim no commit carries).
//   R5 cited tests         (--run-tests) each distinct suite file behind a §2 `test:` entry
//                          must exist and exit 0. A citation resolving nowhere is TRACE-003
//                          drift at run time.
//
// Recommendation (exactly one, spec §1 #1.3)
//   resume_at_phase(N)  every load-bearing rung supports the claimed status
//   route_back          any load-bearing red (R1 drift, R4 uncommitted claim, R5 fail/miss)
//   adopt_candidate     deliverables green at HEAD but the phase artefact set is missing
//   not_applicable      draft / ready_to_implement — nothing claimed, nothing to reconcile
//
// Exits: 0 evaluation completed (ANY recommendation — the verdict is not the tool's),
//        2 usage, 3 task not resolvable. Never non-zero to signal "bad task".
import { readFileSync, writeFileSync, existsSync, readdirSync, statSync, mkdirSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve, dirname, basename, relative, isAbsolute } from "node:path";
import { createHash } from "node:crypto";

const PHASE_OF = {            // claimed status -> ship-tasks skill_chain step (workflow §skill_chain)
  implementing: 1, ready_to_review: 13, reviewing: 17,
  ready_to_test: 21, testing: 23, done: null,   // done -> "confirm done", no resume step
};
const ARTEFACTS_FOR = {       // cumulative phase artefact sets (corpus convention)
  implementing:    ["context-map.md", "edge-case-matrix.md", "impl-plan.md", "obs-injection.md"],
  ready_to_review: ["context-map.md", "edge-case-matrix.md", "impl-plan.md", "obs-injection.md"],
  reviewing:       ["context-map.md", "edge-case-matrix.md", "impl-plan.md", "obs-injection.md", "code-review.md"],
  ready_to_test:   ["context-map.md", "edge-case-matrix.md", "impl-plan.md", "obs-injection.md", "code-review.md"],
  testing:         ["context-map.md", "edge-case-matrix.md", "impl-plan.md", "obs-injection.md", "code-review.md", "coverage-gate.md"],
  done:            ["context-map.md", "edge-case-matrix.md", "impl-plan.md", "obs-injection.md", "code-review.md", "coverage-gate.md"],
};
const NOT_APPLICABLE = new Set(["draft", "ready_to_implement"]);

class Usage extends Error { constructor(m) { super(m); this.code = 2; } }

// PR-review fix (Devin, 2026-07-17): --out was resolved against the root but never CONFINED
// to it - an absolute path or a ../ value wrote (and mkdir'd) anywhere the process could
// reach. The sibling tool coverage-scope.mjs has always refused out-of-root paths; a
// read-only instrument must be the stricter of the two, not the looser. Same predicate,
// same refusal.
const relUnderRoot = (root, p) => {
  const rel = relative(root, resolve(root, p));
  return (rel === "" || rel.startsWith("..") || isAbsolute(rel)) ? null : rel;
};
class Unresolved extends Error { constructor(m) { super(m); this.code = 3; } }

const sh = (cmd, args, cwd) => spawnSync(cmd, args, { cwd, encoding: "utf8" });

// minimal frontmatter reader — the template's subset (scalars + simple lists), same
// discipline as task-lint: fail loudly rather than guess at exotic YAML.
function frontmatter(text, where) {
  const lines = text.split("\n");
  if (lines[0].trim() !== "---") throw new Unresolved(`${where}: no frontmatter fence`);
  const end = lines.findIndex((l, i) => i > 0 && l.trim() === "---");
  if (end < 0) throw new Unresolved(`${where}: unterminated frontmatter fence`);
  const fm = {}; let key = null;
  for (const raw of lines.slice(1, end)) {
    const li = raw.match(/^\s*-\s+(.*)$/);
    if (li && key) { (fm[key] = Array.isArray(fm[key]) ? fm[key] : []).push(unq(li[1])); continue; }
    const m = raw.match(/^([A-Za-z0-9_]+):\s*(.*)$/);
    if (!m) continue;
    key = m[1];
    const v = m[2].trim();
    if (v === "") { fm[key] = []; continue; }
    if (v.startsWith("[") && v.endsWith("]")) {
      fm[key] = v.slice(1, -1).split(",").map(s => unq(s.trim())).filter(Boolean);
    } else fm[key] = unq(v);
  }
  return fm;
}
const unq = s => s.replace(/^["']|["']$/g, "").trim();
const sha = b => createHash("sha256").update(b).digest("hex");

function findTask(root, id) {
  const base = join(root, "docs", "tasks");
  if (!existsSync(base)) throw new Unresolved(`no docs/tasks under ${root}`);
  for (const mod of readdirSync(base)) {
    const md = join(base, mod);
    if (!statSync(md).isDirectory()) continue;
    for (const d of readdirSync(md)) {
      if (d === id || d.startsWith(id + "-")) {
        const spec = join(md, d, "spec.md");
        if (existsSync(spec)) return { dir: join(md, d), spec, module: mod };
      }
    }
  }
  throw new Unresolved(`task ${id} not found under docs/tasks/*/`);
}

// ── R1: spec integrity ───────────────────────────────────────────────────────
const LIFECYCLE_FIELDS = ["status", "shipped", "routed_back_count", "memory_chain_hash"];

// The normative half: body + frontmatter minus the fields the workflow itself flips.
function normativeHalf(text) {
  const lines = text.split("\n");
  if (lines[0]?.trim() !== "---") return text;
  const end = lines.findIndex((l, i) => i > 0 && l.trim() === "---");
  if (end < 0) return text;
  const fmKept = lines.slice(1, end).filter(l => {
    const m = l.match(/^([A-Za-z0-9_]+):/);
    return !(m && LIFECYCLE_FIELDS.includes(m[1]));
  });
  return [...fmKept, ...lines.slice(end + 1)].join("\n");
}

function rung1(root, t) {
  const notes = [];
  const lintTool = ["tools/install/docs-tools/task-lint.mjs", ".cyberos/docs-tools/task-lint.mjs"]
    .map(p => join(root, p)).find(existsSync);
  if (!lintTool) notes.push("task-lint not found in this repo - lint rung skipped");
  else {
    const lint = sh("node", [lintTool, t.spec], root);
    if (lint.status !== 0) notes.push(`task-lint FAILED: ${(lint.stderr || lint.stdout || "").trim().split("\n")[0]}`);
    else notes.push("task-lint clean");
  }
  const auditPath = join(t.dir, "audit.md");
  if (!existsSync(auditPath)) return { verdict: "red", notes: [...notes, "audit.md absent - the spec was never audited"] };
  const afm = frontmatter(readFileSync(auditPath, "utf8"), auditPath);
  if (String(afm.overall_status) !== "pass") notes.push(`audit overall_status is '${afm.overall_status}', not pass`);

  // Byte-binding. The recorded prefix is a whole-FILE sha taken at audit time, and the
  // workflow legitimately rewrites lifecycle fields afterwards - so verify the binding
  // against the spec AS OF THE AUDIT COMMIT, then judge drift on the normative half.
  // TASK-IMP-102: the BODY binding is the verifiable one - it covers exactly what the audit
  // judged and nothing ship-tasks rewrites afterwards. Prefer it; fall back to the legacy
  // whole-file field via the audit commit (and say so) for audits written before the rule.
  const wantBody = String(afm.audited_body_sha256_prefix || "");
  if (wantBody) {
    const gotBody = sha(Buffer.from(normativeHalf(readFileSync(t.spec, "utf8"))));
    if (gotBody.startsWith(wantBody)) notes.push(`audit binding intact (normative half matches ${wantBody}... - lifecycle flips do not affect it)`);
    else notes.push(`SPEC DRIFT: the normative half hashes ${gotBody.slice(0, 16)}..., the audit bound ${wantBody}... - clauses/ACs the audit blessed are not the clauses/ACs on disk`);
    const redB = notes.some(n => /FAILED|absent|DRIFT|not pass/.test(n));
    return { verdict: redB ? "red" : "pass", notes };
  }
  const want = String(afm.audited_file_sha256_prefix || "");
  const specRel = t.spec.replace(root + "/", "");
  const auditRel = auditPath.replace(root + "/", "");
  const now = readFileSync(t.spec, "utf8");
  if (!want) {
    notes.push("audit carries no audited_file_sha256_prefix - binding unverifiable");
  } else if (sha(Buffer.from(now)).startsWith(want)) {
    notes.push("audit binding intact (spec byte-identical to its audited form)");
  } else {
    const c = sh("git", ["log", "-n", "1", "--format=%H", "--", auditRel], root);
    const auditCommit = (c.stdout || "").trim();
    const at = auditCommit ? sh("git", ["show", `${auditCommit}:${specRel}`], root) : { status: 1, stdout: "" };
    if (!auditCommit || at.status !== 0) {
      notes.push(`audit binding unverifiable (${want}... matches neither the working spec nor a committed version reachable from the audit's commit) - a human should look`);
    } else if (!sha(Buffer.from(at.stdout)).startsWith(want)) {
      // The prefix matches no COMMITTED version. Known corpus condition (dogfood finding
      // 2026-07-17): authoring records the sha, then the same run flips `status` before
      // committing - so the audited bytes exist only transiently. That is a binding-hygiene
      // gap worth naming, but it is NOT the substantive question, which we can still answer
      // from the audit commit: did the NORMATIVE half change after the audit?
      const same = normativeHalf(at.stdout) === normativeHalf(now);
      notes.push(`audit binding gap: ${want}... matches no committed version of the spec (legacy audit: no audited_body_sha256_prefix (TASK-IMP-102 defines it); pre-flip whole-file hashing binds to a state no commit carries). Substantive check against the audit commit ${auditCommit.slice(0, 8)}: normative half ${same ? "UNCHANGED" : "CHANGED"}`);
      if (!same) notes.push(`SPEC DRIFT: the normative half changed after the audit (audit commit ${auditCommit.slice(0, 8)}) - clauses/ACs the audit blessed are not the clauses/ACs on disk`);
    } else if (normativeHalf(at.stdout) !== normativeHalf(now)) {
      notes.push(`SPEC DRIFT: the normative half changed after the audit (audit commit ${auditCommit.slice(0, 8)}) - clauses/ACs the audit blessed are not the clauses/ACs on disk`);
    } else {
      notes.push(`audit binding intact via the audit commit ${auditCommit.slice(0, 8)}; only lifecycle fields (${LIFECYCLE_FIELDS.join("/")}) changed since - the normative half is byte-identical`);
    }
  }
  const red = notes.some(n => /FAILED|absent|DRIFT|not pass/.test(n));
  return { verdict: red ? "red" : "pass", notes };
}

// ── R2: artefact set for the claimed phase (either home) ─────────────────────
function rung2(root, t, status, id) {
  // The .workflow home is keyed by the task ID itself - NOT by slicing three dash-segments
  // off the folder name (PR-review note, Devin 2026-07-17: that assumed every id is exactly
  // three segments; a differently-shaped id would resolve the wrong home and miss a real
  // artefact bundle). The id is what the caller resolved the folder with, so use it.
  const wfDir = join(root, "docs", "tasks", ".workflow", id);
  const homes = [t.dir, wfDir].filter(existsSync);
  // PR-review fix (Devin, 2026-07-17): read only FILES. A directory whose name matches the
  // bundle pattern (plausible under .workflow/<id>/) made readFileSync throw EISDIR, which
  // escaped rung2 and aborted the whole run with exit 1 - a measuring tool must degrade to a
  // rung verdict, never take the process down. The name still counts toward the bundle text.
  const bundleText = homes.flatMap(h => readdirSync(h)
    .filter(f => /bundle|packet|artefacts|coverage-and-review/i.test(f))
    .map(f => {
      const fp = join(h, f);
      try {
        if (!statSync(fp).isFile()) return f;         // a dir: its NAME still names the artefact
        return `${f}\n${readFileSync(fp, "utf8")}`;
      } catch { return f; }                            // unreadable: the name is what we have
    })).join("\n");
  const want = ARTEFACTS_FOR[status] || [];
  const missing = want.filter(a => {
    if (homes.some(h => existsSync(join(h, a)))) return false;
    const stem = a.replace(/\.md$/, "").replace(/-/g, "[ -]?");
    return !new RegExp(stem, "i").test(bundleText);   // a bundle naming the artefact satisfies it
  });
  return {
    verdict: missing.length ? "red" : "pass",
    notes: missing.length ? [`missing for claimed status '${status}': ${missing.join(", ")} (searched ${homes.map(h => h.replace(root + "/", "")).join(" and ") || "no artefact home"})`] : [`complete for '${status}'`],
    missing,
  };
}

// ── R3: manifest ─────────────────────────────────────────────────────────────
function rung3(root, id) {
  const mf = join(root, "docs", "tasks", ".workflow", `${id}.ship.json`);
  if (!existsSync(mf)) return { verdict: "absent", notes: ["no ship-manifest (out-of-band work has none - a finding, not a failure)"] };
  const sm = join(root, "tools/install/docs-tools/ship-manifest.mjs");
  if (!existsSync(sm)) return { verdict: "absent", notes: ["manifest present but ship-manifest.mjs not in this repo - cannot verify"] };
  const r = sh("node", [sm, "verify", id], root);
  return r.status === 0
    ? { verdict: "pass", notes: ["manifest verifies (resume semantics own this task - see the Reconcile entry §)"] }
    : { verdict: "red", notes: [`manifest verify exit ${r.status}: ${(r.stdout || r.stderr || "").trim().split("\n")[0]}`] };
}

// ── R4: committed-object presence (TASK-IMP-092 rule) ────────────────────────
function rung4(root, fm) {
  const claimed = [...(Array.isArray(fm.new_files) ? fm.new_files : []),
                   ...(Array.isArray(fm.modified_files) ? fm.modified_files : [])].filter(Boolean);
  if (!claimed.length) return { verdict: "absent", notes: ["frontmatter names no new_files/modified_files - nothing to measure"] };
  // PR-review fix (Devin, 2026-07-17): the verdict tracks a BOOLEAN, not the text and array
  // position of a note. The old expression read notes[0].startsWith("all") - correct for every
  // current input, and silently wrong the day anyone pushes an informational note first.
  // A verdict that depends on prose is a verdict waiting to be broken by a comment.
  const notes = []; let anyMissing = false;
  for (const p of claimed) {
    const at = sh("git", ["ls-tree", "HEAD", "--", p], root);
    if (at.status === 0 && at.stdout.trim() !== "") continue;
    anyMissing = true;
    if (existsSync(join(root, p))) notes.push(`UNCOMMITTED CLAIM: ${p} exists on disk but no commit carries it (TASK-IMP-086 class)`);
    else notes.push(`absent at HEAD and on disk: ${p}`);
  }
  if (!anyMissing) notes.push(`all ${claimed.length} claimed path(s) present at HEAD`);
  return { verdict: anyMissing ? "red" : "pass", notes };
}

// ── R5: cited tests, only under --run-tests ──────────────────────────────────
function rung5(root, specText, run) {
  if (!run) return { verdict: "skipped", notes: ["--run-tests not given"] };
  const files = [...new Set([...specText.matchAll(/test:\s*`?([^`\s:]+\.(?:sh|py|mjs|js|ts))/g)].map(m => m[1]))];
  if (!files.length) return { verdict: "absent", notes: ["no test: citations in §2"] };
  const notes = []; let red = false;
  for (const f of files) {
    // PR-review fix (Devin, 2026-07-17): a spec is INPUT, and its `test:` citation is a
    // string this tool hands to bash. Spec §3 already promised "repo-tracked suite files
    // named by the spec, never constructed commands" - the code checked neither. Two gates
    // now hold, in this order:
    //   (1) CONFINEMENT: the path must resolve inside the repo root (a ../ or absolute
    //       citation is refused, not executed) - same predicate as --out and the sibling
    //       coverage-scope.mjs.
    //   (2) TRACKED: the file must exist at HEAD. An untracked file on disk cannot be a
    //       cited suite (TRACE-003 says citations name repo artefacts), and this is the
    //       difference between running the repo's tests and running whatever a crafted
    //       spec dropped in the working tree.
    // A refusal is a RED rung with the reason named - never a silent skip, never a run.
    const rel = relUnderRoot(root, f);
    if (rel === null) { red = true; notes.push(`cited suite path escapes the repo root: ${f} - REFUSED, not executed`); continue; }
    if (!existsSync(join(root, rel))) { red = true; notes.push(`cited suite resolves nowhere: ${rel} (TRACE-003 drift at run time)`); continue; }
    const tracked = sh("git", ["ls-tree", "HEAD", "--", rel], root);
    if (!(tracked.status === 0 && tracked.stdout.trim() !== "")) {
      red = true; notes.push(`cited suite is not tracked at HEAD: ${rel} - REFUSED, not executed (spec §3: repo-tracked suites only)`); continue;
    }
    const r = sh("bash", [rel], root);
    if (r.status !== 0) { red = true; notes.push(`cited suite FAILS now: ${rel} (exit ${r.status})`); }
    else notes.push(`passes: ${rel}`);
  }
  return { verdict: red ? "red" : "pass", notes };
}

function recommend(status, r1, r2, r3, r4, r5) {
  if (NOT_APPLICABLE.has(status)) return { rec: "not_applicable", why: [`status '${status}' claims no work - reconcile is for claims past the entry state`] };
  const reds = [];
  if (r1.verdict === "red") reds.push(`R1 spec integrity: ${r1.notes.join("; ")}`);
  if (r4.verdict === "red") reds.push(`R4 committed object: ${r4.notes.join("; ")}`);
  if (r5.verdict === "red") reds.push(`R5 cited tests: ${r5.notes.join("; ")}`);
  if (r3.verdict === "red") reds.push(`R3 manifest: ${r3.notes.join("; ")}`);
  if (reds.length) return { rec: "route_back", why: reds };
  if (r2.verdict === "red") return { rec: "adopt_candidate", why: [`deliverables are green at HEAD but the phase artefact set is incomplete: ${r2.notes.join("; ")}`, "backfill the artefacts from the evidence, then re-enter at the verified phase"] };
  const n = PHASE_OF[status];
  return { rec: n === null ? "resume_at_phase(confirm-done)" : `resume_at_phase(${n})`,
           why: [n === null ? "every rung supports the done claim - the human confirms done" : `every rung supports '${status}' - resume the chain at step ${n}`] };
}

function report(id, status, rungs, rec, json) {
  const { r1, r2, r3, r4, r5 } = rungs;
  const drift = [r1, r2, r3, r4, r5].filter(r => r.verdict === "red").length;
  if (json) return JSON.stringify({ artefact: "reconcile-report@1", task: id, claimed_status: status,
    rungs: { r1: r1.verdict, r2: r2.verdict, r3: r3.verdict, r4: r4.verdict, r5: r5.verdict },
    drift_score: drift, recommendation: rec.rec, why: rec.why, hitl: "required" }, null, 2) + "\n";
  const sec = (n, t, r) => `### R${n} ${t} - **${r.verdict}**\n${r.notes.map(x => `- ${x}`).join("\n")}\n`;
  return `---
artefact: reconcile-report@1
task: ${id}
claimed_status: ${status}
rungs: { r1: ${r1.verdict}, r2: ${r2.verdict}, r3: ${r3.verdict}, r4: ${r4.verdict}, r5: ${r5.verdict} }
drift_score: ${drift}
recommendation: ${rec.rec}
hitl: required
---

# Reconcile report - ${id} (claims \`${status}\`)

**Recommendation: ${rec.rec}** - this tool never executes it. The verdict is the human's
(ship-tasks Reconcile entry §; modules/skill/task-reconcile/SKILL.md).

${rec.why.map(w => `- ${w}`).join("\n")}

## Evidence ladder

${sec(1, "spec integrity", r1)}
${sec(2, "artefact set vs claimed phase", r2)}
${sec(3, "run manifest", r3)}
${sec(4, "committed-object presence", r4)}
${sec(5, "cited tests now", r5)}
`;
}

function main(argv) {
  const opts = { repo: process.cwd(), runTests: false, json: false, out: null };
  const rest = [];
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--run-tests") opts.runTests = true;
    else if (a === "--json") opts.json = true;
    else if (a === "--repo") opts.repo = argv[++i];
    else if (a === "--out") opts.out = argv[++i];
    else if (a.startsWith("--")) throw new Usage(`unknown flag ${a}`);
    else rest.push(a);
  }
  if (rest.length !== 1) throw new Usage("usage: task-reconcile <task-id> [--repo <root>] [--run-tests] [--json] [--out <file>]");
  const id = rest[0];
  const root = resolve(opts.repo);
  const t = findTask(root, id);
  const specText = readFileSync(t.spec, "utf8");
  const fm = frontmatter(specText, t.spec);
  const status = String(fm.status || "");

  const na = NOT_APPLICABLE.has(status);
  const r1 = na ? { verdict: "skipped", notes: ["not applicable"] } : rung1(root, t);
  const r2 = na ? { verdict: "skipped", notes: ["not applicable"], missing: [] } : rung2(root, t, status, id);
  const r3 = na ? { verdict: "skipped", notes: ["not applicable"] } : rung3(root, id);
  const r4 = na ? { verdict: "skipped", notes: ["not applicable"] } : rung4(root, fm);
  const r5 = na ? { verdict: "skipped", notes: ["not applicable"] } : rung5(root, specText, opts.runTests);
  const rec = recommend(status, r1, r2, r3, r4, r5);
  const text = report(id, status, { r1, r2, r3, r4, r5 }, rec, opts.json);
  if (opts.out) {
    const outRel = relUnderRoot(root, opts.out);
    if (outRel === null) throw new Usage(`--out '${opts.out}' resolves outside the repo root - refused (a read-only instrument writes only where it was pointed)`);
    const p = resolve(root, outRel);
    mkdirSync(dirname(p), { recursive: true });
    writeFileSync(p, text);
    process.stderr.write(`task-reconcile: wrote ${outRel}\n`);
  } else process.stdout.write(text);
  return 0;
}

try { process.exitCode = main(process.argv.slice(2)); }
catch (e) {
  process.stderr.write(`task-reconcile: ${e.message}\n`);
  process.exitCode = e.code || 1;
}
