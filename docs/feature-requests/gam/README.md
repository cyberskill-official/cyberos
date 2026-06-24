# GAM module — feature request index

gam (Git Alias Manager) is a Tauri v2 desktop app absorbed from zintaen/gam. Unlike the from-scratch module catalogs, these entries are written at as-built fidelity: every FR is already shipped, so each records what the app does today and points at the tests that prove it, rather than specifying work to be done. Each FR now has a per-file spec and a paired audit beside this index (FR-GAM-NNN-<slug>.md and .audit.md), authored 2026-06-24.

Normative keywords follow BCP-14 (MUST, SHOULD, MAY).

## FRs

| FR | Priority | Status | Title |
|---|---|---|---|
| FR-GAM-001 | MUST | done | Alias CRUD across local and global scope |
| FR-GAM-002 | MUST | done | Alias grouping with colors |
| FR-GAM-003 | MUST | done | Consent-gated usage ranking from shell history |
| FR-GAM-004 | MUST | done | Settings persistence |
| FR-GAM-005 | SHOULD | done | Theming with light/dark modes and live preview |
| FR-GAM-006 | SHOULD | done | Import and export of aliases |
| FR-GAM-007 | SHOULD | done | Alias suggestions and command library |
| FR-GAM-008 | MUST | done | Signed auto-update |

## FR-GAM-001 — Alias CRUD across local and global scope

The app MUST create, read, update, and delete Git aliases at both `--local` and `--global` scope through `git config`. It MUST validate alias names (reject names that are not `^[a-zA-Z][\w-]*$`), MUST reject duplicates, and MUST preserve multi-word command values verbatim. Local-scope operations MUST NOT touch the user's global gitconfig.

Verification: src-tauri/src/git_service.rs integration tests (add/get/update/delete against a throwaway temp repo, name validation, duplicate rejection, multiword round-trip).

## FR-GAM-002 — Alias grouping with colors

The app SHOULD let users organize aliases into named groups and assign a color per group, persisted across sessions. Ungrouped aliases MUST remain usable.

Verification: group state hooks and their unit tests under tests/; settings persistence (FR-GAM-004).

## FR-GAM-003 — Consent-gated usage ranking from shell history

The app MAY rank aliases by frequency of use read from local shell history (zsh, bash, fish, PowerShell). This is default-on but MUST be disclosed and MUST be reversible: turning it off MUST stop reading history and MUST clear any cached ranking. Nothing read from history leaves the machine.

Verification: src-tauri/src/ranking_service.rs (enabled gate, cache clear on opt-out), tests/hooks/useHistoryRanking.test.ts, tests/components/PrivacyPanel.test.tsx.

## FR-GAM-004 — Settings persistence

The app MUST persist user settings (theme, usage-ranking consent) to the platform app-config directory and MUST reload them at startup. A missing or partial settings file MUST fall back to defaults without error.

Verification: src-tauri/src/settings_service.rs round-trip tests; startup load path in src-tauri/src/lib.rs.

## FR-GAM-005 — Theming with light/dark modes and live preview

The app SHOULD offer multiple theme styles with light and dark modes, MUST apply the choice live (preview before commit), and MUST persist the committed theme. An invalid stored theme MUST fall back to the default.

Verification: tests/hooks/useTheme.test.ts (default, stored value, invalid fallback, preview vs commit, data-attribute application).

## FR-GAM-006 — Import and export of aliases

The app SHOULD export the current alias set and import an alias set, so users can move aliases between machines. Import MUST apply the same name validation and duplicate rules as FR-GAM-001.

Verification: data panel component and its tests under tests/; shares the git_service write path.

## FR-GAM-007 — Alias suggestions and command library

The app SHOULD suggest alias names and commands from a built-in library to speed up creation. Selecting a suggestion MUST pre-fill the form while leaving every field editable before save.

Verification: tests/services/suggestion-service.test.ts; tests/components/AliasForm.test.tsx (library selection pre-fills, fields stay editable).

## FR-GAM-008 — Signed auto-update

The app MUST verify updates against a pinned minisign public key before applying them, so only updates signed with the project's private key are accepted. The public key lives in src-tauri/tauri.conf.json; it was rotated on 2026-06-23.

Verification: tauri-plugin-updater configuration in src-tauri/tauri.conf.json (pubkey + endpoints); release signing wired in the upstream release workflow.

## Cross-module dependencies

None today. gam is a self-contained desktop app and does not depend on, nor is it depended on by, other CyberOS modules. If it later integrates with CyberOS services (for example shared auth or memory), record those links here.
