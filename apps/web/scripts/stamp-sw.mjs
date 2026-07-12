// Stamp the service worker's cache name with a per-build id (run by `npm run build` after vite build).
// Vite copies public/sw.js into the output verbatim; this rewrites the __BUILD__ placeholder there so every
// deploy activates a fresh cache and the activate handler deletes the old ones. The source file in public/
// keeps the placeholder (a literal "__BUILD__" is still a valid cache name for dev serves).
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const out = resolve(here, "../../console/web/sw.js");

const id = new Date().toISOString().replace(/[-:TZ.]/g, "").slice(0, 14); // yyyymmddhhmmss
const src = readFileSync(out, "utf8");
if (!src.includes("__BUILD__")) {
  console.error("stamp-sw: __BUILD__ placeholder not found in", out, "- was the build output moved?");
  process.exit(1);
}
const stamped = src.replaceAll("__BUILD__", id);
if (!stamped.includes(`const CACHE = "cyberos-shell-${id}"`)) {
  console.error("stamp-sw: the CACHE constant did not stamp - check public/sw.js");
  process.exit(1);
}
writeFileSync(out, stamped);
console.log(`stamp-sw: cache cyberos-shell-${id}`);

// Also publish version.json so the running client can detect a newer deploy (src/lib/useUpdateCheck.ts).
// The build id is the same one that stamps the SW cache, so every deploy bumps it exactly once.
//
// The version comes from the root VERSION - the single platform source of truth - NOT from
// apps/web/package.json. package.json is only reconciled with VERSION by scripts/stamp-release-version.mjs,
// which runs inside release CI on a tag and never commits its edits back. The web bundle, however, is built
// by hand and committed (apps/console/web), so it never sees that stamp: the badge on os.cyberskill.world
// sat at 1.2.0 while the platform shipped 1.7.0. Reading VERSION directly makes the deployed version honest
// regardless of whether the release stamper ever ran.
const version = readFileSync(resolve(here, "../../../VERSION"), "utf8").trim();
if (!/^\d+\.\d+\.\d+$/.test(version)) {
  console.error(`stamp-sw: VERSION is not semver: "${version}"`);
  process.exit(1);
}
const versionOut = resolve(here, "../../console/web/version.json");
writeFileSync(versionOut, JSON.stringify({ build: id, version }) + "\n");
console.log(`stamp-sw: version.json build ${id} v${version}`);
