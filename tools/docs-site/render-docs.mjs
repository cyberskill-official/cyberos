// render-docs.mjs — TASK-DOCS-002: generate the doctrine pages of the website from the
// markdown single source of truth. Module-owned docs live next to the code
// (modules/<m>/docs, services/<s>/docs); pre-code modules and global artifacts live
// under docs/. Output paths mirror the existing site structure, so nav links keep
// working. Deterministic: same input ⇒ byte-identical output. Fails non-zero on a
// missing referenced asset or unreadable source (TASK-DOCS-002 §1 #7).

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
<p class="text-sm" style="opacity:.6">Generated from <code>${sourceRel}</code> — edit the markdown source, not this file (TASK-DOCS-002).</p>
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

// ── tools scopes: tools/* with a deliberate docs home (docs/index.md) ────────
// The index.md requirement keeps vendored third-party trees (e.g. tools/caf/docs
// governance files) off the site: only tools that author a docs home render.
const toolsDir = join(ROOT, "tools");
if (existsSync(toolsDir)) {
  for (const t of readdirSync(toolsDir).sort()) {
    if (existsSync(join(toolsDir, t, "docs", "index.md"))) {
      renderScope(join(toolsDir, t, "docs"), join(SITE, "tools", t), 2);
    }
  }
}

// ── generate the shared nav from the pages that ACTUALLY rendered ────────────
// The chrome ships a static nav.html, but it hardcoded modules and links that do
// not all exist (e.g. modules with no docs page yet) - every such link 404s. Emit
// a nav from the real output tree instead, so a link exists iff its page does.
// {{ROOT}} stays a placeholder; scripts.js substitutes it per-page at runtime.
function titleOf(scopeIndexMd, fallback) {
  try {
    const { meta } = frontmatter(readFileSync(scopeIndexMd, "utf-8"));
    if (meta.title) return String(meta.title).split(/·| - /)[0].trim();
  } catch {}
  return fallback;
}
function generateNav() {
  const has = (rel) => existsSync(join(SITE, rel));

  const navModules = [...mods]
    .sort()
    .filter((m) => has(join("modules", m, "index.html")))
    .map((m) => {
      const label = titleOf(join(moduleHome(m), "index.md"), m);
      const name = label.toLowerCase() === m ? m : label;
      return `   <a class="nav-dropdown-link" href="{{ROOT}}modules/${m}/index.html"><span>${name}</span><span class="code">${m}</span></a>`;
    })
    .join("\n");

  const archOrder = ["infrastructure", "tech-stack", "compliance", "verification-gate", "milestones", "strategy"];
  const archLabels = { "tech-stack": "Tech stack", "verification-gate": "Verification gate" };
  const navArch = archOrder
    .filter((a) => has(join("architecture", `${a}.html`)))
    .map((a) => `   <a class="nav-dropdown-link" href="{{ROOT}}architecture/${a}.html">${archLabels[a] || a[0].toUpperCase() + a.slice(1)}</a>`)
    .join("\n");

  const refItems = [
    ["reference/getting-started.html", "Getting started"],
    ["reference/fr-catalog.html", "FR catalog"],
    ["reference/nfr-catalog.html", "NFR catalog"],
    ["reference/changelog.html", "Changelog"],
    ["reference/status.html", "Status"],
    ["reference/glossary.html", "Glossary"],
    ["reference/risk-register.html", "Risk register"],
  ];
  const navRef = refItems
    .filter(([p]) => has(p))
    .map(([p, l]) => `   <a class="nav-dropdown-link" href="{{ROOT}}${p}">${l}</a>`)
    .join("\n");

  const toolDirs = existsSync(join(ROOT, "tools"))
    ? readdirSync(join(ROOT, "tools")).sort().filter((t) => has(join("tools", t, "index.html")))
    : [];
  const navTools = toolDirs
    .map((t) => `   <a class="nav-dropdown-link" href="{{ROOT}}tools/${t}/index.html">${titleOf(join(ROOT, "tools", t, "docs", "index.md"), t)}</a>`)
    .join("\n");

  const roadmap = has(join("architecture", "milestones.html"))
    ? `\n  <a class="nav-link" href="{{ROOT}}architecture/milestones.html" data-nav-key="milestones.html">Roadmap</a>`
    : "";

  const nav = `<!-- GENERATED by tools/docs-site/render-docs.mjs from the real page set - do not hand-edit. -->
<nav class="sticky-nav no-print">
 <div class="nav-inner">
  <a class="nav-brand" href="{{ROOT}}index.html" data-nav-key="index.html">
   <img class="nav-logo" src="{{ROOT}}assets/cyberskill-logo.svg" alt="CyberSkill" style="object-fit:contain" />
   <div class="nav-brand-text"><div class="title">CyberOS</div><div class="subtitle">documentation</div></div>
  </a>
  <div class="nav-links">
   <a class="nav-link" href="{{ROOT}}index.html" data-nav-key="index.html">Overview</a>
   <div class="nav-dropdown">
    <button class="nav-link" type="button">Modules <span aria-hidden="true">▾</span></button>
    <div class="nav-dropdown-menu cols-2">
${navModules}
    </div>
   </div>
   <div class="nav-dropdown">
    <button class="nav-link" type="button">Architecture <span aria-hidden="true">▾</span></button>
    <div class="nav-dropdown-menu">
${navArch}
    </div>
   </div>
   <div class="nav-dropdown">
    <button class="nav-link" type="button">Reference <span aria-hidden="true">▾</span></button>
    <div class="nav-dropdown-menu">
${navRef}
    </div>
   </div>${navTools ? `
   <div class="nav-dropdown">
    <button class="nav-link" type="button">Tooling <span aria-hidden="true">▾</span></button>
    <div class="nav-dropdown-menu">
${navTools}
    </div>
   </div>` : ""}${roadmap}
  </div>
  <div class="nav-actions">
   <button id="nav-print" class="nav-btn" type="button" title="Print">Print</button>
   <button id="nav-dark" class="nav-btn" type="button" title="Toggle theme" aria-label="Toggle theme">☾</button>
   <button id="nav-menu" class="nav-btn" type="button" title="Menu" aria-label="Open menu" style="display:none">☰</button>
  </div>
 </div>
</nav>
`;
  mkdirSync(join(SITE, "assets"), { recursive: true });
  writeFileSync(join(SITE, "assets", "nav.html"), nav);
}
generateNav();

if (failures > 0) {
  console.error(`render-docs: ${failures} failure(s)`);
  process.exit(1);
}
console.log(`render-docs: ${written} pages generated from markdown sources`);
