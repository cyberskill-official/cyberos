# Deploying the CyberOS docs site

**Current state (2026-05-18):** deployed to **https://cyberos-wiki.cyberskill.world** via **Vercel**, manual deploy from operator's machine.

## Redeploy flow (operator-driven)

The operator (Stephen Cheng) holds Vercel credentials and runs the deploy by hand:

```bash
cd ~/Projects/CyberSkill/cyberos
vercel deploy --prod
```

That's it. The site is pure static (33 HTML + assets + pre-built Pagefind index); no build step.

## Why operator-controlled instead of CI?

- The repo intentionally does **not** carry `vercel.json` or `.vercelignore` — keeps deploys an explicit operator action rather than something that ships automatically on every push.
- If you want Git-integration (auto-deploy on push to `main`), set it up via the Vercel dashboard: Project → Settings → Git → Connect → pick the GitHub repo → production branch `main`. Build command empty, output directory `website/docs`.

## DNS

`cyberos-wiki.cyberskill.world` resolves via a CNAME to Vercel's edge. DNS is managed at whichever provider hosts `cyberskill.world` (Cloudflare / Namecheap / GoDaddy / etc.). The CNAME target is what `vercel domains inspect cyberos-wiki.cyberskill.world` prints.

## Local preview

```bash
cd ~/Projects/CyberSkill/cyberos/website/docs
python3 -m http.server 8000
# open http://localhost:8000
```

## Search index (Pagefind)

The `pagefind/` directory under `website/docs/` is **committed** — Pagefind is pre-built. If you change page content meaningfully, regenerate:

```bash
cd ~/Projects/CyberSkill/cyberos/website/docs
npx pagefind --site .
git add pagefind/ && git commit -m "chore: refresh pagefind index"
```

Then redeploy.

## Sibling sites (not deployed from this repo)

- `cyberskill.world` landing page — separate repo at `../../landing-page/`
- design-system token source — separate repo at `../../design-system/`

Each sibling has its own deploy story.
