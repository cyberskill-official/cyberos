# Building the CyberOS docs site

The site is a folder of plain HTML — no compile step for the pages themselves. The only build artefact is the **Pagefind search index**.

## Pagefind search index

[Pagefind](https://pagefind.app/) is a Rust-based static-site search tool. It crawls the HTML at build time and produces a small index plus a JS UI. Rebuild whenever content changes:

```bash
cd cyberos/website/docs
npx pagefind --site .
```

This drops the index at `pagefind/`. The directory is gitignored — regenerate it at deploy time.

### What gets indexed

- Every `.html` file under `docs/` (32 pages today).
- Content inside `<body>`, with nav / sticky chrome / footer excluded via `pagefind.yml`.
- Two filters: `category` (Module / Architecture / Reference / Overview) and `phase` (P0 / P1 / P2 / P3 / P4, modules only).

### Config

See `pagefind.yml` at the docs root. Most knobs (root selector, exclude selectors, output dir) live there.

## Local preview

```bash
cd cyberos/website/docs
python3 -m http.server 8765
# → http://localhost:8765
```

Pagefind needs the index to exist before the UI works — run `npx pagefind --site .` once, then start the server.

## Deploy (Cloudflare Pages, Netlify, Vercel)

**Build command:**

```bash
npx pagefind --site .
```

**Output / publish dir:** `cyberos/website/docs/` (the entire docs folder is the static site).

**Node version:** any recent LTS — Pagefind ships a prebuilt Rust binary via npm, so Node only needs to launch `npx`.

No backend, no environment variables, no DB. Just static HTML + a 1.7 MB index.

## Tagging new pages

When you add a page, give its `<body>` a category filter so it shows up in Pagefind's faceted search:

```html
<body data-pagefind-filter="category:Module">
<span hidden data-pagefind-filter="phase:P0"></span>
<!-- ...page content... -->
</body>
```

Categories in use: `Module`, `Architecture`, `Reference`, `Overview`. Phases (modules only): `P0`–`P4`.
