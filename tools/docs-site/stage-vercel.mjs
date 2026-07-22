// Stage the generated site for Vercel: the docs live under /docs (the docs homepage
// at os.cyberskill.world/docs), with the domain root redirecting there. Run after
// build.sh; consumed by vercel.json (outputDirectory: .vercel-out, gitignored).
import { rmSync, mkdirSync, cpSync, writeFileSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const SITE = join(ROOT, "dist", "website");
const OUT = join(ROOT, ".vercel-out");

if (!existsSync(join(SITE, "index.html"))) {
  console.error("stage-vercel: dist/website missing - run tools/docs-site/build.sh first");
  process.exit(1);
}

rmSync(OUT, { recursive: true, force: true });
mkdirSync(join(OUT, "docs"), { recursive: true });
cpSync(SITE, join(OUT, "docs"), { recursive: true });
writeFileSync(
  join(OUT, "index.html"),
  '<!doctype html><meta charset="utf-8"><meta http-equiv="refresh" content="0; url=/docs/"><link rel="canonical" href="/docs/"><title>CyberOS docs</title><a href="/docs/">CyberOS docs</a>\n'
);
console.log("stage-vercel: staged at .vercel-out (site under /docs, root redirects there)");
