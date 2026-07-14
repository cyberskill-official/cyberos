# Consumer install / update (CyberOS 1.0)

## Command model

| Command | When | Role |
|---------|------|------|
| `bash install.sh [repo]` | Once (or re-vendor) | Install machine into `.cyberos/` |
| `bash uninstall.sh [repo]` | On demand | Remove machine (keeps FRs; BRAIN kept by default) |
| `bash update.sh` | Manual anytime | Check installed / payload / latest |
| `bash update.sh --apply` | Manual when stale | Re-run install from payload |
| `bash status.sh` | Manual only | Version + rules_sha report |
| Soft check | **Auto** on any `.cyberos` use | `lib/update-check.sh` (throttled 12h) |

There is **no** user-facing `install --page` or `install --check`.
Status page regen is internal: `lib/status-page.sh` (pre-commit + run-gates).

## Soft update triggers (automatic)

- `cuo/gates/run-gates.sh`
- `lib/status-page.sh` (hooks)
- `help.sh`, `status.sh`, `update.sh`
- MCP `fr_install` / `fr_gates` / `fr_status` / `ship_fr`
- Full `install.sh` (always)

```bash
CYBEROS_UPDATE_CHECK=soft    # default: warn, throttle 12h
CYBEROS_UPDATE_CHECK=always  # every invocation
CYBEROS_UPDATE_CHECK=strict  # exit 1 if stale
CYBEROS_UPDATE_CHECK=0       # off
CYBEROS_PAYLOAD=/path/to/dist/cyberos
CYBEROS_OFFLINE=1
```

## Apply latest

```bash
curl -fsSL https://github.com/cyberskill-official/cyberos/releases/latest/download/cyberos-payload.tar.gz \
  | tar -xz -C /tmp
bash /tmp/cyberos/install.sh /path/to/repo

# or from monorepo
bash ~/Projects/CyberSkill/cyberos/dist/cyberos/install.sh /path/to/repo

# or from installed tree when payload is current
bash .cyberos/update.sh --apply
```

## Agent surface

| File | Role |
|------|------|
| Root `AGENTS.md` | Thin pointer → `.cyberos/AGENT-ENTRY.md` (same idea as `CLAUDE.md` / `GEMINI.md`) |
| `.cyberos/AGENT-ENTRY.md` | Full agent one-pager |
| `.cyberos/memory/AGENTS.md` | Layer-1 memory protocol (dense; not at repo root) |
| `CLAUDE.md` / `GEMINI.md` / … | Per-agent pointers |

## Versions

| File | Meaning |
|------|---------|
| `.cyberos/VERSION` | **Platform** (CyberOS) |
| `package.json` version | **Product** (independent) |
