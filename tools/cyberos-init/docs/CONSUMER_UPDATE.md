# Consumer updates (CyberOS 1.0)

## Entry points (all trigger update-check)

Any use of a vendored `.cyberos/` entrypoint runs a soft update-check first
(`lib/update-check.sh`).

| Command | When |
|---------|------|
| `bash .cyberos/init.sh <repo>` | Full vendor + migrate |
| `bash .cyberos/init.sh --page <repo>` | Status page only |
| `bash .cyberos/init.sh --migrate <repo>` | FR layout + page |
| `bash .cyberos/init.sh --check <repo>` | Freshness report (read-only) |
| `bash .cyberos/update.sh` / `--apply` | Check / re-init |
| `bash .cyberos/changelog.sh` | Installed version + fingerprint |
| `bash .cyberos/help.sh` | CLI surface |
| `bash .cyberos/cuo/gates/run-gates.sh` | Every gates run |
| MCP `fr_init` / `fr_gates` / `fr_status` / `ship_fr` | Any MCP tool on the repo |
| pre-commit status hook | Via `init.sh --page` |

```bash
CYBEROS_UPDATE_CHECK=soft    # default: warn, throttle 12h
CYBEROS_UPDATE_CHECK=always  # every invocation
CYBEROS_UPDATE_CHECK=strict  # exit 1 if stale
CYBEROS_UPDATE_CHECK=0       # off
CYBEROS_PAYLOAD=/path/to/dist/cyberos  # compare against local payload
CYBEROS_OFFLINE=1             # skip network latest
```

## Apply latest

```bash
# from release
curl -fsSL https://github.com/cyberskill-official/cyberos/releases/latest/download/cyberos-payload.tar.gz \
  | tar -xz -C /tmp
bash /tmp/cyberos/init.sh /path/to/repo

# from monorepo build
bash ~/Projects/CyberSkill/cyberos/dist/cyberos/init.sh /path/to/repo

# or from installed payload
bash .cyberos/update.sh --apply
```

## Versions

| File | Meaning |
|------|---------|
| `.cyberos/VERSION` | **Platform** (CyberOS) |
| `package.json` version | **Product** (independent) |

## No migrate-frs.sh

There is no `migrate-frs.sh` shim (pre-1.0.0; never released as stable).
Use `init.sh --page` / `--migrate` only. Re-init removes any leftover shim.
