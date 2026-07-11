#!/usr/bin/env node
// stamp-release-version - propagate the single platform VERSION into every shippable artifact.
//
// The root VERSION is the source of truth (auto-bumped by .github/workflows/version.yml). But the
// installers carry their OWN version fields, and they had silently drifted: tauri.conf.json and
// apps/web/package.json still said 1.0.0 while VERSION said 1.2.0, and Android's versionCode was
// hardcoded to 1 - which Google Play REJECTS on any re-upload, because versionCode must strictly
// increase. This stamps them all from VERSION, so a release cannot ship a mislabelled binary.
//
// Targets:
//   apps/desktop/src-tauri/tauri.conf.json   version        -> X.Y.Z
//   apps/web/package.json                    version        -> X.Y.Z
//   apps/web/android/app/build.gradle        versionName    -> "X.Y.Z"
//                                            versionCode    -> major*10000 + minor*100 + patch
//
// versionCode is derived (not incremented) so it is deterministic and reproducible from VERSION
// alone: 1.2.0 -> 10200, 1.2.1 -> 10201, 2.0.0 -> 20000. Strictly increasing for any semver bump
// while minor/patch stay under 100.
//
// Usage:
//   node scripts/stamp-release-version.mjs            # --check: report drift, exit 0
//   node scripts/stamp-release-version.mjs --apply    # write the files
//   node scripts/stamp-release-version.mjs --check --exit-code   # exit 10 if anything is out of date

import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { execSync } from "node:child_process";
import { join } from "node:path";

const root = (() => {
  try { return execSync("git rev-parse --show-toplevel", { encoding: "utf8" }).trim(); } catch { return process.cwd(); }
})();

const version = readFileSync(join(root, "VERSION"), "utf8").trim();
const m = version.match(/^(\d+)\.(\d+)\.(\d+)$/);
if (!m) { console.error(`stamp: VERSION is not semver: "${version}"`); process.exit(2); }
const [, MAJ, MIN, PAT] = m.map(Number);
const versionCode = MAJ * 10000 + MIN * 100 + PAT;

const apply = process.argv.includes("--apply");
const changes = [];

function stampJson(rel, field = "version") {
  const p = join(root, rel);
  if (!existsSync(p)) return;
  const raw = readFileSync(p, "utf8");
  const cur = JSON.parse(raw)[field];
  if (cur === version) return;
  changes.push(`${rel}: ${field} ${cur} -> ${version}`);
  if (!apply) return;
  // regex-replace so formatting/key order survive untouched
  const out = raw.replace(new RegExp(`("${field}"\\s*:\\s*")[^"]*(")`), `$1${version}$2`);
  writeFileSync(p, out);
}

function stampGradle(rel) {
  const p = join(root, rel);
  if (!existsSync(p)) return;
  let raw = readFileSync(p, "utf8");
  const curName = (raw.match(/versionName\s+"([^"]*)"/) || [])[1];
  const curCode = (raw.match(/versionCode\s+(\d+)/) || [])[1];
  if (curName === version && String(curCode) === String(versionCode)) return;
  changes.push(`${rel}: versionName ${curName} -> ${version}, versionCode ${curCode} -> ${versionCode}`);
  if (!apply) return;
  raw = raw.replace(/versionName\s+"[^"]*"/, `versionName "${version}"`);
  raw = raw.replace(/versionCode\s+\d+/, `versionCode ${versionCode}`);
  writeFileSync(p, raw);
}

stampJson("apps/desktop/src-tauri/tauri.conf.json");
stampJson("apps/web/package.json");
stampGradle("apps/web/android/app/build.gradle");

console.log(`VERSION=${version}  androidVersionCode=${versionCode}`);
if (!changes.length) {
  console.log("all release artifacts already match VERSION - nothing to stamp.");
  process.exit(0);
}
for (const c of changes) console.log(`  ${apply ? "stamped" : "DRIFT "} ${c}`);
if (!apply && process.argv.includes("--exit-code")) process.exit(10);
process.exit(0);
