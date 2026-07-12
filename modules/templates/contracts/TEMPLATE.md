# template@1 - the presentation contract (FR-TPL-001)

A template is an HTML file whose root element carries `data-template-id="<name>@1"` and whose
variable regions are named slots. Consumers (node builders or doc-driven agents) render by plain
string substitution - no template engine.

## Slot grammar

| form | meaning |
|---|---|
| `{{slot:<name>}}` | text slot - substitute with HTML-ESCAPED text |
| `{{slot:<name>:html}}` | html slot - substitute with pre-rendered, builder-owned HTML |

Escape set for text slots (exactly these, in this order): `&` -> `&amp;`, `<` -> `&lt;`,
`>` -> `&gt;`, `"` -> `&quot;`.

Unfilled slots MUST be substituted with the empty string by the consumer (a shipped page never
contains a literal `{{slot:`). `:html` slots MUST receive only builder-generated markup - never
raw operator/user input (injection boundary lives at the renderer, matching md.mjs escaping).

## Self-containment rule

Rendered output works from file://: styles are inlined by the consumer (tokens.css + glass.css +
shell styles concatenated into a `<style>` block or shipped as RELATIVE files beside the page);
the only permitted external references are the page's own relative assets. No CDN, no Google
Fonts fetch - `--cs-font-family-ui` falls back through the system stack by design.

## Shells (this module)

| id | file | required slots |
|---|---|---|
| deliverable@1 | html/deliverable.html | title, kind, id, status, meta:html, body:html, footer |
| status-hub@1 | html/status-hub.html | title, deck:html, tab_roadmap:html, tab_backlog:html, tab_changelog:html, footer |
| catalog@1 | html/catalog.html | title, facets:html, cards:html, footer |

Consumers inline `cds/tokens.css` (and `cds/glass.css` when using .cs-surface-*) ahead of the
shell's own `<style>`. Style rule: shells reference `--cs-*` variables only.
