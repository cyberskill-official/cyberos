# CyberOS v{{VERSION}}

Prefer the assets below by role — filenames alone are easy to misread. Every asset name is a direct download link.

## Which asset do I want?

| Asset | Download this if you want… |
|-------|----------------------------|
| **[`cyberos-payload.tar.gz`](https://github.com/cyberskill-official/cyberos/releases/download/v{{VERSION}}/cyberos-payload.tar.gz)** | **Default for any code repo** — portable CyberOS machine. Unpack, then `bash install.sh /path/to/repo`. Stable name always points at this release’s payload. |
| **[`cyberos-payload-{{VERSION}}.tar.gz`](https://github.com/cyberskill-official/cyberos/releases/download/v{{VERSION}}/cyberos-payload-{{VERSION}}.tar.gz)** | Same payload, **version-pinned** name (scripts, air-gapped mirrors, audit trails). |
| **[`cyberos.plugin`](https://github.com/cyberskill-official/cyberos/releases/download/v{{VERSION}}/cyberos.plugin)** | **Claude Code / marketplace plugin** (stable name). Install via Plugins → add marketplace, or copy into your plugin path. |
| **[`cyberos-{{VERSION}}.plugin`](https://github.com/cyberskill-official/cyberos/releases/download/v{{VERSION}}/cyberos-{{VERSION}}.plugin)** | Same plugin zip, version-pinned. |
| **[`SHA256SUMS`](https://github.com/cyberskill-official/cyberos/releases/download/v{{VERSION}}/SHA256SUMS)** | Checksums for the payload + plugin assets above. Verify before install. |
| **[`CyberOS_{{VERSION}}_universal.dmg`](https://github.com/cyberskill-official/cyberos/releases/download/v{{VERSION}}/CyberOS_{{VERSION}}_universal.dmg)** | **macOS desktop app** (universal). |
| **[`CyberOS_universal.app.tar.gz`](https://github.com/cyberskill-official/cyberos/releases/download/v{{VERSION}}/CyberOS_universal.app.tar.gz)** | macOS app bundle (tar) when you do not want the DMG. |
| **[`CyberOS_{{VERSION}}_amd64.AppImage`](https://github.com/cyberskill-official/cyberos/releases/download/v{{VERSION}}/CyberOS_{{VERSION}}_amd64.AppImage)** | **Linux desktop** (AppImage). |
| **[`CyberOS_{{VERSION}}_amd64.deb`](https://github.com/cyberskill-official/cyberos/releases/download/v{{VERSION}}/CyberOS_{{VERSION}}_amd64.deb)** | Debian/Ubuntu package. |
| **[`CyberOS-{{VERSION}}-1.x86_64.rpm`](https://github.com/cyberskill-official/cyberos/releases/download/v{{VERSION}}/CyberOS-{{VERSION}}-1.x86_64.rpm)** | Fedora/RHEL-style package. |
| **[`CyberOS_{{VERSION}}_x64-setup.exe`](https://github.com/cyberskill-official/cyberos/releases/download/v{{VERSION}}/CyberOS_{{VERSION}}_x64-setup.exe)** / **[`.msi`](https://github.com/cyberskill-official/cyberos/releases/download/v{{VERSION}}/CyberOS_{{VERSION}}_x64_en-US.msi)** | **Windows desktop** installers. |
| **`*.sig`** | Signature for the matching installer/package — each sits next to its file in the asset list below (verify the file with the same basename). |
| **[Source code (zip)](https://github.com/cyberskill-official/cyberos/archive/refs/tags/v{{VERSION}}.zip)** / **[tar.gz](https://github.com/cyberskill-official/cyberos/archive/refs/tags/v{{VERSION}}.tar.gz)** | Full monorepo checkout — **not** the consumer payload. Use the payload tarball for `install.sh`. |

## Install & docs

Install steps, day-to-day usage, and everything else live in the official docs — read them rather than a release-page summary:

- **[README](https://github.com/cyberskill-official/cyberos#readme)** — start here; routes to all documentation.
- **[Getting Started](https://os.cyberskill.world/docs/reference/getting-started.html)** — repo layout, quick start, versioning, install, deploy runbook.

---

## Changelog ({{VERSION}})

{{CHANGELOG_SECTION}}

Full history: [CHANGELOG.md](https://github.com/cyberskill-official/cyberos/blob/main/CHANGELOG.md) · Status hub releases lens is generated from the same file.
