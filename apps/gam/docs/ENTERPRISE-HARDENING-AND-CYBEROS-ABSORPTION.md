# gam: enterprise hardening and CyberOS absorption roadmap

Status: analysis and plan. No code changes made by this document.
Date: 2026-06-23
Owner: Stephen Cheng (zintaen)
Scope decided with the user: harden the core and expand into team/enterprise capability, delivered as analysis plus research plus a prioritized roadmap, prepared carefully for a future merge into CyberOS.

This document is the front of a project, not the work itself. It says what gam is today, what "enterprise-grade git tool" actually requires (with current external evidence), where the gaps are, how gam should fold into CyberOS without friction, and the order to do it in. It is written to be executed under CyberOS workflow discipline, so the operating method is described first.

---

## 1. Operating method: the CyberOS workflows this plan runs under

The point of the request was to strengthen gam using CyberOS workflows, not just to list improvements. Every phase below is meant to run through the same loop CyberOS already uses, which gam has partly adopted.

The loop is plan, then create, then test, then deploy, with an evidence gate that decides when a task is actually done. gam already carries the scaffolding: `.agent/workflows/` holds the plan, create, test, debug, deploy, enhance, and orchestrate workflows, and `.awh/` holds the Stage 0/1 evidence gate (`gate.sh` runs `pnpm lint && pnpm test && tsc --noEmit`, `policy.json` makes test files read-only during agent runs, `goldenset.yaml` plus `eval-baseline.json` define the regression baseline).

How each piece is used in this roadmap:

- Plan and brainstorm workflows produce the per-item specs. In CyberOS terms these become FR-GAM and NFR-GAM documents (Section 6).
- Create and enhance workflows do the build, one task at a time, on a dedicated branch.
- The awh gate is the testing-to-done boundary. A task is not done until the gate is green on a clean tree. This is the same rule CyberOS enforces through its pre-commit hook and `awh-gate.yml`, and through CUO `ship-feature-requests` step 28.
- The no-pausing self-verify loop applies: author, test, lint, commit, repeat, and only stop at a genuine fork or at a push or deploy boundary. That is already how the user runs autonomous sessions.

One correction has to happen before the gate can be trusted: gam's committed `eval-baseline.json` was captured on a tree where lint and typecheck were failing (both score 0.0, only vitest green, weighted pass 0.6). A regression gate built on that baseline will tolerate lint and type errors. Re-baselining on a green Mac tree is a P0 item (Section 7).

---

## 2. What gam is today

gam is a desktop GUI for managing Git aliases, built on Tauri v2 with a Rust backend and a React 19 plus TypeScript plus Vite front end. It is at version 1.0.11, MIT licensed, single-author. It ships seven build artifacts per release (macOS aarch64 and x86_64 dmg, Windows exe and msi, Linux deb, rpm, AppImage), auto-updates through a signed Tauri updater pulling from GitHub Releases, and installs on macOS through a fully automated Homebrew cask.

The product is genuinely feature-complete for its stated job: alias CRUD across global and local scope, a curated library of 270-plus aliases, five alias-name suggestion schemes, a usage-ranking algorithm, alias groups with colors, JSON import and export, dangerous-command warnings, and auto-backup of `.gitconfig` before writes. Architecture is clean and layered: user, component, hook, `tauri-bridge.ts`, IPC, Rust command, service, git CLI. There are 21 IPC commands, each mirrored one-to-one between the Rust `#[tauri::command]` functions and the front-end bridge.

Test depth is well above what most indie apps carry: 225 front-end test cases across 22 vitest files plus 66 Rust unit tests, with vitest coverage thresholds enforced (70 percent lines, functions, statements, 60 percent branches).

The honest summary: gam is an above-average indie app with security hygiene above its weight class. The work to make it enterprise-grade is not a rewrite. It is closing a small number of high-impact gaps, adding a team-governance capability the market does not currently serve well, and preparing the merge into CyberOS.

---

## 3. Current-state analysis, grounded in the code

These findings come from reading the actual source. File references are concrete so each can be acted on directly.

### 3.1 Security and supply chain

What is already strong:

- Command execution is safe by construction. `git_service.rs` `exec_git()` uses `std::process::Command::new("git")` with argument arrays, never a shell string. Alias names and commands flow in as discrete argv entries, so classic shell injection is structurally impossible. Defense in depth adds backend name validation (`^[a-zA-Z][\w-]*$`) with explicit injection-rejection tests. This is the correct trust boundary: gam writes aliases to git config, it does not execute them.
- The Content Security Policy in `tauri.conf.json` is tight: `default-src 'self'`, `script-src 'self'` with no `unsafe-inline` or `unsafe-eval`, and `connect-src` limited to the IPC channel and the two GitHub update hosts.
- The release pipeline is mature. Third-party actions are SHA-pinned, an SPDX SBOM is generated through `anchore/sbom-action`, and build provenance attestations are produced through `actions/attest-build-provenance`. CI also runs a 3-OS matrix with `cargo clippy -D warnings`, a `security-audit` job (`pnpm audit`, `cargo audit`), and `dependency-review-action`.
- The updater public key is configured, so the update channel is integrity-checked.

What is wrong and matters:

- Leaked updater signing key, high severity. The untracked `.env` holds a live `TAURI_SIGNING_PRIVATE_KEY` plus its password, and the password is `[redacted]`. This is the private half of the updater public key in `tauri.conf.json`. Anyone with that file and the trivial password can sign an update that every installed copy of gam auto-accepts. It was never committed to git history (verified) and CI uses GitHub Secrets rather than the file, so the blast radius is the local machine and anyone who held the working tree. It must be treated as compromised: rotate the keypair, update the public key, delete the file, move the secret to a manager. This document deliberately does not reproduce the key.
- No operating-system code signing or notarization, high severity for distribution. There is no Apple Developer ID, hardened runtime, or entitlements in `tauri.conf.json`, and no Authenticode or notarization step in CI. Every installer is unsigned, so macOS Gatekeeper and Windows SmartScreen warn users, and the Homebrew cask ships an unnotarized app. The strong updater signing protects the update channel but not first-install OS trust, which is an odd asymmetry to leave standing.
- Capability scope is coarse. `shell:allow-open` and `opener:default` are not constrained to a URL or path pattern. The Rust `open_external` scheme allowlist (HTTPS only) compensates, but capability-level scoping is the missing belt-and-suspenders.
- The install registry in `.npmrc` points at a third-party mirror (`registry.npmmirror.com`). That is a latency choice, but release builds should pin to the canonical registry or rely solely on the committed lockfile.

### 3.2 Code quality and robustness

The front end is idiomatic React 19 with real separation of concerns: `App.tsx` is pure composition, state lives in focused hooks, IPC is isolated behind a typed bridge, optimistic updates roll back correctly on failure, and a real `ErrorBoundary` is mounted at the root with global error and rejection listeners. Rust state is three `RwLock`-guarded services that recover from lock poisoning rather than panicking.

The gaps:

- End-to-end tests are shallow. `e2e/alias-crud.spec.ts` is a 3-test smoke suite that runs against the Vite dev server, not the Tauri shell, and each assertion is guarded by an "if elements exist" check that passes silently when elements are missing. Real alias-CRUD-through-the-backend coverage is absent.
- Rust integration tests are missing. The 66 Rust tests cover parsing, validation, and persistence on canned data. The actual `git` subprocess path and the IPC command layer are not exercised end to end.
- No internationalization. There is no i18n framework and all strings are hardcoded English. For a global CyberSkill rollout this is from-scratch work.
- Accessibility is partial. There are aria and role attributes across 13 components, but no automated a11y gate.
- A privacy and consent item: the ranking service silently reads the user's shell history files (zsh, bash, Fish, PowerShell) to score aliases. It is local and read-only, but an enterprise review will flag an undisclosed read of sensitive history. It needs a disclosure and consent surface.

### 3.3 Distribution

Seven artifacts per release, GitHub Releases plus the Tauri updater, and an automated Homebrew cask. There is no winget, Scoop, Flatpak, Snap, or AUR presence. The unsigned binaries (Section 3.1) are the blocker that gates a real distribution expansion, since every channel except a manual download benefits from signing and from SmartScreen reputation.

### 3.4 Absorption-readiness signals

- `.awh/` is present and structurally aligned with how CyberOS gates its modules, but the baseline is red (see Section 1).
- `.claude/settings.json` wires the harness hooks with absolute host paths under `~/Projects/auto-work-harness`. Those break for any other environment or CI and must be made relative before a merge.
- `.agent/` is a large local agent toolkit, but it is gitignored, so it will not travel into the monorepo as-is. A decision is needed: vendor it under the app folder or drop it.
- The dependence on `@cyberskill/shared` is shallow at runtime. It drives lint, hooks, and CI, but there are zero `@cyberskill/shared` imports in `src/`. The application logic is decoupled from the toolchain, which makes the merge mostly a config reconciliation rather than a code rewrite.

### 3.5 Top gaps ranked by impact

1. Leaked updater signing key with a trivial password (high, security). Rotate now.
2. No OS code signing or notarization (high, distribution and trust).
3. Stale red awh baseline that tolerates lint and type failures (medium-high, process integrity).
4. Hardcoded absolute host paths in `.claude/settings.json` (medium-high, portability and absorption).
5. Unscoped shell and opener capabilities (medium, security depth).
6. Shallow e2e and missing Rust integration tests (medium, robustness).
7. No i18n (medium, global reach).
8. Undisclosed shell-history reads (medium, privacy and consent).
9. `.agent/` toolkit gitignored, will not travel (medium, absorption).
10. Install registry pinned to a third-party mirror (low-medium, supply chain).
11. Doc drift in `docs/CODEBASE.md` (command count, test count, crash-log path) and a stray unresolved value in `pnpm-workspace.yaml` (low, hygiene).

---

## 4. Research: what enterprise-grade actually requires

This section is the external research, organized by the four axes the user weighted, with current evidence. Sources are listed at the end.

### 4.1 Security and supply chain

Code signing and notarization is the single largest trust gap. The current evidence on the two platforms:

- macOS: an Apple Developer ID certificate (99 USD per year, globally available including Vietnam) plus notarization removes the Gatekeeper "unidentified developer" block. Tauri supports this directly through signing identity, hardened runtime, and entitlements config plus a notarization step in CI.
- Windows: the landscape shifted in 2024 and 2025. Extended Validation certificates no longer bypass SmartScreen on first download, that behavior was removed in 2024, so EV-signed files now build reputation the same way OV-signed files do. Microsoft's Azure Trusted Signing (renamed Azure Artifact Signing) is now the cost-effective path at 9.99 USD per month for up to 5,000 signatures, and it integrates cleanly with CI. The catch that matters for CyberSkill: individual developers are limited to the USA and Canada, and organizations to the USA, Canada, the EU, and the UK. A Vietnam-registered entity may not qualify directly, which makes Windows signing a real decision (Section 8), not a checkbox. A traditional OV code-signing certificate (roughly 200 to 400 USD per year from a CA) remains the geography-independent fallback.

Supply-chain posture is already a relative strength. gam ships an SBOM and provenance attestations and pins its actions, which puts it ahead of most apps. Maturing it means moving toward SLSA-style guarantees, adding `cargo-vet` or `cargo crev` for dependency review, and treating the SBOM and attestations as release gates rather than artifacts. The Tauri security guidance reinforces the rest: keep the CSP as restrictive as possible (gam already does), avoid remote scripts, scope capabilities per window, and keep production secrets off development machines and out of version control. The leaked key in Section 3.1 is exactly the development-machine-secret failure that guidance warns about.

### 4.2 Team and enterprise capability

This is where the "expand" half of the scope lives, and where the market is thin. Git itself already has the primitives an enterprise needs, but no friendly tool surfaces them:

- The three-tier config hierarchy (system, global, local) is how settings cascade.
- `includeIf` conditional includes switch configuration by directory or by repository, which is how teams manage multiple identities and per-project rules without manual switching.
- `includeIf "hasconfig:remote.*.url"` switches identity by remote URL, so the right identity and keys apply wherever a repo is cloned.
- `.gitattributes` standardization is how teams avoid line-ending and diff inconsistencies.

At scale, teams hand-roll this through dotfiles repositories, bare-repo dotfile tricks, and onboarding scripts. There is no governed, auditable, GUI-driven way to push a standard set of aliases and config to a team and verify adoption. That gap is gam's enterprise opportunity: shared alias and config packs, policy profiles expressed as `includeIf` rules, an audit view of what is actually configured on a machine versus the org standard, and managed distribution. None of the mainstream GUIs do this (Section 4.4).

### 4.3 Distribution and reach

Tauri documents first-class paths to Debian, RPM, AppImage, Snap, Flatpak, and AUR on Linux, MSI and NSIS on Windows, and dmg on macOS. gam already produces the raw Linux artifacts but does not publish to the managed channels. The practical expansion set, in rough effort order: winget and Scoop on Windows (a manifest pull request each), Flatpak on Flathub (a manifest plus the GNOME runtime), AUR (an account plus SSH keys plus a PKGBUILD), then Snap. Signing is the prerequisite that unlocks most of these and reduces the SmartScreen and Gatekeeper friction that otherwise suppresses installs.

### 4.4 Competitive position

The competitive search returned a useful negative result: there is no established "git alias manager" or "git config manager" GUI category. The well-known clients (GitKraken, Tower, SourceTree, Fork, GitHub Desktop, SmartGit, Sublime Merge, plus terminal UIs like lazygit and gitui) are commit-graph and branch-and-merge visualizers. They are largely Electron-based, several struggle on large repositories, and the leaders are increasingly paywalled (GitKraken private repos now need a paid plan, Tower is 69 USD per year per user). Configuration and alias management is a side feature in all of them, not the product.

That is gam's positioning: a small, fast, free, OS-webview tool (3 to 6 MB versus bundled-Chromium clients) focused on the config and alias surface that the big GUIs treat as an afterthought, with a credible path to the team-governance capability none of them offer. The enterprise wedge is config governance, not yet-another commit graph.

---

## 5. Gap analysis: current versus enterprise-grade target

| Axis                 | Today                                                             | Enterprise-grade target                                                              | Largest single move                          |
| -------------------- | ----------------------------------------------------------------- | ------------------------------------------------------------------------------------ | -------------------------------------------- |
| Code signing         | Unsigned dmg, exe, msi                                            | Apple Developer ID plus notarization, Windows signing, signed Linux where applicable | Apple notarization in CI (global, 99 USD/yr) |
| Update integrity     | Updater key configured but private key leaked with weak password  | Rotated key in a secret manager, signing keys on hardware where possible             | Rotate and re-key now                        |
| Supply chain         | SBOM, provenance, pinned actions, audits                          | The above plus cargo-vet/crev and SBOM and attestation as release gates              | Add dependency review and gate on it         |
| App sandbox          | Tight CSP, no fs capability, coarse shell and opener scope        | Per-capability URL and path scoping, least privilege                                 | Scope shell and opener capabilities          |
| Tests                | 225 front-end plus 66 Rust unit, shallow e2e, no Rust integration | Real Tauri-shell e2e, Rust integration over the git subprocess, a11y gate            | Tauri-driver e2e plus git integration tests  |
| Privacy              | Silent shell-history read                                         | Disclosed, consented, toggleable history use                                         | Consent surface for ranking                  |
| Internationalization | Hardcoded English                                                 | i18n framework, at least English plus Vietnamese                                     | Introduce i18n, extract strings              |
| Team capability      | Single-user only                                                  | Shared alias and config packs, includeIf policy profiles, adoption audit             | Config governance feature                    |
| Distribution         | GitHub Releases plus Homebrew                                     | winget, Scoop, Flatpak, AUR, Snap, signed                                            | Publish to managed channels post-signing     |
| Process gate         | Red awh baseline                                                  | Green re-baselined gate wired into CI and pre-commit                                 | Re-baseline on a clean Mac tree              |

---

## 6. CyberOS absorption plan

The user's controlling constraint is that gam will be merged into CyberOS soon, so the plan is shaped to make that merge clean rather than to optimize gam in isolation. The mapping below is grounded in CyberOS's actual conventions.

Placement. gam goes to `apps/gam/`, owned by the `app` module. The precedent is exact: `apps/desktop` is already a Tauri v2 app in CyberOS (specced as FR-APP-002), and the `app` module's charter is first-party operator surfaces. gam mirrors that structure (front end under `apps/gam/src/`, Rust under `apps/gam/src-tauri/`) and stays a self-contained crate with its own committed `Cargo.lock`, deliberately not a member of the `services/Cargo.toml` workspace. Keeping it standalone contains the toolchain divergence (gam is Rust edition 2024 versus 2021 elsewhere, and uses a different Tauri plugin set).

Versioning. CyberOS uses a single root `VERSION` (currently 0.1.0) propagated by `scripts/release.sh`. `apps/desktop` already conforms to 0.1.0 rather than keeping an independent line. gam carries a real shipped 1.0.11 in three manifests, so conforming is a visible downgrade and therefore a decision (Section 8). The clean answer is to adopt the root VERSION, preserve gam's history as `apps/gam/CHANGELOG.md`, and extend `scripts/release.sh` to also rewrite the app manifests.

Memory protocol. CyberOS's `AGENTS.md` (a symlink to the memory module's protocol) is normative and repo-wide. gam is a non-memory component, so it falls under the INTEROP rule: it must never write to `.cyberos-memory/`, must not create a second memory-like root, and must keep its own local state (theme, groups, known repos) in the OS app-data directory where it already lives. gam has no competing `CLAUDE.md` or `AGENTS.md`, which is clean.

Agent and harness config. Relocate gam's `.claude/` and `.agent/` under `apps/gam/` so they do not shadow CyberOS's `modules/cuo` personas and skills, and repoint the awh hooks from the external `~/Projects/auto-work-harness` path to CyberOS's vendored `tools/awh`. Do not place a `.claude/` at the repo root, since CyberOS gitignores it.

Evidence gates. Move gam's `.awh/` to `apps/gam/.awh/`, re-capture a green baseline on a Mac, and extend CyberOS's `.pre-commit-hooks/awh-gate.sh` and `awh-gate.yml` to scan `apps/`, not only `modules/`. Add an `apps/gam/audit-profile.yaml` so the CAF code-audit gate (CUO step 29) covers gam's real build, lint, typecheck, and test commands. This is the one structural change CyberOS needs to gate an app.

FR and NFR catalog. Re-express gam's features as CyberOS feature requests under `docs/feature-requests/gam/` using the `feature_request@1` template and the BCP-14 normative block, each paired with a 10/10 audit file, plus a domain README and a BACKLOG registration under the 10-state lifecycle. A concrete decomposition mapped to gam's 21 commands: FR-GAM-001 alias CRUD across scope, FR-GAM-002 command validation and danger guards, FR-GAM-003 local-scope repo selection and known-repos, FR-GAM-004 usage ranking from shell history, FR-GAM-005 alias groups, FR-GAM-006 JSON import and export, FR-GAM-007 theming and settings, FR-GAM-008 auto-updater, FR-GAM-009 alias library and suggestions, NFR-GAM-001 fully local with no telemetry beyond the update check, NFR-GAM-002 single small multi-OS binary.

CUO workflow. gam's `.agent/workflows` are a local dev loop and are not CUO workflows, so they should not be registered as such. gam's feature work is driven by the existing CUO `ship-feature-requests` pipeline operating over the FR-GAM backlog, the same as every other component.

CI. gam brings a toolchain CyberOS does not run today (pnpm, Node 24, Vite, vitest, Playwright). The existing Ubuntu Python-and-Rust gate cannot run it, so a macOS-runner workflow (`gam-gate.yml`) is required, or gam silently never gets gated.

Branch. Do the absorption on `auto/gam-absorb`, matching the convention used for the awh and CAF absorptions (`auto/awh-absorb`).

Absorption compliance checklist: placement at `apps/gam/`; version reconcile to root VERSION; never write `.cyberos-memory/`; relocate and repoint `.claude/` and `.agent/`; move and re-baseline `.awh/`; add `audit-profile.yaml`; author the FR-GAM and NFR-GAM catalog with audits; register in BACKLOG; add the macOS CI gate; land on `auto/gam-absorb`.

---

## 7. Prioritized roadmap

Phasing follows CyberOS's P0 to P4 convention. Each phase is a set of tasks that each pass the awh gate before being called done. Effort is rough: S is under a day, M is a few days, L is a week-plus.

### P0, do first, unblocks everything (security and process integrity)

| Item                                                                                                            | Why                                                                          | Effort |
| --------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- | ------ |
| Rotate the updater signing keypair, delete `.env`, move the secret to a manager, update the public key          | A live signing key with password [redacted] is a silent auto-update RCE risk | S      |
| Re-capture a green `eval-baseline.json` on a clean Mac tree (lint, test, typecheck all passing)                 | The current red baseline lets the gate tolerate lint and type errors         | S      |
| Make `.claude/settings.json` hook paths relative or env-driven                                                  | Hardcoded host paths break every other environment and block absorption      | S      |
| Fix doc drift (`CODEBASE.md` command and test counts, crash-log path) and the stray `pnpm-workspace.yaml` value | Cheap correctness, removes confusion before the merge                        | S      |

### P1, harden the core

| Item                                                                                                       | Why                                                                                | Effort |
| ---------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- | ------ |
| Apple Developer ID signing plus notarization in CI                                                         | Removes Gatekeeper friction, real first-install trust on macOS, globally available | M      |
| Windows signing decision and implementation (Section 8)                                                    | SmartScreen friction, but the entity geography constrains the cheapest option      | M      |
| Scope `shell:allow-open` and `opener:default` capabilities to patterns                                     | Least-privilege depth behind the existing scheme allowlist                         | S      |
| Add Rust integration tests over the real git subprocess and the IPC layer                                  | The actual exec path is currently untested                                         | M      |
| Replace the shallow e2e with Tauri-shell-driven alias CRUD tests                                           | Current e2e passes silently when elements are missing                              | M      |
| Add a consent and disclosure surface for shell-history ranking, with an off switch                         | Undisclosed history reads fail enterprise privacy review                           | S      |
| Add cargo-vet or cargo-crev and gate on dependency review; pin release installs off the third-party mirror | Matures an already-strong supply chain                                             | M      |

### P2, make absorption-ready

| Item                                                                                          | Why                                                | Effort |
| --------------------------------------------------------------------------------------------- | -------------------------------------------------- | ------ |
| Restructure to `apps/gam/` layout mirroring `apps/desktop`, keep standalone crate             | The placement CyberOS expects                      | M      |
| Reconcile versioning to root VERSION, preserve history as a per-app changelog                 | Removes the manifest collision at merge time       | S      |
| Author FR-GAM-001..009 and NFR-GAM-001..002 with 10/10 audits, README, BACKLOG entry          | Nothing enters the CyberOS backlog without this    | L      |
| Move and re-baseline `.awh/`, add `audit-profile.yaml`, extend the gate hooks to scan `apps/` | Wires gam into the testing-to-done gate            | M      |
| Add the `gam-gate.yml` macOS CI workflow                                                      | The existing Linux gate cannot run gam's toolchain | M      |
| Relocate `.agent/` and `.claude/` under `apps/gam/`, repoint to vendored `tools/awh`          | Avoids shadowing CUO and the root config           | S      |

### P3, expand to team and enterprise

| Item                                                                   | Why                                                  | Effort |
| ---------------------------------------------------------------------- | ---------------------------------------------------- | ------ |
| Config governance: shared alias and config packs distributed to a team | The market gap no mainstream GUI fills               | L      |
| Policy profiles expressed as includeIf and hasconfig rules             | Standardize identity and per-project config at scale | L      |
| Adoption audit: show configured-versus-standard on a machine           | Makes governance verifiable, an enterprise must-have | M      |
| Internationalization framework, English plus Vietnamese first          | Global rollout and CyberSkill's home market          | L      |
| Automated a11y gate (axe in CI) and accessibility pass                 | Enterprise procurement and quality floor             | M      |
| Harden import and export (signing or checksums for shared packs)       | Shared config becomes a trust surface                | M      |

### P4, reach and scale

| Item                                                          | Why                                                                | Effort |
| ------------------------------------------------------------- | ------------------------------------------------------------------ | ------ |
| Publish to winget, Scoop, Flatpak, AUR, Snap (post-signing)   | Real distribution beyond Homebrew and direct download              | M      |
| Opt-in, privacy-respecting telemetry to inform priorities     | Currently flying blind on usage; must be opt-in to fit NFR-GAM-001 | M      |
| Central or MDM-friendly management for fleets                 | The enterprise distribution story for governed config              | L      |
| Position within CyberOS as an Everything-as-a-Service surface | Aligns gam with the platform's commercial direction                | L      |

---

## 8. Decisions the user needs to make

1. Version reconciliation. Conform gam's 1.0.11 to the CyberOS root 0.1.x line on absorption (matches the `apps/desktop` precedent, but is a visible downgrade for an app with real released users), or extend the release script to let `apps/` keep a higher line. Recommendation: conform, preserve history in a per-app changelog.
2. Windows signing entity. Azure Trusted Signing is the cheapest path but is geography-limited and a Vietnam-registered entity may not qualify. Options: sign macOS now (global) and defer Windows, use a traditional OV certificate for Windows (geography-independent, 200 to 400 USD/yr), or sign Windows through a qualifying CyberSkill entity in an eligible region. This needs a business answer, not just an engineering one.
3. Telemetry stance. NFR-GAM-001 currently promises no telemetry beyond the update check. Any future analytics must be opt-in to keep that promise. Confirm the stance before P4.
4. The `.agent/` toolkit. Vendor it under `apps/gam/` or drop it on absorption. It is gitignored today and will not travel by default.
5. Scope guard for the team capability. Config governance (P3) is the highest-value expansion but also the largest. Decide whether gam owns it, or whether it becomes a CyberOS module that gam is the front end for.

---

## 9. Recommended first slice

If the user approves moving from plan to build, the first executable slice is the P0 block plus the first two P1 security items, run on a dedicated `auto/gam-absorb` branch under the awh gate:

1. Rotate the signing key and purge the secret.
2. Re-baseline the awh gate green on a Mac.
3. Make the harness hook paths portable.
4. Fix the doc and config drift.
5. Add Apple Developer ID notarization to CI.

That slice removes the one high-severity security risk, makes the evidence gate trustworthy, and delivers the single biggest trust improvement (macOS notarization) before any larger feature or absorption work begins. Everything after it runs through the same plan, create, test, gate loop.

---

## Sources

- [Security, Tauri v2](https://v2.tauri.app/security/)
- [Capabilities, Tauri v2](https://v2.tauri.app/security/capabilities/)
- [Content Security Policy, Tauri v2](https://v2.tauri.app/security/csp/)
- [Updater plugin, Tauri v2](https://v2.tauri.app/plugin/updater/)
- [Distribute, Tauri v2](https://v2.tauri.app/distribute/)
- [AUR distribution, Tauri v2](https://v2.tauri.app/distribute/aur/)
- [Flatpak distribution, Tauri v2](https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/distribute/flatpak.mdx)
- [Code signing options for Windows app developers, Microsoft Learn](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options)
- [Azure Artifact Signing (formerly Trusted Signing)](https://azure.microsoft.com/en-us/products/artifact-signing)
- [Code signing Windows apps with Azure Artifact service, devclass](https://www.devclass.com/security/2026/01/14/code-signing-windows-apps-may-be-easier-and-more-secure-with-new-azure-artifact-service/4079554)
- [Code signing on Windows with Azure Trusted Signing, Melatonin](https://melatonin.dev/blog/code-signing-on-windows-with-azure-trusted-signing/)
- [OV vs EV code signing guide, SSL Insights](https://sslinsights.com/best-code-signing-certificate-windows-applications/)
- [git-config documentation](https://git-scm.com/docs/git-config)
- [Directory-based git identity and SSH config](https://github.com/gitmpr/scalable-git-ssh-config)
- [Dotfiles, git config with aliases, DEV](https://dev.to/michaelcurrin/dotfiles-git-config-348o)
- [Best Git GUI clients in 2025, DEV](https://dev.to/_d7eb1c1703182e3ce1782/best-git-gui-clients-in-2025-gitkraken-sourcetree-fork-and-more-compared-4gjd)
- [Git GUI clients, git-scm](https://git-scm.com/tools/guis)
