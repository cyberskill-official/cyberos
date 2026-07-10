// render-docs.mjs — FR-DOCS-002: generate the doctrine pages of the website from the
// markdown single source of truth. Module-owned docs live next to the code
// (modules/<m>/docs, services/<s>/docs); pre-code modules and global artifacts live
// under docs/. Output paths mirror the existing site structure, so nav links keep
// working. Deterministic: same input ⇒ byte-identical output. Fails non-zero on a
// missing referenced asset or unreadable source (FR-DOCS-002 §1 #7).

import { readFileSync, writeFileSync, mkdirSync, readdirSync, existsSync, copyFileSync, statSync } from "node:fs";
import { join, dirname, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { renderMarkdown, frontmatter } from "./md.mjs";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const SITE = join(ROOT, "dist", "website");

// module -> docs source home. Default rule: EVERY module owns its docs at
// modules/<m>/docs from day one (pre-code modules included — the folder is the
// module's home before any code lands). Service-implemented modules whose code
// lives under services/ carry their docs next to that code instead.
const SERVICE_HOMES = {
  ai: "services/ai-gateway/docs",
  auth: "services/auth/docs",
  chat: "services/chat/docs",
  email: "services/email/docs",
  proj: "services/proj/docs",
  mcp: "services/mcp-gateway/docs",
  obs: "services/obs-collector/docs",
};

const moduleHome = (mod) => join(ROOT, SERVICE_HOMES[mod] ?? join("modules", mod, "docs"));

function page(title, depth, bodyHtml, sourceRel) {
  const up = "../".repeat(depth);
  return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${title}</title>
<link rel="stylesheet" href="${up}assets/tokens.css">
<link rel="stylesheet" href="${up}assets/styles.css">
<link rel="stylesheet" href="${up}assets/tailwind.min.css">
<script type="module" src="${up}assets/scripts.js"></script>
</head>
<body>
<div id="shared-nav"></div>
<main class="docs-page mx-auto max-w-4xl px-6 py-10">
${bodyHtml}
<hr>
<p class="text-sm" style="opacity:.6">Generated from <code>${sourceRel}</code> — edit the markdown source, not this file (FR-DOCS-002).</p>
</main>
</body>
</html>
`;
}

let failures = 0;
let written = 0;

function renderOne(srcAbs, outAbs, depth) {
  let raw;
  try {
    raw = readFileSync(srcAbs, "utf-8");
  } catch (e) {
    console.error(`UNREADABLE ${relative(ROOT, srcAbs)}: ${e.message}`);
    failures++;
    return;
  }
  const { meta, body } = frontmatter(raw);
  const html = renderMarkdown(body);
  const title = meta.title || relative(ROOT, srcAbs);
  mkdirSync(dirname(outAbs), { recursive: true });
  writeFileSync(outAbs, page(title, depth, html, relative(ROOT, srcAbs)));
  written++;
  // asset check: every relative ./assets/... or assets/... reference must exist next to the OUTPUT.
  for (const m of body.matchAll(/!\[[^\]]*\]\((\.?\/?assets\/[^)\s]+)\)/g)) {
    const assetOut = join(dirname(outAbs), m[1].replace(/^\.\//, ""));
    if (!existsSync(assetOut)) {
      console.error(`MISSING ASSET ${m[1]} referenced by ${relative(ROOT, srcAbs)}`);
      failures++;
    }
  }
}

function copyAssets(scopeDirAbs, outDirAbs) {
  const assets = join(scopeDirAbs, "assets");
  if (!existsSync(assets)) return;
  const walk = (dir, rel) => {
    for (const name of readdirSync(dir).sort()) {
      const p = join(dir, name);
      const r = join(rel, name);
      if (statSync(p).isDirectory()) walk(p, r);
      else {
        const dst = join(outDirAbs, "assets", r);
        mkdirSync(dirname(dst), { recursive: true });
        copyFileSync(p, dst);
      }
    }
  };
  walk(assets, "");
}

function renderScope(scopeDirAbs, outDirAbs, depth) {
  if (!existsSync(scopeDirAbs)) return;
  copyAssets(scopeDirAbs, outDirAbs); // before rendering: the per-page asset check looks at the OUTPUT tree
  for (const name of readdirSync(scopeDirAbs).sort()) {
    if (!name.endsWith(".md")) continue;
    const out = join(outDirAbs, name.replace(/\.md$/, ".html"));
    renderOne(join(scopeDirAbs, name), out, depth);
  }
  const guides = join(scopeDirAbs, "guides");
  if (existsSync(guides)) {
    for (const name of readdirSync(guides).sort()) {
      if (!name.endsWith(".md")) continue;
      renderOne(join(guides, name), join(outDirAbs, "guides", name.replace(/\.md$/, ".html")), depth + 1);
    }
  }
}

// ── global scopes ────────────────────────────────────────────────────────────
renderScope(join(ROOT, "docs", "architecture"), join(SITE, "architecture"), 1);
for (const name of ["getting-started", "glossary", "risk-register"]) {
  const src = join(ROOT, "docs", "reference", `${name}.md`);
  if (existsSync(src)) renderOne(src, join(SITE, "reference", `${name}.html`), 1);
}

// ── module scopes: union of service-homed modules and modules/* with a docs/ ─
const mods = new Set(Object.keys(SERVICE_HOMES));
const modulesDir = join(ROOT, "modules");
if (existsSync(modulesDir)) {
  for (const m of readdirSync(modulesDir).sort()) {
    if (existsSync(join(modulesDir, m, "docs"))) mods.add(m);
  }
}
for (const mod of [...mods].sort()) {
  renderScope(moduleHome(mod), join(SITE, "modules", mod), 2);
}

if (failures > 0) {
  console.error(`render-docs: ${failures} failure(s)`);
  process.exit(1);
}
console.log(`render-docs: ${written} pages generated from markdown sources`);
