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
