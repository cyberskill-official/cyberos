---
id: FR-GAM-002
title: "Alias grouping with colors"
module: GAM
priority: SHOULD
status: done
fidelity: as-built
shipped: 2026-06-23
owner: Stephen Cheng
source_repo: zintaen/gam @ f55d97c
related_frs: [FR-GAM-001, FR-GAM-004]
---

## §1 — Description (BCP-14 normative)

gam SHOULD let users organize aliases into named groups for readability.

1. The app SHOULD let users create named groups and assign aliases to them.
2. The app SHOULD let users assign one color per group.
3. Group membership and group color MUST persist across sessions (see FR-GAM-004).
4. Ungrouped aliases MUST remain fully usable; grouping is organizational only and MUST NOT change how an alias resolves in `git`.

## §2 — Why this design

Power users accumulate dozens of aliases. Flat lists stop scaling, so grouping with color is a low-cost way to keep the list scannable. Grouping is presentation only: it never rewrites the underlying `git config`, so removing a group can never break an alias. Persisting group state in app settings (not in `git config`) keeps the user's real configuration clean and portable.

## §3 — Implementation

- Group state management in the React layer, with a `set_group_color` IPC command on the Rust side.
- Group membership and colors are written to the app settings store (FR-GAM-004), not to `git config`.
- The sidebar renders groups with their assigned colors; ungrouped aliases render in the main list.

## §4 — Acceptance criteria

1. Create a group, assign aliases to it, restart the app → the group and its members persist.
2. Assign a color to a group → the color persists across restart.
3. An ungrouped alias still resolves and runs through `git` normally.
4. Deleting a group does not delete or break its member aliases.

## §5 — Verification

Group state is persisted through the settings store, so its durability is covered by the FR-GAM-004 settings round-trip tests (`src-tauri/src/settings_service.rs`). The sidebar grouping and color assignment were confirmed by live GUI inspection during the 2026-06-23 hardening session (groups rendered with colors; ungrouped aliases usable).

## §6 — Failure modes

| Failure | Detection | Outcome |
|---|---|---|
| Corrupt/partial group data in settings | settings loader fallback (FR-GAM-004) | falls back to no groups; aliases still listed |
| Group color missing | default color | group still renders |
| Group deleted | membership cleared | member aliases remain, become ungrouped |

## §7 — Notes

Grouping deliberately lives outside `git config` so the user's real Git configuration stays a clean alias-only surface.

*End of FR-GAM-002. Fidelity: as-built (10/10 target).*
