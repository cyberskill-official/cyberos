---
id: FR-GAM-005
title: "Theming with light/dark modes and live preview"
module: GAM
priority: SHOULD
status: done
fidelity: as-built
shipped: 2026-06-23
owner: Stephen Cheng
source_repo: zintaen/gam @ f55d97c
related_frs: [FR-GAM-004]
---

## §1 — Description (BCP-14 normative)

gam SHOULD offer multiple visual themes with light and dark modes.

1. The app SHOULD offer multiple theme styles, each with light and dark modes.
2. The app MUST apply a theme choice live, allowing preview before commit.
3. Previewing a theme MUST NOT persist it; only an explicit commit persists.
4. The app MUST persist the committed theme and reload it at startup (see FR-GAM-004).
5. An invalid or unknown stored theme MUST fall back to the default theme.
6. The applied theme MUST be expressed on the document via `data-style` and `data-mode` attributes so CSS can react.

## §2 — Why this design

Live preview lets users judge a theme in context before committing, which is the difference between a theme picker people use and one they avoid. Preview must not persist, or "just looking" silently changes saved state. Falling back to a default on an unknown stored value makes the app forward- and backward-compatible across theme-set changes without a migration step.

## §3 — Implementation

- `src/hooks/useTheme.ts` — theme state: default `glassmorphism-dark`, read stored `gam-theme`, invalid-value fallback, `setThemeId` (persists), `previewTheme` (does not persist), `cancelPreview`, and application of `data-style` / `data-mode` to `document.documentElement`.
- The theme picker UI in the settings dropdown.
- Committed theme persists via the settings store (FR-GAM-004).

## §4 — Acceptance criteria

1. Default theme is `glassmorphism-dark` on a clean install.
2. A stored theme loads on startup.
3. An invalid stored theme falls back to the default.
4. `setThemeId` changes the theme and persists `gam-theme`.
5. `previewTheme` changes the display but leaves the persisted value untouched; `cancelPreview` reverts to the committed theme.
6. Selecting a theme sets `data-style` and `data-mode` on the document.

## §5 — Verification

`tests/hooks/useTheme.test.ts` covers all six: default, stored value, invalid fallback, `setThemeId` persistence, `data-style`/`data-mode` application, preview-without-persist, `cancelPreview`, and `themeConfig` correctness.

## §6 — Failure modes

| Failure | Detection | Outcome |
|---|---|---|
| Unknown stored theme | validation | default theme |
| Preview left uncommitted | `cancelPreview` / commit | reverts or persists explicitly |
| Settings unwritable | FR-GAM-004 path | theme applies for session, may not persist |

## §7 — Notes

Theme is the most-exercised hook in the frontend suite and a clean example of preview-vs-commit state.

*End of FR-GAM-005. Fidelity: as-built (10/10 target).*
