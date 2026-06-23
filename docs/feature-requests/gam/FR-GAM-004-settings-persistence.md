---
id: FR-GAM-004
title: "Settings persistence"
module: GAM
priority: MUST
status: done
fidelity: as-built
shipped: 2026-06-23
owner: Stephen Cheng
source_repo: zintaen/gam @ f55d97c
related_frs: [FR-GAM-002, FR-GAM-003, FR-GAM-005]
---

## §1 — Description (BCP-14 normative)

gam MUST persist user settings durably and reload them at startup.

1. The app MUST persist user settings (at least theme and usage-ranking consent) to the platform app-config directory.
2. The app MUST reload settings at startup and apply them before the user interacts.
3. A missing settings file MUST fall back to defaults without error (first run is not an error).
4. A partial or unparseable settings file MUST fall back to defaults for the missing or bad keys, without crashing.
5. Writing a setting MUST overwrite the prior value for that key, not append.

## §2 — Why this design

A desktop tool that forgets your theme and consent choice on every launch is annoying and, for the consent setting, a real problem. Storing settings in the platform app-config directory (rather than next to the binary or in `git config`) keeps them per-user and out of the way. Defaulting cleanly on missing or partial files means first run and forward-compatible files both work without special-casing.

## §3 — Implementation

- `src-tauri/src/settings_service.rs` — load/save of the settings file with key-level get/set and overwrite semantics.
- `src-tauri/src/lib.rs` — loads settings at startup (including the usage-ranking consent for FR-GAM-003).
- Settings file location: the platform app-config directory. On macOS this is `~/Library/Application Support/<bundle-id>/settings.json`.
- Keys today: `gam-theme` (FR-GAM-005) and `historyRankingEnabled` (FR-GAM-003); group state (FR-GAM-002) persists through the same store.

## §4 — Acceptance criteria

1. Set a value, restart → the value is reloaded.
2. First run with no settings file → defaults apply, no error.
3. A partial settings file → present keys load, missing keys default.
4. Setting an existing key overwrites it rather than duplicating.

## §5 — Verification

`src-tauri/src/settings_service.rs` ships round-trip tests:

- `set_and_get_roundtrip`
- `set_overwrites_existing`

Plus the live on-disk confirmation from FR-GAM-003: the settings file was created on first write and reflected each toggle.

## §6 — Failure modes

| Failure | Detection | Outcome |
|---|---|---|
| No settings file (first run) | open returns not-found | defaults; no error |
| Partial/garbled file | parse fallback | defaults for missing/bad keys |
| App-config dir unwritable | write error | surfaced; in-memory state still works for the session |

## §7 — Notes

The settings file is the durable substrate for FR-GAM-002 (groups), FR-GAM-003 (consent), and FR-GAM-005 (theme).

*End of FR-GAM-004. Fidelity: as-built (10/10 target).*
