# Secrets inventory and rotation runbook

**Task:** TASK-IMP-041  
**Rule:** This document lists **secret classes**, owners, and blast radius.
It never stores secret values. If a value appears in chat or logs, treat it as
compromised and rotate.

## Inventory

| Class | Where it lives | Used by | Owner | Blast radius if leaked | Rotation trigger |
|---|---|---|---|---|---|
| `GITHUB_TOKEN` (Actions default) | GitHub Actions runtime | `release.yml`, `deploy.yml`, packages:write | Platform eng | Push packages; limited by job permissions | Automatic per job; revoke via GitHub if job token exfiltrated |
| `VERSION_BUMP_SSH_KEY` | GitHub Actions secrets | `version.yml`, `deploy.yml` ruleset bypass push | CTO / release owner | Push to protected branches as deploy key identity | Key compromise; staff offboarding; annual |
| `VPS_HOST` / `VPS_USER` / `VPS_SSH_KEY` | GitHub Actions secrets | `deploy.yml` SSH roll | Platform eng | Full VPS shell as deploy user | Compromise; VPS rebuild; offboarding |
| GHCR pull credentials on VPS | VPS host config (not in repo) | `docker pull` on deploy | Platform eng | Pull private images | Compromise; rotate registry creds |
| VPS git deploy key (read-only) | VPS `authorized_keys` / GitHub deploy keys | `git pull` on deploy | Platform eng | Read repo contents | Key compromise; repo transfer |
| `APPLE_*` / MAS signing + ASC API | GitHub Actions secrets | `notarize.yml`, `release-mas.yml`, `release.yml` | Desktop release owner | Sign/notarize as CyberOS; App Store Connect API | Cert expiry; suspected leak; annual |
| `MSSTORE_*` EV cert + Azure app | GitHub Actions secrets | `release-msstore.yml` | Desktop release owner | Sign Store packages; MS partner center API | Cert expiry; client secret expiry; leak |
| `SNAPCRAFT_STORE_CREDENTIALS` | GitHub Actions secrets | `release-snap.yml` | Desktop release owner | Publish snaps | Credential expiry; leak |
| `MAXMIND_LICENSE_KEY` | GitHub Actions secrets | `services.yml` geo DB fetch | Platform eng | MaxMind account quota / ToS | Leak; MaxMind rotation mail |
| npm publish token (if used) | CI secret or local operator keyring — **confirm before use** | npm publish of install package | Release owner | Publish/overwrite package versions | Leak; staff change; 90-day hygiene |
| Consumer BRAIN / tenant store | Local `.cyberos/memory/store/` (gitignored) | Memory protocol | Repo operator | Tenant memory contents | Per tenant incident |
| OIDC / Google SSO client secrets | Deploy env / IdP console (platform) | Auth services | Security + platform | Account takeover for linked apps | Leak; quarterly IdP review |

## Rotate-on-leak procedure

1. **Contain** — revoke the credential at the source (GitHub secret delete,
   IdP disable, host `authorized_keys` remove, npm token revoke).
2. **Inventory** — tick every row above that shares the same backing principal
   (one SSH key may back both version bump and deploy).
3. **Replace** — mint new material; update GitHub Actions secrets / VPS /
   package registries; do not commit values.
4. **Verify** — re-run the smallest workflow that consumes the secret (e.g.
   dry deploy SSH, notarize on a test tag policy, `npm whoami`).
5. **Record** — append an incident note under `docs/batches/` with class names
   only (no values); open follow-up tasks if blast radius included customer data.

## Hygiene

- Prefer OIDC / short-lived tokens over long-lived PATs where GitHub supports it.
- Never paste secrets into task specs, ADRs, or BRAIN rows.
- After any gam / updater-key style leak: assume CI logs and chat transcripts
  that saw the value are hostile; rotate even if "maybe redacted".

## Pointers

- Release overview: `docs/deploy/RELEASE.md`
- Deploy workflow comments: `.github/workflows/deploy.yml`
- Payload vs platform: [ADR-003](../adrs/ADR-003-payload-vs-platform-scope.md)
