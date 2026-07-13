# How consumers get CyberOS updates

## Model

| Layer | What it is | Version file |
|-------|------------|--------------|
| **CyberOS platform** | Payload under `.cyberos/` (gates, migrate, status hub, skills) | `.cyberos/VERSION` (from release `VERSION`) |
| **Product app** | The repo’s own software (Strategem, shopass, …) | `package.json` `version` and/or root `VERSION` if the product defines one |

These are **independent**. A product at `0.1.0` can run CyberOS `1.0.0`.

## Publish path (maintainers)

1. Bump `VERSION` (semver) when the platform changes.
2. `bash tools/cyberos-init/build.sh` → `dist/cyberos/`.
3. `git tag vX.Y.Z && git push origin vX.Y.Z` (or re-tag with care).
4. CI / `release-assets.sh` uploads **`cyberos-payload.tar.gz`** to GitHub Releases.
5. Optional: Claude marketplace / plugin zip from the same payload.

**Platforms do not magically pull every tag.** Consumers must:

- **Local clone with payload path:** re-run `bash dist/cyberos/init.sh <repo>` or `init.sh --check` then init when `repo_stale`.
- **Download release:**  
  `curl -fsSL https://github.com/cyberskill-official/cyberos/releases/latest/download/cyberos-payload.tar.gz | tar -xz -C /tmp && bash /tmp/cyberos/init.sh <repo>`
- **Agent `/update` command:** runs `init.sh --check` and applies the `next:` line.

### Auto-update options

| Channel | Auto? | How |
|---------|-------|-----|
| GitHub Release asset | Manual or scheduled | Fleet: `rollout-fleet.sh` / `fleet-init-test.sh` on CI cron |
| Local pre-push / agent | Semi | `init.sh --check` in CI; fail if `verdict=repo_stale` |
| Claude marketplace | After publish | Re-publish marketplace entry pointing at new release |
| npm/npx CLI | If published | Bump `cli/package.json` with platform VERSION on release |

Recommended CI on **product** repos (optional job):

```yaml
- name: CyberOS freshness
  run: |
    bash .cyberos/init.sh --check . || true
    # or fail on repo_stale:
    # bash path/to/payload/init.sh --check . | tee /tmp/c.txt
    # grep -q 'verdict=up_to_date' /tmp/c.txt
```

## Local detection

```bash
# From any consumer repo (needs a payload on disk or release extract)
bash /path/to/cyberos/dist/cyberos/init.sh --check .

# installed=…  payload=…  latest=…  verdict=up_to_date|repo_stale|payload_stale
```

- **repo_stale** → re-run full `init.sh <repo>` to refresh `.cyberos/`.
- **payload_stale** → update the payload (git pull cyberos + build, or download latest tarball), then init.

## Combined migrate

Prefer one entry:

| Command | Effect |
|---------|--------|
| `bash init.sh <repo>` | Vendor + full FR migrate + status page |
| `bash init.sh --page <repo>` | Status page only (pre-commit / run-gates) |
| `bash init.sh --migrate <repo>` | Full migrate without re-vendoring |
| `bash migrate-frs.sh --page` | **Shim** → `init.sh --page` (kept for back-compat) |

Do **not** remove the migrate kit: `--page` is permanent for status freshness.
