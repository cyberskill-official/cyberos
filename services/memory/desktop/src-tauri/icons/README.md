# memory app icons

This directory ships **empty** in the scaffold. Tauri 2 expects a fixed set of icon files for bundling.

Generate them with:

```bash
# From a 1024x1024 PNG master:
cd apps/memory
pnpm tauri icon path/to/icon-1024.png
```

That command writes the following into this folder:

```
icons/32x32.png
icons/128x128.png
icons/128x128@2x.png
icons/icon.icns        # macOS
icons/icon.ico         # Windows
icons/icon.png         # tray + Linux PNG
```

Commit the generated icons (they're small, deterministic, and required by `tauri.conf.json` → `bundle.icon` + `app.trayIcon.iconPath`).

Until icons are committed, `pnpm tauri dev` will warn about missing icons but still launch with Tauri's default placeholder.
