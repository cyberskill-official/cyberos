# CHANGELOG — CyberOS deltas vs. upstream Mattermost

This changelog tracks every change CyberOS applies on top of the pinned
upstream Mattermost commit. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

**Every PR touching `services/chat/` MUST add a line here under
`## [unreleased]` with one of the four category prefixes:**

- `feature` — CyberOS-only feature (not from upstream).
- `bug-fix` — CyberOS-only bug fix.
- `security` — security hardening or upstream security cherry-pick.
- `license-cherry-pick` — cherry-pick from upstream that requires legal review.

CI gate `chat-cherry-pick-review.yml` refuses to merge any PR touching
`services/chat/patches/**` without the `legal-reviewed` label.

---

## [unreleased]

(No entries yet — the next PR appends here.)

---

## [0.1.0] — 2026-05-19

### feature
- Initial fork pin established. `PINNED_COMMIT` carries the SHA of the last
  upstream commit dated BEFORE the relicense to non-Apache terms. No
  CyberOS patches in the series yet.

### security
- Distroless runtime base image (`gcr.io/distroless/base-debian12`); the
  container has no shell or package manager, reducing post-exploit surface.
- Non-root execution by default (UID 65532 / `nobody`).

### bug-fix
- (none.)

### license-cherry-pick
- (none.)

---

## How to add an entry

1. PR opens. Author adds a line under `## [unreleased]` with the right
   category prefix.
2. When the next CyberOS patch version ships, the `unreleased` heading is
   renamed to `## [<version>] — <date>` and a fresh `## [unreleased]` is
   started.
3. Version bumps live in `CYBEROS_PATCH_VERSION` at this folder's root.
   `CYBEROS_PATCH_VERSION` follows semver (`MAJOR.MINOR.PATCH`).
