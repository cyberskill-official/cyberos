# CyberOS CHAT — Mattermost fork at pinned MIT-Apache commit

**Status:** FR-CHAT-001 + FR-CHAT-002 shipped as service slices. FR-CHAT-001 pins the fork, license-drift watcher, and cherry-pick policy; FR-CHAT-002 adds the CyberOS AuthBridge plugin scaffold for AUTH JWT login and tenant propagation.
**Upstream:** [`mattermost/mattermost-server`](https://github.com/mattermost/mattermost-server)
**Pinned commit:** see [`PINNED_COMMIT`](PINNED_COMMIT)
**CyberOS patch version:** see [`CYBEROS_PATCH_VERSION`](CYBEROS_PATCH_VERSION)
**License posture:** the pinned upstream commit is dated BEFORE Mattermost's relicense to non-Apache terms; the upstream tree at that SHA carries MIT + Apache-2.0. CyberOS patches in [`patches/`](patches/) are Apache-2.0.

---

## §1 — Why a fork, why pinned, why patches only

CyberOS sells commercial services on top of the chat surface. Mattermost's
post-relicense terms (business-source-style) prohibit competing commercial
offerings; the pre-relicense MIT + Apache-2.0 commit is the last point at
which a commercial fork is permitted.

We pin a single SHA rather than a tag because tags can be re-pointed (the
upstream maintainer can move `v9.x` to a different commit). The SHA is
immutable.

We apply changes as patches in [`patches/`](patches/) at Docker build time,
not as commits in this repository. The diff between the pinned upstream and
the patched build is therefore obvious from the patch directory. We do not
vendor the 2M-line upstream source.

---

## §2 — Fork deviation policy

| Change category | Allowed? | Pathway |
|---|---|---|
| Upstream **security** cherry-pick | ✅ Yes | `scripts/cherry-pick-upstream.sh <sha>` opens a PR; PR requires `legal-reviewed` label per `.github/workflows/chat-cherry-pick-review.yml` |
| Upstream **feature** cherry-pick | ❌ No | Features from post-relicense upstream commits are inadmissible. If a feature is desired, re-implement as a CyberOS-only patch. |
| Upstream **bug fix** cherry-pick | ⚠️ Case-by-case | Allowed if the fix is in a file untouched by license changes and is genuinely a bug fix not a feature. Same legal-reviewed PR gate. |
| **CyberOS-only** patches | ✅ Yes | Drop a `NNN-name.patch` file into [`patches/`](patches/); the Dockerfile applies in lexicographic order. Patches are Apache-2.0. |
| **Rebase from upstream master** | ❌ Forbidden | Per DEC-422. Rebase pulls in untested code AND potential license drift. Cherry-pick only via reviewed PR. |
| **Update `PINNED_COMMIT`** | ⚠️ Legal-team only | CODEOWNERS pins this file to the legal-team approver list. Replacing the pinned SHA requires explicit confirmation that the new SHA is still on the pre-relicense branch (or a fresh DEC entry switching posture). |

---

## §3 — Cherry-pick workflow

```bash
# 1. Identify the upstream security commit (CVE etc).
UPSTREAM_SHA=abc123...

# 2. Run the cherry-pick helper. This fetches the commit from upstream,
#    extracts it as a patch, opens a PR with the CVE/security context.
./scripts/cherry-pick-upstream.sh $UPSTREAM_SHA

# 3. Legal review the PR. When approved, add the `legal-reviewed` label.
#    The `chat-cherry-pick-review.yml` workflow refuses to merge without it.

# 4. Operator merges the PR. The patch file lands in services/chat/patches/.
#    Next Docker build picks it up automatically.
```

---

## §4 — License drift watcher

A scheduled GitHub Actions workflow (`chat-license-drift-watcher.yml`) runs
every Monday 00:00 UTC. It:

1. Reads the pinned SHA from `PINNED_COMMIT`.
2. Queries the upstream `mattermost-server` repository for commits since
   that SHA.
3. Filters for commits touching `LICENSE`, `LICENSE.md`, files under
   `licensing/`, or root-level package metadata.
4. If any drift is detected → files a GitHub issue with the
   `legal-review-needed` and `chat` labels.

The workflow also runs on `workflow_dispatch` so operators can trigger a
fresh scan on demand.

---

## §5 — Build + run

```bash
# Build the image from the pinned commit + patch series.
make chat-build

# Build with explicit verification of the drift watcher state.
make chat-license-check && make chat-build

# Run locally (Postgres + Redis required; see compose.yml at repo root).
docker compose up cyberos-chat
```

The image tag includes the pinned SHA (12-char) and CyberOS patch version:

```
cyberos/chat:cf5fa5a2bb14-0.1.0
```

The tag is the **only** identifier operators need to determine what is
running. No manifest inspection necessary.

---

## §6 — CHANGELOG

CyberOS-specific changes are tracked in [`CHANGELOG.cyberos.md`](CHANGELOG.cyberos.md)
using Keep-a-Changelog format. Every PR touching `services/chat/` MUST
update the changelog with a category line — `feature`, `bug-fix`,
`security`, or `license-cherry-pick`.

---

## §7 — Layout

```
services/chat/
├── PINNED_COMMIT                       40-char SHA + rationale block
├── CYBEROS_PATCH_VERSION               semver of the CyberOS patch series
├── Dockerfile                          two-stage build; distroless runtime
├── README.md                           this file
├── CHANGELOG.cyberos.md                Keep-a-Changelog of CyberOS deltas
├── Makefile                            chat-build, chat-license-check targets
├── compose.yml                         local-dev compose (Postgres + Redis)
├── config/
│   └── config.json                     default config baked into image
├── patches/
│   ├── 010-disable-builtin-auth.patch   route password auth to AuthBridge
│   └── 011-load-authbridge-plugin.patch package AuthBridge at boot
├── plugins/
│   └── cyberos-authbridge/              FR-CHAT-002 plugin source + tests
├── scripts/
│   ├── check-license-drift.sh          drift watcher entry point
│   └── cherry-pick-upstream.sh         operator cherry-pick helper
└── tests/
    ├── license_drift_test.sh           §5 test: drift detector finds license commits
    ├── pinned_commit_test.sh           §5 test: SHA is 40-char hex
    └── patch_apply_test.sh             §5 test: patch series applies cleanly
```

---

## §8 — References

- DEC-420 — fork at LATEST MIT-Apache commit; pin SHA.
- DEC-421 — drift watcher cron; CI fails on drift.
- DEC-422 — cherry-pick only via reviewed PR; no rebase.
- FR-CHAT-001 — feature request authoring this fork policy.
- FR-CHAT-002 — `cyberos-chat-authbridge` plugin: delegates Mattermost auth to AUTH JWTs.
- FR-CHAT-003 (downstream) — per-tenant Fargate deployment.
- FR-CHAT-005 (downstream) — memory bridge via Postgres logical replication.
- FR-CHAT-011 (downstream) — mobile push via the plugin system.
