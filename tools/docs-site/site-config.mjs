// tools/docs-site/site-config.mjs
// Single source of truth for the docs site's public base URL. Every render script that emits
// a <link rel="canonical"> (or any other absolute self-reference) imports SITE_BASE_URL from
// here instead of hardcoding it, so a future domain/path move is a one-line change instead of
// a grep-and-replace across the render scripts (that drift is exactly what happened before
// this file existed: render-changelog.mjs, render-module-changelog.mjs, and
// render-nfr-catalog.mjs had the old cyberos-wiki.cyberskill.world domain, while
// render-task-catalog.mjs independently had docs.cyberos.world -- three render scripts, two
// different stale domains, neither matching the site's actual current home).
export const SITE_BASE_URL = 'https://os.cyberskill.world/docs';
