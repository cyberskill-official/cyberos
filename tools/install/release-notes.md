# CyberOS v{{VERSION}}

**First stable consumer + desktop release.** Prefer the assets below by role — filenames alone are easy to misread.

## Which asset do I want?

| Asset | Download this if you want… |
|-------|----------------------------|
| **`cyberos-payload.tar.gz`** | **Default for any code repo** — portable CyberOS machine. Unpack, then `bash install.sh /path/to/repo`. Stable name always points at this release’s payload. |
| **`cyberos-payload-{{VERSION}}.tar.gz`** | Same payload, **version-pinned** name (scripts, air-gapped mirrors, audit trails). |
| **`cyberos.plugin`** | **Claude Code / marketplace plugin** (stable name). Install via Plugins → add marketplace, or copy into your plugin path. |
| **`cyberos-{{VERSION}}.plugin`** | Same plugin zip, version-pinned. |
| **`SHA256SUMS`** | Checksums for the payload + plugin assets above. Verify before install. |
| **`CyberOS_{{VERSION}}_universal.dmg`** | **macOS desktop app** (universal). |
| **`CyberOS_universal.app.tar.gz`** | macOS app bundle (tar) when you do not want the DMG. |
| **`CyberOS_{{VERSION}}_amd64.AppImage`** | **Linux desktop** (AppImage). |
| **`CyberOS_{{VERSION}}_amd64.deb`** | Debian/Ubuntu package. |
| **`CyberOS-{{VERSION}}-1.x86_64.rpm`** | Fedora/RHEL-style package. |
| **`CyberOS_{{VERSION}}_x64-setup.exe`** / **`.msi`** | **Windows desktop** installers. |
| **`*.sig`** | Signature for the matching installer/package (verify the file with the same basename). |
| **Source code (zip / tar.gz)** | Full monorepo checkout — **not** the consumer payload. Use payload tarball for `install.sh`. |

### Quick install (consumer repo)

```bash
curl -fsSL https://github.com/cyberskill-official/cyberos/releases/latest/download/cyberos-payload.tar.gz \
  | tar -xz -C /tmp
bash /tmp/cyberos/install.sh /path/to/your/repo
```

Day-to-day after install: soft update-check runs on any `.cyberos` use. Manual check: `bash .cyberos/version.sh` (or `/version`). Open the status page: `bash .cyberos/status.sh` (or `/status`). Re-vendor is always `install`.

---

## Changelog ({{VERSION}})

{{CHANGELOG_SECTION}}

Full history: [CHANGELOG.md](https://github.com/cyberskill-official/cyberos/blob/main/CHANGELOG.md) · Status hub releases lens is generated from the same file.
