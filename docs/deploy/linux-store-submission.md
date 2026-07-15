# Linux store submission — Snap Store + Flathub answer sheet (TASK-APP-005)

Two architecturally distinct channels (spec §1 #2): **Snap Store** is CI-automatable (`release-snap.yml`, gated on `SNAP_RELEASE=true`, uploads a CI-built `.snap`); **Flathub** accepts no artifact upload — its own infra builds from the manifest at `flathub-manifest/os.cyberskill.world.desktop.yml`, and the submission PR against the external `flathub/flathub` repository is a Stephen-approved, per-instance action (never automated; structurally enforced by AC #6's standing grep in the workflow's lint job — which passes precisely because that repo reference lives only here in docs/, outside the scanned directories).

## Section 1 — Snap Store

### Hard blockers on `SNAP_RELEASE=true`

| # | Blocker | Owner | Status |
|---|---|---|---|
| 1 | Ubuntu One / Snapcraft account + registration of the `cyberos` snap name (free) | Stephen | pending-human |
| 2 | `snapcraft export-login` → store credentials into secret `SNAPCRAFT_STORE_CREDENTIALS` | Stephen | pending-human |
| 3 | WORKER confirmations flagged in-recipe: `architectures:` schema form for core22; actual `.deb` internal binary path (staging step fails loudly if wrong) | engineering (first `SNAP_RELEASE=true` run surfaces both) | open |

### Snap Store metadata

| Field | Recommended answer | Status |
|---|---|---|
| Name | `cyberos` (must match the registered name) | pending-human |
| Summary | CyberOS — Turn Your Will Into Real | pending-human |
| Description | snapcraft.yaml's description block (thin desktop shell over os.cyberskill.world/web) | pending-human |
| Category | Productivity / Development | pending-human |
| Confinement | `strict` — no classic justification needed; minimal 6-plug set (desktop, desktop-legacy, wayland, x11, network, opengl), each mapped to a real capability (display/input, remote-shell networking, WebKitGTK GPU compositing) | human-confirmed (structural — lint-enforced) |
| Channel strategy | Skeleton releases straight to `stable`; §9 open question — Stephen may prefer candidate/beta + manual `snapcraft release` promotion | pending-human |

### Required manual smoke-test before any `stable` promotion

Install the CI-built artifact locally: `snap install --dangerous cyberos_*.snap`, then verify (a) the webview renders (catches a missing implicit plug — e.g. gsettings — and the `-dev` vs runtime `-N` stage-package class of mistake, spec §10), (b) outbound network works, (c) the window system integrates under both Wayland and X11. A packing success alone proves neither.

### Operational notes

- `SNAPCRAFT_STORE_CREDENTIALS` is a bounded-validity macaroon — `snapcore/action-publish` auth failures mean re-export + rotate (routine ops).
- snapcraft.yaml carries `version: '1.0.0'` literal — **not yet wired into `scripts/stamp-release-version.mjs`**; until a follow-up wires it (or switches to `adopt-info`), bump it manually per release (disclosed in the task review packet as a known drift risk).

## Section 2 — Flathub

### Hard blocker on ever opening the Flathub PR

**App-id / domain-ownership decision (spec §9):** `os.cyberskill.world.desktop` is the working candidate, but Flathub's ownership-verification convention must be confirmed against its current submission docs (likely requiring proof of control of `cyberskill.world` via Flathub's documented mechanism). Until resolved, the manifest filename, `app-id:` field, and the deliberately-unauthored `.desktop` entry are all provisional. Layered on top: Stephen's explicit, fresh chat-turn approval is required per-instance before anyone opens the PR (spec §1 #7) — even after the app-id resolves.

### Flathub review checklist

| Item | Prepared answer | Status |
|---|---|---|
| App-id convention + domain ownership | Working candidate `os.cyberskill.world.desktop`; verify per above | pending-human |
| `finish-args` justification | `--share=network` → thin remote-shell architecture (release.yml's own comment confirms the app is a shell over os.cyberskill.world/web); `--socket=wayland`/`fallback-x11` → display; `--device=dri` → WebKitGTK hardware compositing; `--share=ipc` → X11/WebKit shared-memory use | human-confirmed (drafted; cite at review) |
| .desktop entry + AppStream metainfo | Deferred by design until app-id lands (spec §6) — authoring against a provisional id guarantees rework | open (engineering, post-app-id) |
| Source provenance (`sources:` pinning) | git tag pin today; confirm tag-vs-tarball+checksum expectation against Flathub's live docs at proposal time (spec §6) | open (engineering, at proposal) |
| Icon | 128x128 PNG from the committed Tauri icon set, installed to hicolor per the manifest | human-confirmed (structural) |
| Local validation before PR | `flatpak-builder --force-clean --repo=<scratch> <builddir> flathub-manifest/os.cyberskill.world.desktop.yml` exits 0 (AC #4; exact parse-only flag TBD against the installed version's --help) | open (engineering, pre-PR) |
