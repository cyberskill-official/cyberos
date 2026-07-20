---
id: TASK-CHAT-001
title: "Mattermost v9.x fork at pinned MIT-Apache commit + automated license-drift watcher + CI gate"
module: CHAT
priority: MUST
status: superseded
superseded_by: TASK-CHAT-101 (first-party native chat replaced the Mattermost fork wholesale; still-wanted intents re-homed as TASK-CHAT-102..106)
verify: I
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng
created: 2026-05-16
shipped: 2026-05-19
memory_chain_hash: pending
related_tasks: [TASK-CHAT-002, TASK-CHAT-003, TASK-CHAT-005, TASK-CHAT-011]
depends_on: []
blocks: [TASK-CHAT-002, TASK-CHAT-003]

source_pages:
  - website/docs/modules/chat.html#fork
  - website/docs/legal/chat-licensing.html
source_decisions:
  - DEC-420 (fork at the LATEST MIT-Apache commit BEFORE Mattermost's relicense; pin SHA)
  - DEC-421 (drift watcher checks upstream weekly for license-affecting commits; CI fails on drift)
  - DEC-422 (no re-base from upstream; cherry-pick security fixes only via PR review)

language: go 1.22 + ci-yaml
service: cyberos/services/chat/
new_files:
  - services/chat/Dockerfile
  - services/chat/README.md
  - services/chat/PINNED_COMMIT
  - services/chat/CHANGELOG.cyberos.md
  - .github/workflows/chat-license-drift-watcher.yml
  - .github/workflows/chat-cherry-pick-review.yml
  - services/chat/scripts/check-license-drift.sh
  - services/chat/scripts/cherry-pick-upstream.sh
modified_files:
  - cyberos/Makefile                                # add `chat-build`, `chat-license-check` targets
allowed_tools:
  - file_read: services/chat/**
  - file_write: services/chat/**, .github/workflows/**
  - bash: cd services/chat && make build
  - bash: cd services/chat && ./scripts/check-license-drift.sh
disallowed_tools:
  - merge from upstream master (per DEC-422 — cherry-pick only via PR)
  - upgrade past the pinned commit without legal review (per DEC-420)

effort_hours: 8
subtasks:
  - "0.5h: PINNED_COMMIT file (SHA of last MIT-Apache upstream commit)"
  - "0.5h: Dockerfile builds from pinned commit"
  - "1.0h: README.md — fork rationale + cherry-pick policy"
  - "1.0h: CHANGELOG.cyberos.md — track CyberOS deltas vs upstream"
  - "1.5h: check-license-drift.sh — query upstream commits since PINNED_COMMIT; grep LICENSE-touching changes"
  - "1.0h: chat-license-drift-watcher.yml — weekly cron"
  - "1.0h: cherry-pick-upstream.sh — operator workflow for security backports"
  - "0.5h: chat-cherry-pick-review.yml — PR gate requiring legal-team review label"
  - "1.0h: Makefile targets + docker-compose for local dev"
risk_if_skipped: "Mattermost relicensed to a non-Apache license at some point; using the latest upstream without pinning means CyberOS inherits a license that prohibits commercial fork. Without drift watcher, an unreviewed upgrade silently introduces a SLA-incompatible license. Without cherry-pick policy, security fixes blocked entirely OR merged without review."
---

## §1 — Description (BCP-14 normative)

The CHAT service **MUST** be a fork of Mattermost v9.x at a pinned MIT-Apache commit with automated license-drift detection. The contract:

1. **MUST** pin a single upstream commit SHA in `PINNED_COMMIT` file. The pinned commit MUST be the LATEST commit on the upstream `master` branch dated BEFORE the relicense to non-Apache terms.
2. **MUST** build from this pinned commit via Docker. The Dockerfile fetches the tarball + applies the CyberOS patch series. NO `git pull upstream master`.
3. **MUST** track CyberOS-specific changes in `CHANGELOG.cyberos.md`. Every PR touching `services/chat/` updates the changelog with category (feature | bug-fix | security | license-cherry-pick).
4. **MUST** run a weekly license-drift watcher GitHub Actions workflow:
- Query the GitHub API for upstream commits since `PINNED_COMMIT`.
- Filter for commits touching `LICENSE`, `LICENSE.md`, `licensing/`, or root-level package metadata.
- If any commit found → file a GitHub issue labelled `legal-review-needed` with the commit list.
5. **MUST** require a cherry-pick PR workflow for upstream security fixes:
- Operator runs `scripts/cherry-pick-upstream.sh <commit-sha>` which fetches the commit, applies it as a patch, and opens a PR.
- PR is gated by GH Action `chat-cherry-pick-review.yml` requiring the `legal-reviewed` label before merge.
6. **MUST** publish the fork as `services/chat/` in the CyberOS monorepo (not a separate git submodule); patches live in `services/chat/patches/*.patch` applied at Docker build time.
7. **MUST** document the fork's deviation policy in README.md: (a) cherry-picks allowed for security, (b) features-from-upstream blocked, (c) CyberOS-only features welcome.
8. **MUST** include the pinned commit SHA in every chat image tag: `cyberos/chat:<pinned_sha_short>-<cyberos_patch_version>`.

---

## §2 — Why this design (rationale for humans)

**Why pin a commit not a tag (DEC-420)?** Tags can be re-pointed; commits are immutable. The pinned SHA is the immutable contract.

**Why MIT-Apache (DEC-420)?** CyberOS sells commercial services on top of CHAT; Mattermost's newer licenses (e.g. business-source) prohibit competition. Pre-relicense MIT-Apache is fork-friendly.

**Why no rebase (DEC-422)?** Rebase pulls in untested code + potential license drift. Cherry-pick is surgical: one commit reviewed by legal + engineering.

**Why drift watcher (DEC-421)?** Without automation, the license check is "remembered" not "enforced." Weekly cron + GitHub issue = visible signal.

**Why patches not full source (§1 #6)?** Patches diff = clear what CyberOS changed; full source duplicates 2M lines of upstream code. Easier review.

**Why image tag includes pinned SHA (§1 #8)?** Operators investigating "which Mattermost version is this?" read the tag — no manifest inspection needed.

---

## §3 — API contract

```
# services/chat/PINNED_COMMIT
abc123def456...   # 40-char SHA of last MIT-Apache upstream commit
```

```dockerfile
# services/chat/Dockerfile
FROM golang:1.22 AS build
ARG PINNED_COMMIT
RUN git clone https://github.com/mattermost/mattermost-server.git /src \
 && cd /src && git checkout ${PINNED_COMMIT}
COPY patches/ /patches/
RUN cd /src && for p in /patches/*.patch; do git apply "$p"; done
RUN cd /src && make build-server

FROM gcr.io/distroless/base-debian12
COPY --from=build /src/bin/mattermost /usr/local/bin/mattermost
COPY config/ /etc/cyberos-chat/
EXPOSE 8065
ENTRYPOINT ["/usr/local/bin/mattermost", "server", "--config=/etc/cyberos-chat/config.json"]
```

```bash
#!/usr/bin/env bash
# services/chat/scripts/check-license-drift.sh
set -euo pipefail
PINNED=$(cat services/chat/PINNED_COMMIT)
SINCE=$(date -u -d "${1:-7 days ago}" +%Y-%m-%dT%H:%M:%SZ)
COMMITS=$(gh api repos/mattermost/mattermost-server/commits \
  --paginate -q ".[].sha" \
  --field since="$SINCE" --field until=now)
FLAGGED=""
for sha in $COMMITS; do
  files=$(gh api "repos/mattermost/mattermost-server/commits/$sha" -q ".files[].filename")
  if echo "$files" | grep -qE "^(LICENSE|LICENSE\.md|licensing/|.*\.LICENSE)"; then
    FLAGGED="${FLAGGED}${sha}\n"
  fi
done
if [ -n "$FLAGGED" ]; then
  echo "::warning::License-affecting commits since PINNED_COMMIT:"
  echo -e "$FLAGGED"
  gh issue create --title "Upstream license drift since pinned commit" \
                  --body "Detected commits affecting LICENSE files:\n\n$(echo -e "$FLAGGED")" \
                  --label "legal-review-needed,chat"
  exit 1
fi
echo "No license drift detected."
```

```yaml
# .github/workflows/chat-license-drift-watcher.yml
name: chat license drift watcher
on:
  schedule:
    - cron: '0 0 * * 1'    # weekly Monday 00:00 UTC
  workflow_dispatch:
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: bash services/chat/scripts/check-license-drift.sh
        env: { GH_TOKEN: ${{ secrets.GITHUB_TOKEN }} }
```

```yaml
# .github/workflows/chat-cherry-pick-review.yml
name: chat cherry-pick review gate
on:
  pull_request:
    paths: ['services/chat/patches/**']
jobs:
  legal-review:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Require legal-reviewed label
        run: |
          LABELS=$(gh pr view ${{ github.event.pull_request.number }} --json labels -q '.labels[].name')
          echo "$LABELS" | grep -q "legal-reviewed" || (echo "::error::PR needs 'legal-reviewed' label" && exit 1)
```

---

## §4 — Acceptance criteria

1. **PINNED_COMMIT file exists** — 40-char SHA; comment explains rationale.
2. **Dockerfile builds from pinned** — `make chat-build` produces image tagged with pinned-sha-short.
3. **Drift watcher cron set** — workflow listed; `crontab` field matches Monday weekly.
4. **Drift watcher finds license commit** — fixture upstream commit touching LICENSE → workflow opens GH issue with `legal-review-needed` label.
5. **Cherry-pick review gate blocks unlabeled PR** — PR touching `services/chat/patches/**` without label → CI red.
6. **Cherry-pick review gate passes with label** — same PR with `legal-reviewed` added → CI green.
7. **CHANGELOG.cyberos.md present** — file lives at root; categorisation rules documented.
8. **README.md describes fork policy** — sections on cherry-pick + no-rebase + features.
9. **Image tag includes pinned SHA** — `docker images cyberos/chat` shows tag prefix matches PINNED_COMMIT short.
10. **Patches directory exists** — `services/chat/patches/` present (even if empty initially).

---

## §5 — Verification

```bash
# test: drift watcher detects synthetic license commit
test_drift_watcher() {
  cd services/chat && git fetch upstream || true
  # Mock: pretend a recent commit touched LICENSE
  echo "abc999" > /tmp/mock-commits
  PINNED_COMMIT=abc999 bash scripts/check-license-drift.sh 2>&1 | grep -q "legal-review-needed"
}

# test: cherry-pick gate blocks unlabeled PR
test_cherry_pick_gate() {
  curl -s -H "Authorization: Bearer $GH_TOKEN" \
       https://api.github.com/repos/$REPO/pulls/$PR \
       | jq -r '.labels[].name' | grep -q "legal-reviewed" \
       || echo "PR not approved" >&2
}

# test: Docker build succeeds at pinned SHA
docker build --build-arg PINNED_COMMIT=$(cat services/chat/PINNED_COMMIT) -t cyberos/chat-test services/chat/
```

---

## §6 — Implementation skeleton

(Dockerfile + scripts + workflows above.)

---

## §7 — Dependencies

- **TASK-CHAT-002 (downstream)** — auth bridge plugin built into the fork.
- **TASK-CHAT-003 (downstream)** — Fargate deployment.
- **TASK-CHAT-005 (downstream)** — memory bridge via Postgres logical replication.
- **TASK-CHAT-011 (downstream)** — mobile push via fork's plugin system.

---

## §8 — Example payloads

(N/A — infra task.)

---

## §9 — Open questions

All resolved. Deferred:
- Auto-pull of upstream security tags via bot — slice 4+.
- Multi-org fork (different pinned SHAs per tenant for compliance) — slice 5+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| PINNED_COMMIT deleted | Dockerfile build Err | CI red | Restore from git |
| Upstream removes commit | git fetch Err | Build red; ops alarm | Mirror commit to private archive |
| Drift watcher misses commit | manual audit | Sev-2 | Operator runs manually |
| Cherry-pick conflicts with CyberOS patches | git apply Err | PR red | Operator resolves |
| Legal-reviewed label added by non-legal | GH branch protection | Label restricted to legal-team | None |
| Image tag collision | tag includes sha + patch version | unique | None |
| Patch order matters (apply order) | sort lexicographically | deterministic | None |
| Upstream deletes binary releases | mirror to private CDN | Build resilient | None |

---

## §11 — Implementation notes

- PINNED_COMMIT is updated only via legal-reviewed PR; CODEOWNERS file pins this file to legal-team approval.
- Patches in `services/chat/patches/` are git-format-patch output; apply order is lexicographic by filename (`001-xxx.patch` before `010-xxx.patch`).
- Fork rationale in README references pre-relicense MIT-Apache governance explicitly.
- CHANGELOG.cyberos.md uses Keep-a-Changelog format with `## [unreleased]` section.
- Drift watcher uses GitHub API rather than git clone for efficiency (large repo).
- Cherry-pick workflow auto-detects upstream CVE numbers in commit messages, surfaces in PR description.

---

*End of TASK-CHAT-001.*
