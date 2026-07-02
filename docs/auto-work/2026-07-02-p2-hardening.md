# P2 hardening run - 2026-07-02

Follow-up to the module review (docs/reviews/MODULE-REVIEW-2026-07-02.md) and its plan
(docs/reviews/IMPROVEMENT-PLAN-2026-07-02.md). Operator asked to continue the open P2 items. Each change
was gated before commit; nothing is pushed (a push deploys prod - that stays the operator's call).

## What shipped this run

### P2 #11 - review pass on the parallel-session chat commits (ab2989d, 3b82968)

Audited the i18n catalog, the per-channel notify prefs backend, drafts, and the mobile drawer. Findings and
fixes:

- i18n catalog: clean. 180 keys, every one used, none dead, plurals selected correctly at all call sites,
  and every {var} placeholder is supplied. No English leaks in the touched components. One real gap: the
  brand slogan was hard-coded English ("Turn Your Will Into Real") on the otherwise-localized login header
  and the app topbar, while the brand has an established Vietnamese form. Fixed: added a "brand.slogan" key
  (en "Turn Your Will Into Real", vi "Hiện Thực Hoá Ý Chí", copied brand-exact from the landing-page
  site.ts) and used it in Login.tsx and App.tsx.
- Notify prefs backend (prefs.rs, notify.rs, migration 0012): correct. The delivery rule is a pure,
  unit-tested function that fails open on unknown modes; the fan-out excludes the sender and falls open to
  "all" if the prefs query errors; migration 0012 has the right composite PK, CHECK, tenant RLS, and index.
  No change needed.
- Drafts: the per-channel localStorage swap is correct. The one theoretical load-order edge case cannot
  fire because activeId starts empty and is only set after channels load async (the drafts-load effect has
  already run by then). No change.
- Mobile drawer a11y: three fixes. (1) Escape now closes the drawer (previously only a backdrop tap or a
  channel pick could). (2) The backdrop is marked aria-hidden. (3) The closed off-screen drawer used to
  stay in the tab order and the screen-reader tree on narrow viewports - a keyboard user could Tab into an
  invisible menu. Fixed with a CSS visibility toggle inside the max-width:900px media query (visibility
  hidden after the slide-out, visible on open); wide screens are untouched.

### P2 #13 - theme contrast pass (both themes)

Computed WCAG 2.1 contrast for every load-bearing text-on-surface pair in both themes rather than eyeballing.
Dark theme: every pair passes AA (lowest 4.54). Light theme: two text tiers dipped just under 4.5 on the
raised surface panel_2 (#ebdfc8) - faint 4.30 and accent-text 4.28 - which breaks the sheet's own stated
promise that faint stays >= AA for small text. Fixed by darkening the two light-theme tokens minimally:
--faint #7d6244 -> #755d3d (panel_2 4.70) and --accent-text #8e5d00 -> #835400 (panel_2 4.92). Both now
clear AA on all three surfaces and the faint-vs-muted tier order is preserved. The ochre fill, the accent
border, and the whole dark theme are untouched. The semantic ok/bad colors sit within rounding of 4.5 but
are used as dots, badges, and borders (graphical, 3.0 threshold), so they were left as-is.

### P2 #10 - OBS-007 runbook-URL fabrication fix (precondition for deploying obs-router)

The CUO obs.triage-alert skill is told never to invent a runbook URL, but a local model can still echo the
example URL from the skill doc (`https://kb/.../rollback-gateway`) or fabricate a slug. So CHAT never shows
a made-up runbook link, the router now keeps a suggested runbook only when its URL is exactly one of the
known KB runbook URLs (the allowlist / KB index). Exact match, not host-prefix, so a fabricated slug on the
real kb.cyberos.world host is rejected too. An empty allowlist trusts nothing (fail-closed): the runbook is
dropped, CHAT shows "Runbook: none", and the alert still routes and pages exactly as before.

- New pure module services/obs-router/src/runbook.rs (sanitize_runbook) with unit tests, including the
  SKILL.md example URL, a fabricated slug on the real host, and the fail-closed empty case.
- config.rs: a runbook_allowlist field parsed from OBS_RUNBOOK_ALLOWLIST (comma- or whitespace-separated
  exact URLs).
- handle.rs: route_alert sanitizes the suggested runbook right after triage, so both the CHAT post and the
  obs.alert_triaged audit only ever carry a verified runbook; plus a new end-to-end test proving the runbook
  is dropped unless allowlisted.
- Deploy requirement: set OBS_RUNBOOK_ALLOWLIST to the real KB runbook URLs before obs-router goes live.

## Gate evidence

- obs-router: cargo fmt (clean), clippy -p cyberos-obs-router --all-targets -- -D warnings (0), test -p
  cyberos-obs-router (32 lib tests + 2 integration, all pass; the new runbook + orchestration tests run).
- web: npm run build (tsc --noEmit + vite build + stamp-sw) exit 0.

## Left for the operator

- Push is the operator's call (deploys prod). These changes are on branch auto/p2-hardening for review.
- Pre-existing uncommitted dependency bumps were found in the tree and left untouched: apps/web vite 5 -> 8
  and @vitejs/plugin-react 4 -> 6, and modules/skill wasmtime 27 -> 36 (the ledgered semver-major P2). They
  are not part of these commits. The web bundle here was rebuilt in that tree; if the vite-8 bump is
  finalized, one more npm run build will re-hash the bundle (harmless).
- P2 #12 (AI activation: VPS resize + COMPOSE_PROFILES=llm) stays an operator infra action.
