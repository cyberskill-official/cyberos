# CyberOS website surfaces

One surface lives here:

- `docs/` — comprehensive multi-page documentation site (32 pages, 22 module pages, 226 Mermaid diagrams, Pagefind site-wide search, Liquid Glass via the sibling design-system). Open `docs/index.html` in a browser, or serve via `python3 -m http.server` from inside `docs/`.

When ready to deploy:
- `docs/` → `docs.cyberskill.world` via Cloudflare Pages (see `docs/DEPLOYMENT.md`)

## Sibling: landing page

The `cyberskill.world` landing page is a **separate project** with its own git repo at `/Users/stephencheng/Projects/CyberSkill/landing-page/`. Marketing surface, separate release cadence, separate audit cycle.

When deploying:
- `../../landing-page/` → `cyberskill.world` via Cloudflare Pages

## Sibling: design system

Liquid Glass + Umber/Ochre tokens come from the sibling design-system project at `/Users/stephencheng/Projects/CyberSkill/design-system/`. The docs site under `docs/` consumes design-system tokens via `docs/assets/tokens.css`, which is hand-maintained against the design-system's `DESIGN.md` Part 2 + Part 21. When the design system releases a new version, sync the tokens manually (or via a build script — pending).

See `../strategy/CYBEROS_STRATEGY.md` for the broader distribution plan.
