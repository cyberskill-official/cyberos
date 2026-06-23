# GAM — Codebase Architecture

> **Git Alias Manager** — Tauri v2 desktop app (React 19 + Rust) for managing Git aliases.

## Project Structure

```
gam/
├── src/                    # React frontend
│   ├── App.tsx             # Root component — wires hooks + layout
│   ├── main.tsx            # Entry point — renders <App />
│   ├── index.css           # Tailwind entry
│   ├── components/         # UI components (15 files)
│   ├── hooks/              # Custom React hooks (10 files)
│   ├── services/           # Frontend data services (4 files)
│   ├── lib/                # Constants, Tauri detection, platform utils
│   ├── styles/             # Theme CSS files (15 files)
│   ├── tauri-bridge.ts     # IPC → Tauri invoke bridge
│   └── types/              # TypeScript interfaces
├── src-tauri/              # Rust backend
│   └── src/
│       ├── main.rs         # Entry point
│       ├── lib.rs          # Tauri builder, AppState, plugin setup
│       ├── commands.rs     # 21 IPC commands (Tauri #[command])
│       ├── git_service.rs  # Git alias CRUD via subprocess
│       ├── file_service.rs # JSON import/export (aliases + groups)
│       ├── group_service.rs     # Alias group CRUD
│       ├── ranking_service.rs   # Shell history scoring
│       ├── settings_service.rs  # App settings persistence
│       └── known_repos_service.rs # Tracked repo paths
├── tests/                  # Vitest test suites (225 frontend tests); 66 Rust unit tests live inline in src-tauri/src/
├── docs/                   # User-facing documentation
└── .github/workflows/      # CI — check.yml (3-platform matrix) + release.yml
```

## Architecture

### Data Flow

```
User Action → React Component → Hook → tauri-bridge.ts → IPC → Rust Command → Service → git CLI / filesystem
```

### IPC Commands (commands.rs)

| Command                                             | Service           | Description                              |
| --------------------------------------------------- | ----------------- | ---------------------------------------- |
| `get_aliases`                                       | `GitService`      | List aliases by scope (global/local/all) |
| `add_alias`                                         | `GitService`      | Create new alias                         |
| `update_alias`                                      | `GitService`      | Rename or modify alias                   |
| `delete_alias`                                      | `GitService`      | Remove alias                             |
| `validate_command`                                  | `GitService`      | Check for dangerous patterns             |
| `export_aliases`                                    | `FileService`     | Save to JSON file (includes groups)      |
| `import_aliases`                                    | `FileService`     | Load from JSON file (auto-merges groups) |
| `select_folder`                                     | Dialog            | Native folder picker                     |
| `get_local_path` / `set_local_path`                 | `GitService`      | Current repo scope                       |
| `open_local_folder` / `open_external`               | `open` crate      | Open in OS file manager                  |
| `get_theme` / `set_theme`                           | `SettingsService` | Theme persistence                        |
| `get_groups` / `create_group`                       | `GroupService`    | Group CRUD                               |
| `rename_group` / `set_group_color` / `delete_group` | `GroupService`    | Group mutation (rename, color, delete)   |
| `set_alias_groups` / `get_all_group_assignments`    | `GroupService`    | Alias ↔ group mapping                    |

### Rust Services

- **`GitService`** — Core alias CRUD via `git config` subprocess. Holds `local_path`, `KnownReposService`, `RankingService`.
- **`RankingService`** — Parses `~/.zsh_history`, `~/.bash_history` (plain + timestamped), Fish history, PowerShell history. Scores aliases: `TimeMultiplier × Length^(3/5) × Frequency`.
- **`FileService`** — JSON export/import with schema validation. Supports optional group data.
- **`GroupService`** — CRUD for alias groups stored in `groups.json`. Supports import/merge.
- **`SettingsService`** — Key-value settings in `settings.json`.
- **`KnownReposService`** — Tracks visited repo paths in `known-repos.json`.

### Frontend Hooks

| Hook              | Purpose                                                      |
| ----------------- | ------------------------------------------------------------ |
| `useAliases`      | Alias CRUD + scope state (mock API fallback for browser dev) |
| `useAliasActions` | Save/delete/import/export orchestration with toast feedback  |
| `useGroups`       | Group CRUD, active group filter, alias↔group assignments     |
| `useSearch`       | Debounced search filter across name + command                |
| `useLocalPath`    | Folder selection and clear logic                             |
| `useDragDrop`     | Drag-and-drop folder import                                  |
| `useTheme`        | Theme switching with preview/commit/persist cycle            |
| `useToast`        | Toast notification queue with auto-dismiss                   |
| `useUpdater`      | Auto-update check on launch                                  |

### Theme System

10 visual styles × 2 modes (light/dark) = **20 themes**. Applied via `data-style` and `data-mode` attributes on `<html>`. CSS variables per theme in `src/styles/`.

### App Data Paths

All persistent data via `dirs::data_dir()`:

- macOS: `~/Library/Application Support/com.github.zintaen.gam/`
- Linux: `~/.local/share/com.github.zintaen.gam/`
- Windows: `%APPDATA%/com.github.zintaen.gam/`

Files: `settings.json`, `known-repos.json`, `groups.json`

Crash log: `com.github.zintaen.gam/crash.log` (in the same `dirs::data_dir()` directory as the files above)

## File Dependencies

| If you change...   | Also check...                                      |
| ------------------ | -------------------------------------------------- |
| `types/index.ts`   | `tauri-bridge.ts`, all hooks, all components       |
| `git_service.rs`   | `commands.rs`, `file_service.rs`                   |
| `group_service.rs` | `commands.rs`, `lib.rs`, `file_service.rs`         |
| `lib/constants.ts` | Components using themes, search debounce           |
| `commands.rs`      | `lib.rs` (handler registration), `tauri-bridge.ts` |
| `tauri.conf.json`  | CSP policy, window config, updater endpoints       |
