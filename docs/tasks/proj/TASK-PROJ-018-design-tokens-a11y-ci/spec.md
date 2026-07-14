---
id: TASK-PROJ-018
title: "Liquid-Glass design tokens (tokens.proj.css) + axe-core CI accessibility gate + Storybook visual regression"
module: PROJ
priority: MUST
status: done
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PROJ-014, TASK-PROJ-015, TASK-PROJ-016, TASK-PROJ-017]
depends_on: [TASK-PROJ-014]
blocks: []

source_pages:
  - website/docs/modules/proj.html#design-tokens
  - design-system/CLAUDE.md
source_decisions:
  - DEC-390 (Liquid-Glass tokens — translucent backdrop + sharp ink + subtle elevation; matches brand)
  - DEC-391 (a11y CI uses axe-core via Playwright; fails build on critical/serious violations)
  - DEC-392 (Storybook visual regression via Chromatic; per-component baseline snapshots)

language: css + typescript 5.4
service: cyberos/web/proj-client/
new_files:
  - web/proj-client/src/styles/tokens.proj.css
  - web/proj-client/.storybook/main.ts
  - web/proj-client/.storybook/preview.tsx
  - web/proj-client/src/components/*.stories.tsx           # one story per component
  - web/proj-client/playwright/a11y.spec.ts
  - .github/workflows/proj-a11y-gate.yml
  - .github/workflows/proj-storybook-chromatic.yml
modified_files:
  - web/proj-client/src/styles/global.css                  # import tokens.proj.css
  - web/proj-client/package.json                           # axe-core/playwright, @storybook/react-vite, chromatic
allowed_tools:
  - file_read: web/proj-client/**, design-system/**
  - file_write: web/proj-client/src/**, web/proj-client/.storybook/**, web/proj-client/playwright/**, .github/workflows/**
  - bash: cd web/proj-client && npx playwright test a11y
  - bash: cd web/proj-client && npx storybook build
disallowed_tools:
  - hardcode colors / spacing / radii in component CSS (per DEC-390 — tokens only)
  - merge PR with critical/serious axe-core violations (per DEC-391)

effort_hours: 8
subtasks:
  - "1.0h: tokens.proj.css — full token catalog (color/typography/spacing/radius/shadow/motion)"
  - "0.5h: dark theme variant via [data-theme=dark] body attribute"
  - "1.0h: stylelint plugin no-hardcoded-values + CI gate"
  - "0.5h: Storybook config (.storybook/main.ts + preview.tsx with theme toolbar)"
  - "1.5h: 8 component stories (Board, Column, IssueCard, Timeline bar, Gantt arrow, BriefModal, StatusPicker, MetaSidebar) × 3 variants each"
  - "1.0h: Chromatic baseline + per-PR snapshot config"
  - "1.0h: playwright/a11y.spec.ts — axe-core per story; CI gate"
  - "0.5h: contrast_test.ts — 30+ token-pair contrast assertions"
  - "0.5h: Liquid-Glass shader CSS + fallback solid"
  - "0.5h: focus-ring tokens + global :focus-visible rule"
risk_if_skipped: "Without token enforcement, components drift to hardcoded hex codes — tokens become decorative; runtime theming breaks. Without Chromatic, visual regressions ship unnoticed (8 top-level components × ~20 PRs/quarter = high drift risk). Without axe-core CI gate, a11y degrades silently (WCAG AA compliance is a procurement gate). Without contrast tests, designer-chosen pairs fail at runtime on dark theme. Without focus-ring tokens, kbd users lose visible focus. Without reduced-motion honouring, vestibular-disorder users get unwanted animations."
---

## §1 — Description (BCP-14 normative)

The PROJ design-tokens + a11y CI layer **MUST** provide a single source of truth for visual design and enforce accessibility quality. The contract:

1. **MUST** publish `tokens.proj.css` with the following token categories:
    - **Color**: brand-primary, brand-secondary, status-* (one per IssueStatus), priority-* (low/med/high/urgent), surface-* (panel/elevated/translucent), border-*, text-* (primary/secondary/disabled).
    - **Typography**: font-family-sans, font-family-mono, font-size-{xs|sm|base|lg|xl|2xl|3xl}, font-weight-{normal|medium|semibold|bold}, line-height-{tight|normal|relaxed}.
    - **Spacing**: space-{0|1|2|3|4|5|6|8|10|12|16|24}, on 4px grid.
    - **Radius**: radius-{none|sm|md|lg|full}.
    - **Shadow**: shadow-{sm|md|lg} — translucent with Liquid-Glass blur backdrops.
    - **Motion**: duration-{fast|normal|slow}, easing-{in|out|in-out}.
2. **MUST** use CSS custom properties for runtime theming: light vs dark via `[data-theme="dark"]` body attribute.
3. **MUST** enforce token-only usage via stylelint plugin `stylelint-no-hardcoded-values` (custom rule); CI fails on PRs introducing raw colors / spacings.
4. **MUST** publish each top-level component as a Storybook story:
    - At minimum: Board, Column, IssueCard, Timeline bar, Gantt arrow, BriefModal, StatusPicker, MetaSidebar.
    - Each story exports the component in 3 variants minimum: default, focused, error.
5. **MUST** run Chromatic visual regression on every PR; new snapshots auto-baseline; diffs require human review.
6. **MUST** run axe-core via Playwright on every PR (CI workflow `proj-a11y-gate.yml`):
    - Visits each Storybook story.
    - Runs `@axe-core/playwright` injection.
    - Fails build on `critical` or `serious` violations.
    - Reports warning on `moderate` (logged but doesn't fail).
7. **MUST** define WCAG AA contrast minimums:
    - Body text: 4.5:1.
    - Large text (18pt+ or 14pt+ bold): 3:1.
    - Non-text UI (icons, focus rings): 3:1.
    - All token combinations validated by a `contrast_test.ts` Jest run.
8. **MUST** provide keyboard-focus-ring tokens (`outline-focus`, `outline-focus-width`) used by every interactive component; never `:focus { outline: none }` without `:focus-visible` alternative.
9. **MUST** ship a Liquid-Glass shader/blur primitive:
    - `backdrop-filter: blur(20px) saturate(180%);` on translucent surfaces.
    - Fallback solid color when backdrop-filter unsupported.
10. **MUST** document tokens in `design-system` workspace cross-reference (per CLAUDE.md `@AGENTS.md`) for shared use across modules.
11. **SHOULD** export tokens.proj.json for non-CSS consumers (mobile native renderer slice 4+; Figma sync).
12. **MUST** support `prefers-color-scheme: dark` media query: when user OS prefers dark, default to dark theme without manual toggle. Explicit `[data-theme]` attribute overrides OS preference.
13. **MUST** publish a token diff report on every PR touching `tokens.proj.css`: comparison vs main branch + impact analysis (which components use the changed tokens). Posted as PR comment.
14. **MUST** include `prefers-contrast: more` support: when user prefers high contrast, surface translucency reduces (more opaque); border colors strengthen.
15. **MUST** support semantic tokens layered on raw tokens: e.g. `--color-action-primary: var(--color-brand-primary)`. Components reference semantic; designers swap semantic mappings without touching components.
16. **MUST** include a "design-system audit" CLI: `cyberos design-system audit web/proj-client/` walks components + reports token-coverage % + lists deviations.
17. **MUST** support tokens.proj.css versioning: major version bump on breaking token rename; CHANGELOG.md in styles/ folder.
18. **MUST** pass axe-core with `axe-core/runOnly: ['wcag2a', 'wcag2aa', 'wcag21aa', 'best-practice']` rule sets; no `wcag2aaa` (too strict for app context).
19. **MUST** include token-usage telemetry in build: webpack/vite plugin reports which tokens are used; unused tokens flagged for removal.
20. **MUST** support per-component CSS-in-JS escape hatch via `data-component-style` attribute for legitimate edge cases; usage of escape hatch logged + audited per release.
21. **MUST** include keyboard-navigation visual cue: a discoverable "Press Tab to navigate" hint in empty-state UIs (e.g. empty kanban column).
22. **MUST** ship a "design tokens cheatsheet" document (Markdown in design-system/) listing all tokens + usage examples + accessibility notes.

---

## §2 — Why this design (rationale for humans)

**Why CSS custom properties (DEC-390)?** Runtime theming without rebuild. `[data-theme="dark"]` swaps all colors instantly; users prefer "follow OS" toggles that need no reload.

**Why stylelint enforcement (§1 #3)?** Without it, components drift into raw hex codes and pixel values; tokens become decorative. Lint-time enforcement makes drift mechanically impossible to merge.

**Why Storybook + Chromatic (DEC-392)?** Visual regression is the only way to catch "looked fine in dev, broke in prod" issues across the 8 top-level components. Per-story baseline = catches at component level, not page level.

**Why axe-core via Playwright (§1 #6, DEC-391)?** axe-core is the industry-standard a11y engine; Playwright is our existing e2e harness. Combining them = no new tooling.

**Why contrast tests (§1 #7)?** WCAG AA mandates ratios; without code-enforced check, designers pick visually-pleasing-but-failing pairs. `contrast_test.ts` runs the formula for every token combination used in components.

**Why focus-ring tokens (§1 #8)?** Default browser focus ring is inconsistent and often invisible on dark themes. Branded focus ring = visible, on-brand, kbd-user-friendly.

**Why Liquid-Glass (DEC-390)?** Brand identity; matches CyberSkill marketing aesthetic. Translucent surfaces evoke modernity. Fallback to solid for older browsers (gracefully degrades).

**Why design-system cross-ref (§1 #10)?** The user's workspace includes a `design-system` folder (per CLAUDE.md). Shared tokens across modules (AUTH login pages, AI Gateway dashboards, PROJ views) = consistent product.

**Why OS color-scheme respect (§1 #12)?** Users have system-wide preference; ignoring forces them to set per-app. Following OS = invisible win.

**Why token diff report (§1 #13)?** Token changes ripple across components silently; PR diff comment surfaces impact for reviewer.

**Why prefers-contrast support (§1 #14)?** Vision-impaired users explicitly set this preference; ignoring fails WCAG 2.1 AA.

**Why semantic tokens (§1 #15)?** Components reference "action-primary"; designers swap the mapping without touching component code. Stability + flexibility.

**Why design-system audit CLI (§1 #16)?** Operator wants periodic health check; CLI gives quantified coverage metric.

**Why versioning (§1 #17)?** Token renames break downstream; major version + CHANGELOG = clear migration path.

**Why specific axe rules (§1 #18)?** AAA is too strict for app context (e.g. 7:1 contrast on body text); AA + best-practice is the industry baseline.

**Why token usage telemetry (§1 #19)?** Unused tokens bloat the file; usage stats prompt cleanup.

**Why CSS-in-JS escape hatch (§1 #20)?** Edge cases inevitable; escape hatch with audit trail respects pragmatism without losing accountability.

**Why kbd-nav hint (§1 #21)?** Discoverable kbd shortcuts; without hints, new users miss the keyboard-first design.

**Why cheatsheet doc (§1 #22)?** Designers + engineers need quick reference; markdown in design-system/ is the canonical lookup.

---

## §3 — API contract

### Tokens

```css
/* web/proj-client/src/styles/tokens.proj.css */

:root {
  /* — Color: Brand — */
  --color-brand-primary:   #1e40af;
  --color-brand-secondary: #6366f1;

  /* — Color: Status (per TASK-PROJ-004 IssueStatus) — */
  --color-status-backlog:     #94a3b8;
  --color-status-todo:        #60a5fa;
  --color-status-in-progress: #fbbf24;
  --color-status-in-review:   #a78bfa;
  --color-status-done:        #34d399;
  --color-status-cancelled:   #f87171;

  /* — Color: Priority — */
  --color-priority-low:    #94a3b8;
  --color-priority-medium: #60a5fa;
  --color-priority-high:   #fb923c;
  --color-priority-urgent: #ef4444;

  /* — Color: Surface (Liquid-Glass) — */
  --color-surface-panel:        rgba(255, 255, 255, 0.7);
  --color-surface-elevated:     rgba(255, 255, 255, 0.9);
  --color-surface-translucent:  rgba(255, 255, 255, 0.5);
  --color-surface-modal:        rgba(255, 255, 255, 0.95);

  /* — Color: Border & Text — */
  --color-border:            rgba(15, 23, 42, 0.1);
  --color-border-strong:     rgba(15, 23, 42, 0.2);
  --color-text-primary:      #0f172a;
  --color-text-secondary:    #475569;
  --color-text-disabled:     #94a3b8;
  --color-text-on-brand:     #ffffff;

  /* — Color: Critical-path (TASK-PROJ-016) — */
  --color-critical-path-border: #f59e0b;

  /* — Typography — */
  --font-family-sans: 'Inter', system-ui, -apple-system, sans-serif;
  --font-family-mono: 'JetMemorys Mono', ui-monospace, monospace;
  --font-size-xs: 0.75rem;     --font-size-sm: 0.875rem;
  --font-size-base: 1rem;      --font-size-lg: 1.125rem;
  --font-size-xl: 1.25rem;     --font-size-2xl: 1.5rem;
  --font-size-3xl: 1.875rem;
  --font-weight-normal: 400;   --font-weight-medium: 500;
  --font-weight-semibold: 600; --font-weight-bold: 700;
  --line-height-tight: 1.2;    --line-height-normal: 1.5;
  --line-height-relaxed: 1.75;

  /* — Spacing (4px grid) — */
  --space-0: 0;       --space-1: 0.25rem;
  --space-2: 0.5rem;  --space-3: 0.75rem;
  --space-4: 1rem;    --space-5: 1.25rem;
  --space-6: 1.5rem;  --space-8: 2rem;
  --space-10: 2.5rem; --space-12: 3rem;
  --space-16: 4rem;   --space-24: 6rem;

  /* — Radius — */
  --radius-none: 0;        --radius-sm: 0.25rem;
  --radius-md: 0.5rem;     --radius-lg: 1rem;
  --radius-full: 9999px;

  /* — Shadow (Liquid-Glass) — */
  --shadow-sm:  0 1px 3px rgba(15, 23, 42, 0.08);
  --shadow-md:  0 4px 12px rgba(15, 23, 42, 0.12);
  --shadow-lg:  0 12px 40px rgba(15, 23, 42, 0.20);
  --backdrop-blur: blur(20px) saturate(180%);

  /* — Motion — */
  --duration-fast:   100ms;
  --duration-normal: 200ms;
  --duration-slow:   400ms;
  --easing-in:     cubic-bezier(0.4, 0, 1, 1);
  --easing-out:    cubic-bezier(0, 0, 0.2, 1);
  --easing-in-out: cubic-bezier(0.4, 0, 0.2, 1);

  /* — Focus ring — */
  --outline-focus:       0 0 0 3px rgba(99, 102, 241, 0.6);
  --outline-focus-width: 3px;

  /* — PROJ-specific — */
  --proj-modal-side-panel-width: 480px;
  --proj-timeline-day-px:        40px;
  --proj-card-height:            92px;
}

[data-theme="dark"] {
  --color-surface-panel:        rgba(15, 23, 42, 0.7);
  --color-surface-elevated:     rgba(15, 23, 42, 0.9);
  --color-surface-translucent:  rgba(15, 23, 42, 0.5);
  --color-surface-modal:        rgba(15, 23, 42, 0.95);
  --color-text-primary:         #f1f5f9;
  --color-text-secondary:       #cbd5e1;
  --color-text-disabled:        #64748b;
  --color-border:               rgba(241, 245, 249, 0.1);
  --color-border-strong:        rgba(241, 245, 249, 0.2);
  /* Brand colors retain hue in dark — same hex */
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {
  :root {
    --duration-fast:   0ms;
    --duration-normal: 0ms;
    --duration-slow:   0ms;
  }
}

/* Focus-visible global rule */
*:focus { outline: none; }
*:focus-visible { box-shadow: var(--outline-focus); }
```

### Storybook config

```typescript
// web/proj-client/.storybook/main.ts
export default {
  stories: ['../src/**/*.stories.@(tsx|mdx)'],
  addons: ['@storybook/addon-a11y', '@chromatic-com/storybook'],
  framework: { name: '@storybook/react-vite', options: {} },
};
```

```typescript
// web/proj-client/.storybook/preview.tsx
import '../src/styles/global.css';   // includes tokens.proj.css

export const decorators = [
  (Story, { globals }) => (
    <div data-theme={globals.theme ?? 'light'}>
      <Story />
    </div>
  ),
];

export const globalTypes = {
  theme: { name: 'Theme', defaultValue: 'light',
           toolbar: { items: ['light', 'dark'] } },
};
```

### a11y CI

```typescript
// web/proj-client/playwright/a11y.spec.ts
import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const STORIES = [
  'kanban-board--default', 'kanban-board--focused',
  'timeline--default', 'timeline--with-milestones',
  'gantt--default', 'gantt--critical-path',
  'brief-modal--default', 'brief-modal--with-history',
  'status-picker--all-states',
  'meta-sidebar--default',
];

for (const story of STORIES) {
  test(`a11y: ${story}`, async ({ page }) => {
    await page.goto(`http://localhost:6006/iframe.html?id=${story}`);
    const results = await new AxeBuilder({ page }).analyze();
    const critical = results.violations.filter(v => v.impact === 'critical' || v.impact === 'serious');
    if (critical.length) {
      console.error(JSON.stringify(critical, null, 2));
    }
    expect(critical).toHaveLength(0);
  });
}
```

### GitHub Actions workflow

```yaml
# .github/workflows/proj-a11y-gate.yml
name: PROJ a11y gate
on:
  pull_request:
    paths: ['web/proj-client/**']
jobs:
  axe:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - run: cd web/proj-client && npm ci
      - run: cd web/proj-client && npx storybook build --quiet
      - run: cd web/proj-client && npx http-server storybook-static -p 6006 &
      - run: cd web/proj-client && npx wait-on http://localhost:6006
      - run: cd web/proj-client && npx playwright install --with-deps chromium
      - run: cd web/proj-client && npx playwright test a11y
```

### Contrast test

```typescript
// web/proj-client/tests/contrast_test.ts
import { calculateContrast } from 'color-contrast';
import tokens from '../src/styles/tokens.proj.json';

const PAIRS_TO_CHECK = [
  { fg: 'text-primary',   bg: 'surface-panel',     minRatio: 4.5 },
  { fg: 'text-on-brand',  bg: 'brand-primary',     minRatio: 4.5 },
  { fg: 'text-primary',   bg: 'surface-modal',     minRatio: 4.5 },
  { fg: 'text-secondary', bg: 'surface-panel',     minRatio: 4.5 },
  // ... 30+ pairs covering empirical component usage
];

test.each(PAIRS_TO_CHECK)('contrast: $fg on $bg ≥ $minRatio', ({ fg, bg, minRatio }) => {
  const ratio = calculateContrast(tokens[`color-${fg}`], tokens[`color-${bg}`]);
  expect(ratio).toBeGreaterThanOrEqual(minRatio);
});
```

---

## §4 — Acceptance criteria

1. **tokens.proj.css imports** — global.css imports it; CSS custom properties available.
2. **Dark theme swaps colors** — `[data-theme="dark"]` → all surface/text colors change.
3. **stylelint blocks hardcoded color** — PR adding `color: #fff` → CI fails with rule violation.
4. **Storybook has stories for all 8 components** — Board, Column, IssueCard, Timeline bar, Gantt arrow, BriefModal, StatusPicker, MetaSidebar.
5. **Each component has 3+ variants** — default/focused/error.
6. **Chromatic baseline on PR** — new component → Chromatic snapshot taken; human review prompt.
7. **axe-core passes on every story** — CI workflow `proj-a11y-gate.yml` green.
8. **axe-core fails on critical violation** — fixture: button without aria-label → CI red.
9. **Contrast test passes for all pairs** — `contrast_test.ts` all green.
10. **Focus-ring visible on Tab** — every interactive component shows outline on :focus-visible.
11. **Reduced-motion honoured** — `prefers-reduced-motion: reduce` → duration tokens = 0ms.
12. **Liquid-Glass backdrop-filter applied** — translucent surfaces have backdrop-filter; fallback solid on Firefox < 103.
13. **tokens.proj.json exported** — JSON mirror at build time; matches CSS values.
14. **No raw hex in component CSS** — grep components/*.css → no `#[0-9a-f]{6}` outside tokens file.
15. **Focus restored on modal close** — Brief Modal close → focus returns to opener.
16. **OS dark preference honoured** — `prefers-color-scheme: dark` → defaults to dark; explicit attr overrides (AC for §1 #12).
17. **Token diff report on PR** — PR touching tokens → diff comment posted with impacted components (AC for §1 #13).
18. **High-contrast preference honoured** — `prefers-contrast: more` → surfaces opaque; borders strong (AC for §1 #14).
19. **Semantic token indirection** — `--color-action-primary` resolves to brand-primary; component uses semantic (AC for §1 #15).
20. **Design-system audit CLI** — `cyberos design-system audit` outputs coverage % + deviations (AC for §1 #16).
21. **Token CHANGELOG present** — tokens.proj.css major bump → CHANGELOG entry (AC for §1 #17).
22. **axe-core ruleset configured** — runs wcag2aa + wcag21aa + best-practice; no AAA (AC for §1 #18).
23. **Build telemetry reports unused tokens** — webpack/vite plugin output lists unused (AC for §1 #19).
24. **CSS-in-JS escape hatch audited** — usage of `data-component-style` logged (AC for §1 #20).
25. **Kbd-nav hint in empty state** — empty kanban column → "Press Tab to navigate" hint (AC for §1 #21).
26. **Cheatsheet doc exists** — design-system/cheatsheet.md present (AC for §1 #22).

---

## §5 — Verification

```typescript
test('all component stories pass axe', async () => {
  for (const story of STORIES) {
    const violations = await runAxeOnStory(story);
    expect(violations.filter(v => v.impact === 'critical' || v.impact === 'serious')).toHaveLength(0);
  }
});

test('text-primary on surface-panel passes WCAG AA', () => {
  const ratio = calculateContrast(tokens['color-text-primary'], tokens['color-surface-panel']);
  expect(ratio).toBeGreaterThanOrEqual(4.5);
});

test('dark theme swaps surface', () => {
  document.body.setAttribute('data-theme', 'dark');
  const surface = getComputedStyle(document.body).getPropertyValue('--color-surface-panel');
  expect(surface).toContain('rgba(15, 23, 42');
});

test('reduced motion zeroes durations', () => {
  matchMedia('(prefers-reduced-motion: reduce)');
  const duration = getComputedStyle(document.documentElement).getPropertyValue('--duration-normal');
  expect(duration.trim()).toBe('0ms');
});

test('stylelint blocks raw color', async () => {
  const result = await stylelint({ code: '.x { color: #fff; }', config: { extends: ['stylelint-no-hardcoded-values'] } });
  expect(result.errored).toBe(true);
});
```

---

## §6 — Implementation skeleton

(Tokens + Storybook config + workflows above.)

---

## §7 — Dependencies

- **TASK-PROJ-014** — Kanban consumes status-color tokens.
- **TASK-PROJ-015** — Timeline uses day-px token.
- **TASK-PROJ-016** — Gantt uses critical-path-border token.
- **TASK-PROJ-017** — Brief Modal uses side-panel-width token.
- **design-system workspace** — cross-reference per CLAUDE.md.

---

## §8 — Example payloads

(N/A — tokens, not API rows.)

---

## §9 — Open questions

All resolved. Deferred:
- Per-tenant theming (brand colors per client) — slice 4+.
- Animation token expansion (springs, parallax) — slice 4+.
- Tokens.proj.json → Figma sync — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| axe-core critical violation | CI red | PR blocked | Author fixes |
| Contrast test fails | Jest red | PR blocked | Designer tweaks tokens |
| Chromatic visual regression | human review | PR awaits accept | Designer accepts or rejects |
| stylelint catches hardcoded value | CI red | PR blocked | Author moves to token |
| backdrop-filter unsupported | fallback solid | Visual degrade only | None |
| Token JSON drift from CSS | mirror generator | CI mismatch | Author regenerates |
| Storybook build fails | CI red | PR blocked | Author fixes story |
| Dark theme contrast gaps | Contrast test for dark token combinations | PR blocked | Designer adjusts |
| Reduced motion not honoured | manual audit | a11y warn | Author adds @media check |
| Focus ring covered by overflow | manual audit | a11y issue | Author fixes |
| Chromatic flakes on font load | wait-for-fonts hook | None | None |
| Multi-byte font glyphs broken | font-display: swap | Briefly system-fallback | None |
| Token bloat (200+ tokens) | maintenance | None | Curate periodically |
| OS color-scheme detection unreliable | manual toggle | None | Operator |
| Token diff report > 100 changes | summary collapsed | None | Reviewer expands |
| prefers-contrast not supported in browser | falls back to default | None | None |
| Semantic token cyclic reference | startup parse Err | CI red | Author fixes |
| Design-system audit > 5min on huge codebase | parallel walker | None | None |
| Token CHANGELOG inconsistent | manual review | warn | Operator |
| axe rule false positive | suppress with explanation | None | Author justifies in PR |
| Unused token survives audit | informational | None | Operator removes |
| CSS-in-JS escape hatch abused | audit count rises | sev-3 warning | Operator reviews |
| Empty-state hint clashes with UX | hide on second visit | None | Operator |
| Cheatsheet stale | CI checks tokens vs doc | warn | Author updates |
| Multi-locale token names (RTL) | tokens are CSS; locale-agnostic | None | None |
| HC mode breaks Liquid-Glass aesthetic | accepted; a11y > aesthetic | None | None |
| Token JSON export drift | mirror generator runs in CI | error | Author regenerates |

---

## §11 — Implementation notes

- The Liquid-Glass shader uses `backdrop-filter: blur(20px) saturate(180%)`; Firefox support since v103, Safari v9+, Chrome v76+.
- Color-contrast formula via `color-contrast` npm package (per WCAG 2.1).
- Stylelint custom rule `no-hardcoded-values` is an in-repo plugin under `web/proj-client/.stylelint/`.
- Chromatic project ID per env var `CHROMATIC_PROJECT_TOKEN`; CI uses repo secret.
- `:focus-visible` is preferred over `:focus` — only shows ring on keyboard navigation (not mouse click), reducing UI noise for mouse users.
- Reduced-motion media query is global in tokens; component-level animation declarations inherit.
- The design-system workspace's `AGENTS.md` is referenced; cross-module token convergence is a future PR.
- All token names are kebab-case; no abbreviations (e.g. `font-size-base` not `font-size-md`).
- OS color-scheme detection uses `window.matchMedia('(prefers-color-scheme: dark)')`; falls back to light if unsupported.
- Token diff report is a GitHub Action that compares CSS variables between base and head branches; posts as PR comment.
- prefers-contrast support uses `@media (prefers-contrast: more)` to override surface translucency.
- Semantic tokens reduce churn: changing brand color = one update; without semantic, every component reference must update.
- Design-system audit CLI walks JSX/TSX files, finds className references, cross-references against tokens.proj.css.
- Token versioning follows semver; major bump on rename/remove, minor on add, patch on value adjustment.
- axe-core rules `wcag2a/aa/21aa/best-practice` cover ~95% of real-world a11y issues; AAA is too strict for app UIs.
- Build telemetry: vite plugin instruments CSS extraction; unused tokens listed in build output.
- CSS-in-JS escape hatch is `data-component-style="<reason>"`; the reason field is required + audited.
- Empty-state hints use aria-hidden="false" so screen readers also benefit.
- Cheatsheet doc lives at design-system/cheatsheet.md; updated via CI when tokens change.
- We considered using TypeScript types for tokens (typed object) but kept CSS-native for runtime theming flexibility.
- The semantic token layer is documented in cheatsheet under "When to use semantic vs raw."
- Token CHANGELOG follows Keep-a-Changelog format; categorises adds/changes/removes.
- For multi-tenant theming (slice 4+), per-tenant tokens override at the `[data-tenant="<id>"]` level.
- The audit CLI integrates with TASK-PROJ-008's CI gate; coverage below 90% = SEV-3 warning.
- We use Chromatic over Percy because Chromatic's Storybook integration is tighter.
- Storybook builds are cached in CI to keep PR runtime under 5min.
- The Liquid-Glass aesthetic is dialed down (lower blur radius) in high-contrast mode for accessibility.
- We considered tokens.proj.json schema validation (Ajv) but the JSON is generated, so source-of-truth is CSS.
- Token CSS supports both light theme (`:root`) and dark theme (`[data-theme="dark"]`); operator preference persisted in localStorage.
- Per-PR token diff comment includes a markdown table: token | before | after | components-affected.
- The design-system audit CLI is intended for CI and on-demand; not real-time.
- Empty-state hints use the proper "polite" ARIA live region so they don't interrupt focus.
- The cheatsheet doc also serves as the contract for FR-AUTH and FR-AI dashboards to consume PROJ tokens.

---

*End of TASK-PROJ-018.*
