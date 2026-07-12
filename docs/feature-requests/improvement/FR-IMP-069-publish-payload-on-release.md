---
id: FR-IMP-069
title: "Publish the versioned payload + Claude plugin as GitHub Release assets on every vX.Y.Z tag"
module: improvement
priority: MUST
status: implementing
class: improvement
verify: T
phase: Wave A - version coupling
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: null
memory_chain_hash: null
related_frs: [FR-IMP-068, FR-IMP-070, FR-PLUGIN-008, FR-SKILL-201, FR-APP-001]
depends_on: [FR-IMP-068]
blocks: [FR-IMP-070]
source_pages:
  - .github/workflows/release.yml
  - tools/cyberos-init/README.md
  - tools/cyberos-init/bootstrap.sh
  - tools/cyberos-init/rollout.sh
  - docs/deploy/RELEASE.md
source_decisions:
  - "2026-07-12 operator decision: publish channel = GitHub Releases (payload tarball + cyberos.plugin per tag). npm package and hosted bootstrap remain future follow-ups on top of this channel."
  - "Scope boundary: FR-PLUGIN-008 (plugins.cyberskill.world OCI marketplace) and FR-SKILL-201 (.skill OCI registry) are separate product-distribution systems and are NOT superseded; this FR covers only the cyberos-init payload channel."
language: bash + GitHub Actions YAML
service: .github/workflows/ + tools/cyberos-init/
new_files:
  - tools/cyberos-init/release-assets.sh
  - tools/cyberos-init/tests/test_release_assets.sh
modified_files:
  - .github/workflows/release.yml
  - tools/cyberos-init/bootstrap.sh
  - tools/cyberos-init/rollout.sh
  - tools/cyberos-init/README.md
  - docs/deploy/RELEASE.md
---

# FR-IMP-069: Publish the payload on every release

## §1 - Description

Today the payload exists only as a gitignored local build; every consumer (Claude marketplace add, `.plugin` file pick, `init.sh`, `rollout.sh`) copies whatever a laptop last built. This FR gives the payload one canonical, versioned, downloadable source: the GitHub Release for each `vX.Y.Z` tag.

Normative clauses:

1. A script `tools/cyberos-init/release-assets.sh <payload-dir> <out-dir>` MUST produce, from a built payload: `cyberos-payload-<ver>.tar.gz` (deterministic: sorted paths, numeric owner 0:0, fixed mtime `2000-01-01T00:00:00Z`, gzip -n), `cyberos-<ver>.plugin` (byte-copy of `cyberos.plugin`), stable-name aliases `cyberos-payload.tar.gz` and `cyberos.plugin` (byte-identical copies, so `releases/latest/download/<stable-name>` always resolves), and `SHA256SUMS` covering all four. `<ver>` MUST be read from `<payload-dir>/VERSION`.
2. `release-assets.sh` MUST exit 10 without producing output when `<payload>/VERSION`, root `VERSION`, and (when `$GITHUB_REF_NAME` is set) the tag `v<ver>` do not all agree.
3. `.github/workflows/release.yml` MUST gain a `payload` job that, for every `v*` tag: checks out, runs `build.sh` into a temp dir, runs `check-version-sync.sh` (FR-IMP-068) against it, runs `release-assets.sh`, and uploads the four files + `SHA256SUMS` to that tag's GitHub Release. Default `GITHUB_TOKEN` with `contents: write` MUST suffice; no new secrets.
4. The `payload` job MUST fail when the pushed tag does not equal `v$(cat VERSION)` at that commit.
5. `bootstrap.sh` MUST support fetching the payload from a URL: `CYBEROS_PAYLOAD_URL` overrides; default = the repo's `releases/latest/download/cyberos-payload.tar.gz`. It MUST download `SHA256SUMS` from the same location, verify the tarball's checksum before unpacking, and then run `init.sh` from the unpacked payload. Verification failure MUST abort before any file lands in the target repo.
6. `rollout.sh` MUST accept, in place of a local payload dir, `--from-release [tag]`: download + verify once into a temp dir (per #5's mechanics), then proceed unchanged for every listed repo.
7. `tools/cyberos-init/README.md` and `docs/deploy/RELEASE.md` MUST document the release-install paths for each channel: Claude Code (`/plugin marketplace add` on the unpacked payload), Claude desktop/Cowork (download `cyberos.plugin` from the release), curl bootstrap one-liner, and fleet `rollout.sh --from-release`. The "available once you host a tarball" placeholder MUST be replaced by the real URLs.

## §2 - Why this design

GitHub Releases is the zero-infrastructure channel that matches the existing tag-driven `release.yml` and needs no hosting, keys, or registry. Deterministic tarballs mirror the memory module's export discipline and make the artifact reproducible from the tag alone. Stable-name aliases exist because GitHub's `latest/download/` URL requires a constant asset name; versioned twins keep every historical release independently fetchable. Checksums ship next to the assets so `bootstrap.sh` can verify without a signing infrastructure (cosign-grade signing stays with FR-PLUGIN-008's marketplace, which this FR deliberately does not replace).

## §3 - Contract

```
release-assets.sh <payload-dir> <out-dir>
  exit 0   wrote 5 files into <out-dir> (2 versioned, 2 stable, SHA256SUMS)
  exit 10  version disagreement (payload VERSION vs root VERSION vs $GITHUB_REF_NAME)
  exit 2   payload missing/incomplete (no VERSION or no cyberos.plugin)

bootstrap.sh                         # unchanged local behavior when a payload dir is passed
  CYBEROS_PAYLOAD_URL=<url>          # tarball URL; SHA256SUMS fetched from its dirname
  (no args, no local payload)       -> latest-release default URL

rollout.sh --from-release [vX.Y.Z] <repo> [<repo>...]
```

release.yml addition (shape):

```yaml
  payload:
    runs-on: ubuntu-latest
    permissions: { contents: write }
    steps:
      - uses: actions/checkout@v4
      - run: test "v$(cat VERSION)" = "$GITHUB_REF_NAME"
      - run: bash tools/cyberos-init/build.sh "$RUNNER_TEMP/payload"
      - run: bash tools/cyberos-init/check-version-sync.sh "$RUNNER_TEMP/payload"
      - run: bash tools/cyberos-init/release-assets.sh "$RUNNER_TEMP/payload" "$RUNNER_TEMP/assets"
      - run: gh release upload "$GITHUB_REF_NAME" "$RUNNER_TEMP"/assets/* --clobber
        env: { GH_TOKEN: "${{ github.token }}" }
```

## §4 - Acceptance criteria

1. **Deterministic tarball** (§1 #1) - running `release-assets.sh` twice on the same payload yields byte-identical `cyberos-payload-<ver>.tar.gz` (equal sha256).
2. **All five files, both name forms** (§1 #1) - output contains versioned + stable tarball and plugin names plus `SHA256SUMS`; stable and versioned twins are byte-identical.
3. **Checksums verify** (§1 #1) - `sha256sum -c SHA256SUMS` passes in the out-dir; corrupting one byte of the tarball makes it fail.
4. **Version triple-check** (§1 #2) - payload 1.7.0 + root 1.7.0 + `GITHUB_REF_NAME=v1.6.0` exits 10 and writes nothing.
5. **Tag guard in CI** (§1 #3, #4) - the `payload` job's first step compares `v$(cat VERSION)` to the tag; the job uploads exactly the five §1 #1 files with `--clobber`.
6. **bootstrap URL flow** (§1 #5) - with `CYBEROS_PAYLOAD_URL=file://<fixture>/cyberos-payload.tar.gz` and a matching `SHA256SUMS`, bootstrap downloads, verifies, unpacks, and runs `init.sh` against the target repo (`.cyberos/VERSION` appears).
7. **bootstrap rejects a bad checksum** (§1 #5) - with a tampered fixture tarball, bootstrap aborts before touching the target repo (no `.cyberos/` created).
8. **rollout from a release source** (§1 #6) - `rollout.sh --from-release` against the file:// fixture initializes two scratch repos from one download (fixture served once; second repo reuses the temp payload).
9. **Docs list the four install paths with real URLs** (§1 #7) - README and RELEASE.md contain the `releases/latest/download/` URLs and the placeholder sentence is gone.

## §5 - Verification

```bash
# tools/cyberos-init/tests/test_release_assets.sh
# Fixtures: scratch payload built by build.sh into $TMP; file:// URLs exercise the
# download paths without network. Run: bash tools/cyberos-init/tests/test_release_assets.sh

t01_deterministic_tarball()      # AC 1
t02_five_files_two_name_forms()  # AC 2
t03_sha256sums_roundtrip()       # AC 3
t04_version_triple_check()       # AC 4  (GITHUB_REF_NAME mismatch -> exit 10, empty out-dir)
t05_workflow_shape()             # AC 5  (structural greps on release.yml: tag guard, upload step, --clobber)
t06_bootstrap_url_happy_path()   # AC 6
t07_bootstrap_bad_checksum()     # AC 7
t08_rollout_from_release()       # AC 8
t09_docs_real_urls()             # AC 9
```

## §6 - Implementation skeleton

`release-assets.sh`: version guard; `tar --sort=name --owner=0 --group=0 --mtime='2000-01-01 00:00:00Z' -cf - -C <payload> . | gzip -n`; copies; `sha256sum > SHA256SUMS`. The upload step MUST be create-or-upload idempotent: `gh release create "$GITHUB_REF_NAME" --verify-tag --notes-from-tag 2>/dev/null || true` before `gh release upload ... --clobber`, so the payload job never races the installer jobs on release creation. bootstrap: `curl -sfL` both files, `sha256sum -c --ignore-missing`, `tar -xzf` into mktemp, exec `init.sh`. rollout: argument branch that materializes the payload then falls through to the existing loop.

## §7 - Dependencies

Depends on FR-IMP-068 (`check-version-sync.sh` is the pre-upload gate). Blocks FR-IMP-070 (remote update check needs a published "latest" to compare against). Related: FR-PLUGIN-008 / FR-SKILL-201 stay the long-term product registries; FR-APP-001's Ops tab can later add a "download latest release" affordance on top of this channel.

## §8 - Example payloads

```
$ ls assets/
SHA256SUMS  cyberos-1.7.0.plugin  cyberos-payload-1.7.0.tar.gz  cyberos-payload.tar.gz  cyberos.plugin
$ curl -sfL https://github.com/cyberskill-official/cyberos/releases/latest/download/cyberos-payload.tar.gz -o p.tgz
```

## §9 - Open questions

None blocking. Whether to ALSO publish on every VERSION bump (not just tags) is deferred: tags stay the human release act per version.yml's "auto version, manual release" model.

## §10 - Failure modes inventory

1. Tag pushed at a commit whose VERSION lags (bot bump not yet merged) - the tag guard (§1 #4) fails the job with a one-line explanation instead of publishing mismatched assets.
2. Re-running a release (re-tag / workflow re-run) - `--clobber` makes uploads idempotent; determinism (#1) makes the re-uploaded bytes identical.
3. Partial upload (network drop mid-job) - assets are uploaded in one `gh release upload` invocation; a failed job leaves the release without `SHA256SUMS` at worst, and bootstrap refuses to install anything it cannot verify.
4. GitHub API rate-limited during bootstrap - curl fails, bootstrap aborts cleanly with the URL in the error; no partial `.cyberos/`.
5. A consumer pins `latest` but needs an old version - versioned asset names keep every release addressable; README documents the pinned form.

## §11 - Implementation notes

Keep asset names exactly as specified; FR-IMP-070 and future npm packaging key off them. The tarball packs the payload CONTENTS at archive root (unpack -> `init.sh` at top level), matching what `bootstrap.sh` expects from a local payload dir. Determinism flags (`--sort`, `--owner`, `--mtime`, `gzip -n`) are GNU tar/gzip semantics: the CI job runs on ubuntu; the test suite MUST skip the determinism case with a visible SKIP on non-GNU (macOS bsdtar) hosts rather than false-fail.

*End of FR-IMP-069.*
