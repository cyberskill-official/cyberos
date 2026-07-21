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
//                                            versionCode    -> BUILD_NUMBER
//   apps/web/ios/.../project.pbxproj          MARKETING_VERSION       -> X.Y.Z
//                                             CURRENT_PROJECT_VERSION -> BUILD_NUMBER
//   apps/desktop/src-tauri/snap/snapcraft.yaml   version        -> X.Y.Z
//   apps/desktop/src-tauri/AppxManifest.xml      Identity Version -> X.Y.Z.BUILD_NUMBER
//
// THE STORE BUILD NUMBER IS DECOUPLED FROM SEMVER, ON PURPOSE.
//
// It used to be derived: major*10000 + minor*100 + patch. That is deterministic and reads nicely, and
// it is a trap. Google Play remembers every versionCode it has ever seen and refuses anything that is
// not strictly higher - forever. Play has already accepted 10700 (from 1.7.0). The moment VERSION was
// rolled back to 0.1.0 for the pre-1.0 run-up, the derived code would have become 100, and EVERY future
// Android upload would have been rejected with no way back. Apple has the same rule for
// CFBundleVersion within a version string.
//
// So the store build number now comes from a dedicated BUILD_NUMBER file that only ever increments
// (bumped by scripts/cyberos-version.mjs alongside VERSION). It is seeded at 10701 - one above Play's
// high-water mark - so the rollback to 0.x is safe. The marketing version (0.1.0) and the build number
// (10701+) are simply different things: one is what humans read, the other is a monotonic counter the
// stores use to order uploads. Conflating them cost us an irreversible mistake once; do not re-couple
// them.
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
if (!/^\d+\.\d+\.\d+$/.test(version)) {
  console.error(`stamp: VERSION is not semver: "${version}"`);
  process.exit(2);
}

// The monotonic store build number. Never derived from VERSION - see the header.
const buildNumberPath = join(root, "BUILD_NUMBER");
if (!existsSync(buildNumberPath)) {
  console.error("stamp: BUILD_NUMBER missing. It is the monotonic counter Google Play and App Store Connect order uploads by, and it cannot be recomputed from VERSION.");
  process.exit(2);
}
const buildNumberFile = Number(readFileSync(buildNumberPath, "utf8").trim());
if (!Number.isInteger(buildNumberFile) || buildNumberFile < 1) {
  console.error(`stamp: BUILD_NUMBER is not a positive integer: "${readFileSync(buildNumberPath, "utf8").trim()}"`);
  process.exit(2);
}
// Play has already accepted 10700. Anything at or below it is unshippable on Android, permanently.
const PLAY_HIGH_WATER_MARK = 10700;
if (buildNumberFile <= PLAY_HIGH_WATER_MARK) {
  console.error(`stamp: BUILD_NUMBER ${buildNumberFile} is <= ${PLAY_HIGH_WATER_MARK}, which Google Play has already consumed. Play refuses any versionCode it has seen. Raise BUILD_NUMBER.`);
  process.exit(2);
}

// --store-monotonic (TASK-IMP-078, release CI only): lift the EFFECTIVE build number to
// max(BUILD_NUMBER, minutes-since-epoch). BUILD_NUMBER only moves when version.yml bumps
// VERSION - so while `Release-As` pins the version (as the whole 1.0.0 run-up does), every
// re-tag rebuilt the SAME committed number, and both stores refuse a build number they have
// already consumed. Observed live on v1.0.0: Play rejected the second upload of versionCode
// 10706, and ASC holds CFBundleVersion 10706 for iOS 1.0.0, so the next re-tag would have
// failed the iOS lane identically. Wall-clock minutes are strictly increasing across CI runs,
// immune to same-commit re-tags, and sit far under Play's 2100000000 cap (~29.7M in 2026).
// Only release.yml's android/iOS stamp steps pass the flag; plain --check/--apply keep the
// committed BUILD_NUMBER baseline, so repo state stays deterministic.
const storeMonotonic = process.argv.includes("--store-monotonic");
const versionCode = storeMonotonic
  ? Math.max(buildNumberFile, Math.floor(Date.now() / 60000))
  : buildNumberFile;

const apply = process.argv.includes("--apply");
const changes = [];

function stampCargo(rel) {
  const p = join(root, rel);
  if (!existsSync(p)) return;
  let text = readFileSync(p, "utf8");
  const before = text.match(/^version = "([^"]+)"/m)?.[1];
  if (before === version) return;
  changes.push(`${rel}: version ${before} -> ${version}`);
  if (apply) writeFileSync(p, text.replace(/^version = "[^"]+"/m, `version = "${version}"`));
}

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

// iOS carries the same two numbers as Android under different names, in the Xcode project:
//   MARKETING_VERSION       -> the user-visible "1.7.0" (CFBundleShortVersionString)
//   CURRENT_PROJECT_VERSION -> the build number (CFBundleVersion)
// Capacitor scaffolds both as "1.0" / 1 and never touches them again. App Store Connect REJECTS an upload
// whose build number it has already seen for that version - exactly the failure mode that made Android's
// hardcoded versionCode unshippable. MARKETING_VERSION comes from VERSION; CURRENT_PROJECT_VERSION comes
// from BUILD_NUMBER (see the header for why they are not the same number). Both keys appear once per build
// configuration (Debug + Release), so replace ALL.
function stampXcodeProj(rel) {
  const p = join(root, rel);
  if (!existsSync(p)) return;
  let raw = readFileSync(p, "utf8");
  const curName = (raw.match(/MARKETING_VERSION = ([^;]*);/) || [])[1];
  const curCode = (raw.match(/CURRENT_PROJECT_VERSION = ([^;]*);/) || [])[1];
  if (curName === version && String(curCode) === String(versionCode)) return;
  changes.push(
    `${rel}: MARKETING_VERSION ${curName} -> ${version}, CURRENT_PROJECT_VERSION ${curCode} -> ${versionCode}`,
  );
  if (!apply) return;
  raw = raw.replaceAll(/MARKETING_VERSION = [^;]*;/g, `MARKETING_VERSION = ${version};`);
  raw = raw.replaceAll(/CURRENT_PROJECT_VERSION = [^;]*;/g, `CURRENT_PROJECT_VERSION = ${versionCode};`);
  writeFileSync(p, raw);
}

// Snap Store metadata: cosmetic-only field (no re-upload-rejection rule like the app stores),
// so it just gets the plain marketing version - no BUILD_NUMBER component needed.
function stampYamlVersion(rel) {
  const p = join(root, rel);
  if (!existsSync(p)) return;
  let raw = readFileSync(p, "utf8");
  const cur = (raw.match(/^version:\s*'([^']*)'/m) || [])[1];
  if (cur === version) return;
  changes.push(`${rel}: version ${cur} -> ${version}`);
  if (!apply) return;
  raw = raw.replace(/^version:\s*'[^']*'/m, `version: '${version}'`);
  writeFileSync(p, raw);
}

// MSIX Identity Version is a strict 4-part N.N.N.N (each 0-65535), and the Microsoft Store
// Submission API refuses to accept a package version it has already seen for the app - the same
// re-upload rule Android/iOS have. So this follows their exact pattern: X.Y.Z from VERSION,
// BUILD_NUMBER as the 4th (monotonic) component.
function stampAppxManifest(rel) {
  const p = join(root, rel);
  if (!existsSync(p)) return;
  let raw = readFileSync(p, "utf8");
  const cur = (raw.match(/Version="([^"]*)"/) || [])[1];
  const want = `${version}.${versionCode}`;
  if (cur === want) return;
  changes.push(`${rel}: Version ${cur} -> ${want}`);
  if (!apply) return;
  raw = raw.replace(/Version="[^"]*"/, `Version="${want}"`);
  writeFileSync(p, raw);
}

stampJson("apps/desktop/src-tauri/tauri.conf.json");
// task 1.0.0-consistency leg: the tauri CARGO package version feeds about-dialogs and crate metadata -
// stamp it too so the desktop app never self-reports a stale number.
stampCargo("apps/desktop/src-tauri/Cargo.toml");
// The MCP server source lives at tools/install/mcp/ (build.sh copies $here/mcp into the
// payload). Its package.json is stamped into the PAYLOAD copy by build.sh; stamping the repo
// source keeps the whole codebase consistent (build.sh's stamp becomes a no-op). NOTE: the path
// was originally written as root-level "mcp/package.json", which never existed - the existsSync
// guard hid it here, but version.yml's matching `git add` failed fatally on the first real bump.
stampJson("tools/install/mcp/package.json");
stampJson("apps/web/package.json");
stampGradle("apps/web/android/app/build.gradle");
stampXcodeProj("apps/web/ios/App/App.xcodeproj/project.pbxproj");
// Added so the Snap and MS Store listings can no longer ship a version number that disagrees
// with everything else - same discipline as the targets above (Stephen approved 2026-07-21).
stampYamlVersion("apps/desktop/src-tauri/snap/snapcraft.yaml");
stampAppxManifest("apps/desktop/src-tauri/AppxManifest.xml");

console.log(`VERSION=${version}  BUILD_NUMBER=${versionCode}  (androidVersionCode + iosBuildNumber)`);
if (!changes.length) {
  console.log("all release artifacts already match VERSION - nothing to stamp.");
  process.exit(0);
}
for (const c of changes) console.log(`  ${apply ? "stamped" : "DRIFT "} ${c}`);
if (!apply && process.argv.includes("--exit-code")) process.exit(10);
process.exit(0);
