# Consumer install / update (CyberOS 1.0)

## Commands (final)

| Shell | Slash | Role |
|-------|-------|------|
| `bash install.sh [repo]` | `/install` | Install or re-vendor the machine |
| `bash uninstall.sh [repo]` | `/uninstall` | Remove the machine (keeps FRs; BRAIN kept by default) |
| `bash version.sh [repo]` | `/version` | Check for a newer CyberOS; if stale, ask → `install` |
| `bash status.sh [repo]` | `/status` | Open `docs/status/index.html` in the default browser |
| `bash help.sh` | `/help` | CLI surface |

**Day-to-day:** install once, then forget. Soft update-check runs on any `.cyberos` use.
**Re-vendor path:** only `install` (version never has a second apply path).

## Soft update triggers (automatic)

`run-gates`, status-page hooks, `help`, `version`, `status`, MCP tools, full `install`.

```bash
CYBEROS_UPDATE_CHECK=soft|always|strict|0
CYBEROS_PAYLOAD=/path/to/dist/cyberos
CYBEROS_OFFLINE=1
CYBEROS_NONINTERACTIVE=1   # version.sh: report only, no y/N prompt
```

## Fresh install / update

```bash
# from release
curl -fsSL https://github.com/cyberskill-official/cyberos/releases/latest/download/cyberos-payload.tar.gz \
  | tar -xz -C /tmp
bash /tmp/cyberos/install.sh /path/to/repo

# from monorepo
bash ~/Projects/CyberSkill/cyberos/dist/cyberos/install.sh /path/to/repo

# later: check then accept install
bash .cyberos/version.sh
```

## Agent surface

| File | Role |
|------|------|
| Root `AGENTS.md` | Thin pointer → `.cyberos/AGENT-ENTRY.md` |
| `.cyberos/AGENT-ENTRY.md` | Full agent one-pager |
| `.cyberos/memory/AGENTS.md` | Layer-1 memory protocol |
| `CLAUDE.md` / `GEMINI.md` / … | Per-agent pointers |

## Versions

| File | Meaning |
|------|---------|
| `.cyberos/VERSION` | Platform (CyberOS) |
| Product `package.json` / `VERSION` | Your product (independent) |
