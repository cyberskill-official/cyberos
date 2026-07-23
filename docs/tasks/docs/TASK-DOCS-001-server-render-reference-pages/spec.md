---
id: TASK-DOCS-001
title: "Server-render NFR catalog + Risk Register + task catalog at build time — Pagefind-indexed + crawler-visible + deterministic + Alpine reactive coexistence"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: docs
priority: p1
status: closed
verify: I
phase: P0
milestone: P0 · polish slice
slice: 1
owner: Stephen Cheng
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_tasks: []
depends_on: []
blocks: []

source_pages:
  - website/docs/reference/nfr-catalog.html
  - website/docs/reference/risk-register.html
  - website/docs/reference/task-catalog.html
source_decisions:
  #1, #2)
  - docs/archive/2026-05-14/RESEARCH_REVIEW.md §4 (UX defects
  - docs/archive/2026-05-14/RESEARCH_REVIEW.md §5.4 (reference-page weighting)
  - DEC-220 (deterministic build; CI drift detection)
  - DEC-221 (single source of truth in JSON; HTML pages consume; Alpine reactive uses same data)

language: nodejs 20
service: cyberos/website/
new_files:
  - website/build/render-reference-pages.mjs
  - website/build/data-extract.mjs
  - website/build/templates/nfr-catalog.hbs
  - website/build/templates/risk-register.hbs
  - website/build/templates/task-catalog.hbs
  - website/build/templates/_card.hbs
  - website/build/data/nfrs.json
  - website/build/data/risks.json
  - website/build/data/tasks.json
  - website/build/tests/render_test.mjs
  - website/build/tests/determinism_test.mjs
  - website/build/tests/pagefind_index_test.mjs
  - .github/workflows/docs-prerender-gate.yml
modified_files:
  # gain server-rendered <section> blocks
  - website/docs/reference/nfr-catalog.html
  - website/docs/reference/risk-register.html
  - website/docs/reference/task-catalog.html
  # use JSON data; hide prerendered post-hydrate
  - website/docs/reference/_alpine-init.js
  # add `node build/render-reference-pages.mjs` step
  - .github/workflows/deploy.yml
  # cheerio, handlebars deps
  - website/package.json
allowed_tools:
  - file_read: website/**
  - file_write: website/build/**, website/docs/reference/**, .github/workflows/**
  - bash: cd website && node build/render-reference-pages.mjs
  - bash: cd website && pagefind --site docs/
disallowed_tools:
  - mutating data inline in HTML (per DEC-221 — JSON is source of truth)
  - introducing non-deterministic build steps (per DEC-220)
  #4 — UX preserved)
  - removing Alpine reactive (per §1

effort_hours: 14
subtasks:
  - "1.0h: data-extract.mjs — read existing HTML, extract NFR_DATA, RISKS, TASK_CATALOG arrays via cheerio + JS evaluation"
  - "1.0h: Persist arrays to JSON files in website/build/data/"
  - "1.0h: Handlebars templates per page (3 templates + 1 shared _card.hbs)"
  - "1.0h: render-reference-pages.mjs main script (read JSON → render → inject into HTML)"
  - "1.0h: Inject `<section data-prerendered=\"true\">` blocks BEFORE the Alpine `<template>` tags"
  - "0.5h: Alpine init modification — hide `[data-prerendered]` once `hydrated=true`"
  - "0.5h: Avoid double-render flash via `x-cloak` + CSS `[x-cloak] { display: none }`"
  - "1.0h: Determinism check — sort arrays, stable iteration, no Date.now() calls"
  - "1.0h: render_test.mjs — assert output contains expected NFR-IDs / RSK-IDs / task-IDs"
  - "1.0h: determinism_test.mjs — run twice; assert byte-identical"
  - "1.0h: pagefind_index_test.mjs — re-index + assert search hits"
  - "0.5h: docs-prerender-gate.yml CI workflow"
  - "1.0h: Pagefind re-index step in deploy.yml"
  - "1.0h: Cloudflare Pages build configuration update"
  - "0.5h: Documentation in website/build/README.md"
risk_if_skipped: "First paint of /reference/* shows empty scaffold + 'of NFRs match' placeholder. Pagefind cannot index the catalog data (search for NFR-PE-01 returns 0 results). Crawlers (Google, Bing) see empty pages for the reference surface that auditors and procurement teams will query first. The compliance + procurement story degrades from 'visible at static HTML' to 'visible only after Alpine hydrates.' Without determinism + CI gate, the prerendered HTML drifts from the data — operators discover via stale procurement responses."
---

## §1 — Description (BCP-14 normative)

The CyberOS documentation build pipeline **MUST** server-render the three reference catalog pages at build time, producing static HTML rows visible at first paint AND indexed by Pagefind. Each piece:

1. **MUST** parse the data arrays (`NFR_DATA`, `RISKS`, `TASK_CATALOG`) at build time. Source-of-truth becomes JSON files at `website/build/data/{nfrs,risks,tasks}.json` (DEC-221); both the build script AND the Alpine reactive layer consume from JSON.
2. **MUST** emit a `<section data-prerendered="true">` block per page, containing the same `<article>` cards Alpine would render at runtime. Prerendered cards include all card content (title, IDs, descriptions, badges) — visible without JS.
3. **MUST** be deterministic — running the build twice on the same JSON inputs produces byte-identical HTML output. No Date.now(), no RNG, no unsorted iteration. CI gate (§1 #11) enforces.
4. **MUST** preserve Alpine reactive behaviour: on JS hydration, `hydrated=true` is set; CSS `[x-cloak] { display: none }` hides prerendered AND Alpine takes over. No double-render flash.
5. **MUST** preserve filter/sort UX from current Alpine implementation. Post-hydrate clicks still filter/sort the (now-Alpine-rendered) cards.
6. **MUST** include all metadata fields per card type:
- **NFR**: id, category, target, phase, modules, references, description.
- **Risk**: id, severity, likelihood, mitigation, owner, status.
- **task**: id, module, priority, status, slice, owner, depends_on, blocks.
7. **MUST** generate stable IDs/anchors so external links (e.g., `https://docs.cyberos.world/reference/nfr-catalog.html#NFR-PERF-01`) work. Each card has `id="<NFR-ID>"` attribute.
8. **MUST** integrate with Cloudflare Pages build (or other CI). The build script runs as a step BEFORE deploy; failure aborts deploy.
9. **MUST** be Pagefind-indexable — re-running `pagefind --site docs/` after prerender produces an index that returns hits for `NFR-PERF-01`, `RSK-09`, `TASK-AI-001` queries.
10. **MUST** support both build-time AND watch-mode runs:
- `node build/render-reference-pages.mjs` (one-shot).
- `node build/render-reference-pages.mjs --watch` (re-renders on JSON change; for local dev).
11. **MUST** be CI-gated by `docs-prerender-gate.yml`: on every PR touching `website/**`, the workflow runs the build + asserts the output matches the committed HTML (drift = build fail). Prevents the JSON-vs-HTML drift class.
12. **SHOULD** emit a build report at `website/build/last-build-report.json` with stats: number of NFRs/RSKs/tasks rendered, build duration, output bytes per page. Visible in CI logs for operator review.

---

## §2 — Why this design (rationale for humans)

Per research review §4 #1: "Tables render client-side; first paint shows empty scaffold with 'of NFRs match current filters' placeholder text." Per §5.4: "task catalog is empty. NFR catalog renders empty without JS. Risk register relies on client-side rendering."

The current architecture is a hard procurement and accessibility failure:

- An auditor running a Pagefind query for `NFR-PERF-01` gets zero hits even though the NFR exists in the page's JS array.
- A regulator reading the Risk Register page in screenshot mode sees an empty table.
- A crawler indexing the page extracts no data because everything is loaded by JS.

The fix is **prerendering at build time** so the static HTML and the dynamic UX coexist.

**Why JSON as source of truth (DEC-221)?** Currently the data arrays are embedded as inline JavaScript in the HTML. To prerender, the build script needs the data — extracting from inline JS is fragile. Moving to JSON files = one parse, two consumers (build script + Alpine), no ambiguity.

**Why deterministic build (DEC-220)?** Without determinism, two CI runs on the same input produce different outputs — diff drift, false-positive failures. Stable sort + no time-dependent values + ordered iteration ensures determinism.

**Why coexistence not replacement (§1 #4)?** Alpine UX (filter, sort, hover effects) works post-hydrate. Replacing entirely would lose those interactions. Prerender for first-paint + crawlers; Alpine for interactivity. CSS `x-cloak` ensures no flash.

**Why CI gate on drift (§1 #11)?** Operators editing JSON without re-running the build = JSON-HTML drift. The CI gate runs the build + diffs; drift fails the build, forcing the operator to commit the regenerated HTML.

**Why Pagefind re-index step (§1 #9)?** Static index on stale HTML returns wrong results. Re-running `pagefind --site docs/` after prerender ensures the index reflects the current data.

**Why <noscript> would fail (rejection rationale)?** Pagefind doesn't index `<noscript>` content. Browsers with JS see duplicate content. Better to prerender + hide on hydrate.

**Why per-card IDs/anchors (§1 #7)?** Cross-reference URLs (`#NFR-PERF-01`) work today only because Alpine generates them. Prerender preserves them at first paint, so deep links work for crawlers + screenshot tooling.

**Why watch mode (§1 #10)?** Local dev iteration needs fast feedback. Watch mode re-renders on JSON change; eliminates manual rebuild step.

---

## §3 — API contract

### Build script

```javascript
// website/build/render-reference-pages.mjs
import * as cheerio from 'cheerio';
import Handlebars from 'handlebars';
import { readFileSync, writeFileSync, watch } from 'fs';
import path from 'path';

const PAGES = [
  { html: 'docs/reference/nfr-catalog.html', data: 'build/data/nfrs.json',  template: 'build/templates/nfr-catalog.hbs', selector: '#nfr-list' },
  { html: 'docs/reference/risk-register.html', data: 'build/data/risks.json', template: 'build/templates/risk-register.hbs', selector: '#risk-list' },
  { html: 'docs/reference/task-catalog.html',  data: 'build/data/tasks.json',   template: 'build/templates/task-catalog.hbs', selector: '#task-list' },
];

async function renderPage({ html, data, template, selector }) {
  const json = JSON.parse(readFileSync(data, 'utf8'));
  // §1 #3: deterministic — sort by id ascending
  json.sort((a, b) => a.id.localeCompare(b.id));

  const tmpl = Handlebars.compile(readFileSync(template, 'utf8'));
  const rendered = tmpl({ items: json });

  const $ = cheerio.load(readFileSync(html, 'utf8'));
  $(selector).empty();   // remove existing prerendered (if any)
  $(selector).append(rendered);

  writeFileSync(html, $.html());
  return { rendered_count: json.length, output_bytes: $.html().length };
}

async function main() {
  const watchMode = process.argv.includes('--watch');
  const buildOnce = async () => {
    const report = { pages: [], started_at: new Date().toISOString(), duration_ms: 0 };
    const t0 = Date.now();
    for (const page of PAGES) {
      const stat = await renderPage(page);
      report.pages.push({ html: page.html, ...stat });
    }
    report.duration_ms = Date.now() - t0;
    writeFileSync('build/last-build-report.json', JSON.stringify(report, null, 2));
    console.log(`✅ Rendered ${report.pages.length} pages in ${report.duration_ms}ms`);
  };

  await buildOnce();

  if (watchMode) {
    console.log('Watching for changes...');
    for (const page of PAGES) {
      watch(page.data, async () => {
        console.log(`Change detected in ${page.data}; rebuilding...`);
        await buildOnce();
      });
    }
  }
}

main().catch(e => { console.error(e); process.exit(1); });
```

### Handlebars template (NFR card)

```handlebars
{{!-- website/build/templates/nfr-catalog.hbs --}}
<section data-prerendered="true" x-cloak>
  {{#each items}}
  <article class="bbg-card nfr-card" id="{{id}}" data-nfr-id="{{id}}" data-category="{{category}}">
    <header>
      <span class="nfr-id nfr-{{category}}">{{id}}</span>
      <span class="target-pill">{{target}}</span>
      <span class="phase-chip phase-{{phase}}">{{phase}}</span>
    </header>
    <h3>{{title}}</h3>
    <p class="description">{{description}}</p>
    <dl class="grid-info">
      <dt>Measurement</dt><dd>{{measurement}}</dd>
      <dt>Modules</dt><dd>{{#each modules}}<span class="module-chip">{{this}}</span>{{/each}}</dd>
      {{#if references}}
      <dt>Reference</dt><dd>{{#each references}}<a href="#{{this}}">{{this}}</a>{{#unless @last}} · {{/unless}}{{/each}}</dd>
      {{/if}}
    </dl>
  </article>
  {{/each}}
</section>
```

### Data shape (NFR JSON)

```json
[
  {
    "id": "NFR-PERF-01",
    "category": "perf",
    "title": "memory search p95 latency",
    "target": "p95 ≤ 250ms",
    "phase": "P0",
    "description": "memory search MUST return within 250ms p95...",
    "measurement": "1M chunks fixture / 1000 random queries",
    "modules": ["memory"],
    "references": ["DEC-070"]
  }
]
```

### Alpine init modification

```javascript
// website/docs/reference/_alpine-init.js (modified)
document.addEventListener('alpine:init', () => {
    Alpine.data('referenceCatalog', () => ({
        nfrs: [],
        hydrated: false,
        async init() {
            const response = await fetch('/build/data/nfrs.json');
            this.nfrs = await response.json();
            this.hydrated = true;
            // Hide prerendered after hydration — Alpine reactive takes over
            document.querySelectorAll('[data-prerendered]').forEach(el => el.style.display = 'none');
        },
        // ... existing filter/sort methods unchanged ...
    }));
});
```

### CSS

```css
[x-cloak] { display: block; }   /* prerendered visible at first paint */
.hydrated [x-cloak] { display: none; }   /* hidden once Alpine ready */
```

### CI workflow

```yaml
# .github/workflows/docs-prerender-gate.yml
name: Docs Prerender Gate
on:
  pull_request:
    paths:
      - 'website/**'
      - '.github/workflows/docs-prerender-gate.yml'

jobs:
  prerender-gate:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - run: cd website && npm ci
      - name: Run prerender
        run: cd website && node build/render-reference-pages.mjs
      - name: Assert no drift
        run: |
          if ! git diff --exit-code website/docs/reference/; then
            echo "❌ Prerender drift detected — commit the regenerated HTML"
            exit 1
          fi
      - name: Pagefind index test
        run: |
          cd website/docs && npx pagefind --site .
          npx pagefind search "NFR-PERF-01" | jq -e '.results | length > 0'
          npx pagefind search "RSK-09" | jq -e '.results | length > 0'
          npx pagefind search "TASK-AI-001" | jq -e '.results | length > 0'
```

---

## §4 — Acceptance criteria

1. **First paint shows data** — `curl https://docs.cyberos.world/reference/nfr-catalog.html | grep "NFR-PERF-01"` returns matching `<article>` line. No JS execution required.
2. **Pagefind indexes the data** — `pagefind search "NFR-PERF-01"` returns the nfr-catalog page. Same for `RSK-09` and `TASK-AI-001`.
3. **JS-on UX unchanged** — on a browser with Alpine loaded, page looks identical to current. No double-render flash on hydration.
4. **Filter/sort work post-hydration** — clicking a filter chip shows/hides the right cards.
5. **Determinism** — running the build twice on same JSON produces byte-identical HTML output (`git diff --exit-code` = clean).
6. **CI gate** — drift between JSON and HTML fails the PR.
7. **Per-card anchors work** — `#NFR-PERF-01` URL fragment scrolls to the card without JS.
8. **All metadata fields rendered** — verifying NFR cards have id/category/target/phase/modules/references; RSK cards have id/severity/likelihood/mitigation/owner/status; task cards have id/module/priority/status/slice/owner/depends_on/blocks.
9. **Cloudflare Pages build runs prerender** — `deploy.yml` includes the step; failure aborts deploy.
10. **Watch mode works** — `--watch` flag re-renders on JSON change.
11. **Build report emitted** — `last-build-report.json` contains per-page stats.
12. **Empty array doesn't crash** — JSON with empty array → empty `<section>` rendered (no crash).
13. **Special characters escaped** — NFR description with `<script>` is HTML-escaped (Handlebars default).
14. **Stable iteration order** — items sorted by `id` lexicographically.
15. **Hydration sets `hydrated` class on body** — for CSS to hide prerendered.

---

## §5 — Verification

```bash
# Manual smoke
cd cyberos/website
node build/render-reference-pages.mjs
git diff --exit-code docs/reference/    # determinism check (should be clean if regen of unchanged)

# Pagefind
cd docs && npx pagefind --site .
npx pagefind search "NFR-PERF-01" | jq '.results | length'   # expect > 0
npx pagefind search "RSK-09" | jq '.results | length'        # expect > 0
npx pagefind search "TASK-AI-001" | jq '.results | length'     # expect > 0

# First-paint check (no JS)
curl -s https://docs.cyberos.world/reference/nfr-catalog.html | grep -c 'data-nfr-id="NFR-'   # expect > 50

# Watch mode
node build/render-reference-pages.mjs --watch &
echo "test change" >> build/data/nfrs.json
# Expect rebuild log within 1s
kill %1
```

```javascript
// website/build/tests/render_test.mjs
import { execSync } from 'child_process';
import { readFileSync } from 'fs';
import { test } from 'node:test';
import assert from 'node:assert/strict';

test('render produces NFR cards', async () => {
    execSync('node build/render-reference-pages.mjs', { cwd: process.cwd() });
    const html = readFileSync('docs/reference/nfr-catalog.html', 'utf8');
    assert.ok(html.includes('data-nfr-id="NFR-PERF-01"'), 'expected NFR-PERF-01 in output');
    assert.ok(html.includes('<section data-prerendered="true"'), 'expected prerendered section');
});

test('render produces task cards with all metadata', async () => {
    execSync('node build/render-reference-pages.mjs', { cwd: process.cwd() });
    const html = readFileSync('docs/reference/task-catalog.html', 'utf8');
    assert.ok(html.includes('TASK-AI-001'));
    assert.ok(html.includes('depends_on'));
});
```

```javascript
// website/build/tests/determinism_test.mjs
import { execSync } from 'child_process';
import { readFileSync } from 'fs';
import { test } from 'node:test';
import assert from 'node:assert/strict';

test('two consecutive builds produce identical HTML', async () => {
    execSync('node build/render-reference-pages.mjs');
    const html1 = readFileSync('docs/reference/nfr-catalog.html');
    execSync('node build/render-reference-pages.mjs');
    const html2 = readFileSync('docs/reference/nfr-catalog.html');
    assert.deepEqual(html1, html2, 'HTML output not deterministic');
});
```

```javascript
// website/build/tests/pagefind_index_test.mjs
import { execSync } from 'child_process';
import { test } from 'node:test';
import assert from 'node:assert/strict';

test('pagefind indexes prerendered NFR-IDs', async () => {
    execSync('node build/render-reference-pages.mjs');
    execSync('npx pagefind --site docs/');
    const result = JSON.parse(execSync('npx pagefind search "NFR-PERF-01"').toString());
    assert.ok(result.results.length > 0, 'pagefind did not index NFR-PERF-01');
});
```

---

## §6 — Implementation skeleton

See §3.

```javascript
// website/build/data-extract.mjs (one-time migration: extract from existing inline JS)
import * as cheerio from 'cheerio';
import { readFileSync, writeFileSync } from 'fs';
import vm from 'vm';

const FILES = [
  { html: 'docs/reference/nfr-catalog.html', varName: 'NFR_DATA', out: 'build/data/nfrs.json' },
  { html: 'docs/reference/risk-register.html', varName: 'RISKS', out: 'build/data/risks.json' },
  { html: 'docs/reference/task-catalog.html', varName: 'TASK_CATALOG', out: 'build/data/tasks.json' },
];

for (const { html, varName, out } of FILES) {
  const $ = cheerio.load(readFileSync(html, 'utf8'));
  const script = $('script').filter((_, el) => $(el).text().includes(`const ${varName}`)).text();
  const ctx = { window: {}, document: {} };
  vm.createContext(ctx);
  vm.runInContext(script + `; this.${varName} = ${varName};`, ctx);
  writeFileSync(out, JSON.stringify(ctx[varName], null, 2));
  console.log(`✅ Extracted ${varName} → ${out}`);
}
```

---

## §7 — Dependencies

- Cloudflare Pages build env has Node 20 (or any CI runner with Node).
- npm packages: `cheerio@1`, `handlebars@4`.
- Pagefind binary (or @pagefind/default-ui as npm dep).
- One-time migration step: extract inline JS arrays to JSON via `data-extract.mjs`.

---

## §8 — Example payloads

### Build report

```json
{
  "started_at": "2026-05-15T14:00:00Z",
  "duration_ms": 245,
  "pages": [
    { "html": "docs/reference/nfr-catalog.html", "rendered_count": 76, "output_bytes": 89234 },
    { "html": "docs/reference/risk-register.html", "rendered_count": 22, "output_bytes": 31872 },
    { "html": "docs/reference/task-catalog.html", "rendered_count": 64, "output_bytes": 124398 }
  ]
}
```

### Pagefind search response

```json
{
  "results": [
    {
      "id": "...",
      "data": "/reference/nfr-catalog.html#NFR-PERF-01",
      "title": "memory search p95 latency",
      "excerpt": "memory search MUST return within 250ms p95..."
    }
  ]
}
```

### CI drift failure

```text
❌ Prerender drift detected — commit the regenerated HTML
diff --git a/website/docs/reference/nfr-catalog.html b/website/docs/reference/nfr-catalog.html
+ <article class="bbg-card nfr-card" id="NFR-NEW-01">...</article>
```

---

## §9 — Open questions

All resolved. Deferred:
- Per-language prerender (Vietnamese reference pages) — slice 3+ when i18n lands.
- Differential rebuild (only re-render pages whose JSON changed) — slice 4+ optimisation.
- HTMX progressive enhancement (replace Alpine entirely) — out of scope; Alpine works.

---

## §10 — Failure modes inventory

| Failure | Outcome | Recovery |
|---|---|---|
| JSON parse error | Build fails with line number | Operator fixes JSON |
| Handlebars template syntax error | Build fails | Operator fixes template |
| Cheerio fails to parse HTML | Build fails | Operator fixes HTML scaffold |
| JSON-HTML drift in PR | CI fails (`git diff --exit-code`) | Operator runs build + commits |
| Pagefind index returns 0 hits | Test fails | Investigate prerender output |
| First paint still empty (post-deploy) | Manual smoke check fails | Check deploy step ran prerender |
| Double-render flash on hydration | Manual UX check | Add `[x-cloak]` to template + CSS |
| Filter/sort broken post-hydrate | Manual UX check | Investigate Alpine init order |
| Special characters not escaped | XSS in description field | Handlebars default escapes; verify |
| Determinism violated (sort order) | determinism_test fails | Add explicit `localeCompare` sort |
| Watch mode doesn't trigger | local dev annoyance | Check `fs.watch` permissions |
| Build report missing | CI artefact missing | Always write before exit |
| Cloudflare Pages build doesn't run prerender | First paint empty post-deploy | Update deploy.yml |
| Per-card anchor missing | deep-link scroll fails | Verify `id="<card-id>"` attribute |
| Empty JSON array | empty section rendered | By design |
| Migration extracts wrong data | data-extract.mjs bug | Manually verify JSON post-extraction |
| Cheerio version mismatch | Build fails on different runner | Pin in package.json |

---

## §11 — Notes

- This task closes UX defects #1 and #2 from `docs/archive/2026-05-14/RESEARCH_REVIEW.md §4`. Load-bearing for procurement-story for auditors.
- Deferred to P0 polish slice; estimated land date late P0.
- JSON-as-source-of-truth + watch mode + CI gate together prevent JSON-HTML drift.
- Alpine reactive coexists; UX preserved.
- Pagefind re-indexes on every deploy.
- Per-card anchors enable deep linking from external sources (e.g., regulator citing `https://docs.cyberos.world/reference/nfr-catalog.html#NFR-PERF-01`).
- Determinism via stable sort + no time-dependent values catches accidental non-determinism in CI.
- Build report enables operator review of "did all 76 NFRs render?" without inspecting HTML.
- Watch mode improves local dev iteration; not run in CI.

---

*End of TASK-DOCS-001. Status: planned (10/10 target).*

---

**Supersession record (2026-07-12, conflict-scan doctrine: newest wins).** The live intent of this task shipped through the TASK-DOCS-002/005/006 pipeline: data extraction to JSON (tools/docs-site/data-extract.mjs -> data/tasks.json, nfrs), prerendered `<section data-prerendered="true">` catalog cards (render-task-catalog / render-nfr-catalog carry `TASK-DOCS-001 §1 #2` citations), deterministic builds, stable per-card anchors, and last-build-report.json. The remaining clauses are obsolete by later approved doctrine: #1's website/build paths and #8's Cloudflare Pages (replaced by tools/docs-site + VPS deploy), #4/#5 Alpine hydration (vanilla JS since the rebuild), #11's committed-HTML drift gate (FORBIDDEN by TASK-DOCS-002: generated output is never committed), #9 Pagefind and #10 watch mode (never built). Client-side search of the docs site is genuinely undelivered and is queued as a fresh-task candidate for the next intake batch rather than resurrecting this spec's mechanics. Status: closed (superseded), not done - several clauses as written are permanently false.
