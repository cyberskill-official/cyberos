#!/usr/bin/env node
// cyberos-version - compute and apply the next CyberOS version from Conventional Commits.
//
// Model = "auto version, manual release" (rolling): the VERSION number moves as updates
// land; cutting a release (tag vX.Y.Z -> release.yml) stays a manual, human action. This
// tool NEVER tags, pushes, or deploys.
//
// Baseline = the last commit that touched VERSION (i.e. the last bump). The bump LEVEL is the
// strongest Conventional-Commit signal among the non-merge commits since that baseline:
//   feat -> minor | fix,perf,revert,refactor -> patch | `!` or `BREAKING CHANGE:` -> major
//   chore,docs,test,ci,build,style -> no bump on their own.
// A `Release-As: X.Y.Z` trailer forces an exact next version (escape hatch).
//
// Usage:
//   node scripts/cyberos-version.mjs [--check]     dry run: print the plan (default), exit 0
//   node scripts/cyberos-version.mjs --apply       write VERSION + CHANGELOG for the next version
//   node scripts/cyberos-version.mjs --level minor|patch|major   force the level
//   node scripts/cyberos-version.mjs --set 1.4.0                 force an exact version
//   node scripts/cyberos-version.mjs --since <ref>              override the baseline
//   node scripts/cyberos-version.mjs --json                     machine-readable output
//   node scripts/cyberos-version.mjs --exit-code                exit 20 when a bump is due (CI gate)
//   node scripts/cyberos-version.mjs --selftest                 run built-in checks (exit 1 on fail)

import { execSync } from "node:child_process";
import { readFileSync, writeFileSync, existsSync, appendFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const RANK = { none: 0, patch: 1, minor: 2, major: 3 };
// Conventional-Commit type -> bump level (a trailing `!` or BREAKING CHANGE always wins as major).
const TYPE_LEVEL = { feat: "minor", fix: "patch", perf: "patch", revert: "patch", refactor: "patch" };

function sh(cmd, opts = {}) { return execSync(cmd, { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"], ...opts }).trim(); }
function repoRoot() { try { return sh("git rev-parse --show-toplevel"); } catch { return process.cwd(); } }

function parseSemver(v) {
  const m = String(v).trim().match(/^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?$/);
  if (!m) throw new Error(`not a semver: "${v}"`);
  return { major: +m[1], minor: +m[2], patch: +m[3], pre: m[4] || null };
}
function fmt(s) { return `${s.major}.${s.minor}.${s.patch}`; }
function bump(v, level) {
  const s = parseSemver(v);
  if (level === "major") return { major: s.major + 1, minor: 0, patch: 0, pre: null };
  if (level === "minor") return { major: s.major, minor: s.minor + 1, patch: 0, pre: null };
  if (level === "patch") return { major: s.major, minor: s.minor, patch: s.patch + 1, pre: null };
  return s; // none
}

// Classify one commit (subject + body) -> { level, breaking, releaseAs }.
function classify(subject, body) {
  const out = { level: "none", breaking: false, releaseAs: null };
  const ra = `${subject}\n${body}`.match(/^\s*Release-As:\s*v?(\d+\.\d+\.\d+)\s*$/m);
  if (ra) out.releaseAs = ra[1];
  const m = subject.match(/^(?<type>[a-z]+)(?:\([^)]*\))?(?<bang>!)?:\s/i);
  if (m) {
    const type = m.groups.type.toLowerCase();
    if (m.groups.bang) out.breaking = true;
    out.level = TYPE_LEVEL[type] || "none";
  }
  if (/^BREAKING CHANGE:/m.test(body) || /^BREAKING-CHANGE:/m.test(body)) out.breaking = true;
  if (out.breaking) out.level = "major";
  return out;
}

function baselineRef(root, since) {
  if (since) return since;
  // last commit that changed VERSION = the last bump; commits after it are "new since last bump".
  const c = sh(`git log -1 --format=%H -- VERSION`, { cwd: root });
  if (c) return c;
  try { return sh(`git describe --tags --match "v*" --abbrev=0`, { cwd: root }); } catch { /* no tag */ }
  return sh(`git rev-list --max-parents=0 HEAD | tail -1`, { cwd: root });
}

function commitsSince(root, base) {
  // NUL-delimited records: <hash>\x1f<subject>\x1f<body>\x1e, no-merges.
  let raw = "";
  try { raw = sh(`git log --no-merges --format=%H%x1f%s%x1f%b%x1e ${base}..HEAD`, { cwd: root }); } catch { return []; }
  if (!raw) return [];
  return raw.split("\x1e").map((r) => r.trim()).filter(Boolean).map((rec) => {
    const [hash, subject, body = ""] = rec.split("\x1f");
    return { hash: (hash || "").trim(), subject: (subject || "").trim(), body: body.trim() };
  });
}

function plan(root, opts) {
  const current = readFileSync(join(root, "VERSION"), "utf8").trim();
  if (opts.set) return { current, next: parseSemver(opts.set) && opts.set, level: "set", reason: `forced --set ${opts.set}`, commits: [] };
  const base = baselineRef(root, opts.since);
  const commits = commitsSince(root, base);
  let level = "none"; let releaseAs = null;
  const kept = [];
  for (const c of commits) {
    const k = classify(c.subject, c.body);
    if (k.releaseAs) releaseAs = k.releaseAs;
    if (RANK[k.level] > RANK[level]) level = k.level;
    if (k.level !== "none" || k.breaking) kept.push({ ...c, ...k });
  }
  if (opts.level) level = opts.level;
  if (releaseAs && !opts.level) return { current, next: releaseAs, level: "release-as", reason: `Release-As: ${releaseAs}`, base, commits: kept };
  const next = level === "none" ? current : fmt(bump(current, level));
  const reason = level === "none"
    ? `no feat/fix/breaking commits since ${base.slice(0, 12)} - no bump`
    : `${level} from ${kept.length} commit(s) since ${base.slice(0, 12)}`;
  return { current, next, level, reason, base, commits: kept };
}

function todayUTC() { return new Date().toISOString().slice(0, 10); }

function synthChangelog(next, kept) {
  const groups = { Added: [], Changed: [], Fixed: [] };
  for (const c of kept) {
    const text = c.subject.replace(/^([a-z]+)(\([^)]*\))?!?:\s*/i, "");
    const type = (c.subject.match(/^([a-z]+)/i) || [, ""])[1].toLowerCase();
    if (type === "feat") groups.Added.push(text);
    else if (type === "fix") groups.Fixed.push(text);
    else groups.Changed.push(text);
  }
  let out = `## [${next}] - ${todayUTC()}\n\n`;
  for (const g of ["Added", "Changed", "Fixed"]) {
    if (!groups[g].length) continue;
    out += `${g}\n${groups[g].map((t) => `- ${t}`).join("\n")}\n\n`;
  }
  if (!groups.Added.length && !groups.Changed.length && !groups.Fixed.length) out += `Maintenance release.\n\n`;
  return out;
}

function applyChangelog(root, next, kept) {
  const p = join(root, "CHANGELOG.md");
  if (!existsSync(p)) return;
  let md = readFileSync(p, "utf8");
  const date = todayUTC();
  // 1) if an "## [Unreleased]" section exists, retitle it.
  const unrel = md.match(/^##\s*\[Unreleased\].*$/mi);
  if (unrel) {
    md = md.replace(unrel[0], `## [${next}] - ${date}`);
  } else {
    // 2) otherwise synthesize a section and insert it before the first existing "## [" heading.
    const block = synthChangelog(next, kept);
    const at = md.search(/^##\s*\[/m);
    md = at >= 0 ? md.slice(0, at) + block + md.slice(at) : `${md.trimEnd()}\n\n${block}`;
  }
  writeFileSync(p, md);
}

function apply(root, p) {
  if (p.next === p.current && p.level !== "set" && p.level !== "release-as") return false;
  if (p.next === p.current) return false;
  writeFileSync(join(root, "VERSION"), `${p.next}\n`);
  applyChangelog(root, p.next, p.commits);
  return true;
}

// --- self-test (no git needed) ------------------------------------------------
function selftest() {
  let pass = 0, fail = 0;
  const eq = (name, got, want) => { if (JSON.stringify(got) === JSON.stringify(want)) pass++; else { fail++; console.error(`FAIL ${name}: got ${JSON.stringify(got)} want ${JSON.stringify(want)}`); } };
  eq("bump minor", fmt(bump("1.0.0", "minor")), "1.1.0");
  eq("bump patch", fmt(bump("1.2.3", "patch")), "1.2.4");
  eq("bump major", fmt(bump("1.2.3", "major")), "2.0.0");
  eq("bump none", fmt(bump("1.2.3", "none")), "1.2.3");
  eq("feat->minor", classify("feat(x): y", "").level, "minor");
  eq("fix->patch", classify("fix: y", "").level, "patch");
  eq("bang->major", classify("feat!: y", "").level, "major");
  eq("breaking body->major", classify("feat: y", "BREAKING CHANGE: z").level, "major");
  eq("chore->none", classify("chore: y", "").level, "none");
  eq("docs->none", classify("docs: y", "").level, "none");
  eq("release-as", classify("chore: y", "Release-As: 2.5.0").releaseAs, "2.5.0");
  eq("scoped bang", classify("refactor(core)!: y", "").breaking, true);
  console.log(`selftest: ${pass} passed, ${fail} failed`);
  return fail === 0 ? 0 : 1;
}

// --- main ---------------------------------------------------------------------
function main() {
  const argv = process.argv.slice(2);
  const has = (f) => argv.includes(f);
  const val = (f) => { const i = argv.indexOf(f); return i >= 0 ? argv[i + 1] : null; };
  if (has("--selftest")) process.exit(selftest());

  const root = repoRoot();
  const opts = { since: val("--since"), level: val("--level"), set: val("--set") };
  if (opts.level && !RANK[opts.level]) { console.error(`bad --level: ${opts.level}`); process.exit(2); }
  const p = plan(root, opts);
  const bumpDue = p.next !== p.current;

  if (has("--apply")) {
    const changed = apply(root, p);
    const line = `CYBEROS_VERSION=${p.next} CYBEROS_VERSION_CHANGED=${changed}`;
    console.log(has("--json") ? JSON.stringify({ ...p, changed }) : `${changed ? "bumped" : "no change"}: ${p.current} -> ${p.next}  (${p.reason})\n${line}`);
    if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, `version=${p.next}\nchanged=${changed}\n`);
    process.exit(0);
  }

  // default / --check
  if (has("--json")) console.log(JSON.stringify(p));
  else {
    console.log(`current: ${p.current}`);
    console.log(`next:    ${p.next}  (${p.level})`);
    console.log(`reason:  ${p.reason}`);
    if (p.commits && p.commits.length) console.log(`commits:\n${p.commits.map((c) => `  ${c.level.padEnd(5)} ${c.subject}`).join("\n")}`);
  }
  if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, `version=${p.next}\nchanged=${bumpDue}\n`);
  process.exit(has("--exit-code") && bumpDue ? 20 : 0);
}

main();

export { parseSemver, bump, classify, synthChangelog };
