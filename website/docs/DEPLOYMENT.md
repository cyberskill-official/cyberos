# Deploying the CyberOS docs site

Three recommended paths, in order of friction. Cloudflare Pages is the fastest; GitHub Pages is the simplest; self-hosted is the most flexible.

## Option 1 — Cloudflare Pages (recommended, ~10 minutes)

### Why Cloudflare

- Free tier: unlimited bandwidth, 500 builds/month
- Edge-cached globally (CyberOS docs load fast in Vietnam, EU, US, anywhere)
- Wrangler CLI integrates with the rest of your workflow
- Custom domain (`docs.cyberskill.world`) takes 5 minutes via DNS
- Pagefind builds at deploy time via the build command

### Steps

1. Install Wrangler (Cloudflare's CLI):

```bash
npm install -g wrangler
wrangler login
```

This opens a browser, you authorize Wrangler against your Cloudflare account. One-time setup.

2. Create the Pages project:

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos/website/docs

# First deploy — uses upload mode (no Git connection yet)
wrangler pages deploy . --project-name cyberos-docs
```

Wrangler creates a project and uploads the entire `docs/` folder. It returns a URL like `https://cyberos-docs.pages.dev`.

3. Connect the project to your Git repo (for auto-deploy on push):

In the Cloudflare dashboard → Pages → `cyberos-docs` → Settings → Build & deployments:
- Connect to GitHub (one-time OAuth)
- Pick your `cyberskill/cyberos` repo
- Production branch: `main`
- Build command: `cd website/docs && npx pagefind --site .`
- Output directory: `website/docs`

Save. Now every push to `main` triggers a build + deploy.

4. Add the custom domain:

In the Cloudflare dashboard → Pages → `cyberos-docs` → Custom domains → Add:
- `docs.cyberskill.world`
- Cloudflare auto-creates the CNAME record (since cyberskill.world is presumably already on Cloudflare DNS)
- HTTPS is auto-provisioned via Cloudflare Universal SSL

5. Verify:

```bash
curl -sI https://docs.cyberskill.world | head -5
```

Should return `HTTP/2 200`.

### Cloudflare config files (optional but recommended)

Create `cyberos/website/docs/_redirects` for any URL aliases:

```
# /docs.cyberskill.world rewrites
/  /index.html  200
/modules  /index.html#catalog  302
```

And `cyberos/website/docs/_headers` for cache + security headers:

```
/*
  X-Frame-Options: SAMEORIGIN
  X-Content-Type-Options: nosniff
  Referrer-Policy: strict-origin-when-cross-origin

/pagefind/*
  Cache-Control: public, max-age=86400, immutable

/assets/*
  Cache-Control: public, max-age=86400
```

## Option 2 — GitHub Pages (simplest, ~5 minutes)

If you'd rather skip Cloudflare:

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos

# Create gh-pages branch from docs/
git checkout --orphan gh-pages
git rm -rf .
cp -r website/docs/* .
git add -A
git commit -m "feat: deploy docs"
git push origin gh-pages

# Then in repo Settings → Pages → Source: gh-pages branch, root folder
```

Custom domain on GitHub Pages: Settings → Pages → Custom domain → `docs.cyberskill.world`. Adds a `CNAME` file to the branch.

Downsides vs Cloudflare:
- GitHub Pages has a 100 GB/month bandwidth soft limit
- No build hooks (you can't run `npx pagefind` at deploy time without Actions)
- DNS propagation takes longer

If using GitHub Pages, run `npx pagefind --site .` locally before each `git push origin gh-pages`. Commit the `pagefind/` folder (un-gitignore it).

## Option 3 — Self-hosted (most flexible)

Any static-file webserver works. Examples:

### nginx on a VPS

```nginx
server {
  listen 443 ssl http2;
  server_name docs.cyberskill.world;

  ssl_certificate     /etc/letsencrypt/live/docs.cyberskill.world/fullchain.pem;
  ssl_certificate_key /etc/letsencrypt/live/docs.cyberskill.world/privkey.pem;

  root /var/www/cyberos-docs;
  index index.html;

  location / {
    try_files $uri $uri.html $uri/ =404;
  }

  location /pagefind/ {
    expires 1d;
    add_header Cache-Control "public, immutable";
  }
}
```

### Caddy (simpler — auto HTTPS)

```caddy
docs.cyberskill.world {
  root * /var/www/cyberos-docs
  file_server
  encode gzip
}
```

Rebuild + sync:

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos/website/docs
npx pagefind --site .
rsync -avz --delete . user@server:/var/www/cyberos-docs/
```

## Deployment checklist

Before deploying publicly:

- [ ] All Mermaid diagrams render in latest Chrome + Safari + Firefox
- [ ] Pagefind search returns results for: `CCCD`, `BRAIN`, `MMR`, `Umber`, `vn-vat-invoice`
- [ ] Sticky nav stays translucent on scroll (Liquid Glass working)
- [ ] Dropdown only shows on hover/focus (Bug 1 verified fixed)
- [ ] Catalog headline doesn't overlap paragraph (Bug 3 verified fixed)
- [ ] Print preview produces clean PDFs (glass collapses to solid)
- [ ] Mobile viewport (375px wide) renders cleanly
- [ ] All 22 module pages return 200
- [ ] All cross-links resolve (no 404s)
- [ ] `prefers-reduced-motion` users get no parallax
- [ ] `prefers-reduced-transparency` users get solid surfaces
- [ ] Dark mode toggle works
- [ ] No console errors in browser devtools

## Rebuild after content changes

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos/website/docs

# Rebuild Pagefind index
npx pagefind --site .

# Commit + push (auto-deploys on Cloudflare Pages / GitHub Pages)
cd ..
git add -A
git commit -m "docs: update <section>"
git push
```

## Cost projection

| Hosting | Year 1 cost | Bandwidth |
|---|---|---|
| Cloudflare Pages free | $0 | Unlimited |
| GitHub Pages | $0 | 100 GB/mo soft |
| Self-hosted ($5/mo VPS) | $60 | Unlimited |
| Self-hosted ($20/mo CDN-fronted) | $240 | Unlimited |

For traffic volumes a Vietnamese consultancy would attract in year 1 (~10,000 monthly visitors at peak), Cloudflare Pages free covers comfortably.

## Custom domain DNS

If you don't already have `cyberskill.world` configured:

1. Buy via Cloudflare Registrar (cheapest TLD pricing, includes free DNS)
2. Or transfer existing domain to Cloudflare (free, takes 1-2 days)
3. Create CNAME: `docs` → `cyberos-docs.pages.dev` (Cloudflare auto-flatten handles the apex if needed)
4. Wait 5-30 minutes for propagation
5. Verify: `dig docs.cyberskill.world CNAME`

## Post-deploy steps

After the site is live:

1. **Submit to search engines** — Google Search Console + Bing Webmaster
2. **Add to your LinkedIn** — `Docs · docs.cyberskill.world` as a project link
3. **Add to README** — top of `cyberos/README.md` and `public-skills/README.md`
4. **Announce in VN dev communities** — Daynhauhoc, Toidicodedao, Vietnamese tech LinkedIn groups (drafts at `public-skills/announcements/`)

## See also

- `SESSION_BOOTSTRAP.md` — bootstrap prompt for a new Claude session on this repo
- `strategy/CYBEROS_STRATEGY.md` — strategic posture + 12-month arc
- `BUILD.md` — local build instructions (Pagefind rebuild, preview server)
